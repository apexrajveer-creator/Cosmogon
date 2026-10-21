//! Endpoint inventory and cumulative electrical accounting over actual retained observations.
//! Sparse sampled interval budgets are never integrated into a claimed full history.
use crate::{
    habitat::{Sample, State, Totals},
    habitat_recording::Recording,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{fmt::Write as _, io::Write, path::Path};
pub const LIMITS:&str="Endpoint changes and retained cumulative model counters, not measured hardware or reconstructed intermediate history. Escaped gas remains outside the usable cabin. Latest sampled minute budgets are not integrated over the observation span. Retained configuration/counter checks do not rule out edits or unsupported behavior between stored observations. Electrical/mass closure is numerical bookkeeping, not field calibration, independent replay, physiological survival or food adequacy.";
#[derive(Clone, Debug, Serialize)]
pub struct Quantity {
    pub id: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub initial: f64,
    pub final_value: f64,
    pub change: f64,
    pub scope: &'static str,
}
fn quantity(
    id: &'static str,
    label: &'static str,
    unit: &'static str,
    a: f64,
    b: f64,
    scope: &'static str,
) -> Quantity {
    Quantity {
        id,
        label,
        unit,
        initial: a,
        final_value: b,
        change: b - a,
        scope,
    }
}
fn stocks(s: &State) -> Vec<(&'static str, &'static str, f64)> {
    let g = s.escaped_gas.unwrap_or_default();
    let mut values=vec![
        ("oxygen", "Cabin O₂", s.oxygen_kg),
        ("nitrogen", "Cabin N₂", s.nitrogen_kg),
        ("co2", "Cabin CO₂", s.co2_kg),
        ("oxygen_reserve", "O₂ reserve", s.oxygen_reserve_kg),
        ("potable_water", "Potable water", s.potable_water_kg),
        ("wastewater", "Wastewater", s.wastewater_kg),
        ("brine", "Brine", s.brine_kg),
        ("food_dry", "Stored dry food", s.food_dry_kg),
        ("food_water", "Food water", s.food_water_kg),
        ("dry_waste", "Dry waste", s.dry_waste_kg),
        ("captured_co2", "Captured CO₂", s.captured_co2_kg),
        ("hydrogen", "Isolated H₂", s.hydrogen_kg),
        ("methane", "Isolated methane", s.methane_kg),
        (
            "sabatier_water",
            "Held raw reaction water",
            s.sabatier_water_kg,
        ),
        ("vapor", "Cabin vapor", s.water_vapor_kg),
        ("condensate", "Isolated condensate", s.condensate_kg),
        ("escaped_oxygen", "Escaped O₂", g.oxygen_kg),
        ("escaped_nitrogen", "Escaped N₂", g.nitrogen_kg),
        ("escaped_co2", "Escaped CO₂", g.co2_kg),
        ("escaped_vapor", "Escaped vapor", g.vapor_kg),
        ("battery", "Stored battery energy", s.battery_j),
    ];
    if let Some(c)=&s.crop {values.extend([("crop_nutrients","Crop nutrient-equivalent",c.nutrient_equivalent_kg),("crop_standing_edible","Standing edible dry material",c.standing_dry_kg[0]),("crop_standing_inedible","Standing inedible dry material",c.standing_dry_kg[1]),("crop_harvested_edible","Harvested edible dry material",c.harvested_dry_kg[0]),("crop_harvested_inedible","Harvested inedible dry material",c.harvested_dry_kg[1])]);}
    values
}
fn counters(t: &Totals) -> [(&'static str, &'static str, &'static str, f64); 15] {
    [
        (
            "generated",
            "Generated electrical energy",
            "J",
            t.generated_j,
        ),
        (
            "delivered",
            "Delivered electrical energy",
            "J",
            t.delivered_j,
        ),
        ("losses", "Electrical conversion losses", "J", t.losses_j),
        (
            "curtailed",
            "Curtailed electrical energy",
            "J",
            t.curtailed_j,
        ),
        ("unserved", "Unserved requested energy", "J", t.unserved_j),
        (
            "served_crew",
            "Served nominal crew-cycle person-seconds",
            "person·s",
            t.metabolic_person_seconds,
        ),
        (
            "unserved_crew",
            "Unserved nominal crew-cycle person-seconds",
            "person·s",
            t.unserved_person_seconds,
        ),
        (
            "electrolysis_enthalpy",
            "Cumulative electrolysis enthalpy debit",
            "J",
            t.reaction_enthalpy_j,
        ),
        (
            "sabatier_co2",
            "Cumulative CO₂ processed by reduction",
            "kg",
            t.sabatier_co2_consumed_kg,
        ),
        (
            "sabatier_heat",
            "Cumulative reduction reference heat",
            "J",
            t.sabatier_heat_j,
        ),
        (
            "sabatier_water_processed",
            "Cumulative raw reaction water processed",
            "kg",
            t.sabatier_water_processed_kg,
        ),
        (
            "condensate_processed",
            "Cumulative condensate processed",
            "kg",
            t.condensate_processed_kg,
        ),
        (
            "crew_vapor",
            "Cumulative crew moisture released",
            "kg",
            t.crew_vapor_kg,
        ),
        (
            "condensed",
            "Cumulative vapor condensed",
            "kg",
            t.condensed_kg,
        ),
        (
            "breach_enthalpy",
            "Cumulative escaped gas reference enthalpy",
            "J",
            t.breach_enthalpy_j,
        ),
    ]
}
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub format: &'static str,
    pub schema_version: u32,
    pub recording: Value,
    pub from_index: usize,
    pub to_index: usize,
    pub initial: Sample,
    pub final_sample: Sample,
    pub world_span_s: f64,
    pub completed_span_s: f64,
    pub stocks: Vec<Quantity>,
    pub cumulative_counters: Vec<Quantity>,
    pub mass: Value,
    pub electricity: Value,
    pub retained_support: Value,
    pub thermal: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat: Option<crate::habitat_heat::Span>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_unavailable: Option<String>,
    pub limits: &'static str,
}
pub fn explain(r: &Recording, from: usize, to: usize) -> Result<Report, String> {
    if r.rows.is_empty() || r.rows.len() > 1024 || from > to {
        return Err("Choose ordered retained observations within a 1–1024-row recording".into());
    }
    let a = r
        .rows
        .get(from)
        .ok_or("Initial observation is outside the recording")?;
    let b = r
        .rows
        .get(to)
        .ok_or("Final observation is outside the recording")?;
    for (i, row) in r.rows[from..=to].iter().enumerate() {
        row.habitat.validate(row.simulation_time_s)?;
        if row.habitat.id != a.habitat.id
            || row.habitat.definition != a.habitat.definition
            || (i > 0 && row.simulation_time_s <= r.rows[from + i - 1].simulation_time_s)
        {
            return Err(
                "Retained balance observations must keep identity and increasing world times"
                    .into(),
            );
        }
    }
    let sa = stocks(&a.habitat.state);
    let sb = stocks(&b.habitat.state);
    if sa.len()!=sb.len() {return Err("Crop endpoint stock availability changes within retained range".into());}
    let quantities: Vec<_> = sa
        .iter()
        .zip(&sb)
        .map(|(&(id, label, x), &(_, _, y))| {
            quantity(
                id,
                label,
                if id == "battery" { "J" } else { "kg" },
                x,
                y,
                if id.starts_with("escaped_") {
                    "outside_cabin_unavailable_for_reuse"
                } else if id == "battery" {
                    "electrical_storage"
                } else {
                    "retained_cabin_or_isolated_store"
                },
            )
        })
        .collect();
    let ca = counters(&a.habitat.totals);
    let cb = counters(&b.habitat.totals);
    let cumulative_counters: Vec<_> = ca
        .iter()
        .zip(&cb)
        .map(|(&(id, label, unit, x), &(_, _, _, y))| {
            quantity(
                id,
                label,
                unit,
                x,
                y,
                "difference_of_retained_cumulative_counters",
            )
        })
        .collect();
    let same_config = r.rows[from..=to]
        .iter()
        .all(|s| s.habitat.config == a.habitat.config);
    let monotone = r.rows[from..=to].windows(2).all(|pair| {
        counters(&pair[0].habitat.totals)
            .iter()
            .zip(counters(&pair[1].habitat.totals))
            .all(|(a, b)| b.3 >= a.3)
    });
    let capacity_constant = r.rows[from..=to]
        .iter()
        .all(|s| s.habitat.config.heat_capacity_j_k == a.habitat.config.heat_capacity_j_k);
    let temperature_change = b.habitat.state.temperature_k - a.habitat.state.temperature_k;
    let change = |index: usize| cumulative_counters[index].change;
    let battery = a.habitat.state.battery_j - b.habitat.state.battery_j;
    let input = change(0) + battery;
    let output = change(1) + change(2) + change(3);
    let cabin_a = a.habitat.state.mass_kg();
    let cabin_b = b.habitat.state.mass_kg();
    let outside_a = a
        .habitat
        .state
        .escaped_gas
        .map(|g| g.mass_kg())
        .unwrap_or(0.);
    let outside_b = b
        .habitat
        .state
        .escaped_gas
        .map(|g| g.mass_kg())
        .unwrap_or(0.);
    let stock_sum = quantities
        .iter()
        .filter(|q| q.unit == "kg")
        .map(|q| q.change)
        .sum::<f64>();
    let accounted_change =
        b.habitat.state.accounted_mass_kg() - a.habitat.state.accounted_mass_kg();
    let heat = crate::habitat_heat::span(&r.rows[from..=to]);
    let (heat, heat_unavailable) = match heat {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(e)),
    };
    Ok(Report {
        format: "cosmogon-habitat-recorded-balance",
        schema_version: 1,
        recording: r.summary.clone(),
        from_index: from,
        to_index: to,
        initial: a.clone(),
        final_sample: b.clone(),
        world_span_s: b.simulation_time_s - a.simulation_time_s,
        completed_span_s: b.habitat.last_time - a.habitat.last_time,
        stocks: quantities,
        mass: json!({"cabin_initial_kg":cabin_a,"cabin_final_kg":cabin_b,"cabin_change_kg":cabin_b-cabin_a,"outside_initial_kg":outside_a,"outside_final_kg":outside_b,"outside_change_kg":outside_b-outside_a,"accounted_change_kg":accounted_change,"component_change_sum_kg":stock_sum,"component_sum_residual_kg":stock_sum-accounted_change,"endpoint_accounted_drift_kg":accounted_change,"formula":"accounted cabin/isolated stores + cumulative escaped gas; component change sum - endpoint accounted change","scope":"Absolute endpoint drift retains possible explicit inventory edits, opt-in inward breach replenishment and floating-point accumulation; escaped mass is not available to cabin consumers"}),
        electricity: json!({"generated_j":change(0),"battery_withdrawal_j":battery,"delivered_j":change(1),"losses_j":change(2),"curtailed_j":change(3),"unserved_requested_j":change(4),"input_j":input,"output_j":output,"residual_j":input-output,"formula":"delta generated + initial battery - final battery - delta delivered - delta losses - delta curtailed","retained_configuration_constant":same_config,"retained_counters_monotone":monotone,"scope":"Differences of actual cumulative counters; no sampled-power extrapolation or unexplained residual reassignment"}),
        retained_support: json!({"observations_checked":to-from+1,"uncompleted_budget_count":r.rows[from..=to].iter().filter(|s|s.habitat.budget.seconds<=0.).count(),"configurations_equal":same_config,"counter_sequence_monotone":monotone,"sampled_environment_unavailable_count":r.rows[from..=to].iter().filter(|s|s.habitat.budget.seconds>0. && !s.habitat.budget.environment_available).count(),"sampled_temperature_unsupported_count":r.rows[from..=to].iter().filter(|s|s.habitat.budget.seconds>0. && !s.habitat.budget.temperature_domain_supported).count(),"coverage":"Support checks describe stored observations only, not every intermediate step"}),
        thermal: json!({"initial_temperature_k":a.habitat.state.temperature_k,"final_temperature_k":b.habitat.state.temperature_k,"temperature_change_k":temperature_change,"retained_heat_capacity_constant":capacity_constant,"initial_capacity_j_k":a.habitat.config.heat_capacity_j_k,"final_capacity_j_k":b.habitat.config.heat_capacity_j_k,"sensible_change_j":capacity_constant.then_some(a.habitat.config.heat_capacity_j_k*temperature_change),"initial_phase_reference_j":a.habitat.state.phase_energy_j(),"final_phase_reference_j":b.habitat.state.phase_energy_j(),"phase_reference_change_j":b.habitat.state.phase_energy_j()-a.habitat.state.phase_energy_j(),"formula":"C × (final T - initial T), only for constant retained C; phase reference change = vapor change × model reference latent enthalpy","scope":"Endpoint reference storage changes, not absolute thermodynamic energy or a full-span heat closure. Sparse sampled minute heat terms do not supply a complete cumulative ledger"}),
        cumulative_counters,
        heat,
        heat_unavailable,
        limits: LIMITS,
    })
}
#[derive(Clone, Debug, Serialize)]
pub struct Bundle {
    pub format: &'static str,
    pub schema_version: u32,
    pub mode: &'static str,
    pub reports: Vec<Report>,
    pub paired_context: Option<Value>,
    pub limits: &'static str,
}
impl Bundle {
    pub fn ordinary(r: &Recording, from: usize, to: usize) -> Result<Self, String> {
        Ok(Self {
            format: "cosmogon-habitat-balance-bundle",
            schema_version: 1,
            mode: "ordinary_recording",
            reports: vec![explain(r, from, to)?],
            paired_context: None,
            limits: LIMITS,
        })
    }
    pub fn paired(
        d: &crate::habitat_factorial::Design,
        from: usize,
        to: usize,
    ) -> Result<Self, String> {
        let plan = d.plan()?;
        plan.validate()?;
        if d.recordings.len() != 4 {
            return Err("Paired balance requires all four branches".into());
        }
        Ok(Self {
            format: "cosmogon-habitat-balance-bundle",
            schema_version: 1,
            mode: "four_independent_branch_balances",
            reports: d
                .recordings
                .iter()
                .map(|r| explain(r, from, to))
                .collect::<Result<_, _>>()?,
            paired_context: Some(
                json!({"plan":plan,"order":[[0,0],[1,0],[0,1],[1,1]],"common_state_fingerprint":d.manifest["common_state_fingerprint"],"configuration_assumptions":plan.configuration_assumptions(),"scope":"Independent branch ledgers; first/last-corner differences do not isolate either setting"}),
            ),
            limits: LIMITS,
        })
    }
    pub fn read(path: &Path, from: Option<usize>, to: Option<usize>) -> Result<Self, String> {
        if path.join("plan.json").exists() {
            match crate::habitat_design::read(path)? {
                crate::habitat_design::Design::Paired(d)=>{
                    let last=d.recordings[0].rows.len()-1;
                    Self::paired(&d,from.unwrap_or(0),to.unwrap_or(last))
                },
                crate::habitat_design::Design::Single(d)=>{
                    let r=&d.recordings[0];
                    Self::ordinary(r,from.unwrap_or(0),to.unwrap_or(r.rows.len()-1))
                },
            }
        } else {
            let r = Recording::read(path)?;
            let last = r
                .rows
                .len()
                .checked_sub(1)
                .ok_or("No stored observations")?;
            Self::ordinary(&r, from.unwrap_or(0), to.unwrap_or(last))
        }
    }
    pub fn export(&self, path: &Path) -> Result<Value, String> {
        if !matches!(
            (self.mode, self.reports.len()),
            ("ordinary_recording", 1) | ("four_independent_branch_balances", 4)
        ) {
            return Err("A balance bundle needs one ordinary or all four paired reports".into());
        }
        for r in &self.reports {
            if r.stocks.len() != stocks(&r.initial.habitat.state).len()
                || r.cumulative_counters.len() != 15
                || r.from_index > r.to_index
                || r.to_index >= 1024
                || r.stocks.iter().chain(&r.cumulative_counters).any(|q| {
                    !q.initial.is_finite() || !q.final_value.is_finite() || !q.change.is_finite()
                })
            {
                return Err("Invalid balance quantities or observation range".into());
            }
        }
        for r in &self.reports {
            if let Some(h) = &r.heat {
                if h.terms.len() != if r.final_sample.habitat.state.crop.is_some() {13} else {11}
                    || h.terms.iter().any(|t| {
                        !t.initial_j.is_finite()
                            || !t.final_j.is_finite()
                            || !t.change_j.is_finite()
                    })
                    || !h.heat_minus_storage_residual_j.is_finite()
                {
                    return Err("Invalid heat ledger export quantities".into());
                }
            }
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err("Balance bundle exceeds 64 MiB".into());
        }
        let mut stocks_csv = String::from("branch,id,unit,initial,final,change,scope\n");
        let mut counters_csv = stocks_csv.clone();
        for (i, r) in self.reports.iter().enumerate() {
            for (out, rows) in [
                (&mut stocks_csv, &r.stocks),
                (&mut counters_csv, &r.cumulative_counters),
            ] {
                for q in rows {
                    let _ = writeln!(
                        out,
                        "{i},{},{},{:.17e},{:.17e},{:.17e},{}",
                        q.id, q.unit, q.initial, q.final_value, q.change, q.scope
                    );
                }
            }
        }
        let mut heat_csv = String::from("branch,id,unit,initial,final,change,scope\n");
        let has_heat = self.reports.iter().any(|r| r.heat.is_some());
        for (i, r) in self.reports.iter().enumerate() {
            if let Some(h) = &r.heat {
                for t in &h.terms {
                    let _ = writeln!(
                        heat_csv,
                        "{i},{},J,{:.17e},{:.17e},{:.17e},actual_executed_interval_counters",
                        t.id, t.initial_j, t.final_j, t.change_j
                    );
                }
            }
        }
        let mut manifest = json!({"format":"cosmogon-habitat-balance-export","schema_version":1,"complete":true,"mode":self.mode,"branches":self.reports.len(),"files":["report.json","stocks.csv","counters.csv"],"limits":LIMITS});
        if has_heat {
            manifest["files"]
                .as_array_mut()
                .unwrap()
                .push(json!("heat.csv"));
            manifest["heat_accounting_source"] = crate::habitat_heat::source();
        }
        if self.reports.iter().any(|r|r.final_sample.habitat.uses_crop()) {manifest["crop_source"]=crate::crop_process::source();}
        if path.as_os_str().is_empty() {
            return Err("Choose a nonempty new balance folder".into());
        }
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| {
            format!("Existing balance reports are protected; choose a new folder: {e}")
        })?;
        let mut files = vec![
            ("report.json", bytes),
            ("stocks.csv", stocks_csv.into_bytes()),
            ("counters.csv", counters_csv.into_bytes()),
        ];
        if has_heat {
            files.push(("heat.csv", heat_csv.into_bytes()));
        }
        files.push(("manifest.json", manifest_bytes));
        for (name, data) in files {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .map_err(|e| format!("Incomplete balance report: {e}"))?;
            f.write_all(&data)
                .map_err(|e| format!("Incomplete balance report: {e}"))?;
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::habitat::{Definition, Environment, Habitat};
    use crate::BodyRef;
    fn recording() -> Recording {
        let mut h = Habitat::new(
            0,
            Definition::at(BodyRef { system: 0, body: 0 }, 0., 0.),
            1000.,
        )
        .unwrap();
        let mut rows = Vec::new();
        for i in 0..4 {
            if i > 0 {
                h.step(
                    1000. + i as f64 * 60.,
                    Environment {
                        irradiance_w_m2: Some(0.),
                        exterior_temperature_k: Some(250.),
                        exterior_dose_rate_msv_day: None,
                    },
                )
                .unwrap();
            }
            rows.push(Sample {
                simulation_time_s: 1000. + i as f64 * 60.,
                habitat: h.clone(),
            });
        }
        Recording {
            rows,
            summary: json!({"name":"Hostile <script>&\"' cabin","source":"actual retained identity"}),
        }
    }

    #[test]
    fn cumulative_energy_and_all_stocks_match_independent_arithmetic() {
        let r = recording();
        let b = Bundle::ordinary(&r, 0, 3).unwrap();
        let e = &b.reports[0];
        let a = &r.rows[0].habitat;
        let z = &r.rows[3].habitat;
        let residual = (z.totals.generated_j - a.totals.generated_j) + a.state.battery_j
            - z.state.battery_j
            - (z.totals.delivered_j - a.totals.delivered_j)
            - (z.totals.losses_j - a.totals.losses_j)
            - (z.totals.curtailed_j - a.totals.curtailed_j);
        assert!((e.electricity["residual_j"].as_f64().unwrap() - residual).abs() < 1e-8);
        assert!(residual.abs() < 1e-6);
        assert_eq!(e.stocks.len(), 21);
        assert_eq!(e.cumulative_counters.len(), 15);
        assert_eq!(e.initial, r.rows[0]);
        assert_eq!(e.final_sample, r.rows[3]);
        let mass = |s: &State| {
            [
                s.oxygen_kg,
                s.nitrogen_kg,
                s.co2_kg,
                s.oxygen_reserve_kg,
                s.potable_water_kg,
                s.wastewater_kg,
                s.brine_kg,
                s.food_dry_kg,
                s.food_water_kg,
                s.dry_waste_kg,
                s.captured_co2_kg,
                s.hydrogen_kg,
                s.methane_kg,
                s.sabatier_water_kg,
                s.water_vapor_kg,
                s.condensate_kg,
            ]
            .into_iter()
            .sum::<f64>()
                + s.escaped_gas
                    .map_or(0., |g| g.oxygen_kg + g.nitrogen_kg + g.co2_kg + g.vapor_kg)
        };
        assert_eq!(
            e.mass["accounted_change_kg"].as_f64().unwrap(),
            mass(&z.state) - mass(&a.state)
        );
        assert_eq!(
            e.thermal["sensible_change_j"].as_f64().unwrap(),
            a.config.heat_capacity_j_k * (z.state.temperature_k - a.state.temperature_k)
        );
        assert_eq!(e.retained_support["uncompleted_budget_count"], 1);
        assert_eq!(
            e.retained_support["sampled_temperature_unsupported_count"],
            0
        );
        // A complete counter identity is independent of arbitrary latest interval watt estimates.
        let mut changed = r.clone();
        changed.rows[3].habitat.budget.generated_w += 12345.;
        assert_eq!(explain(&changed, 0, 3).unwrap().electricity, e.electricity);
        assert_eq!(
            explain(&r, 2, 2)
                .unwrap()
                .cumulative_counters
                .iter()
                .map(|q| q.change)
                .sum::<f64>(),
            0.
        );
        assert!(explain(&r, 3, 1).is_err());
        assert!(explain(&r, 0, 4).is_err());
    }
    #[test]
    fn discontinuities_remain_visible_and_capacity_change_has_no_invented_heat() {
        let mut r = recording();
        r.rows[1].habitat.config.heat_capacity_j_k *= 2.;
        r.rows[2].habitat.totals.generated_j = 0.;
        let e = explain(&r, 0, 3).unwrap();
        assert_eq!(e.retained_support["configurations_equal"], false);
        assert_eq!(e.retained_support["counter_sequence_monotone"], false);
        assert!(e.thermal["sensible_change_j"].is_null());
        r.rows[2].simulation_time_s = r.rows[1].simulation_time_s;
        assert!(explain(&r, 0, 3).is_err());
    }
    #[test]
    fn optional_isolated_stores_and_escaped_gas_are_separate_from_usable_cabin() {
        let mut r = recording();
        // Valid optional states come from explicit installed definitions, not invented exports.
        let mut definition = r.rows[0].habitat.definition.clone();
        definition.config.moisture = Some(crate::moisture::Config {
            relative_humidity_setpoint: 0.,
            ..Default::default()
        });
        definition.config.electrolysis = Some(crate::electrolysis::Config {
            hydrogen_capacity_kg: 10.,
            ..Default::default()
        });
        definition.config.sabatier = Some(crate::sabatier::Config {
            methane_capacity_kg: 10.,
            water_capacity_kg: 10.,
            ..Default::default()
        });
        definition.config.breach = Some(crate::breach::Config {
            area_m2: 1e-8,
            ..Default::default()
        });
        let mut h = Habitat::new(0, definition, 1000.).unwrap();
        r.rows.clear();
        for i in 0..4 {
            if i > 0 {
                h.step(
                    1000. + i as f64 * 60.,
                    Environment {
                        irradiance_w_m2: Some(0.),
                        exterior_temperature_k: Some(250.),
                        exterior_dose_rate_msv_day: None,
                    },
                )
                .unwrap();
            }
            r.rows.push(Sample {
                simulation_time_s: 1000. + i as f64 * 60.,
                habitat: h.clone(),
            });
        }
        let e = explain(&r, 0, 3).unwrap();
        assert!(
            e.stocks
                .iter()
                .find(|q| q.id == "hydrogen")
                .unwrap()
                .final_value
                > 0.
        );
        assert!(
            e.cumulative_counters
                .iter()
                .find(|q| q.id == "condensed")
                .unwrap()
                .final_value
                > 0.
        );
        assert!(e
            .stocks
            .iter()
            .filter(|q| q.id.starts_with("escaped_"))
            .all(|q| q.scope == "outside_cabin_unavailable_for_reuse"));
        assert!(e.mass["component_sum_residual_kg"].as_f64().unwrap().abs() < 1e-9);
        assert!(
            e.stocks
                .iter()
                .find(|q| q.id == "escaped_oxygen")
                .unwrap()
                .final_value
                > 0.
        );
    }
    #[test]
    fn protected_export_preserves_exact_inputs_numeric_csv_and_completion_manifest() {
        let r = recording();
        let b = Bundle::ordinary(&r, 0, 3).unwrap();
        let before = serde_json::to_vec(&r.rows).unwrap();
        let path =
            std::env::temp_dir().join(format!("cosmogon-balance-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        b.export(&path).unwrap();
        let v: Value =
            serde_json::from_slice(&std::fs::read(path.join("report.json")).unwrap()).unwrap();
        assert_eq!(
            v["reports"][0]["initial"],
            serde_json::to_value(&r.rows[0]).unwrap()
        );
        for (file, key) in [
            ("stocks.csv", "stocks"),
            ("counters.csv", "cumulative_counters"),
        ] {
            let text = std::fs::read_to_string(path.join(file)).unwrap();
            for (line, q) in text
                .lines()
                .skip(1)
                .zip(v["reports"][0][key].as_array().unwrap())
            {
                let fields: Vec<_> = line.split(',').collect();
                assert_eq!(fields.len(), 7);
                assert_eq!(fields[1], q["id"].as_str().unwrap());
                for (i, key) in [(3, "initial"), (4, "final_value"), (5, "change")] {
                    assert_eq!(fields[i].parse::<f64>().unwrap(), q[key].as_f64().unwrap());
                }
            }
        }
        assert!(b.export(&path).is_err());
        assert_eq!(before, serde_json::to_vec(&r.rows).unwrap());
        assert_eq!(
            serde_json::from_slice::<Value>(&std::fs::read(path.join("manifest.json")).unwrap())
                .unwrap()["complete"],
            true
        );
        let _ = std::fs::remove_dir_all(&path);
    }
    #[test]
    fn paired_keeps_every_independent_ledger_and_actual_branch_context() {
        use crate::habitat_factorial::{Design, Secondary};
        use crate::habitat_sweep::{Parameter, Plan};
        let mut plan = Plan::linear(Parameter::VolumeM3, 250., 1000., 2, 4, 60.).unwrap();
        plan.schema_version = 2;
        plan.secondary = Some(Secondary {
            parameter: Parameter::HeatCapacityJK,
            values: [1e7, 1e8],
        });
        let recordings = (0..4)
            .map(|branch| {
                let mut r = recording();
                r.summary = json!({"branch":branch,"source":"retained-test-source"});
                r.rows[3].habitat.state.battery_j -= branch as f64;
                r
            })
            .collect();
        let mut d = Design {
            recordings,
            manifest: json!({"plan":plan,"common_state_fingerprint":"actual-common"}),
            analysis: Value::Null,
        };
        let b = Bundle::paired(&d, 0, 3).unwrap();
        assert_eq!(b.reports.len(), 4);
        assert_eq!(b.mode, "four_independent_branch_balances");
        assert_eq!(
            b.paired_context.as_ref().unwrap()["order"],
            json!([[0, 0], [1, 0], [0, 1], [1, 1]])
        );
        for (i, r) in b.reports.iter().enumerate() {
            assert_eq!(r.final_sample, d.recordings[i].rows[3]);
            assert_eq!(r.recording["branch"], i);
            assert!((r.electricity["residual_j"].as_f64().unwrap() - i as f64).abs() < 1e-8);
        }
        d.recordings.pop();
        assert!(Bundle::paired(&d, 0, 3).is_err());
    }
}
