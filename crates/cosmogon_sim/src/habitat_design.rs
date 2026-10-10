//! Complete retained single-setting designs. Structural checks are separate from replay.
use crate::{habitat_recording::Recording, habitat_sweep::Plan};
use serde_json::{json, Value};
use std::path::Path;

pub struct Single {
    pub recordings: Vec<Recording>,
    pub manifest: Value,
}
impl Single {
    pub fn plan(&self) -> Result<Plan, String> {
        serde_json::from_value(self.manifest["plan"].clone()).map_err(|e| e.to_string())
    }
    pub fn selected(&self, a: usize, b: usize) -> Result<(&Recording, &Recording), String> {
        Ok((
            self.recordings
                .get(a)
                .ok_or("Branch A is outside the saved design")?,
            self.recordings
                .get(b)
                .ok_or("Branch B is outside the saved design")?,
        ))
    }
    pub fn report_at(&self, index: usize) -> Result<Value, String> {
        let plan = self.plan()?;
        let rows = self.recordings.iter().enumerate().map(|(i, r)| {
            let row = r.rows.get(index).ok_or("Observation index is outside the saved design")?;
            Ok(json!({"branch_index":i,"value":plan.values[i],"recording":r.summary,"sample":row,
                "metrics":plan.metrics(&row.habitat).iter().map(|m| (m.id().into(), json!({"unit":m.unit(),"value":m.value(row)}))).collect::<serde_json::Map<String,Value>>()}))
        }).collect::<Result<Vec<Value>,String>>()?;
        Ok(
            json!({"format":"cosmogon-habitat-single-design-observation","schema_version":1,
            "manifest":self.manifest,"observation_index":index,"branches":rows,
            "scope":"Actual independent common-state branches at the exact stored plan time. Model sensitivity, not empirical validation or uncertainty."}),
        )
    }
    pub fn export_trajectories(&self, path: &Path) -> Result<(), String> {
        use std::{fmt::Write as _, io::Write};
        let plan = self.plan()?;
        let mut csv=String::from("branch_index,setting_value,observation_index,simulation_time_s,completed_habitat_time_s,completed_interval_s,metric,unit,value\n");
        let mut observations = Vec::new();
        for (branch, r) in self.recordings.iter().enumerate() {
            for (index, row) in r.rows.iter().enumerate() {
                let values = plan
                    .metrics(&row.habitat)
                    .iter()
                    .map(|m| {
                        let value = m.value(row);
                        let _ = writeln!(
                            csv,
                            "{branch},{:.17e},{index},{:.17e},{:.17e},{:.17e},{},{},{}",
                            plan.values[branch],
                            row.simulation_time_s,
                            row.habitat.last_time,
                            row.habitat.budget.seconds,
                            m.id(),
                            m.unit(),
                            value.map(|v| format!("{v:.17e}")).unwrap_or_default()
                        );
                        (m.id().into(), json!({"unit":m.unit(),"value":value}))
                    })
                    .collect::<serde_json::Map<String, Value>>();
                observations.push(json!({"branch_index":branch,"setting_value":plan.values[branch],"observation_index":index,"simulation_time_s":row.simulation_time_s,"completed_habitat_time_s":row.habitat.last_time,"completed_interval_s":row.habitat.budget.seconds,"metrics":values}));
            }
        }
        let report = json!({"format":"cosmogon-habitat-single-design-trajectories","schema_version":1,"manifest":self.manifest,"observations":observations,
            "scope":"Actual stored independent model responses at exact plan times; no interpolation or uncertainty. Flow/power metrics retain their completed-interval meaning, not full-span history. Original complete samples/worlds remain in the source design."});
        let data = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
        if data.len() > 32 * 1024 * 1024 || csv.len() > 32 * 1024 * 1024 {
            return Err("Single-design trajectory export exceeds the 32 MiB per-file limit".into());
        }
        let manifest=serde_json::to_vec_pretty(&json!({"format":"cosmogon-habitat-single-design-trajectory-export","schema_version":1,"complete":true,"common_state_fingerprint":self.manifest["common_state_fingerprint"],"files":["trajectories.json","trajectories.csv"]})).map_err(|e|e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| {
            format!("Existing trajectory exports are protected; choose a new folder: {e}")
        })?;
        for (name, bytes) in [
            ("trajectories.json", data),
            ("trajectories.csv", csv.into_bytes()),
            ("manifest.json", manifest),
        ] {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .and_then(|mut f| f.write_all(&bytes))
                .map_err(|e| format!("Incomplete single-design trajectory export: {e}"))?;
        }
        Ok(())
    }
    pub fn figure(
        &self,
        a: usize,
        b: usize,
        layout: &crate::habitat_graph::Layout,
    ) -> Result<crate::habitat_figure::Figure, String> {
        let (left, right) = self.selected(a, b)?;
        let plan = self.plan()?;
        let mut figure = crate::habitat_figure::Figure::ordinary(left, Some(right), layout)?;
        figure.single_design_context = Some(
            json!({"manifest":self.manifest,"selected_branches":[a,b],
            "scope":"Two selected independent branches from the retained single-setting design; complete common-state identity and all branch summaries are retained here."}),
        );
        figure.configuration_assumptions = plan.configuration_assumptions();
        for (side, (series, index)) in figure.series.iter_mut().zip([a, b]).enumerate() {
            series.label = format!(
                "{} · run-{index:03} · setting {}",
                if side == 0 { "A" } else { "B" },
                plan.values[index]
            );
        }
        Ok(figure)
    }
}
pub enum Design {
    Single(Single),
    Paired(crate::habitat_factorial::Design),
}
impl Design {
    pub fn plan(&self) -> Result<Plan, String> {
        match self {
            Self::Single(d) => d.plan(),
            Self::Paired(d) => d.plan(),
        }
    }
    pub fn recordings(&self) -> &[Recording] {
        match self {
            Self::Single(d) => &d.recordings,
            Self::Paired(d) => &d.recordings,
        }
    }
    pub fn manifest(&self) -> &Value {
        match self {
            Self::Single(d) => &d.manifest,
            Self::Paired(d) => &d.manifest,
        }
    }
}
pub fn read(path: &Path) -> Result<Design, String> {
    if Plan::read(&path.join("plan.json"))?.secondary.is_some() {
        crate::habitat_factorial::read(path).map(Design::Paired)
    } else {
        read_single(path).map(Design::Single)
    }
}
pub fn read_single(path: &Path) -> Result<Single, String> {
    use std::io::Read;
    let read_json = |name: &str| -> Result<Value, String> {
        let p = path.join(name);
        let m = std::fs::metadata(&p).map_err(|e| e.to_string())?;
        if !m.is_file() || m.len() > 1024 * 1024 {
            return Err("Design metadata must be a regular file within 1 MiB".into());
        }
        let f = std::fs::File::open(p).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        f.take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 1024 * 1024 {
            return Err("Design metadata exceeds 1 MiB".into());
        }
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    };
    let manifest = read_json("manifest.json")?;
    let plan = Plan::read(&path.join("plan.json"))?;
    if manifest["format"] != "cosmogon-habitat-sweep"
        || manifest["schema_version"] != 1
        || manifest["complete"] != true
        || plan.secondary.is_some()
        || manifest["plan"] != serde_json::to_value(&plan).map_err(|e| e.to_string())?
    {
        return Err("Incomplete or unsupported single-setting design".into());
    }
    if manifest.get("configuration_assumptions") != plan.configuration_assumptions().as_ref() {
        return Err("Design configuration assumptions disagree with its typed plan".into());
    }
    let id = manifest["habitat_id"]
        .as_u64()
        .ok_or("Missing design habitat identity")?;
    let base = crate::save::from_json(&crate::habitat_recording::read(
        &path.join("base.cosmo"),
        32 * 1024 * 1024,
    )?)
    .map_err(|e| e.to_string())?;
    if manifest["common_state_fingerprint"] != crate::research::state_fingerprint(&base)?
        || manifest["common_data_versions"]
            != serde_json::to_value(crate::save::data_versions(&base)).map_err(|e| e.to_string())?
    {
        return Err("Design common state/source identity mismatch".into());
    }
    let configurations = plan.configurations(&base, id)?;
    let runs = manifest["runs"]
        .as_array()
        .filter(|r| r.len() == plan.branch_count())
        .ok_or("Design needs every planned independent run")?;
    let mut recordings = Vec::new();
    for (i, configuration) in configurations.into_iter().enumerate() {
        let folder = format!("run-{i:03}");
        let recording = Recording::read(&path.join(&folder))?;
        let mut expected = base.clone();
        let system = base
            .habitats
            .iter()
            .find(|h| h.id == id)
            .ok_or("Missing design habitat")?
            .definition
            .body
            .system;
        expected.apply_edit(crate::sandbox::Edit::ConfigureHabitat {
            system,
            id,
            config: Box::new(configuration),
        })?;
        if runs[i]["index"] != i
            || runs[i]["folder"] != folder
            || runs[i]["value"] != plan.values[i]
            || runs[i].get("secondary_value").is_some()
            || runs[i]["recording"] != recording.summary
            || recording.summary["initial_state_fingerprint"]
                != crate::research::state_fingerprint(&expected)?
        {
            return Err("Design branch differs from its common-state ordinary edit".into());
        }
        if recording.rows.len() != plan.samples
            || recording.rows.iter().enumerate().any(|(k, row)| {
                row.simulation_time_s != base.time + k as f64 * plan.interval_s
                    || row.habitat.id != id
            })
        {
            return Err(
                "Design observations disagree with exact planned times or habitat identity".into(),
            );
        }
        let last = recording.rows.last().ok_or("Empty design branch")?;
        let expected_metrics = plan
            .metrics(&last.habitat)
            .into_iter()
            .map(|metric| {
                (
                    metric.id().into(),
                    json!({"unit":metric.unit(),"value":metric.value(last)}),
                )
            })
            .collect::<serde_json::Map<String, Value>>();
        if runs[i]["final_metrics"] != Value::Object(expected_metrics) {
            return Err("Design metric catalog disagrees with recorded final observation".into());
        }
        recordings.push(recording);
    }
    if manifest["parameter"] != plan.parameter.id()
        || manifest["unit"] != plan.parameter.unit()
        || manifest.get("factorial_analysis").is_some()
        || path.join("factorial.json").exists()
    {
        return Err("Single-setting design identity disagrees with its plan".into());
    }
    Ok(Single {
        recordings,
        manifest,
    })
}

