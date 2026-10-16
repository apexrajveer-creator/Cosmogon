//! Independently replayed craft-to-craft rendezvous evidence between two
//! complete flight recordings that share one exact origin world and one exact
//! observation grid. Evidence is the closest retained shared sample, never a
//! continuous closest approach, intercept, docking or stationkeeping claim.

use crate::{
    research,
    vehicle_orbit,
    world_vehicle_recording::Bundle,
    world_vehicle_recording::Observation,
    Vec3d,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::Write, path::Path};

const MAX_BYTES: u64 = 32 * 1024 * 1024;
pub const MODEL: &str = "cosmogon-vehicle-rendezvous/1";
const FORMAT: &str = "cosmogon-vehicle-rendezvous-recording";
pub const SCOPE: &str =
    "Closest approach among the exactly shared retained observation times of two complete independently replay-verified flight recordings from the same origin world; sample-grid evidence only, never the continuous closest approach and never an intercept, docking or stationkeeping claim";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub origin_fingerprint: String,
    pub left_final_fingerprint: String,
    pub right_final_fingerprint: String,
    pub system: u32,
}

/// One shared-observation relative state between the two retained craft.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub world_time_s: f64,
    pub distance_m: f64,
    pub relative_speed_m_s: f64,
    pub relative_position_m: [f64; 3],
}

/// Closest shared-sample evidence with its explicit sample-grid scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub common_samples: usize,
    pub closest_index: usize,
    pub closest_world_time_s: f64,
    pub closest_distance_m: f64,
    pub closest_relative_speed_m_s: f64,
    pub closest_relative_position_m: [f64; 3],
    pub scope: String,
}

/// Exact deterministic CSV of the retained shared relative states.
pub fn csv(samples: &[Sample]) -> String {
    let mut output = String::from("observation_index,world_time_s,distance_m,relative_speed_m_s,relative_position_x_m,relative_position_y_m,relative_position_z_m\n");
    for (index, sample) in samples.iter().enumerate() {
        output.push_str(
            &[
                index.to_string(),
                sample.world_time_s.to_string(),
                sample.distance_m.to_string(),
                sample.relative_speed_m_s.to_string(),
                sample.relative_position_m[0].to_string(),
                sample.relative_position_m[1].to_string(),
                sample.relative_position_m[2].to_string(),
            ]
            .join(","),
        );
        output.push('\n');
    }
    output
}

/// Relative states over the exactly shared finite observation grid, then the
/// closest shared sample; equal distances keep the earliest observation.
pub(crate) fn grid(
    left: &[(f64, Vec3d, Vec3d)],
    right: &[(f64, Vec3d, Vec3d)],
) -> Result<(Evidence, Vec<Sample>), String> {
    if left.len() != right.len() {
        return Err(
            "Rendezvous requires both recordings to share one exact observation grid".into(),
        );
    }
    if left.is_empty() {
        return Err("Rendezvous requires at least one shared observation".into());
    }
    let mut samples = Vec::with_capacity(left.len());
    for ((time_a, pos_a, vel_a), (time_b, pos_b, vel_b)) in left.iter().zip(right) {
        if !time_a.is_finite() || time_a != time_b {
            return Err(
                "Rendezvous requires exactly equal finite observation times on both grids".into(),
            );
        }
        let delta = *pos_b - *pos_a;
        let relative = *vel_b - *vel_a;
        let distance = delta.length();
        let speed = relative.length();
        if !distance.is_finite() || !speed.is_finite() {
            return Err("Rendezvous relative distances and speeds must be finite".into());
        }
        samples.push(Sample {
            world_time_s: *time_a,
            distance_m: distance,
            relative_speed_m_s: speed,
            relative_position_m: [delta.x, delta.y, delta.z],
        });
    }
    let mut closest_index = 0;
    for (index, sample) in samples.iter().enumerate() {
        if sample.distance_m < samples[closest_index].distance_m {
            closest_index = index;
        }
    }
    let closest = samples[closest_index];
    Ok((
        Evidence {
            common_samples: samples.len(),
            closest_index,
            closest_world_time_s: closest.world_time_s,
            closest_distance_m: closest.distance_m,
            closest_relative_speed_m_s: closest.relative_speed_m_s,
            closest_relative_position_m: closest.relative_position_m,
            scope: SCOPE.into(),
        },
        samples,
    ))
}

pub fn compute(
    left: &[Observation],
    right: &[Observation],
) -> Result<(Evidence, Vec<Sample>), String> {
    let left_grid: Vec<(f64, Vec3d, Vec3d)> = left
        .iter()
        .map(|o| (o.world_time_s, o.flight.position_m, o.flight.velocity_m_s))
        .collect();
    let right_grid: Vec<(f64, Vec3d, Vec3d)> = right
        .iter()
        .map(|o| (o.world_time_s, o.flight.position_m, o.flight.velocity_m_s))
        .collect();
    grid(&left_grid, &right_grid)
}

#[derive(Clone, Debug)]
pub struct Report {
    pub definition: Definition,
    pub left: Bundle,
    pub right: Bundle,
    pub evidence: Evidence,
    pub samples: Vec<Sample>,
}

