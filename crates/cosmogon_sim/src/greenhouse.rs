//! Source-pinned controlled-regime crop planning. This is not an installed greenhouse:
//! yields require sustained boundary inputs and never credit habitat inventories.
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

pub const MODEL: &str = "cosmogon-lettuce-mec/1";
pub const SOURCE: &str = crate::habitat::SOURCE;
pub const SOURCE_SHA256: &str = "b9eab584c2bc88b52f6eb6744e321eb72ca58003e8ac77dc1ebeb034aba11d1e";
pub const LIMITS: &str = "Controlled terrestrial chamber forecast, Waldmann's Green lettuce only. Fixed 23 C light/dark, 16 h light in a 24 h day, 19.2 plants/m2, harvest at 30 days after emergence. Unlimited regulated CO2, water and nutrients; no crop stress, humidity/transpiration, nutrient chemistry, disease, pressure/gravity correction, fresh mass or nutritional adequacy. Fixture efficacy is user assumed; cooling, pumps and dehumidification energy unavailable. No installed greenhouse, habitat stock transfer, future electricity guarantee or independent crop validation. Predictive uncertainty unquantified.";
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
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            regime: Regime::P200C660,
            area_m2: 20.0,
            fixture_efficacy_umol_j: 2.5,
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
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
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
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
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
            });
        }
        Ok(Forecast {
            format:"cosmogon-greenhouse-forecast".into(),
            schema_version:1,
            definition:self.clone(),
            source:SOURCE.into(),
            source_snapshot_sha256:SOURCE_SHA256.into(),
            source_version:"NASA/TP-2015-218570/REV2, February 2022; Eq4-14,4-17–4-21; Tables4-97,4-99,4-102,4-111–4-113,4-123".into(),
            cultivar:"Lettuce — Waldmann's Green".into(),
            conditions:serde_json::json!({"ppf_umol_m2_s":p,"co2_umol_mol_air":c,"canopy_maximum_absorbed_fraction":a,"canopy_maximum_absorption_age_days":32,"light_hours_per_terrestrial_day":16,"cycle_seconds":86400,"light_temperature_k":296.15,"dark_temperature_k":296.15,"planting_density_plants_m2":19.2,"harvest_days_after_emergence":30,"water_and_nutrients":"unlimited controlled boundary assumption"}),
            coefficients:serde_json::json!({"canopy_exponent":N,"carbon_use_efficiency":CUE,"biomass_carbon_fraction":BCF,"oxygen_mol_per_mol_carbon":OPF,"carbon_g_mol":CARBON_G_MOL,"edible_allocation_fraction_after_day_1":0.95,"quantum_yield_terms":["1","PPF","CO2","CO2*PPF^2","CO2^2","CO2^2*PPF^2"],"quantum_yield_coefficients":CQY}),
            data_states:serde_json::json!({"regime_and_coefficients":"published controlled-crop model inputs","area_and_fixture_efficacy":"user assumed","yields_and_lighting":"modelled","water_nutrient_humidity_facility_energy":"unavailable","field_validation":"unavailable"}),
            uncertainty:None,
            limitations:LIMITS.into(),
            canopy_maximum_absorption_age_days:32.0,
            quantum_yield_mol_carbon_per_mol_absorbed_photons:q,
            daily_light_integral_mol_m2_day:p*16.0*3600.0/1e6,
            lamps_on_power_w:lamps,
            daily_lighting_electric_j:daily_electric,
            rows,
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
        let mut csv="age_days_after_emergence,canopy_absorbed_fraction,net_carbon_mol_day,net_co2_kg_day,net_oxygen_kg_day,dry_biomass_kg_day,total_dry_biomass_kg,edible_dry_biomass_kg,inedible_dry_biomass_kg,net_carbon_mol,net_co2_kg,net_oxygen_kg,lighting_electric_j\n".to_string();
        for r in &report.rows {
            csv += &format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
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
        }
        write("samples.csv", csv.as_bytes())?;
        if let Some(json) = &context_json {
            write("context.cosmo", json.as_bytes())?;
        }
        json(
            "manifest.json",
            &serde_json::json!({"format":"cosmogon-greenhouse-study","schema_version":1,"app_version":env!("CARGO_PKG_VERSION"),"model":MODEL,"source":SOURCE,"source_snapshot_sha256":SOURCE_SHA256,"sample_count":report.rows.len(),"model_descriptor":crate::research::PhysicsModelRegistry::get("greenhouse"),"context_state_fingerprint":context_fingerprint,"observed_habitat":observed,"context_semantics":"Exact current observation only; does not maintain crop boundary inputs, allocate power or credit any inventory","uncertainty":null,"limitations":LIMITS}),
        )
    })();
    result.map_err(|e| format!("Greenhouse export incomplete: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
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
