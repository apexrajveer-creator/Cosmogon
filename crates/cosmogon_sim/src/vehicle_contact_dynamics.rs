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
pub const SHEAR_MODEL: &str = "cosmogon-finite-terrain-contact/2";
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shear: Option<crate::vehicle_shear::Definition>,
    /// Opt-in solver-integrated supported surface-operation window accounting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<crate::vehicle_operation::Definition>,
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
    pub fn with_shear(mut self, shear: crate::vehicle_shear::Definition) -> Result<Self, String> {
        self.model = SHEAR_MODEL.into();
        self.shear = Some(shear);
        for foot in &mut self.feet {
            foot.response.sliding_friction = 0.;
        }
        self.validate()?;
        Ok(self)
    }
    /// Attach the opt-in solver-integrated operation-window envelope with the shared
    /// instantaneous support limits and conservative utilization caps.
    pub fn with_operation(mut self, required_window_s: f64) -> Result<Self, String> {
        self.operation = Some(crate::vehicle_operation::Definition {
            model: crate::vehicle_operation::MODEL.into(),
            support: crate::vehicle_support::Definition::default(),
            required_window_s,
            maximum_sample_gap_s: 1.000001,
            maximum_structural_utilization: 1.,
            maximum_stroke_utilization: 1.,
            maximum_friction_utilization: 1.,
        });
        self.validate()?;
        Ok(self)
    }
    pub fn flight_model(&self) -> &'static str {
        if self.shear.is_some() {
            world_vehicle::SHEAR_MODEL
        } else {
            world_vehicle::CONTACT_MODEL
        }
    }
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
            shear: None,
            operation: None,
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
        if !matches!(
            (self.model.as_str(), &self.shear),
            (MODEL, None) | (SHEAR_MODEL, Some(_))
        ) || !finite(g)
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
            if self.shear.is_some() && foot.response.sliding_friction != 0. {
                return Err("Persistent shear requires zero legacy smooth-sliding coefficient; do not apply both friction laws".into());
            }
            if !finite(foot.offset_body_m) || foot.offset_body_m.length() > 100. {
                return Err("Foot offset exceeds the 100 m contact hardware bound".into());
            }
        }
        if let Some(shear) = &self.shear {
            shear.validate()?;
        }
        if let Some(operation) = &self.operation {
            operation.validate()?;
        }
        Ok(())
    }
    fn inertia_per_mass(&self) -> Vec3d {
        diagonal(self.gyration_radii_m, self.gyration_radii_m)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShearWork {
    pub endpoint_relative_work_j: f64,
    pub elastic_storage_change_j: f64,
    pub plastic_dissipation_j: f64,
    pub unloading_release_j: f64,
    pub numerical_dissipation_j: f64,
    pub balance_residual_j: f64,
    pub plastic_slip_distance_m: f64,
}
impl ShearWork {
    fn add(&mut self, b: &Self) {
        self.endpoint_relative_work_j += b.endpoint_relative_work_j;
        self.elastic_storage_change_j += b.elastic_storage_change_j;
        self.plastic_dissipation_j += b.plastic_dissipation_j;
        self.unloading_release_j += b.unloading_release_j;
        self.numerical_dissipation_j += b.numerical_dissipation_j;
        self.balance_residual_j += b.balance_residual_j;
        self.plastic_slip_distance_m += b.plastic_slip_distance_m;
    }
    fn balance_terms_j(&self) -> f64 {
        self.elastic_storage_change_j
            + self.plastic_dissipation_j
            + self.unloading_release_j
            + self.numerical_dissipation_j
    }
    fn validate(&self) -> Result<(), String> {
        if [
            self.endpoint_relative_work_j,
            self.elastic_storage_change_j,
            self.plastic_dissipation_j,
            self.unloading_release_j,
            self.numerical_dissipation_j,
            self.balance_residual_j,
            self.plastic_slip_distance_m,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || [
                self.plastic_dissipation_j,
                self.unloading_release_j,
                self.numerical_dissipation_j,
                self.plastic_slip_distance_m,
            ]
            .iter()
            .any(|v| *v < 0.)
            || (self.endpoint_relative_work_j + self.balance_terms_j() - self.balance_residual_j)
                .abs()
                > 1e-8 * (1. + self.endpoint_relative_work_j.abs() + self.balance_terms_j().abs())
        {
            return Err("Invalid retained endpoint shear work evidence".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shear: Option<ShearWork>,
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
    fn for_hardware(h: &Hardware) -> Self {
        Self {
            shear: h.shear.as_ref().map(|_| ShearWork::default()),
            ..Default::default()
        }
    }
    fn add(&mut self, b: &Self) {
        if let Some(other) = &b.shear {
            self.shear.get_or_insert_with(ShearWork::default).add(other);
        }
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
        if let Some(shear) = &self.shear {
            shear.validate()?;
        }
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
                + self.shear.as_ref().map_or(0., ShearWork::balance_terms_j)
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
                    + self
                        .shear
                        .as_ref()
                        .map_or(0., |s| 1e-8 * s.balance_terms_j().abs())
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
            model: h.model.clone(),
            orientation_xyzw: h.initial_orientation_xyzw,
            angular_velocity_body_rad_s: h.initial_angular_velocity_body_rad_s,
            feet: vec![],
            last_work: Work::for_hardware(h),
            cumulative_work: Work::for_hardware(h),
        };
        result.feet = sample(world, target, world.time, position, velocity, &result, h)?;
        if result.feet.iter().any(|f| f.surface.tangent_gap_m < 0.) {
            return Err("Initial contact feet must be outside modeled terrain; do not invent preloaded elastic energy".into());
        }
        Ok(result)
    }
    pub(crate) fn validate(&self, h: &Hardware, flight: &Flight) -> Result<(), String> {
        h.validate()?;
        if self.model != h.model
            || self.orientation_xyzw.iter().any(|v| !v.is_finite())
            || (self.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 1e-10
            || !finite(self.angular_velocity_body_rad_s)
            || self.angular_velocity_body_rad_s.length() > 1000.
            || self.feet.len() != h.feet.len()
        {
            return Err("Invalid retained rigid contact state".into());
        }
        if self.last_work.shear.is_some() != h.shear.is_some()
            || self.cumulative_work.shear.is_some() != h.shear.is_some()
        {
            return Err("Contact work and shear hardware capability disagree".into());
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
            if f.model != h.model
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
            let mut response = foot.response.response(
                f.surface.tangent_gap_m,
                f.surface.outward_normal,
                f.velocity_m_s - f.surface.velocity_m_s,
            )?;
            match (&h.shear, &f.shear) {
                (None, None) => {}
                (Some(d), Some(s)) => {
                    s.validate(d)?;
                    if s.normal_load_n != response.normal_load_n
                        || (s.outward_normal - f.surface.outward_normal).length() > 1e-10
                    {
                        return Err(
                            "Retained shear normal/load do not match actual foot response".into(),
                        );
                    }
                    response.force_n += d.force_n(s)?;
                    if response.force_n.length() > foot.response.maximum_load_n {
                        return Err(
                            "Retained combined foot load exceeds authored structural limit".into(),
                        );
                    }
                }
                _ => return Err("Retained foot shear/hardware capability mismatch".into()),
            }
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
        if let Some(d) = &h.shear {
            let storage = self
                .feet
                .iter()
                .map(|f| d.energy_j(f.shear.as_ref().ok_or("Missing retained foot shear")?))
                .collect::<Result<Vec<_>, String>>()?
                .iter()
                .sum::<f64>();
            let change = self
                .cumulative_work
                .shear
                .as_ref()
                .unwrap()
                .elastic_storage_change_j;
            if (change - storage).abs() > 1e-8 * (1. + storage.abs()) {
                return Err(
                    "Retained tangential storage does not close across shear history".into(),
                );
            }
        }
        Ok(())
    }
}
fn sample_geometry(
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
fn attach_shear(
    foot: &mut FootSample,
    definition: &crate::vehicle_shear::Definition,
    state: crate::vehicle_shear::State,
    centre_velocity: Vec3d,
    omega: Vec3d,
    limit: f64,
) -> Result<(), String> {
    foot.model = SHEAR_MODEL.into();
    foot.response.force_n += definition.force_n(&state)?;
    if foot.response.force_n.length() > limit {
        return Err("Combined normal/shear foot force exceeds authored structural limit".into());
    }
    foot.torque_about_centre_n_m = foot.lever_inertial_m.cross(foot.response.force_n);
    foot.centre_force_power_w = foot.response.force_n.dot(centre_velocity);
    foot.torque_power_w = foot.torque_about_centre_n_m.dot(omega);
    foot.ground_force_power_w = foot.response.force_n.dot(foot.surface.velocity_m_s);
    // Existing elastic rate/storage fields retain normal spring semantics.
    // Shear has a discrete history/work ledger, not invented instantaneous loss.
    foot.power_residual_w = foot.centre_force_power_w + foot.torque_power_w
        - foot.ground_force_power_w
        + foot.elastic_storage_rate_w
        + foot.response.normal_dissipation_w;
    foot.shear = Some(state);
    Ok(())
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
    let mut samples = sample_geometry(world, target, time, position, velocity, state, h)?;
    if let Some(d) = &h.shear {
        let omega = q(state.orientation_xyzw).rotate_vec3(state.angular_velocity_body_rad_s);
        for (i, f) in samples.iter_mut().enumerate() {
            let unloaded;
            let from;
            let previous = if let Some(old) = state.feet.get(i) {
                if f.response.normal_load_n <= 0. {
                    from = &f.surface.frame;
                    unloaded = crate::vehicle_shear::State::unloaded(f.surface.outward_normal)?;
                    &unloaded
                } else {
                    from = &old.surface.frame;
                    old.shear
                        .as_ref()
                        .ok_or("Missing persistent foot history")?
                }
            } else {
                from = &f.surface.frame;
                unloaded = crate::vehicle_shear::State::unloaded(f.surface.outward_normal)?;
                &unloaded
            };
            let increment = d.advance_in_rotating_frames(
                previous,
                from,
                &f.surface.frame,
                f.surface.outward_normal,
                Vec3d::ZERO,
                f.response.normal_load_n,
            )?;
            attach_shear(
                f,
                d,
                increment.state,
                velocity,
                omega,
                h.feet[i].response.maximum_load_n,
            )?;
        }
    }
    Ok(samples)
}
fn evolve_shear(
    initial: &[FootSample],
    predicted: &mut [FootSample],
    middle_relative_velocity: &[Vec3d],
    dt: f64,
    state: &RigidState,
    centre_velocity: Vec3d,
    h: &Hardware,
) -> Result<Option<ShearWork>, String> {
    let Some(d) = &h.shear else { return Ok(None) };
    let omega = q(state.orientation_xyzw).rotate_vec3(state.angular_velocity_body_rad_s);
    let mut work = ShearWork::default();
    for (i, (a, b)) in initial.iter().zip(predicted.iter_mut()).enumerate() {
        let start = a.surface.tangent_gap_m;
        let end = b.surface.tangent_gap_m;
        // Linear within-step crossing assumption. Do not accumulate free-flight
        // displacement as spring preload before actual first contact.
        let fraction = if start >= 0. && end >= 0. {
            0.
        } else if start >= 0. {
            (-end / (start - end)).clamp(0., 1.)
        } else if end >= 0. {
            (-start / (end - start)).clamp(0., 1.)
        } else {
            1.
        };
        let v = middle_relative_velocity[i];
        let tangent = v - b.surface.outward_normal * v.dot(b.surface.outward_normal);
        let separate = start >= 0. && end >= 0.;
        let freed;
        let increment = if separate {
            freed = crate::vehicle_shear::State::unloaded(b.surface.outward_normal)?;
            d.advance_in_rotating_frames(
                &freed,
                &b.surface.frame,
                &b.surface.frame,
                b.surface.outward_normal,
                Vec3d::ZERO,
                b.response.normal_load_n,
            )?
        } else {
            d.advance_in_rotating_frames(
                a.shear.as_ref().ok_or("Missing initial foot shear")?,
                &a.surface.frame,
                &b.surface.frame,
                b.surface.outward_normal,
                tangent * (dt * fraction),
                b.response.normal_load_n,
            )?
        };
        work.add(&ShearWork {
            endpoint_relative_work_j: increment.relative_foot_work_j,
            elastic_storage_change_j: increment.elastic_storage_change_j,
            plastic_dissipation_j: increment.plastic_dissipation_j,
            unloading_release_j: increment.unloading_release_j,
            numerical_dissipation_j: increment.numerical_dissipation_j,
            balance_residual_j: increment.balance_residual_j,
            plastic_slip_distance_m: increment.plastic_slip_m.length(),
        });
        attach_shear(
            b,
            d,
            increment.state,
            centre_velocity,
            omega,
            h.feet[i].response.maximum_load_n,
        )?;
    }
    work.validate()?;
    Ok(Some(work))
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
    let held_definition;
    let definition: &crate::supply_vehicle::Definition = match &flight.cargo {
        Some(ledger) => {
            held_definition = ledger.current_definition(&flight.definition.plan.definition)?;
            &held_definition
        }
        None => &flight.definition.plan.definition,
    };
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
    let mut mid_samples = sample_geometry(
        world,
        target,
        time + dt * 0.5,
        mid_position,
        mid_velocity,
        &mid,
        h,
    )?;
    let half_relative = initial
        .iter()
        .zip(&mid_samples)
        .map(|(a, b)| {
            ((a.velocity_m_s - a.surface.velocity_m_s) + (b.velocity_m_s - b.surface.velocity_m_s))
                * 0.5
        })
        .collect::<Vec<_>>();
    evolve_shear(
        &initial,
        &mut mid_samples,
        &half_relative,
        dt * 0.5,
        &mid,
        mid_velocity,
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
    let mut final_feet = sample_geometry(
        world,
        target,
        next.last_time_s,
        position,
        velocity,
        &rigid,
        h,
    )?;
    let middle_relative = mid_samples
        .iter()
        .map(|f| f.velocity_m_s - f.surface.velocity_m_s)
        .collect::<Vec<_>>();
    let shear_work = evolve_shear(
        &initial,
        &mut final_feet,
        &middle_relative,
        dt,
        &rigid,
        velocity,
        h,
    )?;
    rigid.feet = final_feet;
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
        shear: shear_work,
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
        + work.sliding_dissipation_j
        + work.shear.as_ref().map_or(0., ShearWork::balance_terms_j);
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
    // Opt-in operation-window evidence is sampled from the accepted midpoint and
    // committed endpoint of this whole substep. It lives on the previewed flight, so
    // any later macro-step failure discards these samples with the rest of the step.
    if let Some(operation) = h.operation.as_ref() {
        if dt > 0. {
            let mut midpoint = flight.clone();
            midpoint.position_m = mid_position;
            midpoint.velocity_m_s = mid_velocity;
            midpoint.propulsion = mid_propulsion;
            midpoint.last_time_s = time + dt * 0.5;
            let mut midpoint_rigid = mid;
            midpoint_rigid.feet = mid_samples;
            midpoint.contact = Some(midpoint_rigid);
            crate::vehicle_operation::observe(world, &mut midpoint, operation)?;
            next.operation_ledger = midpoint.operation_ledger;
            crate::vehicle_operation::observe(world, &mut next, operation)?;
        }
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
    next.contact.as_mut().unwrap().last_work = Work::for_hardware(&h);
    let mut count = 0;
    let target_elapsed = to - next.launched_at_s;
    while next.propulsion.elapsed_s < target_elapsed {
        let remaining = target_elapsed - next.propulsion.elapsed_s;
        // Only an elapsed-time floating-point remainder may be rounded away,
        // never a represented world task or a positive physical integration span.
        // At real-world epochs the absolute craft clock's ulp (hundreds of
        // nanoseconds) exceeds the residual task-tail fragment; a remainder
        // that cannot move the representable clock is rounded away and re-pinned
        // to the world task boundary at macro completion.
        if remaining <= 32. * f64::EPSILON * target_elapsed.abs().max(1.)
            || next.launched_at_s + next.propulsion.elapsed_s + remaining
                == next.launched_at_s + next.propulsion.elapsed_s
        {
            break;
        }
        while let Some(index) = next.pending_authored_removal() {
            next.apply_authored_removal(index)?;
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
        let mass = next.mass_kg();
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
                stiffness += (foot.response.stiffness_n_m
                    + h.shear.as_ref().map_or(0., |s| s.stiffness_n_m))
                    * leverage;
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
                if let Some(s) = &h.shear {
                    step = step.min(0.1 * s.maximum_elastic_shear_m / speed);
                }
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
        let held_definition;
        let d: &crate::supply_vehicle::Definition = match &next.cargo {
            Some(ledger) => {
                held_definition =
                    ledger.current_definition(&next.definition.plan.definition)?;
                &held_definition
            }
            None => &next.definition.plan.definition,
        };
        let rate = d.rated_thrust_n * command.throttle / d.exhaust_velocity_m_s();
        if rate > 0. && next.propulsion.propellant_kg > 0. {
            step = step.min(world_vehicle::exhaustion_time(
                next.propulsion.propellant_kg,
                rate,
            ));
        }
        if let Some(t) = next.next_authored_removal_time() {
            let gap = t - next.propulsion.elapsed_s;
            if gap > 0. {
                step = step.min(gap);
            }
        }
        // Command-boundary fragments below the absolute craft-clock ulp cannot
        // advance the represented clock; carry the propulsion ledger across the
        // boundary without motion. The sub-ulp span's motion is unrepresentable.
        if left > 0. && next.last_time_s + left == next.last_time_s {
            let boundary = Command {
                duration_s: left,
                ..command.clone()
            };
            let requested = rate * left;
            let prop = next.propulsion.propellant_kg;
            if prop > 0. && requested > 0. && prop - (prop - requested).max(0.) == 0. {
                next.propulsion.elapsed_s += left;
                if let Some(c) = next.contact.as_mut() {
                    c.cumulative_work.duration_s += left;
                    c.last_work.duration_s += left;
                }
                next.last_interval = Some(crate::supply_vehicle::Interval {
                    duration_s: left,
                    initial_mass_kg: next.propulsion.mass_kg(d),
                    final_mass_kg: next.propulsion.mass_kg(d),
                    commanded_thrust_n: d.rated_thrust_n * command.throttle,
                    requested_propellant_kg: requested,
                    actual_propellant_kg: 0.,
                    impulse_n_s: 0.,
                    ideal_delta_v_m_s: 0.,
                    powered_duration_s: 0.,
                    fuel_limited_duration_s: 0.,
                    initial_powered_acceleration_m_s2: None,
                    final_powered_acceleration_m_s2: None,
                    mass_residual_kg: 0.,
                });
                continue;
            }
            let (_, _, propulsion, interval) = world_vehicle::drift(
                d,
                &next.propulsion,
                next.position_m,
                next.velocity_m_s,
                &boundary,
                Vec3d::ZERO,
            )?;
            next.propulsion = propulsion;
            if let Some(c) = next.contact.as_mut() {
                c.cumulative_work.duration_s += left;
                c.last_work.duration_s += left;
            }
            next.last_interval = Some(interval);
            continue;
        }
        if !step.is_finite()
            || step <= 0.
            || next.last_time_s + step == next.last_time_s
            || next.propulsion.elapsed_s + step == next.propulsion.elapsed_s
        {
            return Err("Contact substep is below represented flight-clock precision".into());
        }
        let launch_throttle = command.throttle;
        let launch_thrust_acceleration_m_s2 =
            d.rated_thrust_n * command.throttle / d.inert_mass_kg();
        next = substep(&next, world, &h, step, command)?;
        if next.definition.launch.is_some() {
            let sample_time = next.last_time_s;
            let elapsed_s = next.propulsion.elapsed_s;
            crate::vehicle_launch::observe(
                &mut next,
                world,
                sample_time,
                launch_throttle,
                launch_thrust_acceleration_m_s2,
                elapsed_s,
            )?;
        }
        if next.definition.entry.is_some() {
            let sample_time = next.last_time_s;
            let elapsed_s = next.propulsion.elapsed_s;
            crate::vehicle_entry::observe(&mut next, world, sample_time, elapsed_s, step)?;
        }
        if next.definition.landing.is_some() {
            let sample_time = next.last_time_s;
            let elapsed_s = next.propulsion.elapsed_s;
            crate::vehicle_landing::observe(&mut next, world, sample_time, elapsed_s)?;
        }
        count += 1;
    }
    while let Some(index) = next.pending_authored_removal() {
        next.apply_authored_removal(index)?;
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
pub fn context_for(h: &Hardware) -> serde_json::Value {
    if h.shear.is_none() {
        return context();
    }
    let mut value = context();
    value["model"] = serde_json::json!(world_vehicle::SHEAR_MODEL);
    value["contact_model"] = serde_json::json!(SHEAR_MODEL);
    value["shear_model"] = serde_json::json!(crate::vehicle_shear::MODEL);
    value["shear_model_descriptor"] = serde_json::to_value(
        crate::research::PhysicsModelRegistry::get("vehicle-persistent-shear"),
    )
    .expect("finite static model metadata");
    value["response"] = serde_json::json!("Finite normal spring/damper plus persistent tangential spring/Coulomb endpoint return; material-frame and small-normal rotation transports memory, zero legacy smooth friction, actual combined off-centre forces/torques");
    value["shear_integration"] = serde_json::json!("Midpoint mechanical force uses predicted half-step shear; full endpoint history uses midpoint relative velocity and a linear within-step contact fraction. Endpoint constitutive work/storage/plastic/unloading/numerical balance is separate from actual midpoint mechanical work; coupling residual is retained, not zeroed.");
    value["normal_fields"] = serde_json::json!("Foot elastic_energy_j and elastic_storage_rate_w remain normal-spring quantities. Tangential storage is computed from retained shear state and appears separately in shear.csv/work; sliding_dissipation_w is zero because shear slip is a discrete increment, not invented instantaneous power.");
    value["limits"] = serde_json::json!(["Persistent shear force does not establish footprint stability, attachment, safe landing certification or cargo transfer", "Authored endpoint law and reduced rigid feet, not the exact continuous 3D contact algorithm or measured regolith/gear", "Numerical endpoint dissipation and midpoint coupling residual are not calibrated ground friction", "No hull/other-body collision response, atmosphere/entry/heating or thrust attitude controller", "Finite-difference overview terrain is not measured meter-scale roughness or registered areoid shape", "Finite force/travel, small normal turns and substep budget; unsupported motion stops atomically"]);
    value["shear_sources"] = serde_json::json!(["https://arxiv.org/html/2002.10231v1","https://ntrs.nasa.gov/api/citations/20260000331/downloads/Apollo%2011%20LM%20Touchdown%20Dynamics%20VnV%20for%20HLS.pdf?attachment=true"]);
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        sandbox::{BodyProperty, Edit},
        supply_vehicle::{Cargo, Plan},
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
            cargo_ledgers: false,
            cargo_removals: vec![],
            handling: None,
            launch: None,
            entry: None,
            landing: None,
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
    fn co_rotating_cargo_removal_preserves_angular_velocity_and_closes() {
        let (mut world, mut d) = setup(200000., 0.);
        let h = d.contact.as_mut().unwrap();
        h.initial_angular_velocity_body_rad_s = Vec3d::new(0., 0., 0.2);
        d.cargo_ledgers = true;
        d.plan.definition.payload_capacity_kg = 5_000.;
        d.plan.definition.cargo.potable_water_kg = 2000.;
        world
            .apply_edit(Edit::CreateVehicle {
                definition: Box::new(d),
            })
            .unwrap();
        world.vehicles[0]
            .remove_cargo(
                Cargo {
                    oxygen_reserve_kg: 0.,
                    potable_water_kg: 800.,
                    food_dry_kg: 0.,
                    food_water_kg: 0.,
                },
                Vec3d::ZERO,
            )
            .unwrap();
        let omega_before = world.vehicles[0]
            .contact
            .as_ref()
            .unwrap()
            .angular_velocity_body_rad_s;
        let event = world.vehicles[0].cargo_events.last().unwrap();
        assert_eq!(event.craft_angular_velocity_after_body_rad_s, omega_before);
        assert!(event.carried_angular_momentum_magnitude_kg_m2_s > 0.);
        assert!(event.carried_rotational_energy_j > 0.);
        let start = world.time;
        world.advance_to(start + 1., None, None);
        assert_eq!(
            world.vehicles[0]
                .contact
                .as_ref()
                .unwrap()
                .angular_velocity_body_rad_s,
            omega_before
        );
        world.validate_vehicle_state().unwrap();
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

    fn shear_setup() -> (Universe, Definition) {
        let (world, mut d) = setup(1.6, 0.1);
        let h = d
            .contact
            .take()
            .unwrap()
            .with_shear(crate::vehicle_shear::Definition::example())
            .unwrap();
        d.model = h.flight_model().into();
        d.contact = Some(h);
        d.validate(&world).unwrap();
        (world, d)
    }
    #[test]
    fn persistent_shear_changes_actual_motion_and_keeps_separate_storage_work() {
        let (mut world, d) = shear_setup();
        let cargo = d.plan.definition.cargo.clone();
        let origin = world.time;
        launch(&mut world, d);
        world.advance_to(origin + 8., None, None);
        let f = &world.vehicles[0];
        assert_eq!(f.phase, Phase::Flying, "{:?}", f.phase);
        let r = f.contact.as_ref().unwrap();
        let work = r.cumulative_work.shear.as_ref().unwrap();
        assert!(r
            .feet
            .iter()
            .any(|f| f.shear.as_ref().unwrap().elastic_shear_m.length() > 1e-10));
        assert!(r.feet.iter().any(|f| f.response.normal_load_n > 0.));
        assert!(work.elastic_storage_change_j > 0.);
        assert!(work.numerical_dissipation_j > 0.);
        assert_eq!(f.definition.plan.definition.cargo, cargo);
        assert!(world.habitats.is_empty());
        assert!(r.cumulative_work.contact_balance_residual_j.is_finite());
        world.validate_vehicle_state().unwrap();
        let (mut legacy, old) = setup(1.6, 0.1);
        launch(&mut legacy, old);
        legacy.advance_to(origin + 8., None, None);
        assert_ne!(f.velocity_m_s, legacy.vehicles[0].velocity_m_s);
        assert!(legacy.vehicles[0]
            .contact
            .as_ref()
            .unwrap()
            .cumulative_work
            .shear
            .is_none());
    }
    #[test]
    fn persistent_shear_save_chunks_and_complete_recordings_replay_exactly() {
        let (world, d) = shear_setup();
        let mut direct = world.clone();
        let origin = world.time;
        launch(&mut direct, d.clone());
        direct.advance_to(origin + 4., None, None);
        let mut split = world.clone();
        launch(&mut split, d.clone());
        split.advance_to(origin + 2., None, None);
        split = crate::save::from_json(&crate::save::to_json(&split, "shear").unwrap()).unwrap();
        split.advance_to(origin + 4., None, None);
        assert_eq!(
            crate::research::state_fingerprint(&direct).unwrap(),
            crate::research::state_fingerprint(&split).unwrap()
        );
        let study = crate::world_vehicle_recording::Study::new(&world, d, 5, 1.).unwrap();
        let bundle = crate::world_vehicle_recording::Bundle::run(&world, study).unwrap();
        let path =
            std::env::temp_dir().join(format!("cosmogon-shear-flight-{}", std::process::id()));
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }
        bundle.export(&path).unwrap();
        let replay = crate::world_vehicle_recording::Bundle::read(&path).unwrap();
        assert_eq!(bundle.manifest, replay.manifest);
        assert_eq!(bundle.manifest["model"], world_vehicle::SHEAR_MODEL);
        assert!(bundle.manifest["files"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("shear.csv")));
        let contact = std::fs::read_to_string(path.join("contact.csv")).unwrap();
        let count = contact.lines().next().unwrap().split(',').count();
        assert!(contact
            .lines()
            .skip(1)
            .all(|line| line.split(',').count() == count));
        std::fs::write(path.join("shear.csv"), "changed").unwrap();
        assert!(crate::world_vehicle_recording::Bundle::read(&path).is_err());
        std::fs::remove_dir_all(&path).unwrap();
    }
    #[test]
    fn persistent_shear_versions_history_and_model_pins_are_strict() {
        let (mut world, d) = shear_setup();
        let origin = world.time;
        let mut mixed = d.clone();
        mixed.model = world_vehicle::CONTACT_MODEL.into();
        assert!(mixed.validate(&world).is_err());
        let mut mixed = d.clone();
        mixed.contact.as_mut().unwrap().feet[0]
            .response
            .sliding_friction = 0.1;
        assert!(mixed.validate(&world).is_err());
        launch(&mut world, d);
        world.advance_to(origin + 3., None, None);
        assert_eq!(world.vehicles[0].phase, Phase::Flying);
        let original = serde_json::to_value(
            serde_json::from_str::<serde_json::Value>(
                &crate::save::to_json(&world, "shear").unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        for key in [
            "cosmogon-finite-foot-shear",
            "cosmogon-persistent-terrain-contact",
            "cosmogon-world-vehicle-shear-capability",
        ] {
            let mut bad = original.clone();
            bad["header"]["data_versions"]
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(crate::save::from_json(&bad.to_string()).is_err(), "{key}");
        }
        let mut bad = original.clone();
        bad["universe"]["vehicles"][0]["contact"]["feet"][0]["shear"]["elastic_shear_m"]["x"] =
            serde_json::json!(10.);
        assert!(crate::save::from_json(&bad.to_string()).is_err());
        let mut bad = original.clone();
        bad["universe"]["vehicles"][0]["contact"]["cumulative_work"]
            .as_object_mut()
            .unwrap()
            .remove("shear");
        assert!(crate::save::from_json(&bad.to_string()).is_err());
    }
    #[test]
    fn persistent_shear_excludes_free_motion_before_linear_first_contact() {
        let (mut world, d) = shear_setup();
        launch(&mut world, d.clone());
        let rigid = world.vehicles[0].contact.as_ref().unwrap();
        let h = d.contact.as_ref().unwrap();
        let mut a = rigid.feet[0].clone();
        let mut b = a.clone();
        // Controlled crossing readings: only the final half of the 0.01 s
        // interval is in contact; the 1 m/s tangent motion is not all preload.
        a.surface.tangent_gap_m = 0.01;
        b.surface.tangent_gap_m = -0.01;
        b.surface.frame =
            crate::vehicle_arrival::SurfaceFrame::at(&world, d.parent, world.time + 0.01).unwrap();
        b.response.normal_load_n = 100.;
        b.response.force_n = b.surface.outward_normal * 100.;
        let tangent = b
            .surface
            .outward_normal
            .cross(Vec3d::new(0., 0., 1.))
            .normalize();
        let mut predicted = vec![b];
        let mut one = h.clone();
        one.feet.truncate(1);
        let work = evolve_shear(
            &[a],
            &mut predicted,
            &[tangent],
            0.01,
            rigid,
            world.vehicles[0].velocity_m_s,
            &one,
        )
        .unwrap()
        .unwrap();
        assert!(
            (predicted[0]
                .shear
                .as_ref()
                .unwrap()
                .elastic_shear_m
                .length()
                - 0.005)
                .abs()
                < 1e-10
        );
        assert_eq!(work.plastic_dissipation_j, 0.);
        assert!((work.elastic_storage_change_j - 0.0625).abs() < 1e-10);
        assert!(work.balance_residual_j.abs() < 1e-10);
    }
}
