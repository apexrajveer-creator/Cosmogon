//! Bounded observer recordings and self-contained local experiment exports.
use super::{
    ParameterDescriptor, ParameterId, PhysicsModelRegistry, Reading, ResearchTarget, PARAMETERS,
    SCIENTIFIC_SOURCES,
};
use crate::history::Event;
use crate::sandbox::EditRecord;
use crate::{save, Universe};
use serde::Serialize;
use std::io::Write;
use std::path::Path;

pub const MAX_RESEARCH_SAMPLES: usize = 4096;
const MAX_SNAPSHOT_BYTES: usize = 32 * 1024 * 1024;

/// Stable state identity, matching the existing golden-test algorithm. Not cryptographic.
pub fn state_fingerprint(u: &Universe) -> Result<String, String> {
    struct HashWriter {
        hash: u64,
        bytes: usize,
    }
    impl Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self.bytes.saturating_add(bytes.len());
            if self.bytes > MAX_SNAPSHOT_BYTES {
                return Err(std::io::Error::other(
                    "State exceeds the 32 MiB research limit",
                ));
            }
            for b in bytes {
                self.hash = (self.hash ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter {
        hash: 0xcbf2_9ce4_8422_2325,
        bytes: 0,
    };
    serde_json::to_writer(&mut writer, u).map_err(|e| e.to_string())?;
    Ok(format!("fnv1a64:{:016x}", writer.hash))
}

struct BoundedBytes(Vec<u8>);
impl Write for BoundedBytes {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(buf.len()) > MAX_SNAPSHOT_BYTES {
            return Err(std::io::Error::other(
                "Initial state exceeds the 32 MiB recording limit",
            ));
        }
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Serialize)]
struct InitialSave<'a> {
    header: save::SaveHeader,
    universe: &'a Universe,
}
#[derive(Clone, Debug, Serialize)]
pub struct ResearchSample {
    pub simulation_time_s: f64,
    pub edit_count: usize,
    pub values: Vec<f64>,
}
#[derive(Clone, Serialize)]
pub struct RecordedParameter {
    pub descriptor: ParameterDescriptor,
    pub initial: Reading,
}

