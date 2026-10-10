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
        let active = initial
            .vehicles
            .iter()
            .filter(|v| v.phase == Phase::Flying)
            .count()
            + 1;
        if duration > 86400. || duration * 4. * active as f64 > 2_000_000. {
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
        if duration * 4. * active as f64 * sources as f64 > 20_000_000. {
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
            ("manifest.json",serde_json::to_vec_pretty(&json!({"format":"cosmogon-world-flight-setup","schema_version":1,"complete":true,"study":self,"initial_state_fingerprint":research::state_fingerprint(initial)?,"data_versions":save::data_versions(initial),"source_context":context(),"files":["definition.json","origin.cosmo"]})).map_err(|e|e.to_string())?)];
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
        for i in 0..study.samples {
            let target = initial.time + i as f64 * study.interval_s;
            if i > 0 {
                world.advance_to(target, None, None);
            }
            if world.time != target {
                return Err("World did not reach the requested flight observation time".into());
            }
            let flight = world
                .vehicles
                .iter()
                .find(|v| v.id == id)
                .ok_or("Studied craft disappeared from its actual branch")?
                .clone();
            rows.push(Observation {
                world_time_s: world.time,
                flight,
            });
        }
        world.validate_vehicle_state()?;
        let manifest = json!({"format":"cosmogon-world-vehicle-recording","schema_version":1,"complete":true,
            "study":study,"model":world_vehicle::MODEL,"propulsion_model":supply_vehicle::MODEL,"frame":world_vehicle::FRAME,
            "origin_fingerprint":research::state_fingerprint(initial)?,"final_fingerprint":research::state_fingerprint(&world)?,
            "origin_data_versions":save::data_versions(initial),"final_data_versions":save::data_versions(&world),
            "vehicle_id":id,"observations":rows.len(),"source_context":context(),
            "fingerprint_scope":"FNV-1a canonical full state identity; not cryptographic integrity",
            "files":["definition.json","origin.cosmo","observations.json","trajectories.csv","final.cosmo"],
            "summary":{"final_flight":rows.last().map(|r|&r.flight),"retained_cargo":study.flight.plan.definition.cargo,
                "mass_residual_kg":study.flight.plan.definition.initial_propellant_kg-rows.last().unwrap().flight.propulsion.propellant_kg-rows.last().unwrap().flight.propulsion.exhausted_propellant_kg}});
        Ok(Self {
            study,
            origin: initial.clone(),
            final_world: world,
            rows,
            manifest,
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
        let files = [
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
}
