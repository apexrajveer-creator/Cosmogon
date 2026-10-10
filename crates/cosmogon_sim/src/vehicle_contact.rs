//! Shared modeled terrain/foot response for the active physical-contact build.
//! Not yet an integrated landing/attachment capability of world vehicle flight.
use crate::{
    planet::terrain::{Terrain, METRES_PER_UNIT},
    vehicle_arrival::{KinematicState, SurfaceFrame},
    BodyRef, Universe, Vec3d,
};
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};
pub const MODEL: &str = "cosmogon-finite-terrain-contact/1";
fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub frame: SurfaceFrame,
    pub terrain_context: serde_json::Value,
    pub probe_distance_m: f64,
    pub radial_clearance_m: f64,
    /// First-order tangent-plane gap, not an exact closest-surface distance.
    pub tangent_gap_m: f64,
    pub position_m: Vec3d,
    pub velocity_m_s: Vec3d,
    pub outward_normal: Vec3d,
    pub slope_rad: f64,
}
impl Surface {
    pub fn sample(
        world: &Universe,
        target: BodyRef,
        time: f64,
        point: Vec3d,
        probe_distance_m: f64,
    ) -> Result<Self, String> {
        let frame = SurfaceFrame::at(world, target, time)?;
        if !finite(point)
            || point.length() > 1e15
            || !probe_distance_m.is_finite()
            || !(0.01..=1000.).contains(&probe_distance_m)
            || probe_distance_m > frame.radius_m * 0.01
        {
            return Err("Contact point/probe exceeds finite supported geometry".into());
        }
        let local = frame.rotate_to_fixed(point - frame.centre_m);
        let radius = local.length();
        if !radius.is_finite() || radius <= 0. {
            return Err("Contact direction undefined at body centre".into());
        }
        let up = local / radius;
        let terrain = Terrain::of(world.body(target));
        if terrain.data.is_some() && up.x * up.x + up.y * up.y <= 1e-20 {
            return Err(
                "Polar overview terrain has no unique supported meridian for contact normal".into(),
            );
        }
        let axis = if up.z.abs() < 0.8 {
            Vec3d::new(0., 0., 1.)
        } else {
            Vec3d::new(1., 0., 0.)
        };
        let east = axis.cross(up).normalize();
        let north = up.cross(east);
        let angle = probe_distance_m / frame.radius_m;
        let (sn, cs) = angle.dsin_cos();
        let surface = |dir: Vec3d| -> Result<Vec3d, String> {
            let height = terrain.elevation([dir.x, dir.y, dir.z]) * METRES_PER_UNIT;
            let r = frame.radius_m + height;
            if !height.is_finite()
                || !r.is_finite()
                || r <= 0.
                || height.abs() > frame.radius_m * 0.1
            {
                return Err("Modeled terrain relief exceeds supported contact shape".into());
            }
            Ok(dir * r)
        };
        let centre = surface(up)?;
        let tangent_east = surface(up * cs + east * sn)? - surface(up * cs - east * sn)?;
        let tangent_north = surface(up * cs + north * sn)? - surface(up * cs - north * sn)?;
        let mut normal = tangent_east.cross(tangent_north);
        if !finite(normal) || normal.length_squared() <= 0. {
            return Err("Degenerate modeled terrain tangent".into());
        }
        normal = normal.normalize();
        if normal.dot(up) < 0. {
            normal *= -1.;
        }
        let projected = normal.dot(up);
        if !projected.is_finite() || projected < 0.01 {
            return Err("Contact tangent is outside the supported height-field regime".into());
        }
        let state = frame.to_inertial(&KinematicState {
            position_m: centre,
            velocity_m_s: Vec3d::ZERO,
        })?;
        let gap = radius - centre.length();
        let context = match terrain.data {
            Some(data) => {
                if let Some(dataset) = data.mola() {
                    serde_json::json!({"kind":"observed-derived-overview","dataset":dataset.id(),"datum":crate::planet::mola::DATUM,"registration":"Areoid relief on modeled spherical base; no calibrated local hazard or areoid-shape registration"})
                } else {
                    serde_json::json!({"kind":"observed-derived-plus-procedural-detail","dataset":"embedded-earth-etopo5-overview","terrain_seed":terrain.seed})
                }
            }
            None => serde_json::json!({"kind":"procedural","terrain_seed":terrain.seed}),
        };
        let outward = frame.rotate_to_inertial(normal);
        Ok(Self {
            frame,
            terrain_context: context,
            probe_distance_m,
            radial_clearance_m: gap,
            tangent_gap_m: gap * projected,
            position_m: state.position_m,
            velocity_m_s: state.velocity_m_s,
            outward_normal: outward,
            slope_rad: projected.clamp(-1., 1.).dacos(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Leg {
    pub stiffness_n_m: f64,
    pub damping_n_s_m: f64,
    pub maximum_stroke_m: f64,
    pub maximum_load_n: f64,
    pub sliding_friction: f64,
    pub slip_speed_scale_m_s: f64,
}
impl Leg {
    pub fn validate(&self) -> Result<(), String> {
        if [
            self.stiffness_n_m,
            self.damping_n_s_m,
            self.maximum_stroke_m,
            self.maximum_load_n,
            self.sliding_friction,
            self.slip_speed_scale_m_s,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || !(1.0..=1e12).contains(&self.stiffness_n_m)
            || !(0.0..=1e12).contains(&self.damping_n_s_m)
            || !(0.001..=100.).contains(&self.maximum_stroke_m)
            || !(1.0..=1e12).contains(&self.maximum_load_n)
            || !(0.0..=10.).contains(&self.sliding_friction)
            || !(1e-6..=100.).contains(&self.slip_speed_scale_m_s)
        {
            return Err(
                "Foot response requires finite supported force/stroke/friction parameters".into(),
            );
        }
        Ok(())
    }
    pub fn response(
        &self,
        gap_m: f64,
        normal: Vec3d,
        relative_velocity: Vec3d,
    ) -> Result<Response, String> {
        self.validate()?;
        if !gap_m.is_finite()
            || !finite(normal)
            || (normal.length_squared() - 1.).abs() > 1e-10
            || !finite(relative_velocity)
            || relative_velocity.length() > 1e6
        {
            return Err("Foot response lacks supported gap/unit normal/relative velocity".into());
        }
        if gap_m >= 0. {
            return Ok(Response {
                compression_m: 0.,
                compression_rate_m_s: 0.,
                force_n: Vec3d::ZERO,
                normal_load_n: 0.,
                stored_elastic_energy_j: 0.,
                normal_dissipation_w: 0.,
                sliding_dissipation_w: 0.,
            });
        }
        let compression = -gap_m;
        if compression > self.maximum_stroke_m {
            return Err("Finite foot stroke exhausted".into());
        }
        let rate = -relative_velocity.dot(normal);
        let spring = self.stiffness_n_m * compression;
        let load = (spring + self.damping_n_s_m * rate).max(0.);
        if load > self.maximum_load_n {
            return Err("Foot normal load exceeds authored structural limit".into());
        }
        let tangent = relative_velocity - normal * relative_velocity.dot(normal);
        let friction = tangent
            * (-self.sliding_friction * load / (tangent.length() + self.slip_speed_scale_m_s));
        // With release force clipped to zero, only the applied effective damping
        // dissipates energy. This identity retains spring energy released as well.
        let normal_dissipation = ((load - spring) * rate).max(0.);
        let sliding_dissipation = (-friction.dot(tangent)).max(0.);
        let force = normal * load + friction;
        if force.length() > self.maximum_load_n {
            return Err("Foot resultant force exceeds authored structural limit".into());
        }
        let result = Response {
            compression_m: compression,
            compression_rate_m_s: rate,
            force_n: force,
            normal_load_n: load,
            stored_elastic_energy_j: 0.5 * self.stiffness_n_m * compression * compression,
            normal_dissipation_w: normal_dissipation,
            sliding_dissipation_w: sliding_dissipation,
        };
        if !finite(force)
            || !result.stored_elastic_energy_j.is_finite()
            || !normal_dissipation.is_finite()
            || !sliding_dissipation.is_finite()
        {
            return Err("Foot response exceeds finite numeric support".into());
        }
        Ok(result)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub compression_m: f64,
    pub compression_rate_m_s: f64,
    pub force_n: Vec3d,
    pub normal_load_n: f64,
    pub stored_elastic_energy_j: f64,
    pub normal_dissipation_w: f64,
    pub sliding_dissipation_w: f64,
}
/// Authoritative inputs for applying a gear force at its actual foot position.
/// Rigid-body integration and persistent attachment are the next coupling step.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyPose {
    pub centre_m: Vec3d,
    pub velocity_m_s: Vec3d,
    /// Hamilton xyzw; body axes to system-local inertial axes.
    pub orientation_xyzw: [f64; 4],
    pub angular_velocity_inertial_rad_s: Vec3d,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foot {
    pub offset_body_m: Vec3d,
    pub response: Leg,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FootSample {
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shear: Option<crate::vehicle_shear::State>,
    pub position_m: Vec3d,
    pub velocity_m_s: Vec3d,
    pub lever_inertial_m: Vec3d,
    pub surface: Surface,
    pub response: Response,
    pub torque_about_centre_n_m: Vec3d,
    pub centre_force_power_w: f64,
    pub torque_power_w: f64,
    pub ground_force_power_w: f64,
    pub elastic_storage_rate_w: f64,
    /// Actual instantaneous force/storage/dissipation identity, not a time-integrated ledger.
    pub power_residual_w: f64,
}
impl Foot {
    pub fn sample(
        &self,
        world: &Universe,
        target: BodyRef,
        time: f64,
        pose: &BodyPose,
        probe_distance_m: f64,
    ) -> Result<FootSample, String> {
        self.response.validate()?;
        let q = pose.orientation_xyzw;
        let norm = q.iter().map(|v| v * v).sum::<f64>();
        if q.iter().any(|v| !v.is_finite())
            || (norm - 1.).abs() > 1e-10
            || !finite(pose.centre_m)
            || pose.centre_m.length() > 1e15
            || !finite(pose.velocity_m_s)
            || pose.velocity_m_s.length() > 1e6
            || !finite(pose.angular_velocity_inertial_rad_s)
            || pose.angular_velocity_inertial_rad_s.length() > 1000.
            || !finite(self.offset_body_m)
            || self.offset_body_m.length() > 100.
        {
            return Err("Foot pose requires a finite unit orientation and supported position/velocity/offset".into());
        }
        // Rotation here is pure quaternion arithmetic. Dynamic increments must
        // be constructed with DMath when the coupled integrator is added.
        let rotation = cosmogon_core::math::Quatd::new(q[0], q[1], q[2], q[3]);
        let lever = rotation.rotate_vec3(self.offset_body_m);
        let position = pose.centre_m + lever;
        let velocity = pose.velocity_m_s + pose.angular_velocity_inertial_rad_s.cross(lever);
        let surface = Surface::sample(world, target, time, position, probe_distance_m)?;
        let response = self.response.response(
            surface.tangent_gap_m,
            surface.outward_normal,
            velocity - surface.velocity_m_s,
        )?;
        let torque = lever.cross(response.force_n);
        let centre_power = response.force_n.dot(pose.velocity_m_s);
        let torque_power = torque.dot(pose.angular_velocity_inertial_rad_s);
        let ground_power = response.force_n.dot(surface.velocity_m_s);
        let elastic_rate =
            self.response.stiffness_n_m * response.compression_m * response.compression_rate_m_s;
        let residual = centre_power + torque_power - ground_power
            + elastic_rate
            + response.normal_dissipation_w
            + response.sliding_dissipation_w;
        if !finite(position)
            || !finite(velocity)
            || !finite(torque)
            || [
                centre_power,
                torque_power,
                ground_power,
                elastic_rate,
                residual,
            ]
            .iter()
            .any(|v| !v.is_finite())
        {
            return Err("Foot torque/work exceeds finite numeric support".into());
        }
        Ok(FootSample {
            model: MODEL.into(),
            shear: None,
            position_m: position,
            velocity_m_s: velocity,
            lever_inertial_m: lever,
            surface,
            response,
            torque_about_centre_n_m: torque,
            centre_force_power_w: centre_power,
            torque_power_w: torque_power,
            ground_force_power_w: ground_power,
            elastic_storage_rate_w: elastic_rate,
            power_residual_w: residual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leg() -> Leg {
        Leg {
            stiffness_n_m: 1000.,
            damping_n_s_m: 100.,
            maximum_stroke_m: 1.,
            maximum_load_n: 1000.,
            sliding_friction: 0.5,
            slip_speed_scale_m_s: 0.1,
        }
    }
    #[test]
    fn finite_response_retains_applied_work_and_unilateral_release() {
        let n = Vec3d::new(0., 0., 1.);
        let l = leg();
        let v = Vec3d::new(3., 0., -1.);
        let r = l.response(-0.1, n, v).unwrap();
        assert_eq!(r.normal_load_n, 200.);
        assert_eq!(r.stored_elastic_energy_j, 5.);
        assert!(r.force_n.dot(n) >= 0.);
        assert!(r.force_n.x.abs() <= l.sliding_friction * r.normal_load_n);
        let spring_rate = l.stiffness_n_m * r.compression_m * r.compression_rate_m_s;
        assert!(
            (r.force_n.dot(v) + spring_rate + r.normal_dissipation_w + r.sliding_dissipation_w)
                .abs()
                < 1e-9
        );
        let release = l.response(-0.1, n, Vec3d::new(0., 0., 10.)).unwrap();
        assert_eq!(release.normal_load_n, 0.);
        assert_eq!(release.normal_dissipation_w, 1000.);
        assert_eq!(l.response(0.1, n, v).unwrap().force_n, Vec3d::ZERO);
        assert!(l.response(-2., n, v).is_err());
        assert!(l.response(-0.5, n, Vec3d::new(0., 0., -20.)).is_err());
    }
    #[test]
    fn moving_off_centre_foot_retains_torque_and_actual_power_identity() {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: u.systems[0].find_body("Mars").unwrap() as u32,
        };
        let frame = SurfaceFrame::at(&u, target, u.time).unwrap();
        let direction = crate::planet::terrain::dir_from_lat_lon(0.3, 1.2);
        let above = frame
            .to_inertial(&KinematicState {
                position_m: Vec3d::new(direction[0], direction[1], direction[2])
                    * (frame.radius_m + 10000.),
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let surface = Surface::sample(&u, target, u.time, above.position_m, 1.).unwrap();
        let n = surface.outward_normal;
        let pose = BodyPose {
            centre_m: surface.position_m + n * 0.1,
            velocity_m_s: surface.velocity_m_s + n * (-0.5) + Vec3d::new(0.3, 0.2, -0.1),
            orientation_xyzw: [0., 0., 0., 1.],
            angular_velocity_inertial_rad_s: Vec3d::new(0.1, -0.2, 0.3),
        };
        let foot = Foot {
            offset_body_m: n * (-0.2),
            response: leg(),
        };
        let reading = foot.sample(&u, target, u.time, &pose, 1.).unwrap();
        assert!(reading.response.compression_m > 0. && reading.response.compression_m < 0.2);
        assert!(reading.response.normal_load_n > 0.);
        assert!(reading.torque_about_centre_n_m.length() > 0.);
        let scale = reading.centre_force_power_w.abs() + reading.ground_force_power_w.abs() + 1.;
        assert!(reading.power_residual_w.abs() < scale * 1e-12);
        let invalid = BodyPose {
            orientation_xyzw: [0.; 4],
            ..pose
        };
        assert!(foot.sample(&u, target, u.time, &invalid, 1.).is_err());
    }
    #[test]
    fn actual_mola_surface_and_rotating_velocity_are_distinct_from_sphere() {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: u.systems[0].find_body("Mars").unwrap() as u32,
        };
        let f = SurfaceFrame::at(&u, target, u.time).unwrap();
        let dir = crate::planet::terrain::dir_from_lat_lon(0.3, 1.2);
        let local = Vec3d::new(dir[0], dir[1], dir[2]) * (f.radius_m + 10000.);
        let point = f
            .to_inertial(&KinematicState {
                position_m: local,
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let s = Surface::sample(&u, target, u.time, point.position_m, 1.).unwrap();
        assert!((s.outward_normal.length() - 1.).abs() < 1e-12);
        assert_eq!(s.terrain_context["kind"], "observed-derived-overview");
        let height = crate::planet::mola::Dataset::Four
            .read(0.3f64.to_degrees(), 1.2f64.to_degrees())
            .unwrap()
            .height_above_areoid_m;
        assert!(height.abs() > 1.);
        assert!((s.radial_clearance_m + height - 10000.).abs() < 0.001);
        assert!(s.radial_clearance_m > 0.);
        let fixed = f
            .to_fixed(&KinematicState {
                position_m: s.position_m,
                velocity_m_s: s.velocity_m_s,
            })
            .unwrap();
        assert!(fixed.velocity_m_s.length() < 1e-8);
        let on = Surface::sample(&u, target, u.time, s.position_m, 1.).unwrap();
        assert!(on.radial_clearance_m.abs() < 0.001);
        assert!(Surface::sample(&u, target, u.time, point.position_m, 0.).is_err());
    }
}
