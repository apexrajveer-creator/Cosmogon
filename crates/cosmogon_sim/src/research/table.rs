//! Reopen numeric observer bundles without loading or evolving their initial universe.
use super::{
    DataState, ParameterId, ResearchSample, ResearchTarget, MAX_RESEARCH_SAMPLES, PARAMETERS,
};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::Path};

#[derive(Clone, Debug, Serialize)]
pub struct SavedColumn {
    pub parameter: ParameterId,
    pub key: &'static str,
    pub unit: &'static str,
    pub state: DataState,
    pub source: String,
    pub uncertainty: Option<f64>,
}
#[derive(Clone, Debug, Serialize)]
pub struct SavedRecording {
    pub target: ResearchTarget,
    pub target_name: String,
    pub columns: Vec<SavedColumn>,
    pub samples: Vec<ResearchSample>,
    pub events: Vec<(f64, String)>,
    pub frame: String,
    pub time_coordinate: String,
    pub engine: String,
    pub initial_fingerprint: String,
}
#[derive(Deserialize)]
struct Descriptor {
    id: ParameterId,
    key: String,
    unit: String,
}
#[derive(Deserialize)]
struct Initial {
    state: DataState,
    source: String,
    uncertainty: Option<f64>,
}
#[derive(Deserialize)]
struct Column {
    descriptor: Descriptor,
    initial: Initial,
}
#[derive(Deserialize)]
struct Manifest {
    format: String,
    schema_version: u32,
    target: ResearchTarget,
    target_name: String,
    parameters: Vec<Column>,
    sample_count: usize,
    state_vector_frame: String,
    time_coordinate: String,
    initial_state_fingerprint: String,
    initial_header: Header,
}
#[derive(Deserialize)]
struct Header {
    physics_engine: String,
}
#[derive(Deserialize)]
struct Event {
    time: f64,
    title: String,
}
fn bounded_read(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!(
            "{} exceeds the recording input limit",
            path.display()
        ));
    }
    Ok(bytes)
}
impl SavedRecording {
    /// Only the numeric CSV dialect emitted by Cosmogon is accepted. No general CSV inference.
    pub fn read(directory: &Path) -> Result<Self, String> {
        let manifest: Manifest = serde_json::from_slice(&bounded_read(
            &directory.join("manifest.json"),
            1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        if manifest.format != "cosmogon-experiment" || manifest.schema_version != 1 {
            return Err("Unsupported recording format/version".into());
        }
        if manifest.target_name.len() > 256
            || manifest.target_name.is_empty()
            || manifest.parameters.is_empty()
            || manifest.parameters.len() > PARAMETERS.len()
            || !(1..=MAX_RESEARCH_SAMPLES).contains(&manifest.sample_count)
            || manifest.state_vector_frame.len() > 4096
            || manifest.time_coordinate.len() > 4096
        {
            return Err("Invalid or oversized recording metadata".into());
        }
        let mut keys = vec!["simulation_time_s".to_string(), "edit_count".to_string()];
        let mut columns = Vec::new();
        for c in manifest.parameters {
            let d = c.descriptor.id.descriptor();
            if d.key != c.descriptor.key
                || d.unit != c.descriptor.unit
                || keys.contains(&c.descriptor.key)
                || c.initial.source.len() > 8192
                || c.initial
                    .uncertainty
                    .is_some_and(|v| !v.is_finite() || v < 0.0)
            {
                return Err("Recording has duplicate parameters, incompatible canonical units or invalid source metadata".into());
            }
            keys.push(c.descriptor.key);
            columns.push(SavedColumn {
                parameter: d.id,
                key: d.key,
                unit: d.unit,
                state: c.initial.state,
                source: c.initial.source,
                uncertainty: c.initial.uncertainty,
            });
        }
        let csv = String::from_utf8(bounded_read(
            &directory.join("samples.csv"),
            8 * 1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        let mut lines = csv.lines();
        if lines.next() != Some(keys.join(",").as_str()) {
            return Err("Recording CSV header differs from its unit-labelled manifest".into());
        }
        let mut samples: Vec<ResearchSample> = Vec::new();
        for line in lines {
            if samples.len() >= manifest.sample_count {
                return Err("Recording has more states than its manifest".into());
            }
            let fields: Vec<_> = line.split(',').collect();
            if fields.len() != keys.len() {
                return Err("Recording row has the wrong number of columns".into());
            }
            let time = fields[0]
                .parse::<f64>()
                .map_err(|_| "Invalid recording time")?;
            let edits = fields[1]
                .parse::<usize>()
                .map_err(|_| "Invalid recording edit count")?;
            let values = fields[2..]
                .iter()
                .map(|s| {
                    s.parse::<f64>()
                        .map_err(|_| "Invalid numeric recording value")
                })
                .collect::<Result<Vec<_>, _>>()?;
            if !time.is_finite() || values.iter().any(|v| !v.is_finite()) {
                return Err("Recording contains a nonfinite value".into());
            }
            if let Some(previous) = samples.last() {
                if time < previous.simulation_time_s
                    || edits < previous.edit_count
                    || (time == previous.simulation_time_s && edits == previous.edit_count)
                {
                    return Err("Recording state order is invalid or duplicated".into());
                }
            }
            if samples
                .first()
                .is_some_and(|first| !(time - first.simulation_time_s).is_finite())
            {
                return Err("Recording elapsed time exceeds finite numeric range".into());
            }
            samples.push(ResearchSample {
                simulation_time_s: time,
                edit_count: edits,
                values,
            });
        }
        if samples.len() != manifest.sample_count {
            return Err("Recording state count does not match its manifest".into());
        }
        let raw: Vec<Event> = serde_json::from_slice(&bounded_read(
            &directory.join("events.json"),
            2 * 1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        if raw.len() > 1000
            || raw
                .iter()
                .any(|e| !e.time.is_finite() || e.title.len() > 8192)
        {
            return Err("Invalid or oversized recording events".into());
        }
        let start = samples[0].simulation_time_s;
        let events = raw.into_iter().map(|e| (e.time - start, e.title)).collect();
        Ok(Self {
            target: manifest.target,
            target_name: manifest.target_name,
            columns,
            samples,
            events,
            frame: manifest.state_vector_frame,
            time_coordinate: manifest.time_coordinate,
            engine: manifest.initial_header.physics_engine,
            initial_fingerprint: manifest.initial_state_fingerprint,
        })
    }
    /// Compare exact elapsed states only. No interpolation, epoch alignment or uncertainty implied.
    pub fn difference_from(
        &self,
        baseline: &Self,
        parameter: ParameterId,
    ) -> Result<Vec<(f64, f64)>, String> {
        if !self.columns.iter().any(|c| c.parameter == parameter)
            || !baseline.columns.iter().any(|c| c.parameter == parameter)
        {
            return Err("Comparison parameter is unavailable in one recording".into());
        }
        let own = self.points(parameter);
        let reference = baseline.points(parameter);
        if own.len() != reference.len() || own.iter().zip(&reference).any(|(a, b)| a.0 != b.0) {
            return Err(
                "Difference requires identical elapsed sample times; no interpolation is performed"
                    .into(),
            );
        }
        let points = own
            .iter()
            .zip(&reference)
            .map(|(a, b)| (a.0, a.1 - b.1))
            .collect::<Vec<_>>();
        if points.iter().any(|p| !p.1.is_finite()) {
            return Err("Difference exceeds finite numeric range".into());
        }
        Ok(points)
    }
    pub fn points(&self, parameter: ParameterId) -> Vec<(f64, f64)> {
        let Some(index) = self.columns.iter().position(|p| p.parameter == parameter) else {
            return Vec::new();
        };
        let start = self.samples[0].simulation_time_s;
        self.samples
            .iter()
            .map(|s| (s.simulation_time_s - start, s.values[index]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyRef, Scenario, Universe, UniverseSettings};
    #[test]
    fn saved_tables_preserve_actual_rows_and_reject_corruption() {
        let mut u = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = ResearchTarget::Body(BodyRef { system: 0, body: 2 });
        let mut e = super::super::Experiment::new(&u, target, 3).unwrap();
        u.advance_by(86400.0);
        e.capture(&u).unwrap();
        let path = std::env::temp_dir().join(format!(
            "cosmogon-table-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        e.export(&path).unwrap();
        let table = SavedRecording::read(&path).unwrap();
        assert_eq!(table.target_name, "Earth");
        assert_eq!(
            table.points(ParameterId::Eccentricity),
            e.points(ParameterId::Eccentricity)
        );
        assert_eq!(table.samples[1].values, e.samples[1].values);
        assert!(table
            .difference_from(&table, ParameterId::Eccentricity)
            .unwrap()
            .iter()
            .all(|p| p.1 == 0.0));
        let mut shifted = table.clone();
        shifted.samples[1].simulation_time_s += 1.0;
        assert!(shifted
            .difference_from(&table, ParameterId::Eccentricity)
            .is_err());
        let csv = std::fs::read_to_string(path.join("samples.csv")).unwrap();
        // Test corruption without relying on a particular absolute time's digits.
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let row = lines.next().unwrap();
        let bad = format!("{header}\nNaN,{}\n", row.split_once(',').unwrap().1);
        std::fs::write(path.join("samples.csv"), bad).unwrap();
        assert!(SavedRecording::read(&path).is_err());
        std::fs::write(
            path.join("samples.csv"),
            format!("{header}\n{row}\n{row}\n"),
        )
        .unwrap();
        assert!(SavedRecording::read(&path).is_err());
        std::fs::write(path.join("samples.csv"), csv).unwrap();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path.join("manifest.json")).unwrap()).unwrap();
        manifest["parameters"][0]["descriptor"]["unit"] = serde_json::json!("wrong unit");
        std::fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(SavedRecording::read(&path).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
}
