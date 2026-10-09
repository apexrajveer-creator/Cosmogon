//! Actual finite transfer over an explicitly assumed maintained crop reference cycle.
//! Paused source progress is not a physiological prediction of surviving crop age.
use crate::{crop_chemistry, greenhouse};
use serde::{Deserialize, Serialize};
pub const MODEL: &str = "cosmogon-finite-crop-reference/1";
pub const LIMITS:&str="Finite controlled reference process, not validated crop physiology or chamber hardware. Maintained 23 C/CO2/pressure/humidity are an explicit ideal boundary assumption. Source progress advances only in supported fully powered 16-hour light periods; paused progress does not predict survival or reversible stress. Original daily cumulative source values are piecewise linear between days. CHON-only/no-ash material surrogate; HNO3-equivalent is not literal fertilizer. Dry chemical storage J/kg and auxiliary power are user assumptions, not measured heat of formation or hardware ratings. Vapor/chemical energy must fit actual light electricity. Separate standing/harvested dry stores do not establish calories or feed nominal crew diet. No nutrient solution, root/wet mass, dark physiology, automatic replanting, chamber HVAC/calibration or biological uncertainty.";
const DAY: f64 = 86400.;
const LIGHT: f64 = 57600.;
fn bounds(v: f64, min: f64, max: f64, label: &str) -> Result<(), String> {
    if !v.is_finite() || !(min..=max).contains(&v) {
        Err(format!(
            "{label} must be finite and between {min} and {max}"
        ))
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    pub crop: greenhouse::Definition,
    pub assume_maintained_conditions: bool,
    pub auxiliary_power_w: f64,
    pub dry_chemical_energy_j_kg: f64,
    pub biomass_capacity_kg: f64,
    pub oxygen_capacity_kg: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            crop: Default::default(),
            assume_maintained_conditions: false,
            auxiliary_power_w: 50.,
            dry_chemical_energy_j_kg: 20_000_000.,
            biomass_capacity_kg: 100.,
            oxygen_capacity_kg: 200.,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err("Unsupported finite crop process model".into());
        }
        if self.crop.model != greenhouse::MODEL || self.crop.transpiration.is_none() {
            return Err(
                "Finite crops require the explicit current transpiration source model".into(),
            );
        }
        self.crop.calculate()?;
        bounds(self.auxiliary_power_w, 0., 1e7, "Crop auxiliary power")?;
        bounds(
            self.dry_chemical_energy_j_kg,
            0.,
            1e9,
            "Assumed dry chemical energy",
        )?;
        bounds(
            self.biomass_capacity_kg,
            0.,
            1e7,
            "Crop dry storage capacity",
        )?;
        bounds(
            self.oxygen_capacity_kg,
            0.,
            1e7,
            "Crop shared oxygen reserve capacity",
        )?;
        Ok(())
    }
    pub fn light_power_w(&self) -> f64 {
        self.crop.regime.values().0 * self.crop.area_m2 / self.crop.fixture_efficacy_umol_j
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub model: String,
    pub chemistry_model: String,
    pub atomic_model: String,
    pub source_snapshot_sha256: String,
    pub recipe_snapshot_sha256: String,
    pub source_crop: greenhouse::Definition,
    pub dry_chemical_energy_j_kg: f64,
    pub origin_time_s: f64,
    pub source_progress_days: f64,
    pub initial_nutrient_equivalent_kg: f64,
    pub nutrient_equivalent_kg: f64,
    pub standing_dry_kg: [f64; 2],
    pub harvested_dry_kg: [f64; 2],
    pub harvested: bool,
    /// Original maintained-source daily edible/inedible dry kg and vapor kg; not observations.
    pub source_curve: Vec<[f64; 3]>,
    pub co2_consumed_kg: f64,
    pub metabolic_water_consumed_kg: f64,
    pub vapor_generated_kg: f64,
    pub oxygen_generated_kg: f64,
    pub light_electric_j: f64,
    pub auxiliary_electric_j: f64,
    pub source_progress_seconds: f64,
}
fn curve(c: &greenhouse::Definition) -> Result<Vec<[f64; 3]>, String> {
    c.calculate()?
        .rows
        .into_iter()
        .map(|r| {
            Ok([
                r.edible_dry_biomass_kg,
                r.inedible_dry_biomass_kg,
                r.transpiration
                    .ok_or("Finite crop water source is unavailable")?
                    .transpired_vapor_kg,
            ])
        })
        .collect()
}
impl State {
    pub fn new(c: &Config, nutrient: f64, time: f64) -> Result<Self, String> {
        c.validate()?;
        bounds(nutrient, 0., 1e7, "Initial crop nutrient-equivalent")?;
        bounds(time, -1e16, 1e16, "Crop origin")?;
        Ok(Self {
            model: MODEL.into(),
            chemistry_model: crop_chemistry::MODEL.into(),
            atomic_model: crop_chemistry::ATOMIC_MODEL.into(),
            source_snapshot_sha256: greenhouse::SOURCE_SHA256.into(),
            recipe_snapshot_sha256: crate::greenhouse_supplies::SOURCE_SHA256.into(),
            source_crop: c.crop.clone(),
            dry_chemical_energy_j_kg: c.dry_chemical_energy_j_kg,
            origin_time_s: time,
            source_progress_days: 0.,
            initial_nutrient_equivalent_kg: nutrient,
            nutrient_equivalent_kg: nutrient,
            standing_dry_kg: [0.; 2],
            harvested_dry_kg: [0.; 2],
            harvested: false,
            source_curve: curve(&c.crop)?,
            co2_consumed_kg: 0.,
            metabolic_water_consumed_kg: 0.,
            vapor_generated_kg: 0.,
            oxygen_generated_kg: 0.,
            light_electric_j: 0.,
            auxiliary_electric_j: 0.,
            source_progress_seconds: 0.,
        })
    }
    pub fn dry_mass_kg(&self) -> f64 {
        self.standing_dry_kg
            .iter()
            .chain(self.harvested_dry_kg.iter())
            .sum()
    }
    pub fn mass_kg(&self) -> f64 {
        self.nutrient_equivalent_kg + self.dry_mass_kg()
    }
    pub fn chemical_energy_j(&self) -> f64 {
        self.dry_mass_kg() * self.dry_chemical_energy_j_kg
    }
    pub fn check_config(&self, c: &Config) -> Result<(), String> {
        bounds(c.auxiliary_power_w, 0., 1e7, "Crop auxiliary power")?;
        bounds(c.biomass_capacity_kg, 0., 1e7, "Crop dry capacity")?;
        bounds(c.oxygen_capacity_kg, 0., 1e7, "Crop oxygen capacity")?;
        if c.model != MODEL
            || self.source_crop != c.crop
            || self.dry_chemical_energy_j_kg != c.dry_chemical_energy_j_kg
        {
            return Err("An installed crop source/chemical basis cannot be rewritten; create a new explicit study".into());
        }
        if self.dry_mass_kg() > c.biomass_capacity_kg {
            return Err(
                "Crop capacity is below retained dry material; no biomass discarded".into(),
            );
        }
        Ok(())
    }
    pub fn validate(&self, time: f64) -> Result<(), String> {
        if self.model != MODEL
            || self.chemistry_model != crop_chemistry::MODEL
            || self.atomic_model != crop_chemistry::ATOMIC_MODEL
            || self.source_snapshot_sha256 != greenhouse::SOURCE_SHA256
            || self.recipe_snapshot_sha256 != crate::greenhouse_supplies::SOURCE_SHA256
        {
            return Err("Crop process/source/chemistry pin mismatch".into());
        }
        bounds(time, -1e16, 1e16, "Crop completed time")?;
        if self.origin_time_s > time {
            return Err("Crop origin is after completed clock".into());
        }
        bounds(self.origin_time_s, -1e16, 1e16, "Crop origin")?;
        bounds(self.source_progress_days, 0., 30., "Crop source progress")?;
        bounds(
            self.dry_chemical_energy_j_kg,
            0.,
            1e9,
            "Crop chemical energy basis",
        )?;
        if self.source_crop.model != greenhouse::MODEL
            || self.source_crop.transpiration.is_none()
            || self.source_curve != curve(&self.source_crop)?
        {
            return Err("Crop source curve differs from its pinned original definition".into());
        }
        for v in self
            .standing_dry_kg
            .iter()
            .chain(self.harvested_dry_kg.iter())
            .copied()
            .chain([
                self.initial_nutrient_equivalent_kg,
                self.nutrient_equivalent_kg,
                self.co2_consumed_kg,
                self.metabolic_water_consumed_kg,
                self.vapor_generated_kg,
                self.oxygen_generated_kg,
            ])
        {
            bounds(v, 0., 1e12, "Crop mass counter")?;
        }
        for v in [
            self.light_electric_j,
            self.auxiliary_electric_j,
            self.source_progress_seconds,
        ] {
            bounds(v, 0., 1e30, "Crop execution counter")?;
        }
        let projected = at(&self.source_curve, self.source_progress_days);
        let [e, i, _] = projected;
        let [a, z] = crop_chemistry::pools()?;
        let check = |a: f64, b: f64| {
            if (a - b).abs() > 1e-9 * b.abs().max(1.) {
                Err("Crop material/progress ledger is inconsistent".to_string())
            } else {
                Ok(())
            }
        };
        for (k, value) in projected.iter().copied().enumerate().take(2) {
            check(self.standing_dry_kg[k] + self.harvested_dry_kg[k], value)?;
        }
        check(self.co2_consumed_kg, 1.82 * e + 1.68 * i)?;
        check(self.metabolic_water_consumed_kg, 0.57 * e + 0.55 * i)?;
        check(
            self.initial_nutrient_equivalent_kg - self.nutrient_equivalent_kg,
            0.20 * e + 0.08 * i,
        )?;
        check(self.vapor_generated_kg, projected[2])?;
        check(
            self.oxygen_generated_kg,
            a.conservative_oxygen_kg_per_kg * e + z.conservative_oxygen_kg_per_kg * i,
        )?;
        check(
            self.source_progress_seconds,
            self.source_progress_days * LIGHT,
        )?;
        if self.source_progress_seconds > time - self.origin_time_s + 0.002
            || self.chemical_energy_j()
                + self.vapor_generated_kg * crate::moisture::REFERENCE_PHASE_J_KG
                > self.light_electric_j + 1e-8 * self.light_electric_j.max(1.)
        {
            return Err("Crop production exceeds actual elapsed time or light energy".into());
        }
        if self.harvested {
            if self.source_progress_days != 30. || self.standing_dry_kg != [0.; 2] {
                return Err("Crop harvest requires a completed source cycle".into());
            }
        } else if self.harvested_dry_kg != [0.; 2] {
            return Err("Crop harvested material lacks explicit harvest state".into());
        }
        Ok(())
    }
    pub fn validate_budget(&self, b: &Budget, c: &Config, dt: f64, end: f64) -> Result<(), String> {
        b.validate(c, dt)?;
        if b.source_progress_after != self.source_progress_days {
            return Err("Crop completed progress differs from interval evidence".into());
        }
        let before = at(&self.source_curve, b.source_progress_before);
        let after = at(&self.source_curve, b.source_progress_after);
        for (actual, (a, z)) in [
            b.dry_generated_kg[0],
            b.dry_generated_kg[1],
            b.vapor_generated_kg,
        ]
        .into_iter()
        .zip(before.into_iter().zip(after))
        {
            if (actual - (z - a).max(0.)).abs() > 1e-10 * z.abs().max(1.) {
                return Err("Crop interval transfer disagrees with pinned source progress".into());
            }
        }
        let lit = light_seconds(end - dt - self.origin_time_s, dt);
        let request = if c.assume_maintained_conditions && b.source_progress_before < 30. && dt > 0.
        {
            c.auxiliary_power_w + c.light_power_w() * lit / dt
        } else {
            0.
        };
        if b.light_seconds != lit || b.requested_w != request {
            return Err(
                "Crop interval photoperiod/request disagrees with original clock/source".into(),
            );
        }
        Ok(())
    }
    pub fn harvest(&mut self) -> Result<(), String> {
        if self.source_progress_days != 30. || self.harvested {
            return Err("Harvest requires one complete unharvested reference cycle".into());
        }
        self.harvested_dry_kg = self.standing_dry_kg;
        self.standing_dry_kg = [0.; 2];
        self.harvested = true;
        Ok(())
    }
    pub fn request_w(&self, c: &Config, start: f64, dt: f64) -> f64 {
        if !c.assume_maintained_conditions || self.source_progress_days >= 30. || dt <= 0. {
            0.
        } else {
            c.auxiliary_power_w
                + c.light_power_w() * light_seconds(start - self.origin_time_s, dt) / dt
        }
    }
    pub fn execute(
        &mut self,
        c: &Config,
        start: f64,
        dt: f64,
        power: f64,
        stocks: Stocks,
    ) -> Result<Budget, String> {
        self.check_config(c)?;
        bounds(dt, 0., 60.001, "Crop interval")?;
        bounds(start, self.origin_time_s, 1e16, "Crop interval start")?;
        bounds(power, 0., 1e12, "Crop delivered power")?;
        for v in [stocks.water_kg, stocks.co2_kg, stocks.oxygen_headroom_kg] {
            bounds(v, 0., 1e12, "Crop available stock")?;
        }
        let lit = light_seconds(start - self.origin_time_s, dt);
        let request = self.request_w(c, start, dt);
        let power = power.min(request).max(0.);
        let auxiliary = (c.auxiliary_power_w * dt).min(power * dt);
        let light_energy = (power * dt - auxiliary).max(0.);
        let mut b = Budget {
            requested_w: request,
            delivered_w: power,
            light_seconds: lit,
            light_electric_j: light_energy,
            auxiliary_electric_j: auxiliary,
            source_progress_before: self.source_progress_days,
            source_progress_after: self.source_progress_days,
            reason: String::new(),
            ..Default::default()
        };
        if !c.assume_maintained_conditions {
            b.reason = "Maintained chamber conditions are unconfirmed".into();
        } else if self.source_progress_days >= 30. {
            b.reason = "Reference cycle complete; no automatic new crop".into();
        } else if lit == 0. {
            b.reason = "Scheduled dark period; subdaily physiology is unavailable".into();
        } else if power < request {
            b.reason =
                "Insufficient full-regime lighting/control power; source progress paused".into();
        } else {
            let before = at(&self.source_curve, self.source_progress_days);
            let target = ((self.source_progress_seconds + lit) / LIGHT).min(30.);
            let pools = crop_chemistry::pools()?;
            let delta = |age: f64| {
                let x = at(&self.source_curve, age);
                Transfer::new(
                    std::array::from_fn(|k| (x[k] - before[k]).max(0.)),
                    &pools,
                    c.dry_chemical_energy_j_kg,
                )
            };
            let fits = |t: &Transfer| {
                t.water_kg <= stocks.water_kg
                    && t.co2_kg <= stocks.co2_kg
                    && t.nutrient_kg <= self.nutrient_equivalent_kg
                    && t.oxygen_kg <= stocks.oxygen_headroom_kg
                    && t.dry_kg.iter().sum::<f64>()
                        <= (c.biomass_capacity_kg - self.dry_mass_kg()).max(0.)
                    && t.chemical_j + t.phase_j <= light_energy
            };
            let wanted = delta(target);
            let age = if fits(&wanted) {
                target
            } else {
                let mut low = self.source_progress_days;
                let mut high = target;
                for _ in 0..48 {
                    let mid = low + (high - low) / 2.;
                    if fits(&delta(mid)) {
                        low = mid
                    } else {
                        high = mid
                    }
                }
                low
            };
            let t = delta(age);
            b.reason = if age == target {
                "Supported maintained-reference transfer".into()
            } else {
                "Finite feed, storage or light-energy constraint; source progress limited".into()
            };
            b.source_progress_after = age;
            b.co2_consumed_kg = t.co2_kg;
            b.metabolic_water_kg = t.metabolic_water_kg;
            b.vapor_generated_kg = t.vapor_kg;
            b.nutrient_consumed_kg = t.nutrient_kg;
            b.oxygen_generated_kg = t.oxygen_kg;
            b.dry_generated_kg = t.dry_kg;
            b.chemical_storage_j = t.chemical_j;
            b.vapor_phase_j = t.phase_j;
            b.mass_residual_kg = t.mass_residual_kg;
            b.element_residual_kg = t.element_residual_kg;
            self.source_progress_seconds = if age == target {
                (self.source_progress_seconds + lit).min(30. * LIGHT)
            } else {
                age * LIGHT
            };
            self.source_progress_days = age;
            for k in 0..2 {
                self.standing_dry_kg[k] += t.dry_kg[k];
            }
            self.nutrient_equivalent_kg = (self.nutrient_equivalent_kg - t.nutrient_kg).max(0.);
            self.co2_consumed_kg += t.co2_kg;
            self.metabolic_water_consumed_kg += t.metabolic_water_kg;
            self.vapor_generated_kg += t.vapor_kg;
            self.oxygen_generated_kg += t.oxygen_kg;
        }
        self.light_electric_j += light_energy;
        self.auxiliary_electric_j += auxiliary;
        Ok(b)
    }
}
pub fn light_seconds(start: f64, dt: f64) -> f64 {
    let phase = start.rem_euclid(DAY);
    let first = dt.min(DAY - phase);
    (LIGHT - phase).max(0.).min(first) + (dt - first).min(LIGHT)
}
fn at(curve: &[[f64; 3]], age: f64) -> [f64; 3] {
    let age = age.clamp(0., 30.);
    let day = (age as usize).min(29);
    let f = age - day as f64;
    std::array::from_fn(|k| curve[day][k] + (curve[day + 1][k] - curve[day][k]) * f)
}
#[derive(Clone, Copy, Debug)]
pub struct Stocks {
    pub water_kg: f64,
    pub co2_kg: f64,
    pub oxygen_headroom_kg: f64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub requested_w: f64,
    pub delivered_w: f64,
    pub light_seconds: f64,
    pub light_electric_j: f64,
    pub auxiliary_electric_j: f64,
    pub source_progress_before: f64,
    pub source_progress_after: f64,
    pub reason: String,
    pub co2_consumed_kg: f64,
    pub metabolic_water_kg: f64,
    pub vapor_generated_kg: f64,
    pub nutrient_consumed_kg: f64,
    pub oxygen_generated_kg: f64,
    pub dry_generated_kg: [f64; 2],
    pub chemical_storage_j: f64,
    pub vapor_phase_j: f64,
    pub mass_residual_kg: f64,
    pub element_residual_kg: [f64; 4],
}
impl Budget {
    pub fn validate(&self, c: &Config, dt: f64) -> Result<(), String> {
        for v in [
            self.requested_w,
            self.delivered_w,
            self.light_seconds,
            self.light_electric_j,
            self.auxiliary_electric_j,
            self.co2_consumed_kg,
            self.metabolic_water_kg,
            self.vapor_generated_kg,
            self.nutrient_consumed_kg,
            self.oxygen_generated_kg,
            self.chemical_storage_j,
            self.vapor_phase_j,
            self.dry_generated_kg[0],
            self.dry_generated_kg[1],
        ] {
            bounds(v, 0., 1e30, "Crop interval quantity")?;
        }
        bounds(self.source_progress_before, 0., 30., "Crop prior progress")?;
        bounds(
            self.source_progress_after,
            self.source_progress_before,
            30.,
            "Crop completed progress",
        )?;
        if self.reason.len() > 512
            || self.delivered_w > self.requested_w
            || self.light_seconds > dt + 0.001
            || self.thermal_debit_j() > self.light_electric_j + 1e-8 * self.light_electric_j.max(1.)
            || ((self.light_electric_j + self.auxiliary_electric_j) - self.delivered_w * dt).abs()
                > 1e-8 * (self.delivered_w * dt).max(1.)
            || (self.chemical_storage_j
                - self.dry_generated_kg.iter().sum::<f64>() * c.dry_chemical_energy_j_kg)
                .abs()
                > 1e-8 * self.chemical_storage_j.max(1.)
            || (self.vapor_phase_j
                - self.vapor_generated_kg * crate::moisture::REFERENCE_PHASE_J_KG)
                .abs()
                > 1e-8 * self.vapor_phase_j.max(1.)
        {
            return Err("Crop interval material/power/phase accounting is inconsistent".into());
        }
        for v in self
            .element_residual_kg
            .iter()
            .copied()
            .chain([self.mass_residual_kg])
        {
            bounds(v, -1e12, 1e12, "Crop interval residual")?;
        }
        let expected = Transfer::new(
            [
                self.dry_generated_kg[0],
                self.dry_generated_kg[1],
                self.vapor_generated_kg,
            ],
            &crop_chemistry::pools()?,
            c.dry_chemical_energy_j_kg,
        );
        for (actual, wanted) in [
            (self.co2_consumed_kg, expected.co2_kg),
            (self.metabolic_water_kg, expected.metabolic_water_kg),
            (self.nutrient_consumed_kg, expected.nutrient_kg),
            (self.oxygen_generated_kg, expected.oxygen_kg),
            (self.mass_residual_kg, expected.mass_residual_kg),
        ]
        .into_iter()
        .chain(
            self.element_residual_kg
                .into_iter()
                .zip(expected.element_residual_kg),
        ) {
            if (actual - wanted).abs() > 1e-10 * wanted.abs().max(1.) {
                return Err("Crop interval chemistry differs from retained material basis".into());
            }
        }
        if self.source_progress_after - self.source_progress_before
            > self.light_seconds / LIGHT + 1e-10
        {
            return Err("Crop progress exceeds illuminated interval".into());
        }
        Ok(())
    }
    pub fn water_consumed_kg(&self) -> f64 {
        self.metabolic_water_kg + self.vapor_generated_kg
    }
    pub fn thermal_debit_j(&self) -> f64 {
        self.chemical_storage_j + self.vapor_phase_j
    }
}
struct Transfer {
    co2_kg: f64,
    water_kg: f64,
    metabolic_water_kg: f64,
    vapor_kg: f64,
    nutrient_kg: f64,
    oxygen_kg: f64,
    dry_kg: [f64; 2],
    chemical_j: f64,
    phase_j: f64,
    mass_residual_kg: f64,
    element_residual_kg: [f64; 4],
}
impl Transfer {
    fn new([e, i, v]: [f64; 3], p: &[crop_chemistry::Pool; 2], chemical: f64) -> Self {
        let co2 = 1.82 * e + 1.68 * i;
        let met = 0.57 * e + 0.55 * i;
        let nutrient = 0.20 * e + 0.08 * i;
        let oxygen =
            p[0].conservative_oxygen_kg_per_kg * e + p[1].conservative_oxygen_kg_per_kg * i;
        let input: [f64; 4] = std::array::from_fn(|k| {
            p[0].input_elements_kg_per_kg[k] * e + p[1].input_elements_kg_per_kg[k] * i
        });
        let dry: [f64; 4] = std::array::from_fn(|k| {
            p[0].inferred_dry_elements_kg_per_kg[k] * e
                + p[1].inferred_dry_elements_kg_per_kg[k] * i
        });
        let output = [dry[0], dry[1], dry[2] + oxygen, dry[3]];
        Self {
            co2_kg: co2,
            water_kg: met + v,
            metabolic_water_kg: met,
            vapor_kg: v,
            nutrient_kg: nutrient,
            oxygen_kg: oxygen,
            dry_kg: [e, i],
            chemical_j: (e + i) * chemical,
            phase_j: v * crate::moisture::REFERENCE_PHASE_J_KG,
            mass_residual_kg: co2 + met + nutrient - e - i - oxygen,
            element_residual_kg: std::array::from_fn(|k| input[k] - output[k]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{Definition, Environment, Habitat},
        BodyRef,
    };
    fn enabled() -> Config {
        Config {
            assume_maintained_conditions: true,
            ..Default::default()
        }
    }
    fn supplies() -> Stocks {
        Stocks {
            water_kg: 200.,
            co2_kg: 4.,
            oxygen_headroom_kg: 100.,
        }
    }
    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a:.17e} vs {b:.17e}");
    }
    #[test]
    fn complete_finite_cycle_conserves_actual_mass_energy_elements_and_harvest() {
        let c = enabled();
        let mut s = State::new(&c, 1., 0.).unwrap();
        let mut stocks = supplies();
        let mut oxygen = 100.;
        let mut vapor = 0.;
        let mass = stocks.water_kg + stocks.co2_kg + oxygen + vapor + s.mass_kg();
        let mut sensible = 0.;
        let mut electric = 0.;
        let mut lit = 0.;
        for n in 0..30 * 1440 {
            let start = n as f64 * 60.;
            let request = s.request_w(&c, start, 60.);
            let b = s.execute(&c, start, 60., request, stocks).unwrap();
            b.validate(&c, 60.).unwrap();
            stocks.water_kg -= b.water_consumed_kg();
            stocks.co2_kg -= b.co2_consumed_kg;
            stocks.oxygen_headroom_kg -= b.oxygen_generated_kg;
            oxygen += b.oxygen_generated_kg;
            vapor += b.vapor_generated_kg;
            sensible += b.delivered_w * 60. - b.thermal_debit_j();
            electric += b.delivered_w * 60.;
            if b.light_electric_j > 0. {
                lit += b.light_seconds;
            }
            close(
                stocks.water_kg + stocks.co2_kg + oxygen + vapor + s.mass_kg(),
                mass,
                1e-9,
            );
            for x in b.element_residual_kg {
                assert!(x.abs() < 1e-14);
            }
            assert!(
                stocks.water_kg >= 0.
                    && stocks.co2_kg >= 0.
                    && stocks.oxygen_headroom_kg >= 0.
                    && s.nutrient_equivalent_kg >= 0.
            );
        }
        assert_eq!(s.source_progress_days, 30.);
        s.validate(30. * DAY).unwrap();
        let original = c.crop.calculate().unwrap();
        let last = &original.rows[30];
        close(s.standing_dry_kg[0], last.edible_dry_biomass_kg, 1e-12);
        close(s.standing_dry_kg[1], last.inedible_dry_biomass_kg, 1e-12);
        close(
            s.vapor_generated_kg,
            last.transpiration.as_ref().unwrap().transpired_vapor_kg,
            1e-10,
        );
        close(s.light_electric_j, last.lighting_electric_j, 1e-3);
        close(lit, 30. * LIGHT, 1e-5);
        close(
            sensible + s.chemical_energy_j() + vapor * crate::moisture::REFERENCE_PHASE_J_KG,
            electric,
            1e-3,
        );
        let before = (s.mass_kg(), s.chemical_energy_j(), s.co2_consumed_kg);
        let dry = s.standing_dry_kg;
        s.harvest().unwrap();
        assert_eq!(s.harvested_dry_kg, dry);
        assert_eq!(s.standing_dry_kg, [0.; 2]);
        assert_eq!(
            (s.mass_kg(), s.chemical_energy_j(), s.co2_consumed_kg),
            before
        );
        s.validate(30. * DAY).unwrap();
        assert!(s.harvest().is_err());
        let b = s.execute(&c, 30. * DAY, 60., 0., stocks).unwrap();
        assert_eq!(b.dry_generated_kg, [0.; 2]);
        assert_eq!(s.source_progress_days, 30.);
    }
    #[test]
    fn every_finite_feed_or_product_limit_bounds_progress_without_free_material() {
        let c = enabled();
        let reference = State::new(&c, 1., 0.).unwrap();
        let mut full = reference.clone();
        let b = full
            .execute(&c, 0., 60., full.request_w(&c, 0., 60.), supplies())
            .unwrap();
        assert!(b.water_consumed_kg() > 0. && b.dry_generated_kg[1] > 0.);
        for k in 0..5 {
            let mut config = c.clone();
            let mut state = reference.clone();
            let mut stock = supplies();
            match k {
                0 => stock.water_kg = 0.,
                1 => stock.co2_kg = 0.,
                2 => {
                    state.nutrient_equivalent_kg = 0.;
                    state.initial_nutrient_equivalent_kg = 0.;
                }
                3 => stock.oxygen_headroom_kg = 0.,
                _ => config.biomass_capacity_kg = 0.,
            };
            let request = state.request_w(&config, 0., 60.);
            let r = state.execute(&config, 0., 60., request, stock).unwrap();
            assert_eq!(state.source_progress_days, 0.);
            assert_eq!(r.dry_generated_kg, [0.; 2]);
            assert_eq!(r.oxygen_generated_kg, 0.);
            assert!(r.delivered_w > 0.);
            state.validate(60.).unwrap();
        }
        let mut state = reference;
        let stock = Stocks {
            water_kg: b.water_consumed_kg() / 2.,
            ..supplies()
        };
        let limited = state
            .execute(&c, 0., 60., state.request_w(&c, 0., 60.), stock)
            .unwrap();
        assert!(
            limited.source_progress_after > 0.
                && limited.source_progress_after < b.source_progress_after
        );
        assert!(limited.water_consumed_kg() <= stock.water_kg);
        close(limited.water_consumed_kg(), stock.water_kg, 1e-12);
        state.validate(60.).unwrap();
    }
    #[test]
    fn unconfirmed_conditions_darkness_partial_power_and_energy_constraints_are_explicit() {
        assert_eq!(light_seconds(LIGHT - 30., 60.), 30.);
        assert_eq!(light_seconds(DAY - 30., 60.), 30.);
        assert_eq!(light_seconds(LIGHT + 60., 60.), 0.);
        let mut c = Config::default();
        let mut state = State::new(&c, 1., 0.).unwrap();
        let b = state.execute(&c, 0., 60., 0., supplies()).unwrap();
        assert_eq!(b.requested_w, 0.);
        assert_eq!(state.source_progress_days, 0.);
        assert!(b.reason.contains("unconfirmed"));
        c.assume_maintained_conditions = true;
        let request = state.request_w(&c, 60., 60.);
        let b = state
            .execute(&c, 60., 60., request - 1., supplies())
            .unwrap();
        assert_eq!(state.source_progress_days, 0.);
        assert!(b.light_electric_j > 0. && b.thermal_debit_j() == 0.);
        assert!(b.reason.contains("Insufficient"));
        let b = state
            .execute(&c, LIGHT + 60., 60., c.auxiliary_power_w, supplies())
            .unwrap();
        assert_eq!(b.light_electric_j, 0.);
        assert_eq!(b.auxiliary_electric_j, c.auxiliary_power_w * 60.);
        assert_eq!(state.source_progress_days, 0.);
        c.dry_chemical_energy_j_kg = 1e9;
        let mut high = State::new(&c, 1., 0.).unwrap();
        for n in 0..30 * 1440 {
            let start = n as f64 * 60.;
            let b = high
                .execute(&c, start, 60., high.request_w(&c, start, 60.), supplies())
                .unwrap();
            assert!(b.thermal_debit_j() <= b.light_electric_j + 1e-9);
            b.validate(&c, 60.).unwrap();
        }
        assert!(high.source_progress_days > 0. && high.source_progress_days < 30.);
        high.validate(30. * DAY).unwrap();
    }
    fn habitat() -> Habitat {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0., 0.);
        d.config.crop = Some(enabled());
        d.config.moisture = Some(crate::moisture::Config {
            power_limit_w: 0.,
            rate_kg_s: 0.,
            ..Default::default()
        });
        d.config.crew = 0;
        d.config.base_load_w = 0.;
        d.config.scrubber_power_w = 0.;
        d.config.recycler_power_w = 0.;
        d.config.scrubber_rate_kg_s = 0.;
        d.config.recycler_rate_kg_s = 0.;
        d.config.heater_limit_w = 0.;
        d.config.radiator_pump_w = 0.;
        d.config.radiator_area_m2 = 0.;
        d.config.conductance_w_k = 0.;
        d.config.firm_power_w = 5000.;
        d.initial.captured_co2_kg = 4.;
        d.initial.crop_nutrient_equivalent_kg = 1.;
        d.initial.battery_j = 0.;
        Habitat::new(0, d, 0.).unwrap()
    }
    fn env() -> Environment {
        Environment {
            irradiance_w_m2: Some(0.),
            exterior_temperature_k: Some(210.),
        }
    }
    #[test]
    fn habitat_transfers_and_cabin_heat_include_chemical_and_vapor_storage() {
        let mut h = habitat();
        let initial = h.clone();
        let mass = h.state.accounted_mass_kg();
        for n in 1..=2 * 1440 {
            h.step(n as f64 * 60., env()).unwrap();
            assert!(h.budget.mass_residual_kg.abs() < 1e-10);
            assert!(h.budget.electric_residual_j.abs() < 1e-7);
            assert!(h.budget.thermal_residual_j.abs() < 1e-4);
        }
        h.validate(h.last_time).unwrap();
        let crop = h.state.crop.as_ref().unwrap();
        assert!(crop.source_progress_days > 1.9);
        assert!(crop.standing_dry_kg[0] > 0.);
        assert_eq!(h.state.food_dry_kg, initial.state.food_dry_kg);
        assert_eq!(h.state.food_water_kg, initial.state.food_water_kg);
        close(h.state.accounted_mass_kg(), mass, 1e-9);
        let total = h.state.battery_j - initial.state.battery_j
            + h.config.heat_capacity_j_k * (h.state.temperature_k - initial.state.temperature_k)
            + crop.chemical_energy_j()
            + h.state.phase_energy_j();
        close(total, h.totals.generated_j - h.totals.curtailed_j, 0.01);
        let l = h.heat.as_ref().unwrap();
        assert!(
            l.crop_storage_debits_j.unwrap()[0] < 0. && l.crop_storage_debits_j.unwrap()[1] < 0.
        );
        close(l.heat_path_sum_j(), l.terms_j[10], 0.01);
        let d = crate::habitat_diagnostics::explain(&h);
        let sum = d
            .heat
            .iter()
            .find(|v| v.id == "sum")
            .unwrap()
            .value
            .unwrap();
        close(
            sum + h.budget.thermal_residual_j / 60.,
            h.budget.thermal_storage_w,
            1e-6,
        );
        let allocations = crate::habitat_diagnostics::allocations(&h);
        close(
            allocations.iter().map(|v| v.served_w.unwrap()).sum::<f64>(),
            h.budget.delivered_w,
            1e-6,
        );
    }
    #[test]
    fn original_stocks_sources_and_curve_cannot_be_granted_or_rewritten_by_edits() {
        let h = habitat();
        let mut c = h.config.clone();
        c.crop.as_mut().unwrap().crop.area_m2 *= 2.;
        assert!(h.check_config(&c).is_err());
        c = h.config.clone();
        c.crop.as_mut().unwrap().dry_chemical_energy_j_kg += 1.;
        assert!(h.check_config(&c).is_err());
        c = h.config.clone();
        c.crop = None;
        assert!(h.check_config(&c).is_ok());
        let mut ordinary = h.definition.clone();
        ordinary.config.crop = None;
        ordinary.initial.crop_nutrient_equivalent_kg = 0.;
        let legacy = Habitat::new(1, ordinary, 0.).unwrap();
        assert!(legacy.check_config(&h.config).is_err());
        let mut state = h.state.crop.unwrap();
        state.source_curve[1][0] += 1.;
        assert!(state.validate(0.).is_err());
        let mut c = enabled();
        c.crop.model = greenhouse::LEGACY_MODEL.into();
        c.crop.transpiration = None;
        assert!(c.validate().is_err());
    }
}