pub struct Experiment {
    pub target: ResearchTarget,
    pub target_name: String,
    pub parameters: Vec<RecordedParameter>,
    pub samples: Vec<ResearchSample>,
    pub events: Vec<Event>,
    pub stop_reason: Option<String>,
    pub sampling_policy: String,
    pub limit: usize,
    pub initial_header: save::SaveHeader,
    pub initial_fingerprint: String,
    initial_save: Vec<u8>,
    edit_offset: usize,
    pub edits: Vec<EditRecord>,
    last_event: Option<Event>,
}
impl Experiment {
    pub fn new(u: &Universe, target: ResearchTarget, limit: usize) -> Result<Self, String> {
        let target_name = target
            .name(u)
            .ok_or("Select an existing object")?
            .to_string();
        let parameters: Vec<_> = PARAMETERS
            .iter()
            .filter_map(|d| {
                super::read_parameter(u, target, d.id).map(|initial| RecordedParameter {
                    descriptor: d.clone(),
                    initial,
                })
            })
            .collect();
        if parameters.is_empty() {
            return Err("No supported research readings for this object".into());
        }
        let header = save::header_for(u, "Research initial state");
        let mut snapshot = BoundedBytes(Vec::new());
        serde_json::to_writer(
            &mut snapshot,
            &InitialSave {
                header: header.clone(),
                universe: u,
            },
        )
        .map_err(|e| e.to_string())?;
        let mut out = Self {
            target,
            target_name,
            parameters,
            samples: Vec::new(),
            events: Vec::new(),
            stop_reason: None,
            sampling_policy: "Observed states at explicit simulation times; caller-defined cadence"
                .into(),
            limit: limit.clamp(2, MAX_RESEARCH_SAMPLES),
            initial_header: header,
            initial_fingerprint: state_fingerprint(u)?,
            initial_save: snapshot.0,
            edit_offset: u.edits.len(),
            edits: Vec::new(),
            last_event: u.history.events.last().cloned(),
        };
        out.capture(u)?;
        Ok(out)
    }
    pub fn full(&self) -> bool {
        self.samples.len() >= self.limit
    }
    pub fn observe_context(&mut self, u: &Universe) {
        if u.edits.len() >= self.edit_offset && self.edits.len() != u.edits.len() - self.edit_offset
        {
            self.edits = u.edits[self.edit_offset..].to_vec();
        }
        let events = &u.history.events;
        let start = self
            .last_event
            .as_ref()
            .and_then(|tail| events.iter().rposition(|e| e == tail))
            .map(|i| i + 1)
            .unwrap_or(0);
        for e in &events[start..] {
            if e.time >= self.initial_header.sim_time && !self.events.contains(e) {
                if self.events.len() == 1000 {
                    self.events.remove(0);
                }
                self.events.push(e.clone());
            }
        }
        self.last_event = events.last().cloned();
    }
    /// Returns false for duplicate paused time or a full recording. No interpolation.
    pub fn capture(&mut self, u: &Universe) -> Result<bool, String> {
        self.observe_context(u);
        if self.full()
            || self
                .samples
                .last()
                .is_some_and(|s| s.simulation_time_s == u.time && s.edit_count == u.edits.len())
        {
            return Ok(false);
        }
        if !u.time.is_finite()
            || self
                .samples
                .last()
                .is_some_and(|s| u.time < s.simulation_time_s)
        {
            return Err("Time moved backward; start a new recording".into());
        }
        let values = self
            .parameters
            .iter()
            .map(|p| {
                super::read_parameter(u, self.target, p.descriptor.id)
                    .map(|r| r.value)
                    .ok_or_else(|| {
                        format!(
                            "{} is no longer available; start a new recording",
                            p.descriptor.name
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.samples.push(ResearchSample {
            simulation_time_s: u.time,
            edit_count: u.edits.len(),
            values,
        });
        Ok(true)
    }
    pub fn points(&self, id: ParameterId) -> Vec<(f64, f64)> {
        let Some(column) = self.parameters.iter().position(|p| p.descriptor.id == id) else {
            return Vec::new();
        };
        let start = self.initial_header.sim_time;
        self.samples
            .iter()
            .map(|s| (s.simulation_time_s - start, s.values[column]))
            .collect()
    }
    pub fn csv(&self) -> String {
        let mut csv = String::from("simulation_time_s,edit_count");
        for p in &self.parameters {
            csv.push(',');
            csv.push_str(p.descriptor.key);
        }
        csv.push('\n');
        for s in &self.samples {
            csv.push_str(&s.simulation_time_s.to_string());
            csv.push(',');
            csv.push_str(&s.edit_count.to_string());
            for v in &s.values {
                csv.push(',');
                csv.push_str(&v.to_string());
            }
            csv.push('\n');
        }
        csv
    }
    /// The destination is exclusively created. Existing directories are never replaced.
    pub fn export(&self, directory: &Path) -> Result<(), String> {
        if self.samples.is_empty() {
            return Err("There are no samples to export".into());
        }
        if let Some(parent) = directory.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?;
        let result = (|| -> Result<(), String> {
            let manifest = serde_json::json!({
                "format": "cosmogon-experiment", "schema_version": 1,
                "app_version": env!("CARGO_PKG_VERSION"), "target": self.target, "target_name": self.target_name,
                "initial_header": self.initial_header, "parameters": self.parameters,
                "initial_state_fingerprint": self.initial_fingerprint,
                "time_coordinate": "Simulation seconds since J2000; Solar Lab initializes from a TDB ephemeris epoch, but long-term calendar/simulation evolution is not a full time-scale conversion service.",
                "sample_count": self.samples.len(), "sample_limit": self.limit, "stop_reason": self.stop_reason,
                "start_simulation_time_s": self.samples.first().map(|s| s.simulation_time_s),
                "end_simulation_time_s": self.samples.last().map(|s| s.simulation_time_s),
                "state_vector_frame": "System frame: barycentric with dynamics, fixed primary with analytic orbits; ecliptic J2000 orientation for the Solar System Lab.",
                "sampling": self.sampling_policy,
                "state_order": "edit_count distinguishes observed states changed by edits at the same simulation time. Unchanged paused states are deduplicated; there is no resampling or interpolation.",
                "limitations": ["Per-value uncertainty is unavailable and represented by null.", "No general scientific accuracy or mission-safety guarantee.", "Only the most recent 1000 observed Chronicle events are retained.", "Rewind/universe replacement requires a separate recording; native generation tracking enforces this.", "Model/source descriptions document implemented approximations, not new independent validation."],
                "files": ["samples.csv", "initial.cosmo", "edits.json", "events.json", "models.json"]
            });
            let models = serde_json::json!({ "models": PhysicsModelRegistry::models(), "sources": SCIENTIFIC_SOURCES });
            let files = [
                (
                    "manifest.json",
                    serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
                ),
                ("samples.csv", self.csv().into_bytes()),
                ("initial.cosmo", self.initial_save.clone()),
                (
                    "edits.json",
                    serde_json::to_vec_pretty(&self.edits).map_err(|e| e.to_string())?,
                ),
                (
                    "events.json",
                    serde_json::to_vec_pretty(&self.events).map_err(|e| e.to_string())?,
                ),
                (
                    "models.json",
                    serde_json::to_vec_pretty(&models).map_err(|e| e.to_string())?,
                ),
            ];
            for (name, bytes) in files {
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(directory.join(name))
                    .map_err(|e| e.to_string())?;
                file.write_all(&bytes)
                    .and_then(|()| file.sync_all())
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_dir_all(directory);
        }
        result
    }
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
    #[test]
    fn recording_is_read_only_bounded_and_rejects_rewind() {
        let mut u = lab();
        let before = serde_json::to_string(&u).unwrap();
        let mut e =
            Experiment::new(&u, ResearchTarget::Body(BodyRef { system: 0, body: 2 }), 3).unwrap();
        assert_eq!(serde_json::to_string(&u).unwrap(), before);
        assert!(!e.capture(&u).unwrap());
        u.advance_by(3600.0);
        assert!(e.capture(&u).unwrap());
        let old = u.clone();
        u.advance_by(3600.0);
        assert!(e.capture(&u).unwrap());
        assert!(e.full());
        assert!(!e.capture(&u).unwrap());
        assert_eq!(e.samples.len(), 3);
        e.limit = 10;
        assert!(e.capture(&old).is_err());
    }
    #[test]
    fn bundle_round_trips_initial_state_and_protects_existing_files() {
        let u = lab();
        let before = serde_json::to_string(&u).unwrap();
        let e =
            Experiment::new(&u, ResearchTarget::Body(BodyRef { system: 0, body: 2 }), 5).unwrap();
        let dir = std::env::temp_dir().join(format!(
            "cosmogon-research-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        e.export(&dir).unwrap();
        let bytes = std::fs::read(dir.join("samples.csv")).unwrap();
        assert!(e.export(&dir).is_err());
        assert_eq!(std::fs::read(dir.join("samples.csv")).unwrap(), bytes);
        let loaded = save::read_file(&dir.join("initial.cosmo")).unwrap();
        assert_eq!(serde_json::to_string(&loaded).unwrap(), before);
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["sample_count"], 1);
        assert!(manifest["parameters"][0]["initial"]["uncertainty"].is_null());
        assert_eq!(e.csv().lines().count(), 2);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn observed_edits_and_events_are_export_context_not_model_mutation() {
        let mut u = lab();
        let t = ResearchTarget::Body(BodyRef { system: 0, body: 2 });
        let mut e = Experiment::new(&u, t, 10).unwrap();
        u.apply_edit(super::super::parameter_edit(&u, t, ParameterId::Radius, 7e6).unwrap())
            .unwrap();
        e.observe_context(&u);
        assert!(e.capture(&u).unwrap());
        assert_eq!(e.samples.len(), 2);
        assert_eq!(
            e.samples[0].simulation_time_s,
            e.samples[1].simulation_time_s
        );
        assert_ne!(e.samples[0].edit_count, e.samples[1].edit_count);
        assert_eq!(e.edits.len(), 1);
        assert!(!e.events.is_empty());
        assert_eq!(e.edits[0], *u.edits.last().unwrap());
    }
}
