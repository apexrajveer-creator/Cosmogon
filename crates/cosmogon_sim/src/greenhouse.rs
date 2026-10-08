//! Source-pinned controlled-regime crop planning. This is not an installed greenhouse:
//! yields require sustained boundary inputs and never credit habitat inventories.
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

pub const MODEL: &str = "cosmogon-lettuce-mec/2";
pub const LEGACY_MODEL: &str = "cosmogon-lettuce-mec/1";
pub const WATER_MODEL: &str = "cosmogon-lettuce-transpiration-mec/1";
pub const SOURCE: &str = crate::habitat::SOURCE;
pub const SOURCE_SHA256: &str = "b9eab584c2bc88b52f6eb6744e321eb72ca58003e8ac77dc1ebeb034aba11d1e";
pub const LIMITS: &str = "Controlled terrestrial chamber forecast, Waldmann's Green lettuce only. Fixed 23 C light/dark, 16 h light in a 24 h day, 19.2 plants/m2, harvest at 30 days after emergence. Unlimited regulated CO2, water and nutrients. Optional light-period transpiration requires user-assumed maintained humidity at fixed assumed 101.325 kPa; negative source conductance is refused. No humidity dynamics, dark transpiration, crop stress, nutrient chemistry, disease, pressure/gravity correction, fresh mass or nutritional adequacy. Transpired water is a modeled vapor transfer, not irrigation demand, potable water, condensate recovery or full facility consumption. Fixture efficacy is user assumed; cooling, pumps and dehumidification energy unavailable. No installed greenhouse, habitat stock transfer, future electricity guarantee or independent crop validation. Predictive uncertainty unquantified.";
const CQY: [f64; 6] = [
    0.044763,
    -1.1701e-5,
    5.163e-5,
    -1.9731e-11,
    -2.075e-8,
    8.9265e-15,
];
const N: f64 = 2.5;
const CUE: f64 = 0.625;
const BCF: f64 = 0.40;
const OPF: f64 = 1.08;
const CARBON_G_MOL: f64 = 12.011;
const TEMPERATURE_C: f64 = 23.0;
const PRESSURE_KPA: f64 = 101.325;
const WATER_G_MOL: f64 = 18.015;
const WATER_G_L: f64 = 998.23;
const AERODYNAMIC_CONDUCTANCE: f64 = 2.5;
const WATER_SUBDIVISIONS: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaterAssumptions {
    pub model: String,
    /// Explicit maintained chamber assumption, never taken from the live habitat.
    pub relative_humidity: f64,
}
impl Default for WaterAssumptions {
    fn default() -> Self {
        Self {
            model: WATER_MODEL.into(),
            relative_humidity: 0.7,
        }
    }
}
impl WaterAssumptions {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != WATER_MODEL {
            return Err("Unsupported transpiration model".into());
        }
        if !self.relative_humidity.is_finite() || !(0.0..=1.0).contains(&self.relative_humidity) {
            return Err("Assumed relative humidity must be a finite fraction from 0 to 1".into());
        }
        if self.conductance_factor() < 0.0 {
            return Err("Humidity is outside the source equation's nonnegative conductance domain at 23 C; no dry-air crop-stress model is available".into());
        }
        Ok(())
    }
    fn saturated_vapor_kpa() -> f64 {
        0.611 * (17.4 * TEMPERATURE_C / (TEMPERATURE_C + 239.0)).dexp()
    }
    fn deficit_kpa(&self) -> f64 {
        Self::saturated_vapor_kpa() * (1.0 - self.relative_humidity)
    }
    fn conductance_factor(&self) -> f64 {
        1.717 * TEMPERATURE_C - 19.96 - 10.54 * self.deficit_kpa()
    }
    /// BVAD 4-24–29 use light-cycle net photosynthesis, not the daily CUE gain.
    fn row(&self, absorbed: f64, q: f64, p: f64, c: f64, area: f64) -> WaterRow {
        let gross = absorbed * q * p;
        let net = ((24.0 - 16.0) / 24.0 + 16.0 * CUE / 24.0) * gross;
        let stomatal = self.conductance_factor() * net / c;
        let canopy = AERODYNAMIC_CONDUCTANCE * stomatal / (AERODYNAMIC_CONDUCTANCE + stomatal);
        let liters = 3600.0
            * 16.0
            * (WATER_G_MOL / WATER_G_L)
            * canopy
            * (self.deficit_kpa() / PRESSURE_KPA)
            * area;
        WaterRow {
            gross_carbon_umol_m2_s: gross,
            light_cycle_net_carbon_umol_m2_s: net,
            stomatal_conductance_mol_m2_s: stomatal,
            canopy_conductance_mol_m2_s: canopy,
            transpired_water_l_day: liters,
            transpired_vapor_kg_day: liters * WATER_G_L / 1000.0,
            transpired_water_l: 0.0,
            transpired_vapor_kg: 0.0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WaterRow {
    pub gross_carbon_umol_m2_s: f64,
    pub light_cycle_net_carbon_umol_m2_s: f64,
    pub stomatal_conductance_mol_m2_s: f64,
    pub canopy_conductance_mol_m2_s: f64,
    pub transpired_water_l_day: f64,
    pub transpired_vapor_kg_day: f64,
    pub transpired_water_l: f64,
    pub transpired_vapor_kg: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WaterForecast {
    pub model: String,
    pub assumed_relative_humidity: f64,
    pub assumed_pressure_kpa: f64,
    pub light_temperature_c: f64,
    pub saturated_vapor_pressure_kpa: f64,
    pub air_vapor_pressure_kpa: f64,
    pub vapor_pressure_deficit_kpa: f64,
    pub aerodynamic_conductance_mol_m2_s: f64,
    pub water_molecular_g_mol: f64,
    pub water_density_g_l: f64,
    pub source_equations: String,
    pub coefficients: serde_json::Value,
    pub integration_method: String,
    pub subdivisions_per_day: usize,
    /// Difference of deterministic 256 and 512 subdivisions/day at harvest.
    /// Numerical diagnostic only, not an error bound or biological uncertainty.
    pub harvest_resolution_difference_l: f64,
    pub predictive_uncertainty_l: Option<f64>,
    pub semantics: String,
}

fn simpson(f: impl Fn(f64) -> f64, start: f64, subdivisions: usize) -> f64 {
    let h = 1.0 / subdivisions as f64;
    let mut sum = f(start) + f(start + 1.0);
    for i in 1..subdivisions {
        sum += (if i % 2 == 0 { 2.0 } else { 4.0 }) * f(start + i as f64 * h);
    }
    sum * h / 3.0
}

/// Exactly the five lettuce low-light cases in BVAD Rev2 Table 4-123. No invented
/// interpolation, arbitrary fit extrapolation or nominal Amax=0.93 substitution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    P200C330,
    P200C660,
    P200C990,
    P200C1320,
    P300C330,
}
impl Regime {
    pub const ALL: [Self; 5] = [
        Self::P200C330,
        Self::P200C660,
        Self::P200C990,
        Self::P200C1320,
        Self::P300C330,
    ];
    pub fn values(self) -> (f64, f64, f64) {
        match self {
            Self::P200C330 => (200.0, 330.0, 0.18),
            Self::P200C660 => (200.0, 660.0, 0.35),
            Self::P200C990 => (200.0, 990.0, 0.46),
            Self::P200C1320 => (200.0, 1320.0, 0.49),
            Self::P300C330 => (300.0, 330.0, 0.75),
        }
    }
    pub fn label(self) -> String {
        let (p, c, _) = self.values();
        format!("{p:.0} µmol light/(m² s) · {c:.0} µmol CO₂/mol air")
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub regime: Regime,
    pub area_m2: f64,
    /// Incident photons at planted plane per electric joule, including fixture/optical
    /// losses. Not a measured flight or horticulture fixture rating.
    pub fixture_efficacy_umol_j: f64,
    /// Missing in version 1: water remains unavailable, with no assumed migration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transpiration: Option<WaterAssumptions>,
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            regime: Regime::P200C660,
            area_m2: 20.0,
            fixture_efficacy_umol_j: 2.5,
            transpiration: Some(WaterAssumptions::default()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub age_days_after_emergence: f64,
    pub canopy_absorbed_fraction: f64,
    pub net_carbon_mol_day: f64,
    pub net_co2_kg_day: f64,
    pub net_oxygen_kg_day: f64,
    pub dry_biomass_kg_day: f64,
    pub total_dry_biomass_kg: f64,
    pub edible_dry_biomass_kg: f64,
    pub inedible_dry_biomass_kg: f64,
    pub net_carbon_mol: f64,
    pub net_co2_kg: f64,
    pub net_oxygen_kg: f64,
    pub lighting_electric_j: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transpiration: Option<WaterRow>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forecast {
    pub format: String,
    pub schema_version: u32,
    pub definition: Definition,
    pub source: String,
    pub source_version: String,
    pub source_snapshot_sha256: String,
    pub cultivar: String,
    pub conditions: serde_json::Value,
    pub coefficients: serde_json::Value,
    pub data_states: serde_json::Value,
    pub uncertainty: Option<f64>,
    pub limitations: String,
    pub canopy_maximum_absorption_age_days: f64,
    pub quantum_yield_mol_carbon_per_mol_absorbed_photons: f64,
    pub daily_light_integral_mol_m2_day: f64,
    pub lamps_on_power_w: f64,
    pub daily_lighting_electric_j: f64,
    pub rows: Vec<Row>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transpiration: Option<WaterForecast>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL && self.model != LEGACY_MODEL {
            return Err("Unsupported greenhouse planning model".into());
        }
        for (v, min, max, name) in [
            (self.area_m2, 0.1, 100_000.0, "Planted area (m²)"),
            (
                self.fixture_efficacy_umol_j,
                0.1,
                10.0,
                "Assumed fixture efficacy (µmol/J)",
            ),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!("{name} must be finite and between {min} and {max}"));
            }
        }
        if let Some(water) = &self.transpiration {
            if self.model == LEGACY_MODEL {
                return Err("Version-1 crop definitions are biomass-only; explicitly choose version 2 before adding transpiration".into());
            }
            water.validate()?;
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 65_536 {
            return Err("Greenhouse definition must be a regular file of at most 64 KiB".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65_536 {
            return Err("Greenhouse definition exceeds 64 KiB".into());
        }
        let d: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        d.validate()?;
        Ok(d)
    }
    pub fn calculate(&self) -> Result<Forecast, String> {
        self.validate()?;
        let (p, c, a) = self.regime.values();
        // Six nonzero coefficients, Table 4-102. PPF is incident µmol/(m² s),
        // CO2 is µmol/mol AIR, not partial pressure in Pa.
        let q = CQY[0]
            + CQY[1] * p
            + CQY[2] * c
            + CQY[3] * c * p * p
            + CQY[4] * c * c
            + CQY[5] * c * c * p * p;
        if !q.is_finite() || q <= 0.0 {
            return Err("Source fit returned unsupported quantum yield".into());
        }
        let lamps = p * self.area_m2 / self.fixture_efficacy_umol_j;
        let daily_electric = lamps * 16.0 * 3600.0;
        let carbon_per_absorbed_day = 0.0036 * 16.0 * CUE * q * p * self.area_m2;
        // Every supported Table 4-123 case has tA=32 d; analytic antiderivative
        // of Amax(t/tA)^n avoids arbitrary Euler timestep/yield dependence.
        let absorbed_integral = |t: f64| a * t * (t / 32.0).dpowf(N) / (N + 1.0);
        let mut rows = Vec::with_capacity(31);
        let water_rate = |t: f64, w: &WaterAssumptions| {
            w.row(a * (t / 32.0).dpowf(N), q, p, c, self.area_m2)
                .transpired_water_l_day
        };
        let mut water_l = 0.0;
        let mut finer_water_l = 0.0;
        for day in 0..=30 {
            let t = day as f64;
            let absorbed = a * (t / 32.0).dpowf(N);
            let carbon_day = carbon_per_absorbed_day * absorbed;
            let carbon = carbon_per_absorbed_day * absorbed_integral(t);
            let dry = carbon * CARBON_G_MOL / (BCF * 1000.0);
            // Equation 4-21: edible allocation starts after tE=1 d, not day 0.
            let edible = 0.95
                * carbon_per_absorbed_day
                * (absorbed_integral(t) - absorbed_integral(t.min(1.0)))
                * CARBON_G_MOL
                / (BCF * 1000.0);
            let transpiration = self.transpiration.as_ref().map(|w| {
                if day > 0 {
                    water_l += simpson(|t| water_rate(t, w), t - 1.0, WATER_SUBDIVISIONS);
                    finer_water_l += simpson(|t| water_rate(t, w), t - 1.0, WATER_SUBDIVISIONS * 2);
                }
                let mut row = w.row(absorbed, q, p, c, self.area_m2);
                row.transpired_water_l = water_l;
                row.transpired_vapor_kg = water_l * WATER_G_L / 1000.0;
                row
            });
            rows.push(Row {
                age_days_after_emergence: t,
                canopy_absorbed_fraction: absorbed,
                net_carbon_mol_day: carbon_day,
                net_co2_kg_day: carbon_day * 0.0440095,
                net_oxygen_kg_day: carbon_day * OPF * 0.0319988,
                dry_biomass_kg_day: carbon_day * CARBON_G_MOL / (BCF * 1000.0),
                total_dry_biomass_kg: dry,
                edible_dry_biomass_kg: edible,
                inedible_dry_biomass_kg: dry - edible,
                net_carbon_mol: carbon,
                net_co2_kg: carbon * 0.0440095,
                net_oxygen_kg: carbon * OPF * 0.0319988,
                lighting_electric_j: daily_electric * t,
                transpiration,
            });
        }
        Ok(Forecast {
            format:"cosmogon-greenhouse-forecast".into(),
            schema_version:2,
            definition:self.clone(),
            source:SOURCE.into(),
            source_snapshot_sha256:SOURCE_SHA256.into(),
            source_version:"NASA/TP-2015-218570/REV2, February 2022; Eq4-14,4-17–4-21,4-23–4-27,4-29; Tables4-97,4-99,4-102,4-110–4-113,4-123".into(),
            cultivar:"Lettuce — Waldmann's Green".into(),
            conditions:serde_json::json!({"ppf_umol_m2_s":p,"co2_umol_mol_air":c,"canopy_maximum_absorbed_fraction":a,"canopy_maximum_absorption_age_days":32,"light_hours_per_terrestrial_day":16,"cycle_seconds":86400,"light_temperature_k":296.15,"dark_temperature_k":296.15,"planting_density_plants_m2":19.2,"harvest_days_after_emergence":30,"water_and_nutrients":"unlimited controlled boundary assumption"}),
            coefficients:serde_json::json!({"canopy_exponent":N,"carbon_use_efficiency":CUE,"biomass_carbon_fraction":BCF,"oxygen_mol_per_mol_carbon":OPF,"carbon_g_mol":CARBON_G_MOL,"edible_allocation_fraction_after_day_1":0.95,"quantum_yield_terms":["1","PPF","CO2","CO2*PPF^2","CO2^2","CO2^2*PPF^2"],"quantum_yield_coefficients":CQY}),
            data_states:serde_json::json!({"regime_and_coefficients":"published controlled-crop model inputs","area_and_fixture_efficacy":"user assumed","yields_and_lighting":"modelled","humidity_and_pressure":if self.transpiration.is_some(){"user assumed maintained boundary"}else{"unavailable; no implicit legacy migration"},"transpiration":if self.transpiration.is_some(){"modelled light-period vapor transfer"}else{"unavailable"},"irrigation_nutrients_humidity_dynamics_facility_energy":"unavailable","field_validation":"unavailable"}),
            uncertainty:None,
            limitations:LIMITS.into(),
            canopy_maximum_absorption_age_days:32.0,
            quantum_yield_mol_carbon_per_mol_absorbed_photons:q,
            daily_light_integral_mol_m2_day:p*16.0*3600.0/1e6,
            lamps_on_power_w:lamps,
            daily_lighting_electric_j:daily_electric,
            rows,
            transpiration:self.transpiration.as_ref().map(|w| WaterForecast {
                model:w.model.clone(), assumed_relative_humidity:w.relative_humidity,
                assumed_pressure_kpa:PRESSURE_KPA, light_temperature_c:TEMPERATURE_C,
                saturated_vapor_pressure_kpa:WaterAssumptions::saturated_vapor_kpa(),
                air_vapor_pressure_kpa:WaterAssumptions::saturated_vapor_kpa()*w.relative_humidity,
                vapor_pressure_deficit_kpa:w.deficit_kpa(),
                aerodynamic_conductance_mol_m2_s:AERODYNAMIC_CONDUCTANCE,
                water_molecular_g_mol:WATER_G_MOL, water_density_g_l:WATER_G_L,
                source_equations:"BVAD Rev2 4-23–4-27 (planophile), 4-29; table 4-110".into(),
                coefficients:serde_json::json!({"svp_base_kpa":0.611,"svp_temperature_numerator":17.4,"svp_temperature_offset_c":239.0,"planophile_temperature_factor":1.717,"planophile_offset":19.96,"planophile_deficit_factor":10.54,"light_cycle_net_factor":(24.0-16.0)/24.0+16.0*CUE/24.0}),
                integration_method:"Deterministic composite Simpson over each terrestrial day; doubled-resolution difference is numerical evidence only, not an error bound".into(),
                subdivisions_per_day:WATER_SUBDIVISIONS,
                harvest_resolution_difference_l:(water_l-finer_water_l).abs(),
                predictive_uncertainty_l:None,
                semantics:"Light-period liquid-equivalent vapor transfer under maintained humidity/pressure. Not irrigation demand, potable water production, condensate recovery or facility consumption. No dark transpiration, crop stress, pressure correction or habitat inventory exchange.".into(),
            }),
        })
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    pub definition: Definition,
    pub harvest: Row,
    pub lamps_on_power_w: f64,
    pub daily_light_integral_mol_m2_day: f64,
}
/// Hold planted area and assumed fixture efficacy fixed. Some cases change both PPF
/// and CO2, so this comparison alone is not a single-variable causal experiment.
pub fn compare(d: &Definition) -> Result<Vec<Comparison>, String> {
    Regime::ALL
        .into_iter()
        .map(|regime| {
            let definition = Definition {
                regime,
                ..d.clone()
            };
            let f = definition.calculate()?;
            Ok(Comparison {
                definition,
                harvest: f.rows[30].clone(),
                lamps_on_power_w: f.lamps_on_power_w,
                daily_light_integral_mol_m2_day: f.daily_light_integral_mol_m2_day,
            })
        })
        .collect()
}

/// Protected reproducibility bundle. Optional exact world is context, not a future
/// supply or maintained-regime guarantee. No mutation of the universe occurs.
pub fn export(
    d: &Definition,
    context: Option<(&crate::Universe, Option<u64>)>,
    path: &Path,
) -> Result<(), String> {
    let report = d.calculate()?;
    let comparison = compare(d)?;
    let context_json = context
        .map(|(u, _)| {
            crate::save::to_json(u, "Greenhouse planning context").map_err(|e| e.to_string())
        })
        .transpose()?;
    let context_fingerprint = context
        .map(|(u, _)| crate::research::state_fingerprint(u))
        .transpose()?;
    let observed = match context {
        Some((u, Some(id))) => Some(crate::habitat::sample(u, id)?.habitat.report()),
        _ => None,
    };
    if path.exists() {
        return Err("Greenhouse output folder already exists; choose a new folder".into());
    }
    std::fs::create_dir(path).map_err(|e| e.to_string())?;
    let write = |name: &str, bytes: &[u8]| -> Result<(), String> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())
    };
    let json = |name: &str, value: &serde_json::Value| -> Result<(), String> {
        write(
            name,
            &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
        )
    };
    let result = (|| {
        json(
            "comparison.json",
            &serde_json::to_value(&comparison).map_err(|e| e.to_string())?,
        )?;
        json(
            "definition.json",
            &serde_json::to_value(d).map_err(|e| e.to_string())?,
        )?;
        json(
            "forecast.json",
            &serde_json::to_value(&report).map_err(|e| e.to_string())?,
        )?;
        let mut csv="age_days_after_emergence,canopy_absorbed_fraction,net_carbon_mol_day,net_co2_kg_day,net_oxygen_kg_day,dry_biomass_kg_day,total_dry_biomass_kg,edible_dry_biomass_kg,inedible_dry_biomass_kg,net_carbon_mol,net_co2_kg,net_oxygen_kg,lighting_electric_j,gross_carbon_umol_m2_s,light_cycle_net_carbon_umol_m2_s,stomatal_conductance_mol_m2_s,canopy_conductance_mol_m2_s,transpired_water_l_day,transpired_vapor_kg_day,transpired_water_l,transpired_vapor_kg\n".to_string();
        for r in &report.rows {
            csv += &format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                r.age_days_after_emergence,
                r.canopy_absorbed_fraction,
                r.net_carbon_mol_day,
                r.net_co2_kg_day,
                r.net_oxygen_kg_day,
                r.dry_biomass_kg_day,
                r.total_dry_biomass_kg,
                r.edible_dry_biomass_kg,
                r.inedible_dry_biomass_kg,
                r.net_carbon_mol,
                r.net_co2_kg,
                r.net_oxygen_kg,
                r.lighting_electric_j
            );
            if let Some(w) = &r.transpiration {
                csv += &format!(
                    ",{},{},{},{},{},{},{},{}\n",
                    w.gross_carbon_umol_m2_s,
                    w.light_cycle_net_carbon_umol_m2_s,
                    w.stomatal_conductance_mol_m2_s,
                    w.canopy_conductance_mol_m2_s,
                    w.transpired_water_l_day,
                    w.transpired_vapor_kg_day,
                    w.transpired_water_l,
                    w.transpired_vapor_kg
                );
            } else {
                // Unavailable legacy quantities are empty cells, never fabricated zeros.
                csv += ",,,,,,,,\n";
            }
        }
        write("samples.csv", csv.as_bytes())?;
        if let Some(json) = &context_json {
            write("context.cosmo", json.as_bytes())?;
        }
        json(
            "manifest.json",
            &serde_json::json!({"format":"cosmogon-greenhouse-study","schema_version":2,"app_version":env!("CARGO_PKG_VERSION"),"model":d.model,"transpiration_model":report.transpiration.as_ref().map(|w|&w.model),"source":SOURCE,"source_snapshot_sha256":SOURCE_SHA256,"sample_count":report.rows.len(),"csv_missing_values":"empty cells mean unavailable, not zero","model_descriptor":crate::research::PhysicsModelRegistry::get("greenhouse"),"context_state_fingerprint":context_fingerprint,"observed_habitat":observed,"context_semantics":"Exact current observation only; does not maintain crop boundary inputs, allocate power or credit any inventory","uncertainty":null,"limitations":LIMITS}),
        )
    })();
    result.map_err(|e| format!("Greenhouse export incomplete: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transpiration_matches_independent_source_equations_and_fine_integral() {
        for regime in Regime::ALL {
            let f = Definition {
                regime,
                ..Default::default()
            }
            .calculate()
            .unwrap();
            let (p, c, a) = regime.values();
            let q = f.quantum_yield_mol_carbon_per_mol_absorbed_photons;
            // Independently transcribed BVAD 4-23–29; ordinary math is confined
            // to the reference check. This is numerical, not empirical validation.
            let sat = 0.611_f64 * (17.4_f64 * 23.0 / 262.0).exp();
            let deficit = sat * 0.3;
            let rate = |t: f64| {
                let gross = a * (t / 32.0).powf(2.5) * q * p;
                let light_net = (8.0 + 16.0 * 0.625) / 24.0 * gross;
                let gs = (1.717 * 23.0 - 19.96 - 10.54 * deficit) * light_net / c;
                3600.0 * 16.0 * 18.015 / 998.23 * (gs.recip() + 0.4).recip() * deficit / 101.325
                    * 20.0
            };
            let steps = 100_000;
            let dt = 30.0 / steps as f64;
            let integral = (0..steps)
                .map(|i| rate((i as f64 + 0.5) * dt) * dt)
                .sum::<f64>();
            let w = f.rows[30].transpiration.as_ref().unwrap();
            assert!((w.transpired_water_l_day - rate(30.0)).abs() < 1e-11);
            assert!((w.transpired_water_l - integral).abs() / integral < 1e-9);
            let metadata = f.transpiration.as_ref().unwrap();
            assert!((metadata.saturated_vapor_pressure_kpa - sat).abs() < 1e-13);
            assert!(metadata.harvest_resolution_difference_l < 1e-8);
            assert_eq!(metadata.predictive_uncertainty_l, None);
            assert_eq!(
                f.rows[0].transpiration.as_ref().unwrap().transpired_water_l,
                0.0
            );
            for pair in f.rows.windows(2) {
                assert!(
                    pair[1].transpiration.as_ref().unwrap().transpired_water_l
                        > pair[0].transpiration.as_ref().unwrap().transpired_water_l
                );
            }
            assert!((w.transpired_vapor_kg - w.transpired_water_l * 0.99823).abs() < 1e-12);
            assert!(
                (w.light_cycle_net_carbon_umol_m2_s / w.gross_carbon_umol_m2_s - 0.75).abs()
                    < 1e-14
            );
        }
    }
    #[test]
    fn water_scales_with_area_not_fixture_and_saturation_is_not_a_crop_guarantee() {
        let d = Definition::default();
        let f = d.calculate().unwrap();
        let water = |f: &Forecast| {
            f.rows[30]
                .transpiration
                .as_ref()
                .unwrap()
                .transpired_water_l
        };
        let twice = Definition {
            area_m2: 40.0,
            ..d.clone()
        }
        .calculate()
        .unwrap();
        assert_eq!(water(&twice), water(&f) * 2.0);
        let efficient = Definition {
            fixture_efficacy_umol_j: 5.0,
            ..d.clone()
        }
        .calculate()
        .unwrap();
        assert_eq!(efficient.rows[30].transpiration, f.rows[30].transpiration);
        let saturated = Definition {
            transpiration: Some(WaterAssumptions {
                relative_humidity: 1.0,
                ..Default::default()
            }),
            ..d.clone()
        }
        .calculate()
        .unwrap();
        assert_eq!(water(&saturated), 0.0);
        assert_eq!(
            saturated.rows[30].edible_dry_biomass_kg,
            f.rows[30].edible_dry_biomass_kg
        );
        let drier = Definition {
            transpiration: Some(WaterAssumptions {
                relative_humidity: 0.4,
                ..Default::default()
            }),
            ..d
        }
        .calculate()
        .unwrap();
        assert!(water(&drier).is_finite() && water(&drier) > 0.0);
        // The polynomial is not a monotone stress law; don't assert more dry air
        // always increases transpiration or infer biology from the mathematical fit.
    }
    #[test]
    fn unsupported_water_domains_and_versions_are_refused_without_clipping() {
        for rh in [f64::NAN, f64::INFINITY, -0.1, 1.01, 0.0, 0.2] {
            let d = Definition {
                transpiration: Some(WaterAssumptions {
                    relative_humidity: rh,
                    ..Default::default()
                }),
                ..Default::default()
            };
            assert!(d.calculate().is_err());
        }
        let d = Definition {
            transpiration: Some(WaterAssumptions {
                model: "future/2".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(d.calculate().is_err());
        let d = Definition {
            model: LEGACY_MODEL.into(),
            ..Default::default()
        };
        assert!(d.calculate().is_err());
        let bad = serde_json::json!({"model":MODEL,"regime":"p200_c660","area_m2":20,"fixture_efficacy_umol_j":2.5,"transpiration":{"model":WATER_MODEL,"relative_humidity":0.7,"pressure_kpa":50}});
        assert!(serde_json::from_value::<Definition>(bad).is_err());
    }
    #[test]
    fn legacy_definitions_preserve_biomass_and_unavailable_water_exports() {
        let mut old = serde_json::to_value(Definition::default()).unwrap();
        old["model"] = LEGACY_MODEL.into();
        old.as_object_mut().unwrap().remove("transpiration");
        let d: Definition = serde_json::from_value(old).unwrap();
        let f = d.calculate().unwrap();
        assert!(f.transpiration.is_none() && f.rows.iter().all(|r| r.transpiration.is_none()));
        let mut modern = Definition::default().calculate().unwrap();
        for r in &mut modern.rows {
            r.transpiration = None;
        }
        assert_eq!(f.rows, modern.rows);
        let root =
            std::env::temp_dir().join(format!("cosmogon-water-legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        export(&d, None, &root).unwrap();
        let csv = std::fs::read_to_string(root.join("samples.csv")).unwrap();
        assert!(csv.lines().all(|l| l.split(',').count() == 21));
        for line in csv.lines().skip(1) {
            assert!(line.split(',').skip(13).all(str::is_empty));
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["model"], LEGACY_MODEL);
        assert!(manifest["transpiration_model"].is_null());
        assert_eq!(Definition::read(&root.join("definition.json")).unwrap(), d);
        // Old serialized forecasts deserialize without any fabricated water rows.
        let mut old_report = serde_json::to_value(&f).unwrap();
        old_report["schema_version"] = 1.into();
        let parsed: Forecast = serde_json::from_value(old_report).unwrap();
        assert!(parsed.transpiration.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn greenhouse_fifo_is_refused_before_open() {
        let path = std::env::temp_dir().join(format!("cosmogon-water-pipe-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Definition::read(&path)
                .unwrap_err()
                .contains("regular file")
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn published_low_light_cases_use_corrected_canopy_and_consistent_carbon() {
        for regime in Regime::ALL {
            let d = Definition {
                regime,
                ..Default::default()
            };
            let f = d.calculate().unwrap();
            let (_, _, a) = regime.values();
            assert_eq!(f.rows.len(), 31);
            assert_eq!(f.rows[0].total_dry_biomass_kg, 0.0);
            assert_eq!(f.rows[1].edible_dry_biomass_kg, 0.0);
            let last = &f.rows[30];
            assert!(last.canopy_absorbed_fraction < a && a < 0.93);
            for pair in f.rows.windows(2) {
                assert!(pair[1].total_dry_biomass_kg > pair[0].total_dry_biomass_kg);
            }
            for r in &f.rows {
                assert!(
                    (r.total_dry_biomass_kg - r.edible_dry_biomass_kg - r.inedible_dry_biomass_kg)
                        .abs()
                        < 1e-10
                );
                assert!(
                    (r.total_dry_biomass_kg * BCF / (CARBON_G_MOL / 1000.0) - r.net_carbon_mol)
                        .abs()
                        < 1e-10
                );
                assert!(
                    (r.net_oxygen_kg / (OPF * 0.0319988) - r.net_co2_kg / 0.0440095).abs() < 1e-10
                );
            }
            let twice = Definition {
                area_m2: 40.0,
                ..d.clone()
            }
            .calculate()
            .unwrap();
            assert_eq!(
                twice.rows[30].total_dry_biomass_kg,
                last.total_dry_biomass_kg * 2.0
            );
            assert_eq!(twice.lamps_on_power_w, f.lamps_on_power_w * 2.0);
            let efficient = Definition {
                fixture_efficacy_umol_j: 5.0,
                ..d
            }
            .calculate()
            .unwrap();
            assert_eq!(efficient.rows[30].net_oxygen_kg, last.net_oxygen_kg);
            assert_eq!(
                efficient.rows[30].lighting_electric_j,
                last.lighting_electric_j / 2.0
            );
        }
    }
    #[test]
    fn analytic_integral_matches_independent_fine_quadrature() {
        let f = Definition::default().calculate().unwrap();
        let comparison = compare(&f.definition).unwrap();
        assert_eq!(comparison.len(), 5);
        assert_eq!(comparison[1].harvest, f.rows[30]);
        // Independent integration of the source daily carbon equation. Ordinary powf
        // is confined to this numerical check; authoritative outputs use DMath.
        let mut integral = 0.0;
        let steps = 100_000;
        let dt = 30.0 / steps as f64;
        for i in 0..steps {
            let t = (i as f64 + 0.5) * dt;
            let absorption = 0.35 * (t / 32.0).powf(2.5);
            integral += 0.0036
                * 16.0
                * 0.625
                * absorption
                * f.quantum_yield_mol_carbon_per_mol_absorbed_photons
                * 200.0
                * 20.0
                * dt;
        }
        assert!((integral - f.rows[30].net_carbon_mol).abs() / integral < 1e-9);
        assert_eq!(f.lamps_on_power_w, 1600.0);
        assert_eq!(f.daily_light_integral_mol_m2_day, 11.52);
        assert_eq!(f.rows[30].lighting_electric_j, 2_764_800_000.0);
        assert_eq!(
            f.quantum_yield_mol_carbon_per_mol_absorbed_photons,
            0.06709453693600001
        );
    }
    #[test]
    fn definitions_and_exports_are_bounded_versioned_and_protected() {
        let mut d = Definition {
            area_m2: f64::NAN,
            ..Default::default()
        };
        assert!(d.calculate().is_err());
        d.area_m2 = 20.0;
        d.model = "future/2".into();
        assert!(d.calculate().is_err());
        assert!(serde_json::from_str::<Definition>(r#"{"model":"cosmogon-lettuce-mec/1","regime":"unknown","area_m2":20,"fixture_efficacy_umol_j":2.5}"#).is_err());
        let root = std::env::temp_dir().join(format!("cosmogon-crop-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let p = root.join("out");
        d = Definition::default();
        export(&d, None, &p).unwrap();
        let original = std::fs::read(p.join("samples.csv")).unwrap();
        assert!(export(&d, None, &p).is_err());
        assert_eq!(original, std::fs::read(p.join("samples.csv")).unwrap());
        assert_eq!(Definition::read(&p.join("definition.json")).unwrap(), d);
        let f: Forecast =
            serde_json::from_slice(&std::fs::read(p.join("forecast.json")).unwrap()).unwrap();
        assert_eq!(f, d.calculate().unwrap());
        assert_eq!(String::from_utf8(original).unwrap().lines().count(), 32);
        let big = root.join("big.json");
        std::fs::write(&big, vec![b' '; 65_537]).unwrap();
        assert!(Definition::read(&big).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