/// Exact assumptions/source identity for optional actual finite production.
pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"chemistry_model":crop_chemistry::MODEL,"atomic_model":crop_chemistry::ATOMIC_MODEL,"growth_source":greenhouse::SOURCE,"growth_snapshot_sha256":greenhouse::SOURCE_SHA256,"recipe_source":crate::greenhouse_supplies::SOURCE,"recipe_snapshot_sha256":crate::greenhouse_supplies::SOURCE_SHA256,"limits":LIMITS,"time_scope":"Origin-relative terrestrial 24-hour day / 16-hour light; effective supported reference progress, not physiological age","energy_basis":"Original user-assumed dry J/kg; no measured tissue enthalpy"})
}
/// Separate actual crop evidence keeps the original habitat CSV catalog immutable.
pub fn samples_csv(rows: &[crate::habitat::Sample]) -> String {
    let mut csv=String::from("simulation_time_s,completed_habitat_time_s,source_progress_days,nutrient_equivalent_kg,standing_edible_dry_kg,standing_inedible_dry_kg,harvested_edible_dry_kg,harvested_inedible_dry_kg,harvested,chemical_energy_j,total_co2_consumed_kg,total_metabolic_water_consumed_kg,total_vapor_generated_kg,total_oxygen_generated_kg,total_light_electric_j,total_auxiliary_electric_j,interval_requested_w,interval_delivered_w,interval_water_consumed_kg,interval_oxygen_generated_kg,interval_chemical_storage_j,interval_vapor_phase_j,interval_mass_residual_kg\n");
    for r in rows {
        let mut cells = vec![
            format!("{:.17e}", r.simulation_time_s),
            format!("{:.17e}", r.habitat.last_time),
        ];
        if let Some(s) = &r.habitat.state.crop {
            cells.extend(
                [
                    s.source_progress_days,
                    s.nutrient_equivalent_kg,
                    s.standing_dry_kg[0],
                    s.standing_dry_kg[1],
                    s.harvested_dry_kg[0],
                    s.harvested_dry_kg[1],
                    if s.harvested { 1. } else { 0. },
                    s.chemical_energy_j(),
                    s.co2_consumed_kg,
                    s.metabolic_water_consumed_kg,
                    s.vapor_generated_kg,
                    s.oxygen_generated_kg,
                    s.light_electric_j,
                    s.auxiliary_electric_j,
                ]
                .into_iter()
                .map(|v| format!("{v:.17e}")),
            );
        } else {
            cells.extend(vec![String::new(); 14]);
        }
        if let Some(b) = &r.habitat.budget.crop {
            cells.extend(
                [
                    b.requested_w,
                    b.delivered_w,
                    b.water_consumed_kg(),
                    b.oxygen_generated_kg,
                    b.chemical_storage_j,
                    b.vapor_phase_j,
                    b.mass_residual_kg,
                ]
                .into_iter()
                .map(|v| format!("{v:.17e}")),
            );
        } else {
            cells.extend(vec![String::new(); 7]);
        }
        csv.push_str(&cells.join(","));
        csv.push('\n');
    }
    csv
}
/// New original setup only. Existing habitats cannot acquire these initial stocks through a toggle.
pub fn definition_at(body: crate::BodyRef, lat: f64, lon: f64) -> crate::habitat::Definition {
    let mut d = crate::habitat::Definition::at(body, lat, lon);
    d.name = "Finite controlled crop experiment".into();
    d.config.crop = Some(Config::default());
    d.config.moisture = Some(Default::default());
    d.initial.captured_co2_kg = 10.;
    d.initial.crop_nutrient_equivalent_kg = 1.;
    d
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{
        habitat, habitat_recording::Recording, sandbox::Edit, save, Scenario, Universe,
        UniverseSettings,
    };
    fn world() -> Universe {
        let mut u = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let body = crate::BodyRef {
            system: 0,
            body: u.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut d = definition_at(body, 0., 0.);
        d.config.crop.as_mut().unwrap().assume_maintained_conditions = true;
        d.config.crew = 0;
        d.config.solar_area_m2 = 0.;
        d.config.firm_power_w = 5000.;
        d.config.base_load_w = 0.;
        d.config.scrubber_power_w = 0.;
        d.config.recycler_power_w = 0.;
        d.config.heater_limit_w = 0.;
        d.config.radiator_pump_w = 0.;
        d.config.radiator_area_m2 = 0.;
        d.config.conductance_w_k = 0.;
        d.config.heat_capacity_j_k = 1e9;
        d.config.moisture.as_mut().unwrap().power_limit_w = 0.;
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        u
    }
    #[test]
    fn exact_saved_replay_remainders_disable_and_legacy_pins() {
        let mut a = world();
        a.advance_by(6030.);
        let mut c = a.habitats[0].config.clone();
        c.crop = None;
        a.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        assert_eq!(a.habitats[0].last_time, a.time);
        let held = a.habitats[0].state.crop.clone();
        a.advance_by(10000.);
        assert_eq!(a.habitats[0].state.crop, held);
        let mut c = a.habitats[0].config.clone();
        c.crop = a.habitats[0].definition.config.crop.clone();
        a.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        let text = save::to_json(&a, "finite crop fixture").unwrap();
        let mut b = save::from_json(&text).unwrap();
        a.advance_by(12345.);
        b.advance_by(12345.);
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
        let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
        v["universe"]["habitats"][0]["state"]["crop"]["source_curve"][1][1] = serde_json::json!(9.);
        assert!(save::from_json(&v.to_string()).is_err());
        let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
        v["universe"]["habitats"][0]["heat"]["crop_storage_debits_j"] = serde_json::json!([0., 0.]);
        assert!(save::from_json(&v.to_string()).is_err());
        let old = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let v: serde_json::Value =
            serde_json::from_str(&save::to_json(&old, "legacy").unwrap()).unwrap();
        assert!(!v.to_string().contains(MODEL));
    }
    #[test]
    fn original_closed_cycle_exports_reopen_and_harvest_without_diet_credit() {
        let mut u = world();
        let initial = u.clone();
        let mut rows = vec![habitat::sample(&u, 0).unwrap()];
        for _ in 0..30 {
            u.advance_by(DAY);
            rows.push(habitat::sample(&u, 0).unwrap());
        }
        assert_eq!(
            u.habitats[0]
                .state
                .crop
                .as_ref()
                .unwrap()
                .source_progress_days,
            30.
        );
        let root =
            std::env::temp_dir().join(format!("cosmogon-finite-crop-cycle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        habitat::export(&initial, &u, 0, &rows, &root).unwrap();
        let recorded = Recording::read(&root).unwrap();
        assert_eq!(recorded.rows, rows);
        assert_eq!(
            std::fs::read_to_string(root.join("crop.csv")).unwrap(),
            samples_csv(&rows)
        );
        let balance_path = root.with_extension("balance");
        let _ = std::fs::remove_dir_all(&balance_path);
        let bundle = crate::habitat_balance::Bundle::ordinary(&recorded, 0, 30).unwrap();
        bundle.export(&balance_path).unwrap();
        assert_eq!(bundle.reports[0].stocks.len(), 26);
        assert!(std::fs::read_to_string(balance_path.join("heat.csv"))
            .unwrap()
            .contains("crop_chemical"));
        std::fs::remove_dir_all(balance_path).unwrap();
        let balance = crate::habitat_heat::span(&rows).unwrap();
        assert_eq!(balance.terms.len(), 13);
        assert!(balance.heat_minus_storage_residual_j.abs() < 10.);
        let food = (
            u.habitats[0].state.food_dry_kg,
            u.habitats[0].state.food_water_kg,
        );
        let mass = u.habitats[0].state.accounted_mass_kg();
        let chemical = u.habitats[0]
            .state
            .crop
            .as_ref()
            .unwrap()
            .chemical_energy_j();
        u.apply_edit(Edit::HarvestHabitatCrop { system: 0, id: 0 })
            .unwrap();
        assert_eq!(
            (
                u.habitats[0].state.food_dry_kg,
                u.habitats[0].state.food_water_kg
            ),
            food
        );
        assert_eq!(u.habitats[0].state.accounted_mass_kg(), mass);
        assert_eq!(
            u.habitats[0]
                .state
                .crop
                .as_ref()
                .unwrap()
                .chemical_energy_j(),
            chemical
        );
        let text = save::to_json(&u, "harvested").unwrap();
        save::from_json(&text).unwrap();
        assert!(u
            .apply_edit(Edit::HarvestHabitatCrop { system: 0, id: 0 })
            .is_err());
        std::fs::write(root.join("crop.csv"), "tampered").unwrap();
        assert!(Recording::read(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn priority_partial_power_and_invalid_source_are_atomic() {
        let mut u = world();
        let h = &mut u.habitats[0];
        h.config.firm_power_w = 100.;
        h.config.discharge_limit_w = 0.;
        h.config.base_load_w = 100.;
        let before = h.state.crop.clone();
        h.step(
            h.last_time + 60.,
            habitat::Environment {
                irradiance_w_m2: Some(0.),
                exterior_temperature_k: Some(210.),
            },
        )
        .unwrap();
        assert_eq!(h.budget.crop.as_ref().unwrap().delivered_w, 0.);
        assert_eq!(h.state.crop, before);
        h.config.crop.as_mut().unwrap().crop.area_m2 *= 2.;
        let before = h.clone();
        assert!(h
            .step(
                h.last_time + 60.,
                habitat::Environment {
                    irradiance_w_m2: Some(0.),
                    exterior_temperature_k: Some(210.)
                }
            )
            .is_err());
        assert_eq!(*h, before);
    }
}
