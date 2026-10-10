//! Independently replayed footprint/support evidence; original flight files remain unchanged.
use crate::{
    research,
    sandbox::Edit,
    vehicle_support::{self, Definition, Reading},
    world_vehicle::Phase,
    world_vehicle_recording::Bundle,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::Write, path::Path};
const MAX_BYTES: u64 = 32 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub world_time_s: f64,
    pub reading: Option<Reading>,
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
    pub fn run(flight: &Bundle, definition: Definition) -> Result<Self, String> {
        definition.validate()?;
        if flight.study.flight.contact.is_none() {
            return Err("Footprint support requires an explicit terrain-contact flight".into());
        }
        flight.study.validate(&flight.origin)?;
        let replay = Bundle::run(&flight.origin, flight.study.clone())?;
        if replay.manifest != flight.manifest
            || serde_json::to_value(&replay.rows).map_err(|e| e.to_string())?
                != serde_json::to_value(&flight.rows).map_err(|e| e.to_string())?
            || research::state_fingerprint(&replay.final_world)?
                != research::state_fingerprint(&flight.final_world)?
        {
            return Err(
                "Support source is not the complete independently reproduced flight".into(),
            );
        }

        if flight.rows.len() != flight.study.samples {
            return Err("Flight source has incomplete observations".into());
        }
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
                .ok_or("Support craft disappeared")?;
            if world.time != time
                || row.world_time_s != time
                || serde_json::to_value(craft).map_err(|e| e.to_string())?
                    != serde_json::to_value(&row.flight).map_err(|e| e.to_string())?
            {
                return Err(format!("Support source replay differs at observation {i}"));
            }
            let mut point = if let Some(saved) = &terminal {
                let mut p = saved.clone();
                p.historical = true;
                p
            } else {
                let reading = match &craft.phase {
                    Phase::Unsupported { reason } => Err(format!(
                        "Stopped flight lacks supported footprint geometry: {reason}"
                    )),
                    _ => Reading::live(&world, craft, &definition),
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
            return Err("Support source replay differs from the complete final world".into());
        }
        let mut manifest = json!({"format":"cosmogon-vehicle-support-recording","schema_version":1,"complete":true,"definition":definition,"flight_manifest":flight.manifest,"observations":points.len(),"context":vehicle_support::context(),"files":["definition.json","support.json","support.csv","flight/definition.json","flight/origin.cosmo","flight/observations.json","flight/trajectories.csv","flight/final.cosmo","flight/manifest.json"]});
        if flight.study.flight.contact.is_some() {
            manifest["files"]
                .as_array_mut()
                .unwrap()
                .push(json!("flight/contact.csv"));
        }
        if flight
            .study
            .flight
            .contact
            .as_ref()
            .is_some_and(|h| h.shear.is_some())
        {
            manifest["files"]
                .as_array_mut()
                .unwrap()
                .push(json!("flight/shear.csv"));
        }
        Ok(Self {
            definition,
            flight: flight.clone(),
            points,
            manifest,
        })
    }
    pub fn export(&self, path: &Path) -> Result<Value, String> {
        if path.exists() {
            return Err("Existing support output is protected; choose a fresh folder".into());
        }
        let verified = Self::run(&self.flight, self.definition.clone())?;
        if self.definition != verified.definition
            || serde_json::to_value(&self.points).map_err(|e| e.to_string())?
                != serde_json::to_value(&verified.points).map_err(|e| e.to_string())?
            || self.manifest != verified.manifest
        {
            return Err("Support evidence differs from independently replayed source".into());
        }
        let files = [
            (
                "definition.json",
                serde_json::to_vec_pretty(&self.definition).map_err(|e| e.to_string())?,
            ),
            (
                "support.json",
                serde_json::to_vec_pretty(&self.points).map_err(|e| e.to_string())?,
            ),
            ("support.csv", csv(&self.points).into_bytes()),
            (
                "manifest.json",
                serde_json::to_vec_pretty(&self.manifest).map_err(|e| e.to_string())?,
            ),
        ];
        if files.iter().any(|(_, b)| b.len() as u64 > MAX_BYTES) {
            return Err("Support export exceeds bounded file size".into());
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
        if definition.model != vehicle_support::MODEL {
            return Err("Unsupported support model".into());
        }
        let flight = Bundle::read(&path.join("flight"))?;
        let report = Self::run(&flight, definition)?;
        let points: Vec<Point> =
            serde_json::from_str(&read("support.json", MAX_BYTES)?).map_err(|e| e.to_string())?;
        let manifest: Value = serde_json::from_str(&read("manifest.json", 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if serde_json::to_value(&points).map_err(|e| e.to_string())?
            != serde_json::to_value(&report.points).map_err(|e| e.to_string())?
            || manifest != report.manifest
            || read("support.csv", MAX_BYTES)? != csv(&report.points)
        {
            return Err(
                "Stored support evidence differs from independently replayed complete source"
                    .into(),
            );
        }
        Ok(Self { points, ..report })
    }
}

pub fn csv(points: &[Point]) -> String {
    let mut out=String::from("world_time_s,craft_time_s,source_observed_world_time_s,historical,available,loaded_feet,mass_kg,relative_speed_m_s,relative_angular_speed_rad_s,contact_force_x_n,contact_force_y_n,contact_force_z_n,contact_torque_x_n_m,contact_torque_y_n_m,contact_torque_z_n_m,position_roundoff_indicator_m,plane_available,hull_area_m2,centre_projection_margin_m,weighted_site_margin_m,wrench_centre_margin_m,normal_torque_n_m,hold_available,load_line_margin_m,force_imbalance_n,torque_imbalance_n_m,relative_acceleration_m_s2,derivative_error_indicator_m_s2,within_declared_instantaneous_envelope\n");
    for p in points {
        let mut values = vec![None; 29];
        values[0] = Some(p.world_time_s);
        values[3] = Some(if p.historical { 1. } else { 0. });
        values[4] = Some(if p.reading.is_some() { 1. } else { 0. });
        if let Some(r) = &p.reading {
            values[1] = Some(r.craft_time_s);
            values[2] = Some(r.world_time_s);
            values[5] = Some(r.feet.iter().filter(|f| f.loaded).count() as f64);
            values[6] = Some(r.mass_kg);
            values[7] = Some(r.relative_velocity_m_s.length());
            values[8] = Some(r.relative_angular_velocity_rad_s.length());
            for (i, v) in [
                r.contact_force_n.x,
                r.contact_force_n.y,
                r.contact_force_n.z,
                r.contact_torque_about_centre_n_m.x,
                r.contact_torque_about_centre_n_m.y,
                r.contact_torque_about_centre_n_m.z,
                r.position_roundoff_indicator_m,
            ]
            .into_iter()
            .enumerate()
            {
                values[9 + i] = Some(v);
            }
            values[16] = Some(if r.planar.is_some() { 1. } else { 0. });
            values[22] = Some(if r.hold.is_some() { 1. } else { 0. });
            values[28] = Some(if r.within_declared_instantaneous_envelope {
                1.
            } else {
                0.
            });
            if let Some(a) = &r.planar {
                for (i, v) in [
                    a.hull_area_m2,
                    a.geometric_centre_margin_m,
                    a.force_weighted_site_margin_m,
                    a.wrench_centre_margin_m,
                    a.residual_normal_torque_n_m,
                ]
                .into_iter()
                .enumerate()
                {
                    values[17 + i] = Some(v);
                }
            }
            if let Some(a) = &r.hold {
                values[23] = a.load_line_margin_m;
                values[24] = Some(a.force_imbalance_n.length());
                values[25] = Some(a.torque_imbalance_n_m.length());
                values[26] = Some(a.relative_acceleration_m_s2.length());
                values[27] = Some(a.derivative_error_indicator_m_s2);
            }
        }
        out.push_str(
            &values
                .into_iter()
                .map(|v| v.map(|v| v.to_string()).unwrap_or_default())
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
    use crate::world_vehicle_recording::Study;
    fn flight() -> Bundle {
        let (world, d) = crate::vehicle_support::fixture();
        Bundle::run(&world, Study::new(&world, d, 9, 0.5).unwrap()).unwrap()
    }
    #[test]
    fn full_original_branch_and_typed_support_exports_replay_and_refuse_corruption() {
        let b = flight();
        let original = b.manifest.clone();
        let r = Report::run(&b, Definition::default()).unwrap();
        assert_eq!(r.points.len(), 9);
        assert_eq!(r.flight.manifest, original);
        assert!(r
            .points
            .last()
            .unwrap()
            .reading
            .as_ref()
            .unwrap()
            .planar
            .is_some());
        let path = std::env::temp_dir().join(format!("cosmogon-support-{}", std::process::id()));
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }
        r.export(&path).unwrap();
        assert!(r.export(&path).is_err());
        let opened = Report::read(&path).unwrap();
        assert_eq!(opened.points, r.points);
        assert_eq!(opened.manifest, r.manifest);
        let csv = csv(&r.points);
        let width = csv.lines().next().unwrap().split(',').count();
        assert!(csv.lines().skip(1).all(|l| l.split(',').count() == width));
        let p = path.join("support.json");
        let mut data: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        data[8]["reading"]["within_declared_instantaneous_envelope"] = json!(true);
        data[8]["reading"]["contact_force_n"]["x"] = json!(999999.);
        std::fs::write(&p, serde_json::to_vec_pretty(&data).unwrap()).unwrap();
        assert!(Report::read(&path).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn stopped_contact_history_is_unavailable_and_incomplete_branch_is_refused() {
        let (world, mut d) = crate::vehicle_support::fixture();
        for f in &mut d.contact.as_mut().unwrap().feet {
            f.response.maximum_load_n = 1.;
        }
        let b = Bundle::run(&world, Study::new(&world, d, 5, 1.).unwrap()).unwrap();
        let report = Report::run(&b, Definition::default()).unwrap();
        assert!(report.points[1].reading.is_none());
        assert!(!report.points[1].historical);
        assert!(report.points[2].historical);
        assert_eq!(
            report.points[1].unavailable_reason,
            report.points[4].unavailable_reason
        );
        let mut changed = flight();
        changed.rows[3].flight.position_m.x += 1.;
        assert!(Report::run(&changed, Definition::default()).is_err());
    }
}
