//! Finite, history-dependent tangential compliance for the next contact capability.
//! Endpoint return mapping with separately reported numerical damping. Not yet
//! enabled in world flight; old smooth-sliding contact retains its exact behavior.
use crate::Vec3d;
use cosmogon_core::{dmath::DMath, math::Quatd};
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-finite-foot-shear/1";
fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}
fn unit(v: Vec3d) -> bool {
    finite(v) && (v.length_squared() - 1.).abs() <= 1e-10
}
fn tangent(v: Vec3d, n: Vec3d) -> bool {
    finite(v) && v.dot(n).abs() <= 1e-10 * (1. + v.length())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub stiffness_n_m: f64,
    pub friction_coefficient: f64,
    pub maximum_elastic_shear_m: f64,
    pub maximum_tangential_force_n: f64,
}
impl Definition {
    /// Transport actual terrain material orientation between two completed
    /// source-frame readings. Relative displacement is still supplied by the
    /// coupled integrator; source-centre movement must not become foot slip.
    pub fn advance_in_rotating_frames(
        &self,
        previous: &State,
        from: &crate::vehicle_arrival::SurfaceFrame,
        to: &crate::vehicle_arrival::SurfaceFrame,
        normal: Vec3d,
        displacement_m: Vec3d,
        normal_load_n: f64,
    ) -> Result<Increment, String> {
        let elapsed = to.time_s - from.time_s;
        if !elapsed.is_finite()
            || !(0.0..=0.020001).contains(&elapsed)
            || from.target != to.target
            || from.rotation_basis != to.rotation_basis
            || from.radius_m != to.radius_m
            || from.axial_tilt_rad != to.axial_tilt_rad
        {
            return Err("Shear material transport needs the same continuous target/frame and a resolved contact interval".into());
        }
        let a = frame_orientation(from)?;
        let b = frame_orientation(to)?;
        let inverse = Quatd::new(-a.x, -a.y, -a.z, a.w);
        let r = multiply(b, inverse);
        self.advance(
            previous,
            normal,
            [r.x, r.y, r.z, r.w],
            displacement_m,
            normal_load_n,
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL
            || !self.stiffness_n_m.is_finite()
            || !(1.0..=1e12).contains(&self.stiffness_n_m)
            || !self.friction_coefficient.is_finite()
            || !(0.0..=10.).contains(&self.friction_coefficient)
            || !self.maximum_elastic_shear_m.is_finite()
            || !(0.001..=100.).contains(&self.maximum_elastic_shear_m)
            || !self.maximum_tangential_force_n.is_finite()
            || !(1.0..=1e12).contains(&self.maximum_tangential_force_n)
        {
            return Err("Foot shear requires a known model and finite stiffness/friction/travel/force limits".into());
        }
        Ok(())
    }
    pub fn energy_j(&self, state: &State) -> Result<f64, String> {
        state.validate(self)?;
        Ok(0.5 * self.stiffness_n_m * state.elastic_shear_m.length_squared())
    }
    pub fn force_n(&self, state: &State) -> Result<Vec3d, String> {
        state.validate(self)?;
        Ok(state.elastic_shear_m * -self.stiffness_n_m)
    }

    /// The displacement is supplied in the new tangent plane, excluding motion
    /// before first contact. Rotation is actual material/contact transport, not
    /// vehicle attitude alone. No elapsed-time or zero-speed sticking shortcut.
    pub fn advance(
        &self,
        previous: &State,
        normal: Vec3d,
        material_rotation_xyzw: [f64; 4],
        displacement_m: Vec3d,
        normal_load_n: f64,
    ) -> Result<Increment, String> {
        previous.validate(self)?;
        let r = material_rotation_xyzw;
        if r.iter().any(|v| !v.is_finite())
            || (r.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 1e-10
            || !unit(normal)
            || !tangent(displacement_m, normal)
            || displacement_m.length() > 100.
            || !normal_load_n.is_finite()
            || !(0.0..=1e12).contains(&normal_load_n)
        {
            return Err("Foot shear increment needs unit rotation/normal, finite tangent displacement and supported load".into());
        }
        let rotation = Quatd::new(r[0], r[1], r[2], r[3]);
        let carried_normal = rotation.rotate_vec3(previous.outward_normal);
        let dot = carried_normal.dot(normal).clamp(-1., 1.);
        // cos(0.02 rad); a local normal turn must be resolved by the caller.
        if dot < 0.9998000066665778 {
            return Err(
                "Foot shear normal turns more than the supported 0.02 rad increment".into(),
            );
        }
        // Pure quaternion arithmetic avoids the general arc helper's identity
        // threshold and platform trigonometry for small/contact-normal turns.
        let axis = carried_normal.cross(normal);
        let length = (axis.length_squared() + (1. + dot) * (1. + dot)).sqrt();
        let align = Quatd::new(
            axis.x / length,
            axis.y / length,
            axis.z / length,
            (1. + dot) / length,
        );
        let rotated = align.rotate_vec3(rotation.rotate_vec3(previous.elastic_shear_m));
        let old_length = previous.elastic_shear_m.length();
        let projected = rotated - normal * rotated.dot(normal);
        // Preserve storage under objective transport; remove only accumulated
        // tangent roundoff, never physical strain or a resolved normal turn.
        let carried = if old_length == 0. {
            Vec3d::ZERO
        } else {
            if projected.length() <= 0. {
                return Err("Degenerate objective shear transport".into());
            }
            projected * (old_length / projected.length())
        };
        let capacity = self.friction_coefficient * normal_load_n;
        let radius = capacity / self.stiffness_n_m;
        let bounded = |v: Vec3d| {
            let length = v.length();
            if length > radius && length > 0. {
                v * (radius / length)
            } else {
                v
            }
        };
        let relaxed = bounded(carried);
        let trial = relaxed + displacement_m;
        let strain = bounded(trial);
        let state = State {
            model: MODEL.into(),
            outward_normal: normal,
            elastic_shear_m: strain,
            normal_load_n,
        };
        // Structural limits refuse the whole pure update. They do not act as
        // another friction cap or create an artificial safe result.
        state.validate(self)?;
        let plastic = trial - strain;
        let force = strain * -self.stiffness_n_m;
        let old_energy = 0.5 * self.stiffness_n_m * previous.elastic_shear_m.length_squared();
        let relaxed_energy = 0.5 * self.stiffness_n_m * relaxed.length_squared();
        let energy = 0.5 * self.stiffness_n_m * strain.length_squared();
        let unloading = (old_energy - relaxed_energy).max(0.);
        let plastic_dissipation = (self.stiffness_n_m * strain.dot(plastic)).max(0.);
        let numerical = 0.5 * self.stiffness_n_m * (strain - relaxed).length_squared();
        let work = force.dot(displacement_m);
        let change = energy - old_energy;
        let residual = work + change + unloading + plastic_dissipation + numerical;
        let increment = Increment {
            state,
            regime: if capacity == 0. {
                Regime::Free
            } else if plastic.length_squared() > 0. {
                Regime::Sliding
            } else {
                Regime::Elastic
            },
            tangential_force_n: force,
            plastic_slip_m: plastic,
            relative_foot_work_j: work,
            elastic_storage_change_j: change,
            plastic_dissipation_j: plastic_dissipation,
            unloading_release_j: unloading,
            numerical_dissipation_j: numerical,
            balance_residual_j: residual,
        };
        if [
            work,
            change,
            unloading,
            plastic_dissipation,
            numerical,
            residual,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || !finite(plastic)
            || residual.abs()
                > 1e-9
                    * (1.
                        + work.abs()
                        + old_energy
                        + energy
                        + unloading
                        + plastic_dissipation
                        + numerical)
        {
            return Err("Foot shear increment exceeds finite work/storage support".into());
        }
        Ok(increment)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub model: String,
    pub outward_normal: Vec3d,
    /// System-local inertial tangent vector; material rotation transports it.
    pub elastic_shear_m: Vec3d,
    pub normal_load_n: f64,
}
impl State {
    pub fn unloaded(normal: Vec3d) -> Result<Self, String> {
        if !unit(normal) {
            return Err("Unloaded shear state requires a unit terrain normal".into());
        }
        Ok(Self {
            model: MODEL.into(),
            outward_normal: normal,
            elastic_shear_m: Vec3d::ZERO,
            normal_load_n: 0.,
        })
    }
    pub fn validate(&self, d: &Definition) -> Result<(), String> {
        d.validate()?;
        let length = self.elastic_shear_m.length();
        let load = length * d.stiffness_n_m;
        if self.model != MODEL
            || !unit(self.outward_normal)
            || !tangent(self.elastic_shear_m, self.outward_normal)
            || !self.normal_load_n.is_finite()
            || !(0.0..=1e12).contains(&self.normal_load_n)
            || length > d.maximum_elastic_shear_m
            || !load.is_finite()
            || load > d.maximum_tangential_force_n
            || load > d.friction_coefficient * self.normal_load_n + 1e-10 * (1. + load)
            || d.friction_coefficient * self.normal_load_n == 0. && length != 0.
        {
            return Err("Retained foot shear violates model/tangent/load/finite travel or structural force limits".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Free,
    Elastic,
    Sliding,
}

fn multiply(a: Quatd, b: Quatd) -> Quatd {
    Quatd::new(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    )
}
fn frame_orientation(f: &crate::vehicle_arrival::SurfaceFrame) -> Result<Quatd, String> {
    if f.model != crate::vehicle_arrival::MODEL
        || f.frame != crate::vehicle_arrival::FRAME
        || !f.time_s.is_finite()
        || !f.axial_tilt_rad.is_finite()
        || f.axial_tilt_rad.abs() > std::f64::consts::PI
        || !f.spin_rad.is_finite()
        || f.spin_rad.abs() > std::f64::consts::TAU
        || !f.radius_m.is_finite()
        || f.radius_m <= 0.
    {
        return Err("Shear transport refuses unsupported source-frame orientation".into());
    }
    let (st, ct) = (f.axial_tilt_rad * 0.5).dsin_cos();
    let (ss, cs) = (f.spin_rad * 0.5).dsin_cos();
    Ok(multiply(
        Quatd::new(st, 0., 0., ct),
        Quatd::new(0., 0., ss, cs),
    ))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Increment {
    pub state: State,
    pub regime: Regime,
    pub tangential_force_n: Vec3d,
    pub plastic_slip_m: Vec3d,
    /// Endpoint force dotted with actual supported relative displacement.
    pub relative_foot_work_j: f64,
    pub elastic_storage_change_j: f64,
    pub plastic_dissipation_j: f64,
    /// Explicit irreversible release on reduced capacity/contact loss.
    pub unloading_release_j: f64,
    /// Endpoint constitutive integration loss, not measured material friction.
    pub numerical_dissipation_j: f64,
    pub balance_residual_j: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: [f64; 4] = [0., 0., 0., 1.];
    fn normal() -> Vec3d {
        Vec3d::new(0., 0., 1.)
    }
    fn definition() -> Definition {
        Definition {
            model: MODEL.into(),
            stiffness_n_m: 1000.,
            friction_coefficient: 0.5,
            maximum_elastic_shear_m: 0.5,
            maximum_tangential_force_n: 1000.,
        }
    }
    fn close(a: f64, b: f64) {
        assert!(
            (a - b).abs() < 1e-10 * (1. + a.abs() + b.abs()),
            "{a} != {b}"
        );
    }
    fn balanced(i: &Increment) {
        close(
            i.relative_foot_work_j
                + i.elastic_storage_change_j
                + i.plastic_dissipation_j
                + i.unloading_release_j
                + i.numerical_dissipation_j,
            i.balance_residual_j,
        );
        close(i.balance_residual_j, 0.);
        assert!(
            i.plastic_dissipation_j >= 0.
                && i.unloading_release_j >= 0.
                && i.numerical_dissipation_j >= 0.
        );
    }
    #[test]
    fn finite_static_load_persists_without_velocity_or_added_energy() {
        let d = definition();
        let empty = State::unloaded(normal()).unwrap();
        let loaded = d
            .advance(&empty, normal(), ID, Vec3d::new(0.01, 0., 0.), 100.)
            .unwrap();
        close(loaded.tangential_force_n.x, -10.);
        close(d.energy_j(&loaded.state).unwrap(), 0.05);
        balanced(&loaded);
        let held = d
            .advance(&loaded.state, normal(), ID, Vec3d::ZERO, 100.)
            .unwrap();
        assert_eq!(held.state, loaded.state);
        assert_eq!(held.tangential_force_n, loaded.tangential_force_n);
        assert_eq!(held.elastic_storage_change_j, 0.);
        assert_eq!(held.numerical_dissipation_j, 0.);
        balanced(&held);
    }

    #[test]
    fn initially_unloaded_supported_mass_reaches_elastic_equilibrium_without_creep() {
        let d = definition();
        let mut state = State::unloaded(normal()).unwrap();
        let mass = 1.;
        let external = 10.;
        let dt = 0.01;
        let mut velocity = 0.;
        let mut position = 0.;
        let mut numerical = 0.;
        let mut plastic = 0.;
        for _ in 0..400 {
            // Independent implicit 1D mass/spring solution, below the 50 N
            // friction cap. This is a component verification, not world flight.
            let next_velocity = (velocity
                + dt * (external - d.stiffness_n_m * state.elastic_shear_m.x) / mass)
                / (1. + d.stiffness_n_m * dt * dt / mass);
            let displacement = next_velocity * dt;
            let i = d
                .advance(&state, normal(), ID, Vec3d::new(displacement, 0., 0.), 100.)
                .unwrap();
            close(
                mass * (next_velocity - velocity) / dt,
                external + i.tangential_force_n.x,
            );
            balanced(&i);
            position += displacement;
            velocity = next_velocity;
            numerical += i.numerical_dissipation_j;
            plastic += i.plastic_dissipation_j;
            state = i.state;
        }
        assert!((position - external / d.stiffness_n_m).abs() < 1e-8);
        assert!(velocity.abs() < 1e-7);
        assert!((d.force_n(&state).unwrap().x + external).abs() < 1e-5);
        close(plastic, 0.);
        assert!(numerical > 0.);
        let held = d.advance(&state, normal(), ID, Vec3d::ZERO, 100.).unwrap();
        assert_eq!(held.state, state);
        close(held.relative_foot_work_j, 0.);
        close(held.elastic_storage_change_j, 0.);
    }
    #[test]
    fn sliding_reversal_unloading_and_separation_keep_distinct_work_terms() {
        let d = definition();
        let empty = State::unloaded(normal()).unwrap();
        let slip = d
            .advance(&empty, normal(), ID, Vec3d::new(0.1, 0., 0.), 100.)
            .unwrap();
        assert_eq!(slip.regime, Regime::Sliding);
        close(slip.tangential_force_n.x, -50.);
        close(slip.plastic_slip_m.x, 0.05);
        close(slip.plastic_dissipation_j, 2.5);
        close(slip.numerical_dissipation_j, 1.25);
        balanced(&slip);
        let reverse = d
            .advance(&slip.state, normal(), ID, Vec3d::new(-0.025, 0., 0.), 100.)
            .unwrap();
        assert_eq!(reverse.regime, Regime::Elastic);
        assert!(reverse.relative_foot_work_j > 0.);
        close(reverse.plastic_dissipation_j, 0.);
        balanced(&reverse);
        let unload = d
            .advance(&slip.state, normal(), ID, Vec3d::ZERO, 50.)
            .unwrap();
        close(unload.unloading_release_j, 0.9375);
        close(unload.numerical_dissipation_j, 0.);
        close(unload.relative_foot_work_j, 0.);
        balanced(&unload);
        let separated = d
            .advance(&unload.state, normal(), ID, Vec3d::ZERO, 0.)
            .unwrap();
        assert_eq!(separated.regime, Regime::Free);
        assert_eq!(separated.state.elastic_shear_m, Vec3d::ZERO);
        close(separated.unloading_release_j, 0.3125);
        balanced(&separated);
        let fresh = d
            .advance(&separated.state, normal(), ID, Vec3d::ZERO, 100.)
            .unwrap();
        assert_eq!(fresh.tangential_force_n, Vec3d::ZERO);
        balanced(&fresh);
    }
    #[test]
    fn material_rotation_and_small_normal_turn_preserve_objective_storage() {
        let d = definition();
        let initial = d
            .advance(
                &State::unloaded(normal()).unwrap(),
                normal(),
                ID,
                Vec3d::new(0.01, 0.02, 0.),
                100.,
            )
            .unwrap();
        // 90 degrees about the ground normal: a common material rotation, not slip.
        let s = 0.5_f64.sqrt();
        let rot = [0., 0., s, s];
        let q = Quatd::new(rot[0], rot[1], rot[2], rot[3]);
        let carried = d
            .advance(&initial.state, normal(), rot, Vec3d::ZERO, 100.)
            .unwrap();
        close(
            (carried.tangential_force_n - q.rotate_vec3(initial.tangential_force_n)).length(),
            0.,
        );
        close(
            d.energy_j(&carried.state).unwrap(),
            d.energy_j(&initial.state).unwrap(),
        );
        balanced(&carried);
        let turned_normal = Vec3d::new(0.005, 0., (1. - 0.005_f64 * 0.005).sqrt());
        let turned = d
            .advance(&carried.state, turned_normal, ID, Vec3d::ZERO, 100.)
            .unwrap();
        close(turned.state.elastic_shear_m.dot(turned_normal), 0.);
        close(
            d.energy_j(&turned.state).unwrap(),
            d.energy_j(&carried.state).unwrap(),
        );
        balanced(&turned);
    }
    #[test]
    fn numerical_dissipation_refines_separately_from_actual_plastic_slip() {
        let d = definition();
        let run = |steps| {
            let mut state = State::unloaded(normal()).unwrap();
            let mut numerical = 0.;
            let mut plastic = 0.;
            let mut work = 0.;
            for _ in 0..steps {
                let i = d
                    .advance(
                        &state,
                        normal(),
                        ID,
                        Vec3d::new(0.1 / steps as f64, 0., 0.),
                        100.,
                    )
                    .unwrap();
                balanced(&i);
                numerical += i.numerical_dissipation_j;
                plastic += i.plastic_dissipation_j;
                work += i.relative_foot_work_j;
                state = i.state;
            }
            (state, numerical, plastic, work)
        };
        let coarse = run(10);
        let fine = run(100);
        let finer = run(1000);
        close(coarse.0.elastic_shear_m.x, 0.05);
        close(fine.0.elastic_shear_m.x, 0.05);
        close(finer.0.elastic_shear_m.x, 0.05);
        close(coarse.2, 2.5);
        close(fine.2, 2.5);
        close(finer.2, 2.5);
        assert!(fine.1 < coarse.1 * 0.11 && finer.1 < fine.1 * 0.11);
        // Exact 1D continuum spring work is 1.25 J + 2.5 J slip.
        assert!((fine.3 + 3.75).abs() < (coarse.3 + 3.75).abs());
        assert!((finer.3 + 3.75).abs() < (fine.3 + 3.75).abs());
    }
    #[test]
    fn changing_slip_direction_obeys_friction_and_retains_energy_identity() {
        let d = definition();
        let mut state = State::unloaded(normal()).unwrap();
        for delta in [
            Vec3d::new(0.1, 0., 0.),
            Vec3d::new(0., 0.1, 0.),
            Vec3d::new(-0.1, 0., 0.),
            Vec3d::new(0., -0.1, 0.),
        ] {
            let i = d.advance(&state, normal(), ID, delta, 100.).unwrap();
            balanced(&i);
            assert!(i.tangential_force_n.length() <= 50. + 1e-10);
            assert!(i.plastic_dissipation_j > 0.);
            state = i.state;
        }
        let encoded = serde_json::to_string(&state).unwrap();
        let loaded: State = serde_json::from_str(&encoded).unwrap();
        assert_eq!(loaded, state);
        loaded.validate(&d).unwrap();
    }
    #[test]
    fn invalid_history_geometry_and_structural_limits_are_atomic_refusals() {
        let d = definition();
        let state = State::unloaded(normal()).unwrap();
        let frozen = state.clone();
        for (n, r, delta, load) in [
            (Vec3d::ZERO, ID, Vec3d::ZERO, 100.),
            (normal(), [0., 0., 0., 0.], Vec3d::ZERO, 100.),
            (normal(), ID, normal(), 100.),
            (normal(), ID, Vec3d::ZERO, -1.),
            (Vec3d::new(1., 0., 0.), ID, Vec3d::ZERO, 100.),
        ] {
            assert!(d.advance(&state, n, r, delta, load).is_err());
            assert_eq!(state, frozen);
        }
        let limited = Definition {
            maximum_tangential_force_n: 1.,
            ..d.clone()
        };
        assert!(limited
            .advance(&state, normal(), ID, Vec3d::new(0.01, 0., 0.), 100.)
            .is_err());
        let travel = Definition {
            maximum_elastic_shear_m: 0.001,
            ..d.clone()
        };
        assert!(travel
            .advance(&state, normal(), ID, Vec3d::new(0.01, 0., 0.), 100.)
            .is_err());
        let forged = State {
            elastic_shear_m: Vec3d::new(0.01, 0., 0.),
            ..state.clone()
        };
        assert!(forged.validate(&d).is_err());
        let unsupported = Definition {
            model: "future-model".into(),
            ..d
        };
        assert!(state.validate(&unsupported).is_err());
        let mut json = serde_json::to_value(&state).unwrap();
        json["attached"] = serde_json::json!(true);
        assert!(serde_json::from_value::<State>(json).is_err());
    }

    #[test]
    fn actual_tilted_mola_frame_transports_static_shear_without_ephemeris_slip() {
        use crate::{
            vehicle_arrival::{KinematicState, SurfaceFrame},
            vehicle_contact::Surface,
            BodyRef, Scenario, Universe, UniverseSettings,
        };
        let world = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let from = SurfaceFrame::at(&world, target, world.time).unwrap();
        let fixed = Vec3d::new(0.8, 0.6, 0.) * (from.radius_m + 10000.);
        let start = from
            .to_inertial(&KinematicState {
                position_m: fixed,
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let before = Surface::sample(&world, target, from.time_s, start.position_m, 1.).unwrap();
        let to = SurfaceFrame::at(&world, target, world.time + 0.005).unwrap();
        let endpoint = to
            .to_inertial(&KinematicState {
                position_m: fixed,
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let after = Surface::sample(&world, target, to.time_s, endpoint.position_m, 1.).unwrap();
        let tangent = before
            .outward_normal
            .cross(Vec3d::new(0., 0., 1.))
            .normalize();
        let d = definition();
        let loaded = d
            .advance(
                &State::unloaded(before.outward_normal).unwrap(),
                before.outward_normal,
                ID,
                tangent * 0.01,
                100.,
            )
            .unwrap();
        let transported = d
            .advance_in_rotating_frames(
                &loaded.state,
                &from,
                &to,
                after.outward_normal,
                Vec3d::ZERO,
                100.,
            )
            .unwrap();
        let expected = to.rotate_to_inertial(from.rotate_to_fixed(loaded.tangential_force_n));
        assert!((transported.tangential_force_n - expected).length() < 1e-7);
        close(
            d.energy_j(&transported.state).unwrap(),
            d.energy_j(&loaded.state).unwrap(),
        );
        balanced(&transported);
        // Geometry/source mismatches are refusals rather than resetting memory.
        let other = SurfaceFrame {
            target: BodyRef {
                system: 0,
                body: target.body + 1,
            },
            ..to.clone()
        };
        assert!(d
            .advance_in_rotating_frames(
                &loaded.state,
                &from,
                &other,
                after.outward_normal,
                Vec3d::ZERO,
                100.
            )
            .is_err());
        let late = SurfaceFrame::at(&world, target, world.time + 1.).unwrap();
        assert!(d
            .advance_in_rotating_frames(
                &loaded.state,
                &from,
                &late,
                after.outward_normal,
                Vec3d::ZERO,
                100.
            )
            .is_err());
    }
}
