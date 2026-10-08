//! Optional finite water electrolysis. Ratings are assumptions; chemistry uses NIST
//! reference stoichiometry and liquid-water reaction enthalpy at 298.15 K.
use crate::time::SECONDS_PER_DAY;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-water-electrolysis/1";
pub const ENTHALPY_SOURCE: &str = "https://webbook.nist.gov/cgi/cbook.cgi?Name=water&cTC=on";
pub const HYDROGEN_PER_OXYGEN: f64 = 2.0 * 0.00201588 / 0.0319988;
pub const REFERENCE_ENTHALPY_J_KG_OXYGEN: f64 = 571_660.0 / 0.0319988;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Constraint {
    ZeroRate,
    WaterAfterCrewReserve,
    OxygenStorage,
    HydrogenStorage,
    EquipmentPower,
    BusPower,
}
impl Constraint {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ZeroRate => "configured production rate is zero",
            Self::WaterAfterCrewReserve => "water remaining after nominal crew reserve",
            Self::OxygenStorage => "oxygen product storage headroom",
            Self::HydrogenStorage => "hydrogen product storage headroom",
            Self::EquipmentPower => "equipment electric power limit",
            Self::BusPower => "bus power after higher-priority equipment",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    pub power_limit_w: f64,
    pub oxygen_rate_kg_s: f64,
    pub electricity_j_kg_oxygen: f64,
    pub oxygen_capacity_kg: f64,
    pub hydrogen_capacity_kg: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            power_limit_w: 2000.0,
            oxygen_rate_kg_s: 6.0 / SECONDS_PER_DAY,
            electricity_j_kg_oxygen: 25.0e6,
            oxygen_capacity_kg: 200.0,
            hydrogen_capacity_kg: 2.0,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err(format!("Unsupported electrolysis model {}", self.model));
        }
        for (v, min, max, name) in [
            (
                self.power_limit_w,
                0.0,
                1.0e6,
                "Electrolysis power limit (W)",
            ),
            (
                self.oxygen_rate_kg_s,
                0.0,
                100.0,
                "Electrolysis oxygen rate (kg/s)",
            ),
            (
                self.electricity_j_kg_oxygen,
                REFERENCE_ENTHALPY_J_KG_OXYGEN,
                1.0e9,
                "Electricity per kg O₂ (J/kg; at least reference reaction enthalpy)",
            ),
            (
                self.oxygen_capacity_kg,
                0.0,
                1.0e7,
                "Oxygen product capacity (kg)",
            ),
            (
                self.hydrogen_capacity_kg,
                0.0,
                1.0e7,
                "Isolated hydrogen capacity (kg)",
            ),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!("{name} must be finite and between {min} and {max}"));
            }
        }
        Ok(())
    }
    /// Conservative start-of-interval request. Nominal crew water is reserved first;
    /// newly recovered water and product headroom wait until the next interval.
    pub fn planned_oxygen_kg(
        &self,
        water_kg: f64,
        oxygen_kg: f64,
        hydrogen_kg: f64,
        seconds: f64,
    ) -> f64 {
        (self.oxygen_rate_kg_s * seconds)
            .min(water_kg.max(0.0) / (1.0 + HYDROGEN_PER_OXYGEN))
            .min((self.oxygen_capacity_kg - oxygen_kg).max(0.0))
            .min((self.hydrogen_capacity_kg - hydrogen_kg).max(0.0) / HYDROGEN_PER_OXYGEN)
    }
    /// Record every bound below the rated interval mass, not a unique causal attribution.
    pub fn constraints(
        &self,
        water_kg: f64,
        oxygen_kg: f64,
        hydrogen_kg: f64,
        seconds: f64,
        requested_w: f64,
        delivered_w: f64,
    ) -> Vec<Constraint> {
        let rated = self.oxygen_rate_kg_s * seconds;
        let planned = self.planned_oxygen_kg(water_kg, oxygen_kg, hydrogen_kg, seconds);
        let mut limits = Vec::new();
        for (limited, reason) in [
            (rated == 0.0, Constraint::ZeroRate),
            (
                water_kg.max(0.0) / (1.0 + HYDROGEN_PER_OXYGEN) < rated,
                Constraint::WaterAfterCrewReserve,
            ),
            (
                (self.oxygen_capacity_kg - oxygen_kg).max(0.0) < rated,
                Constraint::OxygenStorage,
            ),
            (
                (self.hydrogen_capacity_kg - hydrogen_kg).max(0.0) / HYDROGEN_PER_OXYGEN < rated,
                Constraint::HydrogenStorage,
            ),
            (
                planned * self.electricity_j_kg_oxygen / seconds > self.power_limit_w,
                Constraint::EquipmentPower,
            ),
            (delivered_w < requested_w, Constraint::BusPower),
        ] {
            if limited {
                limits.push(reason);
            }
        }
        limits
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub requested_w: f64,
    pub delivered_w: f64,
    pub oxygen_generated_kg: f64,
    pub hydrogen_generated_kg: f64,
    pub water_consumed_kg: f64,
    pub electricity_j: f64,
    pub reaction_enthalpy_j: f64,
    pub waste_heat_j: f64,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}
