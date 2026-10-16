//! Actual contact wrench and conditional planar hold evidence; never an attachment.
use crate::{
    vehicle_arrival::{Approach, SurfaceFrame},
    world_vehicle::{Flight, Source},
    Universe, Vec3d,
};
use cosmogon_core::{dmath::DMath, math::Quatd};
use serde::{Deserialize, Serialize};
pub const MODEL: &str = "cosmogon-vehicle-footprint-support/1";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub maximum_plane_departure_m: f64,
    pub maximum_normal_deviation_rad: f64,
    pub maximum_position_roundoff_m: f64,
    pub minimum_hull_edge_m: f64,
    pub minimum_load_margin_m: f64,
    pub maximum_relative_speed_m_s: f64,
    pub maximum_relative_angular_speed_rad_s: f64,
    pub maximum_relative_acceleration_m_s2: f64,
    pub maximum_force_imbalance_fraction: f64,
    pub maximum_torque_imbalance_fraction: f64,
    pub derivative_step_s: f64,
    pub maximum_derivative_error_m_s2: f64,
}
impl Default for Definition {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            maximum_plane_departure_m: 0.02,
            maximum_normal_deviation_rad: 0.01,
            maximum_position_roundoff_m: 0.01,
            minimum_hull_edge_m: 0.05,
            minimum_load_margin_m: 0.1,
            maximum_relative_speed_m_s: 0.02,
            maximum_relative_angular_speed_rad_s: 0.001,
            maximum_relative_acceleration_m_s2: 0.02,
            maximum_force_imbalance_fraction: 0.05,
            maximum_torque_imbalance_fraction: 0.05,
            derivative_step_s: 0.5,
            maximum_derivative_error_m_s2: 1e-5,
        }
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err("Unsupported footprint-support model".into());
        }
        for (v, lo, hi) in [
            (self.maximum_plane_departure_m, 1e-6, 1.),
            (self.maximum_normal_deviation_rad, 1e-6, 0.1),
            (self.maximum_position_roundoff_m, 1e-9, 0.1),
            (self.minimum_hull_edge_m, 1e-6, 100.),
            (self.minimum_load_margin_m, 0., 100.),
            (self.maximum_relative_speed_m_s, 1e-6, 10.),
            (self.maximum_relative_angular_speed_rad_s, 1e-8, 1.),
            (self.maximum_relative_acceleration_m_s2, 1e-8, 10.),
            (self.maximum_force_imbalance_fraction, 1e-6, 1.),
            (self.maximum_torque_imbalance_fraction, 1e-6, 1.),
            (self.derivative_step_s, 0.01, 10.),
            (self.maximum_derivative_error_m_s2, 1e-10, 0.1),
        ] {
            if !v.is_finite() || !(lo..=hi).contains(&v) {
                return Err("Footprint/support assumptions exceed finite supported bounds".into());
            }
        }
        Ok(())
    }
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let v: Self = serde_json::from_str(&crate::habitat_recording::read(path, 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        v.validate()?;
        Ok(v)
    }
    pub fn write(&self, path: &std::path::Path) -> Result<(), String> {
        self.validate()?;
        use std::io::Write;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FootLoad {
    pub index: usize,
    pub loaded: bool,
    pub normal_load_n: f64,
    pub tangent_force_n: f64,
    pub structural_utilization: f64,
    pub stroke_utilization: f64,
    pub friction_utilization: Option<f64>,
    pub relative_velocity_m_s: Vec3d,
    pub normal_storage_j: f64,
    pub tangent_storage_j: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Planar {
    /// Local inertial vector from actual craft centre to the declared ground plane.
    pub origin_from_centre_m: Vec3d,
    pub normal: Vec3d,
    pub axis_u: Vec3d,
    pub axis_v: Vec3d,
    pub maximum_plane_departure_m: f64,
    pub maximum_normal_deviation_rad: f64,
    pub hull_uv_m: Vec<[f64; 2]>,
    pub hull_area_m2: f64,
    pub geometric_centre_margin_m: f64,
    pub force_weighted_site_uv_m: [f64; 2],
    pub force_weighted_site_margin_m: f64,
    pub wrench_centre_uv_m: [f64; 2],
    pub wrench_centre_margin_m: f64,
    pub residual_normal_torque_n_m: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    pub ground_acceleration_at_centre_m_s2: Vec3d,
    pub ground_angular_acceleration_rad_s2: Vec3d,
    pub derivative_error_indicator_m_s2: f64,
    pub gravity_acceleration_m_s2: Vec3d,
    pub commanded_thrust_n: Vec3d,
    pub required_contact_force_n: Vec3d,
    pub required_contact_torque_n_m: Vec3d,
    pub force_imbalance_n: Vec3d,
    pub torque_imbalance_n_m: Vec3d,
    /// Acceleration of body-fixed coordinates, expressed in instantaneous inertial axes.
    pub relative_acceleration_m_s2: Vec3d,
    pub load_line_uv_m: Option<[f64; 2]>,
    pub load_line_margin_m: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    pub model: String,
    pub vehicle_id: u64,
    pub world_time_s: f64,
    pub craft_time_s: f64,
    pub frame: SurfaceFrame,
    pub mass_kg: f64,
    pub position_roundoff_indicator_m: f64,
    pub relative_velocity_m_s: Vec3d,
    pub relative_angular_velocity_rad_s: Vec3d,
    pub contact_force_n: Vec3d,
    pub contact_torque_about_centre_n_m: Vec3d,
    pub feet: Vec<FootLoad>,
    pub planar: Option<Planar>,
    pub planar_unavailable_reason: Option<String>,
    pub hold: Option<Hold>,
    pub hold_unavailable_reason: Option<String>,
    pub within_declared_instantaneous_envelope: bool,
    pub envelope_exclusions: Vec<String>,
}
fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}
fn rotation(v: [f64; 4]) -> Quatd {
    Quatd::new(v[0], v[1], v[2], v[3])
}
fn inv(v: Quatd) -> Quatd {
    Quatd::new(-v.x, -v.y, -v.z, v.w)
}
fn diagonal(v: Vec3d, d: Vec3d) -> Vec3d {
    Vec3d::new(v.x * d.x, v.y * d.y, v.z * d.z)
}
fn cross2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn hull(mut points: Vec<[f64; 2]>, edge: f64) -> Result<Vec<[f64; 2]>, String> {
    if points.len() > 8 || points.iter().flatten().any(|v| !v.is_finite()) {
        return Err("Unsupported finite footprint points".into());
    }
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();
    if points.len() < 3 {
        return Err("At least three distinct loaded sites are required for a planar hull".into());
    }
    let mut h: Vec<[f64; 2]> = vec![];
    for p in &points {
        while h.len() >= 2 && cross2(h[h.len() - 2], h[h.len() - 1], *p) <= 0. {
            h.pop();
        }
        h.push(*p);
    }
    let lower = h.len();
    for p in points.iter().rev().skip(1) {
        while h.len() > lower && cross2(h[h.len() - 2], h[h.len() - 1], *p) <= 0. {
            h.pop();
        }
        h.push(*p);
    }
    h.pop();
    if h.len() < 3
        || (0..h.len()).any(|i| {
            let a = h[i];
            let b = h[(i + 1) % h.len()];
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() < edge
        })
    {
        return Err("Loaded footprint is degenerate or below the declared edge resolution".into());
    }
    if area(&h) < edge * edge {
        return Err("Loaded footprint area is below the declared resolution".into());
    }
    Ok(h)
}
fn area(h: &[[f64; 2]]) -> f64 {
    (0..h.len())
        .map(|i| h[i][0] * h[(i + 1) % h.len()][1] - h[i][1] * h[(i + 1) % h.len()][0])
        .sum::<f64>()
        * 0.5
}
fn margin(h: &[[f64; 2]], p: [f64; 2]) -> f64 {
    (0..h.len())
        .map(|i| {
            let a = h[i];
            let b = h[(i + 1) % h.len()];
            cross2(a, b, p) / ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}
fn planar(
    flight: &Flight,
    d: &Definition,
    force: Vec3d,
    torque: Vec3d,
    resolution: f64,
) -> Result<Planar, String> {
    if resolution > d.maximum_position_roundoff_m {
        return Err(
            "System-local coordinate roundoff exceeds the declared footprint resolution".into(),
        );
    }
    let rigid = flight.contact.as_ref().ok_or("No terrain contact state")?;
    let loaded: Vec<_> = rigid
        .feet
        .iter()
        .filter(|f| f.response.normal_load_n > 0.)
        .collect();
    if loaded.len() < 3 {
        return Err(
            "Fewer than three loaded contact sites; planar support hull unavailable".into(),
        );
    }
    let mut n = Vec3d::ZERO;
    let mut origin = Vec3d::ZERO;
    for f in &loaded {
        n += f.surface.outward_normal;
        origin += f.surface.position_m - flight.position_m;
    }
    if n.length() < loaded.len() as f64 * 0.5 {
        return Err("Contact normals do not define a shared outward plane".into());
    }
    n = n.normalize();
    origin /= loaded.len() as f64;
    let axis = if n.z.abs() < 0.8 {
        Vec3d::new(0., 0., 1.)
    } else {
        Vec3d::new(1., 0., 0.)
    };
    let u = axis.cross(n).normalize();
    let v = n.cross(u);
    let mut departure: f64 = 0.;
    let mut deviation: f64 = 0.;
    let mut points = vec![];
    let mut weighted = [0., 0.];
    let mut weight = 0.;
    for f in &loaded {
        let r = f.surface.position_m - flight.position_m - origin;
        departure = departure.max(r.dot(n).abs());
        deviation = deviation.max(
            n.cross(f.surface.outward_normal)
                .length()
                .datan2(n.dot(f.surface.outward_normal)),
        );
        let p = [r.dot(u), r.dot(v)];
        let w = f.response.force_n.dot(n);
        if w <= 0. {
            return Err("Projected ground reaction is not unilateral on the declared plane".into());
        }
        points.push(p);
        weighted[0] += w * p[0];
        weighted[1] += w * p[1];
        weight += w;
    }
    if departure + resolution > d.maximum_plane_departure_m
        || deviation > d.maximum_normal_deviation_rad
    {
        return Err(
            "Contact terrain is outside the declared common-plane/normal approximation".into(),
        );
    }
    let h = hull(points, d.minimum_hull_edge_m.max(4. * resolution))?;
    let normal_force = force.dot(n);
    if normal_force <= 0. {
        return Err("No positive resultant reaction on the declared plane".into());
    }
    let at_origin = torque - origin.cross(force);
    let cop = n.cross(at_origin) / normal_force;
    let centre = [cop.dot(u), cop.dot(v)];
    let weighted = [weighted[0] / weight, weighted[1] / weight];
    let geometric = [-origin.dot(u), -origin.dot(v)];
    Ok(Planar {
        origin_from_centre_m: origin,
        normal: n,
        axis_u: u,
        axis_v: v,
        maximum_plane_departure_m: departure,
        maximum_normal_deviation_rad: deviation,
        hull_area_m2: area(&h),
        geometric_centre_margin_m: margin(&h, geometric),
        force_weighted_site_uv_m: weighted,
        force_weighted_site_margin_m: margin(&h, weighted),
        wrench_centre_uv_m: centre,
        wrench_centre_margin_m: margin(&h, centre),
        residual_normal_torque_n_m: at_origin.dot(n),
        hull_uv_m: h,
    })
}
fn derivative(
    world: &Universe,
    frame: &SurfaceFrame,
    h: f64,
) -> Result<(Vec3d, Vec3d, f64), String> {
    let a = SurfaceFrame::at(world, frame.target, frame.time_s - h)?;
    let b = SurfaceFrame::at(world, frame.target, frame.time_s + h)?;
    if a.rotation_basis != frame.rotation_basis
        || b.rotation_basis != frame.rotation_basis
        || a.radius_m != frame.radius_m
        || b.radius_m != frame.radius_m
        || a.axial_tilt_rad != frame.axial_tilt_rad
        || b.axial_tilt_rad != frame.axial_tilt_rad
    {
        return Err("Frame derivative crosses unsupported model/geometry identity".into());
    }
    let dt = b.time_s - a.time_s;
    if dt <= 0. || (dt - 2. * h).abs() > h * 1e-3 {
        return Err("Frame derivative is below supported clock resolution".into());
    }
    let acc = (b.centre_velocity_m_s - a.centre_velocity_m_s) / dt;
    let alpha = (b.angular_velocity_inertial_rad_s - a.angular_velocity_inertial_rad_s) / dt;
    let roundoff = 32.
        * f64::EPSILON
        * (a.centre_velocity_m_s.length()
            + b.centre_velocity_m_s.length()
            + frame.radius_m
                * (a.angular_velocity_inertial_rad_s.length()
                    + b.angular_velocity_inertial_rad_s.length()))
        / dt;
    if !finite(acc) || !finite(alpha) || !roundoff.is_finite() {
        return Err("Frame derivative exceeds finite support".into());
    }
    Ok((acc, alpha, roundoff))
}
fn hold(
    world: &Universe,
    flight: &Flight,
    d: &Definition,
    frame: &SurfaceFrame,
    reaction: (Vec3d, Vec3d),
    relative: Vec3d,
    p: Option<&Planar>,
) -> Result<Hold, String> {
    let (force, torque) = reaction;
    let full = derivative(world, frame, d.derivative_step_s)?;
    let half = derivative(world, frame, d.derivative_step_s * 0.5)?;
    let r = flight.position_m - frame.centre_m;
    let error =
        (full.0 - half.0).length() + (full.1 - half.1).length() * r.length() + full.2 + half.2;
    if !error.is_finite() || error > d.maximum_derivative_error_m_s2 {
        return Err("Moving-ground derivative error indicator exceeds the declared limit".into());
    }
    let ground = half.0
        + half.1.cross(r)
        + frame
            .angular_velocity_inertial_rad_s
            .cross(frame.angular_velocity_inertial_rad_s.cross(r));
    let mut gravity = Vec3d::ZERO;
    for source in &flight.last_sources {
        let delta = source.position_m - flight.position_m;
        let length = delta.length();
        if length == 0.
            || (source.source != Source::Body(frame.target.body) && length <= source.radius_m)
        {
            return Err("Unsupported gravity geometry for hold comparison".into());
        }
        gravity += delta * (source.mu_m3_s2 / (length * length * length));
    }
    let hardware = flight
        .definition
        .contact
        .as_ref()
        .ok_or("No contact hardware")?;
    let rigid = flight.contact.as_ref().ok_or("No contact state")?;
    let mass = flight
        .propulsion
        .mass_kg(&flight.definition.plan.definition);
    let (command, _) =
        crate::world_vehicle::active_command(&flight.definition.plan, flight.propulsion.elapsed_s);
    let thrust = if flight.propulsion.propellant_kg > 0. {
        Vec3d::new(
            command.direction[0],
            command.direction[1],
            command.direction[2],
        ) * (flight.definition.plan.definition.rated_thrust_n * command.throttle)
    } else {
        Vec3d::ZERO
    };
    let required = (ground - gravity) * mass - thrust;
    let rotation = rotation(rigid.orientation_xyzw);
    let omega_body = inv(rotation).rotate_vec3(frame.angular_velocity_inertial_rad_s);
    let radii = hardware.gyration_radii_m;
    let inertia = Vec3d::new(radii.x * radii.x, radii.y * radii.y, radii.z * radii.z) * mass;
    let required_torque = rotation.rotate_vec3(
        diagonal(inv(rotation).rotate_vec3(half.1), inertia)
            + omega_body.cross(diagonal(omega_body, inertia)),
    );
    let relative_acceleration = gravity + (force + thrust) / mass
        - ground
        - frame.angular_velocity_inertial_rad_s.cross(relative) * 2.;
    let line = p.and_then(|p| {
        let denominator = p.normal.dot(required);
        if denominator <= mass * 1e-9 {
            return None;
        }
        let t = p.origin_from_centre_m.dot(p.normal) / denominator;
        let delta = required * t - p.origin_from_centre_m;
        Some([delta.dot(p.axis_u), delta.dot(p.axis_v)])
    });
    if [
        ground,
        gravity,
        thrust,
        required,
        required_torque,
        relative_acceleration,
    ]
    .into_iter()
    .any(|v| !finite(v))
    {
        return Err("Hold comparison exceeds finite numerical support".into());
    }
    Ok(Hold {
        ground_acceleration_at_centre_m_s2: ground,
        ground_angular_acceleration_rad_s2: half.1,
        derivative_error_indicator_m_s2: error,
        gravity_acceleration_m_s2: gravity,
        commanded_thrust_n: thrust,
        required_contact_force_n: required,
        required_contact_torque_n_m: required_torque,
        force_imbalance_n: force - required,
        torque_imbalance_n_m: torque - required_torque,
        relative_acceleration_m_s2: relative_acceleration,
        load_line_margin_m: line.zip(p).map(|(line, p)| margin(&p.hull_uv_m, line)),
        load_line_uv_m: line,
    })
}
impl Reading {
    pub fn live(world: &Universe, flight: &Flight, d: &Definition) -> Result<Self, String> {
        d.validate()?;
        flight.validate_saved(world)?;
        // Reject terminal/source-edited live reconstruction before using any frame derivative.
        Approach::live(world, flight, flight.definition.parent)?;
        Self::from_flight(world, flight, d)
    }
    /// Evaluate a retained state at its own craft clock without the public live reader's
    /// `world.time` advancement assumption. Callers (the contact solver's operation
    /// sampler) are responsible for clock, source and phase validity.
    pub(crate) fn from_flight(world: &Universe, flight: &Flight, d: &Definition) -> Result<Self, String> {
        let h = flight
            .definition
            .contact
            .as_ref()
            .ok_or("Spherical flight has no terrain-foot support state")?;
        let rigid = flight
            .contact
            .as_ref()
            .ok_or("No retained terrain-foot state")?;
        let frame = rigid
            .feet
            .first()
            .ok_or("No actual retained contact frame")?
            .surface
            .frame
            .clone();
        let omega = rotation(rigid.orientation_xyzw).rotate_vec3(rigid.angular_velocity_body_rad_s);
        let relative = flight.velocity_m_s
            - frame.centre_velocity_m_s
            - frame
                .angular_velocity_inertial_rad_s
                .cross(flight.position_m - frame.centre_m);
        let angular = omega - frame.angular_velocity_inertial_rad_s;
        let mut force = Vec3d::ZERO;
        let mut torque = Vec3d::ZERO;
        let mut feet = vec![];
        let mut scale = flight.position_m.length();
        for (i, (hardware, f)) in h.feet.iter().zip(&rigid.feet).enumerate() {
            force += f.response.force_n;
            torque += f.torque_about_centre_n_m;
            scale = scale.max(f.surface.position_m.length());
            let tangent =
                (f.response.force_n - f.surface.outward_normal * f.response.normal_load_n).length();
            let coefficient = h
                .shear
                .as_ref()
                .map_or(hardware.response.sliding_friction, |s| {
                    s.friction_coefficient
                });
            let capacity = coefficient * f.response.normal_load_n;
            feet.push(FootLoad {
                index: i,
                loaded: f.response.normal_load_n > 0.,
                normal_load_n: f.response.normal_load_n,
                tangent_force_n: tangent,
                structural_utilization: f.response.force_n.length()
                    / hardware.response.maximum_load_n,
                stroke_utilization: f.response.compression_m / hardware.response.maximum_stroke_m,
                friction_utilization: (capacity > 0.).then(|| tangent / capacity),
                relative_velocity_m_s: f.velocity_m_s - f.surface.velocity_m_s,
                normal_storage_j: f.response.stored_elastic_energy_j,
                tangent_storage_j: h
                    .shear
                    .as_ref()
                    .zip(f.shear.as_ref())
                    .map(|(d, s)| 0.5 * d.stiffness_n_m * s.elastic_shear_m.length_squared()),
            });
        }
        let resolution = 32. * f64::EPSILON * scale;
        let plane = planar(flight, d, force, torque, resolution);
        let (planar, planar_reason) = match plane {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e)),
        };
        let hold = hold(
            world,
            flight,
            d,
            &frame,
            (force, torque),
            relative,
            planar.as_ref(),
        );
        let (hold, hold_reason) = match hold {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e)),
        };
        let mut exclusions = vec![];
        if h.shear.is_none() {
            exclusions
                .push("Smooth sliding friction lacks persistent zero-speed shear support".into());
        }
        if relative.length() > d.maximum_relative_speed_m_s {
            exclusions.push("Ground-relative translation exceeds the declared limit".into());
        }
        if angular.length() > d.maximum_relative_angular_speed_rad_s {
            exclusions.push("Ground-relative rotation exceeds the declared limit".into());
        }
        if let Some(reason) = &planar_reason {
            exclusions.push(reason.clone());
        }
        if let Some(reason) = &hold_reason {
            exclusions.push(reason.clone());
        }
        if let Some(a) = &hold {
            if a.commanded_thrust_n.length() > 0. {
                exclusions
                    .push("Powered thrust excludes passive surface-operation evidence".into());
            }
            if a.relative_acceleration_m_s2.length() > d.maximum_relative_acceleration_m_s2 {
                exclusions.push("Ground-relative acceleration exceeds the declared limit".into());
            }
            let required = a.required_contact_force_n.length();
            if required <= 0.
                || a.force_imbalance_n.length() > required * d.maximum_force_imbalance_fraction
            {
                exclusions
                    .push("Actual reaction does not balance the declared ground-hold force".into());
            }
            let lever = planar.as_ref().map(|p| p.hull_area_m2.sqrt()).unwrap_or(0.);
            if lever <= 0.
                || a.torque_imbalance_n_m.length()
                    > required * lever * d.maximum_torque_imbalance_fraction
            {
                exclusions.push(
                    "Actual torque does not balance the declared ground-following rotation".into(),
                );
            }
            if a.load_line_margin_m
                .is_none_or(|m| m < d.minimum_load_margin_m + resolution)
            {
                exclusions
                    .push("Required hold-load line lacks the declared footprint margin".into());
            }
        }
        if let Some(p) = &planar {
            if p.wrench_centre_margin_m < d.minimum_load_margin_m + resolution {
                exclusions
                    .push("Actual reaction wrench lacks the declared footprint margin".into());
            }
        }
        if !finite(force) || !finite(torque) || !finite(relative) || !finite(angular) {
            return Err("Actual support readings exceed finite numerical support".into());
        }
        Ok(Self {
            model: MODEL.into(),
            vehicle_id: flight.id,
            world_time_s: world.time,
            craft_time_s: flight.last_time_s,
            frame,
            mass_kg: flight
                .propulsion
                .mass_kg(&flight.definition.plan.definition),
            position_roundoff_indicator_m: resolution,
            relative_velocity_m_s: relative,
            relative_angular_velocity_rad_s: angular,
            contact_force_n: force,
            contact_torque_about_centre_n_m: torque,
            feet,
            planar,
            planar_unavailable_reason: planar_reason,
            hold,
            hold_unavailable_reason: hold_reason,
            within_declared_instantaneous_envelope: exclusions.is_empty(),
            envelope_exclusions: exclusions,
        })
    }
}
pub fn context() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"descriptor":crate::research::PhysicsModelRegistry::get("vehicle-footprint-support"),"scope":"Actual retained foot forces, geometry and conditional instantaneous planar ground-hold comparisons; not attachment, a dynamic stability certificate, landing permission or cargo access","data_state":"derived-simulation","plane":"Loaded modeled surface sites; actual applied foot wrench intersects their declared plane, so compliance/tangent forces can separate its centre from the force-weighted site centroid","load_line":"Force-only zero-relative-speed hold line through actual craft centre; rotation torque is checked separately","frame_acceleration":"Centred derivatives of the exact replayed world source velocity/spin operator at the retained craft clock, half-step comparison and roundoff indicators; not a new ephemeris or a rigorous error bound","relative_acceleration":"g + (reaction + current scheduled finite-fuel thrust)/mass - material acceleration at centre - 2 omega cross relative velocity","envelope":"All thresholds are explicit authored limits; instantaneous inclusion neither proves future support nor authorizes transfer","sources":["https://underactuated.csail.mit.edu/humanoids.html"],"empirical_validation":null,"uncertainty":null,"limits":["Finite terrain-normal probe and overview relief are not measured local hazards","Fixed feet and point-mass gravity omit articulation, soil, hull contacts and gravity-gradient torque","No stable-duration, failure probability, hardware calibration or automatic phase change","No generated ground frame from a later live world for terminal historical flight"]})
}

