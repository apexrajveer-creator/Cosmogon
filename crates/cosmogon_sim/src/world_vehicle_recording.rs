//! Actual finite spacecraft trajectories from a pinned complete starting world.
//! Creating a study branches that world; it never advances or supplies the live world.
use crate::{
    research,
    sandbox::Edit,
    save, supply_vehicle,
    world_vehicle::{self, Definition, Flight, Phase},
    Universe,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::Write, path::Path};

pub const FORMAT: &str = "cosmogon-world-vehicle-study";
const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Study {
    pub format: String,
    pub schema_version: u32,
    pub initial_state_fingerprint: String,
    pub flight: Definition,
    pub samples: usize,
    pub interval_s: f64,
    /// Authored atomic transfers applied at exact sample boundaries during the
    /// branch through the same engine edit the live world uses. Absent legacy
    /// studies stay byte-exact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transfers: Vec<AuthoredTransfer>,
}

/// One authored transfer request applied at an exact sample boundary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredTransfer {
    /// Sample index at which the transfer edit runs, after the world reaches
    /// that sample's time and before its observation is recorded.
    pub at_sample: usize,
    pub habitat_id: u64,
    pub requested: supply_vehicle::Cargo,
}
impl Study {
    pub fn new(
        initial: &Universe,
        flight: Definition,
        samples: usize,
        interval_s: f64,
    ) -> Result<Self, String> {
        let study = Self {
            format: FORMAT.into(),
            schema_version: 1,
            initial_state_fingerprint: research::state_fingerprint(initial)?,
            flight,
            samples,
            interval_s,
            transfers: vec![],
        };
        study.validate(initial)?;
        Ok(study)
    }
    pub fn validate(&self, initial: &Universe) -> Result<(), String> {
        if self.format != FORMAT || self.schema_version != 1 {
            return Err("Unsupported world flight study".into());
        }
        if self.initial_state_fingerprint != research::state_fingerprint(initial)? {
            return Err("Flight study starting world differs from its exact retained pin".into());
        }
        self.flight.validate(initial)?;
        if !(2..=4096).contains(&self.samples)
            || !self.interval_s.is_finite()
            || !(0.001..=86400.).contains(&self.interval_s)
        {
            return Err(
                "World flight studies require 2–4096 samples and a finite 0.001–86400 s interval"
                    .into(),
            );
        }
        let duration = (self.samples - 1) as f64 * self.interval_s;
        let work_per_second = initial.vehicles.iter().filter(|v| v.phase==Phase::Flying)
            .map(|v| if v.contact.is_some() {3.*crate::vehicle_contact_dynamics::MAX_STEPS_PER_SECOND as f64} else {4.}).sum::<f64>()
            + if self.flight.contact.is_some() {3.*crate::vehicle_contact_dynamics::MAX_STEPS_PER_SECOND as f64} else {4.};
        if duration > 86400. || duration * work_per_second > 2_000_000. {
            return Err("World flight study exceeds one day or two million nominal fleet substeps; shorten the span".into());
        }
        let mut previous = initial.time;
        for i in 1..self.samples {
            let target = initial.time + i as f64 * self.interval_s;
            if !target.is_finite() || target <= previous {
                return Err(
                    "Observation interval is below represented world-clock precision".into(),
                );
            }
            previous = target;
        }
        initial.validate_vehicle_state()?;
        for transfer in &self.transfers {
            if transfer.at_sample >= self.samples {
                return Err("Authored transfer sample is outside the study".into());
            }
            if !crate::vehicle_cargo_mass::finite_cargo(&transfer.requested)
                || transfer.requested.mass_kg() <= 0.
            {
                return Err(
                    "Authored transfers require finite positive supported cargo".into(),
                );
            }
            if !initial
                .habitats
                .iter()
                .any(|h| h.id == transfer.habitat_id && h.definition.body.system == self.flight.parent.system)
            {
                return Err("Authored transfer references a missing habitat".into());
            }
        }
        // Validate the actual ordinary create path, including task slots and capacity,
        // on a branch before any worker writes an export folder.
        let mut preview = initial.clone();
        preview.apply_edit(Edit::CreateVehicle {
            definition: Box::new(self.flight.clone()),
        })?;
        let sources = preview.systems[self.flight.parent.system as usize]
            .bodies
            .iter()
            .filter(|b| b.exists())
            .count()
            + 2;
        if duration * work_per_second * sources as f64 > 20_000_000. {
            return Err("World flight study exceeds twenty million nominal gravity-source visits; shorten the span or active fleet".into());
        }
        let candidate = preview
            .vehicles
            .last()
            .ok_or("Missing validated flight preview")?;
        let initial_row_bytes = serde_json::to_vec_pretty(candidate)
            .map_err(|e| e.to_string())?
            .len();
        // Immutable plan bytes are already included. Reserve numeric expansion,
        // gravity-sample values and terminal evidence before allocating all rows.
        let estimated_row_bytes = initial_row_bytes
            .saturating_add(2048)
            .saturating_add(sources.saturating_mul(128));
        if estimated_row_bytes.saturating_mul(self.samples) > MAX_BYTES as usize {
            return Err("Flight observation payload exceeds the bounded recording size; reduce samples or plan/source complexity".into());
        }
        Ok(())
    }
    pub fn export_setup(&self, initial: &Universe, path: &Path) -> Result<(), String> {
        self.validate(initial)?;
        let files=[("definition.json",serde_json::to_vec_pretty(self).map_err(|e|e.to_string())?),
            ("origin.cosmo",recorded_save(initial,"Flight setup origin")?),
            ("manifest.json",serde_json::to_vec_pretty(&json!({"format":"cosmogon-world-flight-setup","schema_version":1,"complete":true,"study":self,"initial_state_fingerprint":research::state_fingerprint(initial)?,"data_versions":save::data_versions(initial),"source_context":context_for(&self.flight),"files":["definition.json","origin.cosmo"]})).map_err(|e|e.to_string())?)];
        if files
            .iter()
            .any(|(_, bytes)| bytes.len() as u64 > MAX_BYTES)
        {
            return Err("Flight setup exceeds bounded file size".into());
        }
        if path.exists() {
            return Err("Existing flight setup is protected; choose a new folder".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| e.to_string())?;
        for (name, bytes) in files {
            write_new(&path.join(name), &bytes)?;
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let study: Self = serde_json::from_str(&crate::habitat_recording::read(path, 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if study.format != FORMAT || study.schema_version != 1 {
            return Err("Unsupported world flight study".into());
        }
        study.flight.validate_structure()?;
        Ok(study)
    }
    pub fn write(&self, initial: &Universe, path: &Path) -> Result<(), String> {
        self.validate(initial)?;
        write_new(
            path,
            &serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Actual requested world time. Craft state may be from the last fixed task or
    /// retained contact; never relabel it as propagated to this observation time.
    pub world_time_s: f64,
    pub flight: Flight,
}
#[derive(Clone, Debug)]
pub struct Bundle {
    pub study: Study,
    pub origin: Universe,
    pub final_world: Universe,
    pub rows: Vec<Observation>,
    pub manifest: Value,
    /// Free-flight recordings retain per-observation osculating evidence about
    /// the craft's parent body; contact recordings never do.
    pub orbital: Option<Vec<crate::vehicle_orbit::OrbitSample>>,
}
impl Bundle {
    pub fn run(initial: &Universe, study: Study) -> Result<Self, String> {
        study.validate(initial)?;
        let mut world = initial.clone();
        let id = world.next_vehicle_id;
        world.apply_edit(Edit::CreateVehicle {
            definition: Box::new(study.flight.clone()),
        })?;
        let mut rows = Vec::with_capacity(study.samples);
        let mut orbital_samples: Vec<crate::vehicle_orbit::OrbitSample> = Vec::new();
        for i in 0..study.samples {
            let target = initial.time + i as f64 * study.interval_s;
            if i > 0 {
                world.advance_to(target, None, None);
            }
            if world.time != target {
                return Err("World did not reach the requested flight observation time".into());
            }
            for transfer in study.transfers.iter().filter(|t| t.at_sample == i) {
                world.apply_edit(Edit::TransferVehicleCargoToHabitat {
                    system: world.vehicles.iter().find(|v| v.id == id).expect("studied craft").definition.parent.system,
                    vehicle_id: id,
                    habitat_id: transfer.habitat_id,
                    requested: transfer.requested.clone(),
                })?;
            }
            let flight = world
                .vehicles
                .iter()
                .find(|v| v.id == id)
                .ok_or("Studied craft disappeared from its actual branch")?
                .clone();
            if study.flight.contact.is_none() {
                orbital_samples.push(crate::vehicle_orbit::OrbitSample {
                    world_time_s: world.time,
                    elements: crate::vehicle_orbit::observe(&world, &flight)?,
                });
            }
            rows.push(Observation {
                world_time_s: world.time,
                flight,
            });
        }
        world.validate_vehicle_state()?;
        let mut manifest = json!({"format":"cosmogon-world-vehicle-recording","schema_version":1,"complete":true,
            "study":study,"model":study.flight.model,"propulsion_model":supply_vehicle::MODEL,"frame":world_vehicle::FRAME,
            "origin_fingerprint":research::state_fingerprint(initial)?,"final_fingerprint":research::state_fingerprint(&world)?,
            "origin_data_versions":save::data_versions(initial),"final_data_versions":save::data_versions(&world),
            "vehicle_id":id,"observations":rows.len(),"source_context":context_for(&study.flight),
            "fingerprint_scope":"FNV-1a canonical full state identity; not cryptographic integrity",
            "files":["definition.json","origin.cosmo","observations.json","trajectories.csv","final.cosmo"],
            "summary":{"final_flight":rows.last().map(|r|&r.flight),"retained_cargo":study.flight.plan.definition.cargo,
                "mass_residual_kg":study.flight.plan.definition.initial_propellant_kg-rows.last().unwrap().flight.propulsion.propellant_kg-rows.last().unwrap().flight.propulsion.exhausted_propellant_kg}});
        if study.flight.contact.is_some() {manifest["files"].as_array_mut().unwrap().push(json!("contact.csv"));}
        if study.flight.contact.as_ref().is_some_and(|h|h.shear.is_some()) {manifest["files"].as_array_mut().unwrap().push(json!("shear.csv"));}
        if let Some(operation)=study.flight.contact.as_ref().and_then(|h|h.operation.as_ref()) {
            manifest["files"].as_array_mut().unwrap().push(json!("operation.csv"));
            manifest["operation_window"]=match rows.last().and_then(|r|r.flight.operation_ledger.as_ref()) {
                Some(ledger)=>json!({"model":ledger.model,"required_window_s":operation.required_window_s,"current_window_s":ledger.current_window_s,"longest_window_s":ledger.longest_window_s,"last_sample_time_s":ledger.last_sample_time_s,"samples":ledger.samples,"supported_samples":ledger.supported_samples,"exclusion_samples":ledger.exclusion_samples,"last_exclusion":ledger.last_exclusion,"permission_scope":"current_window_s only; longest window is historical evidence and grants nothing"}),
                None=>json!(null),
            };
        }
        if study.flight.entry.is_some() {
            manifest["entry_evidence"]=match rows.last().and_then(|r|r.flight.entry_ledger.as_ref()) {
                Some(ledger)=>json!({"model":crate::vehicle_entry::MODEL,"above_interface":ledger.above_interface,"interface_elapsed_s":ledger.interface_elapsed_s,"peak_heat_flux_w_m2":ledger.peak_heat_flux_w_m2,"total_heat_j_m2":ledger.total_heat_j_m2,"substeps_observed":ledger.substeps_observed}),
                None=>json!(null),
            };
        }
        if study.flight.landing.is_some() {
            manifest["landing_evidence"]=match rows.last().and_then(|r|r.flight.landing_ledger.as_ref()) {
                Some(ledger)=>json!({"model":crate::vehicle_landing::MODEL,"airborne":ledger.airborne,"touchdown_elapsed_s":ledger.touchdown_elapsed_s,"touchdown_vertical_speed_m_s":ledger.touchdown_vertical_speed_m_s,"touchdown_horizontal_speed_m_s":ledger.touchdown_horizontal_speed_m_s,"substeps_observed":ledger.substeps_observed}),
                None=>json!(null),
            };
        }
        if study.flight.cargo_ledgers {
            let last = rows.last().unwrap();
            let ledger = last.flight.cargo.as_ref().ok_or("Cargo-ledger flight lost its retained ledger")?;
            manifest["cargo"] = json!({"model":ledger.model,"held":ledger.held,
                "events":last.flight.cargo_events,"authored_removals":study.flight.cargo_removals,
                "scope":"Studies replay authored removals only; interactive removals are save-level evidence"});
        }
        if let Some(handling) = study.flight.handling.as_ref() {
            let handling_scope = if study.transfers.is_empty() {
                "Authored handling limits and operation-window evidence only; no transfer, receipt or delivery occurs"
            } else {
                "Authored handling limits consumed by the retained transfer records; rejected stock stays on the craft"
            };
            manifest["handling"] = json!({"model":handling.model,
                "per_species_limit_kg":handling.per_species_limit_kg,"total_limit_kg":handling.total_limit_kg,
                "oxygen_interface_limit_kg":handling.oxygen_interface_limit_kg,
                "power_w":handling.power_w,"thermal_rejection_w":handling.thermal_rejection_w,
                "required_operation_window_s":handling.required_operation_window_s,
                "reach_m":handling.reach_m,"mass_rate_kg_s":handling.mass_rate_kg_s,
                "current_window_s":rows.last().and_then(|r|r.flight.operation_ledger.as_ref()).map(|l|l.current_window_s),
                "scope":handling_scope});
        }
        if !study.transfers.is_empty() {
            let last = rows.last().unwrap();
            let applied = &last.flight.transfers;
            if applied.len() != study.transfers.len() {
                return Err("Applied transfers do not match the authored study".into());
            }
            manifest["transfers"] = json!({"model":crate::vehicle_transfer::MODEL,
                "authored":study.transfers,"applied":applied,
                "habitats":applied.iter().filter_map(|t|world.habitats.iter().find(|h|h.id==t.habitat_id).map(|h|json!({"id":h.id,"receipts":h.receipts}))).collect::<Vec<_>>(),
                "scope":crate::vehicle_transfer::SCOPE});
            manifest["files"].as_array_mut().unwrap().push(json!("transfers.csv"));
        }
        let orbital = (!orbital_samples.is_empty()).then_some(orbital_samples);
        if let Some(samples) = &orbital {
            manifest["files"]
                .as_array_mut()
                .unwrap()
                .push(json!("orbital.csv"));
            manifest["orbital"] =
                crate::vehicle_orbit::summary(study.flight.parent, samples)?;
        }
        Ok(Self {
            study,
            origin: initial.clone(),
            final_world: world,
            rows,
            manifest,
            orbital,
        })
    }
    pub fn export(&self, path: &Path) -> Result<(), String> {
        // Public fields remain reviewable but are not trusted at the output boundary.
        let replay = Self::run(&self.origin, self.study.clone())?;
        if !same_json(&self.rows, &replay.rows)?
            || self.manifest != replay.manifest
            || !same_json(&self.final_world, &replay.final_world)?
        {
            return Err("Flight bundle differs from its actual independent world replay".into());
        }
        let mut files = vec![
            (
                "definition.json",
                serde_json::to_vec_pretty(&self.study).map_err(|e| e.to_string())?,
            ),
            (
                "origin.cosmo",
                recorded_save(&self.origin, "Flight study origin")?,
            ),
            (
                "observations.json",
                serde_json::to_vec_pretty(&self.rows).map_err(|e| e.to_string())?,
            ),
            ("trajectories.csv", trajectory_csv(&self.rows).into_bytes()),
            (
                "final.cosmo",
                recorded_save(&self.final_world, "Flight study final world")?,
            ),
            (
                "manifest.json",
                serde_json::to_vec_pretty(&self.manifest).map_err(|e| e.to_string())?,
            ),
        ];
        if self.study.flight.contact.is_some() {
            let index=files.len()-1;
            files.insert(index,("contact.csv",contact_csv(&self.rows).into_bytes()));
        }
        if self.study.flight.contact.as_ref().is_some_and(|h|h.shear.is_some()) {
            let index=files.len()-1;files.insert(index,("shear.csv",shear_csv(&self.rows)?.into_bytes()));
        }
        if self.study.flight.contact.as_ref().is_some_and(|h|h.operation.is_some()) {
            let index=files.len()-1;files.insert(index,("operation.csv",operation_csv(&self.rows)?.into_bytes()));
        }
        if !self.study.transfers.is_empty() {
            let index=files.len()-1;files.insert(index,("transfers.csv",transfers_csv(&self.rows)?.into_bytes()));
        }
        if let Some(samples) = &self.orbital {
            let index=files.len()-1;
            files.insert(index,("orbital.csv",crate::vehicle_orbit::csv(samples).into_bytes()));
        }
        if files
            .iter()
            .any(|(_, bytes)| bytes.len() as u64 > MAX_BYTES)
        {
            return Err("Flight export exceeds the bounded 32 MiB per-file size".into());
        }
        if path.exists() {
            return Err("Existing flight recordings are protected; choose a new folder".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| e.to_string())?;
        for (name, bytes) in files {
            write_new(&path.join(name), &bytes)?;
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let manifest: Value = serde_json::from_str(&crate::habitat_recording::read(
            &path.join("manifest.json"),
            1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        if manifest["format"] != "cosmogon-world-vehicle-recording"
            || manifest["schema_version"] != 1
            || manifest["complete"] != true
        {
            return Err("World flight recording is incomplete or unsupported".into());
        }
        let study = Study::read(&path.join("definition.json"))?;
        let origin = save::from_json(&crate::habitat_recording::read(
            &path.join("origin.cosmo"),
            MAX_BYTES,
        )?)
        .map_err(|e| e.to_string())?;
        let replay = Self::run(&origin, study.clone())?;
        if manifest != replay.manifest {
            return Err("Flight origin, definition, source pins or full result differs".into());
        }
        let rows: Vec<Observation> = serde_json::from_str(&crate::habitat_recording::read(
            &path.join("observations.json"),
            MAX_BYTES,
        )?)
        .map_err(|e| e.to_string())?;
        if !same_json(&rows, &replay.rows)? {
            return Err(
                "Stored flight observations differ from an exact independent world replay".into(),
            );
        }
        if crate::habitat_recording::read(&path.join("trajectories.csv"), MAX_BYTES)?
            != trajectory_csv(&replay.rows)
        {
            return Err("Stored flight CSV differs from its exact numerical observations".into());
        }
        if study.flight.contact.is_some() && crate::habitat_recording::read(&path.join("contact.csv"),MAX_BYTES)?!=contact_csv(&replay.rows) {
            return Err("Stored contact CSV differs from exact actual foot/work observations".into());
        }
        if study.flight.contact.as_ref().is_some_and(|h|h.shear.is_some()) && crate::habitat_recording::read(&path.join("shear.csv"),MAX_BYTES)?!=shear_csv(&replay.rows)? {
            return Err("Stored shear CSV differs from exact persistent foot/work observations".into());
        }
        if study.flight.contact.as_ref().is_some_and(|h|h.operation.is_some()) && crate::habitat_recording::read(&path.join("operation.csv"),MAX_BYTES)?!=operation_csv(&replay.rows)? {
            return Err("Stored operation CSV differs from exact operation-window observations".into());
        }
        match &replay.orbital {
            Some(samples) => {
                if crate::habitat_recording::read(&path.join("orbital.csv"), MAX_BYTES)?
                    != crate::vehicle_orbit::csv(samples)
                {
                    return Err(
                        "Stored orbital CSV differs from exact retained osculating evidence".into(),
                    );
                }
            }
            None if path.join("orbital.csv").exists() => {
                return Err("Contact flight recording must not carry an orbital CSV".into());
            }
            None => {}
        }
        let final_world = save::from_json(&crate::habitat_recording::read(
            &path.join("final.cosmo"),
            MAX_BYTES,
        )?)
        .map_err(|e| e.to_string())?;
        if !same_json(&final_world, &replay.final_world)? {
            return Err("Stored final world differs from the actual flight branch".into());
        }
        Ok(Self {
            study,
            origin,
            final_world,
            rows,
            manifest,
            orbital: replay.orbital,
        })
    }
}
fn same_json<T: Serialize>(a: &T, b: &T) -> Result<bool, String> {
    Ok(serde_json::to_vec(a).map_err(|e| e.to_string())?
        == serde_json::to_vec(b).map_err(|e| e.to_string())?)
}
fn recorded_save(world: &Universe, name: &str) -> Result<Vec<u8>, String> {
    let mut value: Value =
        serde_json::from_str(&save::to_json(world, name).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    // Stable export bytes: wall-clock creation time is not scientific state. Ordinary
    // user saves keep their real saved_at_unix metadata; only these bundles use zero.
    value["header"]["saved_at_unix"] = json!(0);
    serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Flight evidence is protected: {e}"))?;
    file.write_all(bytes).map_err(|e| e.to_string())
}
pub fn read_world(path: &Path) -> Result<Universe, String> {
    save::from_json(&crate::habitat_recording::read(path, MAX_BYTES)?).map_err(|e| e.to_string())
}
pub fn phase_key(phase: &Phase) -> &'static str {
    match phase {
        Phase::Flying => "flying",
        Phase::Impact { .. } => "spherical_contact",
        Phase::Unsupported { .. } => "unsupported",
    }
}
pub fn trajectory_csv(rows: &[Observation]) -> String {
    let mut csv=String::from("observation_index,world_time_s,craft_time_s,elapsed_s,phase,system_index,parent_body_index,position_x_m,position_y_m,position_z_m,velocity_x_m_s,velocity_y_m_s,velocity_z_m_s,vehicle_mass_kg,held_propellant_kg,exhausted_propellant_kg,cumulative_impulse_n_s,cumulative_ideal_delta_v_m_s,applied_thrust_delta_v_x_m_s,applied_thrust_delta_v_y_m_s,applied_thrust_delta_v_z_m_s,powered_duration_s,fuel_limited_duration_s,cargo_oxygen_reserve_kg,cargo_potable_water_kg,cargo_food_dry_kg,cargo_food_water_kg,mass_residual_kg,last_substep_duration_s,last_substep_propellant_kg,last_substep_impulse_n_s,last_substep_ideal_delta_v_m_s,contact_radius_m,contact_position_residual_m,contact_tolerance_m,contact_source_sample_duration_s\n");
    for (index, row) in rows.iter().enumerate() {
        let f = &row.flight;
        let p = &f.propulsion;
        let d = &f.definition.plan.definition;
        let mut fields = vec![
            index.to_string(),
            row.world_time_s.to_string(),
            f.last_time_s.to_string(),
            p.elapsed_s.to_string(),
            phase_key(&f.phase).into(),
            f.definition.parent.system.to_string(),
            f.definition.parent.body.to_string(),
        ];
        fields.extend(
            [
                f.position_m.x,
                f.position_m.y,
                f.position_m.z,
                f.velocity_m_s.x,
                f.velocity_m_s.y,
                f.velocity_m_s.z,
                p.mass_kg(d),
                p.propellant_kg,
                p.exhausted_propellant_kg,
                p.cumulative_impulse_n_s,
                p.cumulative_ideal_delta_v_m_s,
                p.velocity_change_m_s[0],
                p.velocity_change_m_s[1],
                p.velocity_change_m_s[2],
                p.powered_duration_s,
                p.fuel_limited_duration_s,
                d.cargo.oxygen_reserve_kg,
                d.cargo.potable_water_kg,
                d.cargo.food_dry_kg,
                d.cargo.food_water_kg,
                d.initial_propellant_kg - p.propellant_kg - p.exhausted_propellant_kg,
            ]
            .map(|v| v.to_string()),
        );
        fields.extend(if let Some(i) = &f.last_interval {
            vec![
                i.duration_s.to_string(),
                i.actual_propellant_kg.to_string(),
                i.impulse_n_s.to_string(),
                i.ideal_delta_v_m_s.to_string(),
            ]
        } else {
            vec![String::new(); 4]
        });
        fields.extend(
            if let Phase::Impact {
                radius_m,
                position_residual_m,
                tolerance_m,
                source_sample_duration_s,
                ..
            } = &f.phase
            {
                vec![
                    radius_m.to_string(),
                    position_residual_m.to_string(),
                    tolerance_m.to_string(),
                    source_sample_duration_s.to_string(),
                ]
            } else {
                vec![String::new(); 4]
            },
        );
        csv.push_str(&fields.join(","));
        csv.push('\n');
    }
    csv
}
/// Per-foot actual numeric evidence with both world and retained craft clocks.
/// The ordinary v1 CSV remains byte-identical; only contact-capable bundles add this file.
pub fn contact_csv(rows:&[Observation])->String {
    let base="world_time_s,craft_time_s,foot_index,compression_m,compression_rate_m_s,normal_load_n,elastic_energy_j,normal_dissipation_w,sliding_dissipation_w,gap_m,slope_rad,force_x_n,force_y_n,force_z_n,torque_x_n_m,torque_y_n_m,torque_z_n_m,position_x_m,position_y_m,position_z_m,velocity_x_m_s,velocity_y_m_s,velocity_z_m_s,normal_x,normal_y,normal_z,orientation_x,orientation_y,orientation_z,orientation_w,body_omega_x_rad_s,body_omega_y_rad_s,body_omega_z_rad_s";
    fn numbers(value:&Value,prefix:&str,out:&mut Vec<(String,String)>) {
        if let Some(object)=value.as_object() {for (name,item) in object {numbers(item,&format!("{prefix}{name}_"),out);}}
        else if value.is_number() {out.push((prefix.trim_end_matches('_').into(),value.to_string()));}
    }
    let mut schema=vec![];let empty=crate::vehicle_contact_dynamics::Work::default();
    let first=rows.first().and_then(|r|r.flight.contact.as_ref()).map_or(&empty,|s|&s.cumulative_work);
    numbers(&serde_json::to_value(first).unwrap(),"cumulative_",&mut schema);
    let mut output=format!("{base},{}\n",schema.iter().map(|(name,_)|name.as_str()).collect::<Vec<_>>().join(","));
    for row in rows {if let Some(state)=&row.flight.contact {
        let mut work=vec![];numbers(&serde_json::to_value(&state.cumulative_work).unwrap(),"cumulative_",&mut work);
        for (index,foot) in state.feet.iter().enumerate() {
            let r=&foot.response;let n=foot.surface.outward_normal;let p=foot.position_m;let v=foot.velocity_m_s;let f=r.force_n;let t=foot.torque_about_centre_n_m;let q=state.orientation_xyzw;let w=state.angular_velocity_body_rad_s;
            let mut values=vec![row.world_time_s.to_string(),row.flight.last_time_s.to_string(),index.to_string()];
            values.extend([r.compression_m,r.compression_rate_m_s,r.normal_load_n,r.stored_elastic_energy_j,r.normal_dissipation_w,r.sliding_dissipation_w,foot.surface.tangent_gap_m,foot.surface.slope_rad,f.x,f.y,f.z,t.x,t.y,t.z,p.x,p.y,p.z,v.x,v.y,v.z,n.x,n.y,n.z,q[0],q[1],q[2],q[3],w.x,w.y,w.z].iter().map(ToString::to_string));
            values.extend(work.iter().map(|(_,value)|value.clone()));output.push_str(&values.join(","));output.push('\n');
        }
    }}
    output
}
pub fn shear_csv(rows: &[Observation]) -> Result<String, String> {
    let mut output=String::from("world_time_s,craft_time_s,foot_index,normal_load_n,shear_x_m,shear_y_m,shear_z_m,shear_force_x_n,shear_force_y_n,shear_force_z_n,shear_energy_j,cumulative_endpoint_relative_work_j,cumulative_shear_storage_change_j,cumulative_plastic_dissipation_j,cumulative_unloading_release_j,cumulative_numerical_dissipation_j,cumulative_shear_balance_residual_j,cumulative_plastic_slip_distance_m\n");
    for row in rows {
        let d=row.flight.definition.contact.as_ref().and_then(|h|h.shear.as_ref()).ok_or("Shear CSV requires explicit persistent hardware")?;
        let r=row.flight.contact.as_ref().ok_or("Missing rigid shear state")?;
        let work=r.cumulative_work.shear.as_ref().ok_or("Missing cumulative shear work")?;
        for (index,foot) in r.feet.iter().enumerate() {
            let s=foot.shear.as_ref().ok_or("Missing recorded foot shear")?;let force=d.force_n(s)?;
            let mut values=vec![row.world_time_s.to_string(),row.flight.last_time_s.to_string(),index.to_string()];
            values.extend([s.normal_load_n,s.elastic_shear_m.x,s.elastic_shear_m.y,s.elastic_shear_m.z,force.x,force.y,force.z,d.energy_j(s)?,work.endpoint_relative_work_j,work.elastic_storage_change_j,work.plastic_dissipation_j,work.unloading_release_j,work.numerical_dissipation_j,work.balance_residual_j,work.plastic_slip_distance_m].iter().map(ToString::to_string));
            output.push_str(&values.join(","));output.push('\n');
        }
    }
    Ok(output)
}
pub fn transfers_csv(rows: &[Observation]) -> Result<String, String> {
    let transfers = &rows.last().ok_or("Transfer CSV requires observations")?.flight.transfers;
    let mut output=String::from("index,world_time_s,habitat_id,requested_oxygen_kg,requested_potable_kg,requested_food_dry_kg,requested_food_water_kg,accepted_oxygen_kg,accepted_potable_kg,accepted_food_dry_kg,accepted_food_water_kg,rejected_oxygen_kg,rejected_potable_kg,rejected_food_dry_kg,rejected_food_water_kg,current_window_s,required_window_s,surface_separation_m,reach_m,rate_envelope_kg,ground_velocity_x_m_s,ground_velocity_y_m_s,ground_velocity_z_m_s,absorbed_momentum_kg_m_s,absorbed_kinetic_energy_j,craft_mass_before_kg,craft_mass_after_kg\n");
    for (index, t) in transfers.iter().enumerate() {
        let fields = vec![
            index.to_string(),
            t.world_time_s.to_string(),
            t.habitat_id.to_string(),
            t.requested.oxygen_reserve_kg.to_string(),
            t.requested.potable_water_kg.to_string(),
            t.requested.food_dry_kg.to_string(),
            t.requested.food_water_kg.to_string(),
            t.accepted.oxygen_reserve_kg.to_string(),
            t.accepted.potable_water_kg.to_string(),
            t.accepted.food_dry_kg.to_string(),
            t.accepted.food_water_kg.to_string(),
            t.rejected.oxygen_reserve_kg.to_string(),
            t.rejected.potable_water_kg.to_string(),
            t.rejected.food_dry_kg.to_string(),
            t.rejected.food_water_kg.to_string(),
            t.current_window_s.to_string(),
            t.required_operation_window_s.to_string(),
            t.surface_separation_m.to_string(),
            t.reach_m.to_string(),
            t.rate_envelope_kg.to_string(),
            t.ground_relative_velocity_m_s.x.to_string(),
            t.ground_relative_velocity_m_s.y.to_string(),
            t.ground_relative_velocity_m_s.z.to_string(),
            t.ground_absorbed_linear_momentum_kg_m_s.length().to_string(),
            t.ground_absorbed_kinetic_energy_j.to_string(),
            t.craft_mass_before_kg.to_string(),
            t.craft_mass_after_kg.to_string(),
        ];
        output.push_str(&fields.join(","));
        output.push('\n');
    }
    Ok(output)
}
pub fn operation_csv(rows: &[Observation]) -> Result<String, String> {
    let mut output=String::from("world_time_s,craft_time_s,current_window_s,longest_window_s,current_window_origin_s,longest_window_origin_s,samples,supported_samples,exclusion_samples,last_sample_supported,last_exclusion\n");
    for row in rows {
        row.flight.definition.contact.as_ref().and_then(|h|h.operation.as_ref()).ok_or("Operation CSV requires authored operation hardware")?;
        let empty=String::new();
        let values=match &row.flight.operation_ledger {
            Some(ledger)=>vec![row.world_time_s.to_string(),row.flight.last_time_s.to_string(),ledger.current_window_s.to_string(),ledger.longest_window_s.to_string(),ledger.current_window_origin_s.to_string(),ledger.longest_window_origin_s.to_string(),ledger.samples.to_string(),ledger.supported_samples.to_string(),ledger.exclusion_samples.to_string(),ledger.last_sample_supported.to_string(),ledger.last_exclusion.as_deref().unwrap_or(&empty).replace(',',";")],
            None=>vec![row.world_time_s.to_string(),row.flight.last_time_s.to_string(),empty.clone(),empty.clone(),empty.clone(),empty.clone(),empty.clone(),empty.clone(),empty.clone(),empty.clone(),empty.clone()],
        };
        output.push_str(&values.join(","));output.push('\n');
    }
    Ok(output)
}
pub fn context_for(definition: &Definition) -> Value {
    if definition.contact.is_none() { return context(); }
    let mut result=context();
    let contact=crate::vehicle_contact_dynamics::context_for(definition.contact.as_ref().unwrap());
    result["model"]=json!(definition.model);
    result["operator"]=contact["operator"].clone();
    result["contact"]=contact["response"].clone();
    result["limits"]=contact["limits"].clone();
    result["contact_capability"]=contact;
    if let Some(operation)=definition.contact.as_ref().and_then(|h|h.operation.as_ref()) {
        result["operation_window"]=crate::vehicle_operation::context();
        result["operation_required_window_s"]=json!(operation.required_window_s);
    }
    if let Some(limits)=&definition.launch {
        result["launch"]=crate::vehicle_launch::context();
        result["launch_limits"]=json!({"maximum_acceleration_m_s2":limits.maximum_acceleration_m_s2,"liftoff_altitude_m":limits.liftoff_altitude_m,"maximum_start_altitude_m":limits.maximum_start_altitude_m});
    }
    if let Some(entry)=&definition.entry {
        result["entry"]=crate::vehicle_entry::context();
        result["entry_limits"]=json!({"entry_interface_altitude_m":entry.entry_interface_altitude_m,"nose_radius_m":entry.nose_radius_m,"heat_flux_coefficient":entry.heat_flux_coefficient,"maximum_heat_flux_w_m2":entry.maximum_heat_flux_w_m2,"maximum_total_heat_j_m2":entry.maximum_total_heat_j_m2});
    }
    if let Some(limits)=&definition.landing {
        result["landing"]=crate::vehicle_landing::context();
        result["landing_limits"]=json!({"maximum_touchdown_vertical_speed_m_s":limits.maximum_touchdown_vertical_speed_m_s,"maximum_touchdown_horizontal_speed_m_s":limits.maximum_touchdown_horizontal_speed_m_s});
    }
    result["gravity"]=json!("Existing Newtonian test-particle sources; target point-mass field evaluated outside modeled terrain even where overview relief lies below the spherical base. Other-source hull intersections are unsupported, conservatively refused.");
    result
}
pub fn context() -> Value {
    json!({"model":world_vehicle::MODEL,"frame":world_vehicle::FRAME,
        "simulation_time_coordinate":"Existing world seconds since J2000; body translation and velocity included at creation",
        "operator":"One-second fixed world task; at most 0.25-second gravity-kick / exact finite-mass powered-drift / gravity-kick substeps, split at burn boundaries and fuel cutoff",
        "gravity":"Newtonian test particle; existing primary star, companion and present bodies in fixed vector order; point-mass fields outside spherical contact boundaries; existing analytic/N-body sampled source motion",
        "initial_conditions":"Explicit authored craft, propellant and cargo, added by the ordinary world edit; no settlement stock withdrawal or supply grant",
        "clock_support":"World observation time is separate from craft time; subsecond samples retain the last completed task. Contact or unsupported states retain their historical craft clock.",
        "last_interval_support":"Last actual substep only; never sum sparse last-substep samples into full-span transfers",
        "contact":"First spherical boundary in sampled source motion; source centers are linearly interpolated across each contact substep. Retain raw residual and spatial tolerance. Not terrain collision or safe landing.",
        "regime":{"max_system_local_position_m":1e15,"max_speed_m_s":1e6,"max_initial_parent_mass_ratio":1e-6},
        "propulsion":supply_vehicle::context(),
        "sources":[{"title":"NASA Glenn rocket weight and gravitational forces","url":"https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/rocket-weight/"},{"title":"JPL NAIF reference frames","url":"https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/frames.html"},{"title":"JPL Horizons manual","url":"https://ssd.jpl.nasa.gov/horizons/manual.html"}],
        "uncertainty":null,"empirical_validation":null,"limits":["No relativistic gravity, backreaction, drag/heating, attitude, staging or powered hardware constraints","No terrain-resolved landing, docking, cargo loading/transfer or measured spacecraft performance","Coordinate/source integration is the existing simulation; not a new JPL/SPICE precision claim","Source/model changes are actual world edits; a complete study branches the exact pinned origin","Gravity/frame edits and changed source membership/mass/radius stop at the last resolved state; event-time source discontinuity integration remains unsupported","Finite unsupported motion stops explicitly; high time warp must resolve active flight rather than skip it"],
        "work_limits":{"nominal_fleet_substeps":2000000,"nominal_gravity_source_visits":20000000,"recording_payload_bytes":MAX_BYTES},"export_saved_at_unix":0,"export_saved_at_policy":"Bundle wall-clock header metadata is zero for stable bytes; full scientific state/time/source identity is retained"})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Universe, Study) {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let d = Definition::circular_example(
            &u,
            crate::BodyRef { system: 0, body: 3 },
            supply_vehicle::Plan::example(),
        )
        .unwrap();
        let study = Study::new(&u, d, 9, 0.5).unwrap();
        (u, study)
    }
    #[test]
    fn shared_world_flight_bundle_reopens_exact_full_state_and_refuses_changed_evidence() {
        let (u, study) = setup();
        let origin = research::state_fingerprint(&u).unwrap();
        let bundle = Bundle::run(&u, study.clone()).unwrap();
        assert_eq!(bundle.rows[1].world_time_s, u.time + 0.5);
        assert_eq!(bundle.rows[1].flight.last_time_s, u.time);
        assert_eq!(bundle.rows[2].flight.last_time_s, u.time + 1.);
        assert_eq!(research::state_fingerprint(&u).unwrap(), origin);
        let root = std::env::temp_dir().join(format!(
            "cosmogon-world-flight-recording-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        bundle.export(&root).unwrap();
        let opened = Bundle::read(&root).unwrap();
        assert_eq!(opened.rows, bundle.rows);
        assert!(same_json(&opened.final_world, &bundle.final_world).unwrap());
        assert!(bundle.export(&root).is_err());
        let json_path = root.join("observations.json");
        let original = std::fs::read(&json_path).unwrap();
        let mut changed: Value = serde_json::from_slice(&original).unwrap();
        changed[2]["flight"]["position_m"]["x"] = json!(0);
        std::fs::write(&json_path, changed.to_string()).unwrap();
        assert!(Bundle::read(&root).is_err());
        std::fs::write(&json_path, &original).unwrap();
        let csv_path = root.join("trajectories.csv");
        let csv = std::fs::read(&csv_path).unwrap();
        std::fs::write(&csv_path, "changed").unwrap();
        assert!(Bundle::read(&root).is_err());
        std::fs::write(&csv_path, &csv).unwrap();
        let mut changed =
            serde_json::from_slice::<Value>(&std::fs::read(root.join("manifest.json")).unwrap())
                .unwrap();
        changed["source_context"]["gravity"] = json!("no gravity");
        std::fs::write(root.join("manifest.json"), changed.to_string()).unwrap();
        assert!(Bundle::read(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unsupported_launch_budget_or_changed_origin_is_refused_before_export() {
        let (mut u, mut study) = setup();
        study.samples = 4096;
        study.interval_s = 86400.;
        assert!(study.validate(&u).is_err());
        let (_, mut study) = setup();
        study.interval_s = 1e-10;
        assert!(study.validate(&u).is_err());
        let (_, mut heavy) = setup();
        heavy.samples = 4096;
        heavy.flight.plan.commands = vec![
            supply_vehicle::Command {
                duration_s: 0.,
                throttle: 0.,
                direction: [1., 0., 0.]
            };
            1024
        ];
        assert!(heavy.validate(&u).unwrap_err().contains("payload"));
        let (_, study) = setup();
        u.advance_by(1.);
        assert!(study.validate(&u).is_err());
    }

    #[test]
    fn transfer_studies_apply_the_shared_edit_and_replay_exactly() {
        let (mut world, mars) = crate::vehicle_transfer::tests::habitat_world(|d| {
            d.config.crew = 0;
        });
        world.advance_to(world.time + 30., None, None);
        let definition = crate::vehicle_transfer::tests::site_definition(
            &world,
            mars,
            crate::vehicle_handling::Equipment::example(),
        );
        let requested = supply_vehicle::Cargo {
            oxygen_reserve_kg: 10.,
            potable_water_kg: 20.,
            food_dry_kg: 5.,
            food_water_kg: 5.,
        };
        let mut study = Study::new(&world, definition, 15, 2.0).unwrap();
        study.transfers = vec![AuthoredTransfer {
            at_sample: 14,
            habitat_id: 0,
            requested: requested.clone(),
        }];
        study.validate(&world).unwrap();
        let bundle = Bundle::run(&world, study.clone()).unwrap();
        let transfer = &bundle.final_world.vehicles[0].transfers[0];
        assert_eq!(transfer.accepted, requested);
        assert_eq!(transfer.habitat_id, 0);
        assert_eq!(bundle.final_world.habitats[0].receipts.len(), 1);
        assert_eq!(bundle.final_world.habitats[0].receipts[0].accepted, requested);
        let manifest_transfers = &bundle.manifest["transfers"];
        assert_eq!(manifest_transfers["applied"].as_array().unwrap().len(), 1);
        assert!(bundle.manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "transfers.csv"));
        assert!(bundle.manifest["handling"]["scope"]
            .as_str()
            .unwrap()
            .contains("consumed by the retained transfer records"));
        // Independent re-run is bit-identical.
        let rerun = Bundle::run(&world, study.clone()).unwrap();
        assert_eq!(
            research::state_fingerprint(&bundle.final_world).unwrap(),
            research::state_fingerprint(&rerun.final_world).unwrap()
        );
        // Export and reopen.
        let root = std::env::temp_dir().join(format!(
            "cosmogon-world-transfer-recording-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        bundle.export(&root).unwrap();
        let opened = Bundle::read(&root).unwrap();
        assert_eq!(opened.final_world.vehicles[0].transfers, bundle.final_world.vehicles[0].transfers);
        assert!(root.join("transfers.csv").exists());
        std::fs::remove_dir_all(root).unwrap();
        // A study authored before transfers existed loads unchanged and moves nothing.
        let mut legacy_json = serde_json::to_value(&study).unwrap();
        legacy_json.as_object_mut().unwrap().remove("transfers");
        let legacy: Study =
            serde_json::from_value(legacy_json).expect("legacy studies load without transfers");
        assert!(legacy.transfers.is_empty());
        let legacy_bundle = Bundle::run(&world, legacy).unwrap();
        assert!(legacy_bundle.final_world.vehicles[0].transfers.is_empty());
        assert!(legacy_bundle.manifest.get("transfers").is_none());
        assert_eq!(legacy_bundle.final_world.habitats[0].receipts.len(), 0);
    }

    #[test]
    fn transfer_studies_refuse_missing_habitats_and_bad_samples() {
        let (mut world, mars) = crate::vehicle_transfer::tests::habitat_world(|d| {
            d.config.crew = 0;
        });
        world.advance_to(world.time + 30., None, None);
        let definition = crate::vehicle_transfer::tests::site_definition(
            &world,
            mars,
            crate::vehicle_handling::Equipment::example(),
        );
        let mut study = Study::new(&world, definition, 12, 1.0).unwrap();
        study.transfers = vec![AuthoredTransfer {
            at_sample: 99,
            habitat_id: 0,
            requested: supply_vehicle::Cargo {
                oxygen_reserve_kg: 1.,
                potable_water_kg: 0.,
                food_dry_kg: 0.,
                food_water_kg: 0.,
            },
        }];
        assert!(study.validate(&world).unwrap_err().contains("outside the study"));
        study.transfers[0].at_sample = 2;
        study.transfers[0].habitat_id = 17;
        assert!(study.validate(&world).unwrap_err().contains("missing habitat"));
        study.transfers[0].habitat_id = 0;
        study.transfers[0].requested.oxygen_reserve_kg = -1.;
        assert!(study.validate(&world).unwrap_err().contains("finite positive"));
        study.transfers[0].requested.oxygen_reserve_kg = 0.;
        assert!(study.validate(&world).unwrap_err().contains("finite positive"));
    }

    #[test]
    fn free_flight_studies_retain_and_replay_osculating_orbital_evidence() {
        let (u, study) = setup();
        assert!(study.flight.contact.is_none());
        let parent = study.flight.parent;
        let bundle = Bundle::run(&u, study).unwrap();
        let orbital = bundle.orbital.as_ref().unwrap();
        assert_eq!(orbital.len(), bundle.rows.len());
        for (sample, row) in orbital.iter().zip(&bundle.rows) {
            assert_eq!(sample.world_time_s, row.world_time_s);
            assert!(sample.elements.eccentricity.is_finite());
            assert!(sample.elements.semi_major_axis_m.is_finite());
            assert!(sample.elements.periapsis_m.is_finite());
            assert!(sample.elements.inclination_rad.is_finite());
        }
        assert_eq!(
            bundle.manifest["orbital"]["model"],
            crate::vehicle_orbit::MODEL
        );
        assert_eq!(
            bundle.manifest["orbital"]["observations"],
            bundle.rows.len()
        );
        assert!(bundle.manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "orbital.csv"));
        assert_eq!(
            bundle.manifest["orbital"],
            crate::vehicle_orbit::summary(parent, orbital).unwrap()
        );
        let root = std::env::temp_dir().join(format!(
            "cosmogon-orbital-recording-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        bundle.export(&root).unwrap();
        let opened = Bundle::read(&root).unwrap();
        assert_eq!(opened.orbital, bundle.orbital);
        assert_eq!(opened.manifest, bundle.manifest);
        let mut tampered = std::fs::read_to_string(root.join("orbital.csv")).unwrap();
        tampered = tampered.replacen("observation_index", "tampered_index", 1);
        std::fs::write(root.join("orbital.csv"), tampered).unwrap();
        assert!(Bundle::read(&root).unwrap_err().contains("orbital CSV"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn contact_studies_omit_orbital_evidence_and_refuse_a_stray_csv() {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = crate::BodyRef { system: 0, body: 3 };
        let frame = crate::vehicle_arrival::SurfaceFrame::at(&u, target, u.time).unwrap();
        let outward = crate::Vec3d::new(0.8, 0.6, 0.);
        let above = frame
            .to_inertial(&crate::vehicle_arrival::KinematicState {
                position_m: outward * (frame.radius_m + 10000.),
                velocity_m_s: crate::Vec3d::ZERO,
            })
            .unwrap();
        let hardware =
            crate::vehicle_contact_dynamics::Hardware::example(&u, target, above.position_m)
                .unwrap();
        let mut plan = supply_vehicle::Plan::example();
        plan.commands = vec![supply_vehicle::Command {
            duration_s: 30.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        let definition = crate::vehicle_arrival::GeographicStart::contact_example()
            .contact_flight(&u, target, plan, hardware)
            .unwrap();
        assert!(definition.contact.is_some());
        let study = Study::new(&u, definition, 5, 1.0).unwrap();
        let bundle = Bundle::run(&u, study).unwrap();
        assert!(bundle.orbital.is_none());
        assert!(bundle.manifest.get("orbital").is_none());
        assert!(!bundle.manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "orbital.csv"));
        let root = std::env::temp_dir().join(format!(
            "cosmogon-contact-no-orbital-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        bundle.export(&root).unwrap();
        Bundle::read(&root).unwrap();
        std::fs::write(
            root.join("orbital.csv"),
            "observation_index,world_time_s\n0,0\n",
        )
        .unwrap();
        assert!(Bundle::read(&root)
            .unwrap_err()
            .contains("must not carry an orbital CSV"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
