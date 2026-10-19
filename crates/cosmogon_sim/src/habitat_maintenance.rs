//! Reduced deterministic equipment wear, scheduled maintenance windows and authored
//! capacity calibration for powered cabin life-support components. Rates, wear and repair
//! are user assumptions, not measured hardware reliability or field calibration.
use serde::{Deserialize, Serialize};

pub const MAINTENANCE_MODEL: &str = "cosmogon-cabin-maintenance/1";

fn bounds(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be finite and between {min} and {max}"))
    }
}

/// One component's authored wear, maintenance schedule and as-found capacity multiplier.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ComponentMaintenance {
    /// Condition lost per operating hour at full served power fraction.
    pub wear_per_hour: f64,
    /// Condition restored per maintenance hour; a full window needs duration·repair ≥ 1.
    pub repair_per_hour: f64,
    /// Operating hours between maintenance windows.
    pub interval_hours: f64,
    /// Maintenance window length; the component produces nothing and draws only maintenance power.
    pub duration_hours: f64,
    /// Power drawn only during the maintenance window.
    pub maintenance_power_w: f64,
    /// Authored as-found capacity multiplier on the nominal rating, in (0, 1].
    pub capacity_calibration: f64,
}
impl Default for ComponentMaintenance {
    fn default() -> Self {
        Self {
            wear_per_hour: 0.01,
            repair_per_hour: 1.0,
            interval_hours: 720.0,
            duration_hours: 4.0,
            maintenance_power_w: 50.0,
            capacity_calibration: 1.0,
        }
    }
}
impl ComponentMaintenance {
    pub fn validate(&self, name: &str) -> Result<(), String> {
        bounds(self.wear_per_hour, 0.0, 1000.0, &format!("{name} wear per hour"))?;
        bounds(self.repair_per_hour, 1.0e-6, 1.0e6, &format!("{name} repair per hour"))?;
        bounds(
            self.interval_hours,
            60.0 / 3600.0,
            1.0e6,
            &format!("{name} maintenance interval (h)"),
        )?;
        bounds(
            self.duration_hours,
            60.0 / 3600.0,
            1.0e6,
            &format!("{name} maintenance duration (h)"),
        )?;
        bounds(
            self.maintenance_power_w,
            0.0,
            1.0e6,
            &format!("{name} maintenance power (W)"),
        )?;
        bounds(
            self.capacity_calibration,
            1.0e-6,
            1.0,
            &format!("{name} capacity calibration"),
        )
    }
    pub fn interval_s(&self) -> f64 {
        self.interval_hours * 3600.0
    }
    pub fn duration_s(&self) -> f64 {
        self.duration_hours * 3600.0
    }
    pub fn cycle_s(&self) -> f64 {
        self.interval_s() + self.duration_s()
    }
}

/// Optional per-component maintenance; absent means the component runs at rating forever.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceConfig {
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scrubber: Option<ComponentMaintenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recycler: Option<ComponentMaintenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heater: Option<ComponentMaintenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radiator: Option<ComponentMaintenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moisture: Option<ComponentMaintenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electrolysis: Option<ComponentMaintenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sabatier: Option<ComponentMaintenance>,
}
impl Default for MaintenanceConfig {
    fn default() -> Self {
        Self {
            model: MAINTENANCE_MODEL.into(),
            scrubber: None,
            recycler: None,
            heater: None,
            radiator: None,
            moisture: None,
            electrolysis: None,
            sabatier: None,
        }
    }
}
impl MaintenanceConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MAINTENANCE_MODEL {
            return Err(format!("Unsupported cabin maintenance model {}", self.model));
        }
        for (c, name) in self.components() {
            if let Some(c) = c {
                c.validate(name)?;
            }
        }
        if self.components().iter().all(|(c, _)| c.is_none()) {
            return Err("Cabin maintenance requires at least one configured component".into());
        }
        Ok(())
    }
    /// Canonical component order shared by state, budgets and effects.
    pub fn components(&self) -> [(Option<&ComponentMaintenance>, &'static str); 7] {
        [
            (self.scrubber.as_ref(), "scrubber"),
            (self.recycler.as_ref(), "recycler"),
            (self.heater.as_ref(), "heater"),
            (self.radiator.as_ref(), "radiator"),
            (self.moisture.as_ref(), "moisture"),
            (self.electrolysis.as_ref(), "electrolysis"),
            (self.sabatier.as_ref(), "sabatier"),
        ]
    }
}

