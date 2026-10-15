//! Versioned save files.
//!
//! A save is JSON with a small header and the full [`Universe`] state, which is made only of
//! explicit simulation structures (no runtime/engine objects). Static data that ships with
//! the app (technology graph) is *not* saved; the scientific parameters *are*, so an old
//! save keeps the science it was created with.
//!
//! When the format changes: bump [`CURRENT_VERSION`] and add a migration step to
//! [`MIGRATIONS`] that rewrites version N JSON into version N+1.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::universe::Universe;

pub const FORMAT: &str = "cosmogon-save";
pub const CURRENT_VERSION: u32 = 2;
/// Version of the physics engine (integrator and collision model) recorded in saves.
pub const PHYSICS_ENGINE: &str = "nbody-yoshida4/1";
pub const EXTENSION: &str = "cosmo";

/// `MIGRATIONS[i]` upgrades a version `i + 1` save to version `i + 2`.
const MIGRATIONS: &[fn(&mut Value) -> Result<(), SaveError>] = &[v1_to_v2];

/// v2 (0.4) adds sandbox state — N-body dynamics, provenance, object classes, removals,
/// impacts and the edit journal. Every new field has a default, so v1 data is valid v2
/// data unchanged: old universes load with analytic orbits, exactly as before.
fn v1_to_v2(_v: &mut Value) -> Result<(), SaveError> {
    Ok(())
}

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("not a Cosmogon save file")]
    NotASave,
    #[error("save version {0} is newer than this application supports ({CURRENT_VERSION}); please update Cosmogon")]
    TooNew(u32),
    #[error("save is corrupt: {0}")]
    Corrupt(#[from] serde_json::Error),
    #[error("unsupported or mismatched terrain dataset: {0}")]
    Dataset(String),
    #[error("unsupported or invalid habitat model: {0}")]
    Model(String),
    #[error("unsupported or invalid finite vehicle model: {0}")]
    Vehicle(String),
    #[error("migration failed: {0}")]
    Migration(String),
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SaveHeader {
    pub format: String,
    pub version: u32,
    pub app_version: String,
    pub name: String,
    pub saved_at_unix: u64,
    pub seed: u64,
    pub scenario: String,
    pub date: String,
    pub civilizations: u32,
    /// Integrator / collision model version (reproducibility).
    #[serde(default)]
    pub physics_engine: String,
    /// Datasets (id → version) this universe was built from.
    #[serde(default)]
    pub data_versions: std::collections::BTreeMap<String, String>,
    /// Simulation time (s since J2000).
    #[serde(default)]
    pub sim_time: f64,
}

#[derive(Serialize)]
struct SaveOut<'a> {
    header: SaveHeader,
    universe: &'a Universe,
}

#[derive(Deserialize)]
struct HeaderOnly {
    header: SaveHeader,
}

pub fn header_for(u: &Universe, name: &str) -> SaveHeader {
    SaveHeader {
        format: FORMAT.into(),
        version: CURRENT_VERSION,
        app_version: env!("CARGO_PKG_VERSION").into(),
        name: name.into(),
        saved_at_unix: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        seed: u.settings.seed,
        scenario: u.settings.scenario.label().into(),
        date: u.date_label(),
        civilizations: u.civs.iter().filter(|c| c.is_alive()).count() as u32,
        physics_engine: PHYSICS_ENGINE.into(),
        data_versions: data_versions(u),
        sim_time: u.time,
    }
}

