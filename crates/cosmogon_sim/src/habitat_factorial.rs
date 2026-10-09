//! Four exact common-state responses; deterministic contrasts, not statistical inference.
use crate::{
    habitat_comparison::{self, Alignment},
    habitat_recording::{Metric, Recording},
    habitat_sweep::Plan,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
pub const METHOD_SOURCE: &str = "https://www.itl.nist.gov/div898/handbook/pri/section3/pri3331.htm";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Secondary {
    pub parameter: crate::habitat_sweep::Parameter,
    pub values: [f64; 2],
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Contrasts {
    pub responses: [Option<f64>; 4],
    pub a_at_b0: Option<f64>,
    pub a_at_b1: Option<f64>,
    pub b_at_a0: Option<f64>,
    pub b_at_a1: Option<f64>,
    pub average_a: Option<f64>,
    pub average_b: Option<f64>,
    pub difference_in_differences: Option<f64>,
    pub standard_ab_effect: Option<f64>,
}
impl Contrasts {
    pub fn of(responses: [Option<f64>; 4], supported: bool) -> Self {
        let all = supported
            .then_some(())
            .and_then(|_| Some([responses[0]?, responses[1]?, responses[2]?, responses[3]?]));
        let a0 = all.map(|r| r[1] - r[0]);
        let a1 = all.map(|r| r[3] - r[2]);
        let b0 = all.map(|r| r[2] - r[0]);
        let b1 = all.map(|r| r[3] - r[1]);
        let interaction = a0.zip(a1).map(|(a, b)| b - a);
        Self {
            responses,
            a_at_b0: a0,
            a_at_b1: a1,
            b_at_a0: b0,
            b_at_a1: b1,
            average_a: a0.zip(a1).map(|(a, b)| (a + b) / 2.0),
            average_b: b0.zip(b1).map(|(a, b)| (a + b) / 2.0),
            difference_in_differences: interaction,
            standard_ab_effect: interaction.map(|x| x / 2.0),
        }
    }
}
pub fn report(plan: &Plan, recordings: &[Recording]) -> Result<Value, String> {
    report_at(plan, recordings, plan.samples.saturating_sub(1))
}
pub fn report_at(plan: &Plan, recordings: &[Recording], index: usize) -> Result<Value, String> {
    plan.validate()?;
    let secondary = plan
        .secondary
        .as_ref()
        .ok_or("This is not a paired experiment")?;
    if recordings.len() != 4 || recordings.iter().any(|r| r.rows.len() != plan.samples) {
        return Err("A paired experiment requires four complete branch recordings".into());
    }
    if index >= plan.samples {
        return Err("Requested paired observation is outside the recording".into());
    }
    let reference = &recordings[0];
    let time = reference.rows[index].simulation_time_s;
    let metrics = plan.metrics(&reference.rows[index].habitat).into_iter().map(|metric| {
        let mut supported = true;
        let responses = std::array::from_fn(|i| {
            let pair = habitat_comparison::pair_at(reference,&recordings[i],metric,Alignment::World,index).expect("validated complete rows");
            supported &= pair.support=="matched" && pair.b_index==Some(index);
            pair.b_value
        });
        (metric.id().into(),json!({"unit":metric.unit(),"support":if supported {"matched"} else {"unavailable_or_mismatched_interval"},"contrasts":Contrasts::of(responses,supported)}))
    }).collect::<serde_json::Map<String,Value>>();
    Ok(plan.with_assumptions(
        json!({"format":"cosmogon-habitat-factorial-analysis","schema_version":1,"method_source":METHOD_SOURCE,
        "a":{"parameter":plan.parameter.id(),"unit":plan.parameter.unit(),"levels":plan.values},
        "b":{"parameter":secondary.parameter.id(),"unit":secondary.parameter.unit(),"levels":secondary.values},
        "order":[[0,0],[1,0],[0,1],[1,1]],"completed_interval_configurations_match_current":recordings.iter().map(|r|r.rows[index].habitat.budget.configuration.as_ref()==Some(&r.rows[index].habitat.config)).collect::<Vec<_>>(),"observation_index":index,"simulation_time_s":time,"metrics":metrics,
        "formulas":{"a_at_b0":"Y10 - Y00","a_at_b1":"Y11 - Y01","b_at_a0":"Y01 - Y00","b_at_a1":"Y11 - Y10","average_a":"(A at B0 + A at B1)/2","average_b":"(B at A0 + B at A1)/2","difference_in_differences":"(Y11 - Y01) - (Y10 - Y00)","standard_ab_effect":"difference_in_differences / 2"},
        "scope":if index==plan.samples-1 {"Final stored observation of four common-state branches; initial stores fixed. A0/A1 and B0/B1 mean first/second supplied levels, not necessarily increasing values. Responses remain visible when contrasts lack support."} else {"Selected stored observation of four common-state branches; initial stores fixed. A0/A1 and B0/B1 mean first/second supplied levels, not necessarily increasing values. Responses remain visible when contrasts lack support."},
        "limits":"Conditional model contrasts at these two levels. No interpolation, p-values, significance, confidence intervals, independent noise replications, global optimum or field-hardware validation. First-versus-last corners change both settings and cannot isolate either effect."}),
    ))
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    #[default]
    AverageA,
    AverageB,
    InteractionDifference,
}
impl Effect {
    pub const ALL: [Self; 3] = [Self::AverageA, Self::AverageB, Self::InteractionDifference];
    pub fn parse(s: &str) -> Result<Self, String> {
        Self::ALL.into_iter().find(|e| e.id() == s).ok_or_else(|| {
            "Paired effect must be average_a, average_b or difference_in_differences".into()
        })
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::AverageA => "average_a",
            Self::AverageB => "average_b",
            Self::InteractionDifference => "difference_in_differences",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::AverageA => "Average A change",
            Self::AverageB => "Average B change",
            Self::InteractionDifference => "Difference of conditional A changes",
        }
    }
    pub fn value(self, c: &Contrasts) -> Option<f64> {
        match self {
            Self::AverageA => c.average_a,
            Self::AverageB => c.average_b,
            Self::InteractionDifference => c.difference_in_differences,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Point {
    pub observation_index: usize,
    pub simulation_time_s: f64,
    pub branch_indices: [Option<usize>; 4],
    pub branch_support: [&'static str; 4],
    pub branch_world_time_s: [Option<f64>; 4],
    pub completed_interval_end_s: [Option<f64>; 4],
    pub completed_interval_seconds: [Option<f64>; 4],
    pub completed_interval_configurations_match_current: [bool; 4],
    pub contrasts: Contrasts,
}
/// Exact common-time observations only. Missing values/support remain gaps; no resampling.
pub fn curve(plan: &Plan, recordings: &[Recording], metric: Metric) -> Result<Vec<Point>, String> {
    plan.validate()?;
    if plan.secondary.is_none()
        || recordings.len() != 4
        || recordings.iter().any(|r| r.rows.len() != plan.samples)
    {
        return Err("Paired curves require four complete branch recordings".into());
    }
    let reference = &recordings[0];
    Ok((0..plan.samples)
        .map(|index| {
            let pairs: [habitat_comparison::Pair; 4] = std::array::from_fn(|i| {
                habitat_comparison::pair_at(
                    reference,
                    &recordings[i],
                    metric,
                    Alignment::World,
                    index,
                )
                .expect("validated rows")
            });
            let supported = pairs
                .iter()
                .all(|p| p.support == "matched" && p.b_index == Some(index));
            Point {
                observation_index: index,
                simulation_time_s: reference.rows[index].simulation_time_s,
                branch_indices: std::array::from_fn(|i| pairs[i].b_index),
                branch_support: std::array::from_fn(|i| if pairs[i].support=="matched" && pairs[i].b_index!=Some(index) {"different_observation_index"}else{pairs[i].support}),
                branch_world_time_s: std::array::from_fn(|i| pairs[i].b_world_time_s),
                completed_interval_end_s: std::array::from_fn(|i| {
                    pairs[i]
                        .b_index
                        .map(|j| recordings[i].rows[j].habitat.last_time)
                }),
                completed_interval_seconds: std::array::from_fn(|i| {
                    pairs[i]
                        .b_index
                        .map(|j| recordings[i].rows[j].habitat.budget.seconds)
                }),
                completed_interval_configurations_match_current: std::array::from_fn(|i| {
                    let h = &recordings[i].rows[index].habitat;
                    h.budget.configuration.as_ref() == Some(&h.config)
                }),
                contrasts: Contrasts::of(std::array::from_fn(|i| pairs[i].b_value), supported),
            }
        })
        .collect())
}
impl Design {
    pub fn plan(&self) -> Result<Plan, String> {
        serde_json::from_value(self.manifest["plan"].clone()).map_err(|e| e.to_string())
    }
    pub fn report_at(&self, index: usize) -> Result<Value, String> {
        report_at(&self.plan()?, &self.recordings, index)
    }
    pub fn curve(&self, metric: Metric) -> Result<Vec<Point>, String> {
        curve(&self.plan()?, &self.recordings, metric)
    }
    pub fn trajectory_report(&self) -> Result<Value, String> {
        let plan = self.plan()?;
        let metrics = plan.metrics(&self.recordings[0].rows[0].habitat).into_iter()
            .map(|m| {
                Ok((
                    m.id().into(),
                    json!({"unit":m.unit(),"points":curve(&plan,&self.recordings,m)?}),
                ))
            })
            .collect::<Result<serde_json::Map<String, Value>, String>>()?;
        Ok(plan.with_assumptions(
            json!({"format":"cosmogon-habitat-factorial-trajectories","schema_version":1,"plan":plan,
            "common_state_fingerprint":self.manifest["common_state_fingerprint"],"common_data_versions":self.manifest["common_data_versions"],
            "habitat_id":self.manifest["habitat_id"],"recordings":self.recordings.iter().map(|r|&r.summary).collect::<Vec<_>>(),
            "method_source":METHOD_SOURCE,"order":self.analysis["order"],"formulas":self.analysis["formulas"],"limits":self.analysis["limits"],
            "scope":"All exact stored common-world observations. Interval contrasts require matching positive completed durations and ends. Missing values remain null or empty CSV cells. Snapshot flows may retain pre-edit configurations; flags are retained. No interpolation or live-state substitution.","final_analysis":self.analysis,"metrics":metrics}),
        ))
    }
    pub fn export_trajectories(&self, path: &Path) -> Result<(), String> {
        use std::io::Write;
        // Validate and serialize before creating output; the completion marker is written last.
        let report = self.trajectory_report()?;
        let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
        let plan = self.plan()?;
        let mut csv=String::from("metric,unit,observation_index,simulation_time_s,y00,y10,y01,y11,a_at_b0,a_at_b1,b_at_a0,b_at_a1,average_a,average_b,difference_in_differences,standard_ab_effect,support00,support10,support01,support11,index00,index10,index01,index11,world_time00_s,world_time10_s,world_time01_s,world_time11_s,interval_end00_s,interval_end10_s,interval_end01_s,interval_end11_s,interval_seconds00,interval_seconds10,interval_seconds01,interval_seconds11,interval_config_matches00,interval_config_matches10,interval_config_matches01,interval_config_matches11\n");
        let cell = |v: Option<f64>| v.map(|x| format!("{x:.17e}")).unwrap_or_default();
        for m in plan.metrics(&self.recordings[0].rows[0].habitat) {
            for p in curve(&plan, &self.recordings, m)? {
                let c = &p.contrasts;
                let mut cells = vec![
                    m.id().into(),
                    m.unit().into(),
                    p.observation_index.to_string(),
                    format!("{:.17e}", p.simulation_time_s),
                ];
                cells.extend(c.responses.into_iter().map(cell));
                cells.extend(
                    [
                        c.a_at_b0,
                        c.a_at_b1,
                        c.b_at_a0,
                        c.b_at_a1,
                        c.average_a,
                        c.average_b,
                        c.difference_in_differences,
                        c.standard_ab_effect,
                    ]
                    .into_iter()
                    .map(cell),
                );
                cells.extend(p.branch_support.map(str::to_string));
                cells.extend(
                    p.branch_indices
                        .map(|i| i.map(|j| j.to_string()).unwrap_or_default()),
                );
                for v in [
                    p.branch_world_time_s,
                    p.completed_interval_end_s,
                    p.completed_interval_seconds,
                ] {
                    cells.extend(v.into_iter().map(cell));
                }
                cells.extend(
                    p.completed_interval_configurations_match_current
                        .map(|v| v.to_string()),
                );
                csv.push_str(&cells.join(","));
                csv.push('\n');
            }
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| {
            format!("Existing trajectory analyses are protected; choose a new folder: {e}")
        })?;
        for (name,data) in [("trajectories.json",bytes),("contrasts.csv",csv.into_bytes()),("manifest.json",serde_json::to_vec_pretty(&plan.with_assumptions(json!({"format":"cosmogon-habitat-factorial-trajectory-export","schema_version":1,"complete":true,"observation_count":plan.samples,"metric_count":plan.metrics(&self.recordings[0].rows[0].habitat).len(),"common_state_fingerprint":self.manifest["common_state_fingerprint"],"files":["trajectories.json","contrasts.csv"]}))).map_err(|e|e.to_string())?)] {
            let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(path.join(name)).map_err(|e|format!("Incomplete trajectory analysis: {e}"))?;
            f.write_all(&data).map_err(|e|format!("Incomplete trajectory analysis: {e}"))?;
        }
        Ok(())
    }
}

pub struct Design {
    pub recordings: Vec<Recording>,
    pub manifest: Value,
    pub analysis: Value,
}
pub fn read(path: &Path) -> Result<Design, String> {
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
        || manifest["schema_version"] != 2
        || manifest["complete"] != true
        || plan.secondary.is_none()
        || manifest["plan"] != serde_json::to_value(&plan).map_err(|e| e.to_string())?
    {
        return Err("Incomplete or unsupported paired design".into());
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
        .filter(|r| r.len() == 4)
        .ok_or("Design needs four complete runs")?;
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
            || runs[i]["value"] != plan.values[i % 2]
            || runs[i]["secondary_value"] != plan.secondary.as_ref().unwrap().values[i / 2]
            || runs[i]["recording"] != recording.summary
            || recording.summary["initial_state_fingerprint"]
                != crate::research::state_fingerprint(&expected)?
        {
            return Err("Design branch differs from its common-state ordinary edit".into());
        }
        let last=recording.rows.last().ok_or("Empty design branch")?;
        let expected_metrics=plan.metrics(&last.habitat).into_iter().map(|metric|
            (metric.id().into(),json!({"unit":metric.unit(),"value":metric.value(last)}))
        ).collect::<serde_json::Map<String,Value>>();
        if runs[i]["final_metrics"] != Value::Object(expected_metrics) {
            return Err("Design metric catalog disagrees with recorded final observation".into());
        }
        recordings.push(recording);
    }
    let analysis = report(&plan, &recordings)?;
    if read_json("factorial.json")? != analysis
        || manifest["factorial_analysis"] != analysis
        || manifest["parameter"] != plan.parameter.id()
        || manifest["unit"] != plan.parameter.unit()
    {
        return Err("Design analysis disagrees with actual recorded responses".into());
    }
    Ok(Design {
        recordings,
        manifest,
        analysis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{habitat, habitat_sweep::Parameter, sandbox::Edit, Universe};
    fn world() -> Universe {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut d = habitat::Definition::at(crate::BodyRef { system: 0, body: 3 }, 0.0, 0.0);
        d.config.moisture = Some(crate::moisture::Config::default());
        d.config.firm_power_w = 10_000.0;
        d.config.solar_area_m2 = 0.0;
        d.config.heater_limit_w = 0.0;
        d.config.radiator_area_m2 = 0.0;
        d.config.radiator_pump_w = 0.0;
        d.config.conductance_w_k = 0.0;
        d.config.heat_capacity_j_k = 1e9;
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        u.advance_to(u.time + 86400.0, None, None);
        u
    }
    fn plan() -> Plan {
        let mut p = Plan::linear(Parameter::MoisturePowerW, 0.0, 500.0, 2, 3, 3600.0).unwrap();
        p.schema_version = 2;
        p.secondary = Some(Secondary {
            parameter: Parameter::MoistureAirborneFraction,
            values: [0.2, 1.0],
        });
        p
    }
    #[test]
    fn independent_factorial_arithmetic_and_missing_support() {
        // Y=7+3*A+5*B+11*A*B at A=0,2 and B=0,4.
        let c = Contrasts::of([Some(7.0), Some(13.0), Some(27.0), Some(121.0)], true);
        assert_eq!(c.a_at_b0, Some(6.0));
        assert_eq!(c.a_at_b1, Some(94.0));
        assert_eq!(c.b_at_a0, Some(20.0));
        assert_eq!(c.b_at_a1, Some(108.0));
        assert_eq!(c.average_a, Some(50.0));
        assert_eq!(c.average_b, Some(64.0));
        assert_eq!(c.difference_in_differences, Some(88.0));
        assert_eq!(c.standard_ab_effect, Some(44.0));
        let missing = Contrasts::of([Some(7.0), None, Some(27.0), Some(121.0)], true);
        assert_eq!(missing.average_a, None);
        assert_eq!(missing.standard_ab_effect, None);
        assert_eq!(missing.responses[0], Some(7.0));
        let unsupported = Contrasts::of(c.responses, false);
        assert_eq!(unsupported.average_b, None);
        assert_eq!(unsupported.responses, c.responses);
    }
    #[test]
    fn four_atomic_branches_replay_and_reopen_with_verified_common_state() {
        let u = world();
        let p = plan();
        let before = crate::research::state_fingerprint(&u).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-habitat-factorial-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let manifest = crate::habitat_sweep::execute(&u, 0, &p, &root.join("a")).unwrap();
        crate::habitat_sweep::execute(&u, 0, &p, &root.join("b")).unwrap();
        let design = read(&root.join("a")).unwrap();
        assert_eq!(design.manifest, manifest);
        assert_eq!(design.analysis, read(&root.join("b")).unwrap().analysis);
        for (i, r) in design.recordings.iter().enumerate() {
            assert_eq!(r.rows[0].habitat.state, u.habitats[0].state);
            assert_eq!(r.rows[0].habitat.totals, u.habitats[0].totals);
            let mut c = u.habitats[0].config.clone();
            p.parameter.set(&mut c, p.values[i % 2]).unwrap();
            let b = p.secondary.as_ref().unwrap();
            b.parameter.set(&mut c, b.values[i / 2]).unwrap();
            assert_eq!(r.rows[0].habitat.config, c);
            for file in ["samples.json", "samples.csv", "report.json"] {
                assert_eq!(
                    std::fs::read(root.join("a").join(format!("run-{i:03}")).join(file)).unwrap(),
                    std::fs::read(root.join("b").join(format!("run-{i:03}")).join(file)).unwrap()
                );
            }
        }
        let values =
            std::array::from_fn(|i| Metric::Vapor.value(design.recordings[i].rows.last().unwrap()));
        assert_eq!(
            design.analysis["metrics"]["vapor"]["contrasts"],
            serde_json::to_value(Contrasts::of(values, true)).unwrap()
        );
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), before);
        assert!(crate::habitat_sweep::execute(&u, 0, &p, &root.join("a")).is_err());
        // A tampered summary cannot be substituted for actual branch observations.
        let file = root.join("a/manifest.json");
        let mut m: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        m["runs"][1]["value"] = 42.into();
        std::fs::write(&file, serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(read(&root.join("a")).is_err());
        let d = read(&root.join("b")).unwrap();
        let mut rows = d.recordings.clone();
        rows[2].rows[2].habitat.budget.seconds = 30.0;
        let analysis = report(&p, &rows).unwrap();
        assert_eq!(
            analysis["metrics"]["condensation_heat"]["contrasts"]["average_a"],
            Value::Null
        );
        assert_ne!(
            analysis["metrics"]["vapor"]["contrasts"]["average_a"],
            Value::Null
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn selectable_times_match_independent_arithmetic_and_interval_gaps() {
        let u = world();
        let p = plan();
        let mut recordings = (0..4)
            .map(|branch| Recording {
                rows: (0..3)
                    .map(|index| {
                        let mut h = u.habitats[0].clone();
                        // Independent response law: Y = 100 + t*(3*A - 5*B + 7*A*B).
                        let a = (branch % 2) as f64;
                        let b = (branch / 2) as f64;
                        let t = index as f64;
                        h.state.potable_water_kg = 100.0 + t * (3.0 * a - 5.0 * b + 7.0 * a * b);
                        h.budget.delivered_w = h.state.potable_water_kg;
                        h.budget.seconds = 60.0;
                        h.budget.environment_available = true;
                        h.last_time = 1000.0 + 60.0 * t;
                        crate::habitat::Sample {
                            simulation_time_s: h.last_time,
                            habitat: h,
                        }
                    })
                    .collect(),
                summary: json!({}),
            })
            .collect::<Vec<_>>();
        let points = curve(&p, &recordings, Metric::Water).unwrap();
        for (index, point) in points.iter().enumerate() {
            let t = index as f64;
            assert_eq!(point.contrasts.a_at_b0, Some(3.0 * t));
            assert_eq!(point.contrasts.a_at_b1, Some(10.0 * t));
            assert_eq!(point.contrasts.average_a, Some(6.5 * t));
            assert_eq!(point.contrasts.average_b, Some(-1.5 * t));
            assert_eq!(point.contrasts.difference_in_differences, Some(7.0 * t));
            let selected = report_at(&p, &recordings, index).unwrap();
            assert_eq!(selected["observation_index"], index);
            assert_eq!(
                selected["metrics"]["water"]["contrasts"],
                serde_json::to_value(&point.contrasts).unwrap()
            );
        }
        assert_eq!(
            report(&p, &recordings).unwrap(),
            report_at(&p, &recordings, 2).unwrap()
        );
        assert!(report_at(&p, &recordings, 3).is_err());
        recordings[1].rows[1].habitat.budget.seconds = 30.0;
        let interval = curve(&p, &recordings, Metric::DeliveredPower).unwrap();
        assert!(interval[0].contrasts.average_a.is_some());
        assert_eq!(interval[1].contrasts.average_a, None);
        assert_eq!(interval[1].branch_support[1], "different_interval_support");
        assert!(interval[1].contrasts.responses.iter().all(Option::is_some));
        assert!(interval[2].contrasts.average_a.is_some());
        assert!(curve(&p, &recordings, Metric::Water).unwrap()[1]
            .contrasts
            .average_a
            .is_some());
        recordings[2].rows[1].simulation_time_s += 0.001;
        let points = curve(&p, &recordings, Metric::Water).unwrap();
        assert_eq!(points[1].branch_indices[2], None);
        assert_eq!(points[1].contrasts.responses[2], None);
        assert_eq!(points[1].contrasts.average_b, None);
        assert!(points[2].contrasts.average_b.is_some());
    }
    #[test]
    fn trajectory_export_retains_support_identity_and_is_exact_and_protected() {
        let u = world();
        let p = plan();
        let root = std::env::temp_dir().join(format!(
            "cosmogon-factorial-trajectories-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        crate::habitat_sweep::execute(&u, 0, &p, &root.join("design")).unwrap();
        let d = read(&root.join("design")).unwrap();
        let before = std::fs::read(root.join("design/factorial.json")).unwrap();
        d.export_trajectories(&root.join("a")).unwrap();
        d.export_trajectories(&root.join("b")).unwrap();
        for name in ["trajectories.json", "contrasts.csv", "manifest.json"] {
            assert_eq!(
                std::fs::read(root.join("a").join(name)).unwrap(),
                std::fs::read(root.join("b").join(name)).unwrap()
            );
        }
        assert!(d.export_trajectories(&root.join("a")).is_err());
        assert_eq!(
            std::fs::read(root.join("design/factorial.json")).unwrap(),
            before
        );
        let report: Value =
            serde_json::from_slice(&std::fs::read(root.join("a/trajectories.json")).unwrap())
                .unwrap();
        assert_eq!(
            report["common_state_fingerprint"],
            d.manifest["common_state_fingerprint"]
        );
        assert_eq!(report["final_analysis"], d.analysis);
        for metric in Metric::for_habitat(&d.recordings[0].rows[0].habitat) {
            let points = &report["metrics"][metric.id()]["points"];
            assert_eq!(points.as_array().unwrap().len(), 3);
            assert_eq!(
                points[2]["contrasts"],
                d.analysis["metrics"][metric.id()]["contrasts"]
            );
        }
        assert!(report["metrics"]["water"]["points"][0]
            ["completed_interval_configurations_match_current"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == false));
        let csv = std::fs::read_to_string(root.join("a/contrasts.csv")).unwrap();
        assert_eq!(csv.lines().count(), 1 + 3 * Metric::for_habitat(&d.recordings[0].rows[0].habitat).count());
        assert!(csv.lines().all(|l| l.split(',').count() == 40));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_pairing_and_combined_inventory_edits_fail_before_output() {
        let u = world();
        let root = std::env::temp_dir().join(format!(
            "cosmogon-habitat-factorial-invalid-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let p = plan();
        let mut invalid = p.clone();
        invalid.secondary.as_mut().unwrap().parameter = p.parameter;
        assert!(invalid.validate().is_err());
        let mut invalid = p.clone();
        invalid.values.push(1.0);
        assert!(invalid.validate().is_err());
        let mut invalid = p.clone();
        invalid.secondary.as_mut().unwrap().values = [0.0, 0.0];
        assert!(invalid.validate().is_err());
        let mut invalid = p.clone();
        invalid.schema_version = 1;
        assert!(invalid.validate().is_err());
        let mut invalid = p.clone();
        invalid.secondary = Some(Secondary {
            parameter: Parameter::MoistureCondensateCapacityKg,
            values: [20.0, 0.0],
        });
        assert!(crate::habitat_sweep::execute(&u, 0, &invalid, &root).is_err());
        assert!(!root.exists());
        let mut invalid = p;
        invalid.secondary.as_mut().unwrap().values = [0.0, f64::NAN];
        assert!(invalid.validate().is_err());
        let legacy = Plan::linear(Parameter::FirmPowerW, 0.0, 1000.0, 3, 2, 60.0).unwrap();
        assert!(serde_json::to_value(&legacy)
            .unwrap()
            .get("secondary")
            .is_none());
        assert_eq!(legacy.branch_count(), 3);
    }
    #[test]
    fn volume_capacity_design_retains_pressure_interactions_sources_and_replay() {
        let u=world();
        let mut p=Plan::linear(Parameter::VolumeM3,250.,1000.,2,3,3600.).unwrap();
        p.schema_version=2;p.secondary=Some(Secondary {parameter:Parameter::HeatCapacityJK,values:[5e7,1e9]});
        let root=std::env::temp_dir().join(format!("cosmogon-volume-capacity-pair-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&root);
        crate::habitat_sweep::execute(&u,0,&p,&root.join("design")).unwrap();
        let d=read(&root.join("design")).unwrap();
        assert_eq!(d.analysis["metrics"].as_object().unwrap().len(),19);
        for index in 0..3 {
            let report=d.report_at(index).unwrap();
            let y:Vec<f64>=d.recordings.iter().map(|r|r.rows[index].habitat.state.pressure_pa(&r.rows[index].habitat.config)).collect();
            let c=&report["metrics"]["total_pressure"]["contrasts"];
            assert_eq!(c["responses"],json!(y));
            let did=(y[3]-y[2])-(y[1]-y[0]);
            assert_eq!(c["difference_in_differences"],did);
            assert_eq!(c["standard_ab_effect"],did/2.0);
            if index==0 {assert_eq!(did,0.0);} else {assert_ne!(did,0.0);}
        }
        assert_eq!(d.manifest["configuration_assumptions"],d.analysis["configuration_assumptions"]);
        d.export_trajectories(&root.join("export")).unwrap();
        let export:Value=serde_json::from_slice(&std::fs::read(root.join("export/manifest.json")).unwrap()).unwrap();
        assert_eq!(export["metric_count"],19);
        assert_eq!(export["configuration_assumptions"],d.analysis["configuration_assumptions"]);
        let csv=std::fs::read_to_string(root.join("export/contrasts.csv")).unwrap();
        assert_eq!(csv.lines().count(),58);assert!(csv.lines().all(|l|l.split(',').count()==40));
        assert!(d.export_trajectories(&root.join("export")).is_err());
        let replay=crate::habitat_replay::verify(&root.join("design")).unwrap();
        assert_eq!(replay["verified"],true);
        let file=root.join("design/manifest.json");let original=std::fs::read(&file).unwrap();
        for mutate in 0..2 {
            let mut bad:Value=serde_json::from_slice(&original).unwrap();
            if mutate==0 {bad.as_object_mut().unwrap().remove("configuration_assumptions");}
            else {bad["runs"][0]["final_metrics"]["total_pressure"]["value"]=1.0.into();}
            std::fs::write(&file,serde_json::to_vec(&bad).unwrap()).unwrap();assert!(read(&root.join("design")).is_err());
        }
        std::fs::write(file,original).unwrap();assert!(read(&root.join("design")).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }

}
