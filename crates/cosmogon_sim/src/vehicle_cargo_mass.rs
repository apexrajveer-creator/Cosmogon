//! Finite cargo removal and fuel-driven mass epochs; no world delivery permission.
use crate::{
    supply_vehicle::{Cargo, Command, Definition, Interval, State},
    Vec3d,
};
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};
pub const MODEL: &str = "cosmogon-vehicle-cargo-mass-epochs/1";
const MAX_EVENTS: usize = 1024;
fn quantities(c: &Cargo) -> [f64; 4] {
    [
        c.oxygen_reserve_kg,
        c.potable_water_kg,
        c.food_dry_kg,
        c.food_water_kg,
    ]
}
fn cargo(v: [f64; 4]) -> Cargo {
    Cargo {
        oxygen_reserve_kg: v[0],
        potable_water_kg: v[1],
        food_dry_kg: v[2],
        food_water_kg: v[3],
    }
}
fn finite_cargo(c: &Cargo) -> bool {
    quantities(c)
        .iter()
        .all(|v| v.is_finite() && (0.0..=1e9).contains(v))
}
fn close(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-8 * b.abs().max(1.)
}
/// A central, comoving material event; removal does not create a propulsion impulse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Removal {
    pub elapsed_s: f64,
    pub propellant_kg: f64,
    pub requested: Cargo,
    pub actual_removed: Cargo,
    pub before: Cargo,
    pub after: Cargo,
    pub initial_mass_kg: f64,
    pub final_mass_kg: f64,
    pub mass_residual_kg: f64,
    pub velocity_m_s: Vec3d,
    pub carried_linear_momentum_kg_m_s: Vec3d,
    pub carried_kinetic_energy_j: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub model: String,
    pub initial_definition: Definition,
    pub held: Cargo,
    pub events: Vec<Removal>,
}
fn split(held: &Cargo, requested: &Cargo) -> Result<(Cargo, Cargo), String> {
    if !finite_cargo(held) || !finite_cargo(requested) {
        return Err(
            "Cargo inventories must contain finite nonnegative supported quantities".into(),
        );
    }
    let mut after = [0.; 4];
    let mut removed = [0.; 4];
    for (i, (available, request)) in quantities(held)
        .into_iter()
        .zip(quantities(requested))
        .enumerate()
    {
        if request > available {
            return Err("Cargo removal overdraws a held species".into());
        }
        let mut remainder = available - request;
        // Round toward retained stock so a capacity/rate-limited request is never exceeded.
        if available - remainder > request {
            remainder = remainder.next_up();
        }
        after[i] = remainder;
        removed[i] = available - remainder;
        if request > 0. && removed[i] <= 0. {
            return Err("Cargo request is below the represented inventory precision".into());
        }
        if removed[i] < 0. || removed[i] > request {
            return Err("Cargo removal lacks conservative represented support".into());
        }
    }
    if removed.iter().all(|v| *v == 0.) {
        return Err("Cargo removal requires positive actual material".into());
    }
    Ok((cargo(after), cargo(removed)))
}
impl Ledger {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            model: MODEL.into(),
            initial_definition: d.clone(),
            held: d.cargo.clone(),
            events: vec![],
        })
    }
    pub fn mass_kg(&self, d: &Definition, p: &State) -> f64 {
        d.dry_mass_kg + self.held.mass_kg() + p.propellant_kg
    }
    /// Counterfactual future burn inputs; the immutable initial definition stays intact.
    pub fn current_definition(&self, d: &Definition) -> Result<Definition, String> {
        d.validate()?;
        if self.model != MODEL || self.initial_definition != *d || !finite_cargo(&self.held) {
            return Err("Unsupported cargo mass ledger".into());
        }
        let mut effective = d.clone();
        effective.cargo = self.held.clone();
        effective.validate()?;
        Ok(effective)
    }
    pub fn expected_ideal_delta_v_m_s(&self, d: &Definition, p: &State) -> Result<f64, String> {
        d.validate()?;
        if !p.elapsed_s.is_finite() || p.elapsed_s < 0. {
            return Err("Cargo epochs require a finite nonnegative propulsion clock".into());
        }
        if self.model != MODEL
            || self.initial_definition != *d
            || self.events.len() > MAX_EVENTS
            || !finite_cargo(&self.held)
        {
            return Err("Unsupported cargo mass ledger".into());
        }
        let mut held = d.cargo.clone();
        let mut fuel = d.initial_propellant_kg;
        let mut elapsed = 0.;
        let mut capability = 0.;
        for event in &self.events {
            if !event.elapsed_s.is_finite()
                || event.elapsed_s < elapsed
                || event.elapsed_s > p.elapsed_s
                || !event.propellant_kg.is_finite()
                || event.propellant_kg < 0.
                || event.propellant_kg > fuel
                || event.before != held
            {
                return Err(
                    "Cargo events are out of order or differ from actual held/fuel history".into(),
                );
            }
            let (after, actual) = split(&held, &event.requested)?;
            let initial_mass = d.dry_mass_kg + held.mass_kg() + event.propellant_kg;
            let final_mass = d.dry_mass_kg + after.mass_kg() + event.propellant_kg;
            let v = event.velocity_m_s;
            if final_mass >= initial_mass
                || ![v.x, v.y, v.z].iter().all(|x| x.is_finite())
                || v.length() > 1e6
                || event.after != after
                || event.actual_removed != actual
                || event.initial_mass_kg != initial_mass
                || event.final_mass_kg != final_mass
                || event.mass_residual_kg != initial_mass - final_mass - actual.mass_kg()
                || event.carried_linear_momentum_kg_m_s != v * actual.mass_kg()
                || event.carried_kinetic_energy_j != 0.5 * actual.mass_kg() * v.length_squared()
            {
                return Err(
                    "Cargo event material, mass or comoving momentum/energy evidence differs"
                        .into(),
                );
            }
            capability += d.exhaust_velocity_m_s()
                * ((fuel - event.propellant_kg)
                    / (d.dry_mass_kg + held.mass_kg() + event.propellant_kg))
                    .dln1p();
            fuel = event.propellant_kg;
            elapsed = event.elapsed_s;
            held = after;
        }
        if held != self.held
            || !p.propellant_kg.is_finite()
            || p.propellant_kg < 0.
            || p.propellant_kg > fuel
        {
            return Err("Current held cargo/fuel differs from its mass epochs".into());
        }
        capability += d.exhaust_velocity_m_s()
            * ((fuel - p.propellant_kg) / (d.dry_mass_kg + held.mass_kg() + p.propellant_kg))
                .dln1p();
        Ok(capability)
    }
    pub fn validate(&self, d: &Definition, p: &State) -> Result<(), String> {
        let capability = self.expected_ideal_delta_v_m_s(d, p)?;
        let v = Vec3d::new(
            p.velocity_change_m_s[0],
            p.velocity_change_m_s[1],
            p.velocity_change_m_s[2],
        );
        if [
            p.elapsed_s,
            p.propellant_kg,
            p.exhausted_propellant_kg,
            p.cumulative_impulse_n_s,
            p.cumulative_ideal_delta_v_m_s,
            p.powered_duration_s,
            p.fuel_limited_duration_s,
        ]
        .iter()
        .any(|x| !x.is_finite() || *x < 0.)
            || ![v.x, v.y, v.z].iter().all(|x| x.is_finite())
            || !close(
                p.propellant_kg + p.exhausted_propellant_kg,
                d.initial_propellant_kg,
            )
            || !close(
                p.cumulative_impulse_n_s,
                p.exhausted_propellant_kg * d.exhaust_velocity_m_s(),
            )
            || !close(p.cumulative_ideal_delta_v_m_s, capability)
            || v.length() > capability + 1e-8 * capability.max(1.)
            || p.powered_duration_s + p.fuel_limited_duration_s
                > p.elapsed_s + 1e-8 * p.elapsed_s.max(1.)
        {
            return Err(
                "Propulsion counters do not close against finite fuel and cargo mass epochs".into(),
            );
        }
        Ok(())
    }
    pub fn apply_burn(
        &self,
        d: &Definition,
        p: &mut State,
        c: &Command,
    ) -> Result<Interval, String> {
        self.validate(d, p)?;
        let mut next = p.clone();
        let interval = next.apply(&self.current_definition(d)?, c)?;
        self.validate(d, &next)?;
        *p = next;
        Ok(interval)
    }
    pub fn remove(
        &mut self,
        d: &Definition,
        p: &State,
        requested: Cargo,
        velocity_m_s: Vec3d,
    ) -> Result<Removal, String> {
        self.validate(d, p)?;
        if self.events.len() >= MAX_EVENTS {
            return Err("Cargo event ledger is full; no material is removed".into());
        }
        let (after, actual_removed) = split(&self.held, &requested)?;
        if ![velocity_m_s.x, velocity_m_s.y, velocity_m_s.z]
            .iter()
            .all(|v| v.is_finite())
            || velocity_m_s.length() > 1e6
        {
            return Err("Cargo removal velocity lacks finite system-local support".into());
        }
        let initial_mass = self.mass_kg(d, p);
        let final_mass = d.dry_mass_kg + after.mass_kg() + p.propellant_kg;
        if final_mass >= initial_mass {
            return Err("Cargo removal is below represented total vehicle mass precision".into());
        }
        let event = Removal {
            elapsed_s: p.elapsed_s,
            propellant_kg: p.propellant_kg,
            requested,
            actual_removed: actual_removed.clone(),
            before: self.held.clone(),
            after: after.clone(),
            initial_mass_kg: initial_mass,
            final_mass_kg: final_mass,
            mass_residual_kg: initial_mass - final_mass - actual_removed.mass_kg(),
            velocity_m_s,
            carried_linear_momentum_kg_m_s: velocity_m_s * actual_removed.mass_kg(),
            carried_kinetic_energy_j: 0.5
                * actual_removed.mass_kg()
                * velocity_m_s.length_squared(),
        };
        let mut next = self.clone();
        next.held = after;
        next.events.push(event.clone());
        next.validate(d, p)?;
        *self = next;
        Ok(event)
    }
}
/// Scope is intentionally narrower than authoritative flight/delivery integration.
pub fn context() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"data_state":"derived-simulation","scope":"Separate finite held cargo and piecewise ideal propulsion accounting; no world transfer or landing permission","sources":["https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/ideal-rocket-equation/","https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/specific-impulse/"],"removal":"Central comoving material; carried linear momentum and translational kinetic energy, no free impulse or exhausted propellant","limits":["No angular-momentum/inertia or rotational-energy transfer model","No recipient stock, chemistry, sensible heat or handling equipment","Authoritative support, co-location, finite storage/power and habitat exchange ledgers must precede world delivery"],"maximum_events":MAX_EVENTS,"empirical_validation":null,"uncertainty":null})
}