/// The external datasets a universe depends on.
pub fn data_versions(u: &Universe) -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    let sol = u.systems.iter().any(|s| s.bodies.iter().any(|b| b.real));
    if sol {
        m.insert("nasa-planetary-fact-sheets".into(), "sol.toml@1".into());
    }
    if u.systems.iter().any(|s| s.bodies.iter().any(|b| b.provenance.source.contains("Horizons"))) {
        let d = crate::astro::horizons::Dataset::embedded();
        m.insert("jpl-horizons-sol".into(), format!("{} (epoch {})", d.dataset.version, d.dataset.epoch));
    }
    if u.systems.iter().any(|s| s.bodies.iter().any(|b| b.elevation_data == Some(crate::planet::terrain::ElevationData::Earth))) {
        m.insert("noaa-etopo5-earth".into(), "1".into());
    }
    if u.systems.iter().any(|s| s.bodies.iter().any(|b| b.elevation_data == Some(crate::planet::terrain::ElevationData::MarsMola))) {
        m.insert("nasa-pds-mgs-mola-megdr".into(), crate::planet::mola::DATASET.into());
    }
    if u.systems.iter().any(|s|s.bodies.iter().any(|b|b.elevation_data == Some(crate::planet::terrain::ElevationData::MarsMola16))) {
        m.insert("nasa-pds-mgs-mola-megdr-16ppd".into(), crate::planet::mola::DATASET_16.into());
    }
    if !u.habitats.is_empty() || u.next_habitat_id>0 {m.insert("cosmogon-habitat-budgets".into(),crate::habitat::MODEL.into());}
    if u.habitats.iter().any(crate::habitat::Habitat::uses_crop) {m.insert("cosmogon-finite-crop-reference".into(),crate::crop_process::MODEL.into());m.insert("cosmogon-crop-chon-surrogate".into(),crate::crop_chemistry::MODEL.into());m.insert("ciaaw-chon-nominal".into(),crate::crop_chemistry::ATOMIC_MODEL.into());}
    if u.habitats.iter().any(crate::habitat::Habitat::uses_electrolysis) {m.insert("cosmogon-water-electrolysis".into(),crate::electrolysis::MODEL.into());}
    if u.habitats.iter().any(crate::habitat::Habitat::uses_moisture) {m.insert("cosmogon-cabin-moisture".into(),crate::moisture::MODEL.into());}
    if u.habitats.iter().any(crate::habitat::Habitat::uses_sabatier) {m.insert("cosmogon-sabatier".into(),crate::sabatier::MODEL.into());}
    if u.habitats.iter().any(|h|h.heat.is_some()) {m.insert("cosmogon-cabin-heat-accounting".into(),crate::habitat_heat::MODEL.into());}
    if u.habitats.iter().any(crate::habitat::Habitat::uses_breach) {m.insert("cosmogon-cabin-breach".into(),crate::breach::MODEL.into());}
    if !u.vehicles.is_empty() || u.next_vehicle_id > 0 {
        m.insert("cosmogon-finite-world-vehicle".into(),crate::world_vehicle::MODEL.into());
        m.insert("cosmogon-supply-vehicle-reference".into(),crate::supply_vehicle::MODEL.into());
    }
    if u.vehicles.iter().any(|v| v.contact.is_some()) || u.edits.iter().any(|r| matches!(&r.edit, crate::sandbox::Edit::CreateVehicle {definition} if definition.contact.is_some())) {
        m.insert("cosmogon-finite-terrain-contact".into(),crate::vehicle_contact::MODEL.into());
        m.insert("cosmogon-world-vehicle-contact-capability".into(),crate::world_vehicle::CONTACT_MODEL.into());
    }
    if u.vehicles.iter().any(|v| v.definition.contact.as_ref().is_some_and(|h|h.shear.is_some())) || u.edits.iter().any(|r| matches!(&r.edit, crate::sandbox::Edit::CreateVehicle {definition} if definition.contact.as_ref().is_some_and(|h|h.shear.is_some()))) {
        m.insert("cosmogon-finite-foot-shear".into(),crate::vehicle_shear::MODEL.into());
        m.insert("cosmogon-persistent-terrain-contact".into(),crate::vehicle_contact_dynamics::SHEAR_MODEL.into());
        m.insert("cosmogon-world-vehicle-shear-capability".into(),crate::world_vehicle::SHEAR_MODEL.into());
    }
    if u.vehicles.iter().any(|v| v.definition.contact.as_ref().is_some_and(|h|h.operation.is_some())) || u.edits.iter().any(|r| matches!(&r.edit, crate::sandbox::Edit::CreateVehicle {definition} if definition.contact.as_ref().is_some_and(|h|h.operation.is_some()))) {
        m.insert("cosmogon-vehicle-surface-operation".into(),crate::vehicle_operation::MODEL.into());
    }
    if u.vehicles.iter().any(|v| v.definition.cargo_ledgers) || u.edits.iter().any(|r| matches!(&r.edit, crate::sandbox::Edit::CreateVehicle {definition} if definition.cargo_ledgers)) {
        m.insert("cosmogon-vehicle-cargo-flight".into(),crate::vehicle_cargo_mass::MODEL.into());
    }
    if u.vehicles.iter().any(|v| v.definition.handling.is_some()) || u.edits.iter().any(|r| matches!(&r.edit, crate::sandbox::Edit::CreateVehicle {definition} if definition.handling.is_some())) {
        m.insert("cosmogon-vehicle-handling".into(),crate::vehicle_handling::MODEL.into());
    }
    if u.vehicles.iter().any(|v| !v.transfers.is_empty()) || u.habitats.iter().any(|h| !h.receipts.is_empty()) || u.edits.iter().any(|r| matches!(&r.edit, crate::sandbox::Edit::TransferVehicleCargoToHabitat {..})) {
        m.insert("cosmogon-vehicle-transfer".into(),crate::vehicle_transfer::MODEL.into());
    }
    m
}