impl Report {
    pub fn run(left: &Bundle, right: &Bundle) -> Result<Self, String> {
        left.study.validate(&left.origin)?;
        right.study.validate(&right.origin)?;
        let left_replay = Bundle::run(&left.origin, left.study.clone())?;
        let right_replay = Bundle::run(&right.origin, right.study.clone())?;
        if left_replay.manifest != left.manifest || right_replay.manifest != right.manifest {
            return Err(
                "Rendezvous sources are not complete independently reproduced flights".into(),
            );
        }
        let origin_left = research::state_fingerprint(&left.origin)?;
        let origin_right = research::state_fingerprint(&right.origin)?;
        if origin_left != origin_right {
            return Err(
                "Rendezvous requires both flight recordings to share the same exact origin world"
                    .into(),
            );
        }
        if left.study.flight.parent.system != right.study.flight.parent.system {
            return Err("Rendezvous requires both craft to fly in the same system".into());
        }
        let left_final = left.manifest["final_fingerprint"]
            .as_str()
            .ok_or("Left flight manifest lacks a final fingerprint")?
            .to_string();
        let right_final = right.manifest["final_fingerprint"]
            .as_str()
            .ok_or("Right flight manifest lacks a final fingerprint")?
            .to_string();
        let (evidence, samples) = compute(&left.rows, &right.rows)?;
        Ok(Self {
            definition: Definition {
                model: MODEL.into(),
                origin_fingerprint: origin_left,
                left_final_fingerprint: left_final,
                right_final_fingerprint: right_final,
                system: left.study.flight.parent.system,
            },
            left: left.clone(),
            right: right.clone(),
            evidence,
            samples,
        })
    }

    pub fn manifest(&self) -> Value {
        json!({
            "format": FORMAT,
            "schema_version": 1,
            "complete": true,
            "definition": self.definition,
            "evidence": self.evidence,
            "left_manifest": self.left.manifest,
            "right_manifest": self.right.manifest,
            "context": vehicle_orbit::context(),
            "files": ["definition.json", "rendezvous.json", "rendezvous.csv"],
        })
    }

