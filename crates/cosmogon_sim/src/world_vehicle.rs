//! Physical f64 test-particle flight, reusing finite authored vehicle inventories.
//! This module's core is being connected to the authoritative world scheduler and UI.
use crate::{
    astro::{StarSystem, G},
    supply_vehicle::{Command, Interval, Plan, State},
    BodyRef, Universe, Vec3d,
};
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-finite-world-vehicle/1";
pub const CONTACT_MODEL: &str = "cosmogon-finite-world-vehicle/2";
pub const SHEAR_MODEL: &str = "cosmogon-finite-world-vehicle/3";
pub const FRAME: &str = "system-local-inertial-simulation-axes";
pub(crate) const SUBSTEP_S: f64 = 0.25;
pub(crate) const TASK: usize = 5;
pub(crate) const PERIOD: f64 = 1.;
pub(crate) const MAX_VEHICLES: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub frame: String,
    pub parent: BodyRef,
    pub plan: Plan,
    /// Relative to the body's actual translation, in non-rotating system axes.
    pub initial_relative_position_m: Vec3d,
    /// Added to the body's actual inertial translational velocity; not body-fixed velocity.
    pub initial_relative_velocity_m_s: Vec3d,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<crate::vehicle_contact_dynamics::Hardware>,
}
impl Definition {
    pub fn with_contact(mut self, world:&Universe, hardware:crate::vehicle_contact_dynamics::Hardware)->Result<Self,String> {
        self.model=hardware.flight_model().into();self.contact=Some(hardware);
        Flight::new(world.next_vehicle_id,self.clone(),world)?;
        Ok(self)
    }
    pub fn circular_example(u: &Universe, parent: BodyRef, plan: Plan) -> Result<Self, String> {
        let body = u
            .systems
            .get(parent.system as usize)
            .and_then(|s| s.bodies.get(parent.body as usize))
            .ok_or("No such starting body")?;
        let r = body.radius + 200_000.;
        let d = Self {
            model: MODEL.into(),
            frame: FRAME.into(),
            parent,
            plan,
            initial_relative_position_m: Vec3d::new(r, 0., 0.),
            initial_relative_velocity_m_s: Vec3d::new(0., (G * body.mass / r).sqrt(), 0.),
            contact: None,
        };
        d.validate(u)?;
        Ok(d)
    }
    pub fn validate_structure(&self) -> Result<(), String> {
        if self.frame != FRAME || !matches!((self.model.as_str(), &self.contact), (MODEL, None) | (CONTACT_MODEL | SHEAR_MODEL, Some(_))) {
            return Err("Unsupported finite flight model or reference frame".into());
        }
        if let Some(h) = &self.contact { h.validate()?; if self.model != h.flight_model() {return Err("World-flight and contact hardware model versions disagree".into());} }
        self.plan.validate()?;
        if !finite(self.initial_relative_position_m)
            || self.initial_relative_position_m.length() > 1e15
            || !finite(self.initial_relative_velocity_m_s)
            || self.initial_relative_velocity_m_s.length() > 1e6
        {
            return Err("Initial flight vectors exceed their supported finite bounds".into());
        }
        Ok(())
    }
    pub fn validate(&self, u: &Universe) -> Result<(), String> {
        self.validate_structure()?;
        let body = u
            .systems
            .get(self.parent.system as usize)
            .and_then(|s| s.bodies.get(self.parent.body as usize))
            .ok_or("No such starting body")?;
        if !body.exists()
            || !body.mass.is_finite()
            || body.mass <= 0.
            || !body.radius.is_finite()
            || body.radius <= 0.
        {
            return Err("Starting body lacks supported finite physical mass/radius".into());
        }
        if self.plan.definition.inert_mass_kg() + self.plan.definition.initial_propellant_kg
            > body.mass * 1e-6
        {
            return Err(
                "Vehicle exceeds the supported test-particle initial parent mass ratio (1e-6)"
                    .into(),
            );
        }
        if !finite(self.initial_relative_position_m)
            || self.initial_relative_position_m.length() > 1e15
            || self.contact.is_none() && self.initial_relative_position_m.length() <= body.radius
        {
            return Err("Choose a finite initial position outside the starting body's spherical boundary, within 1e15 m".into());
        }
        if !finite(self.initial_relative_velocity_m_s)
            || self.initial_relative_velocity_m_s.length() > 1e6
        {
            return Err("Initial relative velocity must be finite and at most 1e6 m/s".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "body_index", rename_all = "snake_case")]
pub enum Source {
    Star,
    Companion,
    Body(u32),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GravitySample {
    pub source: Source,
    pub position_m: Vec3d,
    pub mu_m3_s2: f64,
    pub radius_m: f64,
}
fn source_samples(sources: &[Attractor]) -> Vec<GravitySample> {
    sources
        .iter()
        .map(|s| GravitySample {
            source: s.source.clone(),
            position_m: s.position_m,
            mu_m3_s2: s.mu,
            radius_m: s.radius_m,
        })
        .collect()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase {
    Flying,
    Impact {
        source: Source,
        source_position_m: Vec3d,
        radius_m: f64,
        position_residual_m: f64,
        tolerance_m: f64,
        source_sample_duration_s: f64,
    },
    Unsupported {
        reason: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flight {
    pub id: u64,
    pub definition: Definition,
    pub launched_at_s: f64,
    pub last_time_s: f64,
    pub position_m: Vec3d,
    pub velocity_m_s: Vec3d,
    pub propulsion: State,
    pub last_interval: Option<Interval>,
    pub phase: Phase,
    /// Actual source parameters at the retained craft clock; terminal values are historical.
    pub last_sources: Vec<GravitySample>,
    pub(crate) edit_cursor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<crate::vehicle_contact_dynamics::RigidState>,
}
impl Flight {
    pub fn new(id: u64, d: Definition, u: &Universe) -> Result<Self, String> {
        d.validate(u)?;
        let s = &u.systems[d.parent.system as usize];
        let centre = s.body_state(d.parent.body as usize, u.time);
        let position = centre.pos + d.initial_relative_position_m;
        let velocity = centre.vel + d.initial_relative_velocity_m_s;
        let sources = gravity_sources(s, u.time)?;
        if !finite(position)
            || !finite(velocity)
            || velocity.length() > 1e6
            || sources
                .iter()
                .any(|b| !(d.contact.is_some() && b.source == Source::Body(d.parent.body)) && (position - b.position_m).length() <= b.radius_m)
        {
            return Err(
                "Initial craft state is invalid or inside a modeled spherical contact boundary"
                    .into(),
            );
        }
        supported_motion(position, velocity)?;
        let propulsion = State::new(&d.plan.definition)?;
        let contact = d.contact.as_ref().map(|h| crate::vehicle_contact_dynamics::RigidState::new(u, d.parent, position, velocity, h)).transpose()?;
        Ok(Self {
            id,
            definition: d,
            launched_at_s: u.time,
            last_time_s: u.time,
            position_m: position,
            velocity_m_s: velocity,
            propulsion,
            last_interval: None,
            phase: Phase::Flying,
            last_sources: source_samples(&sources),
            edit_cursor: u.edits.len() as u64,
            contact,
        })
    }
    /// Validate retained state without reinterpreting original authored conditions
    /// against a body whose mass, radius or existence may have changed since launch.
    pub(crate) fn validate_saved(&self, u: &Universe) -> Result<(), String> {
        self.definition.validate_structure()?;
        if self.contact.is_some() && matches!(self.phase,Phase::Impact {..}) {
            return Err("Terrain contact cannot reinterpret a version-1 spherical terminal impact".into());
        }
        match (&self.definition.contact, &self.contact) {
            (None, None) => {},
            (Some(h), Some(state)) => state.validate(h, self)?,
            _ => return Err("Flight contact hardware/state model mismatch".into()),
        }
        let system = u
            .systems
            .get(self.definition.parent.system as usize)
            .ok_or("Missing vehicle system")?;
        if self.definition.parent.body as usize >= system.bodies.len() {
            return Err("Missing original vehicle parent reference".into());
        }
        supported_motion(self.position_m, self.velocity_m_s)?;
        if self.edit_cursor > u.edits.len() as u64
            || self.last_sources.is_empty()
            || self.last_sources.len() > system.bodies.len() + 2
            || self.last_sources[0].source != Source::Star
        {
            return Err("Invalid retained gravity source inventory or edit cursor".into());
        }
        let mut source_ids = std::collections::BTreeSet::new();
        for source in &self.last_sources {
            if !source_ids.insert(source.source.clone())
                || !finite(source.position_m)
                || !source.mu_m3_s2.is_finite()
                || source.mu_m3_s2 < 0.
                || !source.radius_m.is_finite()
                || source.radius_m < 0.
                || matches!(source.source,Source::Body(i) if i as usize >= system.bodies.len())
            {
                return Err("Invalid retained gravity sample".into());
            }
        }
        let d = &self.definition.plan.definition;
        let p = &self.propulsion;
        let mut preview = p.clone();
        preview.apply(
            d,
            &Command {
                duration_s: 0.,
                throttle: 0.,
                direction: [1., 0., 0.],
            },
        )?;
        let mass_tolerance = 1e-8 * d.initial_propellant_kg.max(1.);
        if p.exhausted_propellant_kg > d.initial_propellant_kg + mass_tolerance
            || (p.propellant_kg + p.exhausted_propellant_kg - d.initial_propellant_kg).abs()
                > mass_tolerance
        {
            return Err("Retained flight propellant inventory does not close".into());
        }
        let impulse = p.exhausted_propellant_kg * d.exhaust_velocity_m_s();
        let capability =
            d.exhaust_velocity_m_s() * (p.exhausted_propellant_kg / p.mass_kg(d)).dln1p();
        if (p.cumulative_impulse_n_s - impulse).abs() > 1e-8 * impulse.max(1.)
            || (p.cumulative_ideal_delta_v_m_s - capability).abs() > 1e-8 * capability.max(1.)
            || Vec3d::new(
                p.velocity_change_m_s[0],
                p.velocity_change_m_s[1],
                p.velocity_change_m_s[2],
            )
            .length()
                > p.cumulative_ideal_delta_v_m_s + 1e-8 * capability.max(1.)
            || p.powered_duration_s + p.fuel_limited_duration_s
                > p.elapsed_s + 1e-8 * p.elapsed_s.max(1.)
        {
            return Err(
                "Retained propulsion counters do not match finite mass or elapsed support".into(),
            );
        }
        let clock_tolerance =
            (64. * f64::EPSILON * self.last_time_s.abs().max(self.launched_at_s.abs())).max(1e-6);
        if !u.time.is_finite()
            || !self.launched_at_s.is_finite()
            || !self.last_time_s.is_finite()
            || self.launched_at_s > self.last_time_s
            || self.last_time_s > u.time + clock_tolerance
            || (self.launched_at_s + p.elapsed_s - self.last_time_s).abs() > clock_tolerance
            || self.phase == Phase::Flying && u.time - self.last_time_s > PERIOD + clock_tolerance
        {
            return Err("Invalid retained flight/world clocks".into());
        }
        match &self.phase {
            Phase::Flying => {}
            Phase::Unsupported { reason } => {
                if reason.trim().is_empty() || reason.len() > 4096 {
                    return Err("Invalid unsupported-flight reason".into());
                }
            }
            Phase::Impact {
                source,
                source_position_m,
                radius_m,
                position_residual_m,
                tolerance_m,
                source_sample_duration_s,
            } => {
                let valid_source = match source {
                    Source::Star => true,
                    Source::Companion => true,
                    Source::Body(i) => (*i as usize) < system.bodies.len(),
                };
                if !valid_source
                    || !finite(*source_position_m)
                    || !radius_m.is_finite()
                    || *radius_m <= 0.
                    || !position_residual_m.is_finite()
                    || !tolerance_m.is_finite()
                    || *tolerance_m < 0.001
                    || *tolerance_m > radius_m * 0.01
                    || position_residual_m.abs() > 2. * tolerance_m
                    || ((self.position_m - *source_position_m).length()
                        - radius_m
                        - position_residual_m)
                        .abs()
                        > *tolerance_m
                    || !source_sample_duration_s.is_finite()
                    || !(0.0..=SUBSTEP_S).contains(source_sample_duration_s)
                {
                    return Err("Invalid retained spherical contact evidence".into());
                }
            }
        }
        if let Some(i) = &self.last_interval {
            if [
                i.duration_s,
                i.initial_mass_kg,
                i.final_mass_kg,
                i.commanded_thrust_n,
                i.requested_propellant_kg,
                i.actual_propellant_kg,
                i.impulse_n_s,
                i.ideal_delta_v_m_s,
                i.powered_duration_s,
                i.fuel_limited_duration_s,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.)
                || !i.mass_residual_kg.is_finite()
                || i.duration_s > SUBSTEP_S + 1e-8
                || (i.final_mass_kg - p.mass_kg(d)).abs() > mass_tolerance
                || (i.initial_mass_kg - i.final_mass_kg - i.actual_propellant_kg).abs()
                    > mass_tolerance
                || i.powered_duration_s + i.fuel_limited_duration_s > i.duration_s + 1e-8
                || [
                    i.initial_powered_acceleration_m_s2,
                    i.final_powered_acceleration_m_s2,
                ]
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || *v < 0.)
            {
                return Err("Invalid retained last flight subinterval".into());
            }
        } else if p.elapsed_s != 0. {
            return Err("Missing retained flight subinterval".into());
        }
        Ok(())
    }
    /// One scheduled macro interval. External callers cannot skip unresolved flight.
    pub(crate) fn advance_system(&mut self, system: &StarSystem, to: f64) -> Result<(), String> {
        if self.contact.is_some() { return Err("Terrain flight requires the complete modeled world".into()); }
        if self.phase != Phase::Flying {
            return Ok(());
        }
        let dt = to - self.last_time_s;
        if !to.is_finite() || !dt.is_finite() || !(0.0..=1.000001).contains(&dt) {
            return Err(
                "Finite flight requires a supported scheduled interval of at most one second"
                    .into(),
            );
        }
        let retained_sources = gravity_sources(system, self.last_time_s)?;
        if retained_sources.len() != self.last_sources.len()
            || retained_sources
                .iter()
                .zip(&self.last_sources)
                .any(|(a, b)| {
                    a.source != b.source || a.mu != b.mu_m3_s2 || a.radius_m != b.radius_m
                })
        {
            return Err("Gravity source membership, mass or radius changed since the last resolved flight state; this interval cannot reconstruct that discontinuity".into());
        }
        let mut next = self.clone();
        let mut remaining = dt;
        while remaining > 0. {
            let elapsed = next.propulsion.elapsed_s;
            let (command, left) = active_command(&next.definition.plan, elapsed);
            let rate = next.definition.plan.definition.rated_thrust_n * command.throttle
                / next.definition.plan.definition.exhaust_velocity_m_s();
            let cutoff = if rate > 0. && next.propulsion.propellant_kg > 0. {
                exhaustion_time(next.propulsion.propellant_kg, rate)
            } else {
                f64::INFINITY
            };
            let h = remaining.min(SUBSTEP_S).min(left).min(cutoff);
            if h <= 0. || elapsed + h == elapsed {
                return Err("Flight interval is below represented elapsed-time precision".into());
            }
            let time = next.launched_at_s + elapsed;
            let initial = gravity_sources(system, time)?;
            let final_sources = gravity_sources(system, time + h)?;
            let sub = Command {
                duration_s: h,
                ..command
            };
            let d = &next.definition.plan.definition;
            let tolerance = (64.
                * f64::EPSILON
                * next
                    .position_m
                    .x
                    .abs()
                    .max(next.position_m.y.abs())
                    .max(next.position_m.z.abs()))
            .max(0.001);
            if let Some(source) = initial.iter().find(|source| {
                (next.position_m - source.position_m).length() <= source.radius_m + tolerance
            }) {
                next.last_sources = source_samples(&initial);
                next.phase = Phase::Impact {
                    source: source.source.clone(),
                    source_position_m: source.position_m,
                    radius_m: source.radius_m,
                    position_residual_m: (next.position_m - source.position_m).length()
                        - source.radius_m,
                    tolerance_m: tolerance,
                    source_sample_duration_s: 0.,
                };
                *self = next;
                return Ok(());
            }
            let acceleration = gravity(next.position_m, &initial)?;
            let path = |t: f64| -> Result<Vec3d, String> {
                Ok(drift(
                    d,
                    &next.propulsion,
                    next.position_m,
                    next.velocity_m_s,
                    &Command {
                        duration_s: t,
                        ..sub.clone()
                    },
                    acceleration,
                )?
                .0)
            };
            let mut contact: Option<(f64, usize, f64)> = None;
            for (index, (a, b)) in initial.iter().zip(&final_sources).enumerate() {
                let radius = a.radius_m.max(b.radius_m);
                if radius <= 0. {
                    continue;
                }
                if tolerance > radius * 0.01 {
                    return Err(
                        "Source contact boundary is below supported coordinate precision".into(),
                    );
                }
                if let Some(t) = first_contact(
                    &path,
                    a.position_m,
                    b.position_m,
                    radius,
                    h,
                    acceleration.length() + d.rated_thrust_n * sub.throttle / d.inert_mass_kg(),
                    tolerance,
                )? {
                    if contact.as_ref().is_none_or(|(old, _, _)| t < *old) {
                        contact = Some((t, index, radius));
                    }
                }
            }
            if let Some((t, index, radius)) = contact {
                let partial = Command {
                    duration_s: t,
                    ..sub
                };
                let (position, v, propulsion, interval) = drift(
                    d,
                    &next.propulsion,
                    next.position_m,
                    next.velocity_m_s,
                    &partial,
                    acceleration,
                )?;
                let fraction = t / h;
                let sources = initial
                    .iter()
                    .zip(&final_sources)
                    .map(|(a, b)| Attractor {
                        source: a.source.clone(),
                        position_m: a.position_m.lerp(b.position_m, fraction),
                        mu: a.mu,
                        radius_m: a.radius_m.max(b.radius_m),
                    })
                    .collect::<Vec<_>>();
                let velocity =
                    v + gravity_contact(position, &sources, index, tolerance)? * (t / 2.);
                let body = &sources[index];
                let residual = (position - body.position_m).length() - radius;
                if residual > tolerance * 2. || residual < -tolerance * 2. {
                    return Err("Resolved contact exceeds its supported spatial tolerance".into());
                }
                supported_motion(position, velocity)?;
                next.position_m = position;
                next.velocity_m_s = velocity;
                next.propulsion = propulsion;
                next.last_interval = Some(interval);
                next.last_time_s = next.launched_at_s + next.propulsion.elapsed_s;
                next.last_sources = source_samples(&sources);
                next.phase = Phase::Impact {
                    source: body.source.clone(),
                    source_position_m: body.position_m,
                    radius_m: radius,
                    position_residual_m: residual,
                    tolerance_m: tolerance,
                    source_sample_duration_s: h,
                };
                *self = next;
                return Ok(());
            }
            let (position, velocity, propulsion, interval) = integrate(
                d,
                &next.propulsion,
                next.position_m,
                next.velocity_m_s,
                &sub,
                &initial,
                &final_sources,
            )?;
            supported_motion(position, velocity)?;
            next.position_m = position;
            next.velocity_m_s = velocity;
            next.propulsion = propulsion;
            next.last_interval = Some(interval);
            remaining = (remaining - h).max(0.);
        }
        next.last_time_s = to;
        next.last_sources = source_samples(&gravity_sources(system, to)?);
        *self = next;
        Ok(())
    }
}
fn supported_motion(position: Vec3d, velocity: Vec3d) -> Result<(), String> {
    if !finite(position) || !finite(velocity) || position.length() > 1e15 || velocity.length() > 1e6
    {
        return Err(
            "Finite flight left the supported 1e15 m / 1e6 m/s system-local Newtonian regime"
                .into(),
        );
    }
    Ok(())
}

impl Universe {
    pub(crate) fn validate_vehicle_state(&self) -> Result<(), String> {
        if self.vehicles.len() > MAX_VEHICLES {
            return Err("More than 64 finite vehicles".into());
        }
        let active = self.vehicles.iter().any(|v| v.phase == Phase::Flying);
        if !self.vehicles.is_empty() {
            let task = self
                .scheduler
                .tasks
                .get(TASK)
                .ok_or("Missing vehicle scheduler")?;
            if task.name != "vehicles"
                || !task.origin.is_finite()
                || task.steps == u64::MAX
                || !task.next_due().is_finite()
                || active
                    && (task.period != PERIOD
                        || task.next_due() <= self.time
                        || task.next_due() - self.time > PERIOD + 0.001)
                || !active && task.period != crate::scheduler::NEVER
            {
                return Err("Invalid finite vehicle scheduler".into());
            }
            if self.habitats.is_empty() {
                let habitat = self
                    .scheduler
                    .tasks
                    .get(crate::habitat::TASK)
                    .ok_or("Missing habitat task reservation")?;
                if habitat.name != "habitats"
                    || habitat.period != crate::scheduler::NEVER
                    || !habitat.origin.is_finite()
                    || habitat.steps != 0
                {
                    return Err("Invalid reserved habitat task before vehicle scheduler".into());
                }
            }
        } else if self.scheduler.tasks.get(TASK).is_some() {
            return Err("Vehicle scheduler without retained vehicles".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for flight in &self.vehicles {
            if flight.id >= self.next_vehicle_id || !ids.insert(flight.id) {
                return Err("Duplicate or out-of-range finite vehicle identifier".into());
            }
            flight.validate_saved(self)?;
        }
        Ok(())
    }
    pub(crate) fn ensure_vehicle_task(&mut self) {
        if self.vehicles.is_empty() {
            if self.scheduler.tasks.len() == TASK + 1
                && self.scheduler.tasks[TASK].name == "vehicles"
            {
                self.scheduler.tasks.remove(TASK);
                self.ensure_habitat_task();
            }
            return;
        }
        if self.scheduler.tasks.len() == crate::habitat::TASK {
            self.scheduler
                .add("habitats", self.time, crate::scheduler::NEVER);
        }
        let active = self.vehicles.iter().any(|v| v.phase == Phase::Flying);
        if self.scheduler.tasks.len() == TASK {
            self.scheduler.add(
                "vehicles",
                self.time,
                if active {
                    PERIOD
                } else {
                    crate::scheduler::NEVER
                },
            );
        } else if let Some(task) = self.scheduler.tasks.get_mut(TASK) {
            if active && task.period == crate::scheduler::NEVER
                || !active && task.period != crate::scheduler::NEVER
            {
                *task = crate::scheduler::Task {
                    name: "vehicles".into(),
                    origin: self.time,
                    period: if active {
                        PERIOD
                    } else {
                        crate::scheduler::NEVER
                    },
                    steps: 0,
                };
            }
        }
    }
    pub(crate) fn step_vehicles(&mut self, time: f64) {
        for index in 0..self.vehicles.len() {
            let mut flight = self.vehicles[index].clone();
            if flight.phase != Phase::Flying {
                continue;
            }
            let discontinuity = self
                .edits
                .iter()
                .skip(flight.edit_cursor as usize)
                .any(|record| {
                    let e = &record.edit;
                    if e.system() != flight.definition.parent.system {
                        return false;
                    }
                    matches!(
                        e,
                        crate::sandbox::Edit::AddBody { .. }
                            | crate::sandbox::Edit::RemoveBody { .. }
                            | crate::sandbox::Edit::SetState { .. }
                            | crate::sandbox::Edit::Impulse { .. }
                            | crate::sandbox::Edit::SetOrbit { .. }
                            | crate::sandbox::Edit::SetStar { .. }
                            | crate::sandbox::Edit::SetPhysics { .. }
                            | crate::sandbox::Edit::SetProperty {
                                property: crate::sandbox::BodyProperty::Mass(_)
                                    | crate::sandbox::BodyProperty::Radius(_),
                                ..
                            }
                    )
                    || flight.contact.is_some() && matches!(e,
                        crate::sandbox::Edit::SetMarsTerrain { .. }
                        | crate::sandbox::Edit::SetProperty { property: crate::sandbox::BodyProperty::RotationPeriod(_) | crate::sandbox::BodyProperty::AxialTilt(_), .. })
                });
            let result = if discontinuity {
                Err("A gravity/frame edit occurred after the last resolved craft state; event-time source discontinuity integration is not supported".into())
            } else if flight.contact.is_some() {
                crate::vehicle_contact_dynamics::advance(&mut flight, self, time)
            } else {
                self.systems
                    .get(flight.definition.parent.system as usize)
                    .ok_or_else(|| "Vehicle's gravitational system is unavailable".to_string())
                    .and_then(|system| flight.advance_system(system, time))
            };
            flight.edit_cursor = self.edits.len() as u64;
            if let Err(reason) = result {
                // The core previews each macro interval; failed integration retains the
                // last supported state and exposes the reason instead of inventing motion.
                flight.phase = Phase::Unsupported { reason };
            }
            if flight.phase != Phase::Flying {
                self.history.push(crate::history::Event {
                    time:if matches!(flight.phase,Phase::Unsupported {..}) {time} else {flight.last_time_s}, category:crate::history::Category::Space,
                    importance:4, title:format!("Finite vehicle {} stopped", flight.definition.plan.definition.name),
                    detail: match &flight.phase {
                        Phase::Impact {source,position_residual_m,..} => format!("First modeled spherical contact with {source:?}; raw boundary residual {position_residual_m} m. This is not a safe landing or cargo delivery."),
                        Phase::Unsupported {reason} => format!("Integration stopped at the last supported state: {reason}"),
                        Phase::Flying => unreachable!(),
                    }, system:Some(flight.definition.parent.system), body:Some(flight.definition.parent.body), civ:None,
                });
            }
            self.vehicles[index] = flight;
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Attractor {
    pub(crate) source: Source,
    pub(crate) position_m: Vec3d,
    pub(crate) mu: f64,
    pub(crate) radius_m: f64,
}
pub(crate) fn gravity_sources(system: &StarSystem, time: f64) -> Result<Vec<Attractor>, String> {
    let mut out = vec![Attractor {
        source: Source::Star,
        position_m: system.star_local_position(time),
        mu: system.star.mu(),
        radius_m: system.star.current_radius(time),
    }];
    if let (Some(companion), Some(position)) =
        (&system.companion, system.companion_local_position(time))
    {
        out.push(Attractor {
            source: Source::Companion,
            position_m: position,
            mu: companion.star.mu(),
            radius_m: companion.star.current_radius(time),
        });
    }
    for (i, b) in system.bodies.iter().enumerate() {
        if b.exists() {
            out.push(Attractor {
                source: Source::Body(i as u32),
                position_m: system.body_state(i, time).pos,
                mu: G * b.mass,
                radius_m: b.radius,
            });
        }
    }
    if out.iter().any(|b| {
        !finite(b.position_m)
            || !b.mu.is_finite()
            || b.mu < 0.
            || !b.radius_m.is_finite()
            || b.radius_m < 0.
    }) {
        return Err("Unsupported physical gravitational source state".into());
    }
    Ok(out)
}
fn gravity(position: Vec3d, sources: &[Attractor]) -> Result<Vec3d, String> {
    let mut acceleration = Vec3d::ZERO;
    for b in sources {
        let delta = b.position_m - position;
        let r = delta.length();
        if r <= b.radius_m || r == 0. {
            return Err("Craft lies inside a modeled contact boundary".into());
        }
        acceleration += delta * (b.mu / (r * r * r));
    }
    if !finite(acceleration) {
        return Err("Unsupported finite gravitational acceleration".into());
    }
    Ok(acceleration)
}
pub(crate) fn active_command(plan: &Plan, elapsed: f64) -> (Command, f64) {
    let mut end = 0.;
    for c in &plan.commands {
        end += c.duration_s;
        if elapsed < end {
            return (c.clone(), end - elapsed);
        }
    }
    (
        Command {
            duration_s: 1.,
            throttle: 0.,
            direction: [1., 0., 0.],
        },
        f64::INFINITY,
    )
}
/// Exact integral of ideal velocity increase during a constant represented mass-flow burn.
fn powered_displacement(
    d: &crate::supply_vehicle::Definition,
    initial: &State,
    interval: &Interval,
) -> f64 {
    let t = interval.powered_duration_s;
    if t == 0. || interval.actual_propellant_kg == 0. {
        return 0.;
    }
    let z = interval.actual_propellant_kg / initial.mass_kg(d);
    let factor = if z < 1e-3 {
        let mut power = z;
        let mut sum = 0.;
        for n in 1..=8 {
            sum += power / (n * (n + 1)) as f64;
            power *= z;
        }
        sum
    } else {
        1. + (1. - z) / z * (-z).dln1p()
    };
    d.exhaust_velocity_m_s() * t * factor + interval.ideal_delta_v_m_s * (interval.duration_s - t)
}
fn integrate(
    d: &crate::supply_vehicle::Definition,
    state: &State,
    position: Vec3d,
    velocity: Vec3d,
    command: &Command,
    initial: &[Attractor],
    final_sources: &[Attractor],
) -> Result<(Vec3d, Vec3d, State, Interval), String> {
    let (p, v, propulsion, interval) = drift(
        d,
        state,
        position,
        velocity,
        command,
        gravity(position, initial)?,
    )?;
    let velocity = v + gravity(p, final_sources)? * (command.duration_s / 2.);
    if !finite(p) || !finite(velocity) {
        return Err("Unsupported finite propagated flight state".into());
    }
    Ok((p, velocity, propulsion, interval))
}
pub(crate) fn drift(
    d: &crate::supply_vehicle::Definition,
    state: &State,
    position: Vec3d,
    velocity: Vec3d,
    command: &Command,
    acceleration: Vec3d,
) -> Result<(Vec3d, Vec3d, State, Interval), String> {
    let mut propulsion = state.clone();
    let interval = propulsion.apply(d, command)?;
    let v = velocity + acceleration * (command.duration_s / 2.);
    let direction = Vec3d::new(
        command.direction[0],
        command.direction[1],
        command.direction[2],
    );
    let p =
        position + v * command.duration_s + direction * powered_displacement(d, state, &interval);
    let velocity = v + direction * interval.ideal_delta_v_m_s;
    if !finite(p) || !finite(velocity) {
        return Err("Unsupported finite powered drift".into());
    }
    Ok((p, velocity, propulsion, interval))
}
fn gravity_contact(
    position: Vec3d,
    sources: &[Attractor],
    contact: usize,
    tolerance: f64,
) -> Result<Vec3d, String> {
    let mut acceleration = Vec3d::ZERO;
    for (index, b) in sources.iter().enumerate() {
        let delta = b.position_m - position;
        let r = delta.length();
        if r == 0. || (r <= b.radius_m && (index != contact || r < b.radius_m - 2. * tolerance)) {
            return Err("Contact force sample is outside the supported boundary".into());
        }
        acceleration += delta * (b.mu / (r * r * r));
    }
    if !finite(acceleration) {
        return Err("Unsupported finite contact acceleration".into());
    }
    Ok(acceleration)
}
pub(crate) fn exhaustion_time(fuel: f64, rate: f64) -> f64 {
    let mut t = fuel / rate;
    // Round the computed cutoff upward only until represented consumption reaches
    // the held inventory. Avoid leaving an unadvanceable positive one-ulp residue.
    for _ in 0..3 {
        if rate * t >= fuel {
            break;
        }
        t = f64::from_bits(t.to_bits() + 1);
    }
    t
}
/// Earliest supported contact of a curved craft path with a linearly sampled body.
/// Curve/chord deviation is bounded by max_acceleration*duration²/8. Numerical
/// contact tolerance is explicit; exceeding the recursion/evaluation budget refuses.
fn first_contact(
    path: &impl Fn(f64) -> Result<Vec3d, String>,
    centre_start: Vec3d,
    centre_end: Vec3d,
    radius: f64,
    duration: f64,
    max_acceleration: f64,
    tolerance: f64,
) -> Result<Option<f64>, String> {
    if duration <= 0.
        || !duration.is_finite()
        || !radius.is_finite()
        || radius <= 0.
        || !max_acceleration.is_finite()
        || max_acceleration < 0.
        || !tolerance.is_finite()
        || tolerance <= 0.
    {
        return Err("Unsupported contact search domain".into());
    }
    struct Search<'a, F> {
        path: &'a F,
        a: Vec3d,
        b: Vec3d,
        r: f64,
        dt: f64,
        acc: f64,
        tol: f64,
        left: usize,
    }
    impl<F: Fn(f64) -> Result<Vec3d, String>> Search<'_, F> {
        fn relative(&self, t: f64) -> Result<Vec3d, String> {
            Ok((self.path)(t)? - self.a.lerp(self.b, t / self.dt))
        }
        fn visit(
            &mut self,
            lo: f64,
            hi: f64,
            a: Vec3d,
            b: Vec3d,
            depth: u32,
        ) -> Result<Option<f64>, String> {
            let delta = b - a;
            let length = delta.length_squared();
            let f = if length > 0. {
                (-a.dot(delta) / length).clamp(0., 1.)
            } else {
                0.
            };
            let deviation = self.acc * (hi - lo) * (hi - lo) / 8.;
            if (a + delta * f).length() > self.r + self.tol + deviation {
                return Ok(None);
            }
            if a.length() <= self.r + self.tol {
                return Ok(Some(lo));
            }
            if deviation <= self.tol / 4. {
                // The inflated radius includes the certified curve deviation. Keep the
                // actual curved point and residual; this is tolerance-based contact.
                let r = self.r + self.tol + deviation;
                let aa = delta.length_squared();
                let bb = a.dot(delta);
                let cc = a.length_squared() - r * r;
                if aa == 0. {
                    return Ok(None);
                }
                let disc = bb * bb - aa * cc;
                if disc < 0. {
                    return Ok(None);
                }
                let fraction = ((-bb - disc.sqrt()) / aa).clamp(0., 1.);
                return Ok(Some(lo + (hi - lo) * fraction));
            }
            if depth >= 48 || self.left == 0 {
                return Err("Contact search exceeded supported resolution budget; flight must remain unchanged".into());
            }
            self.left -= 1;
            let mid = (lo + hi) / 2.;
            if mid == lo || mid == hi {
                return Err("Contact search is below time representation precision".into());
            }
            let point = self.relative(mid)?;
            if let Some(t) = self.visit(lo, mid, a, point, depth + 1)? {
                return Ok(Some(t));
            }
            self.visit(mid, hi, point, b, depth + 1)
        }
    }
    let mut search = Search {
        path,
        a: centre_start,
        b: centre_end,
        r: radius,
        dt: duration,
        acc: max_acceleration,
        tol: tolerance,
        left: 256,
    };
    let a = search.relative(0.)?;
    let b = search.relative(duration)?;
    search.visit(0., duration, a, b, 0)
}

fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn launched_world() -> Universe {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let d = Definition::circular_example(&u, BodyRef { system: 0, body: 3 }, Plan::example())
            .unwrap();
        u.apply_edit(crate::sandbox::Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        u
    }
    #[test]
    fn authoritative_world_flight_is_exact_across_time_chunks_and_saved_continuation() {
        let mut one = launched_world();
        let mut chunks = one.clone();
        let mut resumed = one.clone();
        let start = one.time;
        one.advance_to(start + 360., None, None);
        for n in 1..=720 {
            chunks.advance_to(start + n as f64 * 0.5, None, None);
        }
        resumed.advance_to(start + 120.25, None, None);
        resumed =
            crate::save::from_json(&crate::save::to_json(&resumed, "mid-flight").unwrap()).unwrap();
        resumed.advance_to(start + 360., None, None);
        let bytes = |u: &Universe| serde_json::to_string(u).unwrap();
        assert_eq!(bytes(&one), bytes(&chunks));
        assert_eq!(bytes(&one), bytes(&resumed));
        assert_eq!(one.vehicles[0].propulsion.propellant_kg, 0.);
        assert_eq!(one.vehicles[0].phase, Phase::Flying);
        assert_eq!(one.vehicles[0].propulsion.elapsed_s, 360.);
        assert!(one.habitats.is_empty());
        one.validate_vehicle_state().unwrap();
    }
    #[test]
    fn habitat_vehicle_task_slots_survive_creation_removal_and_reload() {
        let mut u = launched_world();
        assert_eq!(
            u.scheduler.tasks[crate::habitat::TASK].period,
            crate::scheduler::NEVER
        );
        let d = crate::habitat::Definition::at(BodyRef { system: 0, body: 3 }, 0., 0.);
        u.apply_edit(crate::sandbox::Edit::CreateHabitat {
            definition: Box::new(d.clone()),
        })
        .unwrap();
        assert_eq!(
            u.scheduler.tasks[crate::habitat::TASK].period,
            crate::habitat::PERIOD
        );
        u.advance_by(2.25);
        let vehicle_clock = u.vehicles[0].last_time_s;
        u.apply_edit(crate::sandbox::Edit::RemoveHabitat { system: 0, id: 0 })
            .unwrap();
        assert_eq!(u.scheduler.tasks[TASK].name, "vehicles");
        assert_eq!(
            u.scheduler.tasks[crate::habitat::TASK].period,
            crate::scheduler::NEVER
        );
        u = crate::save::from_json(&crate::save::to_json(&u, "reserved slot").unwrap()).unwrap();
        u.advance_by(1.);
        assert!(u.vehicles[0].last_time_s > vehicle_clock);
        u.apply_edit(crate::sandbox::Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        assert_eq!(
            u.scheduler.tasks[crate::habitat::TASK].period,
            crate::habitat::PERIOD
        );
        u.apply_edit(crate::sandbox::Edit::RemoveVehicle { system: 0, id: 0 })
            .unwrap();
        assert_eq!(u.scheduler.tasks.len(), TASK);
        assert_eq!(
            u.scheduler.tasks[crate::habitat::TASK].period,
            crate::habitat::PERIOD
        );
        u.apply_edit(crate::sandbox::Edit::RemoveHabitat { system: 0, id: 1 })
            .unwrap();
        assert_eq!(u.scheduler.tasks.len(), crate::habitat::TASK);
        u = crate::save::from_json(&crate::save::to_json(&u, "removed fleet").unwrap()).unwrap();
        assert!(u.vehicles.is_empty());
        assert_eq!(u.next_vehicle_id, 1);
        assert!(crate::save::data_versions(&u).contains_key("cosmogon-finite-world-vehicle"));
    }
    #[test]
    fn live_impact_parks_scheduler_and_retains_exact_historical_event() {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = BodyRef { system: 0, body: 3 };
        let mut d = Definition::circular_example(&u, parent, Plan::example()).unwrap();
        d.initial_relative_position_m = Vec3d::new(u.body(parent).radius + 1000., 0., 0.);
        d.initial_relative_velocity_m_s = Vec3d::new(-10000., 0., 0.);
        u.apply_edit(crate::sandbox::Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        u.advance_by(1.);
        assert!(matches!(u.vehicles[0].phase, Phase::Impact { .. }));
        assert_eq!(u.scheduler.tasks[TASK].period, crate::scheduler::NEVER);
        let retained = u.vehicles[0].clone();
        let event_count = u.history.events.len();
        u = crate::save::from_json(&crate::save::to_json(&u, "impact").unwrap()).unwrap();
        u.advance_by(3.);
        assert_eq!(u.vehicles[0], retained);
        assert_eq!(u.history.events.len(), event_count);
        let d = Definition::circular_example(&u, parent, Plan::example()).unwrap();
        u.apply_edit(crate::sandbox::Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        assert_eq!(u.scheduler.tasks[TASK].period, PERIOD);
        u.advance_by(1.);
        assert_eq!(u.vehicles[0], retained);
        assert!(u.vehicles[1].propulsion.elapsed_s > 0.);
    }
    #[test]
    fn corrupt_vehicle_save_pins_counters_clocks_contacts_and_task_slots_are_refused() {
        let mut u = launched_world();
        u.advance_by(4.25);
        let original: serde_json::Value =
            serde_json::from_str(&crate::save::to_json(&u, "flight").unwrap()).unwrap();
        for (pointer, value) in [
            (
                "/header/data_versions/cosmogon-finite-world-vehicle",
                serde_json::json!("wrong"),
            ),
            (
                "/header/data_versions/cosmogon-supply-vehicle-reference",
                serde_json::json!("wrong"),
            ),
            ("/universe/vehicles/0/id", serde_json::json!(5)),
            (
                "/universe/vehicles/0/propulsion/propellant_kg",
                serde_json::json!(999999),
            ),
            (
                "/universe/vehicles/0/propulsion/exhausted_propellant_kg",
                serde_json::json!(999999),
            ),
            (
                "/universe/vehicles/0/propulsion/cumulative_impulse_n_s",
                serde_json::json!(0),
            ),
            (
                "/universe/vehicles/0/launched_at_s",
                serde_json::json!(u.time + 100.),
            ),
            (
                "/universe/vehicles/0/last_time_s",
                serde_json::json!(u.time + 100.),
            ),
            (
                "/universe/vehicles/0/definition/frame",
                serde_json::json!("body-fixed"),
            ),
            (
                "/universe/vehicles/0/definition/parent/body",
                serde_json::json!(999999),
            ),
            (
                "/universe/vehicles/0/last_interval/final_mass_kg",
                serde_json::json!(1),
            ),
            (
                "/universe/scheduler/tasks/5/steps",
                serde_json::json!(u64::MAX),
            ),
            (
                "/universe/scheduler/tasks/5/name",
                serde_json::json!("habitats"),
            ),
            ("/universe/scheduler/tasks/4/period", serde_json::json!(1)),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            assert!(
                crate::save::from_json(&changed.to_string()).is_err(),
                "Accepted corrupt field {pointer}"
            );
        }
        let mut changed = original.clone();
        changed["universe"]["vehicles"][0]["phase"] = serde_json::json!({"kind":"impact","source":{"kind":"body","body_index":3},"source_position_m":{"x":0,"y":0,"z":0},"radius_m":1,"position_residual_m":0,"tolerance_m":0.001,"source_sample_duration_s":0.25});
        assert!(crate::save::from_json(&changed.to_string()).is_err());
        // Original authoring assumptions are historical; later ordinary edits do not
        // rewrite the launch conditions or make the retained world unloadable.
        u.systems[0].bodies[3].radius *= 100.;
        assert!(
            crate::save::from_json(&crate::save::to_json(&u, "changed parent").unwrap()).is_ok()
        );
    }
    #[test]
    fn invalid_launch_and_counter_overflow_are_atomic_and_legacy_state_is_absent() {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let bytes = serde_json::to_string(&u).unwrap();
        assert!(!bytes.contains("next_vehicle_id"));
        assert!(!bytes.contains("\"vehicles\""));
        let mut d =
            Definition::circular_example(&u, BodyRef { system: 0, body: 3 }, Plan::example())
                .unwrap();
        d.initial_relative_position_m = Vec3d::ZERO;
        assert!(u
            .apply_edit(crate::sandbox::Edit::CreateVehicle {
                definition: Box::new(d.clone())
            })
            .is_err());
        assert_eq!(bytes, serde_json::to_string(&u).unwrap());
        d = Definition::circular_example(&u, BodyRef { system: 0, body: 3 }, Plan::example())
            .unwrap();
        u.next_vehicle_id = u64::MAX;
        let before = serde_json::to_string(&u).unwrap();
        assert!(u
            .apply_edit(crate::sandbox::Edit::CreateVehicle {
                definition: Box::new(d)
            })
            .is_err());
        assert_eq!(before, serde_json::to_string(&u).unwrap());
    }
    #[test]
    fn changed_sources_and_unsupported_speed_freeze_the_last_actual_world_state() {
        let mut u = launched_world();
        u.advance_by(1.25);
        let retained = u.vehicles[0].clone();
        u.apply_edit(crate::sandbox::Edit::SetProperty {
            body: BodyRef { system: 0, body: 3 },
            property: crate::sandbox::BodyProperty::Mass(u.systems[0].bodies[3].mass * 0.5),
        })
        .unwrap();
        u.advance_by(1.);
        let stopped = &u.vehicles[0];
        assert!(matches!(stopped.phase, Phase::Unsupported { .. }));
        assert_eq!(stopped.position_m, retained.position_m);
        assert_eq!(stopped.velocity_m_s, retained.velocity_m_s);
        assert_eq!(stopped.propulsion, retained.propulsion);
        assert_eq!(u.scheduler.tasks[TASK].period, crate::scheduler::NEVER);
        assert!(
            crate::save::from_json(&crate::save::to_json(&u, "source discontinuity").unwrap())
                .is_ok()
        );
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut plan = Plan::example();
        plan.definition.dry_mass_kg = 1.;
        plan.definition.cargo = crate::supply_vehicle::Cargo {
            oxygen_reserve_kg: 0.,
            potable_water_kg: 0.,
            food_dry_kg: 0.,
            food_water_kg: 0.,
        };
        plan.definition.initial_propellant_kg = 10.;
        plan.definition.effective_isp_s = 1e5;
        plan.definition.rated_thrust_n = 1e9;
        let d = Definition::circular_example(&u, BodyRef { system: 0, body: 3 }, plan).unwrap();
        u.apply_edit(crate::sandbox::Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        let original = u.vehicles[0].clone();
        u.advance_by(1.);
        assert!(matches!(u.vehicles[0].phase, Phase::Unsupported { .. }));
        assert_eq!(u.vehicles[0].propulsion, original.propulsion);
        assert_eq!(u.vehicles[0].last_time_s, original.last_time_s);
        assert!(
            crate::save::from_json(&crate::save::to_json(&u, "unsupported speed").unwrap()).is_ok()
        );
    }
    #[test]
    fn terminal_flight_parking_is_exact_across_fractional_caller_chunks() {
        let mut one = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = BodyRef { system: 0, body: 3 };
        let mut d = Definition::circular_example(&one, parent, Plan::example()).unwrap();
        d.initial_relative_position_m = Vec3d::new(one.body(parent).radius + 1000., 0., 0.);
        d.initial_relative_velocity_m_s = Vec3d::new(-10000., 0., 0.);
        one.apply_edit(crate::sandbox::Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        let mut chunks = one.clone();
        let start = one.time;
        one.advance_to(start + 4., None, None);
        for n in 1..=40 {
            chunks.advance_to(start + n as f64 * 0.1, None, None);
        }
        assert_eq!(
            serde_json::to_vec(&one).unwrap(),
            serde_json::to_vec(&chunks).unwrap()
        );
        assert!(matches!(one.vehicles[0].phase, Phase::Impact { .. }));
        assert_eq!(one.scheduler.tasks[TASK].origin, start + 1.);
        let restored =
            crate::save::from_json(&crate::save::to_json(&chunks, "chunked impact").unwrap())
                .unwrap();
        assert_eq!(
            serde_json::to_vec(&one).unwrap(),
            serde_json::to_vec(&restored).unwrap()
        );
    }
    #[test]
    fn physical_contact_retains_only_pre_contact_propellant_and_freezes_the_event() {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = BodyRef { system: 0, body: 3 };
        let mut d = Definition::circular_example(&u, parent, Plan::example()).unwrap();
        d.initial_relative_position_m = Vec3d::new(u.body(parent).radius + 1000., 0., 0.);
        d.initial_relative_velocity_m_s = Vec3d::new(-10000., 0., 0.);
        d.plan.commands[0].direction = [-1., 0., 0.];
        let mut flight = Flight::new(0, d, &u).unwrap();
        let cargo = flight.definition.plan.definition.cargo.clone();
        flight.advance_system(&u.systems[0], u.time + 1.).unwrap();
        match &flight.phase {
            Phase::Impact {
                source,
                position_residual_m,
                tolerance_m,
                ..
            } => {
                assert_eq!(*source, Source::Body(3));
                assert!(position_residual_m.abs() <= 2. * tolerance_m);
            }
            _ => panic!("Physical crossing was not retained: {:?}", flight.phase),
        }
        assert!(flight.propulsion.elapsed_s > 0. && flight.propulsion.elapsed_s < 0.11);
        let rate = flight.definition.plan.definition.rated_thrust_n
            / flight.definition.plan.definition.exhaust_velocity_m_s();
        assert!(
            (flight.propulsion.exhausted_propellant_kg - rate * flight.propulsion.elapsed_s).abs()
                < 1e-9
        );
        assert_eq!(flight.definition.plan.definition.cargo, cargo);
        let retained = flight.clone();
        flight.advance_system(&u.systems[0], u.time + 100.).unwrap();
        assert_eq!(flight, retained);
        assert!(flight.last_time_s < u.time + 1.);
    }
    #[test]
    fn live_sampled_gravity_and_fuel_cutoff_do_not_leave_a_precision_residue() {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = BodyRef { system: 0, body: 3 };
        let plan = Plan::example();
        let d = Definition::circular_example(&u, parent, plan.clone()).unwrap();
        let mut flight = Flight::new(0, d, &u).unwrap();
        let start = u.time;
        for step in 1..=360 {
            u.advance_to(start + step as f64, None, None);
            flight.advance_system(&u.systems[0], u.time).unwrap();
        }
        assert_eq!(flight.phase, Phase::Flying);
        assert_eq!(flight.propulsion.propellant_kg, 0.);
        assert!(
            (flight.propulsion.cumulative_ideal_delta_v_m_s
                - plan.definition.initial_ideal_delta_v_m_s())
            .abs()
                < 1e-8
        );
        assert_eq!(flight.propulsion.elapsed_s, 360.);
        assert!(flight.propulsion.fuel_limited_duration_s > 0.);
        assert_eq!(
            flight.definition.plan.definition.cargo,
            plan.definition.cargo
        );
        assert!(u.habitats.is_empty());
    }
    #[test]
    fn swept_curved_contacts_catch_pass_through_and_refuse_false_chord_contacts() {
        let line = |t: f64| Ok(Vec3d::new(-10. + 20. * t, 0., 0.));
        let t = first_contact(&line, Vec3d::ZERO, Vec3d::ZERO, 1., 1., 0., 1e-4)
            .unwrap()
            .unwrap();
        assert!((t - 0.45).abs() < 1e-5);
        let moving = first_contact(
            &line,
            Vec3d::new(1., 0., 0.),
            Vec3d::new(2., 0., 0.),
            1.,
            1.,
            0.,
            1e-4,
        )
        .unwrap()
        .unwrap();
        assert!((moving - 10. / 19.).abs() < 1e-5);
        let miss = |t: f64| Ok(Vec3d::new(-10. + 20. * t, 0.9 + 4. * t * (1. - t), 0.));
        assert!(
            first_contact(&miss, Vec3d::ZERO, Vec3d::ZERO, 1., 1., 8., 1e-4)
                .unwrap()
                .is_none()
        );
        let graze = |t: f64| Ok(Vec3d::new(-10. + 20. * t, 2. - 4. * t * (1. - t), 0.));
        let t = first_contact(&graze, Vec3d::ZERO, Vec3d::ZERO, 1., 1., 8., 1e-4)
            .unwrap()
            .unwrap();
        assert!((t - 0.5).abs() < 0.001);
        assert!((graze(t).unwrap().length() - 1.).abs() < 1.3e-4);
        assert!(first_contact(&line, Vec3d::ZERO, Vec3d::ZERO, 1., 1., f64::NAN, 1e-4).is_err());
    }
    #[test]
    fn companion_gravity_does_not_mix_galactic_and_local_frames() {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::Sol,
            ..Default::default()
        });
        let mut system = u.systems.remove(0);
        system.companion = Some(crate::astro::Companion {
            star: system.star.clone(),
            orbit: crate::astro::Orbit::circular(1e12, 0.),
        });
        let t = u.time;
        let sources = gravity_sources(&system, t).unwrap();
        let sample = Vec3d::new(5e11, 2e11, 0.);
        let force = gravity(sample, &sources).unwrap();
        system.position = Vec3d::new(1e24, -2e24, 3e24);
        let shifted = gravity_sources(&system, t).unwrap();
        assert_eq!(force, gravity(sample, &shifted).unwrap());
        assert_eq!(sources[1].position_m, shifted[1].position_m);
    }
    #[test]
    fn changing_mass_displacement_matches_analytic_vacuum_and_small_ratio() {
        let d = crate::supply_vehicle::Definition::example();
        let state = State::new(&d).unwrap();
        let c = Command {
            duration_s: 30.,
            throttle: 1.,
            direction: [1., 0., 0.],
        };
        let (p, v, s, i) = integrate(
            &d,
            &state,
            Vec3d::ZERO,
            Vec3d::new(10., 0., 0.),
            &c,
            &[],
            &[],
        )
        .unwrap();
        let q = d.rated_thrust_n / d.exhaust_velocity_m_s();
        let mf = s.mass_kg(&d);
        let analytic =
            d.exhaust_velocity_m_s() * (c.duration_s - mf / q * (state.mass_kg(&d) / mf).dln());
        assert!((p.x - 10. * c.duration_s - analytic).abs() < 1e-8);
        assert_eq!(v.x, 10. + i.ideal_delta_v_m_s);
        assert_eq!(s.exhausted_propellant_kg, i.actual_propellant_kg);
        let mut heavy = d;
        heavy.dry_mass_kg = 1e12;
        heavy.initial_propellant_kg = 1.;
        let state = State::new(&heavy).unwrap();
        let command = Command {
            duration_s: 0.001,
            ..c
        };
        let (p, v, _, i) =
            integrate(&heavy, &state, Vec3d::ZERO, Vec3d::ZERO, &command, &[], &[]).unwrap();
        assert!(p.x > 0. && v.x > 0.);
        assert!((p.x / (i.ideal_delta_v_m_s * command.duration_s / 2.) - 1.).abs() < 1e-12);
    }
    #[test]
    fn fixed_gravity_steps_converge_and_preserve_angular_momentum() {
        let sources = [Attractor {
            source: Source::Star,
            position_m: Vec3d::ZERO,
            mu: 1e10,
            radius_m: 1000.,
        }];
        let mut d = crate::supply_vehicle::Definition::example();
        d.rated_thrust_n = 0.;
        let r = 100_000.;
        let omega = (sources[0].mu / (r * r * r)).sqrt();
        let expected = Vec3d::new(r * (omega * 100.).dcos(), r * (omega * 100.).dsin(), 0.);
        let run = |h: f64| {
            let mut p = Vec3d::new(r, 0., 0.);
            let mut v = Vec3d::new(0., (sources[0].mu / r).sqrt(), 0.);
            let mut s = State::new(&d).unwrap();
            for _ in 0..(100. / h) as usize {
                let (pp, vv, ss, _) = integrate(
                    &d,
                    &s,
                    p,
                    v,
                    &Command {
                        duration_s: h,
                        throttle: 0.,
                        direction: [1., 0., 0.],
                    },
                    &sources,
                    &sources,
                )
                .unwrap();
                p = pp;
                v = vv;
                s = ss;
            }
            assert!((p.cross(v).z / (r * (sources[0].mu / r).sqrt()) - 1.).abs() < 1e-12);
            (p - expected).length()
        };
        let coarse = run(1.);
        let fine = run(0.25);
        assert!(fine < coarse / 10.);
    }
    #[test]
    fn actual_world_translation_velocity_and_invalid_steps_are_explicit() {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = BodyRef { system: 0, body: 3 };
        let d = Definition::circular_example(&u, parent, Plan::example()).unwrap();
        let centre = u.systems[0].body_state(3, u.time);
        let mut flight = Flight::new(0, d.clone(), &u).unwrap();
        assert_eq!(
            flight.position_m,
            centre.pos + d.initial_relative_position_m
        );
        assert_eq!(
            flight.velocity_m_s,
            centre.vel + d.initial_relative_velocity_m_s
        );
        let before = flight.clone();
        assert!(flight.advance_system(&u.systems[0], u.time + 2.).is_err());
        assert_eq!(flight, before);
        flight.advance_system(&u.systems[0], u.time + 1.).unwrap();
        assert!(flight.propulsion.propellant_kg < before.propulsion.propellant_kg);
        assert_eq!(
            flight.definition.plan.definition.cargo,
            before.definition.plan.definition.cargo
        );
        assert_ne!(flight.position_m, before.position_m);
    }
}
