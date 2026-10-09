//! Finite inputs for an unchanged maintained-regime crop forecast. This is a
//! resource feasibility projection, never a crop-stress solver or stock transfer.
use crate::{Universe, greenhouse};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

pub const MODEL: &str = "cosmogon-crop-supply-study/1";
pub const RECIPE: &str = "volk-cullingford-1992-lettuce/1";
pub const SOURCE: &str = "https://ntrs.nasa.gov/citations/19930004806";
pub const SOURCE_SHA256: &str = "98498e499f097489c294032611e2532ef70765d11c9b32e41c912945810fdfc5";
pub const LIMITS: &str = "Maintained-regime feasibility at 31 original daily ages, not an installed greenhouse or crop-stress response. Mixing 1992 constant-composition lettuce recipe ratios with separate NASA MEC dry growth is an engineering assumption, not a validated combined model. Inedible composition includes author assumptions; rounded recipe has a retained -0.01 kg/kg inedible mass residual. HNO3-equivalent nutrients are a bookkeeping basis, not a fertilizer/acid tank or complete chemical recipe. Liquid need includes metabolic incorporation plus modeled light-period vapor only; no dark transpiration, irrigation losses or collection/recycling credit. Total liquid demand is unavailable without transpiration. Electricity covers lamps only, excluding pumps, cooling and humidity regulation. Budget support is a necessary input condition, not proof of crop growth, harvest, edible inventory, nutrition, oxygen delivery or habitat survival. No uncertainty or field/hardware calibration. Original potential values beyond supply support remain forecasts under maintained inputs, not actual depleted-crop yields.";
pub const RESOURCES: [&str; 4] = [
    "liquid_water",
    "carbon_dioxide",
    "nutrient_hno3_equivalent",
    "lighting_energy",
];
pub const UNITS: [&str; 4] = ["kg", "kg", "kg", "J"];
pub const LABELS: [&str; 4] = [
    "Liquid water",
    "CO₂",
    "HNO₃-equivalent nutrients",
    "Lighting electricity",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Supplies {
    pub liquid_water_kg: f64,
    pub carbon_dioxide_kg: f64,
    pub nutrient_hno3_equivalent_kg: f64,
    pub lighting_energy_j: f64,
}
impl Default for Supplies {
    fn default() -> Self {
        Self {
            liquid_water_kg: 500.0,
            carbon_dioxide_kg: 10.0,
            nutrient_hno3_equivalent_kg: 1.0,
            lighting_energy_j: 3.0e9,
        }
    }
}
impl Supplies {
    pub fn values(&self) -> [f64; 4] {
        [
            self.liquid_water_kg,
            self.carbon_dioxide_kg,
            self.nutrient_hno3_equivalent_kg,
            self.lighting_energy_j,
        ]
    }
    pub fn validate(&self) -> Result<(), String> {
        for (i, v) in self.values().into_iter().enumerate() {
            let max = if i == 3 { 1.0e16 } else { 1.0e9 };
            if !v.is_finite() || !(0.0..=max).contains(&v) {
                return Err(format!(
                    "{} budget must be finite, nonnegative and at most {max} {}",
                    LABELS[i], UNITS[i]
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub format: String,
    pub schema_version: u32,
    pub model: String,
    pub recipe: String,
    pub crop: greenhouse::Definition,
    pub supplies: Supplies,
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            format: "cosmogon-crop-supply-definition".into(),
            schema_version: 1,
            model: MODEL.into(),
            recipe: RECIPE.into(),
            crop: Default::default(),
            supplies: Default::default(),
        }
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "cosmogon-crop-supply-definition"
            || self.schema_version != 1
            || self.model != MODEL
            || self.recipe != RECIPE
        {
            return Err("Unsupported crop supply definition, model or recipe".into());
        }
        self.crop.validate()?;
        self.supplies.validate()
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let d: Self = serde_json::from_slice(&bounded(path, 65_536)?).map_err(|e| e.to_string())?;
        d.validate()?;
        Ok(d)
    }
    pub fn calculate(&self) -> Result<Report, String> {
        self.validate()?;
        // Preserve the exact original implementation and its operation order.
        let forecast = self.crop.calculate()?;
        let budgets = self.supplies.values();
        let rows = forecast
            .rows
            .iter()
            .map(|r| {
                let e = r.edible_dry_biomass_kg;
                let i = r.inedible_dry_biomass_kg;
                let metabolic_water_kg = 0.57 * e + 0.55 * i;
                let recipe_co2_kg = 1.82 * e + 1.68 * i;
                let nutrient_hno3_equivalent_kg = 0.20 * e + 0.08 * i;
                let recipe_oxygen_kg = 1.59 * e + 1.32 * i;
                let transpired_vapor_kg = r.transpiration.as_ref().map(|w| w.transpired_vapor_kg);
                let demand = [
                    transpired_vapor_kg.map(|w| w + metabolic_water_kg),
                    Some(recipe_co2_kg),
                    Some(nutrient_hno3_equivalent_kg),
                    Some(r.lighting_electric_j),
                ];
                let margin = std::array::from_fn(|k| demand[k].map(|v| budgets[k] - v));
                let support = margin.map(|v| match v {
                    None => Support::Unavailable,
                    Some(v) if v < 0.0 => Support::Exceeded,
                    _ => Support::Supported,
                });
                Row {
                    age_days_after_emergence: r.age_days_after_emergence,
                    edible_dry_biomass_kg: e,
                    inedible_dry_biomass_kg: i,
                    metabolic_water_kg,
                    transpired_vapor_kg,
                    recipe_oxygen_kg,
                    recipe_mass_residual_kg: recipe_co2_kg
                        + metabolic_water_kg
                        + nutrient_hno3_equivalent_kg
                        - e
                        - i
                        - recipe_oxygen_kg,
                    demand,
                    margin,
                    support,
                    overall_support: overall(support),
                }
            })
            .collect::<Vec<_>>();
        let constraints = std::array::from_fn(|k| {
            let last = &rows[30];
            let first = rows.iter().position(|r| r.support[k] == Support::Exceeded);
            Constraint {
                resource: RESOURCES[k].into(),
                unit: UNITS[k].into(),
                budget: budgets[k],
                horizon_demand: last.demand[k],
                horizon_support: last.support[k],
                last_supported_day: if last.demand[k].is_none() {
                    None
                } else {
                    Some(first.map_or(30, |n| n.saturating_sub(1)) as u32)
                },
                first_unsupported_day: first.map(|n| n as u32),
                exhaustion_bracket_days: first.map(|n| [n.saturating_sub(1) as f64, n as f64]),
            }
        });
        let first_known_unsupported_day = constraints
            .iter()
            .filter_map(|c| c.first_unsupported_day)
            .min();
        let complete_demand_coverage = constraints.iter().all(|c| c.horizon_demand.is_some());
        let supported_prefix_last_day = complete_demand_coverage
            .then(|| first_known_unsupported_day.map_or(30, |n| n.saturating_sub(1)));
        Ok(Report {
            format: "cosmogon-crop-supply-report".into(),
            schema_version: 1,
            definition: self.clone(),
            recipe_source: recipe_source(),
            forecast,
            rows,
            constraints,
            complete_demand_coverage,
            supported_prefix_last_day,
            first_known_unsupported_day,
            scope: LIMITS.into(),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Support {
    Supported,
    Exceeded,
    Unavailable,
}
impl Support {
    pub fn key(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Exceeded => "exceeded",
            Self::Unavailable => "unavailable",
        }
    }
}
fn overall(s: [Support; 4]) -> Support {
    if s.contains(&Support::Exceeded) {
        Support::Exceeded
    } else if s.contains(&Support::Unavailable) {
        Support::Unavailable
    } else {
        Support::Supported
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub age_days_after_emergence: f64,
    pub edible_dry_biomass_kg: f64,
    pub inedible_dry_biomass_kg: f64,
    pub metabolic_water_kg: f64,
    pub transpired_vapor_kg: Option<f64>,
    pub recipe_oxygen_kg: f64,
    pub recipe_mass_residual_kg: f64,
    /// Order is RESOURCES. Missing total liquid remains unavailable, even at age zero.
    pub demand: [Option<f64>; 4],
    pub margin: [Option<f64>; 4],
    pub support: [Support; 4],
    pub overall_support: Support,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub resource: String,
    pub unit: String,
    pub budget: f64,
    pub horizon_demand: Option<f64>,
    pub horizon_support: Support,
    pub last_supported_day: Option<u32>,
    pub first_unsupported_day: Option<u32>,
    /// Daily bracket only. No exact crossing time or interpolation implied.
    pub exhaustion_bracket_days: Option<[f64; 2]>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub format: String,
    pub schema_version: u32,
    pub definition: Definition,
    pub recipe_source: serde_json::Value,
    pub forecast: greenhouse::Forecast,
    pub rows: Vec<Row>,
    pub constraints: [Constraint; 4],
    pub complete_demand_coverage: bool,
    pub supported_prefix_last_day: Option<u32>,
    pub first_known_unsupported_day: Option<u32>,
    pub scope: String,
}
pub fn recipe_source() -> serde_json::Value {
    serde_json::json!({"model":RECIPE,"url":SOURCE,"source_snapshot_sha256":SOURCE_SHA256,"table":"Volk and Cullingford 1992, printed p528 Table2","basis":"kg per kg separate edible/inedible dry biomass production","edible":{"co2":1.82,"metabolic_water":0.57,"hno3_equivalent":0.20,"oxygen":1.59},"inedible":{"co2":1.68,"metabolic_water":0.55,"hno3_equivalent":0.08,"oxygen":1.32},"inedible_rounded_mass_residual_kg_per_kg":-0.01,"provenance":"Published rounded ratios; some inedible composition is author assumed. Combination with MEC dry production is an explicit engineering scenario.","predictive_uncertainty":null})
}
impl Report {
    pub fn csv(&self) -> String {
        let mut csv="age_days_after_emergence,edible_dry_biomass_kg,inedible_dry_biomass_kg,metabolic_water_kg,transpired_vapor_kg,recipe_oxygen_kg,recipe_mass_residual_kg".to_string();
        for (k, unit) in RESOURCES.iter().zip(UNITS) {
            csv += &format!(",{k}_demand_{unit},{k}_margin_{unit},{k}_support");
        }
        csv += ",overall_support\n";
        let number = |v: Option<f64>| v.map(|v| format!("{v:.17e}")).unwrap_or_default();
        for r in &self.rows {
            csv += &format!(
                "{:.17e},{:.17e},{:.17e},{:.17e},{},{:.17e},{:.17e}",
                r.age_days_after_emergence,
                r.edible_dry_biomass_kg,
                r.inedible_dry_biomass_kg,
                r.metabolic_water_kg,
                number(r.transpired_vapor_kg),
                r.recipe_oxygen_kg,
                r.recipe_mass_residual_kg
            );
            for k in 0..4 {
                csv += &format!(
                    ",{},{},{}",
                    number(r.demand[k]),
                    number(r.margin[k]),
                    r.support[k].key()
                );
            }
            csv += &format!(",{}\n", r.overall_support.key());
        }
        csv
    }
}
fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > limit {
        return Err(format!(
            "Crop study file must be regular and at most {limit} bytes"
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Crop study file exceeds limit".into());
    }
    Ok(bytes)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub schema_version: u32,
    pub model: String,
    pub recipe: String,
    pub source_snapshot_sha256: String,
    pub crop_source_snapshot_sha256: String,
    pub rows: usize,
    pub context_state_fingerprint: Option<String>,
    pub habitat_id: Option<u64>,
    pub observed_habitat: Option<serde_json::Value>,
    pub context_semantics: String,
    pub missing_values: String,
}
impl Manifest {
    fn new(context: Option<(&Universe, Option<u64>)>) -> Result<Self, String> {
        let observed_habitat = match context {
            Some((u, Some(id))) => Some(crate::habitat::sample(u, id)?.habitat.report()),
            _ => None,
        };
        Ok(Self {
            format: "cosmogon-crop-supply-study".into(),
            schema_version: 1,
            model: MODEL.into(),
            recipe: RECIPE.into(),
            source_snapshot_sha256: SOURCE_SHA256.into(),
            crop_source_snapshot_sha256: greenhouse::SOURCE_SHA256.into(),
            rows: 31,
            context_state_fingerprint: context
                .map(|(u, _)| crate::research::state_fingerprint(u))
                .transpose()?,
            habitat_id: context.and_then(|(_, id)| id),
            observed_habitat,
            context_semantics:
                "Frozen observation only; never supplies, future allocations or crop inventories"
                    .into(),
            missing_values: "Empty CSV cells/null JSON mean unavailable, never zero".into(),
        })
    }
}
/// A loaded bundle carries its own frozen context, independent of the live app.
#[derive(Clone, Debug)]
pub struct Bundle {
    pub report: Report,
    pub manifest: Manifest,
    pub context: Option<Universe>,
}
impl Bundle {
    pub fn calculate(
        d: &Definition,
        context: Option<(&Universe, Option<u64>)>,
    ) -> Result<Self, String> {
        Ok(Self {
            report: d.calculate()?,
            manifest: Manifest::new(context)?,
            context: context.map(|(u, _)| u.clone()),
        })
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        if !path.is_dir() {
            return Self::calculate(&Definition::read(path)?, None);
        }
        let d = Definition::read(&path.join("definition.json"))?;
        let report: Report =
            serde_json::from_slice(&bounded(&path.join("report.json"), 2_000_000)?)
                .map_err(|e| e.to_string())?;
        if d.calculate()? != report {
            return Err(
                "Stored crop supply report differs from the pinned definition/source calculation"
                    .into(),
            );
        }
        let manifest: Manifest =
            serde_json::from_slice(&bounded(&path.join("manifest.json"), 65_536)?)
                .map_err(|e| e.to_string())?;
        let context = if manifest.context_state_fingerprint.is_some() {
            Some(
                crate::save::from_json(
                    std::str::from_utf8(&bounded(&path.join("context.cosmo"), 64_000_000)?)
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?,
            )
        } else {
            if path.join("context.cosmo").exists() {
                return Err("Unclaimed crop context file".into());
            }
            None
        };
        if Manifest::new(context.as_ref().map(|u| (u, manifest.habitat_id)))? != manifest {
            return Err("Crop study manifest/source/context mismatch".into());
        }
        if bounded(&path.join("samples.csv"), 1_000_000)? != report.csv().as_bytes() {
            return Err("Crop study CSV differs from exact report values".into());
        }
        Ok(Self {
            report,
            manifest,
            context,
        })
    }
    pub fn export(&self, path: &Path) -> Result<(), String> {
        // Revalidate all public bundle fields before producing a reproducibility claim.
        if self.report.definition.calculate()? != self.report
            || Manifest::new(self.context.as_ref().map(|u| (u, self.manifest.habitat_id)))?
                != self.manifest
        {
            return Err("Crop study bundle no longer matches its exact inputs".into());
        }
        let files = [
            (
                "definition.json",
                serde_json::to_vec_pretty(&self.report.definition),
            ),
            ("report.json", serde_json::to_vec_pretty(&self.report)),
            ("manifest.json", serde_json::to_vec_pretty(&self.manifest)),
        ]
        .into_iter()
        .map(|(n, v)| v.map(|v| (n, v)).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
        let context = self
            .context
            .as_ref()
            .map(|u| {
                crate::save::to_json(u, "Crop supply study context").map_err(|e| e.to_string())
            })
            .transpose()?;
        std::fs::create_dir(path).map_err(|e| format!("Choose a new crop study folder: {e}"))?;
        let write = |name: &str, bytes: &[u8]| -> Result<(), String> {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .map_err(|e| e.to_string())?
                .write_all(bytes)
                .map_err(|e| e.to_string())
        };
        let result = (|| {
            for (n, v) in files {
                write(n, &v)?;
            }
            write("samples.csv", self.report.csv().as_bytes())?;
            if let Some(c) = context {
                write("context.cosmo", c.as_bytes())?;
            }
            Ok(())
        })();
        result.map_err(|e: String| format!("Crop supply export incomplete: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plentiful() -> Definition {
        Definition {
            supplies: Supplies {
                liquid_water_kg: 1e9,
                carbon_dioxide_kg: 1e9,
                nutrient_hno3_equivalent_kg: 1e9,
                lighting_energy_j: 1e16,
            },
            ..Default::default()
        }
    }
    fn set(d: &mut Definition, k: usize, value: f64) {
        match k {
            0 => d.supplies.liquid_water_kg = value,
            1 => d.supplies.carbon_dioxide_kg = value,
            2 => d.supplies.nutrient_hno3_equivalent_kg = value,
            _ => d.supplies.lighting_energy_j = value,
        }
    }
    fn temp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "cosmogon-crop-supplies-{name}-{}",
            std::process::id()
        ))
    }
    #[test]
    fn recipe_demands_keep_original_forecast_and_independent_mass_residual() {
        for regime in greenhouse::Regime::ALL {
            let mut d = plentiful();
            d.crop.regime = regime;
            let original = d.crop.calculate().unwrap();
            let r = d.calculate().unwrap();
            assert_eq!(original, r.forecast);
            assert_eq!(r, d.calculate().unwrap());
            assert_eq!(r.supported_prefix_last_day, Some(30));
            for (x, g) in r.rows.iter().zip(&original.rows) {
                let e = g.edible_dry_biomass_kg;
                let i = g.total_dry_biomass_kg - e;
                assert_eq!(x.demand[1], Some(e * 1.82 + i * 1.68));
                assert_eq!(x.demand[2], Some(e * 0.20 + i * 0.08));
                assert_eq!(x.metabolic_water_kg, e * 0.57 + i * 0.55);
                assert_eq!(x.recipe_oxygen_kg, e * 1.59 + i * 1.32);
                assert_eq!(
                    x.demand[0],
                    Some(
                        x.metabolic_water_kg
                            + g.transpiration.as_ref().unwrap().transpired_vapor_kg
                    )
                );
                assert_eq!(
                    x.demand[3],
                    Some(original.lamps_on_power_w * 16.0 * 3600.0 * g.age_days_after_emergence)
                );
                assert!((x.recipe_mass_residual_kg + i * 0.01).abs() < 1e-14);
            }
            let zero = &r.rows[0];
            assert_eq!(zero.demand, [Some(0.0); 4]);
            assert_eq!(zero.overall_support, Support::Supported);
            // The source-based recipe and MEC gas projections are deliberately distinct.
            assert_ne!(r.rows[30].recipe_oxygen_kg, original.rows[30].net_oxygen_kg);
            assert!(r.recipe_source["predictive_uncertainty"].is_null());
        }
    }
    #[test]
    fn every_finite_budget_has_exact_daily_brackets_and_ties() {
        let full = plentiful().calculate().unwrap();
        for k in 0..4 {
            let mut d = plentiful();
            set(&mut d, k, full.rows[12].demand[k].unwrap());
            let r = d.calculate().unwrap();
            let c = &r.constraints[k];
            assert_eq!(r.rows[12].margin[k], Some(0.0));
            assert_eq!(r.rows[12].support[k], Support::Supported);
            assert_eq!(r.rows[13].support[k], Support::Exceeded);
            assert_eq!(c.first_unsupported_day, Some(13));
            assert_eq!(c.exhaustion_bracket_days, Some([12.0, 13.0]));
            assert_eq!(r.supported_prefix_last_day, Some(12));
            set(&mut d, k, 0.0);
            let r = d.calculate().unwrap();
            assert_eq!(r.constraints[k].exhaustion_bracket_days, Some([0.0, 1.0]));
            assert_eq!(r.rows[0].support[k], Support::Supported);
            set(&mut d, k, full.rows[30].demand[k].unwrap());
            assert_eq!(
                d.calculate().unwrap().constraints[k].first_unsupported_day,
                None
            );
        }
        let mut d = plentiful();
        set(&mut d, 0, full.rows[10].demand[0].unwrap());
        set(&mut d, 2, full.rows[10].demand[2].unwrap());
        let r = d.calculate().unwrap();
        assert_eq!(r.first_known_unsupported_day, Some(11));
        assert_eq!(
            r.constraints[0].first_unsupported_day,
            r.constraints[2].first_unsupported_day
        );
        // Retain full potential after support ends, never invent a clipped/stressed crop.
        assert_eq!(r.forecast, full.forecast);
        assert_eq!(
            r.rows[30].edible_dry_biomass_kg,
            full.rows[30].edible_dry_biomass_kg
        );
    }
    #[test]
    fn legacy_missing_water_is_unknown_and_saturation_still_needs_metabolic_water() {
        let mut d = plentiful();
        d.crop.model = greenhouse::LEGACY_MODEL.into();
        d.crop.transpiration = None;
        let r = d.calculate().unwrap();
        assert!(!r.complete_demand_coverage);
        assert_eq!(r.supported_prefix_last_day, None);
        for x in &r.rows {
            assert_eq!(x.demand[0], None);
            assert_eq!(x.margin[0], None);
            assert_eq!(x.support[0], Support::Unavailable);
            assert_eq!(x.overall_support, Support::Unavailable);
        }
        d.supplies.lighting_energy_j = 0.0;
        let r = d.calculate().unwrap();
        assert_eq!(r.rows[30].overall_support, Support::Exceeded);
        assert_eq!(r.first_known_unsupported_day, Some(1));
        assert_eq!(r.supported_prefix_last_day, None);
        d.crop.model = greenhouse::MODEL.into();
        d.crop.transpiration = Some(greenhouse::WaterAssumptions {
            relative_humidity: 1.0,
            ..Default::default()
        });
        d.supplies = Supplies::default();
        let r = d.calculate().unwrap();
        assert_eq!(r.rows[30].transpired_vapor_kg, Some(0.0));
        assert!(r.rows[30].metabolic_water_kg > 0.0);
        assert_eq!(r.rows[30].demand[0], Some(r.rows[30].metabolic_water_kg));
    }
    #[test]
    fn area_fixture_scaling_and_source_bounds_remain_explicit() {
        let mut d = plentiful();
        let a = d.calculate().unwrap();
        d.crop.area_m2 *= 2.0;
        let b = d.calculate().unwrap();
        for (x, y) in a.rows.iter().zip(&b.rows) {
            for k in 0..4 {
                assert_eq!(y.demand[k], x.demand[k].map(|v| v * 2.0));
            }
        }
        d.crop.fixture_efficacy_umol_j *= 2.0;
        let c = d.calculate().unwrap();
        assert_eq!(c.rows[30].demand[3], a.rows[30].demand[3]);
        assert_eq!(&c.rows[30].demand[..3], &b.rows[30].demand[..3]);
        for k in 0..4 {
            for value in [
                -1.0,
                f64::NAN,
                f64::INFINITY,
                if k == 3 { 1.1e16 } else { 1.1e9 },
            ] {
                let mut bad = plentiful();
                set(&mut bad, k, value);
                assert!(bad.calculate().is_err());
            }
        }
        for field in ["model", "recipe", "format"] {
            let mut v = serde_json::to_value(plentiful()).unwrap();
            v[field] = "unknown".into();
            assert!(
                serde_json::from_value::<Definition>(v)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        let mut v = serde_json::to_value(plentiful()).unwrap();
        v["invented_recovery_credit"] = 0.5.into();
        assert!(serde_json::from_value::<Definition>(v).is_err());
    }
    #[test]
    fn protected_bundle_keeps_exact_values_and_frozen_context_without_mutation() {
        let path = temp("bundle");
        let _ = std::fs::remove_dir_all(&path);
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let fingerprint = crate::research::state_fingerprint(&u).unwrap();
        let b = Bundle::calculate(&plentiful(), Some((&u, None))).unwrap();
        b.export(&path).unwrap();
        let original = std::fs::read(path.join("samples.csv")).unwrap();
        assert!(b.export(&path).is_err());
        assert_eq!(std::fs::read(path.join("samples.csv")).unwrap(), original);
        let opened = Bundle::read(&path).unwrap();
        assert_eq!(opened.report, b.report);
        assert_eq!(opened.manifest, b.manifest);
        assert_eq!(
            crate::research::state_fingerprint(opened.context.as_ref().unwrap()).unwrap(),
            fingerprint
        );
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), fingerprint);
        let csv = b.report.csv();
        for (line, r) in csv.lines().skip(1).zip(&b.report.rows) {
            let cells = line.split(',').collect::<Vec<_>>();
            for k in 0..4 {
                assert_eq!(
                    cells[7 + 3 * k].parse::<f64>().unwrap().to_bits(),
                    r.demand[k].unwrap().to_bits()
                );
                assert_eq!(
                    cells[8 + 3 * k].parse::<f64>().unwrap().to_bits(),
                    r.margin[k].unwrap().to_bits()
                );
            }
        }
        let invalid = temp("context-invalid");
        let _ = std::fs::remove_dir_all(&invalid);
        assert!(Bundle::calculate(&plentiful(), Some((&u, Some(u64::MAX)))).is_err());
        assert!(!invalid.exists());
        let mut modified = b.clone();
        modified.report.rows[1].demand[0] = Some(1.0);
        assert!(modified.export(&invalid).is_err());
        assert!(!invalid.exists());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn imported_studies_refuse_tampering_and_oversized_inputs() {
        let path = temp("tamper");
        let _ = std::fs::remove_dir_all(&path);
        let b = Bundle::calculate(&plentiful(), None).unwrap();
        b.export(&path).unwrap();
        for file in [
            "definition.json",
            "report.json",
            "manifest.json",
            "samples.csv",
        ] {
            let p = path.join(file);
            let old = std::fs::read(&p).unwrap();
            if file == "samples.csv" {
                std::fs::write(&p, b"altered\n").unwrap();
            } else {
                let mut v: serde_json::Value = serde_json::from_slice(&old).unwrap();
                match file {
                    "definition.json" => v["supplies"]["carbon_dioxide_kg"] = 0.0.into(),
                    "report.json" => v["rows"][0]["demand"][0] = 1.0.into(),
                    _ => v["source_snapshot_sha256"] = "unknown".into(),
                };
                std::fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
            }
            assert!(Bundle::read(&path).is_err(), "{file}");
            std::fs::write(p, old).unwrap();
        }
        std::fs::write(path.join("context.cosmo"), b"unclaimed").unwrap();
        assert!(Bundle::read(&path).is_err());
        std::fs::remove_file(path.join("context.cosmo")).unwrap();
        assert!(Bundle::read(&path).is_ok());
        std::fs::write(path.join("large.json"), vec![b' '; 65_537]).unwrap();
        assert!(Definition::read(&path.join("large.json")).is_err());
        let mut d = plentiful();
        d.crop.transpiration = None;
        d.crop.model = greenhouse::LEGACY_MODEL.into();
        let csv = d.calculate().unwrap().csv();
        assert!(
            csv.lines()
                .skip(1)
                .all(|l| l.split(',').collect::<Vec<_>>()[7..10] == ["", "", "unavailable"])
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}