pub const SEQUENCE_FORMAT: &str = "cosmogon-cargo-epochs-plan";
pub const SEQUENCE_SCHEMA: u32 = 1;
pub const SEQUENCE_LIMITS: &str = "Authored free-space burn/removal sequence bench over the immutable vehicle definition. Cargo removal is central and comoving: it carries material linear momentum and translational kinetic energy, grants no rocket impulse and never exhausts propellant. Prior burns retain their original mass denominators; later burns use current held cargo. No world delivery, recipient stock, handling equipment, angular-momentum transfer, gravity, attitude dynamics, mission feasibility or hardware calibration is implied. Connecting plot segments are display guides, not continuous inferred data.";
pub const MAX_STEPS: usize = 256;
pub const MAX_OBSERVATIONS: usize = 8192;

/// One authored finite species-removal request at the live propulsion clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemovalRequest {
    pub requested: Cargo,
    pub velocity_m_s: [f64; 3],
}
/// An ordered sequence step: one finite burn command or one finite species removal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Step {
    Burn(Command),
    RemoveCargo(RemovalRequest),
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Initial,
    Sample,
    BurnEnd,
    FuelDepletion,
    Removal,
}
impl Phase {
    pub fn key(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Sample => "sample",
            Self::BurnEnd => "burn_end",
            Self::FuelDepletion => "fuel_depletion",
            Self::Removal => "removal",
        }
    }
}
/// One retained sequence observation with the exact held cargo at its clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub phase: Phase,
    pub step_index: Option<usize>,
    pub step: Option<Step>,
    pub state: State,
    pub held: Cargo,
    pub vehicle_mass_kg: f64,
    pub burn: Option<crate::supply_vehicle::Interval>,
    pub removal: Option<Removal>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub format: String,
    pub schema_version: u32,
    pub definition: Definition,
    pub steps: Vec<Step>,
    #[serde(default = "default_observation_interval")]
    pub observation_interval_s: f64,
}
fn default_observation_interval() -> f64 {
    1.
}
impl Plan {
    pub fn example() -> Self {
        Self {
            format: SEQUENCE_FORMAT.into(),
            schema_version: SEQUENCE_SCHEMA,
            definition: Definition::example(),
            observation_interval_s: 30.,
            steps: vec![
                Step::Burn(Command {
                    duration_s: 150.,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                }),
                Step::RemoveCargo(RemovalRequest {
                    requested: cargo([0., 30., 0., 0.]),
                    velocity_m_s: [0., 0., 0.],
                }),
                Step::Burn(Command {
                    duration_s: 150.,
                    throttle: 1.,
                    direction: [1., 0., 0.],
                }),
                Step::RemoveCargo(RemovalRequest {
                    requested: cargo([5., 20., 5., 5.]),
                    velocity_m_s: [12., -3., 4.],
                }),
                Step::Burn(Command {
                    duration_s: 150.,
                    throttle: 1.,
                    direction: [-1., 0., 0.],
                }),
            ],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.format != SEQUENCE_FORMAT || self.schema_version != SEQUENCE_SCHEMA {
            return Err("Unsupported cargo-epochs plan".into());
        }
        self.definition.validate()?;
        if self.steps.is_empty() || self.steps.len() > MAX_STEPS {
            return Err("Cargo-epochs plans require 1–256 explicit steps".into());
        }
        if !self.observation_interval_s.is_finite()
            || !(1e-3..=1e7).contains(&self.observation_interval_s)
        {
            return Err("Cargo-epochs observation interval must be finite within 1e-3–1e7 s".into());
        }
        // Initial row plus at most one extra exact depletion boundary per burn sub-step.
        let mut observations = 1.;
        for step in &self.steps {
            match step {
                Step::Burn(c) => {
                    c.validate()?;
                    observations += (c.duration_s / self.observation_interval_s).ceil().max(1.) + 1.;
                }
                Step::RemoveCargo(r) => {
                    if !finite_cargo(&r.requested) {
                        return Err(
                            "Cargo requests must contain finite nonnegative supported quantities"
                                .into(),
                        );
                    }
                    if r.velocity_m_s.iter().any(|v| !v.is_finite()) || {
                        Vec3d::new(r.velocity_m_s[0], r.velocity_m_s[1], r.velocity_m_s[2])
                            .length()
                            > 1e6
                    } {
                        return Err(
                            "Cargo removal velocity lacks finite system-local support".into(),
                        );
                    }
                    observations += 1.;
                }
            }
        }
        if observations > MAX_OBSERVATIONS as f64 {
            return Err(
                "Cargo-epochs plans support at most 8192 actual observations; increase the interval or shorten burns".into(),
            );
        }
        Ok(())
    }
    pub fn run(&self) -> Result<Vec<Observation>, String> {
        self.validate()?;
        let mut ledger = Ledger::new(&self.definition)?;
        let mut state = State::new(&self.definition)?;
        let mut rows = vec![Observation {
            phase: Phase::Initial,
            step_index: None,
            step: None,
            state: state.clone(),
            held: ledger.held.clone(),
            vehicle_mass_kg: ledger.mass_kg(&self.definition, &state),
            burn: None,
            removal: None,
        }];
        for (index, step) in self.steps.iter().enumerate() {
            match step {
                Step::Burn(command) => {
                    let steps =
                        (command.duration_s / self.observation_interval_s).ceil().max(1.) as usize;
                    let mut previous_target = 0.;
                    for s in 0..steps {
                        let target =
                            ((s + 1) as f64 * self.observation_interval_s).min(command.duration_s);
                        let before = state.clone();
                        let sub = Command {
                            duration_s: target - previous_target,
                            ..command.clone()
                        };
                        let mut interval = ledger.apply_burn(&self.definition, &mut state, &sub)?;
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
                                step_index: Some(index),
                                step: Some(step.clone()),
                                state: cutoff.clone(),
                                held: ledger.held.clone(),
                                vehicle_mass_kg: ledger.mass_kg(&self.definition, &cutoff),
                                burn: Some(active),
                                removal: None,
                            });
                            interval = crate::supply_vehicle::Interval {
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
                            } else if s + 1 == steps {
                                Phase::BurnEnd
                            } else {
                                Phase::Sample
                            },
                            step_index: Some(index),
                            step: Some(step.clone()),
                            state: state.clone(),
                            held: ledger.held.clone(),
                            vehicle_mass_kg: ledger.mass_kg(&self.definition, &state),
                            burn: Some(interval),
                            removal: None,
                        });
                        previous_target = target;
                        if rows.len() > MAX_OBSERVATIONS {
                            return Err("Cargo-epochs run exceeded its observation budget".into());
                        }
                    }
                }
                Step::RemoveCargo(request) => {
                    let event = ledger.remove(
                        &self.definition,
                        &state,
                        request.requested.clone(),
                        Vec3d::new(
                            request.velocity_m_s[0],
                            request.velocity_m_s[1],
                            request.velocity_m_s[2],
                        ),
                    )?;
                    rows.push(Observation {
                        phase: Phase::Removal,
                        step_index: Some(index),
                        step: Some(step.clone()),
                        state: state.clone(),
                        held: ledger.held.clone(),
                        vehicle_mass_kg: ledger.mass_kg(&self.definition, &state),
                        burn: None,
                        removal: Some(event),
                    });
                }
            }
            if rows.len() > MAX_OBSERVATIONS {
                return Err("Cargo-epochs run exceeded its observation budget".into());
            }
        }
        Ok(rows)
    }
    pub fn summary(&self) -> Result<serde_json::Value, String> {
        let rows = self.run()?;
        Ok(self.summary_from(&rows))
    }
    fn summary_from(&self, rows: &[Observation]) -> serde_json::Value {
        let last = rows
            .last()
            .expect("Validated plans retain an initial observation");
        let epochs = rows
            .iter()
            .filter_map(|r| match r.phase {
                Phase::Initial => None,
                Phase::Removal => r.removal.as_ref().map(|e| {
                    serde_json::json!({"kind":"removal","step_index":r.step_index,"elapsed_s":e.elapsed_s,"requested":e.requested,"actual_removed":e.actual_removed,"held_after":e.after,"mass_residual_kg":e.mass_residual_kg,"carried_linear_momentum_kg_m_s":e.carried_linear_momentum_kg_m_s,"carried_kinetic_energy_j":e.carried_kinetic_energy_j})
                }),
                _ => (r.phase == Phase::BurnEnd || r.phase == Phase::FuelDepletion).then(|| {
                    let i = r.burn.as_ref().expect("burn phases retain an interval");
                    serde_json::json!({"kind":"burn","step_index":r.step_index,"phase":r.phase.key(),"elapsed_s":r.state.elapsed_s,"initial_mass_kg":i.initial_mass_kg,"final_mass_kg":i.final_mass_kg,"ideal_delta_v_m_s":i.ideal_delta_v_m_s,"actual_propellant_kg":i.actual_propellant_kg,"held":r.held,"vehicle_mass_kg":r.vehicle_mass_kg})
                }),
            })
            .collect::<Vec<_>>();
        serde_json::json!({"format":"cosmogon-cargo-epochs-result","schema_version":1,"model":MODEL,"name":self.definition.name,"step_count":self.steps.len(),"removal_count":rows.iter().filter(|r|r.phase==Phase::Removal).count(),"initial_mass_kg":self.definition.inert_mass_kg()+self.definition.initial_propellant_kg,"initial_held_cargo":self.definition.cargo,"final_held_cargo":last.held,"final_state":last.state,"final_mass_kg":last.vehicle_mass_kg,"cumulative_ideal_delta_v_m_s":last.state.cumulative_ideal_delta_v_m_s,"epochs":epochs,"context":context()})
    }
    fn manifest_from(&self, rows: &[Observation]) -> serde_json::Value {
        serde_json::json!({"format":"cosmogon-cargo-epochs-recording","schema_version":1,"complete":true,"plan":self,"summary":self.summary_from(rows),"source_context":context(),"observations":rows.len(),"columns":OBSERVATION_COLUMNS})
    }
    pub fn export(&self, path: &std::path::Path) -> Result<serde_json::Value, String> {
        use std::io::Write;
        self.validate()?;
        let rows = self.run()?;
        let csv = epochs_csv(&rows);
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
            ("epochs.csv", csv.into_bytes()),
            (
                "manifest.json",
                serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
            ),
        ];
        if files
            .iter()
            .any(|(_, bytes)| bytes.len() > 32 * 1024 * 1024)
        {
            return Err("Cargo-epochs export exceeds bounded file size".into());
        }
        if path.exists() {
            return Err(
                "Existing cargo-epochs recordings are protected; choose a new folder".into(),
            );
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
            .map_err(|e| format!("Cargo-epochs definitions are protected: {e}"))?;
        file.write_all(&bytes).map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug)]
