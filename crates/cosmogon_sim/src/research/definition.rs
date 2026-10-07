//! Portable experiment definitions. Each run branches from the same frozen state.
use super::{
    parameter_edit, read_parameter, Experiment, ParameterId, ResearchTarget, MAX_RESEARCH_SAMPLES,
};
use crate::{save, Universe};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

pub const MAX_SWEEP_RUNS: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterOverride {
    pub target: ResearchTarget,
    pub parameter: ParameterId,
    /// Canonical descriptor units, not a display-unit choice.
    pub value: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterSweep {
    pub target: ResearchTarget,
    pub parameter: ParameterId,
    pub values: Vec<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentDefinition {
    pub format: String,
    pub schema_version: u32,
    pub name: String,
    pub measurement_target: ResearchTarget,
    pub samples: usize,
    pub interval_s: f64,
    #[serde(default)]
    pub initial_fingerprint: Option<String>,
    #[serde(default)]
    pub overrides: Vec<ParameterOverride>,
    #[serde(default)]
    pub sweep: Option<ParameterSweep>,
}
impl ExperimentDefinition {
    pub fn new(target: ResearchTarget) -> Self {
        Self {
            format: "cosmogon-experiment-definition".into(),
            schema_version: 1,
            name: "Exploratory experiment".into(),
            measurement_target: target,
            samples: 32,
            interval_s: 86400.0,
            initial_fingerprint: None,
            overrides: Vec::new(),
            sweep: None,
        }
    }
    pub fn runs(&self) -> usize {
        self.sweep.as_ref().map_or(1, |s| s.values.len())
    }
    pub fn validate(&self, initial: &Universe) -> Result<(), String> {
        if let Some(expected) = &self.initial_fingerprint {
            if *expected != super::state_fingerprint(initial)? {
                return Err(
                    "Definition belongs to a different frozen state; load its exported base.cosmo"
                        .into(),
                );
            }
        }
        if self.format != "cosmogon-experiment-definition" || self.schema_version != 1 {
            return Err("Unsupported experiment definition format/version".into());
        }
        if self.name.trim().is_empty() || self.name.len() > 256 {
            return Err("Choose an experiment name of 1–256 bytes".into());
        }
        if !(2..=MAX_RESEARCH_SAMPLES).contains(&self.samples)
            || !self.interval_s.is_finite()
            || !(0.0864..=315_576_000.0).contains(&self.interval_s)
        {
            return Err("Choose 2–4096 samples and 0.0864–315576000 seconds between them".into());
        }
        let end = initial.time + (self.samples - 1) as f64 * self.interval_s;
        if !initial.time.is_finite() || !end.is_finite() || end <= initial.time {
            return Err("Sample times are not representable at this epoch".into());
        }
        if !(1..=MAX_SWEEP_RUNS).contains(&self.runs()) || self.overrides.len() > 64 {
            return Err("Limit the definition to 64 runs and 64 overrides".into());
        }
        if let Some(s) = &self.sweep {
            if self
                .overrides
                .iter()
                .any(|o| o.target == s.target && o.parameter == s.parameter)
            {
                return Err("The swept input cannot also be a fixed override".into());
            }
        }
        // Validate complete edit sequences before starting a batch, including interactions.
        for run in 0..self.runs() {
            let branch = self.branch(initial, run)?;
            if self.measurement_target.name(&branch).is_none() {
                return Err("Measurement target is unavailable in the frozen state".into());
            }
            Experiment::new(&branch, self.measurement_target, self.samples)?;
        }
        Ok(())
    }
    pub fn branch(&self, initial: &Universe, run: usize) -> Result<Universe, String> {
        if run >= self.runs() {
            return Err("Run index is outside the definition".into());
        }
        let mut u = initial.clone();
        for (i, o) in self.overrides.iter().enumerate() {
            if self.overrides[..i]
                .iter()
                .any(|p| p.target == o.target && p.parameter == o.parameter)
            {
                return Err("Duplicate fixed override; specify each input once".into());
            }
            u.apply_edit(parameter_edit(&u, o.target, o.parameter, o.value)?)?;
        }
        if let Some(s) = &self.sweep {
            u.apply_edit(parameter_edit(&u, s.target, s.parameter, s.values[run])?)?;
        }
        Ok(u)
    }
    /// An exact ordered linear grid; no randomization or distribution is implied.
    pub fn linear_values(start: f64, end: f64, count: usize) -> Result<Vec<f64>, String> {
        if !start.is_finite() || !end.is_finite() || !(2..=MAX_SWEEP_RUNS).contains(&count) {
            return Err("Choose finite endpoints and 2–64 values".into());
        }
        let values: Vec<_> = (0..count)
            .map(|i| {
                if i == 0 {
                    start
                } else if i + 1 == count {
                    end
                } else {
                    start * (1.0 - i as f64 / (count - 1) as f64)
                        + end * (i as f64 / (count - 1) as f64)
                }
            })
            .collect();
        if values.iter().any(|v| !v.is_finite()) {
            return Err("Sweep range overflowed".into());
        }
        Ok(values)
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 1024 * 1024 {
            return Err("Definition exceeds 1 MiB".into());
        }
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }
    /// A reusable prepared study with an exact initial-state identity; no integration.
    pub fn save_bundle(&self, initial: &Universe, directory: &Path) -> Result<(), String> {
        if directory.exists() {
            return Err("Destination already exists; choose a new path".into());
        }
        let mut definition = self.clone();
        definition.initial_fingerprint = Some(super::state_fingerprint(initial)?);
        definition.validate(initial)?;
        if let Some(parent) = directory.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(directory).map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            write_new(
                &directory.join("definition.json"),
                &serde_json::to_vec_pretty(&definition).map_err(|e| e.to_string())?,
            )?;
            save::write_file(&directory.join("base.cosmo"), initial, &definition.name)
                .map_err(|e| e.to_string())?;
            write_batch_status(directory, &definition, initial, "prepared", None, &[])
        })();
        if result.is_err() {
            let _ = std::fs::remove_dir_all(directory);
        }
        result
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchRunSummary {
    pub run: usize,
    pub input_value: Option<f64>,
    pub directory: String,
    pub final_simulation_time_s: f64,
    /// Canonical unit keys align across runs; unavailable readings fail the run.
    pub final_values: std::collections::BTreeMap<String, f64>,
}
#[derive(Clone, Debug, Serialize)]
pub struct BatchProgress {
    pub completed_runs: usize,
    pub total_runs: usize,
    pub current_sample: usize,
    pub total_samples: usize,
}

/// Native and CLI use this same runner. The caller supplies cancellation and progress.
/// Completed runs survive cancellation/failure; unfinished runs are never called complete.
pub fn run_batch(
    initial: &Universe,
    definition: &ExperimentDefinition,
    directory: &Path,
    mut cancelled: impl FnMut() -> bool,
    mut progress: impl FnMut(BatchProgress),
) -> Result<Vec<BatchRunSummary>, String> {
    if directory.exists() {
        return Err("Output directory already exists; choose a new path".into());
    }
    definition.validate(initial)?;
    if cancelled() {
        return Err("Experiment cancelled before export".into());
    }
    if let Some(parent) = directory.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(directory).map_err(|e| e.to_string())?;
    let mut summaries = Vec::new();
    let result = (|| -> Result<(), String> {
        write_new(
            &directory.join("definition.json"),
            &serde_json::to_vec_pretty(definition).map_err(|e| e.to_string())?,
        )?;
        save::write_file(&directory.join("base.cosmo"), initial, &definition.name)
            .map_err(|e| e.to_string())?;
        write_batch_status(directory, definition, initial, "running", None, &summaries)?;
        for run in 0..definition.runs() {
            if cancelled() {
                return Err("Experiment cancelled; completed runs are retained".into());
            }
            let mut u = definition.branch(initial, run)?;
            let mut recording =
                Experiment::new(&u, definition.measurement_target, definition.samples)?;
            recording.sampling_policy = format!(
                "Definition runner: uniform requested interval {} seconds",
                definition.interval_s
            );
            let start = u.time;
            for sample in 1..definition.samples {
                let to = start + sample as f64 * definition.interval_s;
                loop {
                    if cancelled() {
                        return Err("Experiment cancelled; completed runs are retained".into());
                    }
                    let report = u.advance_to(to, Some(Duration::from_millis(10)), None);
                    progress(BatchProgress {
                        completed_runs: run,
                        total_runs: definition.runs(),
                        current_sample: sample,
                        total_samples: definition.samples,
                    });
                    if !report.lagging {
                        break;
                    }
                }
                if u.time != to {
                    return Err("Simulation did not reach the requested sample time".into());
                }
                if !recording.capture(&u)? {
                    return Err("The requested sample interval produced no distinct state".into());
                }
            }
            recording.stop_reason = Some("Definition sample count reached".into());
            let name = format!("run-{:04}", run + 1);
            recording.export(&directory.join(&name))?;
            let final_values = recording
                .parameters
                .iter()
                .filter_map(|p| {
                    read_parameter(&u, definition.measurement_target, p.descriptor.id)
                        .map(|v| (p.descriptor.key.to_string(), v.value))
                })
                .collect();
            summaries.push(BatchRunSummary {
                run: run + 1,
                input_value: definition.sweep.as_ref().map(|s| s.values[run]),
                directory: name,
                final_simulation_time_s: u.time,
                final_values,
            });
            write_batch_status(directory, definition, initial, "running", None, &summaries)?;
            progress(BatchProgress {
                completed_runs: run + 1,
                total_runs: definition.runs(),
                current_sample: definition.samples,
                total_samples: definition.samples,
            });
        }
        Ok(())
    })();
    let status = if result.is_ok() {
        "complete"
    } else {
        "incomplete"
    };
    let error = result.as_ref().err().map(String::as_str);
    write_batch_status(directory, definition, initial, status, error, &summaries)?;
    result.map(|()| summaries)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(bytes)
        .and_then(|()| f.sync_all())
        .map_err(|e| e.to_string())
}
fn write_batch_status(
    dir: &Path,
    d: &ExperimentDefinition,
    initial: &Universe,
    status: &str,
    error: Option<&str>,
    runs: &[BatchRunSummary],
) -> Result<(), String> {
    let m = serde_json::json!({"format":"cosmogon-experiment-batch", "schema_version":1,
        "name":d.name,"status":status,"error":error,"completed_runs":runs.len(),"requested_runs":d.runs(),
        "initial_header":save::header_for(initial,&d.name),"runs":runs,
        "initial_state_fingerprint":super::state_fingerprint(initial)?,
        "files":["definition.json","base.cosmo"],
        "limitations":["Exploratory implemented models, not independent scientific validation.","Every run branches from base.cosmo; overrides precede recording. No cross-run state carryover.","Input values use canonical parameter units; no uncertainty propagation or randomized distributions."]});
    let temp = dir.join("batch.json.tmp");
    write_new(
        &temp,
        &serde_json::to_vec_pretty(&m).map_err(|e| e.to_string())?,
    )?;
    std::fs::rename(&temp, dir.join("batch.json")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyRef, Scenario, UniverseSettings};
    fn lab() -> Universe {
        Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        })
    }
    fn earth() -> ResearchTarget {
        ResearchTarget::Body(BodyRef { system: 0, body: 2 })
    }
    fn path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "cosmogon-batch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn branches_are_isolated_and_inputs_really_change_outputs() {
        let u = lab();
        let before = serde_json::to_string(&u).unwrap();
        let mut d = ExperimentDefinition::new(earth());
        let radius = read_parameter(&u, earth(), ParameterId::Radius)
            .unwrap()
            .value;
        d.sweep = Some(ParameterSweep {
            target: earth(),
            parameter: ParameterId::Radius,
            values: vec![radius, radius * 2.0],
        });
        d.validate(&u).unwrap();
        let a = d.branch(&u, 0).unwrap();
        let b = d.branch(&u, 1).unwrap();
        assert_eq!(
            read_parameter(&a, earth(), ParameterId::Gravity)
                .unwrap()
                .value
                / 4.0,
            read_parameter(&b, earth(), ParameterId::Gravity)
                .unwrap()
                .value
        );
        assert_eq!(serde_json::to_string(&u).unwrap(), before);
        assert_eq!(a.edits.len(), u.edits.len() + 1);
        assert_eq!(b.edits.len(), a.edits.len());
    }
    #[test]
    fn definitions_reject_invalid_inputs_and_versions_before_files_are_created() {
        let u = lab();
        let mut d = ExperimentDefinition::new(earth());
        d.initial_fingerprint = Some(super::super::state_fingerprint(&u).unwrap());
        let mut changed = u.clone();
        changed.advance_by(1.0);
        assert!(d.validate(&changed).is_err());
        d.initial_fingerprint = None;
        let out = path();
        d.sweep = Some(ParameterSweep {
            target: earth(),
            parameter: ParameterId::Gravity,
            values: vec![1.0, 2.0],
        });
        assert!(run_batch(&u, &d, &out, || false, |_| {}).is_err());
        assert!(!out.exists());
        d.sweep = None;
        d.interval_s = f64::NAN;
        assert!(d.validate(&u).is_err());
        d.interval_s = 1.0;
        d.schema_version = 2;
        assert!(d.validate(&u).is_err());
        assert_eq!(
            ExperimentDefinition::linear_values(3.0, 1.0, 3).unwrap(),
            vec![3.0, 2.0, 1.0]
        );
        assert!(ExperimentDefinition::linear_values(0.0, 1.0, 65).is_err());
        let bad = r#"{"format":"cosmogon-experiment-definition","schema_version":1,"name":"X","measurement_target":{"kind":"star","id":0},"samples":2,"interval_s":1,"mystery":true}"#;
        assert!(serde_json::from_str::<ExperimentDefinition>(bad).is_err());
    }
    #[test]
    fn batch_replays_exactly_and_cancel_preserves_only_completed_runs() {
        let u = lab();
        let mut d = ExperimentDefinition::new(earth());
        d.samples = 3;
        d.interval_s = 60.0;
        d.sweep = Some(ParameterSweep {
            target: earth(),
            parameter: ParameterId::Albedo,
            values: vec![0.2, 0.4],
        });
        let out = path();
        let runs = run_batch(&u, &d, &out, || false, |_| {}).unwrap();
        assert_eq!(runs.len(), 2);
        let base = save::read_file(&out.join("base.cosmo")).unwrap();
        let plan = ExperimentDefinition::read(&out.join("definition.json")).unwrap();
        let replay = path();
        run_batch(&base, &plan, &replay, || false, |_| {}).unwrap();
        for n in 1..=2 {
            assert_eq!(
                std::fs::read(out.join(format!("run-{n:04}/samples.csv"))).unwrap(),
                std::fs::read(replay.join(format!("run-{n:04}/samples.csv"))).unwrap()
            );
        }
        assert!(run_batch(&u, &d, &out, || false, |_| {}).is_err());
        let prepared = path();
        d.save_bundle(&u, &prepared).unwrap();
        assert!(d.save_bundle(&u, &prepared).is_err());
        let saved = ExperimentDefinition::read(&prepared.join("definition.json")).unwrap();
        assert!(saved.initial_fingerprint.is_some());
        let stop = std::cell::Cell::new(false);
        let partial = path();
        assert!(run_batch(
            &u,
            &d,
            &partial,
            || stop.get(),
            |p| {
                if p.completed_runs == 1 {
                    stop.set(true);
                }
            }
        )
        .is_err());
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(partial.join("batch.json")).unwrap()).unwrap();
        assert_eq!(manifest["status"], "incomplete");
        assert_eq!(manifest["completed_runs"], 1);
        assert!(!partial.join("run-0002").exists());
        for p in [out, replay, partial, prepared] {
            std::fs::remove_dir_all(p).unwrap();
        }
    }
}
