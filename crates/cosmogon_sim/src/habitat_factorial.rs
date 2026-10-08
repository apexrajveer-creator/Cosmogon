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
    plan.validate()?;
    let secondary = plan
        .secondary
        .as_ref()
        .ok_or("This is not a paired experiment")?;
    if recordings.len() != 4 || recordings.iter().any(|r| r.rows.len() != plan.samples) {
        return Err("A paired experiment requires four complete branch recordings".into());
    }
    let index = plan.samples - 1;
    let reference = &recordings[0];
    let time = reference.rows[index].simulation_time_s;
    let metrics = Metric::ALL.into_iter().map(|metric| {
        let mut supported = true;
        let responses = std::array::from_fn(|i| {
            let pairs = habitat_comparison::pairs(reference,&recordings[i],metric,Alignment::World);
            let pair = &pairs[index];
            supported &= pair.support=="matched" && pair.b_index==Some(index);
            pair.b_value
        });
        (metric.id().into(),json!({"unit":metric.unit(),"support":if supported {"matched"} else {"unavailable_or_mismatched_interval"},"contrasts":Contrasts::of(responses,supported)}))
    }).collect::<serde_json::Map<String,Value>>();
    Ok(
        json!({"format":"cosmogon-habitat-factorial-analysis","schema_version":1,"method_source":METHOD_SOURCE,
        "a":{"parameter":plan.parameter.id(),"unit":plan.parameter.unit(),"levels":plan.values},
        "b":{"parameter":secondary.parameter.id(),"unit":secondary.parameter.unit(),"levels":secondary.values},
        "order":[[0,0],[1,0],[0,1],[1,1]],"completed_interval_configurations_match_current":recordings.iter().map(|r|r.rows[index].habitat.budget.configuration.as_ref()==Some(&r.rows[index].habitat.config)).collect::<Vec<_>>(),"observation_index":index,"simulation_time_s":time,"metrics":metrics,
        "formulas":{"a_at_b0":"Y10 - Y00","a_at_b1":"Y11 - Y01","b_at_a0":"Y01 - Y00","b_at_a1":"Y11 - Y10","average_a":"(A at B0 + A at B1)/2","average_b":"(B at A0 + B at A1)/2","difference_in_differences":"(Y11 - Y01) - (Y10 - Y00)","standard_ab_effect":"difference_in_differences / 2"},
        "scope":"Final stored observation of four common-state branches; initial stores fixed. A0/A1 and B0/B1 mean first/second supplied levels, not necessarily increasing values. Responses remain visible when contrasts lack support.",
        "limits":"Conditional model contrasts at these two levels. No interpolation, p-values, significance, confidence intervals, independent noise replications, global optimum or field-hardware validation. First-versus-last corners change both settings and cannot isolate either effect."}),
    )
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
        for metric in Metric::ALL {
            if runs[i]["final_metrics"][metric.id()]
                != json!({"unit":metric.unit(),"value":metric.value(recording.rows.last().ok_or("Empty design branch")?)})
            {
                return Err("Design metric disagrees with recorded final observation".into());
            }
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
}