pub struct Bundle {
    pub plan: Plan,
    pub rows: Vec<Observation>,
    pub manifest: serde_json::Value,
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
        let manifest: serde_json::Value =
            serde_json::from_slice(&read_bounded(&path.join("manifest.json"), 1024 * 1024)?)
                .map_err(|e| e.to_string())?;
        if manifest["format"] != "cosmogon-cargo-epochs-recording"
            || manifest["schema_version"] != 1
            || manifest["complete"] != true
        {
            return Err("Cargo-epochs recording is incomplete or unsupported".into());
        }
        let plan = Plan::read(&path.join("plan.json"))?;
        let expected = plan.run()?;
        if manifest != plan.manifest_from(&expected) {
            return Err("Cargo-epochs recording plan, sources or result differs".into());
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
                "Stored cargo-epochs trajectory differs from an independent exact replay".into(),
            );
        }
        if read_bounded(&path.join("epochs.csv"), 32 * 1024 * 1024)?
            != epochs_csv(&expected).into_bytes()
        {
            return Err("Stored cargo-epochs CSV differs from its exact trajectory".into());
        }
        Ok(Self {
            plan,
            rows,
            manifest,
        })
    }
}
pub fn read_recording(path: &std::path::Path) -> Result<serde_json::Value, String> {
    Ok(Bundle::read(path)?.manifest)
}
const OBSERVATION_COLUMNS: usize = 39;
fn read_bounded(path: &std::path::Path, limit: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Cargo-epochs input exceeds bounded file size".into());
    }
    Ok(bytes)
}
fn epochs_csv(rows: &[Observation]) -> String {
    let mut csv=String::from("observation_index,phase,step_index,elapsed_s,vehicle_mass_kg,held_propellant_kg,exhausted_propellant_kg,cumulative_impulse_n_s,cumulative_ideal_delta_v_m_s,velocity_change_x_m_s,velocity_change_y_m_s,velocity_change_z_m_s,powered_duration_s,fuel_limited_duration_s,held_oxygen_kg,held_potable_water_kg,held_food_dry_kg,held_food_water_kg,interval_initial_mass_kg,interval_final_mass_kg,interval_ideal_delta_v_m_s,interval_actual_propellant_kg,interval_mass_residual_kg,removal_requested_oxygen_kg,removal_requested_potable_water_kg,removal_requested_food_dry_kg,removal_requested_food_water_kg,removal_actual_oxygen_kg,removal_actual_potable_water_kg,removal_actual_food_dry_kg,removal_actual_food_water_kg,removal_mass_residual_kg,removal_momentum_x_kg_m_s,removal_momentum_y_kg_m_s,removal_momentum_z_kg_m_s,removal_kinetic_energy_j,removal_velocity_x_m_s,removal_velocity_y_m_s,removal_velocity_z_m_s\n");
    for (index, row) in rows.iter().enumerate() {
        let s = &row.state;
        let mut fields = vec![
            index.to_string(),
            row.phase.key().into(),
            row.step_index.map(|v| v.to_string()).unwrap_or_default(),
            s.elapsed_s.to_string(),
            row.vehicle_mass_kg.to_string(),
            s.propellant_kg.to_string(),
            s.exhausted_propellant_kg.to_string(),
            s.cumulative_impulse_n_s.to_string(),
            s.cumulative_ideal_delta_v_m_s.to_string(),
            s.velocity_change_m_s[0].to_string(),
            s.velocity_change_m_s[1].to_string(),
            s.velocity_change_m_s[2].to_string(),
            s.powered_duration_s.to_string(),
            s.fuel_limited_duration_s.to_string(),
            row.held.oxygen_reserve_kg.to_string(),
            row.held.potable_water_kg.to_string(),
            row.held.food_dry_kg.to_string(),
            row.held.food_water_kg.to_string(),
        ];
        if let Some(i) = &row.burn {
            fields.extend([
                i.initial_mass_kg.to_string(),
                i.final_mass_kg.to_string(),
                i.ideal_delta_v_m_s.to_string(),
                i.actual_propellant_kg.to_string(),
                i.mass_residual_kg.to_string(),
            ]);
        } else {
            fields.extend(std::iter::repeat_n(String::new(), 5));
        }
        if let Some(e) = &row.removal {
            fields.extend([
                e.requested.oxygen_reserve_kg.to_string(),
                e.requested.potable_water_kg.to_string(),
                e.requested.food_dry_kg.to_string(),
                e.requested.food_water_kg.to_string(),
                e.actual_removed.oxygen_reserve_kg.to_string(),
                e.actual_removed.potable_water_kg.to_string(),
                e.actual_removed.food_dry_kg.to_string(),
                e.actual_removed.food_water_kg.to_string(),
                e.mass_residual_kg.to_string(),
                e.carried_linear_momentum_kg_m_s.x.to_string(),
                e.carried_linear_momentum_kg_m_s.y.to_string(),
                e.carried_linear_momentum_kg_m_s.z.to_string(),
                e.carried_kinetic_energy_j.to_string(),
                e.velocity_m_s.x.to_string(),
                e.velocity_m_s.y.to_string(),
                e.velocity_m_s.z.to_string(),
            ]);
        } else {
            fields.extend(std::iter::repeat_n(String::new(), 16));
        }
        debug_assert_eq!(fields.len(), OBSERVATION_COLUMNS);
        csv.push_str(&fields.join(","));
        csv.push('\n');
    }
    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Definition, State, Ledger) {
        let d = Definition::example();
        let p = State::new(&d).unwrap();
        let l = Ledger::new(&d).unwrap();
        (d, p, l)
    }
    fn burn(seconds: f64) -> Command {
        Command {
            duration_s: seconds,
            throttle: 1.,
            direction: [1., 0., 0.],
        }
    }
    fn water(kg: f64) -> Cargo {
        cargo([0., kg, 0., 0.])
    }
    #[test]
    fn burns_across_two_removals_match_independent_piecewise_rocket_equation() {
        let (d, mut p, mut l) = setup();
        let original = d.clone();
        let first = l.apply_burn(&d, &mut p, &burn(10.)).unwrap();
        l.remove(&d, &p, water(30.), Vec3d::ZERO).unwrap();
        let second = l.apply_burn(&d, &mut p, &burn(20.)).unwrap();
        l.remove(&d, &p, cargo([5., 20., 5., 5.]), Vec3d::ZERO)
            .unwrap();
        let third = l.apply_burn(&d, &mut p, &burn(30.)).unwrap();
        let independent = [first, second, third]
            .iter()
            .map(|x| d.exhaust_velocity_m_s() * (x.initial_mass_kg / x.final_mass_kg).ln())
            .sum::<f64>();
        assert!((p.cumulative_ideal_delta_v_m_s - independent).abs() < 1e-8);
        assert_eq!(d, original);
        let wrong = d.exhaust_velocity_m_s()
            * ((d.inert_mass_kg() + d.initial_propellant_kg) / l.mass_kg(&d, &p)).ln();
        assert!((wrong - independent).abs() > 10.);
        l.validate(&d, &p).unwrap();
    }
    #[test]
    fn central_comoving_removal_carries_material_without_free_impulse_or_fuel() {
        let (d, p, mut l) = setup();
        let before = p.clone();
        let v = Vec3d::new(12., -3., 4.);
        let e = l.remove(&d, &p, water(20.), v).unwrap();
        assert_eq!(p, before);
        assert_eq!(e.carried_linear_momentum_kg_m_s, v * 20.);
        assert_eq!(e.carried_kinetic_energy_j, 1690.);
        let initial_momentum = v * e.initial_mass_kg;
        let final_momentum = v * e.final_mass_kg + e.carried_linear_momentum_kg_m_s;
        assert!((initial_momentum - final_momentum).length() < 1e-9);
        assert_eq!(l.held.food_water_kg, d.cargo.food_water_kg);
        assert_eq!(l.held.oxygen_reserve_kg, d.cargo.oxygen_reserve_kg);
    }
    #[test]
    fn invalid_overdraw_unrepresented_and_nonfinite_removals_are_atomic() {
        let (d, p, mut l) = setup();
        let before = l.clone();
        for request in [
            water(101.),
            water(-1.),
            water(f64::NAN),
            water(0.),
            water(1e-30),
        ] {
            assert!(l.remove(&d, &p, request, Vec3d::ZERO).is_err());
            assert_eq!(l, before);
        }
        assert!(l
            .remove(&d, &p, water(1.), Vec3d::new(f64::INFINITY, 0., 0.))
            .is_err());
        assert_eq!(l, before);
    }
    #[test]
    fn conservative_rounding_never_exceeds_request_and_preserves_species() {
        let (d, p, mut l) = setup();
        let e = l
            .remove(&d, &p, cargo([0.1, 0.1, 0.1, 0.1]), Vec3d::ZERO)
            .unwrap();
        for ((a, b), r) in quantities(&e.before)
            .into_iter()
            .zip(quantities(&e.after))
            .zip(quantities(&e.actual_removed))
        {
            assert_eq!(a - b, r);
            assert!(r > 0. && r <= 0.1);
        }
        l.validate(&d, &p).unwrap();
    }
    #[test]
    fn serialized_epoch_corruption_and_old_single_mass_denominator_are_refused() {
        let (d, mut p, mut l) = setup();
        l.apply_burn(&d, &mut p, &burn(10.)).unwrap();
        l.remove(&d, &p, water(50.), Vec3d::ZERO).unwrap();
        l.apply_burn(&d, &mut p, &burn(10.)).unwrap();
        let exact: Ledger = serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
        exact.validate(&d, &p).unwrap();
        assert_eq!(exact, l);
        for path in [
            "/initial_definition/dry_mass_kg",
            "/held/potable_water_kg",
            "/events/0/propellant_kg",
            "/events/0/carried_kinetic_energy_j",
            "/events/0/actual_removed/potable_water_kg",
        ] {
            let mut j = serde_json::to_value(&l).unwrap();
            let cell = j.pointer_mut(path).unwrap();
            *cell = serde_json::json!(cell.as_f64().unwrap() + 1.);
            let bad: Ledger = serde_json::from_value(j).unwrap();
            assert!(bad.validate(&d, &p).is_err());
        }
        let mut bad = p.clone();
        bad.cumulative_ideal_delta_v_m_s = d.exhaust_velocity_m_s()
            * (bad.exhausted_propellant_kg / (d.inert_mass_kg() + bad.propellant_kg)).dln1p();
        assert!(l.validate(&d, &bad).is_err());
    }
    #[test]
    fn unchanged_ledger_preserves_legacy_propulsion_and_exact_chunking() {
        let (d, mut p, l) = setup();
        let mut old = p.clone();
        let c = burn(4.);
        assert_eq!(
            l.apply_burn(&d, &mut p, &c).unwrap(),
            old.apply(&d, &c).unwrap()
        );
        assert_eq!(p, old);
        let mut one = p.clone();
        let mut parts = p.clone();
        l.apply_burn(&d, &mut one, &burn(4.)).unwrap();
        for _ in 0..4 {
            l.apply_burn(&d, &mut parts, &burn(1.)).unwrap();
        }
        assert!(
            (one.cumulative_ideal_delta_v_m_s - parts.cumulative_ideal_delta_v_m_s).abs() < 1e-9
        );
        assert!((one.propellant_kg - parts.propellant_kg).abs() < 1e-10);
    }
    #[test]
    fn unresolved_total_mass_and_epoch_budget_do_not_drop_material() {
        let (mut d, p, _) = setup();
        d.dry_mass_kg = 1e12;
        let mut l = Ledger::new(&d).unwrap();
        let before = l.clone();
        assert!(l.remove(&d, &p, water(1e-6), Vec3d::ZERO).is_err());
        assert_eq!(l, before);
        let (d, p, mut l) = setup();
        // Equal-time finite split events are legitimate material accounting, not timed arrival.
        for _ in 0..MAX_EVENTS {
            l.remove(&d, &p, water(0.001), Vec3d::ZERO).unwrap();
        }
        let before = l.clone();
        assert!(l.remove(&d, &p, water(0.001), Vec3d::ZERO).is_err());
        assert_eq!(l, before);
    }

    #[test]
    fn exhaustion_after_unloading_retains_all_nonfuel_material_and_closes_counters() {
        let (d, mut p, mut l) = setup();
        l.remove(&d, &p, water(100.), Vec3d::ZERO).unwrap();
        l.apply_burn(&d, &mut p, &burn(1000.)).unwrap();
        assert_eq!(p.propellant_kg, 0.);
        assert_eq!(l.held.potable_water_kg, 0.);
        assert_eq!(l.mass_kg(&d, &p), 1070.);
        l.validate(&d, &p).unwrap();
        let saved = p.clone();
        assert!(l.apply_burn(&d, &mut p, &burn(f64::NAN)).is_err());
        assert_eq!(p, saved);
    }
}