/// Explicit zero-based branch pair; rejects ignored/ambiguous or unbounded selections.
pub fn parse_branches(text: &str) -> Result<[usize; 2], String> {
    let parts = text
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<usize>()
                .map_err(|_| "Branches require two indices separated by a comma".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 2 || parts.iter().any(|i| *i >= 16) {
        return Err("Branches require two indices between 0 and 15".into());
    }
    Ok([parts[0], parts[1]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat,
        habitat_sweep::{self, Parameter},
        sandbox::Edit,
        Universe,
    };
    fn fixture(name: &str, crop: bool) -> (std::path::PathBuf, Universe, Plan) {
        let path =
            std::env::temp_dir().join(format!("cosmogon-single-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut d = if crop {
            crate::crop_process::definition_at(crate::BodyRef { system: 0, body: 3 }, 0., 0.)
        } else {
            habitat::Definition::at(crate::BodyRef { system: 0, body: 3 }, 0., 0.)
        };
        d.config.crew = 0;
        d.config.firm_power_w = 5000.;
        d.config.solar_area_m2 = 0.;
        if let Some(c) = &mut d.config.crop {
            c.assume_maintained_conditions = true;
        }
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        u.advance_to(u.time + 86410.25, None, None);
        let p = Plan::linear(
            if crop {
                Parameter::CropAuxiliaryPowerW
            } else {
                Parameter::FirmPowerW
            },
            0.,
            10000.,
            3,
            3,
            3600.,
        )
        .unwrap();
        habitat_sweep::execute(&u, 0, &p, &path).unwrap();
        (path, u, p)
    }
    #[test]
    fn single_design_retains_common_remainders_catalogs_and_selected_exact_figures() {
        let (path, u, plan) = fixture("identity", true);
        let before = crate::research::state_fingerprint(&u).unwrap();
        let d = read_single(&path).unwrap();
        assert_eq!(d.plan().unwrap(), plan);
        assert_eq!(d.recordings.len(), 3);
        for (i, r) in d.recordings.iter().enumerate() {
            let mut expected = u.clone();
            let config = plan.configurations(&u, 0).unwrap().remove(i);
            expected
                .apply_edit(Edit::ConfigureHabitat {
                    system: 0,
                    id: 0,
                    config: Box::new(config),
                })
                .unwrap();
            assert_eq!(r.rows[0], habitat::sample(&expected, 0).unwrap());
            assert!(d.manifest["runs"][i]["final_metrics"]
                .get("crop_progress")
                .is_some());
        }
        let report = d.report_at(1).unwrap();
        assert_eq!(report["branches"].as_array().unwrap().len(), 3);
        let layout = crate::habitat_graph::Layout {metric: "crop_progress".into(),..Default::default()};
        let f = d.figure(1, 2, &layout).unwrap();
        assert_eq!(f.exact_samples[0], d.recordings[1].rows);
        assert_eq!(
            f.single_design_context.as_ref().unwrap()["manifest"],
            d.manifest
        );
        assert_eq!(
            f.single_design_context.as_ref().unwrap()["selected_branches"],
            json!([1, 2])
        );
        assert!(f.crop_trajectory_context.is_some());
        assert!(f.configuration_assumptions.is_some());
        assert!(d.figure(3, 0, &layout).is_err());
        assert!(d.report_at(3).is_err());
        let ordinary =
            crate::habitat_figure::Figure::ordinary(&d.recordings[1], None, &layout).unwrap();
        assert!(serde_json::to_value(ordinary)
            .unwrap()
            .get("single_design_context")
            .is_none());
        d.export_trajectories(&path.join("export")).unwrap();
        assert!(d.export_trajectories(&path.join("export")).is_err());
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), before);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn single_reader_refuses_changed_completion_sources_plan_catalog_and_planned_times() {
        let (path, _, _) = fixture("tamper", false);
        let bytes = std::fs::read(path.join("manifest.json")).unwrap();
        for (field, value) in [
            ("complete", json!(false)),
            ("schema_version", json!(2)),
            ("parameter", json!("heater_limit_w")),
            ("common_state_fingerprint", json!("changed")),
            ("common_data_versions", json!({})),
            ("runs", json!([])),
            ("configuration_assumptions", json!({"fake":true})),
            ("factorial_analysis", json!({})),
        ] {
            let mut m: Value = serde_json::from_slice(&bytes).unwrap();
            m[field] = value;
            std::fs::write(path.join("manifest.json"), serde_json::to_vec(&m).unwrap()).unwrap();
            assert!(read_single(&path).is_err(), "accepted {field}");
        }
        let mut m: Value = serde_json::from_slice(&bytes).unwrap();
        m["runs"][1]["final_metrics"]["water"]["value"] = json!(-1.);
        std::fs::write(path.join("manifest.json"), serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(read_single(&path).is_err());
        std::fs::write(path.join("manifest.json"), bytes).unwrap();
        let sample = path.join("run-001/samples.json");
        let mut rows: Vec<habitat::Sample> =
            serde_json::from_slice(&std::fs::read(&sample).unwrap()).unwrap();
        rows[1].simulation_time_s += 0.5;
        std::fs::write(&sample, serde_json::to_vec(&rows).unwrap()).unwrap();
        std::fs::write(
            path.join("run-001/samples.csv"),
            habitat::samples_csv(&rows),
        )
        .unwrap();
        assert!(Recording::read(&path.join("run-001")).is_ok());
        assert!(read_single(&path).err().unwrap().contains("planned times"));
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn single_replay_checks_every_branch_and_reports_plausible_interior_changes() {
        let (path, _, _) = fixture("replay", false);
        let original = crate::habitat_replay::verify(&path).unwrap();
        assert_eq!(original["verified"], true);
        assert_eq!(original["checked_rows"], 9);
        assert_eq!(original["branches"].as_array().unwrap().len(), 3);
        let sample = path.join("run-001/samples.json");
        let mut rows: Vec<habitat::Sample> =
            serde_json::from_slice(&std::fs::read(&sample).unwrap()).unwrap();
        rows[1].habitat.state.potable_water_kg -= 1.;
        rows[1].habitat.state.wastewater_kg += 1.;
        std::fs::write(&sample, serde_json::to_vec(&rows).unwrap()).unwrap();
        std::fs::write(
            path.join("run-001/samples.csv"),
            habitat::samples_csv(&rows),
        )
        .unwrap();
        assert!(read_single(&path).is_ok());
        let report = crate::habitat_replay::verify(&path).unwrap();
        assert_eq!(report["verified"], false);
        assert_eq!(report["sample_mismatch_count"], 1);
        assert_eq!(report["mismatches"][0]["branch_index"], 1);
        assert_eq!(report["mismatches"][0]["observation_index"], 1);
        let before = std::fs::read(&sample).unwrap();
        let mut visits = 0;
        assert!(crate::habitat_replay::verify_with(&path, |_| {
            visits += 1;
            Err("cancel".into())
        })
        .is_err());
        assert!(visits > 0);
        assert_eq!(std::fs::read(&sample).unwrap(), before);
        std::fs::remove_dir_all(path).unwrap();
    }
}
