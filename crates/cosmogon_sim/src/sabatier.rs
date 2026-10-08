//! Optional finite CO₂ reduction with isolated methane and raw liquid product.
//! Reference enthalpy includes condensation at 298.15 K; ratings are assumptions.
use crate::time::SECONDS_PER_DAY;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-sabatier/1";
pub const CO2_SOURCE: &str = "https://webbook.nist.gov/cgi/cbook.cgi?ID=C124389&Mask=1";
pub const METHANE_SOURCE: &str = "https://webbook.nist.gov/cgi/cbook.cgi?ID=C74828&Mask=1";
pub const HYDROGEN_PER_CO2: f64 = 4.0 * 0.00201588 / 0.0440095;
pub const WATER_PER_CO2: f64 = 2.0 * 0.01801528 / 0.0440095;
// Derive CH4 from the chosen reactant/product masses, avoiding rounded-table drift.
pub const METHANE_PER_CO2: f64 = (0.0440095 + 4.0 * 0.00201588 - 2.0 * 0.01801528) / 0.0440095;
pub const REFERENCE_HEAT_J_KG_CO2: f64 = 252_750.0 / 0.0440095;

#[derive(Clone, Copy, Debug)]
pub struct Stocks {
    pub co2_kg: f64,
    pub hydrogen_kg: f64,
    pub methane_kg: f64,
    pub water_kg: f64,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Constraint {
    ZeroRate,
    CapturedCo2,
    Hydrogen,
    MethaneStorage,
    RawWaterStorage,
    EquipmentPower,
    BusPower,
}
impl Constraint {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ZeroRate => "configured processing rate is zero",
            Self::CapturedCo2 => "start-of-interval captured carbon dioxide",
            Self::Hydrogen => "start-of-interval isolated hydrogen",
            Self::MethaneStorage => "methane product storage headroom",
            Self::RawWaterStorage => "raw liquid product storage headroom",
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
    pub co2_rate_kg_s: f64,
    pub electricity_j_kg_co2: f64,
    pub methane_capacity_kg: f64,
    pub water_capacity_kg: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            power_limit_w: 1000.0,
            co2_rate_kg_s: 4.0 / SECONDS_PER_DAY,
            electricity_j_kg_co2: 2.0e6,
            methane_capacity_kg: 2.0,
            water_capacity_kg: 10.0,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err(format!("Unsupported Sabatier model {}", self.model));
        }
        for (v, min, max, name) in [
            (
                self.power_limit_w,
                0.0,
                1.0e6,
                "Sabatier electric power limit (W)",
            ),
            (
                self.co2_rate_kg_s,
                0.0,
                100.0,
                "Sabatier CO₂ processing rate (kg/s)",
            ),
            (
                self.electricity_j_kg_co2,
                1.0,
                1.0e9,
                "Sabatier process electricity (J/kg CO₂)",
            ),
            (
                self.methane_capacity_kg,
                0.0,
                1.0e7,
                "Methane product capacity (kg)",
            ),
            (
                self.water_capacity_kg,
                0.0,
                1.0e7,
                "Raw liquid product capacity (kg)",
            ),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!("{name} must be finite and between {min} and {max}"));
            }
        }
        Ok(())
    }
    pub fn planned_co2_kg(&self, s: Stocks, seconds: f64) -> f64 {
        (self.co2_rate_kg_s * seconds)
            .min(s.co2_kg.max(0.0))
            .min(s.hydrogen_kg.max(0.0) / HYDROGEN_PER_CO2)
            .min((self.methane_capacity_kg - s.methane_kg).max(0.0) / METHANE_PER_CO2)
            .min((self.water_capacity_kg - s.water_kg).max(0.0) / WATER_PER_CO2)
    }
    /// Every bound below the configured interval mass; not unique causal attribution.
    pub fn constraints(
        &self,
        s: Stocks,
        seconds: f64,
        requested_w: f64,
        delivered_w: f64,
    ) -> Vec<Constraint> {
        let rated = self.co2_rate_kg_s * seconds;
        let planned = self.planned_co2_kg(s, seconds);
        let mut limits = Vec::new();
        for (limited, reason) in [
            (rated == 0.0, Constraint::ZeroRate),
            (s.co2_kg < rated, Constraint::CapturedCo2),
            (
                s.hydrogen_kg / HYDROGEN_PER_CO2 < rated,
                Constraint::Hydrogen,
            ),
            (
                (self.methane_capacity_kg - s.methane_kg).max(0.0) / METHANE_PER_CO2 < rated,
                Constraint::MethaneStorage,
            ),
            (
                (self.water_capacity_kg - s.water_kg).max(0.0) / WATER_PER_CO2 < rated,
                Constraint::RawWaterStorage,
            ),
            (
                planned * self.electricity_j_kg_co2 / seconds > self.power_limit_w,
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
    pub co2_consumed_kg: f64,
    pub hydrogen_consumed_kg: f64,
    pub methane_generated_kg: f64,
    pub water_generated_kg: f64,
    pub electricity_j: f64,
    pub reaction_heat_j: f64,
    pub total_heat_j: f64,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}
impl Budget {
    pub fn from_power(
        c: &Config,
        planned: f64,
        requested_w: f64,
        delivered_w: f64,
        seconds: f64,
    ) -> Self {
        let electricity_j = delivered_w * seconds;
        let co2_consumed_kg = planned.min(electricity_j / c.electricity_j_kg_co2);
        let reaction_heat_j = co2_consumed_kg * REFERENCE_HEAT_J_KG_CO2;
        Self {
            requested_w,
            delivered_w,
            co2_consumed_kg,
            hydrogen_consumed_kg: co2_consumed_kg * HYDROGEN_PER_CO2,
            methane_generated_kg: co2_consumed_kg * METHANE_PER_CO2,
            water_generated_kg: co2_consumed_kg * WATER_PER_CO2,
            electricity_j,
            reaction_heat_j,
            total_heat_j: electricity_j + reaction_heat_j,
            constraints: Vec::new(),
        }
    }
    pub fn validate(&self, seconds: f64) -> Result<(), String> {
        for v in [
            self.requested_w,
            self.delivered_w,
            self.co2_consumed_kg,
            self.hydrogen_consumed_kg,
            self.methane_generated_kg,
            self.water_generated_kg,
            self.electricity_j,
            self.reaction_heat_j,
            self.total_heat_j,
        ] {
            if !v.is_finite() || !(0.0..=1.0e30).contains(&v) {
                return Err("Invalid Sabatier interval quantity".into());
            }
        }
        if self.delivered_w > self.requested_w + 1e-6 {
            return Err("Sabatier supply exceeds demand".into());
        }
        for (a, e) in [
            (
                self.hydrogen_consumed_kg,
                self.co2_consumed_kg * HYDROGEN_PER_CO2,
            ),
            (
                self.methane_generated_kg,
                self.co2_consumed_kg * METHANE_PER_CO2,
            ),
            (
                self.water_generated_kg,
                self.co2_consumed_kg * WATER_PER_CO2,
            ),
            (
                self.co2_consumed_kg + self.hydrogen_consumed_kg,
                self.methane_generated_kg + self.water_generated_kg,
            ),
            (self.electricity_j, self.delivered_w * seconds),
            (
                self.reaction_heat_j,
                self.co2_consumed_kg * REFERENCE_HEAT_J_KG_CO2,
            ),
            (self.total_heat_j, self.electricity_j + self.reaction_heat_j),
        ] {
            if (a - e).abs() > 1e-9 * a.abs().max(e.abs()).max(1.0) {
                return Err("Sabatier interval violates reference mass or energy balance".into());
            }
        }
        Ok(())
    }
}
pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"reaction":"CO₂(g) + 4 H₂(g) → CH₄(g) + 2 H₂O(l)","reference_temperature_k":298.15,"source_version":"NIST SRD69 CODATA CO2/water and Manion 2002 adopted methane recommendation, retrieved 2026-10-08","co2_molar_mass_kg_mol":0.0440095,"hydrogen_molar_mass_kg_mol":0.00201588,"derived_water_molar_mass_kg_mol":0.01801528,"derived_methane_molar_mass_kg_mol":0.01604246,"co2_formation_enthalpy_j_mol":-393510.0,"reported_co2_formation_enthalpy_uncertainty_j_mol":130.0,"water_formation_enthalpy_j_mol":-285830.0,"reported_water_formation_enthalpy_uncertainty_j_mol":40.0,"methane_formation_enthalpy_j_mol":-74600.0,"reported_methane_formation_enthalpy_uncertainty_j_mol":300.0,"methane_source_choice":"Manion 2002 adopted recommendation; distinct from Chase/JANAF -74870 J/mol","reaction_enthalpy_j_mol_co2":-252750.0,"reaction_heat_j_kg_co2":REFERENCE_HEAT_J_KG_CO2,"hydrogen_kg_per_kg_co2":HYDROGEN_PER_CO2,"water_kg_per_kg_co2":WATER_PER_CO2,"methane_kg_per_kg_co2":METHANE_PER_CO2,"source_uncertainty_scope":"Reported source values, not propagated predictive confidence","source_urls":[CO2_SOURCE,METHANE_SOURCE,crate::electrolysis::ENTHALPY_SOURCE,"https://webbook.nist.gov/cgi/cbook.cgi?Name=H2&cTG=on","https://www.nasa.gov/reference/environmental-control-and-life-support-systems-eclss/"],"equipment":"User-assumed rate, process electricity, power and separated product capacities; raw liquid later uses shared powered recycler recovery assumption","limits":"Overall reference heat includes liquid condensation at 298.15 K, not reactor temperature/kinetics or heat rejection hardware. No catalyst startup, purity, compression, tank pressure, venting, combustion or calibration. Predictive uncertainty unavailable."})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{self, Definition, Environment, Habitat},
        BodyRef,
    };
    fn experiment() -> Habitat {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        let c = &mut d.config;
        c.crew = 0;
        c.firm_power_w = 5000.0;
        c.solar_area_m2 = 0.0;
        c.base_load_w = 0.0;
        c.scrubber_power_w = 0.0;
        c.recycler_power_w = 0.0;
        c.heater_limit_w = 0.0;
        c.radiator_pump_w = 0.0;
        c.radiator_area_m2 = 0.0;
        c.conductance_w_k = 0.0;
        c.battery_capacity_j = 0.0;
        c.sabatier = Some(Config::default());
        d.initial.battery_j = 0.0;
        let mut h = Habitat::new(0, d, 0.0).unwrap();
        h.state.captured_co2_kg = 5.0;
        h.state.hydrogen_kg = 1.0;
        h.totals.reaction_enthalpy_j = h.state.hydrogen_kg
            / crate::electrolysis::HYDROGEN_PER_OXYGEN
            * crate::electrolysis::REFERENCE_ENTHALPY_J_KG_OXYGEN;
        h
    }
    fn env() -> Environment {
        Environment {
            irradiance_w_m2: Some(0.0),
            exterior_temperature_k: Some(293.15),
        }
    }
    #[test]
    fn independent_day_fixture_conserves_mass_and_adds_reference_heat() {
        let mut h = experiment();
        let before = h.state.clone();
        for n in 1..=1440 {
            h.step(n as f64 * habitat::PERIOD, env()).unwrap();
            assert!(h.budget.mass_residual_kg.abs() < 1e-10);
            assert!(h.budget.thermal_residual_j.abs() < 1e-5);
        }
        let hydrogen = 4.0 / 44.0095 * 4.0 * 2.01588;
        let water = 4.0 / 44.0095 * 2.0 * 18.01528;
        let methane = 4.0 + hydrogen - water;
        let heat = 4.0 / 44.0095 * (74600.0 + 2.0 * 285830.0 - 393510.0) * 1000.0;
        assert!((before.captured_co2_kg - h.state.captured_co2_kg - 4.0).abs() < 1e-10);
        assert!((before.hydrogen_kg - h.state.hydrogen_kg - hydrogen).abs() < 1e-12);
        assert!((h.state.methane_kg - methane).abs() < 1e-12);
        assert!((h.state.sabatier_water_kg - water).abs() < 1e-11);
        assert!((h.totals.sabatier_heat_j - heat).abs() < 1e-5);
        assert!(
            (h.state.temperature_k
                - before.temperature_k
                - (8e6 + heat) / h.config.heat_capacity_j_k)
                .abs()
                < 1e-10
        );
        assert!((h.state.mass_kg() - before.mass_kg()).abs() < 1e-9);
        h.validate(h.last_time).unwrap();
        h.budget.sabatier.as_ref().unwrap().validate(60.0).unwrap();
    }
    #[test]
    fn finite_inputs_products_and_power_bound_reduction_without_disposal() {
        for which in 0..7 {
            let mut h = experiment();
            match which {
                0 => h.state.captured_co2_kg = 0.0,
                1 => h.state.hydrogen_kg = 0.0,
                2 => h.config.sabatier.as_mut().unwrap().methane_capacity_kg = 0.0,
                3 => h.config.sabatier.as_mut().unwrap().water_capacity_kg = 0.0,
                4 => {
                    h.config.firm_power_w = 0.0;
                }
                5 => h.config.sabatier.as_mut().unwrap().power_limit_w = 0.0,
                _ => {
                    h.config.base_load_w = h.config.firm_power_w;
                }
            }
            let before = h.state.clone();
            h.step(60.0, env()).unwrap();
            assert_eq!(h.budget.sabatier.as_ref().unwrap().co2_consumed_kg, 0.0);
            assert_eq!(h.state.hydrogen_kg, before.hydrogen_kg);
            assert_eq!(h.state.methane_kg, 0.0);
            assert!(!h.budget.sabatier.as_ref().unwrap().constraints.is_empty());
        }
        let mut h = experiment();
        h.config.sabatier.as_mut().unwrap().water_capacity_kg = 0.0001;
        h.step(60.0, env()).unwrap();
        assert!((h.state.sabatier_water_kg - 0.0001).abs() < 1e-15);
        h.step(120.0, env()).unwrap();
        assert_eq!(h.budget.sabatier.as_ref().unwrap().co2_consumed_kg, 0.0);
        let mut c = h.config.clone();
        c.sabatier.as_mut().unwrap().water_capacity_kg = 0.0;
        assert!(h.check_config(&c).is_err());
        c.sabatier.as_mut().unwrap().methane_capacity_kg = 0.0;
        assert!(h.check_config(&c).is_err());
    }
    #[test]
    fn raw_product_waits_for_later_powered_processing_and_keeps_history_when_disabled() {
        let mut h = experiment();
        h.config.recycler_power_w = 100.0;
        h.config.recycler_rate_kg_s = 1.0;
        let start = h.state.potable_water_kg;
        h.step(60.0, env()).unwrap();
        let raw = h.state.sabatier_water_kg;
        let methane = h.state.methane_kg;
        assert!(raw > 0.0);
        assert_eq!(h.budget.sabatier_water_processed_kg, 0.0);
        assert_eq!(h.state.potable_water_kg, start);
        h.config.sabatier = None;
        h.state.wastewater_kg = 59.999;
        h.step(120.0, env()).unwrap();
        assert!((h.budget.sabatier_water_processed_kg - 0.001).abs() < 1e-12);
        assert!(h.state.sabatier_water_kg > 0.0);
        h.step(180.0, env()).unwrap();
        assert_eq!(h.state.sabatier_water_kg, 0.0);
        assert_eq!(h.state.methane_kg, methane);
        assert!((h.totals.sabatier_water_processed_kg - raw).abs() < 1e-12);
        assert!(h.budget.sabatier.is_none());
        assert!(h.uses_sabatier());
        h.validate(180.0).unwrap();
    }
    #[test]
    fn unsupported_and_nonfinite_equipment_and_corrupt_reactions_are_rejected() {
        for which in 0..4 {
            let mut c = Config::default();
            match which {
                0 => c.model = "future/2".into(),
                1 => c.electricity_j_kg_co2 = 0.0,
                2 => c.co2_rate_kg_s = f64::NAN,
                _ => c.water_capacity_kg = -1.0,
            };
            assert!(c.validate().is_err());
        }
        let mut value = serde_json::to_value(Config::default()).unwrap();
        value["fake_yield"] = 1.into();
        assert!(serde_json::from_value::<Config>(value).is_err());
        let mut h = experiment();
        h.step(60.0, env()).unwrap();
        h.budget.sabatier.as_mut().unwrap().water_generated_kg = 0.0;
        assert!(h.validate(60.0).is_err());
        let mut h = experiment();
        h.step(60.0, env()).unwrap();
        h.state.methane_kg += 0.001;
        assert!(h.validate(60.0).is_err());
    }
    #[test]
    fn coupled_scheduler_save_edits_and_exports_preserve_source_and_actual_flows() {
        use crate::{sandbox::Edit, save, Scenario, Universe, UniverseSettings};
        let mut a = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = BodyRef {
            system: 0,
            body: a.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut d = Definition::at(r, 0.0, 0.0);
        d.config.electrolysis = Some(crate::electrolysis::Config::default());
        d.config.sabatier = Some(Config::default());
        a.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        let initial = a.clone();
        let first = habitat::sample(&a, 0).unwrap();
        let epoch = a.time;
        let mut b = a.clone();
        a.advance_to(epoch + 7200.0, None, None);
        for n in 1..=240 {
            b.advance_to(epoch + n as f64 * 30.0, None, None);
        }
        assert_eq!(a, b);
        assert!(a.habitats[0].state.methane_kg > 0.0);
        let mut resumed = save::from_json(&save::to_json(&a, "coupled").unwrap()).unwrap();
        a.advance_by(1800.0);
        resumed.advance_by(1800.0);
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let original: serde_json::Value =
            serde_json::from_str(&save::to_json(&a, "coupled").unwrap()).unwrap();
        for version in [None, Some("future/2")] {
            let mut v = original.clone();
            if let Some(s) = version {
                v["header"]["data_versions"]["cosmogon-sabatier"] = s.into();
            } else {
                v["header"]["data_versions"]
                    .as_object_mut()
                    .unwrap()
                    .remove("cosmogon-sabatier");
            }
            assert!(matches!(
                save::from_json(&v.to_string()),
                Err(save::SaveError::Model(_))
            ));
        }
        let mut config = a.habitats[0].config.clone();
        config.sabatier.as_mut().unwrap().methane_capacity_kg = 0.0;
        let before = a.clone();
        assert!(a
            .apply_edit(Edit::ConfigureHabitat {
                system: 0,
                id: 0,
                config: Box::new(config)
            })
            .is_err());
        assert_eq!(a, before);
        a.advance_by(17.0);
        let reacted = a.habitats[0].totals.sabatier_co2_consumed_kg;
        let mut config = a.habitats[0].config.clone();
        config.sabatier = None;
        a.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(config),
        })
        .unwrap();
        assert!(a.habitats[0].totals.sabatier_co2_consumed_kg > reacted);
        let methane = a.habitats[0].state.methane_kg;
        a.advance_by(120.0);
        assert_eq!(a.habitats[0].state.methane_kg, methane);
        assert!(save::from_json(&save::to_json(&a, "disabled").unwrap()).is_ok());
        assert_eq!(
            save::header_for(&a, "disabled").data_versions["cosmogon-sabatier"],
            MODEL
        );
        let root =
            std::env::temp_dir().join(format!("cosmogon-sabatier-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        habitat::export(
            &initial,
            &a,
            0,
            &[first, habitat::sample(&a, 0).unwrap()],
            &root,
        )
        .unwrap();
        let csv = std::fs::read_to_string(root.join("samples.csv")).unwrap();
        let mut lines = csv.lines();
        let headers = lines.next().unwrap().split(',').collect::<Vec<_>>();
        let i = headers
            .iter()
            .position(|v| *v == "sabatier_co2_consumed_kg")
            .unwrap();
        for row in lines {
            let values = row.split(',').collect::<Vec<_>>();
            assert_eq!(headers.len(), values.len());
            assert_eq!(values[i], "");
        }
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["sabatier_source"]["model"], MODEL);
        assert_eq!(manifest["sabatier_model_descriptor"]["id"], "sabatier");
        assert!(habitat::export(&initial, &a, 0, &[], &root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