pub fn to_json(u: &Universe, name: &str) -> Result<String, SaveError> {
    Ok(serde_json::to_string(&SaveOut { header: header_for(u, name), universe: u })?)
}

pub fn from_json(s: &str) -> Result<Universe, SaveError> {
    let mut v: Value = serde_json::from_str(s)?;
    let version = check_header(&v)?;
    for step in &MIGRATIONS[(version as usize - 1)..] {
        step(&mut v)?;
    }
    let terrain_versions = v["header"]["data_versions"].clone();
    let universe = v.get_mut("universe").map(Value::take).ok_or(SaveError::NotASave)?;
    let mut u: Universe = serde_json::from_value(universe)?;
    let expected = data_versions(&u);
    for key in ["nasa-pds-mgs-mola-megdr", "nasa-pds-mgs-mola-megdr-16ppd"] {
        if let Some(version) = terrain_versions.get(key) {
            if version.as_str() != expected.get(key).map(String::as_str) {return Err(SaveError::Dataset(format!("{key}: the header and body's pinned source disagree")));}
        } else if key.ends_with("16ppd") && expected.contains_key(key) {
            return Err(SaveError::Dataset("16ppd world is missing its required dataset version".into()));
        }
    }
    if u.next_habitat_id>0 && terrain_versions.get("cosmogon-habitat-budgets").and_then(Value::as_str)!=Some(crate::habitat::MODEL) {return Err(SaveError::Model("Missing or mismatched experiment model version".into()));}
    for (id,model) in [("cosmogon-finite-crop-reference",crate::crop_process::MODEL),("cosmogon-crop-chon-surrogate",crate::crop_chemistry::MODEL),("ciaaw-chon-nominal",crate::crop_chemistry::ATOMIC_MODEL)] {if (expected.contains_key(id) || terrain_versions.get(id).is_some()) && terrain_versions.get(id).and_then(Value::as_str)!=Some(model){return Err(SaveError::Model(format!("Missing or mismatched {id} model pin")));}}
    if (expected.contains_key("cosmogon-water-electrolysis") || terrain_versions.get("cosmogon-water-electrolysis").is_some())
        && terrain_versions.get("cosmogon-water-electrolysis").and_then(Value::as_str)!=Some(crate::electrolysis::MODEL) {
        return Err(SaveError::Model("Missing or mismatched electrolysis model version".into()));
    }
    if (expected.contains_key("cosmogon-cabin-moisture") || terrain_versions.get("cosmogon-cabin-moisture").is_some())
        && terrain_versions.get("cosmogon-cabin-moisture").and_then(Value::as_str)!=Some(crate::moisture::MODEL) {
        return Err(SaveError::Model("Missing or mismatched cabin moisture version".into()));
    }
    if (expected.contains_key("cosmogon-sabatier") || terrain_versions.get("cosmogon-sabatier").is_some())
        && terrain_versions.get("cosmogon-sabatier").and_then(Value::as_str)!=Some(crate::sabatier::MODEL) {
        return Err(SaveError::Model("Missing or mismatched Sabatier model version".into()));
    }
    if (expected.contains_key("cosmogon-cabin-breach") || terrain_versions.get("cosmogon-cabin-breach").is_some())
        && terrain_versions.get("cosmogon-cabin-breach").and_then(Value::as_str)!=Some(crate::breach::MODEL) {return Err(SaveError::Model("Missing or mismatched cabin breach version".into()));}
    if (expected.contains_key("cosmogon-cabin-heat-accounting") || terrain_versions.get("cosmogon-cabin-heat-accounting").is_some()) && terrain_versions.get("cosmogon-cabin-heat-accounting").and_then(Value::as_str)!=Some(crate::habitat_heat::MODEL) {return Err(SaveError::Model("Missing or mismatched cabin heat accounting version".into()));}
    if !u.habitats.is_empty() {
        if terrain_versions.get("cosmogon-habitat-budgets").and_then(Value::as_str)!=Some(crate::habitat::MODEL) {return Err(SaveError::Model("Missing or mismatched model version".into()));}
        if u.habitats.len()>64 {return Err(SaveError::Model("More than 64 habitats".into()));}
        let task=u.scheduler.tasks.get(crate::habitat::TASK).ok_or_else(||SaveError::Model("Missing habitat scheduler".into()))?;
        if task.name!="habitats" || task.period!=crate::habitat::PERIOD || !task.origin.is_finite() || task.next_due()<=u.time || task.next_due()-u.time>crate::habitat::PERIOD+0.001 {return Err(SaveError::Model("Invalid habitat scheduler".into()));}
        let mut ids=std::collections::BTreeSet::new();
        for h in &u.habitats {
            h.validate(u.time).map_err(SaveError::Model)?;
            if h.id>=u.next_habitat_id || !ids.insert(h.id) || u.systems.get(h.definition.body.system as usize).and_then(|s|s.bodies.get(h.definition.body.body as usize)).is_none() {return Err(SaveError::Model("Duplicate habitat identifier or invalid world reference".into()));}
        }
    }
    for (id,model) in [("cosmogon-finite-world-vehicle",crate::world_vehicle::MODEL),("cosmogon-supply-vehicle-reference",crate::supply_vehicle::MODEL),("cosmogon-finite-terrain-contact",crate::vehicle_contact::MODEL),("cosmogon-world-vehicle-contact-capability",crate::world_vehicle::CONTACT_MODEL),("cosmogon-finite-foot-shear",crate::vehicle_shear::MODEL),("cosmogon-persistent-terrain-contact",crate::vehicle_contact_dynamics::SHEAR_MODEL),("cosmogon-world-vehicle-shear-capability",crate::world_vehicle::SHEAR_MODEL),("cosmogon-vehicle-surface-operation",crate::vehicle_operation::MODEL),("cosmogon-vehicle-cargo-flight",crate::vehicle_cargo_mass::MODEL),("cosmogon-vehicle-handling",crate::vehicle_handling::MODEL),("cosmogon-vehicle-transfer",crate::vehicle_transfer::MODEL)] {
        if (expected.contains_key(id) || terrain_versions.get(id).is_some()) && terrain_versions.get(id).and_then(Value::as_str) != Some(model) {
            return Err(SaveError::Vehicle(format!("Missing or mismatched {id} model pin")));
        }
    }
    u.validate_vehicle_state().map_err(SaveError::Vehicle)?;
    // Acceleration is a derived, skipped cache. Restore it before any position reader
    // extrapolates between macro steps (illumination, climate, camera and habitat budgets).
    // Otherwise a resumed dynamic world briefly uses linear instead of quadratic motion.
    for sys in &mut u.systems {sys.refresh_accelerations();}
    // Tasks added by newer versions (stellar evolution) start running in older saves.
    u.ensure_star_task();
    Ok(u)
}

