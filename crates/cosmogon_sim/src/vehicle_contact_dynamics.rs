//! Versioned six-degree-of-freedom motion against the existing rotating height field.
//! Authored compliance and inertia, not calibrated landing hardware or attachment.
use crate::{
    supply_vehicle::Command,
    vehicle_contact::{BodyPose, Foot, FootSample, MODEL},
    world_vehicle::{self, Flight, GravitySample, Phase, Source},
    BodyRef, Universe, Vec3d,
};
use cosmogon_core::{dmath::DMath, math::Quatd};
use serde::{Deserialize, Serialize};

pub const MAX_STEPS_PER_SECOND: usize = 10_000;
fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}
fn q(v: [f64; 4]) -> Quatd {
    Quatd::new(v[0], v[1], v[2], v[3])
}
fn array(q: Quatd) -> [f64; 4] {
    [q.x, q.y, q.z, q.w]
}
fn inverse(q: Quatd) -> Quatd {
    Quatd::new(-q.x, -q.y, -q.z, q.w)
}
fn multiply(a: Quatd, b: Quatd) -> Quatd {
    Quatd::new(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    )
}
fn rotate(q0: Quatd, body_omega: Vec3d, h: f64) -> Result<Quatd, String> {
    let speed = body_omega.length();
    if !speed.is_finite() || speed > 1000. {
        return Err("Rigid-body angular speed exceeds 1000 rad/s".into());
    }
    if speed == 0. {
        return Ok(q0);
    }
    let (s, c) = (speed * h * 0.5).dsin_cos();
    let axis = body_omega * (s / speed);
    let r = multiply(q0, Quatd::new(axis.x, axis.y, axis.z, c));
    let n = (r.x * r.x + r.y * r.y + r.z * r.z + r.w * r.w).sqrt();
    if !n.is_finite() || n <= 0. {
        return Err("Unsupported quaternion increment".into());
    }
    Ok(Quatd::new(r.x / n, r.y / n, r.z / n, r.w / n))
}
fn diagonal(v: Vec3d, d: Vec3d) -> Vec3d {
    Vec3d::new(v.x * d.x, v.y * d.y, v.z * d.z)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hardware {
    pub model: String,
    /// Fixed authored mass distribution: I_i = actual mass * gyration_radius_i².
    pub gyration_radii_m: Vec3d,
    pub initial_orientation_xyzw: [f64; 4],
    pub initial_angular_velocity_body_rad_s: Vec3d,
    /// Spherical hull around the centre, independently checked against terrain.
    pub hull_radius_m: f64,
    pub feet: Vec<Foot>,
    pub normal_probe_m: f64,
    pub maximum_step_s: f64,
}
impl Hardware {
    /// Authored four-foot example aligned to the actual local modeled normal.
    /// Numerical parameters are editable assumptions, not a measured vehicle.
    pub fn example(world: &Universe, target: BodyRef, point: Vec3d) -> Result<Self, String> {
        let surface =
            crate::vehicle_contact::Surface::sample(world, target, world.time, point, 1.)?;
        let n = surface.outward_normal;
        let rotation = if n.z < -1. + 1e-12 {
            Quatd::new(1., 0., 0., 0.)
        } else {
            let v = Vec3d::new(-n.y, n.x, 0.);
            let w = 1. + n.z;
            let length = (v.length_squared() + w * w).sqrt();
            Quatd::new(v.x / length, v.y / length, v.z / length, w / length)
        };
        let leg = crate::vehicle_contact::Leg {
            stiffness_n_m: 15000.,
            damping_n_s_m: 2000.,
            maximum_stroke_m: 0.5,
            maximum_load_n: 100000.,
            sliding_friction: 0.5,
            slip_speed_scale_m_s: 0.05,
        };
        let result = Self {
            model: MODEL.into(),
            gyration_radii_m: Vec3d::new(1.5, 1.5, 1.5),
            initial_orientation_xyzw: array(rotation),
            initial_angular_velocity_body_rad_s: inverse(rotation)
                .rotate_vec3(surface.frame.angular_velocity_inertial_rad_s),
            hull_radius_m: 0.5,
            feet: [(-1.5, -1.5), (-1.5, 1.5), (1.5, -1.5), (1.5, 1.5)]
                .into_iter()
                .map(|(x, y)| Foot {
                    offset_body_m: Vec3d::new(x, y, -1.5),
                    response: leg.clone(),
                })
                .collect(),
            normal_probe_m: 1.,
            maximum_step_s: 0.005,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let value: Self = serde_json::from_str(&crate::habitat_recording::read(path, 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        let g = self.gyration_radii_m;
        let d = diagonal(g, g);
        let orientation = self.initial_orientation_xyzw;
        if self.model != MODEL
            || !finite(g)
            || [g.x, g.y, g.z].iter().any(|v| !(0.01..=100.).contains(v))
            || d.x > d.y + d.z
            || d.y > d.x + d.z
            || d.z > d.x + d.y
            || orientation.iter().any(|v| !v.is_finite())
            || (orientation.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 1e-10
            || !finite(self.initial_angular_velocity_body_rad_s)
            || self.initial_angular_velocity_body_rad_s.length() > 1000.
            || !self.hull_radius_m.is_finite()
            || !(0.01..=100.).contains(&self.hull_radius_m)
            || !(1..=8).contains(&self.feet.len())
            || !self.normal_probe_m.is_finite()
            || !(0.01..=1000.).contains(&self.normal_probe_m)
            || !self.maximum_step_s.is_finite()
            || !(0.0001..=0.02).contains(&self.maximum_step_s)
        {
            return Err("Contact hardware requires supported physical inertia, unit orientation, hull, 1–8 feet, probe and step bounds".into());
        }
        let extent = self
            .feet
            .iter()
            .map(|f| f.offset_body_m.length())
            .fold(self.hull_radius_m, f64::max);
        if 0.5 * (d.x + d.y + d.z) > extent * extent {
            return Err("Authored inertia cannot fit within the hull/gear bounding extent".into());
        }
        for foot in &self.feet {
            foot.response.validate()?;
            if !finite(foot.offset_body_m) || foot.offset_body_m.length() > 100. {
                return Err("Foot offset exceeds the 100 m contact hardware bound".into());
            }
        }
        Ok(())
    }
    fn inertia_per_mass(&self) -> Vec3d {
        diagonal(self.gyration_radii_m, self.gyration_radii_m)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub duration_s: f64,
    pub substeps: u64,
    pub contact_force_work_j: f64,
    pub contact_torque_work_j: f64,
    pub moving_ground_work_j: f64,
    pub elastic_storage_change_j: f64,
    pub normal_dissipation_j: f64,
    pub sliding_dissipation_j: f64,
    /// Actual finite storage change + midpoint work/dissipation; not forced to zero.
    pub contact_balance_residual_j: f64,
    pub translational_kinetic_change_j: f64,
    pub gravity_work_j: f64,
    pub central_thrust_work_j: f64,
    pub exhausted_translational_kinetic_j: f64,
    pub translational_balance_residual_j: f64,
    pub rotational_kinetic_change_j: f64,
    pub exhausted_rotational_kinetic_j: f64,
    pub rotational_balance_residual_j: f64,
    pub angular_impulse_n_m_s: Vec3d,
    pub exhausted_angular_momentum_kg_m2_s: Vec3d,
    pub angular_momentum_residual_kg_m2_s: Vec3d,
    pub maximum_compression_m: f64,
    pub maximum_resultant_load_n: f64,
}
impl Work {
    fn add(&mut self, b: &Self) {
        self.duration_s += b.duration_s;
        self.substeps += b.substeps;
        self.contact_force_work_j += b.contact_force_work_j;
        self.contact_torque_work_j += b.contact_torque_work_j;
        self.moving_ground_work_j += b.moving_ground_work_j;
        self.elastic_storage_change_j += b.elastic_storage_change_j;
        self.normal_dissipation_j += b.normal_dissipation_j;
        self.sliding_dissipation_j += b.sliding_dissipation_j;
        self.contact_balance_residual_j += b.contact_balance_residual_j;
        self.translational_kinetic_change_j += b.translational_kinetic_change_j;
        self.gravity_work_j += b.gravity_work_j;
        self.central_thrust_work_j += b.central_thrust_work_j;
        self.exhausted_translational_kinetic_j += b.exhausted_translational_kinetic_j;
        self.translational_balance_residual_j += b.translational_balance_residual_j;
        self.rotational_kinetic_change_j += b.rotational_kinetic_change_j;
        self.exhausted_rotational_kinetic_j += b.exhausted_rotational_kinetic_j;
        self.rotational_balance_residual_j += b.rotational_balance_residual_j;
        self.angular_impulse_n_m_s += b.angular_impulse_n_m_s;
        self.exhausted_angular_momentum_kg_m2_s += b.exhausted_angular_momentum_kg_m2_s;
        self.angular_momentum_residual_kg_m2_s += b.angular_momentum_residual_kg_m2_s;
        self.maximum_compression_m = self.maximum_compression_m.max(b.maximum_compression_m);
        self.maximum_resultant_load_n = self
            .maximum_resultant_load_n
            .max(b.maximum_resultant_load_n);
    }
    fn validate(&self) -> Result<(), String> {
        if [
            self.duration_s,
            self.contact_force_work_j,
            self.contact_torque_work_j,
            self.moving_ground_work_j,
            self.elastic_storage_change_j,
            self.normal_dissipation_j,
            self.sliding_dissipation_j,
            self.contact_balance_residual_j,
            self.maximum_compression_m,
            self.maximum_resultant_load_n,
            self.translational_kinetic_change_j,
            self.gravity_work_j,
            self.central_thrust_work_j,
            self.exhausted_translational_kinetic_j,
            self.translational_balance_residual_j,
            self.rotational_kinetic_change_j,
            self.exhausted_rotational_kinetic_j,
            self.rotational_balance_residual_j,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || [
                self.duration_s,
                self.normal_dissipation_j,
                self.sliding_dissipation_j,
                self.maximum_compression_m,
                self.maximum_resultant_load_n,
            ]
            .iter()
            .any(|v| *v < 0.)
            || !finite(self.angular_impulse_n_m_s)
            || !finite(self.exhausted_angular_momentum_kg_m2_s)
            || !finite(self.angular_momentum_residual_kg_m2_s)
            || (self.contact_force_work_j + self.contact_torque_work_j - self.moving_ground_work_j
                + self.elastic_storage_change_j
                + self.normal_dissipation_j
                + self.sliding_dissipation_j
                - self.contact_balance_residual_j)
                .abs()
                > 1e-8
                    * (1.
                        + self.contact_force_work_j.abs()
                        + self.contact_torque_work_j.abs()
                        + self.moving_ground_work_j.abs()
                        + self.elastic_storage_change_j.abs()
                        + self.normal_dissipation_j
                        + self.sliding_dissipation_j)
        {
            return Err("Invalid retained contact work evidence".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidState {
    pub model: String,
    pub orientation_xyzw: [f64; 4],
    pub angular_velocity_body_rad_s: Vec3d,
    pub feet: Vec<FootSample>,
    pub last_work: Work,
    pub cumulative_work: Work,
}
impl RigidState {
    pub(crate) fn new(
        world: &Universe,
        target: BodyRef,
        position: Vec3d,
        velocity: Vec3d,
        h: &Hardware,
    ) -> Result<Self, String> {
        h.validate()?;
        let mut result = Self {
            model: MODEL.into(),
            orientation_xyzw: h.initial_orientation_xyzw,
            angular_velocity_body_rad_s: h.initial_angular_velocity_body_rad_s,
            feet: vec![],
            last_work: Work::default(),
            cumulative_work: Work::default(),
        };
        result.feet = sample(world, target, world.time, position, velocity, &result, h)?;
        if result.feet.iter().any(|f| f.surface.tangent_gap_m < 0.) {
            return Err("Initial contact feet must be outside modeled terrain; do not invent preloaded elastic energy".into());
        }
        Ok(result)
    }
    pub(crate) fn validate(&self, h: &Hardware, flight: &Flight) -> Result<(), String> {
        h.validate()?;
        if self.model != MODEL
            || self.orientation_xyzw.iter().any(|v| !v.is_finite())
            || (self.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 1e-10
            || !finite(self.angular_velocity_body_rad_s)
            || self.angular_velocity_body_rad_s.length() > 1000.
            || self.feet.len() != h.feet.len()
        {
            return Err("Invalid retained rigid contact state".into());
        }
        self.last_work.validate()?;
        self.cumulative_work.validate()?;
        if (self.cumulative_work.duration_s - flight.propulsion.elapsed_s).abs()
            > 1e-8 * (1. + flight.propulsion.elapsed_s)
            || self.last_work.duration_s > 1.000001
            || self.last_work.substeps > MAX_STEPS_PER_SECOND as u64
            || self.cumulative_work.substeps as f64
                > (flight.propulsion.elapsed_s.ceil() + 1.) * MAX_STEPS_PER_SECOND as f64
        {
            return Err(
                "Contact work clocks or substep support do not match retained flight".into(),
            );
        }
        let pose = BodyPose {
            centre_m: flight.position_m,
            velocity_m_s: flight.velocity_m_s,
            orientation_xyzw: self.orientation_xyzw,
            angular_velocity_inertial_rad_s: q(self.orientation_xyzw)
                .rotate_vec3(self.angular_velocity_body_rad_s),
        };
        for (foot, f) in h.feet.iter().zip(&self.feet) {
            let lever = q(self.orientation_xyzw).rotate_vec3(foot.offset_body_m);
            let tol = (128. * f64::EPSILON * flight.position_m.length()).max(1e-6);
            if f.model != MODEL
                || f.surface.frame.target != flight.definition.parent
                || (f.surface.frame.time_s - flight.last_time_s).abs() > 1e-6
                || !finite(f.position_m)
                || !finite(f.velocity_m_s)
                || !finite(f.surface.position_m)
                || !finite(f.surface.velocity_m_s)
                || (f.position_m - (pose.centre_m + lever)).length() > tol
                || (f.velocity_m_s
                    - (pose.velocity_m_s + pose.angular_velocity_inertial_rad_s.cross(lever)))
                .length()
                    > tol
                || (f.surface.outward_normal.length_squared() - 1.).abs() > 1e-10
                || !f.surface.radial_clearance_m.is_finite()
                || !f.surface.tangent_gap_m.is_finite()
                || !f.surface.slope_rad.is_finite()
                || f.surface.probe_distance_m != h.normal_probe_m
            {
                return Err("Invalid retained foot geometry, pose or clock".into());
            }
            let response = foot.response.response(
                f.surface.tangent_gap_m,
                f.surface.outward_normal,
                f.velocity_m_s - f.surface.velocity_m_s,
            )?;
            // Compare canonical serde values: original f64 values round-trip through the save reader.
            if serde_json::to_value(&response).map_err(|e| e.to_string())?
                != serde_json::to_value(&f.response).map_err(|e| e.to_string())?
            {
                return Err("Retained foot force does not match authored response and actual relative motion".into());
            }
            let expected_torque = lever.cross(response.force_n);
            let scale = 1.
                + expected_torque.length()
                + response.force_n.length() * pose.velocity_m_s.length();
            let centre_power = response.force_n.dot(pose.velocity_m_s);
            let torque_power = expected_torque.dot(pose.angular_velocity_inertial_rad_s);
            let ground_power = response.force_n.dot(f.surface.velocity_m_s);
            let elastic = foot.response.stiffness_n_m
                * response.compression_m
                * response.compression_rate_m_s;
            if !finite(f.lever_inertial_m)
                || !finite(f.torque_about_centre_n_m)
                || (f.lever_inertial_m - lever).length() > 1e-8
                || (f.torque_about_centre_n_m - expected_torque).length() > 1e-8 * scale
                || [
                    f.centre_force_power_w,
                    f.torque_power_w,
                    f.ground_force_power_w,
                    f.elastic_storage_rate_w,
                    f.power_residual_w,
                ]
                .iter()
                .any(|v| !v.is_finite())
                || (f.centre_force_power_w - centre_power).abs() > 1e-8 * scale
                || (f.torque_power_w - torque_power).abs() > 1e-8 * scale
                || (f.ground_force_power_w - ground_power).abs() > 1e-8 * scale
                || (f.elastic_storage_rate_w - elastic).abs() > 1e-8 * scale
                || (f.power_residual_w
                    - (centre_power + torque_power - ground_power
                        + elastic
                        + response.normal_dissipation_w
                        + response.sliding_dissipation_w))
                    .abs()
                    > 1e-8 * scale
            {
                return Err("Invalid retained off-centre force/torque work".into());
            }
        }
        let storage = self
            .feet
            .iter()
            .map(|f| f.response.stored_elastic_energy_j)
            .sum::<f64>();
        if (self.cumulative_work.elastic_storage_change_j - storage).abs()
            > 1e-8 * (1. + storage.abs())
        {
            return Err(
                "Retained actual elastic storage does not close across contact work history".into(),
            );
        }
        Ok(())
    }
}
fn sample(
    world: &Universe,
    target: BodyRef,
    time: f64,
    position: Vec3d,
    velocity: Vec3d,
    state: &RigidState,
    h: &Hardware,
) -> Result<Vec<FootSample>, String> {
    let pose = BodyPose {
        centre_m: position,
        velocity_m_s: velocity,
        orientation_xyzw: state.orientation_xyzw,
        angular_velocity_inertial_rad_s: q(state.orientation_xyzw)
            .rotate_vec3(state.angular_velocity_body_rad_s),
    };
    let hull =
        crate::vehicle_contact::Surface::sample(world, target, time, position, h.normal_probe_m)?;
    if hull.tangent_gap_m <= h.hull_radius_m {
        return Err("Authored spherical hull touches modeled terrain; gear-only response does not support hull collision".into());
    }
    let resolution = 16.
        * f64::EPSILON
        * position
            .x
            .abs()
            .max(position.y.abs())
            .max(position.z.abs())
            .max(1.);
    let allowed = h
        .feet
        .iter()
        .map(|f| f.response.maximum_stroke_m)
        .fold(h.hull_radius_m.min(h.normal_probe_m), f64::min)
        * 0.01;
    if resolution > allowed {
        return Err("System-local coordinate precision does not resolve authored gear stroke/hull/probe geometry".into());
    }
    let radius = hull.frame.radius_m;
    for foot in &h.feet {
        if foot.response.maximum_stroke_m > radius * 0.001 {
            return Err(
                "Foot stroke exceeds supported small-stroke terrain curvature regime".into(),
            );
        }
    }
    h.feet
        .iter()
        .map(|f| f.sample(world, target, time, &pose, h.normal_probe_m))
        .collect()
}
fn force(samples: &[FootSample]) -> Vec3d {
    samples
        .iter()
        .fold(Vec3d::ZERO, |a, b| a + b.response.force_n)
}
fn torque(samples: &[FootSample]) -> Vec3d {
    samples
        .iter()
        .fold(Vec3d::ZERO, |a, b| a + b.torque_about_centre_n_m)
}
fn angular_acceleration(state: &RigidState, h: &Hardware, mass: f64, torque: Vec3d) -> Vec3d {
    let inertia = h.inertia_per_mass() * mass;
    let omega = state.angular_velocity_body_rad_s;
    let body_torque = inverse(q(state.orientation_xyzw)).rotate_vec3(torque);
    let numerator = body_torque - omega.cross(diagonal(omega, inertia));
    Vec3d::new(
        numerator.x / inertia.x,
        numerator.y / inertia.y,
        numerator.z / inertia.z,
    )
}
fn momentum(state: &RigidState, h: &Hardware, mass: f64) -> Vec3d {
    q(state.orientation_xyzw).rotate_vec3(diagonal(
        state.angular_velocity_body_rad_s,
        h.inertia_per_mass() * mass,
    ))
}
fn gravity(
    world: &Universe,
    target: BodyRef,
    time: f64,
    position: Vec3d,
) -> Result<(Vec3d, Vec<GravitySample>), String> {
    let sources = world_vehicle::gravity_sources(&world.systems[target.system as usize], time)?;
    let mut a = Vec3d::ZERO;
    let mut samples = vec![];
    for s in sources {
        let r = s.position_m - position;
        let distance = r.length();
        if distance <= 0. || !distance.is_finite() {
            return Err("Contact gravity is singular or nonfinite".into());
        }
        if s.source != Source::Body(target.body) && distance <= s.radius_m {
            return Err("Terrain craft intersects another modeled spherical source".into());
        }
        a += r * (s.mu / (distance * distance * distance));
        samples.push(GravitySample {
            source: s.source,
            position_m: s.position_m,
            mu_m3_s2: s.mu,
            radius_m: s.radius_m,
        });
    }
    if !finite(a) {
        return Err("Contact gravity left finite support".into());
    }
    Ok((a, samples))
}
/// Midpoint coupled translation/Euler rotation, with exact finite-mass central thrust.
fn substep(
    flight: &Flight,
    world: &Universe,
    h: &Hardware,
    dt: f64,
    command: Command,
) -> Result<Flight, String> {
    let state = flight.contact.as_ref().ok_or("Missing rigid state")?;
    let definition = &flight.definition.plan.definition;
    let target = flight.definition.parent;
    let time = flight.last_time_s;
    let initial = sample(
        world,
        target,
        time,
        flight.position_m,
        flight.velocity_m_s,
        state,
        h,
    )?;
    let mass = flight.propulsion.mass_kg(definition);
    let acceleration = gravity(world, target, time, flight.position_m)?.0 + force(&initial) / mass;
    let half_command = Command {
        duration_s: dt * 0.5,
        ..command.clone()
    };
    let (mid_position, half_velocity, mid_propulsion, _) = world_vehicle::drift(
        definition,
        &flight.propulsion,
        flight.position_m,
        flight.velocity_m_s,
        &half_command,
        acceleration,
    )?;
    let mid_velocity = half_velocity + acceleration * (dt * 0.25);
    let mut mid = state.clone();
    let initial_alpha = angular_acceleration(state, h, mass, torque(&initial));
    mid.orientation_xyzw = array(rotate(
        q(state.orientation_xyzw),
        state.angular_velocity_body_rad_s,
        dt * 0.5,
    )?);
    mid.angular_velocity_body_rad_s =
        state.angular_velocity_body_rad_s + initial_alpha * (dt * 0.5);
    let mid_samples = sample(
        world,
        target,
        time + dt * 0.5,
        mid_position,
        mid_velocity,
        &mid,
        h,
    )?;
    let mid_mass = mid_propulsion.mass_kg(definition);
    let mid_gravity = gravity(world, target, time + dt * 0.5, mid_position)?.0;
    let mid_acceleration = mid_gravity + force(&mid_samples) / mid_mass;
    let full_command = Command {
        duration_s: dt,
        ..command
    };
    let (position, half_velocity, propulsion, interval) = world_vehicle::drift(
        definition,
        &flight.propulsion,
        flight.position_m,
        flight.velocity_m_s,
        &full_command,
        mid_acceleration,
    )?;
    let velocity = half_velocity + mid_acceleration * (dt * 0.5);
    let mut next = flight.clone();
    next.position_m = position;
    next.velocity_m_s = velocity;
    next.propulsion = propulsion;
    next.last_interval = Some(interval.clone());
    next.last_time_s = next.launched_at_s + next.propulsion.elapsed_s;
    let mut rigid = state.clone();
    rigid.orientation_xyzw = array(rotate(
        q(state.orientation_xyzw),
        mid.angular_velocity_body_rad_s,
        dt,
    )?);
    rigid.angular_velocity_body_rad_s = state.angular_velocity_body_rad_s
        + angular_acceleration(&mid, h, mid_mass, torque(&mid_samples)) * dt;
    rigid.feet = sample(
        world,
        target,
        next.last_time_s,
        position,
        velocity,
        &rigid,
        h,
    )?;
    let final_sources = gravity(world, target, next.last_time_s, position)?.1;
    // Other source contact is conservatively refused before accepting a whole step.
    // A swept hull bound includes midpoint deviation and acceleration curvature.
    for (a, b) in flight.last_sources.iter().zip(&final_sources) {
        if a.source == Source::Body(target.body) {
            continue;
        }
        let start = flight.position_m - a.position_m;
        let end = position - b.position_m;
        let chord = end - start;
        let f = if chord.length_squared() > 0. {
            (-start.dot(chord) / chord.length_squared()).clamp(0., 1.)
        } else {
            0.
        };
        let deviation = (mid_position - (flight.position_m + position) * 0.5).length()
            + mid_acceleration.length() * dt * dt / 8.;
        let extent = h
            .feet
            .iter()
            .map(|f| f.offset_body_m.length())
            .fold(h.hull_radius_m, f64::max);
        if (start + chord * f).length() <= a.radius_m.max(b.radius_m) + extent + deviation {
            return Err("Swept rigid hull may intersect another spherical source; that collision response is unsupported".into());
        }
    }
    let mut work = Work {
        duration_s: dt,
        substeps: 1,
        ..Default::default()
    };
    for ((a, m), b) in initial.iter().zip(&mid_samples).zip(&rigid.feet) {
        work.contact_force_work_j += m.response.force_n.dot(position - flight.position_m);
        work.contact_torque_work_j += m
            .torque_about_centre_n_m
            .dot(q(mid.orientation_xyzw).rotate_vec3(mid.angular_velocity_body_rad_s))
            * dt;
        work.moving_ground_work_j += m.ground_force_power_w * dt;
        work.elastic_storage_change_j +=
            b.response.stored_elastic_energy_j - a.response.stored_elastic_energy_j;
        work.normal_dissipation_j += m.response.normal_dissipation_w * dt;
        work.sliding_dissipation_j += m.response.sliding_dissipation_w * dt;
        work.maximum_compression_m = work.maximum_compression_m.max(
            a.response
                .compression_m
                .max(m.response.compression_m)
                .max(b.response.compression_m),
        );
        work.maximum_resultant_load_n = work.maximum_resultant_load_n.max(
            a.response
                .force_n
                .length()
                .max(m.response.force_n.length())
                .max(b.response.force_n.length()),
        );
    }
    work.contact_balance_residual_j = work.contact_force_work_j + work.contact_torque_work_j
        - work.moving_ground_work_j
        + work.elastic_storage_change_j
        + work.normal_dissipation_j
        + work.sliding_dissipation_j;
    let final_mass = next.propulsion.mass_kg(definition);
    work.translational_kinetic_change_j = 0.5
        * (final_mass * velocity.length_squared() - mass * flight.velocity_m_s.length_squared());
    work.gravity_work_j = mid_gravity.dot(position - flight.position_m) * mid_mass;
    work.central_thrust_work_j = Vec3d::new(
        full_command.direction[0],
        full_command.direction[1],
        full_command.direction[2],
    )
    .dot(mid_velocity)
        * interval.commanded_thrust_n
        * interval.powered_duration_s;
    work.exhausted_translational_kinetic_j =
        0.5 * interval.actual_propellant_kg * mid_velocity.length_squared();
    work.translational_balance_residual_j = work.translational_kinetic_change_j
        - work.gravity_work_j
        - work.central_thrust_work_j
        - work.contact_force_work_j
        + work.exhausted_translational_kinetic_j;
    let rotational_energy = |s: &RigidState, m: f64| {
        0.5 * s.angular_velocity_body_rad_s.dot(diagonal(
            s.angular_velocity_body_rad_s,
            h.inertia_per_mass() * m,
        ))
    };
    work.rotational_kinetic_change_j =
        rotational_energy(&rigid, final_mass) - rotational_energy(state, mass);
    work.exhausted_rotational_kinetic_j = rotational_energy(&mid, interval.actual_propellant_kg);
    work.rotational_balance_residual_j = work.rotational_kinetic_change_j
        - work.contact_torque_work_j
        + work.exhausted_rotational_kinetic_j;
    work.angular_impulse_n_m_s = torque(&mid_samples) * dt;
    // Uniformly scaling inertia and symmetric propellant removal carry the removed
    // material's co-rotating angular momentum. Euler's equation then has no spurious
    // mass-loss spin-up term. Central commanded thrust has no authored lever torque.
    work.exhausted_angular_momentum_kg_m2_s = momentum(&mid, h, interval.actual_propellant_kg);
    work.angular_momentum_residual_kg_m2_s =
        momentum(&rigid, h, next.propulsion.mass_kg(definition))
            - momentum(state, h, mass)
            - work.angular_impulse_n_m_s
            + work.exhausted_angular_momentum_kg_m2_s;
    work.validate()?;
    rigid.last_work.add(&work);
    rigid.cumulative_work.add(&work);
    next.contact = Some(rigid);
    next.last_sources = final_sources;
    if !finite(position) || position.length() > 1e15 || !finite(velocity) || velocity.length() > 1e6
    {
        return Err("Rigid contact motion exceeds finite world-flight bounds".into());
    }
    Ok(next)
}
/// Preview an entire ordinary one-second scheduler interval, committing atomically.
pub(crate) fn advance(flight: &mut Flight, world: &Universe, to: f64) -> Result<(), String> {
    if flight.phase != Phase::Flying {
        return Ok(());
    }
    let dt = to - flight.last_time_s;
    if !to.is_finite() || !(0.0..=1.000001).contains(&dt) {
        return Err(
            "Contact flight requires a finite scheduled interval of at most one second".into(),
        );
    }
    let h = flight
        .definition
        .contact
        .as_ref()
        .ok_or("Missing contact hardware")?
        .clone();
    h.validate()?;
    let current = gravity(
        world,
        flight.definition.parent,
        flight.last_time_s,
        flight.position_m,
    )?
    .1;
    if current.len() != flight.last_sources.len()
        || current.iter().zip(&flight.last_sources).any(|(a, b)| {
            a.source != b.source || a.mu_m3_s2 != b.mu_m3_s2 || a.radius_m != b.radius_m
        })
    {
        return Err("Contact gravity source inventory changed since the resolved state".into());
    }
    let mut next = flight.clone();
    next.contact.as_mut().unwrap().last_work = Work::default();
    let mut count = 0;
    let target_elapsed = to - next.launched_at_s;
    while next.propulsion.elapsed_s < target_elapsed {
        let remaining = target_elapsed - next.propulsion.elapsed_s;
        // Only an elapsed-time floating-point remainder may be rounded away,
        // never a represented world task or a positive physical integration span.
        if remaining <= 32. * f64::EPSILON * target_elapsed.abs().max(1.) {
            break;
        }
        if count >= MAX_STEPS_PER_SECOND {
            return Err("Contact interval exceeded 10000 bounded rigid substeps; no unresolved time is skipped".into());
        }
        let state = next.contact.as_ref().unwrap();
        let samples = sample(
            world,
            next.definition.parent,
            next.last_time_s,
            next.position_m,
            next.velocity_m_s,
            state,
            &h,
        )?;
        let mass = next.propulsion.mass_kg(&next.definition.plan.definition);
        let r = h.inertia_per_mass();
        let min_gyration = r.x.min(r.y).min(r.z);
        let mut stiffness = 0.;
        let mut damping = 0.;
        let mut step = h
            .maximum_step_s
            .min(target_elapsed - next.propulsion.elapsed_s);
        for (foot, sample) in h.feet.iter().zip(&samples) {
            let leverage = 1. + foot.offset_body_m.length_squared() / min_gyration;

            let speed = (sample.velocity_m_s - sample.surface.velocity_m_s).length() + 1.;
            let gap = sample.surface.tangent_gap_m.max(0.);
            // Resolving entry is tighter than resolving distant flight. This also
            // bounds foot travel across a finite normal probe near contact.
            if gap < foot.response.maximum_stroke_m + 2. * h.maximum_step_s * speed {
                stiffness += foot.response.stiffness_n_m * leverage;
                // The smooth Coulomb force has a low-speed tangential derivative
                // bounded by mu*normal_load/slip_scale. Normal damping alone cannot
                // bound the stiffness of this friction regularization. Include an
                // approaching-foot spring/damper load estimate so first contact
                // does not begin with an unbounded tangential response.
                let closing = (-(sample.velocity_m_s - sample.surface.velocity_m_s)
                    .dot(sample.surface.outward_normal))
                .max(0.);
                let anticipated_load = (foot.response.stiffness_n_m
                    * (sample.response.compression_m + closing * step)
                    + foot.response.damping_n_s_m * closing)
                    .min(foot.response.maximum_load_n)
                    .max(sample.response.normal_load_n);
                let friction_damping = foot.response.sliding_friction * anticipated_load
                    / foot.response.slip_speed_scale_m_s;
                damping += (foot.response.damping_n_s_m + friction_damping) * leverage;
                step = step
                    .min(0.1 * foot.response.maximum_stroke_m / speed)
                    .min(0.25 * h.normal_probe_m / speed);
            } else {
                step = step.min(0.25 * gap / speed);
            }
        }
        if stiffness > 0. {
            step = step.min(0.1 * (mass / stiffness).sqrt());
        }
        if damping > 0. {
            step = step.min(0.1 * mass / damping);
        }
        let omega = state.angular_velocity_body_rad_s.length();
        if omega > 0. {
            step = step.min(0.02 / omega);
        }
        // Dyadic step sizes make ordinary one-second boundaries exactly representable;
        // command/cutoff/final fragments retain their separately represented durations.
        let mut dyadic = 0.015625;
        while dyadic > step {
            dyadic *= 0.5;
            if dyadic == 0. {
                return Err("Contact step underflow".into());
            }
        }
        step = dyadic;
        let (command, left) =
            world_vehicle::active_command(&next.definition.plan, next.propulsion.elapsed_s);
        step = step.min(left);
        let rate = next.definition.plan.definition.rated_thrust_n * command.throttle
            / next.definition.plan.definition.exhaust_velocity_m_s();
        if rate > 0. && next.propulsion.propellant_kg > 0. {
            step = step.min(world_vehicle::exhaustion_time(
                next.propulsion.propellant_kg,
                rate,
            ));
        }
        if !step.is_finite()
            || step <= 0.
            || next.last_time_s + step == next.last_time_s
            || next.propulsion.elapsed_s + step == next.propulsion.elapsed_s
        {
            return Err("Contact substep is below represented flight-clock precision".into());
        }
        next = substep(&next, world, &h, step, command)?;
        count += 1;
    }
    next.last_time_s = to;
    next.contact.as_ref().unwrap().validate(&h, &next)?;
    *flight = next;
    Ok(())
}

pub fn context() -> serde_json::Value {
    serde_json::json!({"model":world_vehicle::CONTACT_MODEL,"contact_model":MODEL,
        "operator":"Ordinary one-second task; bounded explicit midpoint translation/Euler rigid-body rotation, deterministic Hamilton quaternion increments, exact finite-mass central inertial thrust",
        "terrain":"Existing body-fixed tilt/spin height field, finite-difference tangent normal and first-order penetration; modeled terrain and derivative-aware moving surface, not calibrated local hazards",
        "inertia":"Authored diagonal inertia = actual dry+cargo+remaining fuel mass times fixed gyration radii squared; symmetric co-rotating fuel removal carries corresponding angular momentum; thrust passes through the centre in inertial command axes",
        "response":"Per-foot finite unilateral spring/damper and smooth bounded sliding friction, actual off-centre forces/torques; strict stroke/resultant load and spherical hull limits",
        "evidence":"Actual final feet, cumulative and last macro contact work, changing stored elastic energy, normal/sliding dissipation, moving-ground work and angular-momentum integration residuals; numerical residuals retained without zeroing",
        "mass_removal_energy_scope":"Exhausted kinetic terms are the pre-ejection co-moving/co-rotating removed-mass share used in the body mechanical balance; not exhaust-jet kinetic energy, fuel chemical energy or a complete engine energy budget",
        "limits":["No static friction, persistent attachment, safe landing certification or cargo transfer","No hull/other-body collision response, atmosphere/entry/heating or thrust attitude controller","Explicit authored mechanics, not empirical hardware calibration or uncertainty","Finite-difference overview terrain is not measured meter-scale roughness or registered areoid shape","Small-stroke and bounded stability/travel steps; inability to resolve fails atomically at the prior supported macro state"],
        "max_substeps_per_second":MAX_STEPS_PER_SECOND,"empirical_validation":null,"uncertainty":null})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        sandbox::{BodyProperty, Edit},
        supply_vehicle::Plan,
        vehicle_arrival::{KinematicState, SurfaceFrame},
        vehicle_contact::Surface,
        world_vehicle::Definition,
    };
    fn setup(altitude: f64, descent: f64) -> (Universe, Definition) {
        let world = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let frame = SurfaceFrame::at(&world, target, world.time).unwrap();
        let direction = Vec3d::new(0.8, 0.6, 0.); // explicit nonpolar source-supported location
        let above = frame
            .to_inertial(&KinematicState {
                position_m: direction * (frame.radius_m + 10000.),
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let surface = Surface::sample(&world, target, world.time, above.position_m, 1.).unwrap();
        let position = surface.position_m + surface.outward_normal * altitude;
        let fixed = frame.rotate_to_fixed(position - frame.centre_m);
        let motion = frame
            .to_inertial(&KinematicState {
                position_m: fixed,
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let velocity = motion.velocity_m_s - surface.outward_normal * descent;
        let mut plan = Plan::example();
        plan.commands = vec![Command {
            duration_s: 10.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        let body = world.systems[0].body_state(target.body as usize, world.time);
        let d = Definition {
            model: world_vehicle::CONTACT_MODEL.into(),
            frame: world_vehicle::FRAME.into(),
            parent: target,
            plan,
            initial_relative_position_m: position - body.pos,
            initial_relative_velocity_m_s: velocity - body.vel,
            contact: Some(Hardware::example(&world, target, position).unwrap()),
        };
        d.validate(&world).unwrap();
        (world, d)
    }
    fn launch(world: &mut Universe, d: Definition) {
        world
            .apply_edit(Edit::CreateVehicle {
                definition: Box::new(d),
            })
            .unwrap();
    }
    #[test]
    fn unresolved_low_speed_friction_is_refused_before_granting_motion() {
        let (mut world, mut definition) = setup(1.6, 0.1);
        for foot in &mut definition.contact.as_mut().unwrap().feet {
            foot.response.sliding_friction = 10.;
            foot.response.slip_speed_scale_m_s = 1e-6;
        }
        launch(&mut world, definition);
        let prior = world.vehicles[0].clone();
        world.advance_to(world.time + 1., None, None);
        let stopped = &world.vehicles[0];
        assert!(
            matches!(&stopped.phase,Phase::Unsupported {reason} if reason.contains("precision") || reason.contains("substeps")),
            "{:?}",
            stopped.phase
        );
        assert_eq!(stopped.position_m, prior.position_m);
        assert_eq!(stopped.contact, prior.contact);
        assert_eq!(stopped.propulsion, prior.propulsion);
    }
    #[test]
    fn explicit_contact_geography_supports_actual_terrain_below_the_old_sphere() {
        let (mut world, _) = setup(10000., 0.);
        let target = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut start = crate::vehicle_arrival::GeographicStart::contact_example();
        start.up_velocity_m_s = -0.1;
        let mut found = false;
        for longitude in [130., 120., 140., 180., -30.] {
            start.longitude_east_deg = longitude;
            start.spherical_altitude_m = 10000.;
            let absolute = start.absolute(&world, target).unwrap();
            let surface =
                Surface::sample(&world, target, world.time, absolute.position_m, 1.).unwrap();
            let height = 10000. - surface.radial_clearance_m;
            if height < -10. {
                start.spherical_altitude_m = height + 3.;
                found = true;
                break;
            }
        }
        assert!(
            found,
            "Source-supported MOLA fixture requires a below-base terrain location"
        );
        let absolute = start.absolute(&world, target).unwrap();
        let hardware = Hardware::example(&world, target, absolute.position_m).unwrap();
        let mut plan = Plan::example();
        for command in &mut plan.commands {
            command.throttle = 0.;
        }
        assert!(start.flight(&world, target, plan.clone()).is_err());
        let mut legacy = start.clone();
        legacy.model = crate::vehicle_arrival::MODEL.into();
        assert!(legacy.validate().is_err());
        let definition = start
            .contact_flight(&world, target, plan, hardware)
            .unwrap();
        assert!(definition.initial_relative_position_m.length() < world.body(target).radius);
        launch(&mut world, definition);
        world.advance_to(world.time + 2., None, None);
        assert_eq!(
            world.vehicles[0].phase,
            Phase::Flying,
            "{:?}",
            world.vehicles[0].phase
        );
        assert!(
            world.vehicles[0]
                .contact
                .as_ref()
                .unwrap()
                .cumulative_work
                .normal_dissipation_j
                > 0.
        );
        world.validate_vehicle_state().unwrap();
    }
    #[test]
    fn terrain_response_changes_motion_attitude_and_retains_finite_work() {
        let (mut world, mut d) = setup(1.52, 0.1);
        // Unequal gear is an actual torque input, not a prescribed attitude animation.
        d.contact.as_mut().unwrap().feet[0].response.stiffness_n_m *= 1.8;
        let original = d.clone();
        launch(&mut world, d);
        let start = world.vehicles[0].clone();
        world.advance_to(world.time + 1., None, None);
        let flight = &world.vehicles[0];
        assert_eq!(flight.phase, Phase::Flying, "{:?}", flight.phase);
        let rigid = flight.contact.as_ref().unwrap();
        assert!(rigid.feet.iter().any(|f| f.response.compression_m > 0.));
        assert!(rigid.last_work.normal_dissipation_j > 0.);
        assert!(rigid.last_work.contact_force_work_j != 0.);
        assert!(rigid.last_work.contact_torque_work_j != 0.);
        assert!(
            (rigid.angular_velocity_body_rad_s
                - start.contact.as_ref().unwrap().angular_velocity_body_rad_s)
                .length()
                > 1e-4
        );
        assert_eq!(
            flight.definition.plan.definition.cargo,
            original.plan.definition.cargo
        );
        assert!(world.habitats.is_empty());
        assert_eq!(flight.last_time_s, start.last_time_s + 1.);
        assert!(rigid.last_work.substeps > 50);
        assert!(
            (rigid.cumulative_work.elastic_storage_change_j
                - rigid
                    .feet
                    .iter()
                    .map(|f| f.response.stored_elastic_energy_j)
                    .sum::<f64>())
            .abs()
                < 1e-6
        );
        world.validate_vehicle_state().unwrap();
    }
    #[test]
    fn rigid_motion_is_exact_across_world_chunks_save_and_full_record_replay() {
        let (mut world, d) = setup(1.6, 0.1);
        let origin = world.clone();
        let study = crate::world_vehicle_recording::Study::new(&origin, d.clone(), 5, 0.5).unwrap();
        launch(&mut world, d);
        let start = world.time;
        let mut chunked = world.clone();
        let mut resumed = world.clone();
        world.advance_to(start + 2., None, None);
        for i in 1..=8 {
            chunked.advance_to(start + i as f64 * 0.25, None, None);
        }
        resumed.advance_to(start + 1.25, None, None);
        resumed =
            crate::save::from_json(&crate::save::to_json(&resumed, "contact").unwrap()).unwrap();
        resumed.advance_to(start + 2., None, None);
        assert_eq!(
            serde_json::to_value(&world).unwrap(),
            serde_json::to_value(&chunked).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&world).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let bundle = crate::world_vehicle_recording::Bundle::run(&origin, study).unwrap();
        let path =
            std::env::temp_dir().join(format!("cosmogon-contact-record-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        bundle.export(&path).unwrap();
        let replay = crate::world_vehicle_recording::Bundle::read(&path).unwrap();
        assert_eq!(replay.manifest, bundle.manifest);
        assert_eq!(replay.manifest["model"], world_vehicle::CONTACT_MODEL);
        assert!(bundle.export(&path).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn central_finite_burn_carries_authored_exhaust_momentum_without_spin_up() {
        let (world, mut d) = setup(200000., 0.);
        let h = d.contact.as_mut().unwrap();
        h.initial_angular_velocity_body_rad_s = Vec3d::new(0., 0., 0.2);
        d.plan.commands = vec![Command {
            duration_s: 1.,
            throttle: 1.,
            direction: [1., 0., 0.],
        }];
        let mut flight = Flight::new(0, d, &world).unwrap();
        let before = flight.clone();
        advance(&mut flight, &world, world.time + 1.).unwrap();
        let state = flight.contact.as_ref().unwrap();
        assert_eq!(state.angular_velocity_body_rad_s, Vec3d::new(0., 0., 0.2));
        assert!(flight.propulsion.exhausted_propellant_kg > 0.);
        assert!(
            state
                .cumulative_work
                .exhausted_angular_momentum_kg_m2_s
                .length()
                > 0.
        );
        let hardware = flight.definition.contact.as_ref().unwrap();
        let removed = before
            .propulsion
            .mass_kg(&before.definition.plan.definition)
            - flight
                .propulsion
                .mass_kg(&flight.definition.plan.definition);
        assert!(
            (state.cumulative_work.exhausted_angular_momentum_kg_m2_s
                - momentum(state, hardware, removed))
            .length()
                < 1e-8
        );
        assert!(
            state
                .cumulative_work
                .angular_momentum_residual_kg_m2_s
                .length()
                < 1e-8
        );
        assert!((state.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.).abs() < 1e-12);
    }
    #[test]
    fn off_principal_axis_rotation_matches_euler_and_converges() {
        let (world, mut d) = setup(200000., 0.);
        let h = d.contact.as_mut().unwrap();
        h.gyration_radii_m = Vec3d::new(1., 1.1, 1.3);
        h.initial_angular_velocity_body_rad_s = Vec3d::new(0.2, 0.3, 0.4);
        h.maximum_step_s = 0.02;
        let mut coarse = Flight::new(0, d.clone(), &world).unwrap();
        let initial = coarse.clone();
        d.contact.as_mut().unwrap().maximum_step_s = 0.005;
        let mut fine = Flight::new(0, d, &world).unwrap();
        advance(&mut coarse, &world, world.time + 1.).unwrap();
        advance(&mut fine, &world, world.time + 1.).unwrap();
        let c = coarse.contact.as_ref().unwrap();
        let f = fine.contact.as_ref().unwrap();
        assert!(
            (f.angular_velocity_body_rad_s
                - initial
                    .contact
                    .as_ref()
                    .unwrap()
                    .angular_velocity_body_rad_s)
                .length()
                > 0.01
        );
        assert!(
            f.cumulative_work.angular_momentum_residual_kg_m2_s.length()
                < c.cumulative_work.angular_momentum_residual_kg_m2_s.length() * 0.2
        );
        let energy = |v: &Flight| {
            let s = v.contact.as_ref().unwrap();
            let w = s.angular_velocity_body_rad_s;
            0.5 * w.dot(diagonal(
                w,
                v.definition.contact.as_ref().unwrap().inertia_per_mass()
                    * v.propulsion.mass_kg(&v.definition.plan.definition),
            ))
        };
        assert!(
            (energy(&fine) - energy(&initial)).abs()
                < (energy(&coarse) - energy(&initial)).abs() * 0.2
        );
    }
    #[test]
    fn structural_failure_and_frame_edits_retain_prior_supported_state() {
        let (mut world, mut d) = setup(1.6, 3.);
        for foot in &mut d.contact.as_mut().unwrap().feet {
            foot.response.maximum_load_n = 1.;
        }
        launch(&mut world, d);
        let prior = world.vehicles[0].clone();
        world.advance_to(world.time + 1., None, None);
        let stopped = &world.vehicles[0];
        assert!(
            matches!(&stopped.phase,Phase::Unsupported {reason} if reason.contains("load")),
            "{:?}",
            stopped.phase
        );
        assert_eq!(stopped.position_m, prior.position_m);
        assert_eq!(stopped.propulsion, prior.propulsion);
        assert_eq!(stopped.contact, prior.contact);
        let (mut world, d) = setup(200000., 0.);
        let target = d.parent;
        launch(&mut world, d);
        world
            .apply_edit(Edit::SetProperty {
                body: target,
                property: BodyProperty::RotationPeriod(10000.),
            })
            .unwrap();
        world.advance_to(world.time + 1., None, None);
        assert!(matches!(world.vehicles[0].phase, Phase::Unsupported { .. }));
        assert_eq!(world.vehicles[0].last_time_s, world.time - 1.);
        crate::save::from_json(&crate::save::to_json(&world, "edited contact").unwrap()).unwrap();
    }
    #[test]
    fn contact_hardware_save_pins_and_initial_geometry_are_strict() {
        let (mut world, mut d) = setup(1.6, 0.);
        let mut invalid = d.clone();
        invalid.contact.as_mut().unwrap().initial_orientation_xyzw = [0.; 4];
        assert!(Flight::new(0, invalid, &world).is_err());
        let mut invalid = d.clone();
        invalid.contact.as_mut().unwrap().gyration_radii_m = Vec3d::new(10., 1., 1.);
        assert!(Flight::new(0, invalid, &world).is_err());
        let mut invalid = d.clone();
        invalid.contact.as_mut().unwrap().gyration_radii_m = Vec3d::new(10., 10., 10.);
        assert!(Flight::new(0, invalid, &world).is_err());
        let mut invalid = d.clone();
        invalid.model = world_vehicle::MODEL.into();
        assert!(invalid.validate(&world).is_err());
        d.initial_relative_position_m =
            d.initial_relative_position_m.normalize() * world.body(d.parent).radius;
        assert!(Flight::new(0, d, &world).is_err());
        let (_, d) = setup(1.6, 0.);
        launch(&mut world, d);
        world.advance_to(world.time + 1., None, None);
        let original: serde_json::Value =
            serde_json::from_str(&crate::save::to_json(&world, "contact").unwrap()).unwrap();
        for pointer in [
            "/header/data_versions/cosmogon-finite-terrain-contact",
            "/header/data_versions/cosmogon-world-vehicle-contact-capability",
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = serde_json::json!("unsupported");
            assert!(crate::save::from_json(&changed.to_string()).is_err());
        }
        // Version-1 spherical event evidence cannot acquire terrain semantics by
        // attaching the new state and changing a format/model label.
        let flight = &world.vehicles[0];
        let centre = flight
            .last_sources
            .iter()
            .find(|sample| sample.source == Source::Body(flight.definition.parent.body))
            .unwrap()
            .position_m;
        let mut changed = original.clone();
        changed["universe"]["vehicles"][0]["phase"] = serde_json::json!({"kind":"impact","source":{"kind":"body","body_index":flight.definition.parent.body},"source_position_m":centre,"radius_m":(flight.position_m-centre).length(),"position_residual_m":0.,"tolerance_m":0.001,"source_sample_duration_s":0.});
        // Give this forged terminal state the valid parked scheduler it would
        // otherwise require, so the test reaches the format-semantic gate.
        changed["universe"]["scheduler"]["tasks"][world_vehicle::TASK]["period"] =
            serde_json::json!(crate::scheduler::NEVER);
        changed["universe"]["scheduler"]["tasks"][world_vehicle::TASK]["origin"] =
            serde_json::json!(world.time);
        changed["universe"]["scheduler"]["tasks"][world_vehicle::TASK]["steps"] =
            serde_json::json!(0);
        let error = crate::save::from_json(&changed.to_string())
            .unwrap_err()
            .to_string();
        assert!(error.contains("reinterpret"), "{error}");
        let mut changed = original.clone();
        changed["universe"]["vehicles"][0]["contact"]["orientation_xyzw"] =
            serde_json::json!([0., 0., 0., 0.]);
        assert!(crate::save::from_json(&changed.to_string()).is_err());
        let mut changed = original.clone();
        changed["universe"]["vehicles"][0]["contact"]["feet"][0]["response"]["normal_load_n"] =
            serde_json::json!(99999.);
        assert!(crate::save::from_json(&changed.to_string()).is_err());
    }
}
