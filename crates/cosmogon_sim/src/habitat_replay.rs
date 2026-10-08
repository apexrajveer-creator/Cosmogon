//! Recompute a saved four-branch design. Reproducibility is not empirical validation.
use crate::{habitat, habitat_factorial, research, sandbox::Edit, save};
use serde::Serialize;
use serde_json::{json, Value};
use std::{path::Path, time::Duration};
pub const METHODOLOGY_SOURCE:&str="https://standards.nasa.gov/sites/default/files/standards/NASA/B/1/NASA-STD-7009B-Final-3-5-2024.pdf";

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Progress {
    pub branch_index: usize,
    pub observation_index: usize,
    pub checked_rows: usize,
    pub total_rows: usize,
    pub simulation_time_s: f64,
    pub target_time_s: f64,
}
fn fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for b in bytes {
        hash = (hash ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}
// Diagnostic previews are bounded independently of the complete byte comparison.
fn preview(value: &Value) -> Value {
    match value {
        Value::String(s) if s.chars().count() > 160 => {
            json!({"kind":"text_preview","characters":s.chars().count(),"preview":s.chars().take(160).collect::<String>(),"truncated":true})
        }
        Value::Array(a) => json!({"kind":"array_preview","length":a.len()}),
        Value::Object(o) => json!({"kind":"object_preview","field_count":o.len()}),
        _ => value.clone(),
    }
}
fn differences(recorded: &Value, actual: &Value, path: &str, out: &mut Vec<Value>) {
    if recorded.to_string() == actual.to_string() || out.len() >= 8 {
        return;
    }
    match (recorded, actual) {
        (Value::Object(a), Value::Object(b)) if a.keys().eq(b.keys()) => {
            for (key, value) in a {
                if out.len() >= 8 {
                    break;
                }
                differences(value, &b[key], &format!("{path}.{key}"), out);
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                if out.len() >= 8 {
                    break;
                }
                differences(x, y, &format!("{path}[{i}]"), out);
            }
        }
        _ => {
            out.push(json!({"path":path,"recorded":preview(recorded),"recomputed":preview(actual)}))
        }
    }
}
/// Callback may cancel at source/observation/scheduler boundaries. No recorded file is modified.
pub fn verify_with(
    path: &Path,
    mut progress: impl FnMut(Progress) -> Result<(), String>,
) -> Result<Value, String> {
    let design = habitat_factorial::read(path)?;
    let plan = design.plan()?;
    let id = design.manifest["habitat_id"]
        .as_u64()
        .ok_or("Missing habitat identity")?;
    let base = save::from_json(&crate::habitat_recording::read(
        &path.join("base.cosmo"),
        32 * 1024 * 1024,
    )?)
    .map_err(|e| e.to_string())?;
    let configurations = plan.configurations(&base, id)?;
    let system = base
        .habitats
        .iter()
        .find(|h| h.id == id)
        .ok_or("Missing habitat")?
        .definition
        .body
        .system;
    let total_rows = plan.samples * 4;
    let mut checked_rows = 0;
    let mut mismatched_rows = 0;
    let mut mismatches = Vec::new();
    let mut branches = Vec::new();
    for (branch, config) in configurations.into_iter().enumerate() {
        let stored = &design.recordings[branch];
        let mut world = base.clone();
        world.apply_edit(Edit::ConfigureHabitat {
            system,
            id,
            config: Box::new(config),
        })?;
        let initial_time = world.time;
        let initial_fingerprint = research::state_fingerprint(&world)?;
        let initial_versions =
            serde_json::to_value(save::data_versions(&world)).map_err(|e| e.to_string())?;
        let mut matched = 0;
        for index in 0..plan.samples {
            let target = initial_time + index as f64 * plan.interval_s;
            // Fixed simulation steps are preserved across wall-time slices; no variable dt.
            loop {
                progress(Progress {
                    branch_index: branch,
                    observation_index: index,
                    checked_rows,
                    total_rows,
                    simulation_time_s: world.time,
                    target_time_s: target,
                })?;
                let advanced = world.advance_to(target, Some(Duration::from_millis(50)), None);
                if !advanced.lagging {
                    break;
                }
            }
            if world.time != target {
                return Err("Replay did not reach the exact requested world time".into());
            }
            let actual = habitat::sample(&world, id)?;
            let recorded = &stored.rows[index];
            let actual_bytes = serde_json::to_vec(&actual).map_err(|e| e.to_string())?;
            let recorded_bytes = serde_json::to_vec(recorded).map_err(|e| e.to_string())?;
            if actual_bytes == recorded_bytes {
                matched += 1;
            } else {
                mismatched_rows += 1;
                if mismatches.len() < 16 {
                    let mut fields = Vec::new();
                    differences(
                        &serde_json::to_value(recorded).map_err(|e| e.to_string())?,
                        &serde_json::to_value(&actual).map_err(|e| e.to_string())?,
                        "sample",
                        &mut fields,
                    );
                    mismatches.push(json!({"branch_index":branch,"observation_index":index,"simulation_time_s":target,"recorded_sample_fingerprint":fingerprint(&recorded_bytes),"recomputed_sample_fingerprint":fingerprint(&actual_bytes),"differences":fields,"difference_limit":8,"comparison":"Exact canonical typed Sample serialization, including signed-zero representation; hashes are diagnostic, not cryptographic"}));
                }
            }
            checked_rows += 1;
            progress(Progress {
                branch_index: branch,
                observation_index: index,
                checked_rows,
                total_rows,
                simulation_time_s: world.time,
                target_time_s: target,
            })?;
        }
        let final_fingerprint = research::state_fingerprint(&world)?;
        let final_versions =
            serde_json::to_value(save::data_versions(&world)).map_err(|e| e.to_string())?;
        let initial_match = stored.summary["initial_state_fingerprint"] == initial_fingerprint;
        let final_match = stored.summary["final_state_fingerprint"] == final_fingerprint;
        let sources_match = initial_versions == stored.summary["initial_dataset_versions"]
            && final_versions == stored.summary["final_dataset_versions"];
        branches.push(json!({"index":branch,"checked_observations":plan.samples,"matched_observations":matched,"sample_mismatch_count":plan.samples-matched,
            "initial_fingerprint_match":initial_match,"final_fingerprint_match":final_match,"source_versions_match":sources_match,
            "recorded_initial_state_fingerprint":stored.summary["initial_state_fingerprint"],"recomputed_initial_state_fingerprint":initial_fingerprint,
            "recorded_final_state_fingerprint":stored.summary["final_state_fingerprint"],"recomputed_final_state_fingerprint":final_fingerprint,"verified":matched==plan.samples && initial_match && final_match && sources_match}));
    }
    let verified = branches.iter().all(|b| b["verified"] == true);
    Ok(
        json!({"format":"cosmogon-habitat-replay-verification","schema_version":1,"complete":true,"verified":verified,
        "engine_version":env!("CARGO_PKG_VERSION"),"methodological_context":{"source":METHODOLOGY_SOURCE,"version":"NASA-STD-7009B 5 March 2024","sections":"4.2.3–4.2.6, Appendix E.3.3–E.3.4","scope":"Verification/validation distinction motivates explicit evidence/scope; this replay mechanism is an engineering reproducibility test, not a claim of NASA-standard compliance"},"plan":plan,"habitat_id":id,"common_state_fingerprint":design.manifest["common_state_fingerprint"],
        "common_data_versions":design.manifest["common_data_versions"],"recordings":design.recordings.iter().map(|r|&r.summary).collect::<Vec<_>>(),
        "checked_rows":checked_rows,"total_rows":total_rows,"matched_rows":checked_rows-mismatched_rows,"sample_mismatch_count":mismatched_rows,
        "branches":branches,"mismatches":mismatches,"mismatch_details_limit":16,"diagnostic_preview_scope":"Up to eight fields per differing observation; long text/compound values use bounded previews. Exact comparison uses complete canonical typed samples.",
        "method":"Reopen the source-pinned common world; apply each ordinary configuration edit; recompute fixed scheduler steps at every exact plan time; compare complete canonical Sample bytes and initial/final Universe fingerprints/source versions. Original observations remain read-only.",
        "limits":"Reproducibility against this engine and retained source versions only. No empirical/hardware validation, independent physical model, uncertainty, statistical significance or cryptographic authenticity. Complete means the verification ran to completion; verified states whether every row and endpoint matches. Cancellation/input errors produce no successful verification report."}),
    )
}
pub fn verify(path: &Path) -> Result<Value, String> {
    verify_with(path, |_| Ok(()))
}
pub fn export(report: &Value, path: &Path) -> Result<(), String> {
    use std::io::Write;
    if report["format"] != "cosmogon-habitat-replay-verification"
        || report["schema_version"] != 1
        || report["complete"] != true
    {
        return Err("Only a completed replay verification can be exported".into());
    }
    let bytes = serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Err("Replay verification exceeds the 1 MiB report limit".into());
    }
    let manifest=serde_json::to_vec_pretty(&json!({"format":"cosmogon-habitat-replay-verification-export","schema_version":1,"complete":true,"verified":report["verified"],"common_state_fingerprint":report["common_state_fingerprint"],"files":["verification.json"]})).map_err(|e|e.to_string())?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(path).map_err(|e| {
        format!("Existing verification reports are protected; choose a new folder: {e}")
    })?;
    for (name, data) in [("verification.json", bytes), ("manifest.json", manifest)] {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| format!("Incomplete verification export: {e}"))?;
        f.write_all(&data)
            .map_err(|e| format!("Incomplete verification export: {e}"))?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat_factorial::Secondary,
        habitat_sweep::{self, Parameter, Plan},
        Universe,
    };
    fn design(root: &Path) {
        let mut world = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut definition = habitat::Definition::at(crate::BodyRef { system: 0, body: 3 }, 0., 0.);
        definition.config.moisture = Some(crate::moisture::Config::default());
        world
            .apply_edit(Edit::CreateHabitat {
                definition: Box::new(definition),
            })
            .unwrap();
        world.advance_to(world.time + 86410.25, None, None);
        let mut p = Plan::linear(Parameter::MoisturePowerW, 0., 500., 2, 3, 3600.).unwrap();
        p.schema_version = 2;
        p.secondary = Some(Secondary {
            parameter: Parameter::MoistureAirborneFraction,
            values: [0.2, 1.0],
        });
        habitat_sweep::execute(&world, 0, &p, root).unwrap();
    }
    #[test]
    fn exact_replay_checks_every_row_endpoint_and_protects_source_and_report() {
        let root =
            std::env::temp_dir().join(format!("cosmogon-replay-good-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        design(&root.join("design"));
        let before = std::fs::read(root.join("design/run-001/samples.json")).unwrap();
        let report = verify(&root.join("design")).unwrap();
        assert_eq!(report["verified"], true);
        assert_eq!(report["checked_rows"], 12);
        assert_eq!(report["matched_rows"], 12);
        assert_eq!(verify(&root.join("design")).unwrap(), report);
        export(&report, &root.join("report")).unwrap();
        let bytes = std::fs::read(root.join("report/verification.json")).unwrap();
        assert!(export(&report, &root.join("report")).is_err());
        assert_eq!(
            std::fs::read(root.join("report/verification.json")).unwrap(),
            bytes
        );
        assert_eq!(
            std::fs::read(root.join("design/run-001/samples.json")).unwrap(),
            before
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn plausible_interior_edit_is_structurally_readable_but_fails_replay_at_exact_row() {
        let root =
            std::env::temp_dir().join(format!("cosmogon-replay-edit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        design(&root);
        let path = root.join("run-002/samples.json");
        let mut rows: Vec<habitat::Sample> =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        rows[1].habitat.state.potable_water_kg -= 1.;
        rows[1].habitat.state.wastewater_kg += 1.;
        rows[1]
            .habitat
            .budget
            .moisture
            .as_mut()
            .unwrap()
            .constraints = vec!["altered retained explanation".repeat(20000)];
        std::fs::write(&path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
        std::fs::write(
            root.join("run-002/samples.csv"),
            habitat::samples_csv(&rows),
        )
        .unwrap();
        assert!(habitat_factorial::read(&root).is_ok());
        let report = verify(&root).unwrap();
        assert_eq!(report["verified"], false);
        assert_eq!(report["sample_mismatch_count"], 1);
        assert_eq!(report["matched_rows"], 11);
        assert!(serde_json::to_vec(&report).unwrap().len() < 64 * 1024);
        assert_eq!(report["mismatches"][0]["branch_index"], 2);
        assert_eq!(report["mismatches"][0]["observation_index"], 1);
        assert_eq!(
            report["mismatches"][0]["simulation_time_s"],
            rows[1].simulation_time_s
        );
        assert!(report["branches"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["final_fingerprint_match"] == true));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cancellation_cannot_be_reported_as_success() {
        let root =
            std::env::temp_dir().join(format!("cosmogon-replay-cancel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        design(&root);
        let result = verify_with(&root, |p| {
            if p.checked_rows >= 1 {
                Err("Canceled".into())
            } else {
                Ok(())
            }
        });
        assert_eq!(result.unwrap_err(), "Canceled");
        assert!(export(&json!({"complete":false}), &root.join("bad")).is_err());
        assert!(!root.join("bad").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
