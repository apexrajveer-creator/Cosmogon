//! Rotating surface frames and geographic initial conditions for finite spacecraft.
//! These observations do not change spherical contact into a landing or grant cargo.
use crate::{
    astro::StarSystem,
    planet::terrain::{Terrain, METRES_PER_UNIT},
    supply_vehicle::Plan,
    world_vehicle::{Definition, Flight, Phase, Source},
    BodyRef, Universe, Vec3d,
};
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-rotating-vehicle-arrival/1";
pub const FRAME: &str = "modelled-planetocentric-body-fixed-tilt-spin";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KinematicState {
    pub position_m: Vec3d,
    pub velocity_m_s: Vec3d,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceFrame {
    pub model: String,
    pub frame: String,
    pub target: BodyRef,
    pub time_s: f64,
    pub centre_m: Vec3d,
    /// Derivative of the existing modeled source position, not a new ephemeris.
    pub centre_velocity_m_s: Vec3d,
    pub radius_m: f64,
    pub axial_tilt_rad: f64,
    pub spin_rad: f64,
    pub spin_rate_rad_s: f64,
    pub angular_velocity_inertial_rad_s: Vec3d,
    pub rotation_basis: String,
}
fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}
fn bounded(v: Vec3d, bound: f64) -> bool {
    finite(v) && v.length() <= bound
}