fn check_header(v: &Value) -> Result<u32, SaveError> {
    let h = v.get("header").ok_or(SaveError::NotASave)?;
    if h.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(SaveError::NotASave);
    }
    let version = h.get("version").and_then(Value::as_u64).ok_or(SaveError::NotASave)? as u32;
    if version == 0 {
        return Err(SaveError::NotASave);
    }
    if version > CURRENT_VERSION {
        return Err(SaveError::TooNew(version));
    }
    Ok(version)
}

pub fn read_header(s: &str) -> Result<SaveHeader, SaveError> {
    let h: HeaderOnly = serde_json::from_str(s)?;
    if h.header.format != FORMAT {
        return Err(SaveError::NotASave);
    }
    Ok(h.header)
}

/// Write atomically: a crash mid-save never destroys the previous file.
pub fn write_file(path: &Path, u: &Universe, name: &str) -> Result<(), SaveError> {
    let json = to_json(u, name)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

pub fn read_file(path: &Path) -> Result<Universe, SaveError> {
    from_json(&std::fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::SECONDS_PER_KYR;
    use crate::universe::{Scenario, UniverseSettings};

    #[test]
    fn round_trip_then_continue_matches_uninterrupted_run() {
        let settings = UniverseSettings { seed: 5, scenario: Scenario::Sol, system_count: 4, ..Default::default() };
        let mut a = Universe::new(settings);
        a.advance_by(2.0 * SECONDS_PER_KYR);
        let json = to_json(&a, "test").unwrap();
        let mut b = from_json(&json).unwrap();
        assert_eq!(a, b);
        a.advance_by(2.0 * SECONDS_PER_KYR);
        b.advance_by(2.0 * SECONDS_PER_KYR);
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
        assert_eq!(read_header(&json).unwrap().name, "test");
    }

    #[test]
    fn dynamic_save_restores_between_step_positions_and_acceleration_cache() {
        let mut a=Universe::new(UniverseSettings {scenario:Scenario::SolarSystemLab,..Default::default()});a.advance_by(120.0);
        let mut b=from_json(&to_json(&a,"dynamic").unwrap()).unwrap();
        assert_eq!(serde_json::to_string(&a).unwrap(),serde_json::to_string(&b).unwrap());
        for (i,body) in a.systems[0].bodies.iter().enumerate() {if body.exists() {assert_eq!(a.systems[0].body_local_position(i,a.time),b.systems[0].body_local_position(i,b.time));}}
        assert_eq!(a.systems[0].dynamics.as_ref().unwrap().acc,b.systems[0].dynamics.as_ref().unwrap().acc);
        a.advance_by(3600.0);b.advance_by(3600.0);assert_eq!(serde_json::to_string(&a).unwrap(),serde_json::to_string(&b).unwrap());
    }

    #[test]
    fn version_1_saves_still_load() {
        let u = Universe::new(UniverseSettings { seed: 9, scenario: Scenario::GardenWorld, system_count: 3, ..Default::default() });
        let json = to_json(&u, "old").unwrap().replace(&format!("\"version\":{CURRENT_VERSION}"), "\"version\":1");
        assert!(json.contains("\"version\":1"));
        let back = from_json(&json).unwrap();
        assert_eq!(back, u);
        assert!(back.systems.iter().all(|s| !s.is_dynamic()));
    }

    #[test]
    fn rejects_foreign_and_future_files() {
        assert!(matches!(from_json(r#"{"hello":1}"#), Err(SaveError::NotASave)));
        let future = format!(r#"{{"header":{{"format":"{FORMAT}","version":99}},"universe":{{}}}}"#);
        assert!(matches!(from_json(&future), Err(SaveError::TooNew(99))));
    }

    #[test]
    fn cargo_flight_pin_is_conditional_and_enforced() {
        let settings = UniverseSettings { scenario: Scenario::SolarSystemLab, ..Default::default() };
        let plain = Universe::new(settings.clone());
        let header = serde_json::to_value(header_for(&plain, "plain")).unwrap();
        assert!(header["data_versions"].get("cosmogon-vehicle-cargo-flight").is_none());
        let mut ledger_world = plain.clone();
        let definition = crate::world_vehicle::Definition::circular_example(
            &ledger_world,
            crate::BodyRef { system: 0, body: 3 },
            crate::supply_vehicle::Plan::example(),
        )
        .unwrap()
        .with_cargo_ledgers()
        .unwrap();
        ledger_world
            .apply_edit(crate::sandbox::Edit::CreateVehicle { definition: Box::new(definition) })
            .unwrap();
        let header = serde_json::to_value(header_for(&ledger_world, "ledger")).unwrap();
        assert_eq!(
            header["data_versions"]["cosmogon-vehicle-cargo-flight"].as_str(),
            Some(crate::vehicle_cargo_mass::MODEL)
        );
        let json = to_json(&ledger_world, "ledger").unwrap();
        assert!(from_json(&json).is_ok());
        let mut stripped: Value = serde_json::from_str(&json).unwrap();
        stripped["header"]["data_versions"]
            .as_object_mut()
            .unwrap()
            .remove("cosmogon-vehicle-cargo-flight");
        assert!(matches!(from_json(&stripped.to_string()), Err(SaveError::Vehicle(_))));
        let mut tampered: Value = serde_json::from_str(&json).unwrap();
        tampered["header"]["data_versions"]["cosmogon-vehicle-cargo-flight"] = serde_json::json!("wrong/9");
        assert!(matches!(from_json(&tampered.to_string()), Err(SaveError::Vehicle(_))));
    }

    #[test]
    fn handling_pin_is_conditional_and_enforced() {
        let settings = UniverseSettings { scenario: Scenario::SolarSystemLab, ..Default::default() };
        let plain = Universe::new(settings);
        let header = serde_json::to_value(header_for(&plain, "plain")).unwrap();
        assert!(header["data_versions"].get("cosmogon-vehicle-handling").is_none());

        let mut world = plain;
        let mars = crate::BodyRef { system: 0, body: world.systems[0].find_body("Mars").unwrap() as u32 };
        let frame = crate::vehicle_arrival::SurfaceFrame::at(&world, mars, world.time).unwrap();
        let above = frame
            .to_inertial(&crate::vehicle_arrival::KinematicState {
                position_m: crate::Vec3d::new(0.8, 0.6, 0.) * (frame.radius_m + 10000.),
                velocity_m_s: crate::Vec3d::ZERO,
            })
            .unwrap();
        let surface = crate::vehicle_contact::Surface::sample(&world, mars, world.time, above.position_m, 1.).unwrap();
        let position = surface.position_m + surface.outward_normal * 1.6;
        let mut hardware = crate::vehicle_contact_dynamics::Hardware::example(&world, mars, position).unwrap();
        hardware.operation = Some(crate::vehicle_operation::Definition {
            model: crate::vehicle_operation::MODEL.into(),
            support: crate::vehicle_support::Definition::default(),
            required_window_s: 5.,
            maximum_sample_gap_s: 1.000001,
            maximum_structural_utilization: 1.,
            maximum_stroke_utilization: 1.,
            maximum_friction_utilization: 1.,
        });
        let centre = world.systems[0].body_state(mars.body as usize, world.time);
        let definition = crate::world_vehicle::Definition {
            model: crate::world_vehicle::CONTACT_MODEL.into(),
            frame: crate::world_vehicle::FRAME.into(),
            parent: mars,
            plan: crate::supply_vehicle::Plan::example(),
            initial_relative_position_m: position - centre.pos,
            initial_relative_velocity_m_s: crate::Vec3d::ZERO,
            contact: Some(hardware),
            cargo_ledgers: true,
            cargo_removals: vec![],
            handling: Some(crate::vehicle_handling::Equipment::example()),
        };
        definition.validate(&world).unwrap();
        world
            .apply_edit(crate::sandbox::Edit::CreateVehicle { definition: Box::new(definition) })
            .unwrap();
        let header = serde_json::to_value(header_for(&world, "handling")).unwrap();
        assert_eq!(
            header["data_versions"]["cosmogon-vehicle-handling"].as_str(),
            Some(crate::vehicle_handling::MODEL)
        );
        let json = to_json(&world, "handling").unwrap();
        assert!(from_json(&json).is_ok());
        let mut stripped: Value = serde_json::from_str(&json).unwrap();
        stripped["header"]["data_versions"]
            .as_object_mut()
            .unwrap()
            .remove("cosmogon-vehicle-handling");
        assert!(matches!(from_json(&stripped.to_string()), Err(SaveError::Vehicle(_))));
        let mut tampered: Value = serde_json::from_str(&json).unwrap();
        tampered["header"]["data_versions"]["cosmogon-vehicle-handling"] = serde_json::json!("wrong/9");
        assert!(matches!(from_json(&tampered.to_string()), Err(SaveError::Vehicle(_))));
    }

    #[test]
    fn transfer_pin_is_conditional_and_enforced() {
        let settings = UniverseSettings { scenario: Scenario::SolarSystemLab, ..Default::default() };
        let plain = Universe::new(settings);
        let header = serde_json::to_value(header_for(&plain, "plain")).unwrap();
        assert!(header["data_versions"].get("cosmogon-vehicle-transfer").is_none());

        let mut world = plain.clone();
        let mars = crate::BodyRef { system: 0, body: world.systems[0].find_body("Mars").unwrap() as u32 };
        world
            .apply_edit(crate::sandbox::Edit::CreateHabitat { definition: Box::new(crate::habitat::Definition::at(mars, -4.59, 137.44)) })
            .unwrap();
        // Journaling a transfer edit is enough for the pin; no live craft needed.
        world.edits.push(crate::sandbox::EditRecord {
            time: world.time,
            summary: "pinned transfer evidence".into(),
            edit: crate::sandbox::Edit::TransferVehicleCargoToHabitat {
                system: 0,
                vehicle_id: 7,
                habitat_id: 0,
                requested: crate::supply_vehicle::Cargo {
                    oxygen_reserve_kg: 1.,
                    potable_water_kg: 0.,
                    food_dry_kg: 0.,
                    food_water_kg: 0.,
                },
            },
        });
        let header = serde_json::to_value(header_for(&world, "transfer")).unwrap();
        assert_eq!(
            header["data_versions"]["cosmogon-vehicle-transfer"].as_str(),
            Some(crate::vehicle_transfer::MODEL)
        );
        let json = to_json(&world, "transfer").unwrap();
        assert!(from_json(&json).is_ok());
        let mut stripped: Value = serde_json::from_str(&json).unwrap();
        stripped["header"]["data_versions"]
            .as_object_mut()
            .unwrap()
            .remove("cosmogon-vehicle-transfer");
        assert!(matches!(from_json(&stripped.to_string()), Err(SaveError::Vehicle(_))));
        let mut tampered: Value = serde_json::from_str(&json).unwrap();
        tampered["header"]["data_versions"]["cosmogon-vehicle-transfer"] = serde_json::json!("wrong/9");
        assert!(matches!(from_json(&tampered.to_string()), Err(SaveError::Vehicle(_))));
    }
}

#[cfg(test)]
mod golden {
    use crate::time::SECONDS_PER_KYR;
    use crate::universe::{Scenario, UniverseSettings};
    use crate::Universe;

    /// FNV-1a over the serialised state: a compact fingerprint of an entire universe.
    pub fn fingerprint(u: &Universe) -> u64 {
        let json = serde_json::to_string(u).unwrap();
        json.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3))
    }

    /// Cross-platform determinism guard. CI runs this on macOS (ARM), Linux (x86-64) and
    /// Windows (x86-64): the same seed must yield bit-identical universes everywhere.
    /// If a deliberate model change alters results, update the constant (and say so in the
    /// changelog — old saves still load, but replays of old seeds will differ).
    #[test]
    fn golden_universe_fingerprint() {
        let mut u = Universe::new(UniverseSettings { seed: 2026, scenario: Scenario::Sol, system_count: 5, ..Default::default() });
        u.advance_by(5.0 * SECONDS_PER_KYR);
        let fp = fingerprint(&u);
        println!("golden fingerprint: {fp:#018x}");
        assert_eq!(fp, GOLDEN, "universe fingerprint changed: {fp:#018x}");
    }

    const GOLDEN: u64 = 0xf2c4140a45ae509b;
}
