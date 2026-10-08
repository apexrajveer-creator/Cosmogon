//! Compare actual stored observations. Exact time/interval support, no interpolation.
use crate::habitat_recording::{Metric, Recording};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    #[default]
    Elapsed,
    World,
}
impl Alignment {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "elapsed" => Ok(Self::Elapsed),
            "world" => Ok(Self::World),
            _ => Err("Alignment must be elapsed or world".into()),
        }
    }
    pub fn coordinate(self, time: f64, first: f64) -> f64 {
        match self {
            Self::Elapsed => time - first,
            Self::World => time,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pair {
    pub a_index: usize,
    pub b_index: Option<usize>,
    pub coordinate_s: f64,
    pub a_world_time_s: f64,
    pub b_world_time_s: Option<f64>,
    pub a_value: Option<f64>,
    pub b_value: Option<f64>,
    pub b_minus_a: Option<f64>,
    pub support: &'static str,
}
/// One exact stored observation; shared by pair plots and bounded factorial curves.
pub fn pair_at(
    a: &Recording,
    b: &Recording,
    metric: Metric,
    alignment: Alignment,
    i: usize,
) -> Option<Pair> {
    let ta = a.rows.first()?.simulation_time_s;
    let tb = b.rows.first()?.simulation_time_s;
    let ar = a.rows.get(i)?;
    let t = alignment.coordinate(ar.simulation_time_s, ta);
    let index = b
        .rows
        .binary_search_by(|br| alignment.coordinate(br.simulation_time_s, tb).total_cmp(&t))
        .ok();
    let br = index.map(|j| &b.rows[j]);
    let av = metric.value(ar);
    let bv = br.and_then(|r| metric.value(r));
    let same_interval = br.is_some_and(|r| {
        ar.habitat.budget.seconds > 0.0
            && ar.habitat.budget.seconds == r.habitat.budget.seconds
            && alignment.coordinate(ar.habitat.last_time, ta)
                == alignment.coordinate(r.habitat.last_time, tb)
    });
    let support = if br.is_none() {
        "no_exact_timestamp"
    } else if metric.is_interval() && !same_interval {
        "different_interval_support"
    } else if av.is_none() || bv.is_none() {
        "unavailable_value"
    } else {
        "matched"
    };
    Some(Pair {
        a_index: i,
        b_index: index,
        coordinate_s: t,
        a_world_time_s: ar.simulation_time_s,
        b_world_time_s: br.map(|r| r.simulation_time_s),
        a_value: av,
        b_value: bv,
        b_minus_a: if support == "matched" {
            av.zip(bv).map(|(x, y)| y - x)
        } else {
            None
        },
        support,
    })
}
pub fn pairs(a: &Recording, b: &Recording, metric: Metric, alignment: Alignment) -> Vec<Pair> {
    (0..a.rows.len())
        .filter_map(|i| pair_at(a, b, metric, alignment, i))
        .collect()
}

pub fn report(
    a: &Recording,
    b: &Recording,
    metric: Metric,
    alignment: Alignment,
) -> serde_json::Value {
    let rows = pairs(a, b, metric, alignment);
    let ah = &a.rows[0].habitat;
    let bh = &b.rows[0].habitat;
    serde_json::json!({"format":"cosmogon-habitat-comparison","schema_version":1,"metric":metric.id(),"unit":metric.unit(),"alignment":alignment,"pairs":rows,"matched_value_count":rows.iter().filter(|r|r.b_minus_a.is_some()).count(),"a":a.summary,"b":b.summary,"a_initial_conditions":{"definition":ah.definition,"configuration":ah.config,"state":ah.state},"b_initial_conditions":{"definition":bh.definition,"configuration":bh.config,"state":bh.state},"initial_conditions_differ":ah.definition!=bh.definition || ah.config!=bh.config || ah.state!=bh.state,"source_versions_differ":a.summary["initial_dataset_versions"]!=b.summary["initial_dataset_versions"] || a.summary["electrolysis_source"]!=b.summary["electrolysis_source"] || a.summary["sabatier_source"]!=b.summary["sabatier_source"],"limits":"Exact stored times only; interval differences require equal completed durations and aligned interval ends. No interpolation, extrapolation, causal attribution, confidence interval or field validation. Elapsed alignment may compare different physical epochs."})
}
pub fn export(
    a: &Recording,
    b: &Recording,
    metric: Metric,
    alignment: Alignment,
    path: &std::path::Path,
) -> Result<(), String> {
    use std::io::Write;
    let report =
        serde_json::to_vec_pretty(&report(a, b, metric, alignment)).map_err(|e| e.to_string())?;
    let rows = pairs(a, b, metric, alignment);
    let mut csv=String::from("a_index,b_index,coordinate_s,a_world_time_s,b_world_time_s,a_value,b_value,b_minus_a,support\n");
    let cell = |v: Option<f64>| v.map(|v| format!("{v:.17e}")).unwrap_or_default();
    for r in rows {
        csv.push_str(&format!(
            "{},{},{:.17e},{:.17e},{},{},{},{},{}\n",
            r.a_index,
            r.b_index.map(|v| v.to_string()).unwrap_or_default(),
            r.coordinate_s,
            r.a_world_time_s,
            cell(r.b_world_time_s),
            cell(r.a_value),
            cell(r.b_value),
            cell(r.b_minus_a),
            r.support
        ));
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(path)
        .map_err(|e| format!("Existing comparisons are protected; choose a new folder: {e}"))?;
    for (name, bytes) in [("comparison.json", report), ("pairs.csv", csv.into_bytes())] {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| format!("Incomplete comparison: {e}"))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Incomplete comparison: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn recording(times: &[f64], values: &[f64]) -> Recording {
        let h = crate::habitat::Habitat::new(
            0,
            crate::habitat::Definition::at(crate::BodyRef { system: 0, body: 0 }, 0.0, 0.0),
            0.0,
        )
        .unwrap();
        Recording {
            rows: times
                .iter()
                .zip(values)
                .map(|(&t, &v)| {
                    let mut h = h.clone();
                    h.state.potable_water_kg = v;
                    h.last_time = t;
                    h.budget.seconds = 60.0;
                    h.budget.environment_available = true;
                    h.budget.temperature_domain_supported = true;
                    h.budget.delivered_w = v;
                    crate::habitat::Sample {
                        simulation_time_s: t,
                        habitat: h,
                    }
                })
                .collect(),
            summary: serde_json::json!({}),
        }
    }
    #[test]
    fn exact_alignment_never_subtracts_nearest_or_extrapolated_values() {
        let a = recording(&[100., 160., 220.], &[10., 9., 8.]);
        let b = recording(&[200., 260.], &[11., 7.]);
        let p = pairs(&a, &b, Metric::Water, Alignment::Elapsed);
        assert_eq!(p[0].b_minus_a, Some(1.));
        assert_eq!(p[1].b_minus_a, Some(-2.));
        assert_eq!(p[2].b_index, None);
        assert_eq!(p[2].b_minus_a, None);
        assert!(pairs(&a, &b, Metric::Water, Alignment::World)
            .iter()
            .all(|p| p.b_index.is_none()));
        let mut b = recording(&[100., 160., 220.], &[11., 7., 8.]);
        b.rows[1].simulation_time_s += 0.000001;
        assert_eq!(
            pairs(&a, &b, Metric::Water, Alignment::World)[1].support,
            "no_exact_timestamp"
        );
    }
    #[test]
    fn interval_duration_and_completed_end_are_required_and_gaps_stay_empty() {
        let a = recording(&[100., 160., 220.], &[10., 9., 8.]);
        let mut b = a.clone();
        b.rows[0].habitat.budget.seconds = 30.;
        b.rows[1].habitat.last_time -= 1.;
        b.rows[2].habitat.budget.environment_available = false;
        let p = pairs(&a, &b, Metric::DeliveredPower, Alignment::World);
        assert_eq!(
            p.iter().map(|r| r.support).collect::<Vec<_>>(),
            vec![
                "different_interval_support",
                "different_interval_support",
                "unavailable_value"
            ]
        );
        assert!(p.iter().all(|r| r.b_minus_a.is_none()));
        assert!(pairs(&a, &b, Metric::Water, Alignment::World)
            .iter()
            .all(|r| r.b_minus_a == Some(0.)));
        let root = std::env::temp_dir().join(format!("cosmogon-comparison-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        export(&a, &b, Metric::DeliveredPower, Alignment::World, &root).unwrap();
        let before = std::fs::read(root.join("pairs.csv")).unwrap();
        assert!(export(&a, &b, Metric::Water, Alignment::Elapsed, &root).is_err());
        assert_eq!(std::fs::read(root.join("pairs.csv")).unwrap(), before);
        std::fs::remove_dir_all(root).unwrap();
    }
}
