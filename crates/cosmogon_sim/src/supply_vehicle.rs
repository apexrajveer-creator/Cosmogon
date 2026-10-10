//! Explicit finite supply-vehicle mass and ideal free-space propulsion.
//! Not a launched vehicle or delivered inventory; no world state is changed.
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MODEL: &str = "cosmogon-supply-vehicle-reference/1";
pub const G0: f64 = 9.80665;
pub const LIMITS: &str = "Authored constant effective-Isp free-space propulsion bench, not calibrated flight hardware. Actual finite interval endpoints and an exact depletion boundary are retained; connecting plot segments are display guides, not continuous inferred data. Dry mass includes structure, engines, tanks, packaging and all other non-consumable equipment by user assumption. Cargo remains aboard and is distinct from propellant; no habitat supply credit. Ideal vector velocity changes exclude gravity, aerodynamic/pressure variation, attitude dynamics, staging, thermal, power, boiloff, navigation, launch, landing and delivery. Standard g0 is a unit convention, not local planetary gravity. These calculations do not establish mission feasibility, uncertainty or safety.";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cargo {
    pub oxygen_reserve_kg: f64,
    pub potable_water_kg: f64,
    pub food_dry_kg: f64,
    pub food_water_kg: f64,
}
impl Cargo {
    pub fn mass_kg(&self) -> f64 {
        self.oxygen_reserve_kg + self.potable_water_kg + self.food_dry_kg + self.food_water_kg
    }
    fn validate(&self) -> Result<(), String> {
        for (v, n) in [
            (self.oxygen_reserve_kg, "Cargo reserve oxygen"),
            (self.potable_water_kg, "Cargo potable water"),
            (self.food_dry_kg, "Cargo dry food"),
            (self.food_water_kg, "Cargo food-bound water"),
        ] {
            bounds(v, 0., 1e9, n)?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub name: String,
    pub dry_mass_kg: f64,
    pub propellant_capacity_kg: f64,
    pub initial_propellant_kg: f64,
    pub payload_capacity_kg: f64,
    pub cargo: Cargo,
    pub rated_thrust_n: f64,
    pub effective_isp_s: f64,
}
impl Definition {
    pub fn example() -> Self {
        Self {
            model: MODEL.into(),
            name: "Authored supply-vehicle propulsion bench".into(),
            dry_mass_kg: 1000.,
            propellant_capacity_kg: 1000.,
            initial_propellant_kg: 1000.,
            payload_capacity_kg: 200.,
            cargo: Cargo {
                oxygen_reserve_kg: 20.,
                potable_water_kg: 100.,
                food_dry_kg: 30.,
                food_water_kg: 20.,
            },
            rated_thrust_n: 10000.,
            effective_isp_s: 300.,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err("Unsupported supply-vehicle model".into());
        }
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err("Vehicle name requires 1–120 bytes".into());
        }
        bounds(self.dry_mass_kg, 1., 1e12, "Vehicle dry mass")?;
        bounds(
            self.propellant_capacity_kg,
            0.,
            1e12,
            "Propellant tank capacity",
        )?;
        bounds(
            self.initial_propellant_kg,
            0.,
            self.propellant_capacity_kg,
            "Held propellant",
        )?;
        bounds(self.payload_capacity_kg, 0., 4e9, "Payload mass capacity")?;
        self.cargo.validate()?;
        if self.cargo.mass_kg() > self.payload_capacity_kg {
            return Err("Held cargo exceeds payload mass capacity".into());
        }
        bounds(self.rated_thrust_n, 0., 1e9, "Rated effective thrust")?;
        bounds(
            self.effective_isp_s,
            0.1,
            1e5,
            "Assumed effective specific impulse",
        )?;
        Ok(())
    }
    pub fn inert_mass_kg(&self) -> f64 {
        self.dry_mass_kg + self.cargo.mass_kg()
    }
    pub fn exhaust_velocity_m_s(&self) -> f64 {
        self.effective_isp_s * G0
    }
    pub fn initial_ideal_delta_v_m_s(&self) -> f64 {
        self.exhaust_velocity_m_s() * (self.initial_propellant_kg / self.inert_mass_kg()).dln1p()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub duration_s: f64,
    pub throttle: f64,
    /// Already-unit inertial bench direction. Coasting ignores it but still validates input.
    pub direction: [f64; 3],
}
impl Command {
    fn validate(&self) -> Result<(), String> {
        bounds(self.duration_s, 0., 1e7, "Command duration")?;
        bounds(self.throttle, 0., 1., "Command throttle")?;
        if self
            .direction
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1.)
            || (self.direction.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 1e-12
        {
            return Err("Burn direction must be an explicit finite unit vector".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub format: String,
    pub schema_version: u32,
    pub definition: Definition,
    pub commands: Vec<Command>,
    #[serde(default = "default_observation_interval")]
    pub observation_interval_s: f64,
}
impl Plan {
    pub fn example() -> Self {
        Self {
            format: "cosmogon-supply-vehicle-plan".into(),
            schema_version: 1,
            observation_interval_s: 1.,
            definition: Definition::example(),
            commands: vec![
                Command {
                    duration_s: 150.,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                },
                Command {
                    duration_s: 60.,
                    throttle: 0.,
                    direction: [1., 0., 0.],
                },
                Command {
                    duration_s: 150.,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                },
            ],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "cosmogon-supply-vehicle-plan" || self.schema_version != 1 {
            return Err("Unsupported supply-vehicle plan".into());
        }
        self.definition.validate()?;
        if self.commands.is_empty() || self.commands.len() > 1024 {
            return Err("Vehicle plans require 1–1024 explicit commands".into());
        }
        bounds(
            self.observation_interval_s,
            1e-3,
            1e7,
            "Observation interval",
        )?;
        let mut observations = 2.; // Initial row and at most one extra exact fuel-depletion boundary.
        for c in &self.commands {
            c.validate()?;
            observations += (c.duration_s / self.observation_interval_s).ceil().max(1.);
        }
        if observations > 8192. {
            return Err("Vehicle plans support at most 8192 actual observations; increase the interval or shorten commands".into());
        }
        Ok(())
    }
    pub fn run(&self) -> Result<Vec<Observation>, String> {
        self.validate()?;
        let mut state = State::new(&self.definition)?;
        let mut rows = vec![Observation {
            phase: Phase::Initial,
            command_index: None,
            command: None,
            state: state.clone(),
            interval: None,
        }];
        for (index, c) in self.commands.iter().enumerate() {
            let steps = (c.duration_s / self.observation_interval_s).ceil().max(1.) as usize;
            let mut previous_target = 0.;
            for step in 0..steps {
                let target = ((step + 1) as f64 * self.observation_interval_s).min(c.duration_s);
                let before = state.clone();
                let sub = Command {
                    duration_s: target - previous_target,
                    ..c.clone()
                };
                let mut interval = state.apply(&self.definition, &sub)?;
                if interval.powered_duration_s > 0.
                    && interval.fuel_limited_duration_s > 0.
                    && state.propellant_kg == 0.
                {
                    let mut cutoff = state.clone();
                    cutoff.elapsed_s = before.elapsed_s + interval.powered_duration_s;
                    cutoff.fuel_limited_duration_s = before.fuel_limited_duration_s;
                    let mut active = interval.clone();
                    active.duration_s = interval.powered_duration_s;
                    active.fuel_limited_duration_s = 0.;
                    active.requested_propellant_kg = active.actual_propellant_kg;
                    rows.push(Observation {
                        phase: Phase::FuelDepletion,
                        command_index: Some(index),
                        command: Some(c.clone()),
                        state: cutoff,
                        interval: Some(active),
                    });
                    interval = Interval {
                        duration_s: interval.fuel_limited_duration_s,
                        initial_mass_kg: interval.final_mass_kg,
                        final_mass_kg: interval.final_mass_kg,
                        commanded_thrust_n: interval.commanded_thrust_n,
                        requested_propellant_kg: interval.commanded_thrust_n
                            / self.definition.exhaust_velocity_m_s()
                            * interval.fuel_limited_duration_s,
                        actual_propellant_kg: 0.,
                        impulse_n_s: 0.,
                        ideal_delta_v_m_s: 0.,
                        powered_duration_s: 0.,
                        fuel_limited_duration_s: interval.fuel_limited_duration_s,
                        initial_powered_acceleration_m_s2: None,
                        final_powered_acceleration_m_s2: None,
                        mass_residual_kg: 0.,
                    };
                }
                let boundary = before.propellant_kg > 0.
                    && state.propellant_kg == 0.
                    && interval.powered_duration_s > 0.;
                rows.push(Observation {
                    phase: if boundary {
                        Phase::FuelDepletion
                    } else if step + 1 == steps {
                        Phase::CommandEnd
                    } else {
                        Phase::Sample
                    },
                    command_index: Some(index),
                    command: Some(c.clone()),
                    state: state.clone(),
                    interval: Some(interval),
                });
                previous_target = target;
            }
        }
        Ok(rows)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub elapsed_s: f64,
    pub propellant_kg: f64,
    pub exhausted_propellant_kg: f64,
    pub cumulative_impulse_n_s: f64,
    pub cumulative_ideal_delta_v_m_s: f64,
    pub velocity_change_m_s: [f64; 3],
    pub powered_duration_s: f64,
    pub fuel_limited_duration_s: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub duration_s: f64,
    pub initial_mass_kg: f64,
    pub final_mass_kg: f64,
    pub commanded_thrust_n: f64,
    pub requested_propellant_kg: f64,
    pub actual_propellant_kg: f64,
    pub impulse_n_s: f64,
    pub ideal_delta_v_m_s: f64,
    pub powered_duration_s: f64,
    pub fuel_limited_duration_s: f64,
    pub initial_powered_acceleration_m_s2: Option<f64>,
    pub final_powered_acceleration_m_s2: Option<f64>,
    pub mass_residual_kg: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub phase: Phase,
    pub command_index: Option<usize>,
    pub command: Option<Command>,
    pub state: State,
    pub interval: Option<Interval>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Initial,
    Sample,
    CommandEnd,
    FuelDepletion,
}
impl Phase {
    pub fn key(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Sample => "sample",
            Self::CommandEnd => "command_end",
            Self::FuelDepletion => "fuel_depletion",
        }
    }
}
fn default_observation_interval() -> f64 {
    1.
}
impl State {
    fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            elapsed_s: 0.,
            propellant_kg: d.initial_propellant_kg,
            exhausted_propellant_kg: 0.,
            cumulative_impulse_n_s: 0.,
            cumulative_ideal_delta_v_m_s: 0.,
            velocity_change_m_s: [0.; 3],
            powered_duration_s: 0.,
            fuel_limited_duration_s: 0.,
        })
    }
    pub fn mass_kg(&self, d: &Definition) -> f64 {
        d.inert_mass_kg() + self.propellant_kg
    }
    fn apply(&mut self, d: &Definition, c: &Command) -> Result<Interval, String> {
        d.validate()?;
        c.validate()?;
        // Reject invalid retained state before any mutation, including non-finite counters.
        bounds(
            self.propellant_kg,
            0.,
            d.initial_propellant_kg,
            "Remaining propellant",
        )?;
        for v in [
            self.elapsed_s,
            self.exhausted_propellant_kg,
            self.cumulative_impulse_n_s,
            self.cumulative_ideal_delta_v_m_s,
            self.powered_duration_s,
            self.fuel_limited_duration_s,
        ] {
            if !v.is_finite() || v < 0. {
                return Err("Unsupported retained propulsion state".into());
            }
        }
        if self.velocity_change_m_s.iter().any(|v| !v.is_finite()) {
            return Err("Unsupported retained velocity change".into());
        }
        let initial_mass = self.mass_kg(d);
        let thrust = d.rated_thrust_n * c.throttle;
        let rate = thrust / d.exhaust_velocity_m_s();
        let requested = rate * c.duration_s;
        let remaining = (self.propellant_kg - requested.min(self.propellant_kg)).max(0.);
        // The represented inventory difference is authoritative; never credit unremoved fuel.
        let used = self.propellant_kg - remaining;
        if requested > 0. && self.propellant_kg > 0. && used == 0. {
            return Err("Requested burn is below the represented propellant inventory precision; increase duration or reduce the held mass scale".into());
        }
        let powered = if rate > 0. {
            c.duration_s.min(self.propellant_kg / rate)
        } else {
            0.
        };
        let limited = if rate > 0. {
            (c.duration_s - powered).max(0.)
        } else {
            0.
        };
        let final_mass = d.inert_mass_kg() + remaining;
        let impulse = used * d.exhaust_velocity_m_s();
        let delta_v = if used > 0. {
            d.exhaust_velocity_m_s() * (used / final_mass).dln1p()
        } else {
            0.
        };
        self.elapsed_s += c.duration_s;
        self.propellant_kg = remaining;
        self.exhausted_propellant_kg += used;
        self.cumulative_impulse_n_s += impulse;
        self.cumulative_ideal_delta_v_m_s += delta_v;
        for i in 0..3 {
            self.velocity_change_m_s[i] += c.direction[i] * delta_v;
        }
        self.powered_duration_s += powered;
        self.fuel_limited_duration_s += limited;
        Ok(Interval {
            duration_s: c.duration_s,
            initial_mass_kg: initial_mass,
            final_mass_kg: final_mass,
            commanded_thrust_n: thrust,
            requested_propellant_kg: requested,
            actual_propellant_kg: used,
            impulse_n_s: impulse,
            ideal_delta_v_m_s: delta_v,
            powered_duration_s: powered,
            fuel_limited_duration_s: limited,
            initial_powered_acceleration_m_s2: (powered > 0.).then_some(thrust / initial_mass),
            final_powered_acceleration_m_s2: (powered > 0.).then_some(thrust / final_mass),
            mass_residual_kg: initial_mass - final_mass - used,
        })
    }
}
pub fn context() -> Value {
    json!({"model":MODEL,"g0_m_s2":G0,"limits":LIMITS,"sources":[
        {"url":"https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/specific-impulse/","role":"F=mdot*Isp*g0, finite propellant mass flow"},
        {"url":"https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/ideal-rocket-equation/","role":"Ideal constant-Isp variable-mass velocity change, no gravity/aerodynamics"},
        {"url":"https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9","role":"Exact conventional standard acceleration g0=9.80665 m/s², not local gravity"}],"inputs":"All design masses, capacities, thrust, Isp and commands are explicit user assumptions. Cargo stock names match nominal habitat bookkeeping but no supply delivery occurs."})
}
fn bounds(value: f64, low: f64, high: f64, label: &str) -> Result<(), String> {
    if value.is_finite() && value >= low && value <= high {
        Ok(())
    } else {
        Err(format!("{label} must be finite within {low}–{high}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_thrust_coast_and_aligned_depletion_have_honest_support() {
        let mut p = Plan::example();
        p.definition.rated_thrust_n = 0.;
        let rows = p.run().unwrap();
        assert!(rows.iter().all(
            |r| r.state.propellant_kg == p.definition.initial_propellant_kg
                && r.state.cumulative_impulse_n_s == 0.
                && r.state.powered_duration_s == 0.
                && r.state.fuel_limited_duration_s == 0.
        ));
        p = Plan::example();
        p.definition.initial_propellant_kg =
            p.definition.rated_thrust_n / p.definition.exhaust_velocity_m_s();
        p.commands = vec![Command {
            duration_s: 1.,
            throttle: 1.,
            direction: [1., 0., 0.],
        }];
        let rows = p.run().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].phase, Phase::FuelDepletion);
        assert_eq!(rows[1].state.elapsed_s, 1.);
        assert_eq!(rows[1].state.propellant_kg, 0.);
        assert_eq!(rows[1].state.fuel_limited_duration_s, 0.);
        p.commands[0].duration_s = 0.;
        let rows = p.run().unwrap();
        assert_eq!(rows[1].state, rows[0].state);
        assert!(rows[1]
            .interval
            .as_ref()
            .unwrap()
            .initial_powered_acceleration_m_s2
            .is_none());
    }
    #[test]
    fn sampling_retains_depletion_boundary_and_small_mass_ratios_without_fake_shortage() {
        let plan = Plan::example();
        let rows = plan.run().unwrap();
        let point = rows
            .iter()
            .find(|r| r.phase == Phase::FuelDepletion)
            .unwrap();
        let expected = plan.definition.initial_propellant_kg
            * plan.definition.exhaust_velocity_m_s()
            / plan.definition.rated_thrust_n
            + 60.;
        assert!((point.state.elapsed_s - expected).abs() < 1e-8);
        assert_eq!(point.state.fuel_limited_duration_s, 0.);
        assert!(rows.last().unwrap().state.fuel_limited_duration_s > 0.);
        let mut coarse = plan.clone();
        coarse.observation_interval_s = 30.;
        let other = coarse.run().unwrap();
        assert!(
            (other.last().unwrap().state.cumulative_ideal_delta_v_m_s
                - rows.last().unwrap().state.cumulative_ideal_delta_v_m_s)
                .abs()
                < 1e-8
        );
        let mut d = Definition::example();
        d.dry_mass_kg = 1e12;
        d.initial_propellant_kg = 1.;
        d.cargo = Cargo {
            oxygen_reserve_kg: 0.,
            potable_water_kg: 0.,
            food_dry_kg: 0.,
            food_water_kg: 0.,
        };
        let mut state = State::new(&d).unwrap();
        let interval = state
            .apply(
                &d,
                &Command {
                    duration_s: 0.001,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                },
            )
            .unwrap();
        assert!(interval.ideal_delta_v_m_s > 0.);
        assert_eq!(interval.fuel_limited_duration_s, 0.);
        d.initial_propellant_kg = 1e12;
        d.propellant_capacity_kg = 1e12;
        d.rated_thrust_n = 1e-3;
        d.effective_isp_s = 1e5;
        let mut state = State::new(&d).unwrap();
        let before = state.clone();
        assert!(state
            .apply(
                &d,
                &Command {
                    duration_s: 1.,
                    throttle: 1.,
                    direction: [1., 0., 0.]
                }
            )
            .is_err());
        assert_eq!(state, before);
        let mut unbounded = plan;
        unbounded.observation_interval_s = 0.001;
        assert!(unbounded.run().is_err());
    }
    #[test]
    fn actual_fuel_depletion_retains_cargo_and_matches_ideal_capability() {
        let p = Plan::example();
        let rows = p.run().unwrap();
        let last = rows.last().unwrap();
        assert_eq!(last.state.propellant_kg, 0.);
        // Independently accumulated interval exhaust retains its floating residual.
        // Same-plan replay stays exact; the analytic mass identity is numerical.
        assert!(
            (last.state.exhausted_propellant_kg - p.definition.initial_propellant_kg).abs() < 1e-9
        );
        assert!(last.state.fuel_limited_duration_s > 0.);
        assert_eq!(
            last.state.mass_kg(&p.definition),
            p.definition.inert_mass_kg()
        );
        assert!(
            (last.state.cumulative_ideal_delta_v_m_s - p.definition.initial_ideal_delta_v_m_s())
                .abs()
                < 1e-10
        );
        assert!(
            (last.state.cumulative_impulse_n_s
                - p.definition.initial_propellant_kg * p.definition.exhaust_velocity_m_s())
            .abs()
                < 1e-8
        );
        let burn_end = &rows
            .iter()
            .rfind(|r| r.command_index == Some(0))
            .unwrap()
            .state;
        let coast_end = &rows
            .iter()
            .rfind(|r| r.command_index == Some(1))
            .unwrap()
            .state;
        assert_eq!(burn_end.propellant_kg, coast_end.propellant_kg);
        assert_eq!(burn_end.velocity_change_m_s, coast_end.velocity_change_m_s);
        assert_eq!(
            rows.iter()
                .filter(|r| r.phase == Phase::FuelDepletion)
                .count(),
            1
        );
        assert_eq!(p.run().unwrap(), rows);
        let mut state = last.state.clone();
        let v = state.velocity_change_m_s;
        let i = state
            .apply(
                &p.definition,
                &Command {
                    duration_s: 100.,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                },
            )
            .unwrap();
        assert_eq!(i.actual_propellant_kg, 0.);
        assert_eq!(i.impulse_n_s, 0.);
        assert_eq!(i.fuel_limited_duration_s, 100.);
        assert_eq!(state.velocity_change_m_s, v);
        assert!(i.initial_powered_acceleration_m_s2.is_none());
    }
    #[test]
    fn invalid_commands_are_atomic_and_payload_reduces_capability() {
        let d = Definition::example();
        let mut state = State::new(&d).unwrap();
        let before = state.clone();
        for c in [
            Command {
                duration_s: -1.,
                throttle: 1.,
                direction: [1., 0., 0.],
            },
            Command {
                duration_s: 100.,
                throttle: f64::NAN,
                direction: [1., 0., 0.],
            },
            Command {
                duration_s: 100.,
                throttle: 1.,
                direction: [0.; 3],
            },
        ] {
            assert!(state.apply(&d, &c).is_err());
            assert_eq!(state, before);
        }
        let mut light = d.clone();
        light.cargo = Cargo {
            oxygen_reserve_kg: 0.,
            potable_water_kg: 0.,
            food_dry_kg: 0.,
            food_water_kg: 0.,
        };
        assert!(light.initial_ideal_delta_v_m_s() > d.initial_ideal_delta_v_m_s());
        let mut invalid = d.clone();
        invalid.initial_propellant_kg = invalid.propellant_capacity_kg + 1.;
        assert!(State::new(&invalid).is_err());
        invalid = d.clone();
        invalid.payload_capacity_kg = 0.;
        assert!(State::new(&invalid).is_err());
    }
    #[test]
    fn opposing_burns_track_vector_velocity_separately_from_expended_capability() {
        let d = Definition::example();
        let mut state = State::new(&d).unwrap();
        let first = state
            .apply(
                &d,
                &Command {
                    duration_s: 30.,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                },
            )
            .unwrap();
        let second = state
            .apply(
                &d,
                &Command {
                    duration_s: 30.,
                    throttle: 1.,
                    direction: [-1., 0., 0.],
                },
            )
            .unwrap();
        assert!(second.ideal_delta_v_m_s > first.ideal_delta_v_m_s);
        assert!(state.velocity_change_m_s[0] < 0.);
        assert_eq!(
            state.cumulative_ideal_delta_v_m_s,
            first.ideal_delta_v_m_s + second.ideal_delta_v_m_s
        );
        assert!(
            (state.exhausted_propellant_kg + state.propellant_kg - d.initial_propellant_kg).abs()
                < 1e-10
        );
    }
}

impl Plan {
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let plan: Self =
            serde_json::from_slice(&read_bounded(path, 1024 * 1024)?).map_err(|e| e.to_string())?;
        plan.validate()?;
        Ok(plan)
    }
    pub fn write(&self, path: &std::path::Path) -> Result<(), String> {
        use std::io::Write;
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Vehicle definitions are protected: {e}"))?;
        file.write_all(&bytes).map_err(|e| e.to_string())
    }
    pub fn summary(&self) -> Result<Value, String> {
        let rows = self.run()?;
        Ok(self.summary_from(&rows))
    }
    fn summary_from(&self, rows: &[Observation]) -> Value {
        let last = &rows
            .last()
            .expect("Validated plans retain an initial observation")
            .state;
        json!({"format":"cosmogon-supply-vehicle-result","schema_version":1,"model":MODEL,"name":self.definition.name,"command_count":self.commands.len(),"initial_mass_kg":self.definition.inert_mass_kg()+self.definition.initial_propellant_kg,"initial_ideal_delta_v_m_s":self.definition.initial_ideal_delta_v_m_s(),"retained_cargo":self.definition.cargo,"final_state":last,"final_mass_kg":last.mass_kg(&self.definition),"mass_residual_kg":self.definition.initial_propellant_kg-last.propellant_kg-last.exhausted_propellant_kg,"context":context()})
    }
    fn manifest_from(&self, rows: &[Observation]) -> Value {
        json!({"format":"cosmogon-supply-vehicle-recording","schema_version":1,"complete":true,"plan":self,"summary":self.summary_from(rows),"source_context":context(),"observations":rows.len(),"columns":24})
    }
    pub fn export(&self, path: &std::path::Path) -> Result<Value, String> {
        use std::io::Write;
        self.validate()?;
        let rows = self.run()?;
        let csv = trajectory_csv(&self.definition, &rows);
        let manifest = self.manifest_from(&rows);
        let files = [
            (
                "plan.json",
                serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
            ),
            (
                "observations.json",
                serde_json::to_vec_pretty(&rows).map_err(|e| e.to_string())?,
            ),
            ("trajectories.csv", csv.into_bytes()),
            (
                "manifest.json",
                serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
            ),
        ];
        if files
            .iter()
            .any(|(_, bytes)| bytes.len() > 32 * 1024 * 1024)
        {
            return Err("Vehicle export exceeds bounded file size".into());
        }
        if path.exists() {
            return Err("Existing vehicle recordings are protected; choose a new folder".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir(path).map_err(|e| e.to_string())?;
        for (name, bytes) in files {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(name))
                .map_err(|e| e.to_string())?;
            f.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        Ok(manifest)
    }
}
#[derive(Clone, Debug)]
pub struct Bundle {
    pub plan: Plan,
    pub rows: Vec<Observation>,
    pub manifest: Value,
}
impl Bundle {
    pub fn run(plan: Plan) -> Result<Self, String> {
        let rows = plan.run()?;
        let manifest = plan.manifest_from(&rows);
        Ok(Self {
            plan,
            rows,
            manifest,
        })
    }
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let manifest: Value =
            serde_json::from_slice(&read_bounded(&path.join("manifest.json"), 1024 * 1024)?)
                .map_err(|e| e.to_string())?;
        if manifest["format"] != "cosmogon-supply-vehicle-recording"
            || manifest["schema_version"] != 1
            || manifest["complete"] != true
        {
            return Err("Vehicle recording is incomplete or unsupported".into());
        }
        let plan = Plan::read(&path.join("plan.json"))?;
        let expected = plan.run()?;
        if manifest != plan.manifest_from(&expected) {
            return Err("Vehicle recording plan, sources or result differs".into());
        }
        let rows: Vec<Observation> = serde_json::from_slice(&read_bounded(
            &path.join("observations.json"),
            32 * 1024 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        if serde_json::to_vec(&rows).map_err(|e| e.to_string())?
            != serde_json::to_vec(&expected).map_err(|e| e.to_string())?
        {
            return Err(
                "Stored vehicle trajectory differs from an independent exact replay".into(),
            );
        }
        if read_bounded(&path.join("trajectories.csv"), 32 * 1024 * 1024)?
            != trajectory_csv(&plan.definition, &expected).into_bytes()
        {
            return Err("Stored vehicle CSV differs from its exact trajectory".into());
        }
        Ok(Self {
            plan,
            rows,
            manifest,
        })
    }
}
pub fn read_recording(path: &std::path::Path) -> Result<Value, String> {
    Ok(Bundle::read(path)?.manifest)
}
fn read_bounded(path: &std::path::Path, limit: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Vehicle input exceeds bounded file size".into());
    }
    Ok(bytes)
}

fn trajectory_csv(definition: &Definition, rows: &[Observation]) -> String {
    let mut csv=String::from("observation_index,phase,command_index,elapsed_s,vehicle_mass_kg,held_propellant_kg,exhausted_propellant_kg,cumulative_impulse_n_s,cumulative_ideal_delta_v_m_s,velocity_change_x_m_s,velocity_change_y_m_s,velocity_change_z_m_s,powered_duration_s,fuel_limited_duration_s,interval_propellant_kg,interval_impulse_n_s,interval_ideal_delta_v_m_s,interval_powered_s,interval_fuel_limited_s,commanded_thrust_n,initial_powered_acceleration_m_s2,final_powered_acceleration_m_s2,mass_residual_kg,interval_duration_s\n");
    for (index, row) in rows.iter().enumerate() {
        let s = &row.state;
        let mut fields = vec![
            index.to_string(),
            row.phase.key().into(),
            row.command_index.map(|v| v.to_string()).unwrap_or_default(),
            s.elapsed_s.to_string(),
            s.mass_kg(definition).to_string(),
            s.propellant_kg.to_string(),
            s.exhausted_propellant_kg.to_string(),
            s.cumulative_impulse_n_s.to_string(),
            s.cumulative_ideal_delta_v_m_s.to_string(),
            s.velocity_change_m_s[0].to_string(),
            s.velocity_change_m_s[1].to_string(),
            s.velocity_change_m_s[2].to_string(),
            s.powered_duration_s.to_string(),
            s.fuel_limited_duration_s.to_string(),
        ];
        if let Some(i) = &row.interval {
            fields.extend([
                i.actual_propellant_kg.to_string(),
                i.impulse_n_s.to_string(),
                i.ideal_delta_v_m_s.to_string(),
                i.powered_duration_s.to_string(),
                i.fuel_limited_duration_s.to_string(),
                i.commanded_thrust_n.to_string(),
                i.initial_powered_acceleration_m_s2
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                i.final_powered_acceleration_m_s2
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                i.mass_residual_kg.to_string(),
                i.duration_s.to_string(),
            ]);
        } else {
            fields.extend(std::iter::repeat_n(String::new(), 10));
        }
        csv.push_str(&fields.join(","));
        csv.push('\n');
    }
    csv
}

#[cfg(test)]
mod recording_tests {
    use super::*;
    #[test]
    fn protected_recording_reopens_full_numbers_and_detects_altered_csv() {
        let root =
            std::env::temp_dir().join(format!("cosmogon-vehicle-recording-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let plan = Plan::example();
        let manifest = plan.export(&root).unwrap();
        assert_eq!(read_recording(&root).unwrap(), manifest);
        let before = std::fs::read(root.join("observations.json")).unwrap();
        assert!(plan.export(&root).is_err());
        assert_eq!(
            std::fs::read(root.join("observations.json")).unwrap(),
            before
        );
        std::fs::write(root.join("trajectories.csv"), "tampered").unwrap();
        assert!(read_recording(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn reopening_refuses_incomplete_source_and_numeric_changes_without_replacing_files() {
        let root =
            std::env::temp_dir().join(format!("cosmogon-vehicle-strict-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let p = Plan::example();
        p.export(&root).unwrap();
        let original = std::fs::read(root.join("manifest.json")).unwrap();
        for (key, value) in [
            ("complete", json!(false)),
            ("source_context", json!({"model":"unavailable"})),
            ("extra_unknown", json!(true)),
        ] {
            let mut v: Value = serde_json::from_slice(&original).unwrap();
            v[key] = value;
            std::fs::write(root.join("manifest.json"), serde_json::to_vec(&v).unwrap()).unwrap();
            assert!(Bundle::read(&root).is_err());
        }
        std::fs::write(root.join("manifest.json"), original).unwrap();
        let original_rows = std::fs::read(root.join("observations.json")).unwrap();
        let mut rows: Value = serde_json::from_slice(&original_rows).unwrap();
        rows[1]["state"]["velocity_change_m_s"][0] = json!(999.);
        std::fs::write(
            root.join("observations.json"),
            serde_json::to_vec(&rows).unwrap(),
        )
        .unwrap();
        assert!(Bundle::read(&root).is_err());
        std::fs::write(root.join("observations.json"), original_rows).unwrap();
        assert_eq!(Bundle::read(&root).unwrap().rows, p.run().unwrap());
        p.write(&root.join("new-plan.json")).unwrap();
        assert!(p.write(&root.join("new-plan.json")).is_err());
        assert_eq!(Plan::read(&root.join("new-plan.json")).unwrap(), p);
        std::fs::remove_dir_all(root).unwrap();
    }
}
