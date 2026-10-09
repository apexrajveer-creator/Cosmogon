//! Read-only, bounded habitat studies. Structural/source checks are not field validation
//! or a cryptographic guarantee that a trajectory was produced by this application.
use crate::{
    habitat::{self, Sample},
    research::state_fingerprint,
    save,
};
use serde::Serialize;
use serde_json::Value;
use std::{io::Read, path::Path};

pub(crate) fn read(path: &Path, limit: u64) -> Result<String, String> {
    let metadata=std::fs::metadata(path).map_err(|e|format!("{}: {e}",path.display()))?;
    if !metadata.is_file() {return Err("Study entries must be regular files".into());}
    if metadata.len()>limit {return Err(format!("{} exceeds the {} byte study limit",path.display(),limit));}
    let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Study entries must be regular files".into());
    }
    let mut bytes = Vec::new();
    f.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!(
            "{} exceeds the {} byte study limit",
            path.display(),
            limit
        ));
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn json(path: &Path, limit: u64) -> Result<Value, String> {
    serde_json::from_str(&read(path, limit)?).map_err(|e| e.to_string())
}
#[derive(Clone, Debug)]
pub struct Recording {
    pub rows: Vec<Sample>,
    pub summary: Value,
}
impl Recording {
    pub fn read(path: &Path) -> Result<Self, String> {
        let m = json(&path.join("manifest.json"), 64 * 1024)?;
        if m["format"] != "cosmogon-habitat-study"
            || m["schema_version"] != 1
            || m["model"] != habitat::MODEL
        {
            return Err("Unsupported habitat study format, schema or model".into());
        }
        if m["source"] != habitat::SOURCE {
            return Err("Habitat study reference source does not match this model".into());
        }
        let count = m["sample_count"]
            .as_u64()
            .filter(|n| (1..=1024).contains(n))
            .ok_or("Habitat study requires 1–1024 samples")?;
        let id = m["habitat_id"].as_u64().ok_or("Missing habitat identity")?;
        let rows: Vec<Sample> =
            serde_json::from_str(&read(&path.join("samples.json"), 16 * 1024 * 1024)?)
                .map_err(|e| e.to_string())?;
        if rows.len() != count as usize {
            return Err("Habitat sample count disagrees with manifest".into());
        }
        let initial = save::from_json(&read(&path.join("initial.cosmo"), 32 * 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        let final_world = save::from_json(&read(&path.join("final.cosmo"), 32 * 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        let first = habitat::sample(&initial, id)?;
        let last = habitat::sample(&final_world, id)?;
        if rows.first() != Some(&first) || rows.last() != Some(&last) {
            return Err("Study endpoint samples disagree with exact saved worlds".into());
        }
        let initial_fp = state_fingerprint(&initial)?;
        let final_fp = state_fingerprint(&final_world)?;
        if m["initial_state_fingerprint"] != initial_fp || m["final_state_fingerprint"] != final_fp
        {
            return Err("Study state identity disagrees with manifest".into());
        }
        let definition: habitat::Definition =
            serde_json::from_value(json(&path.join("definition.json"), 64 * 1024)?)
                .map_err(|e| e.to_string())?;
        if definition != first.habitat.definition || definition != last.habitat.definition {
            return Err("Study original definition disagrees with endpoints".into());
        }
        if final_world.time < initial.time
            || final_world.time - initial.time > 30.0 * crate::time::SECONDS_PER_DAY
        {
            return Err("Study time range exceeds the 30-day reader horizon".into());
        }
        let mass = first.habitat.state.accounted_mass_kg();
        let mut previous = None;
        let mut uses_electrolysis = false;
        let mut uses_sabatier = false;
        let mut uses_moisture=false;let mut uses_breach=false;
        for row in &rows {
            if !row.simulation_time_s.is_finite()
                || row.simulation_time_s < initial.time
                || row.simulation_time_s > final_world.time
                || previous.is_some_and(|t| row.simulation_time_s <= t)
            {
                return Err("Habitat observations must have finite increasing world times".into());
            }
            previous = Some(row.simulation_time_s);
            if row.habitat.id != id || row.habitat.definition != definition {
                return Err(
                    "Habitat identity or original definition changes within recording".into(),
                );
            }
            row.habitat.validate(row.simulation_time_s)?;
            if (row.habitat.state.accounted_mass_kg() - mass).abs() > 1e-8 * mass.max(1.0) {
                return Err("Recorded closed habitat mass disagrees with initial inventory".into());
            }
            uses_electrolysis |= row.habitat.uses_electrolysis();
            uses_sabatier |= row.habitat.uses_sabatier();
            uses_moisture |= row.habitat.uses_moisture();uses_breach |= row.habitat.uses_breach();
        }
        for (used, key, descriptor, expected) in [
            (
                uses_electrolysis,
                "electrolysis_source",
                "electrolysis_model_descriptor",
                crate::electrolysis::source(),
            ),
            (
                uses_sabatier,
                "sabatier_source",
                "sabatier_model_descriptor",
                crate::sabatier::source(),
            ),
            (uses_moisture,"moisture_source","moisture_model_descriptor",crate::moisture::source()),
            (uses_breach,"breach_source","breach_model_descriptor",crate::breach::source()),
        ] {
            if used
                && (m[key] != expected
                    || m[descriptor]["id"]
                        != if key == "breach_source" {"breach"} else if key == "moisture_source" {"moisture"} else if key == "sabatier_source" {
                            "sabatier"
                        } else {
                            "electrolysis"
                        })
            {
                return Err(
                    "Missing or mismatched recorded reaction source/model descriptor".into(),
                );
            }
            if !used && !m[key].is_null() {
                return Err("Recording declares an unused reaction source".into());
            }
        }
        if definition.record_heat {
            if m["heat_accounting_source"]!=crate::habitat_heat::source() || m["heat_accounting_model_descriptor"]!=serde_json::to_value(crate::research::PhysicsModelRegistry::get("habitat-heat")).map_err(|e|e.to_string())? {return Err("Missing or mismatched recorded heat accounting source/model".into());}
        } else if !m["heat_accounting_source"].is_null() || !m["heat_accounting_model_descriptor"].is_null() {return Err("Recording declares unused heat accounting".into());}
        let report = json(&path.join("report.json"), 2 * 1024 * 1024)?;
        let expected = last.habitat.report();
        for key in [
            "format",
            "schema_version",
            "habitat",
            "oxygen_partial_pressure_pa",
            "co2_partial_pressure_pa",
            "total_pressure_pa",
            "source",
        ] {
            if report[key] != expected[key] {
                return Err(format!(
                    "Recorded report {key} disagrees with final habitat"
                ));
            }
        }
        for key in ["electrolysis_source", "sabatier_source", "moisture_source", "breach_source", "heat_accounting"] {
            if report[key] != expected[key] {
                return Err("Recorded report reaction source disagrees with final habitat".into());
            }
        }
        // Diagnostics are additive derived evidence. Old studies have no such key.
        if report.get("diagnostics").is_some() && report["diagnostics"] != expected["diagnostics"] {
            return Err("Recorded diagnostics disagree with the completed habitat interval".into());
        }
        // Previous 38-/49-column studies remain valid. New columns are additive; compare
        // every existing cell to the same shared formatter rather than silently trust CSV.
        let actual = read(&path.join("samples.csv"), 16 * 1024 * 1024)?;
        let expected = habitat::samples_csv(&rows);
        let a = actual.lines().collect::<Vec<_>>();
        let e = expected.lines().collect::<Vec<_>>();
        if a.len() != e.len() {
            return Err("CSV row count disagrees with JSON observations".into());
        }
        let header = a
            .first()
            .ok_or("Missing CSV header")?
            .split(',')
            .collect::<Vec<_>>();
        if !matches!(header.len(), 38 | 49 | 66 | 85 | 110) {
            return Err("Unsupported habitat CSV columns".into());
        }
        for (a, e) in a.iter().zip(&e) {
            let av = a.split(',').collect::<Vec<_>>();
            let ev = e.split(',').collect::<Vec<_>>();
            if av.len() != header.len() || av != ev[..header.len()] {
                return Err("CSV values disagree with JSON observations".into());
            }
        }
        let mut summary = serde_json::json!({"format":"cosmogon-habitat-recording-inspection","schema_version":1,"model":habitat::MODEL,"habitat_id":id,"name":definition.name,"body":definition.body,"latitude_deg":definition.latitude_deg,"longitude_east_deg":definition.longitude_east_deg,"sample_count":count,"initial_time_s":initial.time,"final_time_s":final_world.time,"duration_s":final_world.time-initial.time,"initial_state_fingerprint":initial_fp,"final_state_fingerprint":final_fp,"initial_dataset_versions":save::data_versions(&initial),"final_dataset_versions":save::data_versions(&final_world),"electrolysis_source":m["electrolysis_source"],"sabatier_source":m["sabatier_source"],"moisture_source":m["moisture_source"],"checks":"Bounded formats, sources/models, row budgets/identity/mass/time, exact endpoint worlds/fingerprints, original definition, report and CSV/JSON consistency","validation_scope":"Structural numerical consistency; no independent trajectory replay, cryptographic authenticity, predictive confidence or field calibration","metrics":Metric::for_habitat(&first.habitat).map(|v|serde_json::json!({"id":v.id(),"label":v.label(),"unit":v.unit()})).collect::<Vec<_>>()});
        if definition.record_heat {summary["heat_accounting_source"]=m["heat_accounting_source"].clone();}
        if uses_breach {summary["breach_source"]=m["breach_source"].clone();}
        Ok(Self { rows, summary })
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    #[default]
    Water,
    OxygenReserve,
    Hydrogen,
    Methane,
    RawWater,
    Temperature,
    OxygenPressure,
    Co2Pressure,
    Battery,
    DeliveredPower,
    UnservedPower,
    ElectrolysisOxygen,
    SabatierWater,
    SabatierHeat,
    Vapor,Condensate,Humidity,CondensationHeat,
    TotalPressure,EscapedGas,BreachLoss,BreachCooling,
}
impl Metric {
    pub const ALL: [Self; 22] = [
        Self::Water,
        Self::OxygenReserve,
        Self::Hydrogen,
        Self::Methane,
        Self::RawWater,
        Self::Temperature,
        Self::OxygenPressure,
        Self::Co2Pressure,
        Self::Battery,
        Self::DeliveredPower,
        Self::UnservedPower,
        Self::ElectrolysisOxygen,
        Self::SabatierWater,
        Self::SabatierHeat,Self::Vapor,Self::Condensate,Self::Humidity,Self::CondensationHeat,Self::TotalPressure,Self::EscapedGas,Self::BreachLoss,Self::BreachCooling,
    ];
    /// Additive breach metrics do not rewrite older design analyses/summary identities.
    pub fn for_habitat(h:&habitat::Habitat)->impl Iterator<Item=Self> {let used=h.uses_breach();Self::ALL.into_iter().filter(move |m| used || !matches!(m,Self::TotalPressure|Self::EscapedGas|Self::BreachLoss|Self::BreachCooling))}
    pub fn id(self) -> &'static str {
        match self {
            Self::TotalPressure=>"total_pressure",Self::EscapedGas=>"escaped_gas",Self::BreachLoss=>"breach_loss",Self::BreachCooling=>"breach_cooling",
            Self::Water => "water",
            Self::OxygenReserve => "oxygen_reserve",
            Self::Hydrogen => "hydrogen",
            Self::Methane => "methane",
            Self::RawWater => "raw_water",
            Self::Temperature => "temperature",
            Self::OxygenPressure => "oxygen_pressure",
            Self::Co2Pressure => "co2_pressure",
            Self::Battery => "battery",
            Self::DeliveredPower => "delivered_power",
            Self::UnservedPower => "unserved_power",
            Self::ElectrolysisOxygen => "electrolysis_oxygen",
            Self::SabatierWater => "sabatier_water",
            Self::SabatierHeat => "sabatier_heat",
            Self::Vapor=>"vapor",Self::Condensate=>"condensate",Self::Humidity=>"humidity",Self::CondensationHeat=>"condensation_heat",
        }
    }
    pub fn parse(id: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|v| v.id() == id)
            .ok_or_else(|| format!("Unknown habitat recording metric {id}"))
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::TotalPressure=>"Total cabin pressure",Self::EscapedGas=>"Cumulative escaped cabin gas",Self::BreachLoss=>"Supported interval gas escape",Self::BreachCooling=>"Supported interval equivalent cooling",
            Self::Water => "Water stock",
            Self::OxygenReserve => "Oxygen reserve",
            Self::Hydrogen => "Isolated hydrogen",
            Self::Methane => "Methane product",
            Self::RawWater => "Raw liquid product",
            Self::Temperature => "Cabin temperature",
            Self::OxygenPressure => "Oxygen partial pressure",
            Self::Co2Pressure => "CO₂ partial pressure",
            Self::Battery => "Battery energy",
            Self::DeliveredPower => "Served bus power",
            Self::UnservedPower => "Unserved bus power",
            Self::ElectrolysisOxygen => "Interval electrolytic oxygen",
            Self::SabatierWater => "Interval raw water produced",
            Self::SabatierHeat => "Interval reference reaction heat",
            Self::Vapor=>"Cabin water vapor",Self::Condensate=>"Isolated condensate",Self::Humidity=>"Relative humidity",Self::CondensationHeat=>"Interval reference condensation heat",
        }
    }
    pub fn unit(self) -> &'static str {
        match self {
            Self::Temperature => "K",
            Self::OxygenPressure | Self::Co2Pressure | Self::TotalPressure => "Pa",
            Self::Battery => "kWh",
            Self::DeliveredPower | Self::UnservedPower => "W",
            Self::SabatierHeat | Self::CondensationHeat | Self::BreachCooling => "kJ",
            Self::Humidity=>"%",
            _ => "kg",
        }
    }
    pub fn is_interval(self) -> bool {
        matches!(self,Self::DeliveredPower | Self::UnservedPower | Self::ElectrolysisOxygen | Self::SabatierWater | Self::SabatierHeat | Self::CondensationHeat | Self::BreachLoss | Self::BreachCooling)
    }
    pub fn value(self, row: &Sample) -> Option<f64> {
        let h = &row.habitat;
        let s = &h.state;
        let b = &h.budget;
        Some(match self {
            Self::TotalPressure=>s.pressure_pa(&h.config),
            Self::EscapedGas=>{if !h.uses_breach(){return None;}s.escaped_gas.unwrap_or_default().mass_kg()},
            Self::BreachLoss|Self::BreachCooling=>{let e=b.breach.as_ref()?;if !e.status.supported(){return None;}
                if self==Self::BreachLoss{e.escaped.mass_kg()}else{e.lumped_cooling_j/1000.0}},
            Self::Water => s.potable_water_kg,
            Self::Vapor=>{if !h.uses_moisture(){return None;}s.water_vapor_kg},
            Self::Condensate=>{if !h.uses_moisture(){return None;}s.condensate_kg},
            Self::Humidity=>{if !h.uses_moisture(){return None;}100.0*s.relative_humidity(&h.config)?},
            Self::CondensationHeat=>b.moisture.as_ref()?.condensation_heat_j/1000.0,
            Self::OxygenReserve => s.oxygen_reserve_kg,
            Self::Hydrogen => s.hydrogen_kg,
            Self::Methane => s.methane_kg,
            Self::RawWater => s.sabatier_water_kg,
            Self::Temperature => s.temperature_k,
            Self::OxygenPressure => s.oxygen_pa(&h.config),
            Self::Co2Pressure => s.co2_pa(&h.config),
            Self::Battery => s.battery_j / 3.6e6,
            Self::DeliveredPower | Self::UnservedPower => {
                if b.seconds <= 0.0 || !b.environment_available || !b.temperature_domain_supported {
                    return None;
                }
                if self == Self::DeliveredPower {
                    b.delivered_w
                } else {
                    b.unserved_w
                }
            }
            Self::ElectrolysisOxygen => b.electrolysis.as_ref()?.oxygen_generated_kg,
            Self::SabatierWater => b.sabatier.as_ref()?.water_generated_kg,
            Self::SabatierHeat => b.sabatier.as_ref()?.reaction_heat_j / 1000.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sandbox::Edit, BodyRef, Scenario, Universe, UniverseSettings};
    fn bundle(name: &str, reactions: u8) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "cosmogon-habitat-reader-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let mut u = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = BodyRef {
            system: 0,
            body: u.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut d = habitat::Definition::at(r, 0.0, 0.0);
        if reactions > 0 {
            d.config.electrolysis = Some(crate::electrolysis::Config::default());
        }
        if reactions > 1 {
            d.config.sabatier = Some(crate::sabatier::Config::default());
        }
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        let initial = u.clone();
        let mut rows = vec![habitat::sample(&u, 0).unwrap()];
        for _ in 0..3 {
            u.advance_by(3600.0);
            rows.push(habitat::sample(&u, 0).unwrap());
        }
        habitat::export(&initial, &u, 0, &rows, &root).unwrap();
        root
    }
    #[test]
    fn valid_optional_and_legacy_columns_reopen_exact_observations_with_unavailable_flows() {
        for n in 0..=2 {
            let root = bundle(&format!("valid-{n}"), n);
            let r = Recording::read(&root).unwrap();
            assert_eq!(r.rows.len(), 4);
            assert_eq!(r.summary["sample_count"], 4);
            assert_eq!(Metric::DeliveredPower.value(&r.rows[0]), None);
            assert_eq!(Metric::Water.value(&r.rows[0]), Some(200.0));
            assert_eq!(Metric::SabatierWater.value(&r.rows[0]), None);
            assert_eq!(Metric::SabatierWater.value(&r.rows[1]).is_some(), n == 2);
            for metric in Metric::ALL {
                assert_eq!(Metric::parse(metric.id()).unwrap(), metric);
                assert!(!metric.unit().is_empty());
            }
            if n < 2 {
                let csv = std::fs::read_to_string(root.join("samples.csv")).unwrap();
                let count = if n == 0 { 38 } else { 49 };
                let old = csv
                    .lines()
                    .map(|row| row.split(',').take(count).collect::<Vec<_>>().join(","))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n";
                std::fs::write(root.join("samples.csv"), old).unwrap();
                assert_eq!(Recording::read(&root).unwrap().rows, r.rows);
            }
            std::fs::remove_dir_all(root).unwrap();
        }
        assert!(Metric::parse("future_metric").is_err());
    }
    #[test]
    fn corrupt_sources_endpoints_reports_csv_and_observations_fail_explicitly() {
        let root = bundle("corrupt", 2);
        for (file, key, value) in [
            ("manifest.json", "schema_version", serde_json::json!(2)),
            ("manifest.json", "sample_count", serde_json::json!(1025)),
            (
                "manifest.json",
                "final_state_fingerprint",
                serde_json::json!("fake"),
            ),
            ("manifest.json", "sabatier_source", Value::Null),
            ("report.json", "total_pressure_pa", serde_json::json!(0)),
            ("report.json", "diagnostics", serde_json::json!({"invented":true})),
        ] {
            let p = root.join(file);
            let original = std::fs::read_to_string(&p).unwrap();
            let mut v: Value = serde_json::from_str(&original).unwrap();
            v[key] = value;
            std::fs::write(&p, v.to_string()).unwrap();
            assert!(Recording::read(&root).is_err(), "{file} {key}");
            std::fs::write(&p, original).unwrap();
        }
        let p = root.join("samples.json");
        let original = std::fs::read_to_string(&p).unwrap();
        for variant in 0..4 {
            let mut v: Value = serde_json::from_str(&original).unwrap();
            match variant {
                0 => v[1]["simulation_time_s"] = v[0]["simulation_time_s"].clone(),
                1 => v[1]["habitat"]["state"]["potable_water_kg"] = 0.into(),
                2 => v[3]["habitat"]["state"]["battery_j"] = 0.into(),
                _ => v[1]["invented"] = true.into(),
            };
            std::fs::write(&p, v.to_string()).unwrap();
            assert!(Recording::read(&root).is_err(), "variant {variant}");
        }
        std::fs::write(&p, original).unwrap();
        let p = root.join("samples.csv");
        let original = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, original.replacen("temperature_k", "invented", 1)).unwrap();
        assert!(Recording::read(&root).is_err());
        std::fs::write(&p, original).unwrap();
        assert!(Recording::read(&root).is_ok());
        let p = root.join("report.json");
        let mut legacy: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        legacy.as_object_mut().unwrap().remove("diagnostics");
        std::fs::write(p, legacy.to_string()).unwrap();
        assert!(Recording::read(&root).is_ok(), "Legacy reports without derived diagnostics remain valid");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn snapshot_and_bounded_input_are_distinct_from_trajectories() {
        let root = bundle("snapshot", 0);
        let u =
            save::from_json(&std::fs::read_to_string(root.join("final.cosmo")).unwrap()).unwrap();
        let snap = root.with_extension("snapshot");
        let _ = std::fs::remove_dir_all(&snap);
        habitat::export(&u, &u, 0, &[habitat::sample(&u, 0).unwrap()], &snap).unwrap();
        let r = Recording::read(&snap).unwrap();
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.summary["duration_s"], 0.0);
        std::fs::write(snap.join("manifest.json"), vec![b' '; 64 * 1024 + 1]).unwrap();
        assert!(Recording::read(&snap).unwrap_err().contains("exceeds"));
        std::fs::remove_dir_all(snap).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