/// Held condition and cycle position for one component.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ComponentCondition {
    pub condition: f64,
    pub cycle_s: f64,
}
impl Default for ComponentCondition {
    fn default() -> Self {
        Self {
            condition: 1.0,
            cycle_s: 0.0,
        }
    }
}
impl ComponentCondition {
    pub fn validate(&self, c: &ComponentMaintenance) -> Result<(), String> {
        bounds(self.condition, 0.0, 1.0, "Equipment condition")?;
        // Runtime interval edits may shrink the cycle; stepping keeps cycle_s canonical
        // by remainder, so only finiteness is required across a retained save boundary.
        bounds(self.cycle_s, 0.0, 1.0e12, "Equipment maintenance cycle position")?;
        let _ = c;
        Ok(())
    }
}

/// Optional per-component held condition; absent slots initialize on first use.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct MaintenanceState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scrubber: Option<ComponentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recycler: Option<ComponentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heater: Option<ComponentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radiator: Option<ComponentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moisture: Option<ComponentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electrolysis: Option<ComponentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sabatier: Option<ComponentCondition>,
}
impl MaintenanceState {
    pub fn slots_mut(&mut self) -> [&mut Option<ComponentCondition>; 7] {
        [
            &mut self.scrubber,
            &mut self.recycler,
            &mut self.heater,
            &mut self.radiator,
            &mut self.moisture,
            &mut self.electrolysis,
            &mut self.sabatier,
        ]
    }
    pub fn slots(&self) -> [&Option<ComponentCondition>; 7] {
        [
            &self.scrubber,
            &self.recycler,
            &self.heater,
            &self.radiator,
            &self.moisture,
            &self.electrolysis,
            &self.sabatier,
        ]
    }
}

