//! Read-only ideal-gas accounting between actual retained states. The ordered
//! decomposition is algebra, not unique causal attribution or missing-history replay.
use crate::{
    breach::{self, Gas, R},
    habitat::Sample,
    habitat_recording::Recording,
};
use serde::Serialize;
use std::{io::Write, path::Path};

pub const MODEL: &str = "cosmogon-recorded-pressure/1";
pub const SOURCE: &str = "https://www.grc.nasa.gov/www/k-12/airplane/eqstat.html";
pub const ORDER: &str = "inventory_then_temperature_then_volume";
pub const LIMITS: &str = "Ideal, uniformly mixed gas at the retained lumped temperature. Ordered mathematical accounting, not unique physical causation: another ordering assigns interactions differently. Stored observations do not reconstruct missing minutes or separate equipment causes. Modelled inputs, no measured atmosphere, quantified uncertainty, field calibration or safety assessment. Breach accounting covers only the retained resolved substep, not the full interval between observations.";

#[derive(Clone, Debug, Serialize)]
pub struct Component {
    pub id: &'static str,
    pub label: &'static str,
    pub mass_kg: f64,
    pub molar_mass_kg_mol: f64,
    pub moles: f64,
    pub mole_fraction: Option<f64>,
    pub partial_pressure_pa: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct State {
    pub components: Vec<Component>,
    pub total_moles: f64,
    pub temperature_k: f64,
    pub volume_m3: f64,
    pub total_pressure_pa: f64,
    pub component_sum_residual_pa: f64,
}
fn state(g: Gas, temperature_k: f64, volume_m3: f64) -> Result<State, String> {
    g.validate()?;
    if !temperature_k.is_finite()
        || temperature_k <= 0.0
        || !volume_m3.is_finite()
        || volume_m3 <= 0.0
    {
        return Err(
            "Pressure accounting requires finite positive absolute temperature and volume".into(),
        );
    }
    let total_moles = g.moles();
    let factor = R * temperature_k / volume_m3;
    let components: Vec<_> = [
        ("oxygen", "O₂", g.oxygen_kg, 0.0319988),
        ("nitrogen", "N₂", g.nitrogen_kg, 0.0280134),
        ("co2", "CO₂", g.co2_kg, 0.0440095),
        (
            "vapor",
            "Water vapor",
            g.vapor_kg,
            crate::moisture::WATER_MOLAR_KG,
        ),
    ]
    .into_iter()
    .map(|(id, label, mass_kg, molar_mass_kg_mol)| {
        let moles = mass_kg / molar_mass_kg_mol;
        Component {
            id,
            label,
            mass_kg,
            molar_mass_kg_mol,
            moles,
            mole_fraction: (total_moles > 0.0).then(|| moles / total_moles),
            partial_pressure_pa: moles * factor,
        }
    })
    .collect();
    let total_pressure_pa = total_moles * factor;
    if !factor.is_finite() || !total_pressure_pa.is_finite() {
        return Err("Pressure accounting exceeds finite numeric range".into());
    }
    let component_sum_residual_pa = total_pressure_pa
        - components
            .iter()
            .map(|c| c.partial_pressure_pa)
            .sum::<f64>();
    Ok(State {
        components,
        total_moles,
        temperature_k,
        volume_m3,
        total_pressure_pa,
        component_sum_residual_pa,
    })
}
#[derive(Clone, Debug, Serialize)]
pub struct Term {
    pub id: &'static str,
    pub label: &'static str,
    pub pressure_change_pa: f64,
    pub formula: &'static str,
}
#[derive(Clone, Debug, Serialize)]
pub struct Change {
    pub ordering: &'static str,
    pub summary: String,
    pub terms: Vec<Term>,
    pub pressure_change_pa: f64,
    pub term_sum_pa: f64,
    pub residual_pa: f64,
}
fn change(a: &State, b: &State) -> Change {
    let mut terms: Vec<_> = a
        .components
        .iter()
        .zip(&b.components)
        .map(|(a_component, b_component)| Term {
            id: a_component.id,
            label: a_component.label,
            pressure_change_pa: (b_component.moles - a_component.moles) * R * a.temperature_k
                / a.volume_m3,
            formula: "(n1_i - n0_i) R T0 / V0",
        })
        .collect();
    terms.push(Term {
        id: "temperature",
        label: "Temperature",
        pressure_change_pa: b.total_moles * R * (b.temperature_k - a.temperature_k) / a.volume_m3,
        formula: "n1 R (T1 - T0) / V0",
    });
    terms.push(Term {
        id: "volume",
        label: "Volume",
        pressure_change_pa: b.total_moles
            * R
            * b.temperature_k
            * (1.0 / b.volume_m3 - 1.0 / a.volume_m3),
        formula: "n1 R T1 (1/V1 - 1/V0)",
    });
    let term_sum_pa = terms.iter().map(|t| t.pressure_change_pa).sum();
    let pressure_change_pa = b.total_pressure_pa - a.total_pressure_pa;
    let direction = if pressure_change_pa > 0.0 {
        "higher than"
    } else if pressure_change_pa < 0.0 {
        "lower than"
    } else {
        "equal to"
    };
    let mut summary = format!("Selected pressure is {direction} the reference; signed difference {pressure_change_pa:.9} Pa.");
    for (positive, label) in [(true, "positive"), (false, "negative")] {
        if let Some(term) = terms
            .iter()
            .filter(|t| {
                if positive {
                    t.pressure_change_pa > 0.0
                } else {
                    t.pressure_change_pa < 0.0
                }
            })
            .max_by(|a, b| {
                a.pressure_change_pa
                    .abs()
                    .total_cmp(&b.pressure_change_pa.abs())
            })
        {
            summary.push_str(&format!(
                " Largest {label} term in this ordering: {} ({:+.9} Pa).",
                term.label, term.pressure_change_pa
            ));
        }
    }
    if b.total_moles < a.total_moles && pressure_change_pa > 0.0 {
        summary.push_str(" Pressure rose despite fewer gas molecules; temperature and volume also enter the equation.");
    }
    Change {
        ordering: ORDER,
        summary,
        terms,
        pressure_change_pa,
        term_sum_pa,
        residual_pa: pressure_change_pa - term_sum_pa,
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Observation {
    pub index: usize,
    pub sample: Sample,
    pub state: State,
    pub habitat_reported_pressure_pa: f64,
    pub habitat_pressure_residual_pa: f64,
}
fn observation(r: &Recording, index: usize) -> Result<Observation, String> {
    let sample = r
        .rows
        .get(index)
        .ok_or("Pressure observation/reference index is outside the recording")?;
    sample.habitat.validate(sample.simulation_time_s)?;
    let h = &sample.habitat;
    let state = state(
        h.state.cabin_gas(),
        h.state.temperature_k,
        h.config.volume_m3,
    )?;
    let habitat_reported_pressure_pa = h.state.pressure_pa(&h.config);
    let habitat_pressure_residual_pa = habitat_reported_pressure_pa - state.total_pressure_pa;
    Ok(Observation {
        index,
        sample: sample.clone(),
        state,
        habitat_reported_pressure_pa,
        habitat_pressure_residual_pa,
    })
}
#[derive(Clone, Debug, Serialize)]
pub struct Substep {
    pub scope: &'static str,
    pub requested_seconds: f64,
    pub resolved_seconds: f64,
    pub full_interval_supported: bool,
    pub completed_interval_end_s: f64,
    pub initial: State,
    pub final_state: State,
    pub change: Change,
    pub budget: breach::Budget,
}
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub format: &'static str,
    pub schema_version: u32,
    pub model: &'static str,
    pub equation: &'static str,
    pub gas_constant_j_mol_k: f64,
    pub source: serde_json::Value,
    pub recording: serde_json::Value,
    pub reference_mode: &'static str,
    pub reference: Option<Observation>,
    pub observation: Observation,
    pub change: Option<Change>,
    pub breach_substep: Option<Substep>,
    pub uncertainty: Option<f64>,
    pub limits: &'static str,
}
pub fn explain(
    r: &Recording,
    index: usize,
    reference_index: Option<usize>,
) -> Result<Report, String> {
    if r.rows.is_empty() || r.rows.len() > 1024 {
        return Err("Pressure accounting requires 1–1024 retained observations".into());
    }
    let observation = observation(r, index)?;
    let reference = reference_index
        .or_else(|| index.checked_sub(1))
        .map(|i| self::observation(r, i))
        .transpose()?;
    let change = reference
        .as_ref()
        .map(|a| self::change(&a.state, &observation.state));
    let h = &observation.sample.habitat;
    let breach_substep=h.budget.breach.as_ref().map(|b|->Result<Substep,String>{
        let c=h.budget.configuration.as_ref().ok_or("Breach substep configuration is unavailable")?;
        let g=b.initial_gas;
        let e=b.escaped;
        let i=b.inflow;
        let final_gas=Gas {oxygen_kg:g.oxygen_kg-e.oxygen_kg+i.oxygen_kg,nitrogen_kg:g.nitrogen_kg-e.nitrogen_kg+i.nitrogen_kg,co2_kg:g.co2_kg-e.co2_kg+i.co2_kg,vapor_kg:g.vapor_kg-e.vapor_kg+i.vapor_kg};
        let initial=state(g,b.initial_temperature_k,c.volume_m3)?;
        let final_state=state(final_gas,b.final_temperature_k,c.volume_m3)?;
        let change=self::change(&initial,&final_state);
        Ok(Substep {scope:"Resolved breach substep after the minute's process/thermal update; not the full observation interval",requested_seconds:h.budget.seconds,resolved_seconds:b.integrated_seconds,full_interval_supported:b.status.supported(),completed_interval_end_s:h.last_time,initial,final_state,change,budget:b.clone()})
    }).transpose()?;
    Ok(Report {
        format: "cosmogon-recorded-pressure-report",
        schema_version: 1,
        model: MODEL,
        equation: "p_i = n_i R T / V; p = sum(p_i)",
        gas_constant_j_mol_k: R,
        source: serde_json::json!({"url":SOURCE,"version":"NASA Glenn Equation of State, May 13 2021","use":"Ideal-gas equation; ordered decomposition is derived algebra, not a NASA causal model","molecular_masses":"Same assumed species molecular masses as the habitat model"}),
        recording: r.summary.clone(),
        reference_mode: if reference_index.is_some() {
            "explicit_stored_index"
        } else if reference.is_some() {
            "previous_stored_observation"
        } else {
            "previous_stored_observation_unavailable"
        },
        reference,
        observation,
        change,
        breach_substep,
        uncertainty: None,
        limits: LIMITS,
    })
}
/// Capture the immutable derived report at the request boundary. No import or write
/// into the source recording. The folder and each file are create-new protected.
pub fn export(report: &Report, path: &Path) -> Result<(), String> {
    let mut components=String::from("scope,state,species,mass_kg,molar_mass_kg_mol,moles,mole_fraction,partial_pressure_pa,temperature_k,volume_m3,total_pressure_pa,parent_observation_index,parent_world_time_s,parent_completed_habitat_time_s\n");
    let mut terms = String::from("scope,term,pressure_change_pa,formula\n");
    let mut add = |scope: &str, kind: &str, s: &State, parent: &Observation| {
        for c in &s.components {
            components.push_str(&format!(
                "{scope},{kind},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                c.id,
                c.mass_kg,
                c.molar_mass_kg_mol,
                c.moles,
                c.mole_fraction.map(|v| v.to_string()).unwrap_or_default(),
                c.partial_pressure_pa,
                s.temperature_k,
                s.volume_m3,
                s.total_pressure_pa,
                parent.index,
                parent.sample.simulation_time_s,
                parent.sample.habitat.last_time
            ));
        }
    };
    if let Some(a) = &report.reference {
        add("stored_observations", "reference", &a.state, a);
    }
    add(
        "stored_observations",
        "selected",
        &report.observation.state,
        &report.observation,
    );
    if let Some(b) = &report.breach_substep {
        add(
            "resolved_breach_substep",
            "initial",
            &b.initial,
            &report.observation,
        );
        add(
            "resolved_breach_substep",
            "final",
            &b.final_state,
            &report.observation,
        );
    }
    for (scope, change) in [
        ("stored_observations", report.change.as_ref()),
        (
            "resolved_breach_substep",
            report.breach_substep.as_ref().map(|b| &b.change),
        ),
    ] {
        if let Some(c) = change {
            for t in &c.terms {
                terms.push_str(&format!(
                    "{scope},{},{},\"{}\"\n",
                    t.id, t.pressure_change_pa, t.formula
                ));
            }
            for (id, v) in [
                ("pressure_change", c.pressure_change_pa),
                ("term_sum", c.term_sum_pa),
                ("residual", c.residual_pa),
            ] {
                terms.push_str(&format!("{scope},{id},{v},\"{}\"\n", c.ordering));
            }
        }
    }
    let manifest = serde_json::json!({"format":"cosmogon-recorded-pressure","schema_version":1,"model":MODEL,"complete":true,"observation_index":report.observation.index,"reference_index":report.reference.as_ref().map(|a|a.index),"source":report.source,"recording":report.recording,"files":["report.json","components.csv","terms.csv"],"scope":"Derived mathematical accounting with exact retained samples; no trajectory replay or field validation"});
    let files = [
        (
            "report.json",
            serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?,
        ),
        ("components.csv", components.into_bytes()),
        ("terms.csv", terms.into_bytes()),
        (
            "manifest.json",
            serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
        ),
    ];
    if files.iter().any(|(_, bytes)| bytes.len() > 1024 * 1024) {
        return Err("Pressure evidence exceeds the 1 MiB per-file bound".into());
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(path).map_err(|e| {
        format!("Existing pressure evidence is protected; choose a new folder: {e}")
    })?;
    for (name, bytes) in files {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| format!("Incomplete pressure export: {e}"))?;
        f.write_all(&bytes)
            .map_err(|e| format!("Incomplete pressure export: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{Definition, Environment, Habitat},
        BodyRef,
    };
    fn near(a: f64, b: f64) {
        assert!(
            (a - b).abs() < 1e-8 * a.abs().max(b.abs()).max(1.0),
            "{a} != {b}"
        );
    }
    fn gas(n: f64) -> Gas {
        Gas {
            oxygen_kg: n * 0.0319988,
            ..Gas::default()
        }
    }
    fn recording(breach: bool) -> Recording {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        if breach {
            d.config.breach = Some(breach::Config::default());
            d.config.moisture = Some(crate::moisture::Config::default());
        }
        let mut h = Habitat::new(0, d, 0.0).unwrap();
        let first = Sample {
            simulation_time_s: 0.0,
            habitat: h.clone(),
        };
        h.step(
            60.0,
            Environment {
                irradiance_w_m2: Some(0.0),
                exterior_temperature_k: Some(250.0),
                exterior_dose_rate_msv_day: None,
            },
        )
        .unwrap();
        Recording {
            rows: vec![
                first,
                Sample {
                    simulation_time_s: 60.0,
                    habitat: h,
                },
            ],
            summary: serde_json::json!({"initial_state_fingerprint":"test-input","source":"retained-test-identity"}),
        }
    }
    #[test]
    fn isolated_inventory_temperature_and_volume_match_independent_ideal_gas_laws() {
        let a = state(gas(10.0), 300.0, 2.0).unwrap();
        near(a.total_pressure_pa, 10.0 * 8.31446261815324 * 300.0 / 2.0);
        for (n, t, v, which, expected) in [
            (12.0, 300.0, 2.0, 0, 2.0 * R * 300.0 / 2.0),
            (10.0, 330.0, 2.0, 4, 10.0 * R * 30.0 / 2.0),
            (10.0, 300.0, 4.0, 5, -10.0 * R * 300.0 / 4.0),
        ] {
            let b = state(gas(n), t, v).unwrap();
            let c = change(&a, &b);
            near(c.terms[which].pressure_change_pa, expected);
            for (i, t) in c.terms.iter().enumerate() {
                if i != which {
                    near(t.pressure_change_pa, 0.0);
                }
            }
            near(c.residual_pa, 0.0);
        }
    }
    #[test]
    fn combined_species_changes_telescope_and_ordering_assigns_interactions_differently() {
        let g = Gas {
            oxygen_kg: 1.0,
            nitrogen_kg: 2.0,
            co2_kg: 0.1,
            vapor_kg: 0.01,
        };
        let h = Gas {
            oxygen_kg: 0.8,
            nitrogen_kg: 1.8,
            co2_kg: 0.2,
            vapor_kg: 0.04,
        };
        let a = state(g, 290.0, 500.0).unwrap();
        let b = state(h, 305.0, 600.0).unwrap();
        let c = change(&a, &b);
        near(
            c.pressure_change_pa,
            h.moles() * R * 305.0 / 600.0 - g.moles() * R * 290.0 / 500.0,
        );
        near(c.term_sum_pa, c.pressure_change_pa);
        near(c.residual_pa, 0.0);
        assert!(c.terms[0].pressure_change_pa < 0.0 && c.terms[3].pressure_change_pa > 0.0);
        // Temperature-first convention closes too, but gives a different inventory term.
        let alternative_temperature = g.moles() * R * (305.0 - 290.0) / 500.0;
        let alternative_inventory = (h.moles() - g.moles()) * R * 305.0 / 500.0;
        near(
            alternative_temperature + alternative_inventory + c.terms[5].pressure_change_pa,
            c.pressure_change_pa,
        );
        assert!((alternative_temperature - c.terms[4].pressure_change_pa).abs() > 1.0);
        near(
            b.components.iter().map(|c| c.mole_fraction.unwrap()).sum(),
            1.0,
        );
        near(b.component_sum_residual_pa, 0.0);
        let reversed = change(&b, &a);
        near(reversed.pressure_change_pa, -c.pressure_change_pa);
        near(reversed.residual_pa, 0.0);
    }
    #[test]
    fn zero_gas_is_real_zero_pressure_with_unavailable_fractions_and_bad_domains_are_refused() {
        let a = state(Gas::default(), 300.0, 500.0).unwrap();
        assert_eq!(a.total_pressure_pa, 0.0);
        assert!(a
            .components
            .iter()
            .all(|c| c.mole_fraction.is_none() && c.partial_pressure_pa == 0.0));
        near(change(&a, &a).residual_pa, 0.0);
        for (t, v) in [
            (0.0, 1.0),
            (f64::NAN, 1.0),
            (300.0, 0.0),
            (300.0, f64::INFINITY),
            (f64::MAX, f64::MIN_POSITIVE),
        ] {
            assert!(state(gas(10.0), t, v).is_err());
        }
        assert!(state(
            Gas {
                vapor_kg: -1.0,
                ..Gas::default()
            },
            300.0,
            1.0
        )
        .is_err());
    }
    #[test]
    fn retained_samples_distinguish_missing_previous_explicit_same_and_invalid_reference() {
        let r = recording(false);
        let before = serde_json::to_vec(&r.rows).unwrap();
        let first = explain(&r, 0, None).unwrap();
        assert!(first.reference.is_none() && first.change.is_none());
        assert_eq!(
            first.reference_mode,
            "previous_stored_observation_unavailable"
        );
        assert!(first.breach_substep.is_none());
        let same = explain(&r, 0, Some(0)).unwrap();
        assert_eq!(same.change.unwrap().pressure_change_pa, 0.0);
        let next = explain(&r, 1, None).unwrap();
        assert_eq!(next.reference.unwrap().sample, r.rows[0]);
        assert_eq!(next.observation.sample, r.rows[1]);
        assert_eq!(next.recording, r.summary);
        near(next.change.unwrap().residual_pa, 0.0);
        for (i, j) in [(2, None), (0, Some(2)), (usize::MAX, None)] {
            assert!(explain(&r, i, j).is_err());
        }
        assert!(explain(
            &Recording {
                rows: vec![],
                summary: serde_json::Value::Null
            },
            0,
            None
        )
        .is_err());
        assert_eq!(serde_json::to_vec(&r.rows).unwrap(), before);
    }
    #[test]
    fn breach_substep_uses_completed_inputs_even_after_current_volume_edit_and_at_partial_boundary()
    {
        let mut r = recording(true);
        let original = explain(&r, 1, None).unwrap();
        let sub = original.breach_substep.unwrap();
        let b = r.rows[1].habitat.budget.breach.as_ref().unwrap();
        near(sub.initial.total_pressure_pa, b.initial_pressure_pa);
        near(sub.final_state.total_pressure_pa, b.final_pressure_pa);
        near(sub.change.residual_pa, 0.0);
        assert_eq!(sub.resolved_seconds, b.integrated_seconds);
        assert!(sub.full_interval_supported);
        r.rows[1].habitat.config.volume_m3 *= 2.0;
        let edited = explain(&r, 1, None).unwrap();
        near(
            edited.observation.state.total_pressure_pa,
            original.observation.state.total_pressure_pa / 2.0,
        );
        assert_eq!(
            serde_json::to_value(&edited.breach_substep).unwrap(),
            serde_json::to_value(Some(&sub)).unwrap()
        );
        let mut boundary = recording(true);
        let mut h = boundary.rows[0].habitat.clone();
        h.config.breach.as_mut().unwrap().area_m2 = 1.0;
        h.step(
            60.0,
            Environment {
                irradiance_w_m2: Some(0.0),
                exterior_temperature_k: Some(250.0),
                exterior_dose_rate_msv_day: None,
            },
        )
        .unwrap();
        boundary.rows[1].habitat = h.clone();
        let partial = explain(&boundary, 1, None).unwrap().breach_substep.unwrap();
        assert!(!partial.full_interval_supported);
        assert!(partial.resolved_seconds < partial.requested_seconds);
        near(partial.change.residual_pa, 0.0);
    }
    #[test]
    fn protected_complete_exports_preserve_exact_inputs_and_distinguish_unavailable_changes() {
        let r = recording(true);
        let report = explain(&r, 1, None).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-pressure-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        export(&report, &root).unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("report.json")).unwrap()).unwrap();
        assert_eq!(
            json["observation"]["sample"],
            serde_json::to_value(&r.rows[1]).unwrap()
        );
        assert_eq!(
            json["reference"]["sample"],
            serde_json::to_value(&r.rows[0]).unwrap()
        );
        assert_eq!(json["recording"], r.summary);
        let m: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(m["complete"], true);
        assert_eq!(m["model"], MODEL);
        let bytes = std::fs::read(root.join("report.json")).unwrap();
        assert!(export(&report, &root).is_err());
        assert_eq!(bytes, std::fs::read(root.join("report.json")).unwrap());
        let first = root.join("first");
        export(&explain(&r, 0, None).unwrap(), &first).unwrap();
        assert_eq!(
            std::fs::read_to_string(first.join("terms.csv"))
                .unwrap()
                .lines()
                .count(),
            1
        );
        assert_eq!(
            std::fs::read_to_string(root.join("components.csv"))
                .unwrap()
                .lines()
                .count(),
            17
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