#[cfg(test)]
mod sequence_tests {
    use super::*;
    use crate::supply_vehicle as bench;
    fn burn(seconds: f64) -> Step {
        Step::Burn(Command {
            duration_s: seconds,
            throttle: 1.,
            direction: [1., 0., 0.],
        })
    }
    #[test]
    fn example_sequence_closes_independent_piecewise_deltas_and_epoch_denominators() {
        let plan = Plan::example();
        let rows = plan.run().unwrap();
        // Independent per-burn rocket-equation sum across actual removal epochs.
        let mut independent = 0.;
        let mut naive_intervals = vec![];
        for r in &rows {
            if let Some(i) = &r.burn {
                if i.actual_propellant_kg > 0. {
                    independent += plan.definition.exhaust_velocity_m_s()
                        * (i.initial_mass_kg / i.final_mass_kg).ln();
                    naive_intervals.push((i.initial_mass_kg, i.final_mass_kg));
                }
            }
        }
        let last = rows.last().unwrap();
        assert!((last.state.cumulative_ideal_delta_v_m_s - independent).abs() < 1e-8);
        // The invalid single initial-to-final logarithm must differ once cargo is removed.
        let naive = plan.definition.exhaust_velocity_m_s()
            * ((plan.definition.inert_mass_kg() + plan.definition.initial_propellant_kg)
                / last.vehicle_mass_kg)
                .ln();
        assert!((naive - independent).abs() > 1.);
        // Every burn epoch denominator equals the actual masses at its retained interval.
        for r in &rows {
            if let Some(i) = &r.burn {
                if i.actual_propellant_kg > 0. {
                    assert!(i.final_mass_kg < i.initial_mass_kg);
                    assert!(i.initial_mass_kg.is_finite() && i.final_mass_kg.is_finite());
                }
            }
        }
        assert_eq!(plan.run().unwrap(), rows);
    }
    #[test]
    fn burn_only_sequence_is_bit_identical_to_legacy_propulsion_bench() {
        let commands = vec![
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
        ];
        let legacy = bench::Plan {
            format: "cosmogon-supply-vehicle-plan".into(),
            schema_version: 1,
            definition: Definition::example(),
            commands: commands.clone(),
            observation_interval_s: 1.,
        };
        let mut sequence = Plan::example();
        sequence.definition = legacy.definition.clone();
        sequence.observation_interval_s = 1.;
        sequence.steps = commands.into_iter().map(Step::Burn).collect();
        let seq_rows = sequence.run().unwrap();
        let old_rows = legacy.run().unwrap();
        assert_eq!(seq_rows.len(), old_rows.len());
        for (s, o) in seq_rows.iter().zip(&old_rows) {
            assert_eq!(s.state, o.state);
            assert_eq!(s.held, legacy.definition.cargo);
        }
    }
    #[test]
    fn removal_observations_retain_requested_actual_residual_and_momentum_energy() {
        let plan = Plan::example();
        let rows = plan.run().unwrap();
        let removals: Vec<_> = rows
            .iter()
            .filter_map(|r| r.removal.as_ref().map(|e| (r, e)))
            .collect();
        assert_eq!(removals.len(), 2);
        let (row, event) = &removals[1];
        assert_eq!(event.requested, cargo([5., 20., 5., 5.]));
        assert_eq!(event.actual_removed, event.requested);
        assert!((event.mass_residual_kg).abs() < 1e-9);
        let v = Vec3d::new(12., -3., 4.);
        assert_eq!(event.velocity_m_s, v);
        assert_eq!(event.carried_linear_momentum_kg_m_s, v * event.actual_removed.mass_kg());
        assert_eq!(
            event.carried_kinetic_energy_j,
            0.5 * event.actual_removed.mass_kg() * v.length_squared()
        );
        // The removal observation state is the unchanged propulsion state at its clock.
        let index = rows.iter().position(|r| r.removal.as_ref() == Some(event)).unwrap();
        assert_eq!(row.state, rows[index - 1].state);
        assert_eq!(row.held, event.after);
        assert_eq!(row.vehicle_mass_kg, event.final_mass_kg);
    }
    #[test]
    fn invalid_sequence_inputs_are_refused_before_any_mutation() {
        let plan = Plan::example();
        for mutate in [
            |p: &mut Plan| p.steps.clear(),
            |p: &mut Plan| p.observation_interval_s = 0.,
            |p: &mut Plan| {
                p.steps[0] = Step::Burn(Command {
                    duration_s: 10.,
                    throttle: 1.,
                    direction: [0.; 3],
                })
            },
            |p: &mut Plan| {
                p.steps[1] = Step::RemoveCargo(RemovalRequest {
                    requested: cargo([0., -1., 0., 0.]),
                    velocity_m_s: [0., 0., 0.],
                })
            },
            |p: &mut Plan| {
                p.steps[1] = Step::RemoveCargo(RemovalRequest {
                    requested: cargo([0., 1., 0., 0.]),
                    velocity_m_s: [f64::INFINITY, 0., 0.],
                })
            },
            |p: &mut Plan| p.format = "unknown".into(),
        ] {
            let mut bad = plan.clone();
            mutate(&mut bad);
            assert!(bad.validate().is_err());
            assert!(bad.run().is_err());
        }
        // Overdraw fails at run time without partial retained rows escaping the API.
        let mut overdraw = plan.clone();
        overdraw.steps = vec![
            burn(1.),
            Step::RemoveCargo(RemovalRequest {
                requested: cargo([0., 101., 0., 0.]),
                velocity_m_s: [0., 0., 0.],
            }),
        ];
        assert!(overdraw.run().is_err());
        // Too many steps are refused up front.
        let mut many = plan.clone();
        many.steps = vec![burn(1.); MAX_STEPS + 1];
        assert!(many.run().is_err());
        // Unbounded sampling is refused up front rather than running forever.
        let mut dense = plan.clone();
        dense.observation_interval_s = 1e-3;
        dense.steps = vec![burn(1e7)];
        assert!(dense.validate().is_err());
    }
    #[test]
    fn protected_sequence_recording_reopens_exactly_and_detects_tampering() {
        let root = std::env::temp_dir().join(format!(
            "cosmogon-cargo-epochs-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let plan = Plan::example();
        let manifest = plan.export(&root).unwrap();
        assert_eq!(read_recording(&root).unwrap(), manifest);
        assert_eq!(Bundle::read(&root).unwrap().rows, plan.run().unwrap());
        let before = std::fs::read(root.join("observations.json")).unwrap();
        assert!(plan.export(&root).is_err());
        assert_eq!(
            std::fs::read(root.join("observations.json")).unwrap(),
            before
        );
        std::fs::write(root.join("epochs.csv"), "tampered").unwrap();
        assert!(Bundle::read(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn reopening_refuses_altered_plan_manifest_and_rows_without_replacing_files() {
        let root = std::env::temp_dir().join(format!(
            "cosmogon-cargo-epochs-strict-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let p = Plan::example();
        p.export(&root).unwrap();
        let original = std::fs::read(root.join("manifest.json")).unwrap();
        for (key, value) in [
            ("complete", serde_json::json!(false)),
            ("source_context", serde_json::json!({"model":"unavailable"})),
            ("extra_unknown", serde_json::json!(true)),
        ] {
            let mut v: serde_json::Value = serde_json::from_slice(&original).unwrap();
            v[key] = value;
            std::fs::write(root.join("manifest.json"), serde_json::to_vec(&v).unwrap()).unwrap();
            assert!(Bundle::read(&root).is_err());
        }
        std::fs::write(root.join("manifest.json"), original).unwrap();
        let original_rows = std::fs::read(root.join("observations.json")).unwrap();
        let mut rows: serde_json::Value = serde_json::from_slice(&original_rows).unwrap();
        rows[1]["vehicle_mass_kg"] = serde_json::json!(999999.);
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
    #[test]
    fn summary_exposes_epoch_denominators_requested_actual_and_stated_scope() {
        let plan = Plan::example();
        let summary = plan.summary().unwrap();
        assert_eq!(summary["model"], MODEL);
        assert_eq!(summary["removal_count"], 2);
        let epochs = summary["epochs"].as_array().unwrap();
        let removals: Vec<_> = epochs
            .iter()
            .filter(|e| e["kind"] == "removal")
            .collect();
        assert_eq!(removals.len(), 2);
        assert_eq!(removals[1]["requested"]["potable_water_kg"], 20.);
        assert_eq!(removals[1]["actual_removed"]["potable_water_kg"], 20.);
        assert!(removals[1]["carried_kinetic_energy_j"].as_f64().unwrap() > 0.);
        let burns: Vec<_> = epochs.iter().filter(|e| e["kind"] == "burn").collect();
        assert!(!burns.is_empty());
        for b in burns {
            let m0 = b["initial_mass_kg"].as_f64().unwrap();
            let m1 = b["final_mass_kg"].as_f64().unwrap();
            assert!(m0 > 0. && m1 > 0.);
        }
        assert_eq!(
            summary["context"]["scope"],
            "Separate finite held cargo and piecewise ideal propulsion accounting; no world transfer or landing permission"
        );
    }
}