/// Differentiate the source-position operator. On-rails initialization velocities
/// may use a different mass parameter; never substitute them for this derivative.
fn translation(sys: &StarSystem, body: usize, time: f64) -> (Vec3d, Vec3d) {
    if let Some(d) = &sys.dynamics {
        if d.bodies.get(body).is_some_and(|b| b.is_some()) {
            let state = sys.body_state(body, time);
            return (state.pos, state.vel);
        }
    }
    let (p, v) = match sys.bodies[body].parent {
        Some(parent) => translation(sys, parent as usize, time),
        None => {
            let vel = sys
                .dynamics
                .as_ref()
                .map(|d| {
                    d.star.vel + d.acc.first().copied().unwrap_or(Vec3d::ZERO) * (time - d.time())
                })
                .unwrap_or(Vec3d::ZERO);
            (sys.star_local_position(time), vel)
        }
    };
    let mu = sys.parent_mu(body);
    let orbit = &sys.bodies[body].orbit;
    (
        p + orbit.position(mu, time),
        v + orbit.state(mu, mu, time).1,
    )
}
impl SurfaceFrame {
    pub fn at(world: &Universe, target: BodyRef, time: f64) -> Result<Self, String> {
        if !time.is_finite() {
            return Err("Surface frame requires a finite clock".into());
        }
        let sys = world
            .systems
            .get(target.system as usize)
            .ok_or("Missing arrival system")?;
        let i = target.body as usize;
        let body = sys.bodies.get(i).ok_or("Missing arrival body")?;
        if !body.exists()
            || !body.kind.has_surface()
            || !body.radius.is_finite()
            || !(0.0..=1e15).contains(&body.radius)
            || body.radius == 0.
            || !body.axial_tilt.is_finite()
        {
            return Err(
                "Arrival frame requires an existing solid body with supported finite radius/tilt"
                    .into(),
            );
        }
        // Validate the complete parent chain before calling recursive source APIs.
        let mut cursor = Some(i);
        let mut seen = Vec::new();
        while let Some(j) = cursor {
            if seen.contains(&j) {
                return Err("Cyclic arrival parent chain".into());
            }
            seen.push(j);
            let b = sys.bodies.get(j).ok_or("Invalid arrival parent chain")?;
            let active = sys
                .dynamics
                .as_ref()
                .is_some_and(|d| d.bodies.get(j).is_some_and(|v| v.is_some()));
            if !active {
                let mu = match b.parent {
                    Some(p) => sys
                        .bodies
                        .get(p as usize)
                        .ok_or("Invalid arrival parent")?
                        .mu(),
                    None => sys.star.mu(),
                };
                if !mu.is_finite()
                    || mu <= 0.
                    || !b.orbit.a.is_finite()
                    || b.orbit.a <= 0.
                    || !b.orbit.e.is_finite()
                    || !(0.0..1.0).contains(&b.orbit.e)
                    || [b.orbit.i, b.orbit.node, b.orbit.peri, b.orbit.m0]
                        .iter()
                        .any(|v| !v.is_finite())
                {
                    return Err("Arrival rail lacks a supported finite bound orbit".into());
                }
            }
            cursor = b.parent.map(|p| p as usize);
        }
        let (centre, velocity) = translation(sys, i, time);
        if !bounded(centre, 1e15) || !bounded(velocity, 1e6) {
            return Err("Arrival source exceeds supported system-local position/velocity".into());
        }
        let (rate, basis) = if body.tidally_locked {
            let (parent, parent_velocity) = match body.parent {
                Some(p) => translation(sys, p as usize, time),
                None => (
                    sys.star_local_position(time),
                    sys.dynamics
                        .as_ref()
                        .map(|d| {
                            d.star.vel
                                + d.acc.first().copied().unwrap_or(Vec3d::ZERO) * (time - d.time())
                        })
                        .unwrap_or(Vec3d::ZERO),
                ),
            };
            let r = centre - parent;
            let v = velocity - parent_velocity;
            let projected = r.x * r.x + r.y * r.y;
            if projected <= r.length_squared() * 1e-20 || !projected.is_finite() {
                return Err("Parent-facing spin has undefined projected azimuth".into());
            }
            (
                (r.x * v.y - r.y * v.x) / projected,
                "Derivative of modeled parent-facing azimuth",
            )
        } else {
            let period = body.rotation_period;
            if !period.is_finite()
                || period.abs() < 0.001
                || period.abs() > 1e15
                || (time / period).abs() > 2f64.powi(48)
            {
                return Err("Stored spin period/clock lacks supported phase precision".into());
            }
            (
                std::f64::consts::TAU / period,
                "Stored constant spin period",
            )
        };
        let spin = sys.spin_angle(i, time);
        let (st, ct) = body.axial_tilt.dsin_cos();
        let angular = Vec3d::new(0., -st * rate, ct * rate);
        if !rate.is_finite() || !spin.is_finite() || !finite(angular) {
            return Err("Arrival rotation is not finite".into());
        }
        Ok(Self {
            model: MODEL.into(),
            frame: FRAME.into(),
            target,
            time_s: time,
            centre_m: centre,
            centre_velocity_m_s: velocity,
            radius_m: body.radius,
            axial_tilt_rad: body.axial_tilt,
            spin_rad: spin,
            spin_rate_rad_s: rate,
            angular_velocity_inertial_rad_s: angular,
            rotation_basis: basis.into(),
        })
    }
    pub fn rotate_to_fixed(&self, v: Vec3d) -> Vec3d {
        let (st, ct) = self.axial_tilt_rad.dsin_cos();
        let (ss, cs) = self.spin_rad.dsin_cos();
        let p = Vec3d::new(v.x, v.y * ct + v.z * st, -v.y * st + v.z * ct);
        Vec3d::new(p.x * cs + p.y * ss, -p.x * ss + p.y * cs, p.z)
    }
    pub fn rotate_to_inertial(&self, v: Vec3d) -> Vec3d {
        let (st, ct) = self.axial_tilt_rad.dsin_cos();
        let (ss, cs) = self.spin_rad.dsin_cos();
        let p = Vec3d::new(v.x * cs - v.y * ss, v.x * ss + v.y * cs, v.z);
        Vec3d::new(p.x, p.y * ct - p.z * st, p.y * st + p.z * ct)
    }
    pub fn to_fixed(&self, state: &KinematicState) -> Result<KinematicState, String> {
        if !bounded(state.position_m, 1e15) || !bounded(state.velocity_m_s, 1e6) {
            return Err("Arrival state exceeds supported flight bounds".into());
        }
        let r = state.position_m - self.centre_m;
        let v = state.velocity_m_s
            - self.centre_velocity_m_s
            - self.angular_velocity_inertial_rad_s.cross(r);
        let result = KinematicState {
            position_m: self.rotate_to_fixed(r),
            velocity_m_s: self.rotate_to_fixed(v),
        };
        if !bounded(result.position_m, 1e15) || !bounded(result.velocity_m_s, 1e6) {
            return Err("Surface-relative state exceeds supported bounds".into());
        }
        Ok(result)
    }
    pub fn to_inertial(&self, state: &KinematicState) -> Result<KinematicState, String> {
        if !bounded(state.position_m, 1e15) || !bounded(state.velocity_m_s, 1e6) {
            return Err("Geographic state exceeds supported finite bounds".into());
        }
        let r = self.rotate_to_inertial(state.position_m);
        let result = KinematicState {
            position_m: self.centre_m + r,
            velocity_m_s: self.centre_velocity_m_s
                + self.rotate_to_inertial(state.velocity_m_s)
                + self.angular_velocity_inertial_rad_s.cross(r),
        };
        if !bounded(result.position_m, 1e15) || !bounded(result.velocity_m_s, 1e6) {
            return Err("Geographic start exceeds supported system-local bounds".into());
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeographicStart {
    pub model: String,
    pub latitude_deg: f64,
    pub longitude_east_deg: f64,
    /// Above the flight model's spherical radius, NOT above terrain/areoid.
    pub spherical_altitude_m: f64,
    pub east_velocity_m_s: f64,
    pub north_velocity_m_s: f64,
    pub up_velocity_m_s: f64,
}
impl Default for GeographicStart {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            latitude_deg: 0.,
            longitude_east_deg: 0.,
            spherical_altitude_m: 10_000.,
            east_velocity_m_s: 0.,
            north_velocity_m_s: 0.,
            up_velocity_m_s: -10.,
        }
    }
}
impl GeographicStart {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err("Unsupported geographic arrival model".into());
        }
        crate::planet::mola::validate_coordinates(self.latitude_deg, self.longitude_east_deg)?;
        if !self.spherical_altitude_m.is_finite()
            || !(0.001..=1e12).contains(&self.spherical_altitude_m)
            || [
                self.east_velocity_m_s,
                self.north_velocity_m_s,
                self.up_velocity_m_s,
            ]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1e6)
        {
            return Err(
                "Geographic altitude/velocities exceed their finite supported bounds".into(),
            );
        }
        Ok(())
    }
    pub fn flight(
        &self,
        world: &Universe,
        target: BodyRef,
        plan: Plan,
    ) -> Result<Definition, String> {
        self.validate()?;
        let frame = SurfaceFrame::at(world, target, world.time)?;
        let lat = self.latitude_deg.to_radians();
        let lon = self.longitude_east_deg.to_radians();
        let (up, east, north) = enu(lat, lon);
        let absolute = frame.to_inertial(&KinematicState {
            position_m: up * (frame.radius_m + self.spherical_altitude_m),
            velocity_m_s: east * self.east_velocity_m_s
                + north * self.north_velocity_m_s
                + up * self.up_velocity_m_s,
        })?;
        let centre =
            world.systems[target.system as usize].body_state(target.body as usize, world.time);
        let result = Definition {
            model: crate::world_vehicle::MODEL.into(),
            frame: crate::world_vehicle::FRAME.into(),
            parent: target,
            plan,
            initial_relative_position_m: absolute.position_m - centre.pos,
            initial_relative_velocity_m_s: absolute.velocity_m_s - centre.vel,
        };
        result.validate(world)?;
        // The ordinary creation path also checks other contact sources.
        Flight::new(world.next_vehicle_id, result.clone(), world)?;
        Ok(result)
    }
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let start: Self = serde_json::from_str(&crate::habitat_recording::read(path, 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        start.validate()?;
        Ok(start)
    }
}
fn enu(lat: f64, lon: f64) -> (Vec3d, Vec3d, Vec3d) {
    let (sl, cl) = lat.dsin_cos();
    let (so, co) = lon.dsin_cos();
    (
        Vec3d::new(cl * co, cl * so, sl),
        Vec3d::new(-so, co, 0.),
        Vec3d::new(-sl * co, -sl * so, cl),
    )
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approach {
    pub model: String,
    pub frame: SurfaceFrame,
    pub world_time_s: f64,
    pub craft_time_s: f64,
    pub vehicle_id: u64,
    pub body_fixed_state: KinematicState,
    pub latitude_deg: f64,
    pub longitude_east_deg: Option<f64>,
    pub spherical_altitude_m: f64,
    pub radial_velocity_m_s: f64,
    pub east_velocity_m_s: Option<f64>,
    pub north_velocity_m_s: Option<f64>,
    pub horizontal_speed_m_s: f64,
    pub surface_relative_speed_m_s: f64,
    pub current_mass_kg: f64,
    pub radial_kinetic_energy_j: f64,
    pub horizontal_kinetic_energy_j: f64,
    pub terrain_height_above_sphere_m: Option<f64>,
    pub terrain_clearance_m: Option<f64>,
    pub terrain_context: serde_json::Value,
}
impl Approach {
    /// Live observations only accept active flight. Terminal orientation/velocity
    /// must be observed while replaying the original branch, never at a later clock.
    pub fn live(world: &Universe, flight: &Flight, target: BodyRef) -> Result<Self, String> {
        if flight.phase != Phase::Flying {
            return Err("Live terminal orientation was not retained; use a replayed approach report for historical contact evidence".into());
        }
        if world.time - flight.last_time_s > 1.000001 {
            return Err("Craft clock is behind the supported source frame interval".into());
        }
        if world
            .edits
            .iter()
            .skip(flight.edit_cursor as usize)
            .any(|record| {
                record.edit.system() == target.system
                    && matches!(
                        &record.edit,
                        crate::sandbox::Edit::SetProperty { .. }
                            | crate::sandbox::Edit::SetMarsTerrain { .. }
                            | crate::sandbox::Edit::SetState { .. }
                            | crate::sandbox::Edit::SetOrbit { .. }
                            | crate::sandbox::Edit::SetStar { .. }
                            | crate::sandbox::Edit::SetPhysics { .. }
                            | crate::sandbox::Edit::Impulse { .. }
                            | crate::sandbox::Edit::RemoveBody { .. }
                            | crate::sandbox::Edit::AddBody { .. }
                    )
            })
        {
            return Err("Source/frame edit occurred since retained craft state; resolve a new flight task before live approach observation".into());
        }
        Self::observe(world, flight, target)
    }
    pub(crate) fn observe(
        world: &Universe,
        flight: &Flight,
        target: BodyRef,
    ) -> Result<Self, String> {
        if target.system != flight.definition.parent.system {
            return Err("Arrival target and craft must share the same system-local frame".into());
        }
        if !world.time.is_finite() || world.time < flight.last_time_s {
            return Err("Arrival world clock precedes its craft state".into());
        }
        let mut frame = SurfaceFrame::at(world, target, flight.last_time_s)?;
        let source = flight
            .last_sources
            .iter()
            .find(|s| s.source == Source::Body(target.body))
            .ok_or("Arrival target absent from retained gravity sources")?;
        let body = world.body(target);
        if source.radius_m != body.radius || source.mu_m3_s2 != body.mu() {
            return Err("Arrival target radius/mass differs from retained craft source".into());
        }
        // Contact centres were interpolated during the finite source substep; retain
        // that actual centre. The velocity is the derivative of the existing source
        // ephemeris, separately identified in context, not a recovered chord slope.
        frame.centre_m = source.position_m;
        let fixed = frame.to_fixed(&KinematicState {
            position_m: flight.position_m,
            velocity_m_s: flight.velocity_m_s,
        })?;
        let radius = fixed.position_m.length();
        if !radius.is_finite() || radius <= 0. {
            return Err("Arrival direction is undefined at the body centre".into());
        }
        let up = fixed.position_m / radius;
        let lat = up.z.datan2((up.x * up.x + up.y * up.y).sqrt());
        let polar = (up.x * up.x + up.y * up.y) <= 1e-20;
        let lon = (!polar).then(|| up.y.datan2(up.x));
        let radial = fixed.velocity_m_s.dot(up);
        let horizontal = (fixed.velocity_m_s - up * radial).length();
        let (east, north) = lon
            .map(|lon| {
                let (_, east, north) = enu(lat, lon);
                (
                    Some(fixed.velocity_m_s.dot(east)),
                    Some(fixed.velocity_m_s.dot(north)),
                )
            })
            .unwrap_or((None, None));
        let d = [up.x, up.y, up.z];
        let terrain = Terrain::of(body);
        let height = terrain.elevation(d) * METRES_PER_UNIT;
        let terrain_context = if polar {
            serde_json::json!({"kind":"unavailable","reason":"Longitude/tangent basis undefined at pole; no arbitrary terrain meridian is assigned"})
        } else {
            match terrain.data {
                Some(data) => {
                    if let Some(dataset) = data.mola() {
                        serde_json::json!({"kind":"observed-derived-overview","dataset":dataset.id(),"reading":dataset.read(lat.to_degrees(),lon.unwrap_or(0.).to_degrees())?,"datum":crate::planet::mola::DATUM,"registration":"Areoid height added to modeled spherical base; areoid shape and landing-site registration unverified"})
                    } else {
                        serde_json::json!({"kind":"observed-derived-plus-procedural-detail","dataset":"embedded-earth-etopo5-overview","terrain_seed":terrain.seed,"registration":"Embedded relief plus generated sub-grid detail; spherical base, not landing-site registration"})
                    }
                }
                None => {
                    serde_json::json!({"kind":"procedural","terrain_seed":terrain.seed,"registration":"Generated relief on modeled sphere; no observed local landing geometry"})
                }
            }
        };
        let mass =
            flight.definition.plan.definition.inert_mass_kg() + flight.propulsion.propellant_kg;
        let result = Self {
            model: MODEL.into(),
            frame,
            world_time_s: world.time,
            craft_time_s: flight.last_time_s,
            vehicle_id: flight.id,
            body_fixed_state: fixed,
            latitude_deg: lat.to_degrees(),
            longitude_east_deg: lon.map(|x| x.to_degrees().rem_euclid(360.)),
            spherical_altitude_m: radius - body.radius,
            radial_velocity_m_s: radial,
            east_velocity_m_s: east,
            north_velocity_m_s: north,
            horizontal_speed_m_s: horizontal,
            surface_relative_speed_m_s: (radial * radial + horizontal * horizontal).sqrt(),
            current_mass_kg: mass,
            radial_kinetic_energy_j: 0.5 * mass * radial * radial,
            horizontal_kinetic_energy_j: 0.5 * mass * horizontal * horizontal,
            terrain_height_above_sphere_m: (!polar).then_some(height),
            terrain_clearance_m: (!polar).then_some(radius - body.radius - height),
            terrain_context,
        };
        if !mass.is_finite()
            || mass <= 0.
            || !height.is_finite()
            || !result.radial_kinetic_energy_j.is_finite()
            || !result.horizontal_kinetic_energy_j.is_finite()
        {
            return Err("Arrival numeric evidence exceeds finite support".into());
        }
        Ok(result)
    }
}
pub fn context() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"frame":FRAME,"rotation":"Existing inverse tilt then spin; rotating velocity subtracts omega cross relative position","translation":"Derivative of existing sampled/analytic source position; on-rails anomaly parameter is retained","position":"Actual retained gravity-source centre at craft clock; spherical contact source centre may be linearly interpolated","velocity":"Derivative of modeled source ephemeris at craft clock; does not recover the source-contact chord velocity","coordinates":"Planetocentric north latitude, east longitude; east/north/longitude unavailable at pole","terrain":"Overview/procedural relief sampled separately; flight collision remains spherical. Terrain clearance is not hazard/landing clearance","kinetic_energy":"0.5 times actual finite mass times squared relative speed; no force, shock, attitude, gear, restitution, structural or safety model","limits":"No touchdown/attachment, descent controller, atmospheric entry, source calibration, uncertainty or cargo delivery","sources":["https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html","https://www.nasa.gov/safe-and-precise-landing-integrated-capabilities-evolution-splice/"],"empirical_validation":null,"uncertainty":null})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> Universe {
        Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        })
    }
    fn target(u: &Universe, name: &str) -> BodyRef {
        BodyRef {
            system: 0,
            body: u.systems[0].find_body(name).unwrap() as u32,
        }
    }
    fn close(a: Vec3d, b: Vec3d, tol: f64) {
        assert!((a - b).length() < tol, "{a:?} != {b:?}");
    }
    #[test]
    fn frame_inverse_and_terrain_orientation_match_with_tilt_and_retrograde_spin() {
        let mut u = world();
        let r = target(&u, "Mars");
        u.systems[0].bodies[r.body as usize].rotation_period = -86000.;
        let f = SurfaceFrame::at(&u, r, u.time).unwrap();
        let p = Vec3d::new(100., -200., 300.);
        close(f.rotate_to_inertial(f.rotate_to_fixed(p)), p, 1e-10);
        let old = u.systems[0].inertial_to_body_fixed(r.body as usize, p, u.time);
        close(
            f.rotate_to_fixed(p),
            Vec3d::new(old[0], old[1], old[2]),
            1e-12,
        );
        let s = KinematicState {
            position_m: Vec3d::new(f.radius_m + 10000., 0., 0.),
            velocity_m_s: Vec3d::new(-20., 35., -8.),
        };
        let absolute = f.to_inertial(&s).unwrap();
        let back = f.to_fixed(&absolute).unwrap();
        close(s.position_m, back.position_m, 0.001);
        close(s.velocity_m_s, back.velocity_m_s, 1e-8);
        assert!(f.spin_rate_rad_s < 0.);
    }
    #[test]
    fn stationary_surface_point_has_zero_rotating_velocity_and_correct_derivative() {
        let u = world();
        let r = target(&u, "Mars");
        let f = SurfaceFrame::at(&u, r, u.time).unwrap();
        let local = KinematicState {
            position_m: Vec3d::new(3e6, 1e6, 2e5),
            velocity_m_s: Vec3d::ZERO,
        };
        let a = f.to_inertial(&local).unwrap();
        close(f.to_fixed(&a).unwrap().velocity_m_s, Vec3d::ZERO, 1e-8);
        let before = SurfaceFrame::at(&u, r, u.time - 1.)
            .unwrap()
            .to_inertial(&local)
            .unwrap();
        let after = SurfaceFrame::at(&u, r, u.time + 1.)
            .unwrap()
            .to_inertial(&local)
            .unwrap();
        close(
            (after.position_m - before.position_m) / 2.,
            a.velocity_m_s,
            0.005,
        );
    }
    #[test]
    fn tidal_spin_derivative_and_geographic_moon_start_are_supported() {
        let u = world();
        let r = target(&u, "Moon");
        let f = SurfaceFrame::at(&u, r, u.time).unwrap();
        assert!(f.rotation_basis.contains("parent-facing"));
        let before = u.systems[0].spin_angle(r.body as usize, u.time - 1.);
        let after = u.systems[0].spin_angle(r.body as usize, u.time + 1.);
        let delta = (after - before + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        assert!((delta / 2. - f.spin_rate_rad_s).abs() < 1e-10);
        let mut start = GeographicStart::default();
        start.latitude_deg = 20.;
        start.longitude_east_deg = 130.;
        start.east_velocity_m_s = 12.;
        start.north_velocity_m_s = -5.;
        let craft = Flight::new(0, start.flight(&u, r, Plan::example()).unwrap(), &u).unwrap();
        let reading = Approach::live(&u, &craft, r).unwrap();
        assert!((reading.latitude_deg - 20.).abs() < 1e-7);
        assert!((reading.longitude_east_deg.unwrap() - 130.).abs() < 1e-7);
        assert!((reading.radial_velocity_m_s + 10.).abs() < 1e-7);
        assert!((reading.east_velocity_m_s.unwrap() - 12.).abs() < 1e-7);
        assert!((reading.north_velocity_m_s.unwrap() + 5.).abs() < 1e-7);
        assert!((reading.spherical_altitude_m - 10000.).abs() < 0.001);
    }
    #[test]
    fn poles_terrain_source_and_mass_energy_are_explicit_without_state_mutation() {
        let u = world();
        let r = target(&u, "Mars");
        let fingerprint = crate::research::state_fingerprint(&u).unwrap();
        let start = GeographicStart {
            latitude_deg: 90.,
            longitude_east_deg: 37.,
            ..Default::default()
        };
        let craft = Flight::new(0, start.flight(&u, r, Plan::example()).unwrap(), &u).unwrap();
        let reading = Approach::live(&u, &craft, r).unwrap();
        assert!(reading.longitude_east_deg.is_none());
        assert!(reading.east_velocity_m_s.is_none());
        assert!(reading.north_velocity_m_s.is_none());
        assert_eq!(reading.terrain_context["kind"], "unavailable");
        assert!(reading.terrain_height_above_sphere_m.is_none());
        assert!(reading.terrain_context["reading"]["uncertainty_m"].is_null());
        assert!((reading.radial_kinetic_energy_j - 0.5 * 2170. * 100.).abs() < 0.01);
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), fingerprint);
    }
    #[test]
    fn dynamic_rail_translation_differentiates_positions_and_excludes_galactic_origin() {
        let mut u = world();
        let r = target(&u, "Moon");
        let t = u.time;
        u.systems[0].activate_dynamics(t, Default::default());
        let f = SurfaceFrame::at(&u, r, t).unwrap();
        let sys = &u.systems[0];
        let delta = (sys.body_local_position(r.body as usize, t + 1.)
            - sys.body_local_position(r.body as usize, t - 1.))
            / 2.;
        close(f.centre_velocity_m_s, delta, 0.01);
        u.systems[0].position = Vec3d::new(1e25, -1e25, 1e25);
        assert_eq!(SurfaceFrame::at(&u, r, t).unwrap(), f);
    }
    #[test]
    fn pending_orientation_edits_cannot_relabel_an_older_live_craft_frame() {
        let mut u = world();
        let r = target(&u, "Mars");
        let d = GeographicStart::default()
            .flight(&u, r, Plan::example())
            .unwrap();
        u.apply_edit(crate::sandbox::Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        let craft = u.vehicles[0].clone();
        u.apply_edit(crate::sandbox::Edit::SetProperty {
            body: r,
            property: crate::sandbox::BodyProperty::AxialTilt(0.8),
        })
        .unwrap();
        assert!(Approach::live(&u, &craft, r).is_err());
    }
    #[test]
    fn invalid_geometry_and_historical_live_contact_are_refused() {
        let mut u = world();
        let r = target(&u, "Mars");
        let craft = Flight::new(
            0,
            GeographicStart::default()
                .flight(&u, r, Plan::example())
                .unwrap(),
            &u,
        )
        .unwrap();
        let mut stopped = craft.clone();
        stopped.phase = Phase::Unsupported {
            reason: "test".into(),
        };
        assert!(Approach::live(&u, &stopped, r).is_err());
        assert!(SurfaceFrame::at(&u, target(&u, "Jupiter"), u.time).is_err());
        assert!(SurfaceFrame::at(&u, r, f64::NAN).is_err());
        u.systems[0].bodies[r.body as usize].rotation_period = 0.;
        assert!(SurfaceFrame::at(&u, r, u.time).is_err());
        u.systems[0].bodies[r.body as usize].parent = Some(r.body);
        assert!(SurfaceFrame::at(&u, r, u.time).is_err());
        for start in [
            GeographicStart {
                latitude_deg: 91.,
                ..Default::default()
            },
            GeographicStart {
                spherical_altitude_m: 0.,
                ..Default::default()
            },
            GeographicStart {
                up_velocity_m_s: f64::INFINITY,
                ..Default::default()
            },
        ] {
            assert!(start.validate().is_err());
        }
    }
}
