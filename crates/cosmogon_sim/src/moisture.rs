//! Optional finite cabin water phase budget. Equipment and vapor partition are assumptions.
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-cabin-moisture/1";
pub const SATURATION_SOURCE: &str = "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Mask=4";
pub const GAS_SOURCE: &str = "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Mask=1";
pub const WATER_MOLAR_KG: f64 = 0.0180153;
pub const REFERENCE_PHASE_J_KG: f64 = 44_004.0 / WATER_MOLAR_KG;
pub const REFERENCE_AIRBORNE_FRACTION: f64 = 6_118_000.0 / (2.946 * REFERENCE_PHASE_J_KG);
const R: f64 = 8.31446261815324;

pub fn saturation_pa(temperature_k: f64) -> Option<f64> {
    if !(273.15..=303.0).contains(&temperature_k) {
        return None;
    }
    Some(
        100_000.0
            * (std::f64::consts::LN_10 * (5.40221 - 1838.675 / (temperature_k - 31.737))).dexp(),
    )
}
pub fn vapor_pressure_pa(vapor_kg: f64, temperature_k: f64, volume_m3: f64) -> f64 {
    vapor_kg / WATER_MOLAR_KG * R * temperature_k / volume_m3
}
pub fn relative_humidity(vapor_kg: f64, temperature_k: f64, volume_m3: f64) -> Option<f64> {
    Some(vapor_pressure_pa(vapor_kg, temperature_k, volume_m3) / saturation_pa(temperature_k)?)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    pub airborne_fraction: f64,
    pub relative_humidity_setpoint: f64,
    pub power_limit_w: f64,
    pub rate_kg_s: f64,
    pub electricity_j_kg: f64,
    pub condensate_capacity_kg: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            airborne_fraction: REFERENCE_AIRBORNE_FRACTION,
            relative_humidity_setpoint: 0.5,
            power_limit_w: 500.0,
            rate_kg_s: 15.0 / 86_400.0,
            electricity_j_kg: 1_500_000.0,
            condensate_capacity_kg: 20.0,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err(format!("Unsupported cabin moisture model {}", self.model));
        }
        for (v, min, max, label) in [
            (
                self.airborne_fraction,
                0.0,
                1.0,
                "Airborne moisture fraction",
            ),
            (
                self.relative_humidity_setpoint,
                0.0,
                1.0,
                "Relative humidity setpoint",
            ),
            (self.power_limit_w, 0.0, 1e6, "Dehumidifier power limit (W)"),
            (self.rate_kg_s, 0.0, 100.0, "Dehumidifier rate (kg/s)"),
            (
                self.electricity_j_kg,
                1.0,
                1e9,
                "Dehumidifier electricity (J/kg)",
            ),
            (
                self.condensate_capacity_kg,
                0.0,
                1e7,
                "Condensate capacity (kg)",
            ),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!(
                    "{label} must be finite and between {min} and {max}"
                ));
            }
        }
        Ok(())
    }
    pub fn plan(
        &self,
        vapor_kg: f64,
        condensate_kg: f64,
        temperature_k: f64,
        volume_m3: f64,
        seconds: f64,
    ) -> Plan {
        let saturation = saturation_pa(temperature_k);
        let target = saturation.map(|p| {
            self.relative_humidity_setpoint * p * volume_m3 * WATER_MOLAR_KG / (R * temperature_k)
        });
        let excess = target.map(|m| (vapor_kg - m).max(0.0)).unwrap_or(0.0);
        let space = (self.condensate_capacity_kg - condensate_kg).max(0.0);
        let rated = self.rate_kg_s * seconds;
        let planned_kg = rated.min(excess).min(space);
        let requested_w = (planned_kg * self.electricity_j_kg / seconds).min(self.power_limit_w);
        let mut constraints = Vec::new();
        if saturation.is_none() {
            constraints.push("saturation fit unavailable outside 273.15–303 K".into());
        }
        if rated == 0.0 {
            constraints.push("configured condensation rate is zero".into());
        }
        if excess < rated {
            constraints.push("start vapor above the assumed humidity setpoint".into());
        }
        if space < rated {
            constraints.push("start condensate storage headroom".into());
        }
        if requested_w * seconds < planned_kg * self.electricity_j_kg {
            constraints.push("equipment electric power limit".into());
        }
        Plan {
            planned_kg,
            requested_w,
            saturation_pa: saturation,
            constraints,
        }
    }
}
pub struct Plan {
    pub planned_kg: f64,
    pub requested_w: f64,
    pub saturation_pa: Option<f64>,
    pub constraints: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub requested_w: f64,
    pub delivered_w: f64,
    pub crew_vapor_kg: f64,
    pub crew_phase_energy_j: f64,
    pub condensed_kg: f64,
    pub condensation_heat_j: f64,
    pub electricity_j: f64,
    pub start_saturation_pa: Option<f64>,
    pub constraints: Vec<String>,
}
impl Budget {
    pub fn from_power(
        c: &Config,
        mut plan: Plan,
        delivered_w: f64,
        crew_vapor_kg: f64,
        seconds: f64,
    ) -> Self {
        let condensed = plan
            .planned_kg
            .min(delivered_w * seconds / c.electricity_j_kg);
        if delivered_w + 1e-9 < plan.requested_w {
            plan.constraints
                .push("bus power after existing services and reactors".into());
        }
        Self {
            requested_w: plan.requested_w,
            delivered_w,
            crew_vapor_kg,
            crew_phase_energy_j: crew_vapor_kg * REFERENCE_PHASE_J_KG,
            condensed_kg: condensed,
            condensation_heat_j: condensed * REFERENCE_PHASE_J_KG,
            electricity_j: delivered_w * seconds,
            start_saturation_pa: plan.saturation_pa,
            constraints: plan.constraints,
        }
    }
    pub fn validate(&self, c: &Config, seconds: f64) -> Result<(), String> {
        for v in [
            self.requested_w,
            self.delivered_w,
            self.crew_vapor_kg,
            self.crew_phase_energy_j,
            self.condensed_kg,
            self.condensation_heat_j,
            self.electricity_j,
        ] {
            if !v.is_finite() || v < 0.0 {
                return Err("Invalid moisture interval quantity".into());
            }
        }
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-8 * a.abs().max(b.abs()).max(1.0);
        if self.delivered_w > self.requested_w + 1e-6
            || self.requested_w > c.power_limit_w + 1e-6
            || self.condensed_kg > c.rate_kg_s * seconds + 1e-9
            || !near(self.electricity_j, self.delivered_w * seconds)
            || !near(self.electricity_j, self.condensed_kg * c.electricity_j_kg)
            || !near(
                self.crew_phase_energy_j,
                self.crew_vapor_kg * REFERENCE_PHASE_J_KG,
            )
            || !near(
                self.condensation_heat_j,
                self.condensed_kg * REFERENCE_PHASE_J_KG,
            )
            || self.start_saturation_pa.is_none() && self.condensed_kg > 0.0
            || self
                .start_saturation_pa
                .is_some_and(|v| !v.is_finite() || v <= 0.0)
        {
            return Err("Moisture interval disagrees with phase/equipment energy budget".into());
        }
        Ok(())
    }
}
pub const WALL_MODEL: &str = "cosmogon-cabin-wall-condensation/1";
/// Passive signed vapor↔wall-liquid exchange. Wall temperature is an explicit linear
/// coupling assumption; the flux is a lumped surface mass-transfer assumption.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WallConfig {
    pub model: String,
    pub area_m2: f64,
    pub mass_transfer_kg_s_m2_pa: f64,
    pub wall_coupling: f64,
    pub liquid_capacity_kg: f64,
    pub rate_kg_s: f64,
}
impl Default for WallConfig {
    fn default() -> Self {
        Self {
            model: WALL_MODEL.into(),
            area_m2: 50.0,
            mass_transfer_kg_s_m2_pa: 1.0e-8,
            wall_coupling: 0.5,
            liquid_capacity_kg: 5.0,
            rate_kg_s: 1.0e-3,
        }
    }
}
impl WallConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != WALL_MODEL {
            return Err(format!("Unsupported wall condensation model {}", self.model));
        }
        for (v, min, max, label) in [
            (self.area_m2, 0.0, 1e5, "Wall condensation area (m²)"),
            (
                self.mass_transfer_kg_s_m2_pa,
                0.0,
                1e-6,
                "Assumed wall mass-transfer coefficient (kg/(s·m²·Pa))",
            ),
            (self.wall_coupling, 0.0, 1.0, "Assumed wall thermal coupling"),
            (
                self.liquid_capacity_kg,
                0.0,
                1e7,
                "Wall liquid film capacity (kg)",
            ),
            (self.rate_kg_s, 0.0, 100.0, "Wall exchange rate limit (kg/s)"),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!(
                    "{label} must be finite and between {min} and {max}"
                ));
            }
        }
        Ok(())
    }
    /// Signed net condensation (positive) or evaporation (negative) over one interval.
    /// Outside the NIST saturation domain no exchange occurs.
    pub fn exchange(
        &self,
        vapor_kg: f64,
        wall_liquid_kg: f64,
        cabin_temperature_k: f64,
        exterior_temperature_k: f64,
        volume_m3: f64,
        seconds: f64,
    ) -> (f64, WallBudget) {
        let wall_temperature_k =
            exterior_temperature_k + self.wall_coupling * (cabin_temperature_k - exterior_temperature_k);
        let saturation = saturation_pa(wall_temperature_k);
        let mut constraints = Vec::new();
        let Some(saturation) = saturation else {
            constraints.push("wall temperature outside the 273.15–303 K saturation fit".into());
            return (
                0.0,
                WallBudget {
                    wall_temperature_k: Some(wall_temperature_k),
                    saturation_pa: None,
                    net_condensed_kg: 0.0,
                    condensation_heat_j: 0.0,
                    constraints,
                },
            );
        };
        let p_vapor = vapor_pressure_pa(vapor_kg, cabin_temperature_k, volume_m3);
        let driving = p_vapor - saturation;
        let rated = self.mass_transfer_kg_s_m2_pa * self.area_m2 * driving * seconds;
        let capped = rated.clamp(-self.rate_kg_s * seconds, self.rate_kg_s * seconds);
        let net = if capped > 0.0 {
            capped.min(vapor_kg).min((self.liquid_capacity_kg - wall_liquid_kg).max(0.0))
        } else {
            capped.max(-wall_liquid_kg)
        };
        if rated > capped {
            constraints.push("finite wall exchange rate limit".into());
        }
        if capped > 0.0 && vapor_kg < capped {
            constraints.push("start vapor available for condensation".into());
        }
        if capped > 0.0 && self.liquid_capacity_kg - wall_liquid_kg < capped {
            constraints.push("wall liquid film capacity headroom".into());
        }
        if capped < 0.0 && wall_liquid_kg < -capped {
            constraints.push("held wall liquid available for evaporation".into());
        }
        (
            net,
            WallBudget {
                wall_temperature_k: Some(wall_temperature_k),
                saturation_pa: Some(saturation),
                net_condensed_kg: net,
                condensation_heat_j: net * REFERENCE_PHASE_J_KG,
                constraints,
            },
        )
    }
}
/// Last completed interval's passive wall exchange; signed condensation is positive.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WallBudget {
    pub wall_temperature_k: Option<f64>,
    pub saturation_pa: Option<f64>,
    pub net_condensed_kg: f64,
    pub condensation_heat_j: f64,
    pub constraints: Vec<String>,
}
impl WallBudget {
    pub fn validate(&self, c: &WallConfig, seconds: f64) -> Result<(), String> {
        for v in [self.net_condensed_kg, self.condensation_heat_j] {
            if !v.is_finite() {
                return Err("Invalid wall condensation interval quantity".into());
            }
        }
        if self.wall_temperature_k.is_some_and(|v| !v.is_finite() || v <= 0.0)
            || self.saturation_pa.is_some_and(|v| !v.is_finite() || v <= 0.0)
            || self.net_condensed_kg.abs() > c.rate_kg_s * seconds + 1e-9
            || (self.condensation_heat_j - self.net_condensed_kg * REFERENCE_PHASE_J_KG).abs()
                > 1e-8 * self.condensation_heat_j.abs().max(1.0)
            || (self.saturation_pa.is_none() && self.net_condensed_kg != 0.0)
        {
            return Err("Wall interval disagrees with retained wall configuration".into());
        }
        Ok(())
    }
}
pub fn leak_source() -> serde_json::Value {
    serde_json::json!({"model":crate::habitat::LEAK_MODEL,
        "version":"Explicit engineering assumption; no empirical leakage dataset claimed",
        "description":"Nominal slow cabin gas leakage proportional to current species masses with assumed constant cabin-gas specific heat enthalpy removal; independent of the pressure-driven breach orifice model.",
        "uncertainty":null,"limits":"No seal degradation, pressure- or damage-driven flow, inward/rarefied flow, humidity-dependent leak rate, wall icing or collection plumbing."})
}
pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"saturation_source":SATURATION_SOURCE,"gas_source":GAS_SOURCE,"liquid_source":crate::electrolysis::ENTHALPY_SOURCE,"crew_source":crate::habitat::SOURCE,
        "version":"NIST SRD69 selected coefficients/CODATA, retrieved 2026-10-09; NASA BVAD Rev2 Table3-31",
        "antoine":{"a":5.40221,"b":1838.675,"c":-31.737,"equation":"log10(P_bar)=A-B/(T_K+C)","supported_temperature_k":[273.15,303.0]},
        "wall_condensation":{"model":WALL_MODEL,"flux":"rate_kg_s = k·area·(p_vapor − p_sat(T_wall)) with T_wall = T_exterior + coupling·(T_cabin − T_exterior)","sign":"Positive net condenses vapor onto walls; negative evaporates held wall liquid","phase_energy":"net·REFERENCE_PHASE_J_KG released (condensation) or debited (evaporation) from cabin sensible heat","domain":"Wall temperature must lie in the 273.15–303 K NIST saturation fit; otherwise no exchange","film":"Bounded by liquid_capacity_kg and a finite rate_kg_s cap; wall liquid is held, with no recycling path"},
        "water_molar_mass_kg_mol":WATER_MOLAR_KG,"gas_constant_j_mol_k":R,"gas_formation_kj_mol":-241.826,"liquid_formation_kj_mol":-285.830,"reference_phase_j_kg":REFERENCE_PHASE_J_KG,"reference_temperature_k":298.15,
        "nominal_respiration_perspiration_kg_person_day":2.946,"reference_latent_j_person_day":6_118_000.0,"inferred_default_airborne_fraction":REFERENCE_AIRBORNE_FRACTION,
        "uncertainty":null,"limits":"Well-mixed ideal vapor and constant reference phase enthalpy; assumed moisture partition, equipment and passive wall coupling (lumped mass transfer, wall temperature by linear cabin↔exterior coupling). No ice condensation, temperature-dependent latent heat/COP, purification quality, crop exchange or field calibration. Supersaturation is retained explicitly."})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{self, Definition, Environment, Habitat},
        BodyRef,
    };
    fn env() -> Environment {
        Environment {
            irradiance_w_m2: Some(0.0),
            exterior_temperature_k: Some(293.15),
        }
    }
    fn experiment() -> Habitat {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        d.config.crew = 1;
        d.config.moisture = Some(Config::default());
        d.config.firm_power_w = 10_000.0;
        d.config.base_load_w = 0.0;
        d.config.scrubber_power_w = 0.0;
        d.config.recycler_power_w = 0.0;
        d.config.heater_limit_w = 0.0;
        d.config.radiator_pump_w = 0.0;
        d.config.radiator_area_m2 = 0.0;
        d.config.conductance_w_k = 0.0;
        d.config.heat_capacity_j_k = 1e9;
        Habitat::new(0, d, 0.0).unwrap()
    }
    fn near(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "{a} vs {b}");
    }
    #[test]
    fn source_arithmetic_and_domain() {
        for (t, p) in [
            (273.15, 610.8086256559275),
            (293.15, 2336.727117682812),
            (303.0, 4207.358396638073),
        ] {
            near(saturation_pa(t).unwrap(), p, 1e-8);
        }
        for t in [273.149, 303.001, 500.0, f64::NAN] {
            assert_eq!(saturation_pa(t), None);
        }
        near(REFERENCE_PHASE_J_KG, 2_442_590.4647716107, 1e-6);
        near(
            2.946 * REFERENCE_AIRBORNE_FRACTION * REFERENCE_PHASE_J_KG,
            6_118_000.0,
            1e-8,
        );
        near(
            relative_humidity(1.0, 293.15, 500.0).unwrap(),
            vapor_pressure_pa(1.0, 293.15, 500.0) / 2336.727117682812,
            1e-12,
        );
    }
    #[test]
    fn closed_mass_electric_and_phase_energy() {
        let mut h = experiment();
        let initial = h.state.clone();
        for n in 1..=1440 {
            h.step(n as f64 * 60.0, env()).unwrap();
            assert!(h.budget.mass_residual_kg.abs() < 1e-9);
            assert!(h.budget.electric_residual_j.abs() < 1e-6);
            h.validate(h.last_time).unwrap();
        }
        near(h.state.mass_kg(), initial.mass_kg(), 1e-8);
        near(
            h.totals.crew_vapor_kg,
            2.946 * REFERENCE_AIRBORNE_FRACTION,
            1e-10,
        );
        near(
            h.state.water_vapor_kg + h.state.condensate_kg,
            2.946 * REFERENCE_AIRBORNE_FRACTION,
            1e-10,
        );
        near(
            h.state.wastewater_kg,
            4.467 - 2.946 * REFERENCE_AIRBORNE_FRACTION,
            1e-10,
        );
        let thermal = h.config.heat_capacity_j_k * (h.state.temperature_k - initial.temperature_k);
        near(
            thermal + h.state.phase_energy_j(),
            12_426_000.0 + h.totals.delivered_j + h.totals.losses_j,
            0.2,
        );
        // Reference pressure includes finite vapor; dry-gas laws remain separate.
        let dry = (h.state.oxygen_kg / 0.0319988
            + h.state.nitrogen_kg / 0.0280134
            + h.state.co2_kg / 0.0440095)
            * R
            * h.state.temperature_k
            / h.config.volume_m3;
        near(
            h.state.pressure_pa(&h.config) - dry,
            h.state.vapor_pressure_pa(&h.config),
            1e-10,
        );
    }
    #[test]
    fn start_stocks_later_recycling_and_disable_retain_mass() {
        let mut h = experiment();
        h.config
            .moisture
            .as_mut()
            .unwrap()
            .relative_humidity_setpoint = 0.0;
        h.config.recycler_power_w = 100.0;
        h.config.recycler_rate_kg_s = 1.0;
        h.step(60.0, env()).unwrap();
        assert_eq!(h.budget.moisture.as_ref().unwrap().condensed_kg, 0.0);
        let old_vapor = h.state.water_vapor_kg;
        h.step(120.0, env()).unwrap();
        near(
            h.budget.moisture.as_ref().unwrap().condensed_kg,
            old_vapor,
            1e-12,
        );
        assert_eq!(h.budget.condensate_processed_kg, 0.0);
        let collected = h.state.condensate_kg;
        let vapor = h.state.water_vapor_kg;
        let mass = h.state.mass_kg();
        h.config.moisture = None;
        h.config.crew = 0;
        h.step(180.0, env()).unwrap();
        near(h.budget.condensate_processed_kg, collected, 1e-12);
        near(
            h.budget.condensate_recovered_kg,
            collected * h.config.water_recovery_fraction,
            1e-12,
        );
        assert_eq!(h.state.water_vapor_kg, vapor);
        assert_eq!(h.state.condensate_kg, 0.0);
        near(h.state.mass_kg(), mass, 1e-10);
        assert!(h.uses_moisture());
        h.validate(180.0).unwrap();
    }
    #[test]
    fn finite_limits_no_power_unsupported_temperature_and_starvation() {
        for which in 0..5 {
            let mut h = experiment();
            h.config
                .moisture
                .as_mut()
                .unwrap()
                .relative_humidity_setpoint = 0.0;
            h.state.water_vapor_kg = 1.0;
            h.totals.crew_vapor_kg = 1.0;
            match which {
                0 => h.config.moisture.as_mut().unwrap().power_limit_w = 0.0,
                1 => {
                    h.config.firm_power_w = 0.0;
                    h.state.battery_j = 0.0;
                }
                2 => h.config.moisture.as_mut().unwrap().condensate_capacity_kg = 0.0,
                3 => h.state.temperature_k = 310.0,
                _ => h.config.moisture.as_mut().unwrap().rate_kg_s = 0.0,
            }
            h.step(60.0, env()).unwrap();
            let b = h.budget.moisture.as_ref().unwrap();
            assert_eq!(b.condensed_kg, 0.0);
            assert!(!b.constraints.is_empty());
            assert!(h.state.water_vapor_kg >= 1.0);
            h.validate(60.0).unwrap();
        }
        let mut h = experiment();
        h.state.potable_water_kg = 0.0;
        h.step(60.0, env()).unwrap();
        assert_eq!(h.budget.moisture.as_ref().unwrap().crew_vapor_kg, 0.0);
        assert_eq!(h.state.water_vapor_kg, 0.0);
        let mut h = experiment();
        h.config
            .moisture
            .as_mut()
            .unwrap()
            .relative_humidity_setpoint = 0.0;
        h.config.moisture.as_mut().unwrap().condensate_capacity_kg = 0.0001;
        h.step(60.0, env()).unwrap();
        h.step(120.0, env()).unwrap();
        near(h.state.condensate_kg, 0.0001, 1e-12);
        let mut c = h.config.clone();
        c.moisture.as_mut().unwrap().condensate_capacity_kg = 0.0;
        assert!(h.check_config(&c).is_err());
    }
    #[test]
    fn strict_config_and_corrupt_phase_history_are_rejected() {
        for which in 0..7 {
            let mut c = Config::default();
            match which {
                0 => c.model = "future/2".into(),
                1 => c.airborne_fraction = f64::NAN,
                2 => c.airborne_fraction = 1.01,
                3 => c.relative_humidity_setpoint = -0.01,
                4 => c.electricity_j_kg = 0.0,
                5 => c.rate_kg_s = f64::INFINITY,
                _ => c.condensate_capacity_kg = -1.0,
            };
            assert!(c.validate().is_err());
        }
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v["guaranteed_recovery"] = true.into();
        assert!(serde_json::from_value::<Config>(v).is_err());
        let mut h = experiment();
        h.step(60.0, env()).unwrap();
        h.state.water_vapor_kg += 0.001;
        assert!(h.validate(60.0).is_err());
        let mut h = experiment();
        h.step(60.0, env()).unwrap();
        h.budget.moisture.as_mut().unwrap().crew_phase_energy_j += 100.0;
        assert!(h.validate(60.0).is_err());
    }
    #[test]
    fn diagnostics_close_optional_phase_and_keep_missing_evidence_unavailable() {
        let mut h = experiment();
        h.config
            .moisture
            .as_mut()
            .unwrap()
            .relative_humidity_setpoint = 0.0;
        h.step(60.0, env()).unwrap();
        h.step(120.0, env()).unwrap();
        let d = crate::habitat_diagnostics::explain(&h);
        let get = |id| d.heat.iter().find(|t| t.id == id).unwrap().value;
        near(get("sum").unwrap(), h.budget.thermal_storage_w, 0.001);
        near(
            d.allocations.iter().map(|a| a.served_w.unwrap()).sum(),
            h.budget.delivered_w,
            1e-9,
        );
        assert!(get("crew_vapor_phase").unwrap() < 0.0);
        assert!(get("condensation").unwrap() > 0.0);
        h.budget.moisture = None;
        let d = crate::habitat_diagnostics::explain(&h);
        assert_eq!(d.heat.iter().find(|t| t.id == "sum").unwrap().value, None);
        let legacy = habitat::Config::default();
        let v = serde_json::to_value(&legacy).unwrap();
        assert!(v.get("moisture").is_none());
    }
    #[test]
    fn coupled_save_sources_exports_and_recorded_observations() {
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
        d.config.moisture = Some(Config {
            relative_humidity_setpoint: 0.0,
            ..Default::default()
        });
        a.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        let initial = a.clone();
        let first = habitat::sample(&a, 0).unwrap();
        let mut b = a.clone();
        a.advance_by(7200.0);
        for _ in 0..240 {
            b.advance_by(30.0);
        }
        assert_eq!(a, b);
        let saved = save::to_json(&a, "moisture").unwrap();
        let mut restored = save::from_json(&saved).unwrap();
        a.advance_by(60.0);
        restored.advance_by(60.0);
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        for pin in [None, Some("future/2")] {
            let mut v: serde_json::Value = serde_json::from_str(&saved).unwrap();
            if let Some(vv) = pin {
                v["header"]["data_versions"]["cosmogon-cabin-moisture"] = vv.into();
            } else {
                v["header"]["data_versions"]
                    .as_object_mut()
                    .unwrap()
                    .remove("cosmogon-cabin-moisture");
            }
            assert!(save::from_json(&v.to_string()).is_err());
        }
        let mut c = a.habitats[0].config.clone();
        c.moisture = None;
        a.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        assert!(save::data_versions(&a).contains_key("cosmogon-cabin-moisture"));
        let root =
            std::env::temp_dir().join(format!("cosmogon-moisture-bundle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let rows = [first, habitat::sample(&a, 0).unwrap()];
        habitat::export(&initial, &a, 0, &rows, &root).unwrap();
        let rec = crate::habitat_recording::Recording::read(&root).unwrap();
        assert_eq!(rec.rows, rows);
        assert!(crate::habitat_recording::Metric::Humidity
            .value(&rec.rows[1])
            .is_some());
        assert!(habitat::export(&initial, &a, 0, &rows, &root).is_err());
        std::fs::remove_dir_all(root).unwrap();

    }
    #[test]
    fn wall_exchange_is_signed_bounded_and_domain_refused() {
        let w = WallConfig::default();
        // Cold wall in domain: supersaturated cabin vapor condenses, bounded by film headroom.
        let small = WallConfig {
            liquid_capacity_kg: 1.0e-9,
            ..Default::default()
        };
        let (net, b) = w.exchange(20.0, 0.0, 293.15, 275.15, 500.0, 60.0);
        assert!(net > 0.0);
        assert!(net <= w.rate_kg_s * 60.0 + 1e-12);
        assert!(b.condensation_heat_j > 0.0);
        near(b.condensation_heat_j, net * REFERENCE_PHASE_J_KG, 1e-6);
        let (capped, _) = small.exchange(20.0, 0.0, 293.15, 275.15, 500.0, 60.0);
        assert_eq!(capped, 1.0e-9);
        // Warm wall with a held film evaporates, bounded by held liquid.
        let (evap, eb) = w.exchange(1.0e-6, 0.5, 293.15, 303.0, 500.0, 60.0);
        assert!(evap < 0.0 && (-0.5..=0.0).contains(&evap));
        assert!(eb.condensation_heat_j < 0.0);
        let (dry, db) = w.exchange(1.0e-9, 0.0, 293.15, 303.0, 500.0, 60.0);
        assert_eq!(dry, 0.0);
        assert!(db.constraints.iter().any(|c| c.contains("evaporation")));
        // Outside the saturation domain the exchange is exactly zero.
        let (none, nb) = w.exchange(1.0, 0.5, 293.15, 200.0, 500.0, 60.0);
        assert_eq!(none, 0.0);
        assert!(nb.saturation_pa.is_none());
        assert!(nb.constraints.iter().any(|c| c.contains("saturation fit")));
        nb.validate(&w, 60.0).unwrap();
        b.validate(&w, 60.0).unwrap();
        eb.validate(&w, 60.0).unwrap();
    }
    #[test]
    fn wall_and_leak_config_validation_refuses_invalid_models_and_bounds() {
        let w = WallConfig {
            model: "future/2".into(),
            ..Default::default()
        };
        assert!(w.validate().is_err());
        let w = WallConfig {
            mass_transfer_kg_s_m2_pa: f64::NAN,
            ..Default::default()
        };
        assert!(w.validate().is_err());
        let l = crate::habitat::LeakConfig {
            model: "future/2".into(),
            ..Default::default()
        };
        assert!(l.validate().is_err());
        let l = crate::habitat::LeakConfig {
            gas_specific_heat_j_kg: 0.0,
            ..Default::default()
        };
        assert!(l.validate().is_err());
    }
}
