//! Passive accounting of actual completed model intervals, never sampled reconstruction.
use crate::habitat::{Budget, Habitat, Sample};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub const MODEL: &str = "cosmogon-cabin-heat-accounting/1";
pub const SOURCE: &str = "https://www.grc.nasa.gov/www/k-12/airplane/specheat.html";
pub const TERMS: [(&str, &str); 11] = [
    ("bus", "Delivered bus heat"),
    ("battery", "Conversion loss heat"),
    ("metabolic", "Nominal metabolic heat"),
    ("electrolysis", "Electrolysis enthalpy debit"),
    ("sabatier", "Reduction reference heat"),
    ("crew_vapor", "Crew vapor phase debit"),
    ("condensation", "Condensation reference heat"),
    ("radiator", "Radiator rejection debit"),
    ("conduction", "Signed envelope conduction"),
    ("breach_cooling", "Breach lumped cooling debit"),
    ("storage", "Sum of interval C × ΔT"),
];
pub const LIMITS: &str = "Actual executed model intervals; signed joules enter the lumped cabin. Heater is included in bus heat. Breach lumped cooling is distinct from full outflow enthalpy. Storage is summed interval C×ΔT, not absolute thermodynamic energy; capacity edits do not model construction or heat transfer. Excluded intervals have no reconstructed heat. Numerical accounting, not calibrated hardware or whole-system thermodynamic closure.";
pub fn source() -> Value {
    json!({"model":MODEL,"url":SOURCE,"method":"Passive signed actual interval energy counters; sum heat paths minus summed interval C×ΔT","basis":"Process-dependent heat capacity; effective capacity is a user assumption","term_order":TERMS.iter().map(|(id,_)|id).collect::<Vec<_>>(),"units":"J; coverage seconds; capacity J/K","limits":LIMITS})
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub crop_storage_debits_j: Option<[f64;2]>,
    /// Optional [wall condensation heat released, −leak enthalpy removal] accumulation.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub wall_leak_storage_j: Option<[f64;2]>,
    pub model: String,
    pub origin_time_s: f64,
    pub executed_seconds: f64,
    pub environment_missing_seconds: f64,
    pub start_temperature_unsupported_seconds: f64,
    pub capacity_transitions: u64,
    pub last_executed_capacity_j_k: f64,
    pub terms_j: [f64; 11],
    pub accumulated_interval_residual_j: f64,
}
impl Ledger {
    pub fn new(time: f64, capacity: f64) -> Self {
        Self {
            crop_storage_debits_j: None,
            wall_leak_storage_j: None,
            model: MODEL.into(),
            origin_time_s: time,
            executed_seconds: 0.,
            environment_missing_seconds: 0.,
            start_temperature_unsupported_seconds: 0.,
            capacity_transitions: 0,
            last_executed_capacity_j_k: capacity,
            terms_j: [0.; 11],
            accumulated_interval_residual_j: 0.,
        }
    }
    pub(crate) fn exclude(&mut self, dt: f64, environment: bool) {
        if environment {
            self.start_temperature_unsupported_seconds += dt;
        } else {
            self.environment_missing_seconds += dt;
        }
    }
    pub(crate) fn record(&mut self, b: &Budget) {
        let c = b
            .configuration
            .as_ref()
            .expect("Completed habitat interval has a configuration");
        if c.heat_capacity_j_k != self.last_executed_capacity_j_k {
            self.capacity_transitions += 1;
        }
        self.last_executed_capacity_j_k = c.heat_capacity_j_k;
        self.executed_seconds += b.seconds;
        let moisture = b.moisture.as_ref();
        let energies = [
            b.delivered_w * b.seconds,
            b.battery_losses_w * b.seconds,
            b.metabolic_heat_w * b.seconds,
            -b.electrolysis
                .as_ref()
                .map_or(0., |v| v.reaction_enthalpy_j),
            b.sabatier.as_ref().map_or(0., |v| v.reaction_heat_j),
            -moisture.map_or(0., |v| v.crew_phase_energy_j),
            moisture.map_or(0., |v| v.condensation_heat_j),
            -b.radiator_rejection_w * b.seconds,
            b.conduction_w * b.seconds,
            -b.breach.as_ref().map_or(0., |v| v.lumped_cooling_j),
            b.thermal_storage_w * b.seconds,
        ];
        for (total, energy) in self.terms_j.iter_mut().zip(energies) {
            *total += energy;
        }
        if let Some(crop)=&b.crop {let terms=self.crop_storage_debits_j.get_or_insert([0.;2]);terms[0]-=crop.chemical_storage_j;terms[1]-=crop.vapor_phase_j;}
        if b.wall.is_some()||b.leak.is_some(){let terms=self.wall_leak_storage_j.get_or_insert([0.;2]);terms[0]+=b.wall.as_ref().map_or(0.,|w|w.condensation_heat_j);terms[1]-=b.leak.as_ref().map_or(0.,|l|l.cooling_j);}
        self.accumulated_interval_residual_j += b.thermal_residual_j;
    }
    pub fn heat_path_sum_j(&self)->f64 {let mut base=self.terms_j[..10].iter().sum::<f64>();base+=self.crop_storage_debits_j.map_or(0.,|t|t[0]+t[1]);base+=self.wall_leak_storage_j.map_or(0.,|t|t[0]+t[1]);base}
    pub fn validate(&self, time: f64) -> Result<(), String> {
        if self.crop_storage_debits_j.is_some_and(|v|v.iter().any(|x|!x.is_finite() || *x>0. || *x< -1e30)){return Err("Invalid crop chemical/phase heat debits".into());}
        if self.wall_leak_storage_j.is_some_and(|v|v.iter().any(|x|!x.is_finite() || x.abs()>1e30)){return Err("Invalid wall/leak heat accumulation".into());}
        if self.model != MODEL {
            return Err("Unsupported cabin heat accounting model".into());
        }
        if !time.is_finite()
            || !self.origin_time_s.is_finite()
            || self.origin_time_s > time
            || self.origin_time_s.abs() > 1e16
        {
            return Err("Invalid heat ledger origin".into());
        }
        let coverage = [
            self.executed_seconds,
            self.environment_missing_seconds,
            self.start_temperature_unsupported_seconds,
        ];
        if coverage
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=1e20).contains(v))
            || (coverage.iter().sum::<f64>() - (time - self.origin_time_s)).abs() > 0.002
        {
            return Err("Heat ledger coverage disagrees with completed clock".into());
        }
        if self.capacity_transitions > 1_000_000_000_000_000 {
            return Err("Invalid heat capacity transition count".into());
        }
        if !self.last_executed_capacity_j_k.is_finite()
            || !(1e6..=1e12).contains(&self.last_executed_capacity_j_k)
        {
            return Err("Invalid heat ledger capacity".into());
        }
        if self
            .terms_j
            .iter()
            .chain([&self.accumulated_interval_residual_j])
            .any(|v| !v.is_finite() || v.abs() > 1e30)
        {
            return Err("Invalid cumulative heat quantity".into());
        }
        for (i, v) in self.terms_j.iter().enumerate() {
            if (matches!(i, 0 | 1 | 2 | 4 | 6) && *v < 0.)
                || (matches!(i, 3 | 5 | 7 | 9) && *v > 0.)
            {
                return Err("Heat ledger term has an invalid sign".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Term {
    pub id: &'static str,
    pub label: &'static str,
    pub initial_j: f64,
    pub final_j: f64,
    pub change_j: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Span {
    pub source: Value,
    pub origin_time_s: f64,
    pub completed_span_s: f64,
    pub executed_seconds: f64,
    pub environment_missing_seconds: f64,
    pub start_temperature_unsupported_seconds: f64,
    pub complete_model_coverage: bool,
    pub capacity_transitions: u64,
    pub terms: Vec<Term>,
    pub heat_path_sum_j: f64,
    pub storage_sum_j: f64,
    pub heat_minus_storage_residual_j: f64,
    pub accumulated_interval_residual_change_j: f64,
    pub residual_convention: &'static str,
    pub limits: &'static str,
}
/// Return an unavailable reason without inventing counters for legacy or discontinuous rows.
pub fn span(rows: &[Sample]) -> Result<Span, String> {
    let a = rows.first().ok_or("No retained observations")?;
    let b = rows.last().unwrap();
    let first = a.habitat.heat.as_ref().ok_or(
        "This recording has no cumulative heat ledger; sampled minute budgets cannot supply it",
    )?;
    let last = b
        .habitat
        .heat
        .as_ref()
        .ok_or("Final heat ledger is unavailable")?;
    let mut previous = first;
    for row in rows {
        let l = row
            .habitat
            .heat
            .as_ref()
            .ok_or("Heat ledger is absent within retained range")?;
        l.validate(row.habitat.last_time)?;
        if l.origin_time_s != first.origin_time_s
            || l.model != first.model
            || l.capacity_transitions < previous.capacity_transitions
            || l.executed_seconds < previous.executed_seconds
            || l.environment_missing_seconds < previous.environment_missing_seconds
            || l.start_temperature_unsupported_seconds
                < previous.start_temperature_unsupported_seconds
        {
            return Err("Retained heat ledger coverage/origin is discontinuous".into());
        }
        for (i, (now, old)) in l.terms_j.iter().zip(previous.terms_j).enumerate() {
            if (matches!(i, 0 | 1 | 2 | 4 | 6) && *now < old)
                || (matches!(i, 3 | 5 | 7 | 9) && *now > old)
            {
                return Err("Retained heat counters reverse their signed accumulation".into());
            }
        }
        if previous.crop_storage_debits_j.is_some() && l.crop_storage_debits_j.is_none() {return Err("Crop heat ledger is absent within retained range".into());}
        let old=previous.crop_storage_debits_j.unwrap_or([0.;2]);let now=l.crop_storage_debits_j.unwrap_or([0.;2]);if now[0]>old[0] || now[1]>old[1]{return Err("Crop heat counters reverse their signed accumulation".into());}
        if previous.wall_leak_storage_j.is_some() && l.wall_leak_storage_j.is_none() {return Err("Wall/leak heat ledger is absent within retained range".into());}
        previous = l;
    }
    let mut terms = TERMS
        .iter()
        .enumerate()
        .map(|(i, (id, label))| Term {
            id,
            label,
            initial_j: first.terms_j[i],
            final_j: last.terms_j[i],
            change_j: last.terms_j[i] - first.terms_j[i],
        })
        .collect::<Vec<_>>();
    let mut heat = terms[..10].iter().map(|t| t.change_j).sum::<f64>();
    let storage = terms[10].change_j;
    if first.crop_storage_debits_j.is_some() || last.crop_storage_debits_j.is_some(){let a=first.crop_storage_debits_j.unwrap_or([0.;2]);let b=last.crop_storage_debits_j.unwrap_or([0.;2]);for (k,(id,label)) in [("crop_chemical","Crop dry chemical storage debit"),("crop_vapor","Crop vapor phase storage debit")].into_iter().enumerate(){let change=b[k]-a[k];heat+=change;terms.push(Term{id,label,initial_j:a[k],final_j:b[k],change_j:change});}}
    if first.wall_leak_storage_j.is_some() || last.wall_leak_storage_j.is_some(){let a=first.wall_leak_storage_j.unwrap_or([0.;2]);let b=last.wall_leak_storage_j.unwrap_or([0.;2]);for (k,(id,label)) in [("wall_condensation_heat","Wall condensation reference heat released"),("leak_cooling","Structural leak enthalpy removal")].into_iter().enumerate(){let change=b[k]-a[k];heat+=change;terms.push(Term{id,label,initial_j:a[k],final_j:b[k],change_j:change});}}
    let executed = last.executed_seconds - first.executed_seconds;
    let missing = last.environment_missing_seconds - first.environment_missing_seconds;
    let unsupported =
        last.start_temperature_unsupported_seconds - first.start_temperature_unsupported_seconds;
    Ok(Span{source:source_for(last),origin_time_s:first.origin_time_s,completed_span_s:b.habitat.last_time-a.habitat.last_time,executed_seconds:executed,environment_missing_seconds:missing,start_temperature_unsupported_seconds:unsupported,complete_model_coverage:missing==0. && unsupported==0.,capacity_transitions:last.capacity_transitions-first.capacity_transitions,terms,heat_path_sum_j:heat,storage_sum_j:storage,heat_minus_storage_residual_j:heat-storage,accumulated_interval_residual_change_j:last.accumulated_interval_residual_j-first.accumulated_interval_residual_j,residual_convention:"Span residual is heat minus storage; saved interval residual uses storage minus heat. Independent accumulation may differ by floating-point roundoff.",limits:LIMITS})
}
fn source_for(l:&Ledger)->Value {
    let mut v=source();
    if l.crop_storage_debits_j.is_some(){
        v["optional_crop_storage"]=json!({"model":crate::crop_process::MODEL,"terms":["crop_chemical","crop_vapor"],"sign":"Negative chemical and vapor storage debits; original eleven-term array preserved"});
    }
    if l.wall_leak_storage_j.is_some() {
        v["optional_wall_leak"]=json!({"models":[crate::moisture::WALL_MODEL,crate::habitat::LEAK_MODEL],"terms":["wall_condensation_heat","leak_cooling"],"sign":"Signed wall condensation heat release and structural leak enthalpy removal; original eleven-term array preserved"});
    }
    v
}
pub fn habitat_span(h: &Habitat) -> Value {
    match &h.heat {
        Some(l) => json!({"ledger":l,"source":source_for(l)}),
        None => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{Definition, Environment},
        BodyRef,
    };
    fn habitat() -> Habitat {
        Habitat::new(
            0,
            Definition::at(BodyRef { system: 0, body: 0 }, 0., 0.),
            1000.,
        )
        .unwrap()
    }
    fn env(t: f64) -> Environment {
        Environment {
            irradiance_w_m2: Some(0.),
            exterior_temperature_k: Some(t),
        }
    }
    fn row(h: &Habitat) -> Sample {
        Sample {
            simulation_time_s: h.last_time,
            habitat: h.clone(),
        }
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-4_f64.max(1e-12 * a.abs()), "{a} vs {b}");
    }
    #[test]
    fn actual_intervals_sum_independently_and_preserve_physics() {
        let mut h = habitat();
        let mut d = h.definition.clone();
        d.record_heat = false;
        let mut legacy = Habitat::new(0, d, 1000.).unwrap();
        let mut rows = vec![row(&h)];
        let mut independent = [0.; 11];
        let initial_t = h.state.temperature_k;
        let c = h.config.heat_capacity_j_k;
        for i in 1..=40 {
            let time = 1000. + i as f64 * 60.;
            let exterior = if i % 3 == 0 { 350. } else { 210. };
            h.step(time, env(exterior)).unwrap();
            legacy.step(time, env(exterior)).unwrap();
            assert_eq!(h.state, legacy.state);
            assert_eq!(h.totals, legacy.totals);
            assert_eq!(h.budget, legacy.budget);
            let b = &h.budget;
            independent[0] += b.delivered_w * 60.;
            independent[1] += b.battery_losses_w * 60.;
            independent[2] += b.metabolic_heat_w * 60.;
            independent[7] -= b.radiator_rejection_w * 60.;
            independent[8] += b.conduction_w * 60.;
            independent[10] += b.thermal_storage_w * 60.;
            if i % 10 == 0 {
                rows.push(row(&h));
            }
        }
        assert_eq!(h.heat.as_ref().unwrap().terms_j, independent);
        close(independent[10], c * (h.state.temperature_k - initial_t));
        let s = span(&rows).unwrap();
        assert_eq!(s.executed_seconds, 2400.);
        assert!(s.complete_model_coverage);
        close(s.heat_path_sum_j, s.storage_sum_j);
        close(
            s.heat_minus_storage_residual_j,
            -s.accumulated_interval_residual_change_j,
        );
        assert!((s.terms[8].change_j - h.budget.conduction_w * 2400.).abs() > 1.);
        let late = span(&rows[2..]).unwrap();
        assert_eq!(late.executed_seconds, 1200.);
        close(
            late.terms[0].change_j,
            h.heat.as_ref().unwrap().terms_j[0] - rows[2].habitat.heat.as_ref().unwrap().terms_j[0],
        );
    }
    #[test]
    fn excluded_intervals_zero_steps_and_refusals_keep_honest_coverage() {
        let mut h = habitat();
        let start = row(&h);
        let unchanged = h.heat.clone();
        h.step(1000., env(200.)).unwrap();
        assert_eq!(h.heat, unchanged);
        assert!(h
            .step(
                1060.,
                Environment {
                    irradiance_w_m2: Some(-1.),
                    exterior_temperature_k: Some(200.)
                }
            )
            .is_err());
        assert_eq!(h.heat, unchanged);
        h.step(1060., Environment::default()).unwrap();
        h.state.temperature_k = 600.;
        h.step(1120., env(200.)).unwrap();
        h.state.temperature_k = 293.15;
        h.step(1150., env(200.)).unwrap();
        let s = span(&[start, row(&h)]).unwrap();
        assert_eq!(s.completed_span_s, 150.);
        assert_eq!(s.executed_seconds, 30.);
        assert_eq!(s.environment_missing_seconds, 60.);
        assert_eq!(s.start_temperature_unsupported_seconds, 60.);
        assert!(!s.complete_model_coverage);
        h.heat.as_ref().unwrap().validate(h.last_time).unwrap();
        let mut bad = h.heat.clone().unwrap();
        bad.executed_seconds += 1.;
        assert!(bad.validate(h.last_time).is_err());
        let mut bad = h.heat.clone().unwrap();
        bad.terms_j[3] = 1.;
        assert!(bad.validate(h.last_time).is_err());
        let mut bad = h.heat.clone().unwrap();
        bad.terms_j[0] = f64::NAN;
        assert!(bad.validate(h.last_time).is_err());
        let mut broken = row(&h);
        broken.habitat.heat.as_mut().unwrap().origin_time_s += 1.;
        assert!(span(&[row(&h), broken]).is_err());
    }
    #[test]
    fn optional_heat_paths_and_breach_enthalpy_remain_distinct() {
        let mut h = habitat();
        h.config.moisture = Some(crate::moisture::Config {
            relative_humidity_setpoint: 0.,
            ..Default::default()
        });
        h.config.electrolysis = Some(Default::default());
        h.config.sabatier = Some(Default::default());
        h.config.breach = Some(Default::default());
        let start = row(&h);
        let mut expected = [0.; 11];
        let mut saw = [false; 5];
        for i in 1..=120 {
            h.step(1000. + i as f64 * 60., env(210.)).unwrap();
            let b = &h.budget;
            let e = b.electrolysis.as_ref().unwrap();
            let s = b.sabatier.as_ref().unwrap();
            let m = b.moisture.as_ref().unwrap();
            let breach = b.breach.as_ref().unwrap();
            let energies = [
                b.delivered_w * 60.,
                b.battery_losses_w * 60.,
                b.metabolic_heat_w * 60.,
                -e.reaction_enthalpy_j,
                s.reaction_heat_j,
                -m.crew_phase_energy_j,
                m.condensation_heat_j,
                -b.radiator_rejection_w * 60.,
                b.conduction_w * 60.,
                -breach.lumped_cooling_j,
                b.thermal_storage_w * 60.,
            ];
            for (v, x) in expected.iter_mut().zip(energies) {
                *v += x;
            }
            saw[0] |= e.reaction_enthalpy_j > 0.;
            saw[1] |= s.reaction_heat_j > 0.;
            saw[2] |= m.crew_phase_energy_j > 0.;
            saw[3] |= m.condensation_heat_j > 0.;
            saw[4] |= breach.lumped_cooling_j > 0.;
        }
        assert!(saw.iter().all(|v| *v));
        assert_eq!(h.heat.as_ref().unwrap().terms_j, expected);
        let s = span(&[start, row(&h)]).unwrap();
        assert!(saw[..4].iter().all(|v| *v));
        close(s.heat_path_sum_j, s.storage_sum_j);
        assert!(h.totals.breach_enthalpy_j > -s.terms[9].change_j);
        for i in 0..9 {
            assert!(s.terms[i].change_j.is_finite());
        }
    }
    fn world() -> (crate::Universe, BodyRef) {
        let u = crate::Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let body = u.systems[0].find_body("Mars").unwrap() as u32;
        (u, BodyRef { system: 0, body })
    }
    #[test]
    fn legacy_save_and_source_pins_preserve_exact_replay() {
        let (mut new, r) = world();
        let mut old = new.clone();
        let d = Definition::at(r, 0., 0.);
        let mut legacy = serde_json::to_value(&d).unwrap();
        legacy.as_object_mut().unwrap().remove("record_heat");
        let legacy: Definition = serde_json::from_value(legacy).unwrap();
        assert!(!legacy.record_heat);
        new.apply_edit(crate::sandbox::Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        old.apply_edit(crate::sandbox::Edit::CreateHabitat {
            definition: Box::new(legacy),
        })
        .unwrap();
        new.advance_by(3600.);
        old.advance_by(3600.);
        assert_eq!(new.habitats[0].state, old.habitats[0].state);
        assert_eq!(new.habitats[0].totals, old.habitats[0].totals);
        assert!(old.habitats[0].heat.is_none());
        assert!(old.habitats[0].report().get("heat_accounting").is_none());
        let mut resumed =
            crate::save::from_json(&crate::save::to_json(&new, "heat").unwrap()).unwrap();
        new.advance_by(1800.);
        resumed.advance_by(1800.);
        assert_eq!(new.habitats, resumed.habitats);
        let old_json = crate::save::to_json(&old, "legacy").unwrap();
        assert!(!old_json.contains("record_heat") && !old_json.contains(MODEL));
        let mut old_resumed = crate::save::from_json(&old_json).unwrap();
        old.advance_by(1800.);
        old_resumed.advance_by(1800.);
        assert_eq!(old.habitats, old_resumed.habitats);
        let mut bad: Value =
            serde_json::from_str(&crate::save::to_json(&new, "bad").unwrap()).unwrap();
        bad["header"]["data_versions"]
            .as_object_mut()
            .unwrap()
            .remove("cosmogon-cabin-heat-accounting");
        assert!(crate::save::from_json(&bad.to_string()).is_err());
        let mut bad = new.habitats[0].clone();
        bad.heat = None;
        assert!(bad.validate(new.time).is_err());
    }
    #[test]
    fn capacity_changes_record_actual_edit_remainders_and_roundtrip() {
        let (mut u, r) = world();
        u.apply_edit(crate::sandbox::Edit::CreateHabitat {
            definition: Box::new(Definition::at(r, 0., 0.)),
        })
        .unwrap();
        let start = crate::habitat::sample(&u, 0).unwrap();
        let epoch = u.time;
        u.advance_to(epoch + 15., None, None);
        let old_t = u.habitats[0].state.temperature_k;
        let mut c = u.habitats[0].config.clone();
        c.heat_capacity_j_k *= 2.;
        u.apply_edit(crate::sandbox::Edit::ConfigureHabitat {
            system: r.system,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        assert!(u.habitats[0].last_time > epoch);
        let middle = crate::habitat::sample(&u, 0).unwrap();
        assert_eq!(middle.habitat.heat.as_ref().unwrap().executed_seconds, 15.);
        assert_eq!(
            middle.habitat.heat.as_ref().unwrap().capacity_transitions,
            0
        );
        let first_storage = middle.habitat.heat.as_ref().unwrap().terms_j[10];
        close(
            first_storage,
            start.habitat.config.heat_capacity_j_k * (middle.habitat.state.temperature_k - old_t),
        );
        u.advance_to(epoch + 120., None, None);
        let end = crate::habitat::sample(&u, 0).unwrap();
        let s = span(&[start, middle.clone(), end.clone()]).unwrap();
        assert_eq!(s.capacity_transitions, 1);
        assert_eq!(s.executed_seconds, 120.);
        close(s.heat_path_sum_j, s.storage_sum_j);
        close(
            s.storage_sum_j,
            first_storage
                + end.habitat.config.heat_capacity_j_k
                    * (end.habitat.state.temperature_k - middle.habitat.state.temperature_k),
        );
    }
    #[test]
    fn strict_recorded_source_and_full_precision_heat_export() {
        let (mut u, r) = world();
        u.apply_edit(crate::sandbox::Edit::CreateHabitat {
            definition: Box::new(Definition::at(r, 0., 0.)),
        })
        .unwrap();
        let initial = u.clone();
        let first = crate::habitat::sample(&u, 0).unwrap();
        u.advance_by(120.);
        let rows = vec![first, crate::habitat::sample(&u, 0).unwrap()];
        let folder =
            std::env::temp_dir().join(format!("cosmogon-heat-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir(&folder).unwrap();
        crate::habitat::export(&initial, &u, 0, &rows, &folder.join("study")).unwrap();
        let rec = crate::habitat_recording::Recording::read(&folder.join("study")).unwrap();
        let bundle = crate::habitat_balance::Bundle::ordinary(&rec, 0, 1).unwrap();
        assert!(bundle.reports[0].heat.is_some());
        let manifest = bundle.export(&folder.join("balance")).unwrap();
        assert!(manifest["files"]
            .as_array()
            .unwrap()
            .contains(&json!("heat.csv")));
        assert!(bundle.export(&folder.join("balance")).is_err());
        let csv = std::fs::read_to_string(folder.join("balance/heat.csv")).unwrap();
        assert_eq!(csv.lines().count(), 12);
        for (line, t) in csv
            .lines()
            .skip(1)
            .zip(&bundle.reports[0].heat.as_ref().unwrap().terms)
        {
            let cells = line.split(',').collect::<Vec<_>>();
            assert_eq!(cells[1], t.id);
            assert_eq!(cells[5].parse::<f64>().unwrap(), t.change_j);
        }
        let p = folder.join("study/manifest.json");
        let mut m: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        m["heat_accounting_source"]["model"] = json!("fake/9");
        std::fs::write(p, m.to_string()).unwrap();
        assert!(
            crate::habitat_recording::Recording::read(&folder.join("study"))
                .unwrap_err()
                .contains("heat accounting source")
        );
        std::fs::remove_dir_all(folder).unwrap();
    }
}
