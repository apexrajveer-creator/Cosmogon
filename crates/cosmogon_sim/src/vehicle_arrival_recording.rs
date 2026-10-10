//! Independently replayed rotating approach evidence, without changing flight recordings.
use crate::{
    research,
    sandbox::Edit,
    vehicle_arrival::{self, Approach},
    world_vehicle::{Phase, Source},
    world_vehicle_recording::Bundle,
    BodyRef,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::Write, path::Path};
const MAX_BYTES: u64 = 32 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub target: BodyRef,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub world_time_s: f64,
    pub reading: Option<Approach>,
    pub unavailable_reason: Option<String>,
    /// Repeated terminal readings retain the first observed source-frame evidence.
    pub historical: bool,
}
#[derive(Clone, Debug)]
pub struct Report {
    pub definition: Definition,
    pub flight: Bundle,
    pub points: Vec<Point>,
    pub manifest: Value,
}
impl Report {
    pub fn run(flight: &Bundle, target: BodyRef) -> Result<Self, String> {
        flight.study.validate(&flight.origin)?;
        let replay = Bundle::run(&flight.origin, flight.study.clone())?;
        if replay.manifest != flight.manifest
            || serde_json::to_value(&replay.rows).map_err(|e| e.to_string())?
                != serde_json::to_value(&flight.rows).map_err(|e| e.to_string())?
            || research::state_fingerprint(&replay.final_world)?
                != research::state_fingerprint(&flight.final_world)?
        {
            return Err(
                "Approach source is not the complete independently reproduced flight".into(),
            );
        }

        if target.system != flight.study.flight.parent.system {
            return Err("Approach target must share the flight system".into());
        }
        vehicle_arrival::SurfaceFrame::at(&flight.origin, target, flight.origin.time)?;
        if flight.rows.len() != flight.study.samples {
            return Err("Flight source has incomplete observations".into());
        }
        let definition = Definition {
            model: vehicle_arrival::MODEL.into(),
            target,
        };
        let mut world = flight.origin.clone();
        let id = world.next_vehicle_id;
        world.apply_edit(Edit::CreateVehicle {
            definition: Box::new(flight.study.flight.clone()),
        })?;
        let mut points = Vec::with_capacity(flight.rows.len());
        let mut terminal: Option<Point> = None;
        for (i, row) in flight.rows.iter().enumerate() {
            let time = flight.origin.time + i as f64 * flight.study.interval_s;
            if i > 0 {
                world.advance_to(time, None, None);
            }
            let craft = world
                .vehicles
                .iter()
                .find(|c| c.id == id)
                .ok_or("Approach craft disappeared")?;
            if world.time != time
                || row.world_time_s != time
                || serde_json::to_value(craft).map_err(|e| e.to_string())?
                    != serde_json::to_value(&row.flight).map_err(|e| e.to_string())?
            {
                return Err(format!("Approach source replay differs at observation {i}"));
            }
            let mut point = if let Some(saved) = &terminal {
                let mut p = saved.clone();
                p.historical = true;
                p
            } else {
                let reading=match &craft.phase {
                    Phase::Unsupported {reason}=>Err(format!("Stopped flight lacks supported arrival geometry: {reason}")),
                    Phase::Impact {source,..} if source!=&Source::Body(target.body)=>Err("First contact was with a different source; selected target is not the contact surface".into()),
                    _=>Approach::observe(&world,craft,target),
                };
                let p = match reading {
                    Ok(reading) => Point {
                        world_time_s: time,
                        reading: Some(reading),
                        unavailable_reason: None,
                        historical: false,
                    },
                    Err(reason) => Point {
                        world_time_s: time,
                        reading: None,
                        unavailable_reason: Some(reason),
                        historical: false,
                    },
                };
                if craft.phase != Phase::Flying {
                    terminal = Some(p.clone());
                }
                p
            };
            point.world_time_s = time;
            points.push(point);
        }
        if research::state_fingerprint(&world)? != research::state_fingerprint(&flight.final_world)?
        {
            return Err("Approach source replay differs from the complete final world".into());
        }
        let mut manifest = json!({"format":"cosmogon-vehicle-approach-recording","schema_version":1,"complete":true,"definition":definition,"flight_manifest":flight.manifest,"observations":points.len(),"context":vehicle_arrival::context(),"files":["definition.json","approach.json","approach.csv","flight/definition.json","flight/origin.cosmo","flight/observations.json","flight/trajectories.csv","flight/final.cosmo","flight/manifest.json"]});
        if flight.study.flight.contact.is_some() {manifest["files"].as_array_mut().unwrap().push(json!("flight/contact.csv"));}
        if flight.study.flight.contact.as_ref().is_some_and(|h|h.shear.is_some()) {manifest["files"].as_array_mut().unwrap().push(json!("flight/shear.csv"));}
        Ok(Self {
            definition,
            flight: flight.clone(),
            points,
            manifest,
        })
    }
    pub fn export(&self, path: &Path) -> Result<Value, String> {
        if path.exists() {
            return Err("Existing approach output is protected; choose a fresh folder".into());
        }
        let verified = Self::run(&self.flight, self.definition.target)?;
        if self.definition != verified.definition
            || serde_json::to_value(&self.points).map_err(|e| e.to_string())?
                != serde_json::to_value(&verified.points).map_err(|e| e.to_string())?
            || self.manifest != verified.manifest
        {
            return Err("Approach evidence differs from independently replayed source".into());
        }
        let files = [
            (
                "definition.json",
                serde_json::to_vec_pretty(&self.definition).map_err(|e| e.to_string())?,
            ),
            (
                "approach.json",
                serde_json::to_vec_pretty(&self.points).map_err(|e| e.to_string())?,
            ),
            ("approach.csv", csv(&self.points).into_bytes()),
            (
                "manifest.json",
                serde_json::to_vec_pretty(&self.manifest).map_err(|e| e.to_string())?,
            ),
        ];
        if files.iter().any(|(_, b)| b.len() as u64 > MAX_BYTES) {
            return Err("Approach export exceeds bounded file size".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| e.to_string())?;
        self.flight.export(&path.join("flight"))?;
        for (name, bytes) in files {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .map_err(|e| e.to_string())?;
            f.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        Ok(self.manifest.clone())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let read = |name: &str, max: u64| crate::habitat_recording::read(&path.join(name), max);
        let definition: Definition = serde_json::from_str(&read("definition.json", 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if definition.model != vehicle_arrival::MODEL {
            return Err("Unsupported approach model".into());
        }
        let flight = Bundle::read(&path.join("flight"))?;
        let report = Self::run(&flight, definition.target)?;
        let points: Vec<Point> =
            serde_json::from_str(&read("approach.json", MAX_BYTES)?).map_err(|e| e.to_string())?;
        let manifest: Value = serde_json::from_str(&read("manifest.json", 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if serde_json::to_value(&points).map_err(|e| e.to_string())?
            != serde_json::to_value(&report.points).map_err(|e| e.to_string())?
            || manifest != report.manifest
            || read("approach.csv", MAX_BYTES)? != csv(&report.points)
        {
            return Err(
                "Stored approach evidence differs from independently replayed complete source"
                    .into(),
            );
        }
        Ok(Self { points, ..report })
    }
}
pub fn csv(points: &[Point]) -> String {
    let mut out=String::from("world_time_s,craft_time_s,source_observed_world_time_s,historical,available,latitude_deg,longitude_east_deg,spherical_altitude_m,radial_velocity_m_s,east_velocity_m_s,north_velocity_m_s,horizontal_speed_m_s,surface_relative_speed_m_s,current_mass_kg,radial_kinetic_energy_j,horizontal_kinetic_energy_j,terrain_height_above_sphere_m,terrain_clearance_m,spin_rate_rad_s\n");
    for p in points {
        let values = if let Some(r) = &p.reading {
            vec![
                Some(p.world_time_s),
                Some(r.craft_time_s),
                Some(r.world_time_s),
                Some(if p.historical { 1. } else { 0. }),
                Some(1.),
                Some(r.latitude_deg),
                r.longitude_east_deg,
                Some(r.spherical_altitude_m),
                Some(r.radial_velocity_m_s),
                r.east_velocity_m_s,
                r.north_velocity_m_s,
                Some(r.horizontal_speed_m_s),
                Some(r.surface_relative_speed_m_s),
                Some(r.current_mass_kg),
                Some(r.radial_kinetic_energy_j),
                Some(r.horizontal_kinetic_energy_j),
                r.terrain_height_above_sphere_m,
                r.terrain_clearance_m,
                Some(r.frame.spin_rate_rad_s),
            ]
        } else {
            let mut v = vec![None; 19];
            v[0] = Some(p.world_time_s);
            v[3] = Some(if p.historical { 1. } else { 0. });
            v[4] = Some(0.);
            v
        };
        out.push_str(
            &values
                .into_iter()
                .map(|v| v.map(|x| format!("{x:.17}")).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        supply_vehicle::Plan, vehicle_arrival::GeographicStart, world_vehicle_recording::Study,
    };
    fn flight() -> Bundle {
        let u = crate::Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: u.systems[0].find_body("Moon").unwrap() as u32,
        };
        let mut plan = Plan::example();
        for c in &mut plan.commands {
            c.throttle = 0.;
        }
        let start = GeographicStart {
            spherical_altitude_m: 1.,
            up_velocity_m_s: -10.,
            ..Default::default()
        };
        let d = start.flight(&u, target, plan).unwrap();
        Bundle::run(&u, Study::new(&u, d, 5, 1.).unwrap()).unwrap()
    }
    #[test]
    fn replay_retains_original_contact_frame_and_protects_complete_evidence() {
        let flight = flight();
        let report = Report::run(&flight, flight.study.flight.parent).unwrap();
        assert_eq!(report.points.len(), 5);
        assert!(!report.points[1].historical);
        assert!(report.points[2].historical);
        let at = &report.points[1].reading.as_ref().unwrap();
        let later = &report.points[4].reading.as_ref().unwrap();
        assert_eq!(at, later);
        assert!(later.world_time_s < report.points[4].world_time_s);
        assert!(at.radial_velocity_m_s < 0.);
        assert!(at.radial_kinetic_energy_j > 0.);
        let root = std::env::temp_dir().join(format!("cosmogon-approach-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        report.export(&root).unwrap();
        assert!(report.export(&root).is_err());
        let opened = Report::read(&root).unwrap();
        assert_eq!(opened.points, report.points);
        assert_eq!(opened.manifest, report.manifest);
        let path = root.join("approach.json");
        let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        v[1]["reading"]["radial_velocity_m_s"] = json!(999.);
        std::fs::write(path, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
        assert!(Report::read(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn corrupted_flight_and_wrong_system_cannot_create_arrival_evidence() {
        let mut source = flight();
        let target = source.study.flight.parent;
        source.rows[1].flight.velocity_m_s.x += 1.;
        assert!(Report::run(&source, target).is_err());
        assert!(Report::run(
            &source,
            BodyRef {
                system: 99,
                body: 0
            }
        )
        .is_err());
    }
}