#[cfg(test)]
pub(crate) fn fixture() -> (Universe, crate::world_vehicle::Definition) {
    use crate::{
        vehicle_arrival::KinematicState, vehicle_contact::Surface,
        vehicle_contact_dynamics::Hardware, world_vehicle,
    };
    let world = Universe::new(crate::UniverseSettings {
        scenario: crate::Scenario::SolarSystemLab,
        ..Default::default()
    });
    let target = crate::BodyRef {
        system: 0,
        body: world.systems[0].find_body("Mars").unwrap() as u32,
    };
    let frame = SurfaceFrame::at(&world, target, world.time).unwrap();
    let above = frame
        .to_inertial(&KinematicState {
            position_m: Vec3d::new(0.8, 0.6, 0.) * (frame.radius_m + 10000.),
            velocity_m_s: Vec3d::ZERO,
        })
        .unwrap();
    let surface = Surface::sample(&world, target, world.time, above.position_m, 1.).unwrap();
    let position = surface.position_m + surface.outward_normal * 1.6;
    let motion = frame
        .to_inertial(&KinematicState {
            position_m: frame.rotate_to_fixed(position - frame.centre_m),
            velocity_m_s: Vec3d::ZERO,
        })
        .unwrap();
    let velocity = motion.velocity_m_s - surface.outward_normal * 0.1;
    let mut plan = crate::supply_vehicle::Plan::example();
    for c in &mut plan.commands {
        c.throttle = 0.;
    }
    let body = world.systems[0].body_state(target.body as usize, world.time);
    let h = Hardware::example(&world, target, position)
        .unwrap()
        .with_shear(crate::vehicle_shear::Definition::example())
        .unwrap();
    let d = world_vehicle::Definition {
        model: h.flight_model().into(),
        frame: world_vehicle::FRAME.into(),
        parent: target,
        plan,
        initial_relative_position_m: position - body.pos,
        initial_relative_velocity_m_s: velocity - body.vel,
        contact: Some(h),
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_hull_has_signed_margins_and_refuses_unresolved_support() {
        let points = vec![
            [1., 1.],
            [-1., 1.],
            [1., -1.],
            [-1., -1.],
            [0., 0.],
            [1., 1.],
        ];
        let h = hull(points.clone(), 0.01).unwrap();
        let mut reversed = points;
        reversed.reverse();
        assert_eq!(h, hull(reversed, 0.01).unwrap());
        assert_eq!(area(&h), 4.);
        assert_eq!(margin(&h, [0., 0.]), 1.);
        assert_eq!(margin(&h, [2., 0.]), -1.);
        assert_eq!(margin(&h, [1., 0.]), 0.);
        assert!(hull(vec![[0., 0.], [1., 0.], [2., 0.]], 0.01).is_err());
        assert!(hull(vec![[0., 0.], [1., 0.], [1., 1e-8]], 0.01).is_err());
        assert!(hull(vec![[0., 0.], [1., 0.]], 0.01).is_err());
    }
    #[test]
    fn wrench_plane_centre_retains_compliance_moment_instead_of_site_average() {
        let (world, d) = fixture();
        let mut f = Flight::new(world.next_vehicle_id, d, &world).unwrap();
        let rigid = f.contact.as_mut().unwrap();
        let mut force = Vec3d::ZERO;
        let mut torque = Vec3d::ZERO;
        for (foot, (x, y)) in
            rigid
                .feet
                .iter_mut()
                .zip([(-1.5, -1.5), (-1.5, 1.5), (1.5, -1.5), (1.5, 1.5)])
        {
            // Independent planar wrench fixture: applications 0.1m above the ground plane.
            foot.surface.position_m = f.position_m + Vec3d::new(x, y, -1.5);
            foot.surface.outward_normal = Vec3d::new(0., 0., 1.);
            foot.response.normal_load_n = 100.;
            foot.response.force_n = Vec3d::new(30., 0., 100.);
            foot.torque_about_centre_n_m = Vec3d::new(x, y, -1.4).cross(foot.response.force_n);
            force += foot.response.force_n;
            torque += foot.torque_about_centre_n_m;
        }
        let p = planar(&f, &Definition::default(), force, torque, 0.).unwrap();
        assert!(p.force_weighted_site_uv_m.iter().all(|v| v.abs() < 1e-9));
        let actual = p.axis_u * p.wrench_centre_uv_m[0] + p.axis_v * p.wrench_centre_uv_m[1];
        assert!((actual - Vec3d::new(-0.03, 0., 0.)).length() < 1e-9);
        assert!((p.wrench_centre_margin_m - 1.47).abs() < 1e-9);
        assert!(planar(&f, &Definition::default(), force, torque, 1.).is_err());
        f.contact.as_mut().unwrap().feet[0].surface.position_m.z += 0.2;
        assert!(planar(&f, &Definition::default(), force, torque, 0.).is_err());
    }
    #[test]
    fn actual_loaded_reading_keeps_wrench_motion_and_unavailable_geometry_separate() {
        let (mut world, d) = fixture();
        world
            .apply_edit(crate::sandbox::Edit::CreateVehicle {
                definition: Box::new(d),
            })
            .unwrap();
        let initial = Reading::live(&world, &world.vehicles[0], &Definition::default()).unwrap();
        assert!(initial.planar.is_none());
        assert!(!initial.within_declared_instantaneous_envelope);
        assert!(initial.planar_unavailable_reason.is_some());
        world.advance_to(world.time + 4., None, None);
        let f = &world.vehicles[0];
        let r = Reading::live(&world, f, &Definition::default()).unwrap();
        assert_eq!(r.feet.iter().filter(|f| f.loaded).count(), 4);
        assert!(r.planar.is_some(), "{:?}", r.planar_unavailable_reason);
        assert!(r.hold.is_some(), "{:?}", r.hold_unavailable_reason);
        let actual = f
            .contact
            .as_ref()
            .unwrap()
            .feet
            .iter()
            .fold(Vec3d::ZERO, |s, f| s + f.response.force_n);
        assert_eq!(r.contact_force_n, actual);
        let a = r.hold.unwrap();
        assert_eq!(
            a.force_imbalance_n,
            r.contact_force_n - a.required_contact_force_n
        );
        assert!(
            a.derivative_error_indicator_m_s2
                <= Definition::default().maximum_derivative_error_m_s2
        );
        let strict = Definition {
            maximum_relative_speed_m_s: 1e-6,
            ..Default::default()
        };
        let r = Reading::live(&world, f, &strict).unwrap();
        assert!(r.planar.unwrap().force_weighted_site_margin_m > 0.);
        assert!(!r.within_declared_instantaneous_envelope);
        assert!(r
            .envelope_exclusions
            .iter()
            .any(|v| v.contains("translation")));
        let target = f.definition.parent;
        world
            .apply_edit(crate::sandbox::Edit::SetProperty {
                body: target,
                property: crate::sandbox::BodyProperty::RotationPeriod(1000.),
            })
            .unwrap();
        assert!(Reading::live(&world, &world.vehicles[0], &Definition::default()).is_err());
    }
    #[test]
    fn source_velocity_derivative_has_bounded_error_against_independent_kepler_acceleration() {
        let world = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::Sol,
            ..Default::default()
        });
        let target = crate::BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let f = SurfaceFrame::at(&world, target, world.time).unwrap();
        let (a, alpha, _) = derivative(&world, &f, 0.5).unwrap();
        let sys = &world.systems[0];
        let r = f.centre_m - sys.star_local_position(world.time);
        let expected = r * (-sys.star.mu() / r.length().powi(3)); // Existing DMath orbital state operator has approximation error distinct from derivative refinement.
        assert!(
            (a - expected).length() < expected.length() * 0.001,
            "a={a:?} expected={expected:?}"
        );
        assert_eq!(alpha, Vec3d::ZERO);
        let invalid = Definition {
            derivative_step_s: f64::NAN,
            ..Definition::default()
        };
        assert!(invalid.validate().is_err());
        let invalid = Definition {
            model: "unknown".into(),
            ..Definition::default()
        };
        assert!(invalid.validate().is_err());
    }
}