impl Budget {
    pub fn from_power(
        c: &Config,
        planned_kg: f64,
        requested_w: f64,
        delivered_w: f64,
        seconds: f64,
    ) -> Self {
        let electricity_j = delivered_w * seconds;
        let oxygen_generated_kg = planned_kg.min(electricity_j / c.electricity_j_kg_oxygen);
        let hydrogen_generated_kg = oxygen_generated_kg * HYDROGEN_PER_OXYGEN;
        let reaction_enthalpy_j = oxygen_generated_kg * REFERENCE_ENTHALPY_J_KG_OXYGEN;
        Self {
            requested_w,
            delivered_w,
            oxygen_generated_kg,
            hydrogen_generated_kg,
            water_consumed_kg: oxygen_generated_kg + hydrogen_generated_kg,
            electricity_j,
            reaction_enthalpy_j,
            waste_heat_j: (electricity_j - reaction_enthalpy_j).max(0.0),
            constraints: Vec::new(),
        }
    }
    pub fn validate(&self, seconds: f64) -> Result<(), String> {
        for v in [
            self.requested_w,
            self.delivered_w,
            self.oxygen_generated_kg,
            self.hydrogen_generated_kg,
            self.water_consumed_kg,
            self.electricity_j,
            self.reaction_enthalpy_j,
            self.waste_heat_j,
        ] {
            if !v.is_finite() || !(0.0..=1.0e30).contains(&v) {
                return Err("Invalid electrolysis interval quantity".into());
            }
        }
        if self.delivered_w > self.requested_w + 1e-6 {
            return Err("Electrolysis supply exceeds demand".into());
        }
        for (actual, expected) in [
            (
                self.hydrogen_generated_kg,
                self.oxygen_generated_kg * HYDROGEN_PER_OXYGEN,
            ),
            (
                self.water_consumed_kg,
                self.oxygen_generated_kg + self.hydrogen_generated_kg,
            ),
            (
                self.reaction_enthalpy_j,
                self.oxygen_generated_kg * REFERENCE_ENTHALPY_J_KG_OXYGEN,
            ),
            (self.electricity_j, self.delivered_w * seconds),
            (
                self.electricity_j,
                self.reaction_enthalpy_j + self.waste_heat_j,
            ),
        ] {
            if (actual - expected).abs() > 1e-9 * actual.abs().max(expected.abs()).max(1.0) {
                return Err(
                    "Electrolysis interval violates reference mass or energy balance".into(),
                );
            }
        }
        Ok(())
    }
}
pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"reaction":"2 H₂O(l) → 2 H₂(g) + O₂(g)","reference_temperature_k":298.15,"source_version":"NIST SRD69 CODATA review and molar masses, retrieved 2026-10-08","oxygen_molar_mass_kg_mol":0.0319988,"hydrogen_molar_mass_kg_mol":0.00201588,"derived_water_molar_mass_kg_mol":0.01801528,"water_formation_enthalpy_j_mol":-285830.0,"reported_water_formation_enthalpy_uncertainty_j_mol":40.0,"source_uncertainty_scope":"Reported source value; not propagated or assigned a confidence level here","reaction_enthalpy_j_per_mol_oxygen":571660.0,"reaction_enthalpy_j_per_kg_oxygen":REFERENCE_ENTHALPY_J_KG_OXYGEN,"hydrogen_kg_per_kg_oxygen":HYDROGEN_PER_OXYGEN,"source_urls":[ENTHALPY_SOURCE,"https://webbook.nist.gov/cgi/cbook.cgi?Name=H2&cTG=on","https://webbook.nist.gov/cgi/cbook.cgi?Name=O2","https://www.nasa.gov/reference/environmental-control-and-life-support-systems-eclss/"],"equipment":"User-assumed power, rate, electrical intensity and product capacities","limits":"Fixed reference enthalpy, ideal separated products; no purification, compression, tank structure, leaks/ignition, Sabatier, venting or fuel-cell recovery. Reaction enthalpy is not minimum reversible electrical work. Predictive uncertainty is unavailable."})
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::habitat::{self, Definition, Environment, Habitat};
    fn experiment() -> Habitat {
        let mut d = Definition::at(crate::BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        d.config.crew = 0;
        d.config.firm_power_w = 5000.0;
        d.config.solar_area_m2 = 0.0;
        d.config.base_load_w = 0.0;
        d.config.scrubber_power_w = 0.0;
        d.config.recycler_power_w = 0.0;
        d.config.heater_limit_w = 0.0;
        d.config.radiator_pump_w = 0.0;
        d.config.radiator_area_m2 = 0.0;
        d.config.conductance_w_k = 0.0;
        d.config.battery_capacity_j = 0.0;
        d.initial.battery_j = 0.0;
        d.config.electrolysis = Some(Config::default());
        Habitat::new(0, d, 0.0).unwrap()
    }
    fn env() -> Environment {
        Environment {
            irradiance_w_m2: Some(0.0),
            exterior_temperature_k: Some(293.15),
        }
    }
    #[test]
    fn reference_stoichiometry_mass_and_reaction_heat_match_independent_day_fixture() {
        let mut h = experiment();
        let start = h.state.clone();
        let mass = start.mass_kg();
        for n in 1..=1440 {
            h.step(n as f64 * habitat::PERIOD, env()).unwrap();
            assert!(h.budget.mass_residual_kg.abs() < 1e-10);
            assert!(h.budget.thermal_residual_j.abs() < 1e-5);
        }
        let oxygen = h.state.oxygen_reserve_kg - start.oxygen_reserve_kg;
        let expected_hydrogen = 6.0 / 31.9988 * 2.0 * 2.01588;
        let expected_chemical = 6.0 / 31.9988 * 571660000.0;
        let heat = 6.0 * 25.0e6 - expected_chemical;
        assert!((oxygen - 6.0).abs() < 1e-10);
        assert!((h.state.hydrogen_kg - expected_hydrogen).abs() < 1e-12);
        assert!(
            (start.potable_water_kg - h.state.potable_water_kg - 6.0 - expected_hydrogen).abs()
                < 1e-10
        );
        assert!((h.state.mass_kg() - mass).abs() < 1e-9);
        assert!((h.totals.reaction_enthalpy_j - expected_chemical).abs() < 1e-4);
        assert!(
            (h.state.temperature_k - start.temperature_k - heat / h.config.heat_capacity_j_k).abs()
                < 1e-10
        );
        let b = h.budget.electrolysis.as_ref().unwrap();
        assert!((b.electricity_j - b.reaction_enthalpy_j - b.waste_heat_j).abs() < 1e-8);
        let c = Config {
            electricity_j_kg_oxygen: REFERENCE_ENTHALPY_J_KG_OXYGEN,
            ..Default::default()
        };
        let b = Budget::from_power(&c, 1.0, 1000.0, 1000.0, 60.0);
        assert!(b.waste_heat_j.abs() < 1e-9);
    }
    #[test]
    fn finite_water_product_space_power_and_crew_priority_bound_production() {
        let mut cases = Vec::new();
        let mut h = experiment();
        h.config.firm_power_w = 0.0;
        cases.push(h);
        let mut h = experiment();
        h.state.potable_water_kg = 0.0;
        cases.push(h);
        let mut h = experiment();
        h.config.electrolysis.as_mut().unwrap().oxygen_capacity_kg = h.state.oxygen_reserve_kg;
        cases.push(h);
        let mut h = experiment();
        h.config.electrolysis.as_mut().unwrap().hydrogen_capacity_kg = 0.0;
        cases.push(h);
        let mut h = experiment();
        h.config.firm_power_w = 500.0;
        h.config.base_load_w = 500.0;
        cases.push(h);
        let mut h = experiment();
        h.config.crew = 4;
        h.state.potable_water_kg = 4.0 * 3.217 * 60.0 / SECONDS_PER_DAY;
        cases.push(h);
        for mut h in cases {
            h.step(60.0, env()).unwrap();
            let b = h.budget.electrolysis.as_ref().unwrap();
            assert_eq!(b.oxygen_generated_kg, 0.0);
            assert_eq!(b.reaction_enthalpy_j, 0.0);
            assert_eq!(h.state.hydrogen_kg, 0.0);
            assert!(h.state.potable_water_kg >= 0.0);
        }
        let mut h = experiment();
        h.config.electrolysis.as_mut().unwrap().hydrogen_capacity_kg = 0.0001;
        h.step(60.0, env()).unwrap();
        assert!((h.state.hydrogen_kg - 0.0001).abs() < 1e-18);
        h.step(120.0, env()).unwrap();
        assert_eq!(
            h.budget.electrolysis.as_ref().unwrap().oxygen_generated_kg,
            0.0
        );
        let held = h.state.hydrogen_kg;
        let transferred = h.totals.reaction_enthalpy_j;
        let mut c = h.config.clone();
        c.electrolysis.as_mut().unwrap().hydrogen_capacity_kg = 0.0;
        assert!(h.check_config(&c).is_err());
        h.config.electrolysis = None;
        h.step(180.0, env()).unwrap();
        assert_eq!(h.state.hydrogen_kg, held);
        assert_eq!(h.totals.reaction_enthalpy_j, transferred);
        assert!(h.budget.electrolysis.is_none());
    }
    #[test]
    fn generated_oxygen_uses_the_existing_regulator_on_the_next_interval() {
        let mut h = experiment();
        h.config.crew = 4;
        h.state.oxygen_kg = 0.0;
        h.state.oxygen_reserve_kg = 0.0;
        h.step(60.0, env()).unwrap();
        assert_eq!(h.budget.served_cycle_fraction, 0.0);
        assert_eq!(h.budget.oxygen_makeup_kg, 0.0);
        assert!(h.state.oxygen_reserve_kg > 0.0);
        h.step(120.0, env()).unwrap();
        assert_eq!(h.budget.served_cycle_fraction, 1.0);
        assert!(h.budget.oxygen_makeup_kg > 0.0);
        // Serving nominal inventory demand is not evidence of breathable cabin pressure.
        assert!(h.state.oxygen_pa(&h.config) < h.config.oxygen_alert_pa);
    }
    #[test]
    fn version_domain_and_initial_capacity_errors_are_explicit() {
        let mut c = Config {model:"future/2".into(),..Default::default()};
        assert!(c.validate().is_err());
        c = Config {electricity_j_kg_oxygen:REFERENCE_ENTHALPY_J_KG_OXYGEN - 1.0,..Default::default()};
        assert!(c.validate().is_err());
        c.electricity_j_kg_oxygen = f64::NAN;
        assert!(c.validate().is_err());
        let mut d = experiment().definition;
        d.config.electrolysis.as_mut().unwrap().oxygen_capacity_kg = 0.0;
        assert!(d.validate().is_err());
        let json = serde_json::to_value(Config::default()).unwrap();
        let mut bad = json.clone();
        bad["unknown"] = 1.into();
        assert!(serde_json::from_value::<Config>(bad).is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{
        habitat::{export, sample, Definition},
        sandbox::Edit,
        save, BodyRef, Scenario, Universe, UniverseSettings,
    };
    #[test]
    fn optional_reaction_replays_and_rejects_invalid_versions_or_discarded_products() {
        let mut a = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = BodyRef {
            system: 0,
            body: a.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut d = Definition::at(r, 0.0, 0.0);
        d.config.electrolysis = Some(Config::default());
        a.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        let initial = a.clone();
        let first = sample(&a, 0).unwrap();
        let epoch = a.time;
        let mut b = a.clone();
        a.advance_to(epoch + 7200.0, None, None);
        for n in 1..=240 {
            b.advance_to(epoch + n as f64 * 30.0, None, None);
        }
        assert_eq!(a, b);
        let mut resumed = save::from_json(&save::to_json(&a, "oxygen").unwrap()).unwrap();
        a.advance_by(1800.0);
        resumed.advance_by(1800.0);
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let header = save::header_for(&a, "oxygen");
        assert_eq!(header.data_versions["cosmogon-water-electrolysis"], MODEL);
        let original: serde_json::Value =
            serde_json::from_str(&save::to_json(&a, "oxygen").unwrap()).unwrap();
        for bad_version in [None, Some("future/2")] {
            let mut v = original.clone();
            if let Some(version) = bad_version {
                v["header"]["data_versions"]["cosmogon-water-electrolysis"] = version.into();
            } else {
                v["header"]["data_versions"]
                    .as_object_mut()
                    .unwrap()
                    .remove("cosmogon-water-electrolysis");
            }
            assert!(matches!(
                save::from_json(&v.to_string()),
                Err(save::SaveError::Model(_))
            ));
        }
        let mut corrupt = original.clone();
        corrupt["universe"]["habitats"][0]["budget"]["electrolysis"]["water_consumed_kg"] =
            0.into();
        assert!(save::from_json(&corrupt.to_string()).is_err());
        let held = a.habitats[0].state.hydrogen_kg;
        assert!(held > 0.0);
        let mut config = a.habitats[0].config.clone();
        config.electrolysis.as_mut().unwrap().hydrogen_capacity_kg = 0.0;
        let old = a.clone();
        assert!(a
            .apply_edit(Edit::ConfigureHabitat {
                system: 0,
                id: 0,
                config: Box::new(config)
            })
            .is_err());
        assert_eq!(a, old);
        a.advance_by(17.0);
        let old_transfer = a.habitats[0].totals.reaction_enthalpy_j;
        let mut config = a.habitats[0].config.clone();
        config.electrolysis = None;
        a.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(config),
        })
        .unwrap();
        assert!(a.habitats[0].totals.reaction_enthalpy_j > old_transfer);
        let held = a.habitats[0].state.hydrogen_kg;
        a.advance_by(120.0);
        assert_eq!(a.habitats[0].state.hydrogen_kg, held);
        assert!(save::from_json(&save::to_json(&a, "disabled").unwrap()).is_ok());
        assert!(save::header_for(&a, "disabled")
            .data_versions
            .contains_key("cosmogon-water-electrolysis"));
        let root =
            std::env::temp_dir().join(format!("cosmogon-oxygen-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        export(&initial, &a, 0, &[first, sample(&a, 0).unwrap()], &root).unwrap();
        let csv = std::fs::read_to_string(root.join("samples.csv")).unwrap();
        let mut rows = csv.lines();
        let headers = rows.next().unwrap().split(',').collect::<Vec<_>>();
        let initial_row = rows.next().unwrap().split(',').collect::<Vec<_>>();
        let disabled_row = rows.next().unwrap().split(',').collect::<Vec<_>>();
        assert_eq!(headers.len(), initial_row.len());
        assert_eq!(headers.len(), disabled_row.len());
        let index = headers
            .iter()
            .position(|v| *v == "electrolysis_oxygen_generated_kg")
            .unwrap();
        assert_eq!(initial_row[index], "");
        assert_eq!(disabled_row[index], "");
        let h_index = headers.iter().position(|v| *v == "hydrogen_kg").unwrap();
        assert_eq!(disabled_row[h_index].parse::<f64>().unwrap(), held);
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["electrolysis_source"]["model"], MODEL);
        assert_eq!(
            manifest["electrolysis_model_descriptor"]["id"],
            "electrolysis"
        );
        assert!(export(&initial, &a, 0, &[], &root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