/// Last completed interval's per-component maintenance accounting.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ComponentBudget {
    pub condition_before: f64,
    pub condition_after: f64,
    pub operating_s: f64,
    pub maintenance_s: f64,
    pub served_fraction: f64,
    pub effective_scale: f64,
    pub wear: f64,
    pub restored: f64,
}
impl ComponentBudget {
    pub fn validate(&self, c: &ComponentMaintenance, seconds: f64) -> Result<(), String> {
        for v in [
            self.condition_before,
            self.condition_after,
            self.operating_s,
            self.maintenance_s,
            self.served_fraction,
            self.effective_scale,
            self.wear,
            self.restored,
        ] {
            if !v.is_finite() || v < 0.0 {
                return Err("Invalid maintenance interval quantity".into());
            }
        }
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-8 * a.abs().max(b.abs()).max(1.0);
        bounds(self.condition_before, 0.0, 1.0, "Condition before interval")?;
        bounds(self.condition_after, 0.0, 1.0, "Condition after interval")?;
        bounds(self.served_fraction, 0.0, 1.0, "Maintenance served fraction")?;
        if self.operating_s + self.maintenance_s > seconds + 1e-6
            || !near(self.effective_scale, c.capacity_calibration * self.condition_before)
            || !near(
                self.wear,
                (c.wear_per_hour * self.served_fraction * self.operating_s / 3600.0)
                    .min(self.condition_before),
            )
            || !near(
                self.restored,
                (c.repair_per_hour * self.maintenance_s / 3600.0)
                    .min(1.0 - (self.condition_before - self.wear)),
            )
            || !near(
                self.condition_after,
                (self.condition_before - self.wear + self.restored).clamp(0.0, 1.0),
            )
        {
            return Err("Maintenance interval disagrees with retained component configuration".into());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct MaintenanceBudget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scrubber: Option<ComponentBudget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recycler: Option<ComponentBudget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heater: Option<ComponentBudget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radiator: Option<ComponentBudget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moisture: Option<ComponentBudget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electrolysis: Option<ComponentBudget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sabatier: Option<ComponentBudget>,
}
impl MaintenanceBudget {
    pub fn slots_mut(&mut self) -> [&mut Option<ComponentBudget>; 7] {
        [
            &mut self.scrubber,
            &mut self.recycler,
            &mut self.heater,
            &mut self.radiator,
            &mut self.moisture,
            &mut self.electrolysis,
            &mut self.sabatier,
        ]
    }
    pub fn slots(&self) -> [&Option<ComponentBudget>; 7] {
        [
            &self.scrubber,
            &self.recycler,
            &self.heater,
            &self.radiator,
            &self.moisture,
            &self.electrolysis,
            &self.sabatier,
        ]
    }
    pub fn validate(&self, m: &MaintenanceConfig, seconds: f64) -> Result<(), String> {
        for (budget, (config, name)) in self.slots().into_iter().zip(m.components()) {
            match (budget, config) {
                (Some(b), Some(c)) => b.validate(c, seconds)?,
                (None, _) => {}
                (Some(_), None) => {
                    return Err(format!("Maintenance interval for unconfigured {name}"));
                }
            }
        }
        Ok(())
    }
}

/// Per-component interval effects consumed by the habitat step.
#[derive(Clone, Copy, Debug)]
pub struct ComponentEffects {
    /// capacity_calibration × start-of-interval condition, in (0, 1].
    pub scale: f64,
    /// Operating share of the interval.
    pub op_fraction: f64,
    /// Maintenance share of the interval.
    pub maint_fraction: f64,
    /// Maintenance power request contribution (W), already time-prorated.
    pub maintenance_w: f64,
}
impl Default for ComponentEffects {
    fn default() -> Self {
        Self {
            scale: 1.0,
            op_fraction: 1.0,
            maint_fraction: 0.0,
            maintenance_w: 0.0,
        }
    }
}

/// Exact intersection of `[cycle_s, cycle_s+dt)` with the operating and maintenance phases
/// of the repeating cycle. Accepted intervals (dt ≤ 60 s, cycle ≥ 120 s) cross one boundary.
pub fn cycle_split(cycle_s: f64, dt: f64, interval_s: f64, duration_s: f64) -> (f64, f64) {
    let cycle = interval_s + duration_s;
    let mut remaining = dt;
    let mut pos = cycle_s.rem_euclid(cycle);
    let mut operating = 0.0;
    let mut maint = 0.0;
    while remaining > 1.0e-12 {
        let to_boundary = if pos < interval_s {
            interval_s - pos
        } else {
            cycle - pos
        };
        let step = remaining.min(to_boundary.max(0.0));
        if pos < interval_s {
            operating += step;
        } else {
            maint += step;
        }
        remaining -= step;
        pos = (pos + step).rem_euclid(cycle);
    }
    (operating, maint)
}

/// Phase-A plan from start-of-interval state; does not mutate conditions or cycles.
pub fn plan(m: &MaintenanceConfig, state: Option<&MaintenanceState>, dt: f64) -> [ComponentEffects; 7] {
    let mut effects = [ComponentEffects::default(); 7];
    for (i, (config, _)) in m.components().into_iter().enumerate() {
        let Some(c) = config else { continue };
        let condition = state
            .map(|s| (*s.slots()[i]).unwrap_or_default())
            .unwrap_or_default();
        let (operating, maint) = cycle_split(condition.cycle_s, dt, c.interval_s(), c.duration_s());
        effects[i] = ComponentEffects {
            scale: c.capacity_calibration * condition.condition,
            op_fraction: operating / dt,
            maint_fraction: maint / dt,
            maintenance_w: c.maintenance_power_w * maint / dt,
        };
    }
    effects
}

/// Phase-B settle: apply wear from served fractions, restore during maintenance and
/// advance cycle positions. `served` is the whole-slot served power fraction per component.
pub fn settle(
    m: &MaintenanceConfig,
    state: Option<&mut MaintenanceState>,
    dt: f64,
    served: [f64; 7],
) -> (MaintenanceBudget, [f64; 7]) {
    let mut budget = MaintenanceBudget::default();
    let mut hours = [0.0; 7];
    let Some(state) = state else {
        return (budget, hours);
    };
    let slots = state.slots_mut();
    for (i, (config, _)) in m.components().into_iter().enumerate() {
        let Some(c) = config else { continue };
        let slot = slots[i].get_or_insert_with(Default::default);
        let condition_before = slot.condition;
        let (operating, maint) = cycle_split(slot.cycle_s, dt, c.interval_s(), c.duration_s());
        let wear = (c.wear_per_hour * served[i] * operating / 3600.0).min(condition_before);
        let restored =
            (c.repair_per_hour * maint / 3600.0).min(1.0 - (condition_before - wear));
        slot.condition = (condition_before - wear + restored).clamp(0.0, 1.0);
        slot.cycle_s = (slot.cycle_s + dt).rem_euclid(c.cycle_s());
        hours[i] = maint / 3600.0;
        *budget.slots_mut()[i] = Some(ComponentBudget {
            condition_before,
            condition_after: slot.condition,
            operating_s: operating,
            maintenance_s: maint,
            served_fraction: served[i],
            effective_scale: c.capacity_calibration * condition_before,
            wear,
            restored,
        });
    }
    (budget, hours)
}

pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MAINTENANCE_MODEL,
        "version":"Explicit engineering assumption; no measured hardware reliability dataset claimed",
        "description":"Per-component linear condition wear while powered, scheduled maintenance windows that restore condition at maintenance power, and an authored as-found capacity multiplier on nominal ratings.",
        "law":"condition += repair·maint_h − wear·served·op_h, clamped to [0,1]; process capacity scales by calibration·condition",
        "uncertainty":null,"limits":"No stochastic failures, spares, crew labor mass, consumables, age curves beyond linear scaling, or maintenance of solar, battery, regulator, crop or storage hardware. Field calibration and predictive uncertainty remain unquantified."})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component() -> ComponentMaintenance {
        ComponentMaintenance {
            wear_per_hour: 0.36,
            repair_per_hour: 0.5,
            interval_hours: 1.0,
            duration_hours: 0.5,
            maintenance_power_w: 40.0,
            capacity_calibration: 0.9,
        }
    }

    #[test]
    fn cycle_split_is_exact_across_phase_boundaries_and_wrap() {
        let interval = 3600.0;
        let duration = 1800.0;
        assert_eq!(cycle_split(0.0, 60.0, interval, duration), (60.0, 0.0));
        assert_eq!(
            cycle_split(interval - 30.0, 60.0, interval, duration),
            (30.0, 30.0)
        );
        assert_eq!(
            cycle_split(interval + duration - 10.0, 60.0, interval, duration),
            (50.0, 10.0)
        );
        let (op, maint) = cycle_split(interval + duration - 10.0, 60.0, interval, duration);
        assert!((op + maint - 60.0).abs() < 1e-12);
    }

    #[test]
    fn wear_and_restore_follow_authored_rates_and_clamp() {
        let c = component();
        let m = MaintenanceConfig {
            scrubber: Some(c.clone()),
            ..Default::default()
        };
        m.validate().unwrap();
        let mut state = MaintenanceState::default();
        // One full operating hour at full served fraction: wear = 0.36.
        let (b, hours) = settle(&m, Some(&mut state), 60.0, [1.0; 7]);
        let _ = hours;
        let sb = b.scrubber.unwrap();
        assert!((sb.wear - 0.36 * 60.0 / 3600.0).abs() < 1e-12);
        assert!((sb.condition_after - (1.0 - sb.wear)).abs() < 1e-12);
        // Run to the maintenance window and restore half an hour at 0.5/h.
        let mut served = [0.0; 7];
        served[0] = 1.0;
        for _ in 0..59 {
            settle(&m, Some(&mut state), 60.0, served);
        }
        let cond = state.scrubber.unwrap().condition;
        assert!((cond - (1.0 - 0.36)).abs() < 1e-9);
        let (b, _) = settle(&m, Some(&mut state), 60.0, served);
        let sb = b.scrubber.unwrap();
        assert!(sb.maintenance_s > 0.0);
        assert!(sb.restored > 0.0);
        assert!(state.scrubber.unwrap().condition > cond);
    }

    #[test]
    fn validation_refuses_bad_parameters_and_empty_config() {
        let mut c = component();
        c.capacity_calibration = 1.5;
        assert!(c.validate("scrubber").is_err());
        c = component();
        c.repair_per_hour = 0.0;
        assert!(c.validate("scrubber").is_err());
        c = component();
        c.interval_hours = 0.0;
        assert!(c.validate("scrubber").is_err());
        let empty = MaintenanceConfig::default();
        assert!(empty.validate().is_err());
        let mut bad = MaintenanceConfig {
            scrubber: Some(component()),
            ..Default::default()
        };
        bad.model = "future/2".into();
        assert!(bad.validate().is_err());
    }

    #[test]
    fn component_budget_rejects_inconsistent_wear_arithmetic() {
        let c = component();
        let mut b = ComponentBudget {
            condition_before: 1.0,
            condition_after: 0.994,
            operating_s: 60.0,
            maintenance_s: 0.0,
            served_fraction: 1.0,
            effective_scale: 0.9,
            wear: 0.006,
            restored: 0.0,
        };
        b.validate(&c, 60.0).unwrap();
        b.wear = 0.5;
        assert!(b.validate(&c, 60.0).is_err());
    }
}
