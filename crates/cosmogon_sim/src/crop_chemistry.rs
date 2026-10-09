//! Explicit CHON-only engineering surrogate over unchanged crop source reports.
//! Published rounded recipe values remain retained; conservative O2 is distinct.
use crate::greenhouse_supplies::{self as supply, Bundle};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
pub const MODEL: &str = "cosmogon-crop-chon-surrogate/1";
pub const SOURCE: &str = "https://ciaaw.org/abridged-atomic-weights.htm";
pub const ATOMIC_MODEL: &str = "ciaaw-abridged-2024-chon-nominal/1";
/// C, H, O, N in g/mol; nominal abridged values, not an uncertainty model.
pub const ATOMIC_G_MOL: [f64; 4] = [12.011, 1.0080, 15.999, 14.007];
pub const ELEMENTS: [&str; 4] = ["carbon", "hydrogen", "oxygen", "nitrogen"];
pub const LIMITS: &str = "Engineered CHON-only material surrogate, not measured tissue composition, nutrient solution chemistry, a corrected source measurement or a validated crop model. Published rounded CO2, metabolic-water and HNO3-equivalent inputs are retained. Assuming all input C/H/N enters one kg dry matter and only O2 is an output gas, dry oxygen is inferred by mass closure and product O2 follows oxygen conservation. Ash, P/K/S/trace elements, ions, pH, root exudates, other gases and isotope variation are unavailable. Nominal CIAAW masses are separately pinned and do not change existing habitat/reaction constants. No physical crop installation, resource allocation, food/O2 inventory grant or stressed-yield prediction. Original source forecasts, recipe residual, finite supply support and frozen context remain independent and unchanged. Predictive uncertainty and empirical/hardware calibration remain unavailable.";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    pub name: String,
    pub input_co2_kg_per_kg: f64,
    pub input_metabolic_water_kg_per_kg: f64,
    pub input_hno3_equivalent_kg_per_kg: f64,
    pub published_oxygen_kg_per_kg: f64,
    pub conservative_oxygen_kg_per_kg: f64,
    pub oxygen_difference_kg_per_kg: f64,
    pub inferred_dry_elements_kg_per_kg: [f64; 4],
    pub inferred_formula_atoms_per_carbon: [f64; 4],
    pub input_elements_kg_per_kg: [f64; 4],
    pub output_elements_kg_per_kg: [f64; 4],
    pub element_residual_kg_per_kg: [f64; 4],
    pub mass_residual_kg_per_kg: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub age_days_after_emergence: f64,
    pub edible_dry_biomass_kg: f64,
    pub inedible_dry_biomass_kg: f64,
    pub source_recipe_oxygen_kg: f64,
    pub conservative_oxygen_kg: f64,
    pub oxygen_difference_kg: f64,
    pub source_recipe_mass_residual_kg: f64,
    pub inferred_dry_elements_kg: [f64; 4],
    pub input_elements_kg: [f64; 4],
    pub output_elements_kg: [f64; 4],
    pub element_residual_kg: [f64; 4],
    pub mass_residual_kg: f64,
    pub source_supply_support: supply::Support,
    pub transpired_vapor_kg: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub format: String,
    pub schema_version: u32,
    pub model: String,
    pub atomic_model: String,
    pub atomic_source: serde_json::Value,
    pub atomic_g_mol: [f64; 4],
    pub compound_g_mol: [f64; 4],
    pub pools: [Pool; 2],
    pub rows: Vec<Row>,
    pub original_report: supply::Report,
    pub original_manifest: supply::Manifest,
    pub scope: String,
}
fn pool(name: &str, co2: f64, water: f64, nutrient: f64, published: f64) -> Result<Pool, String> {
    let [c, h, o, n] = ATOMIC_G_MOL;
    let mc = c + 2.0 * o;
    let mw = 2.0 * h + o;
    let mn = h + n + 3.0 * o;
    let input = [
        co2 * c / mc,
        water * 2.0 * h / mw + nutrient * h / mn,
        co2 * 2.0 * o / mc + water * o / mw + nutrient * 3.0 * o / mn,
        nutrient * n / mn,
    ];
    let dry = [
        input[0],
        input[1],
        1.0 - input[0] - input[1] - input[3],
        input[3],
    ];
    let oxygen = input[2] - dry[2];
    if input[0] <= 0.0
        || input
            .iter()
            .chain(dry.iter())
            .chain(std::iter::once(&oxygen))
            .any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err("Rounded inputs do not support a nonnegative CHON-only surrogate".into());
    }
    let output = [dry[0], dry[1], dry[2] + oxygen, dry[3]];
    let atoms = std::array::from_fn(|k| (dry[k] / ATOMIC_G_MOL[k]) / (dry[0] / c));
    Ok(Pool {
        name: name.into(),
        input_co2_kg_per_kg: co2,
        input_metabolic_water_kg_per_kg: water,
        input_hno3_equivalent_kg_per_kg: nutrient,
        published_oxygen_kg_per_kg: published,
        conservative_oxygen_kg_per_kg: oxygen,
        oxygen_difference_kg_per_kg: oxygen - published,
        inferred_dry_elements_kg_per_kg: dry,
        inferred_formula_atoms_per_carbon: atoms,
        input_elements_kg_per_kg: input,
        output_elements_kg_per_kg: output,
        element_residual_kg_per_kg: std::array::from_fn(|k| input[k] - output[k]),
        mass_residual_kg_per_kg: co2 + water + nutrient - 1.0 - oxygen,
    })
}
pub fn calculate(b: &Bundle) -> Result<Report, String> {
    let fresh = Bundle::calculate(
        &b.report.definition,
        b.context.as_ref().map(|u| (u, b.manifest.habitat_id)),
    )?;
    if fresh.report != b.report || fresh.manifest != b.manifest {
        return Err("Crop chemistry requires an exact validated original supply study".into());
    }
    let pools = [
        pool("edible", 1.82, 0.57, 0.20, 1.59)?,
        pool("inedible", 1.68, 0.55, 0.08, 1.32)?,
    ];
    let rows = b
        .report
        .rows
        .iter()
        .map(|r| {
            let e = r.edible_dry_biomass_kg;
            let i = r.inedible_dry_biomass_kg;
            let a = &pools[0];
            let z = &pools[1];
            let blend = |av: [f64; 4], zv: [f64; 4]| std::array::from_fn(|k| av[k] * e + zv[k] * i);
            let dry = blend(
                a.inferred_dry_elements_kg_per_kg,
                z.inferred_dry_elements_kg_per_kg,
            );
            let input = blend(a.input_elements_kg_per_kg, z.input_elements_kg_per_kg);
            let oxygen = a.conservative_oxygen_kg_per_kg * e + z.conservative_oxygen_kg_per_kg * i;
            let output = [dry[0], dry[1], dry[2] + oxygen, dry[3]];
            let incoming = r.demand[1].ok_or("Source CO2 demand is unavailable")?
                + r.metabolic_water_kg
                + r.demand[2].ok_or("Source nutrient demand is unavailable")?;
            Ok(Row {
                age_days_after_emergence: r.age_days_after_emergence,
                edible_dry_biomass_kg: e,
                inedible_dry_biomass_kg: i,
                source_recipe_oxygen_kg: r.recipe_oxygen_kg,
                conservative_oxygen_kg: oxygen,
                oxygen_difference_kg: oxygen - r.recipe_oxygen_kg,
                source_recipe_mass_residual_kg: r.recipe_mass_residual_kg,
                inferred_dry_elements_kg: dry,
                input_elements_kg: input,
                output_elements_kg: output,
                element_residual_kg: std::array::from_fn(|k| input[k] - output[k]),
                mass_residual_kg: incoming - e - i - oxygen,
                source_supply_support: r.overall_support,
                transpired_vapor_kg: r.transpired_vapor_kg,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let [c, h, o, n] = ATOMIC_G_MOL;
    Ok(Report {
        format: "cosmogon-crop-chemistry-report".into(),
        schema_version: 1,
        model: MODEL.into(),
        atomic_model: ATOMIC_MODEL.into(),
        atomic_source: serde_json::json!({"url":SOURCE,"version":"Abridged Standard Atomic Weights 2024, checked 2026-10-09","order":ELEMENTS,"nominal_values_g_mol":ATOMIC_G_MOL,"source_variations":[0.002,0.0002,0.001,0.001],"semantics":"Fixed nominal chemistry basis; listed source atomic-weight variations are not propagated crop predictive uncertainty"}),
        atomic_g_mol: ATOMIC_G_MOL,
        compound_g_mol: [c + 2.0 * o, 2.0 * h + o, h + n + 3.0 * o, 2.0 * o],
        pools,
        rows,
        original_report: b.report.clone(),
        original_manifest: b.manifest.clone(),
        scope: LIMITS.into(),
    })
}
impl Report {
    pub fn csv(&self) -> String {
        let mut csv="age_days_after_emergence,edible_dry_biomass_kg,inedible_dry_biomass_kg,published_recipe_oxygen_kg,conservative_oxygen_kg,oxygen_difference_kg,source_recipe_mass_residual_kg,surrogate_mass_residual_kg,transpired_vapor_kg,source_supply_support".to_string();
        for scope in ["inferred_dry", "input", "output", "residual"] {
            for element in ELEMENTS {
                csv += &format!(",{scope}_{element}_kg");
            }
        }
        csv.push('\n');
        for r in &self.rows {
            csv += &format!(
                "{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{}",
                r.age_days_after_emergence,
                r.edible_dry_biomass_kg,
                r.inedible_dry_biomass_kg,
                r.source_recipe_oxygen_kg,
                r.conservative_oxygen_kg,
                r.oxygen_difference_kg,
                r.source_recipe_mass_residual_kg,
                r.mass_residual_kg,
                r.transpired_vapor_kg
                    .map(|v| format!("{v:.17e}"))
                    .unwrap_or_default(),
                r.source_supply_support.key()
            );
            for array in [
                r.inferred_dry_elements_kg,
                r.input_elements_kg,
                r.output_elements_kg,
                r.element_residual_kg,
            ] {
                for v in array {
                    csv += &format!(",{v:.17e}");
                }
            }
            csv.push('\n');
        }
        csv
    }
}
pub fn export(b: &Bundle, path: &Path) -> Result<(), String> {
    let report = calculate(b)?;
    let json = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::create_dir(path).map_err(|e| format!("Choose a new crop chemistry folder: {e}"))?;
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
        b.export(&path.join("original"))?;
        write("chemistry.json", &json)?;
        write("elements.csv", report.csv().as_bytes())?;
        Ok(())
    })();
    result.map_err(|e: String| format!("Crop chemistry export incomplete: {e}"))
}
pub fn read(path: &Path) -> Result<(Bundle, Report), String> {
    let b = Bundle::read(&path.join("original"))?;
    let expected = calculate(&b)?;
    let bounded = |p: &Path, limit: u64| -> Result<Vec<u8>, String> {
        let m = std::fs::metadata(p).map_err(|e| e.to_string())?;
        if !m.is_file() || m.len() > limit {
            return Err("Crop chemistry file exceeds bounded regular-file size".into());
        }
        let mut bytes = Vec::new();
        std::fs::File::open(p)
            .map_err(|e| e.to_string())?
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > limit {
            return Err("Crop chemistry file exceeds size limit".into());
        }
        Ok(bytes)
    };
    let r: Report = serde_json::from_slice(&bounded(&path.join("chemistry.json"), 4_000_000)?)
        .map_err(|e| e.to_string())?;
    if r != expected || bounded(&path.join("elements.csv"), 1_000_000)? != expected.csv().as_bytes()
    {
        return Err("Crop chemistry source/model/element report mismatch".into());
    }
    Ok((b, r))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Bundle {
        Bundle::calculate(&supply::Definition::default(), None).unwrap()
    }
    fn temp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "cosmogon-crop-chemistry-{name}-{}",
            std::process::id()
        ))
    }
    #[test]
    fn independent_atoms_close_each_pool_and_source_rounding_stays_retained() {
        let b = source();
        let original = b.report.clone();
        let r = calculate(&b).unwrap();
        assert_eq!(r.original_report, original);
        assert_eq!(b.report, original);
        for p in &r.pools {
            let co2_mol = p.input_co2_kg_per_kg * 1000.0 / (12.011 + 2.0 * 15.999);
            let h2o_mol = p.input_metabolic_water_kg_per_kg * 1000.0 / (2.0 * 1.008 + 15.999);
            let hno3_mol =
                p.input_hno3_equivalent_kg_per_kg * 1000.0 / (1.008 + 14.007 + 3.0 * 15.999);
            let o2_mol = p.conservative_oxygen_kg_per_kg * 1000.0 / (2.0 * 15.999);
            let input_atoms = [
                co2_mol,
                2.0 * h2o_mol + hno3_mol,
                2.0 * co2_mol + h2o_mol + 3.0 * hno3_mol,
                hno3_mol,
            ];
            let mut output_atoms: [f64; 4] = std::array::from_fn(|k| {
                p.inferred_dry_elements_kg_per_kg[k] * 1000.0 / ATOMIC_G_MOL[k]
            });
            output_atoms[2] += 2.0 * o2_mol;
            for k in 0..4 {
                assert!((input_atoms[k] - output_atoms[k]).abs() < 5e-14);
                assert!(p.inferred_dry_elements_kg_per_kg[k] > 0.0);
                assert!(p.element_residual_kg_per_kg[k].abs() < 1e-15);
            }
            assert!((p.inferred_dry_elements_kg_per_kg.iter().sum::<f64>() - 1.0).abs() < 1e-15);
            assert!(p.mass_residual_kg_per_kg.abs() < 1e-15);
            assert_eq!(p.inferred_formula_atoms_per_carbon[0], 1.0);
        }
        assert!((r.pools[0].conservative_oxygen_kg_per_kg - 1.59).abs() < 1e-14);
        assert!((r.pools[1].conservative_oxygen_kg_per_kg - 1.31).abs() < 1e-14);
        assert!((r.pools[1].oxygen_difference_kg_per_kg + 0.01).abs() < 1e-14);
        assert_eq!(r.pools[1].published_oxygen_kg_per_kg, 1.32);
        assert_eq!(r.original_report.recipe_source, supply::recipe_source());
        assert_eq!(r, calculate(&b).unwrap());
    }
    #[test]
    fn actual_daily_projection_keeps_supply_support_and_exact_separate_vapor() {
        let b = source();
        let r = calculate(&b).unwrap();
        let zero = &r.rows[0];
        assert_eq!(zero.conservative_oxygen_kg, 0.0);
        assert_eq!(zero.inferred_dry_elements_kg, [0.0; 4]);
        assert_eq!(zero.element_residual_kg, [0.0; 4]);
        for (row, original) in r.rows.iter().zip(&b.report.rows) {
            assert_eq!(
                row.source_recipe_mass_residual_kg,
                original.recipe_mass_residual_kg
            );
            assert_eq!(row.transpired_vapor_kg, original.transpired_vapor_kg);
            assert_eq!(row.source_supply_support, original.overall_support);
            let total = row.edible_dry_biomass_kg + row.inedible_dry_biomass_kg;
            assert!((row.inferred_dry_elements_kg.iter().sum::<f64>() - total).abs() < 1e-14);
            assert!(row.mass_residual_kg.abs() < 1e-14);
            for k in 0..4 {
                assert!(row.element_residual_kg[k].abs() < 1e-14);
            }
            assert!((row.oxygen_difference_kg + 0.01 * row.inedible_dry_biomass_kg).abs() < 1e-14);
        }
        let mut d = b.report.definition.clone();
        d.supplies.lighting_energy_j = 0.0;
        let limited = Bundle::calculate(&d, None).unwrap();
        let x = calculate(&limited).unwrap();
        assert_eq!(x.rows[30].source_supply_support, supply::Support::Exceeded);
        assert_eq!(
            x.rows[30].conservative_oxygen_kg,
            r.rows[30].conservative_oxygen_kg
        );
    }
    #[test]
    fn legacy_area_scaling_and_nonphysical_source_tampering_are_explicit() {
        let b = source();
        let r = calculate(&b).unwrap();
        let mut d = b.report.definition.clone();
        d.crop.area_m2 *= 2.0;
        let twice = calculate(&Bundle::calculate(&d, None).unwrap()).unwrap();
        for (a, z) in r.rows.iter().zip(&twice.rows) {
            assert_eq!(z.conservative_oxygen_kg, a.conservative_oxygen_kg * 2.0);
            assert_eq!(
                z.inferred_dry_elements_kg,
                a.inferred_dry_elements_kg.map(|v| v * 2.0)
            );
        }
        d.crop.model = crate::greenhouse::LEGACY_MODEL.into();
        d.crop.transpiration = None;
        let legacy = calculate(&Bundle::calculate(&d, None).unwrap()).unwrap();
        assert!(legacy.rows.iter().all(|r| r.transpired_vapor_kg.is_none()));
        assert_eq!(
            legacy.rows[30].conservative_oxygen_kg,
            twice.rows[30].conservative_oxygen_kg
        );
        assert_eq!(legacy.original_report.supported_prefix_last_day, None);
        let mut altered = b.clone();
        altered.report.recipe_source["inedible"]["oxygen"] = 1.31.into();
        assert!(calculate(&altered).is_err());
        let path = temp("invalid");
        let _ = std::fs::remove_dir_all(&path);
        assert!(export(&altered, &path).is_err());
        assert!(!path.exists());
        assert!(pool("invalid", 0.0, 10.0, 10.0, 1.0).is_err());
    }
    #[test]
    fn protected_chemistry_bundle_retains_original_context_and_full_precision() {
        let path = temp("bundle");
        let _ = std::fs::remove_dir_all(&path);
        let u = crate::Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let before = crate::research::state_fingerprint(&u).unwrap();
        let b = Bundle::calculate(&supply::Definition::default(), Some((&u, None))).unwrap();
        let r = calculate(&b).unwrap();
        export(&b, &path).unwrap();
        let csv = std::fs::read(path.join("elements.csv")).unwrap();
        assert!(export(&b, &path).is_err());
        assert_eq!(std::fs::read(path.join("elements.csv")).unwrap(), csv);
        let (opened, actual) = read(&path).unwrap();
        assert_eq!(actual, r);
        assert_eq!(opened.report, b.report);
        assert_eq!(opened.manifest, b.manifest);
        assert_eq!(
            crate::research::state_fingerprint(opened.context.as_ref().unwrap()).unwrap(),
            before
        );
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), before);
        for (line, r) in r.csv().lines().skip(1).zip(&r.rows) {
            let cells = line.split(',').collect::<Vec<_>>();
            assert_eq!(
                cells[4].parse::<f64>().unwrap().to_bits(),
                r.conservative_oxygen_kg.to_bits()
            );
            for k in 0..4 {
                assert_eq!(
                    cells[22 + k].parse::<f64>().unwrap().to_bits(),
                    r.element_residual_kg[k].to_bits()
                );
            }
        }
        let p = path.join("chemistry.json");
        let old = std::fs::read(&p).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&old).unwrap();
        v["atomic_g_mol"][0] = 12.0.into();
        std::fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(read(&path).is_err());
        std::fs::write(&p, old).unwrap();
        std::fs::write(path.join("elements.csv"), b"changed").unwrap();
        assert!(read(&path).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
}