    pub fn export(&self, path: &Path) -> Result<Value, String> {
        if path.exists() {
            return Err("Existing rendezvous output is protected; choose a fresh folder".into());
        }
        let verified = Self::run(&self.left, &self.right)?;
        if verified.definition != self.definition
            || verified.evidence != self.evidence
            || verified.samples != self.samples
        {
            return Err("Rendezvous evidence differs from independently replayed sources".into());
        }
        let files = [
            (
                "definition.json",
                serde_json::to_vec_pretty(&self.definition).map_err(|e| e.to_string())?,
            ),
            (
                "rendezvous.json",
                serde_json::to_vec_pretty(&self.manifest()).map_err(|e| e.to_string())?,
            ),
            ("rendezvous.csv", csv(&self.samples).into_bytes()),
        ];
        if files.iter().any(|(_, b)| b.len() as u64 > MAX_BYTES) {
            return Err("Rendezvous export exceeds bounded file size".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| e.to_string())?;
        for (name, bytes) in files {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        Ok(self.manifest())
    }

    pub fn read(left_path: &Path, right_path: &Path, path: &Path) -> Result<Self, String> {
        let left = Bundle::read(left_path)?;
        let right = Bundle::read(right_path)?;
        Self::read_from_bundles(&left, &right, path)
    }

    pub fn read_from_bundles(
        left: &Bundle,
        right: &Bundle,
        path: &Path,
    ) -> Result<Self, String> {
        let read = |name: &str, max: u64| crate::habitat_recording::read(&path.join(name), max);
        let definition: Definition = serde_json::from_str(&read("definition.json", 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if definition.model != MODEL {
            return Err("Unsupported rendezvous model".into());
        }
        let manifest: Value = serde_json::from_str(&read("rendezvous.json", 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        if manifest["format"] != FORMAT || manifest["schema_version"] != 1 || manifest["complete"] != true {
            return Err("Rendezvous recording is incomplete or unsupported".into());
        }
        let report = Self::run(left, right)?;
        if definition != report.definition
            || manifest != report.manifest()
            || read("rendezvous.csv", MAX_BYTES)? != csv(&report.samples)
        {
            return Err(
                "Stored rendezvous evidence differs from independently replayed complete sources"
                    .into(),
            );
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        supply_vehicle::{Command, Plan},
        world_vehicle::Definition,
        BodyRef,
    };

    fn coast_plan() -> Plan {
        let mut plan = Plan::example();
        plan.commands = vec![Command {
            duration_s: 10.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        plan
    }

    fn shared_origin_bundles() -> (crate::Universe, Bundle, Bundle) {
        let u = crate::Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = BodyRef { system: 0, body: 3 };
        let low = Definition::circular_example(&u, parent, coast_plan()).unwrap();
        let high = Definition {
            initial_relative_position_m: low.initial_relative_position_m * 1.02,
            initial_relative_velocity_m_s: low.initial_relative_velocity_m_s * 0.99,
            ..low.clone()
        };
        let study_low = crate::world_vehicle_recording::Study::new(&u, low, 9, 0.5).unwrap();
        let study_high = crate::world_vehicle_recording::Study::new(&u, high, 9, 0.5).unwrap();
        let left = Bundle::run(&u, study_low).unwrap();
        let right = Bundle::run(&u, study_high).unwrap();
        (u, left, right)
    }

    #[test]
    fn grid_finds_the_closest_shared_sample_and_keeps_earliest_tie() {
        let left = vec![
            (0.0, Vec3d::new(0., 0., 0.), Vec3d::ZERO),
            (1.0, Vec3d::new(0., 0., 0.), Vec3d::ZERO),
            (2.0, Vec3d::new(0., 0., 0.), Vec3d::ZERO),
        ];
        let right = vec![
            (0.0, Vec3d::new(10., 0., 0.), Vec3d::ZERO),
            (1.0, Vec3d::new(3., 4., 0.), Vec3d::new(1., 0., 0.)),
            (2.0, Vec3d::new(10., 0., 0.), Vec3d::ZERO),
        ];
        let (evidence, samples) = grid(&left, &right).unwrap();
        assert_eq!(evidence.common_samples, 3);
        assert_eq!(evidence.closest_index, 1);
        assert_eq!(evidence.closest_world_time_s, 1.0);
        assert_eq!(evidence.closest_distance_m, 5.0);
        assert_eq!(evidence.closest_relative_speed_m_s, 1.0);
        assert_eq!(evidence.closest_relative_position_m, [3.0, 4.0, 0.0]);
        assert_eq!(samples.len(), 3);
        assert_eq!(evidence, grid(&left, &right).unwrap().0);
        let tied = vec![
            (0.0, Vec3d::ZERO, Vec3d::ZERO),
            (1.0, Vec3d::ZERO, Vec3d::ZERO),
        ];
        let tied_right = vec![
            (0.0, Vec3d::new(5., 0., 0.), Vec3d::ZERO),
            (1.0, Vec3d::new(5., 0., 0.), Vec3d::ZERO),
        ];
        assert_eq!(grid(&tied, &tied_right).unwrap().0.closest_index, 0);
    }

    #[test]
    fn grid_refuses_mismatched_lengths_times_and_empty_inputs() {
        let one = vec![(0.0, Vec3d::ZERO, Vec3d::ZERO)];
        let two = vec![
            (0.0, Vec3d::ZERO, Vec3d::ZERO),
            (1.0, Vec3d::ZERO, Vec3d::ZERO),
        ];
        assert!(grid(&one, &two)
            .unwrap_err()
            .contains("one exact observation grid"));
        assert!(grid(&one, &[])
            .unwrap_err()
            .contains("one exact observation grid"));
        assert!(grid(&[], &[]).unwrap_err().contains("at least one"));
        let shifted = vec![(1.0, Vec3d::ZERO, Vec3d::ZERO)];
        assert!(grid(&one, &shifted)
            .unwrap_err()
            .contains("exactly equal finite observation times"));
        let infinite = vec![(f64::INFINITY, Vec3d::ZERO, Vec3d::ZERO)];
        assert!(grid(&infinite, &infinite).is_err());
        let nan = vec![(0.0, Vec3d::new(f64::NAN, 0., 0.), Vec3d::ZERO)];
        assert!(grid(&nan, &nan)
            .unwrap_err()
            .contains("distances and speeds must be finite"));
    }

    #[test]
    fn shared_origin_flights_replay_rendezvous_and_refuse_tampering() {
        let (_u, left, right) = shared_origin_bundles();
        let report = Report::run(&left, &right).unwrap();
        assert_eq!(report.evidence.common_samples, left.rows.len());
        assert_eq!(report.definition.system, 0);
        assert!(report.evidence.closest_distance_m.is_finite());
        assert!(report.evidence.closest_distance_m > 0.);
        let rerun = Report::run(&left, &right).unwrap();
        assert_eq!(report.evidence, rerun.evidence);
        assert_eq!(report.definition, rerun.definition);
        assert_eq!(report.samples, rerun.samples);
        assert_eq!(
            serde_json::to_value(report.manifest()).unwrap(),
            serde_json::to_value(rerun.manifest()).unwrap()
        );

        let root = std::env::temp_dir().join(format!(
            "cosmogon-rendezvous-recording-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        report.export(&root).unwrap();
        let opened = Report::read_from_bundles(&left, &right, &root).unwrap();
        assert_eq!(opened.evidence, report.evidence);
        assert_eq!(opened.definition, report.definition);

        let mut tampered_csv = std::fs::read_to_string(root.join("rendezvous.csv")).unwrap();
        tampered_csv = tampered_csv.replacen("observation_index", "tampered_index", 1);
        std::fs::write(root.join("rendezvous.csv"), tampered_csv).unwrap();
        assert!(Report::read_from_bundles(&left, &right, &root)
            .unwrap_err()
            .contains("differs from independently replayed"));
        let _ = std::fs::remove_dir_all(&root);

        let mut other_origin = right.clone();
        other_origin.origin.time += 1.;
        assert!(Report::run(&left, &other_origin)
            .unwrap_err()
            .contains("retained pin"));
    }
}
