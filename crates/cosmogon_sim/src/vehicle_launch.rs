//! Authored finite launch from a modeled surface: versioned structural limits
//! on engine specific force, automatic liftoff/burnout/apogee trajectory
//! evidence measured above the local terrain, and an atomic structural-failure
//! stop at the last supported state when the authored bound is exceeded.
//! Launch-limited craft must rest on authored contact landing gear; the parent
//! surface is the existing modeled tilt/spin height field, not a calibrated
//! launch site.

use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-vehicle-launch/1";
pub const SCOPE: &str =
    "Authored finite launch with structural specific-force limits and retained trajectory evidence";

/// Versioned structural limits for an authored launch-limited flight.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Sensed engine specific force bound (thrust over current inert mass).
    /// Exceeding it stops the craft atomically at the last supported state.
    pub maximum_acceleration_m_s2: f64,
    /// Height above the authored start position that counts as liftoff
    /// evidence (settling onto the gear never triggers it).
    pub liftoff_altitude_m: f64,
    /// Launch definitions must start at or below this height above the local
    /// modeled terrain (surface pads, not orbital starts).
    pub maximum_start_altitude_m: f64,
}

impl Limits {
    pub fn example() -> Self {
        Self {
            maximum_acceleration_m_s2: 15.,
            liftoff_altitude_m: 1.,
            maximum_start_altitude_m: 100.,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for (value, name, maximum) in [
            (
                self.maximum_acceleration_m_s2,
                "Launch maximum acceleration",
                1e4,
            ),
            (self.liftoff_altitude_m, "Launch liftoff altitude", 1e5),
            (
                self.maximum_start_altitude_m,
                "Launch maximum start altitude",
                1e5,
            ),
        ] {
            if !value.is_finite() || !(0.0..=maximum).contains(&value) {
                return Err(format!("{name} requires a finite value in 0–{maximum}"));
            }
        }
        if self.liftoff_altitude_m > self.maximum_start_altitude_m {
            return Err(
                "Launch liftoff altitude cannot exceed the maximum start altitude".into(),
            );
        }
        Ok(())
    }
}

/// Retained trajectory evidence for one launch-limited flight. Times are on
/// the craft propulsion clock (elapsed since the definition's launch
/// reference); altitudes are heights above the local modeled terrain along the
/// radial, not certified geodetic altitudes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub initial_propellant_kg: f64,
    /// Surface-relative height at the authored start; liftoff is this plus
    /// the versioned liftoff bound, so gear settling never counts as liftoff.
    pub start_altitude_m: f64,
    pub liftoff_elapsed_s: Option<f64>,
    pub burnout_elapsed_s: Option<f64>,
    pub apogee_altitude_m: f64,
    pub peak_acceleration_m_s2: f64,
    pub propellant_consumed_kg: f64,
    pub substeps_observed: u64,
    /// True once any positive-thrust substep has been observed; retained so
    /// burnout detection survives substep boundaries in the saved ledger.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub powered: bool,
}

impl Ledger {
    pub fn new(initial_propellant_kg: f64, start_altitude_m: f64) -> Self {
        Self {
            initial_propellant_kg,
            start_altitude_m,
            liftoff_elapsed_s: None,
            burnout_elapsed_s: None,
            apogee_altitude_m: 0.,
            peak_acceleration_m_s2: 0.,
            propellant_consumed_kg: 0.,
            substeps_observed: 0,
            powered: false,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.initial_propellant_kg.is_finite() || self.initial_propellant_kg < 0. {
            return Err(
                "Launch ledger initial propellant requires a finite non-negative value".into(),
            );
        }
        if !self.start_altitude_m.is_finite()
            || !(0.0..=1e5).contains(&self.start_altitude_m)
        {
            return Err("Launch ledger start altitude requires a finite 0–1e5 m value".into());
        }
        if !self.propellant_consumed_kg.is_finite()
            || self.propellant_consumed_kg < 0.
            || !self.apogee_altitude_m.is_finite()
            || self.apogee_altitude_m < 0.
            || !self.peak_acceleration_m_s2.is_finite()
            || self.peak_acceleration_m_s2 < 0.
            || self.substeps_observed > 10_000_000_000_000
        {
            return Err("Launch ledger evidence exceeds its supported finite bounds".into());
        }
        for t in [self.liftoff_elapsed_s, self.burnout_elapsed_s] {
            if t.is_some_and(|t| !t.is_finite() || !(0.0..=1e7).contains(&t)) {
                return Err("Launch ledger times require finite 0–1e7 s clocks".into());
            }
        }
        match (self.liftoff_elapsed_s, self.burnout_elapsed_s) {
            (Some(l), Some(b)) if b < l => {
                return Err("Launch burnout cannot precede liftoff".into())
            }
            (None, Some(_)) => {
                return Err("Launch burnout evidence requires liftoff evidence".into())
            }
            _ => {}
        }
        Ok(())
    }
}

/// Observe one accepted post-substep state of a launch-limited flight. The
/// specific force is the effective propulsion-bench thrust over its inert
/// mass; the surface altitude is resolved at `sample_time` on the parent's
/// existing tilt/spin height field.
pub(crate) fn observe(
    flight: &mut crate::world_vehicle::Flight,
    world: &crate::Universe,
    sample_time: f64,
    throttle: f64,
    thrust_acceleration_m_s2: f64,
    elapsed_s: f64,
) -> Result<(), String> {
    let limits = flight
        .definition
        .launch
        .ok_or("Launch observation requires authored launch limits")?;
    limits.validate()?;
    if !thrust_acceleration_m_s2.is_finite()
        || thrust_acceleration_m_s2 < 0.
        || !throttle.is_finite()
        || !elapsed_s.is_finite()
    {
        return Err("Launch observation requires a finite thrust and clock".into());
    }
    let ledger = flight
        .launch_ledger
        .get_or_insert_with(|| Ledger::new(flight.propulsion.propellant_kg, 0.));
    ledger.substeps_observed += 1;
    ledger.peak_acceleration_m_s2 = ledger.peak_acceleration_m_s2.max(thrust_acceleration_m_s2);
    ledger.propellant_consumed_kg =
        (ledger.initial_propellant_kg - flight.propulsion.propellant_kg).max(0.);
    if thrust_acceleration_m_s2 > limits.maximum_acceleration_m_s2 {
        return Err(format!(
            "Launch acceleration exceeded the authored structural limit ({thrust_acceleration_m_s2} > {} m/s²)",
            limits.maximum_acceleration_m_s2
        ));
    }
    let surface = crate::vehicle_contact::Surface::sample(
        world,
        flight.definition.parent,
        sample_time,
        flight.position_m,
        1.,
    )?;
    let altitude = surface.radial_clearance_m;
    if !altitude.is_finite() {
        return Err("Launch altitude above the local terrain is not finite".into());
    }
    if ledger.liftoff_elapsed_s.is_none()
        && altitude > ledger.start_altitude_m + limits.liftoff_altitude_m
    {
        ledger.liftoff_elapsed_s = Some(elapsed_s);
    }
    if throttle > 0. {
        ledger.powered = true;
    }
    if ledger.liftoff_elapsed_s.is_some()
        && ledger.powered
        && throttle <= 0.
        && ledger.burnout_elapsed_s.is_none()
    {
        ledger.burnout_elapsed_s = Some(elapsed_s);
    }
    if ledger.liftoff_elapsed_s.is_some() {
        ledger.apogee_altitude_m = ledger.apogee_altitude_m.max(altitude);
    }
    Ok(())
}

pub fn context() -> serde_json::Value {
    serde_json::json!({
        "model":MODEL,
        "operator":"Authored inertial fixed-direction engine thrust through the craft centre on the existing bounded contact solver; sensed specific force is thrust over current inert mass",
        "limits":"Structural bound on sensed specific force plus versioned liftoff/start heights above the local terrain; exceeding the bound stops the craft atomically at the last supported macro state",
        "evidence":"Liftoff/burnout on the craft propulsion clock, running post-liftoff apogee above the local modeled terrain along the radial, peak sensed specific force, consumed propellant and observed substep count",
        "surface":"Existing body-fixed tilt/spin height field via the contact surface sampler; no pad, weather, or areoid launch-site hazards",
        "scope":SCOPE,
        "limits_list":["No attitude controller, gimbal, aerodynamic drag/lift, plume/ground interaction or pad modeling","Specific force is an engine bound, not measured structural load paths or vehicle loads distribution","Trajectory evidence is accepted-substep geometry, not telemetry or navigation truth","No atmospheric ascent losses beyond gravity and the modeled thrust; entry/heating limits are a separate model","Local-terrain altitude is the contact sampler's radial clearance, not a certified geodetic altitude"],
        "empirical_validation":null,"uncertainty":null
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        sandbox::Edit,
        supply_vehicle::{Command, Plan},
        universe::Universe,
        vehicle_arrival::{KinematicState, SurfaceFrame},
        vehicle_contact::Surface,
        world_vehicle::{Definition, Phase},
        BodyRef, Vec3d,
    };

    /// A Mars surface pad with contact landing gear and a single upward (local
    /// zenith at ignition) burn on the example craft.
    pub(crate) fn launch_world(
        thrust_s: f64,
        limits: Limits,
        thrust_n: Option<f64>,
        start_clearance_m: f64,
    ) -> Universe {
        let mut world = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let frame = SurfaceFrame::at(&world, target, world.time).unwrap();
        // Explicit nonpolar source-supported launch site.
        let outward = Vec3d::new(0.8, 0.6, 0.);
        let above = frame
            .to_inertial(&KinematicState {
                position_m: outward * (frame.radius_m + 10000.),
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let surface = Surface::sample(&world, target, world.time, above.position_m, 1.).unwrap();
        let body = world.systems[0].body_state(target.body as usize, world.time);
        let up = surface.outward_normal;
        let definition = Definition {
            model: crate::world_vehicle::CONTACT_MODEL.into(),
            frame: crate::world_vehicle::FRAME.into(),
            parent: target,
            plan: {
                let mut plan = Plan::example();
                plan.definition = {
                    let mut d = crate::supply_vehicle::Definition::example();
                    if let Some(n) = thrust_n {
                        d.rated_thrust_n = n;
                    }
                    d
                };
                plan.commands = vec![Command {
                    duration_s: thrust_s,
                    throttle: 1.,
                    direction: [up.x, up.y, up.z],
                }];
                plan
            },
            initial_relative_position_m: (surface.position_m + up * start_clearance_m) - body.pos,
            initial_relative_velocity_m_s: Vec3d::ZERO,
            contact: Some(
                crate::vehicle_contact_dynamics::Hardware::example(
                    &world,
                    target,
                    surface.position_m + up * start_clearance_m,
                )
                .expect("contact gear example is finite"),
            ),
            cargo_ledgers: false,
            cargo_removals: Vec::new(),
            handling: None,
            launch: Some(limits),
            entry: None,
            landing: None,
        };
        definition.validate(&world).expect("launch definition validates");
        world
            .apply_edit(Edit::CreateVehicle {
                definition: Box::new(definition),
            })
            .expect("launch craft starts on the pad");
        world
    }

    #[test]
    fn launch_limits_validate_their_bounds() {
        let good = Limits::example();
        good.validate().unwrap();
        assert!(Limits {
            maximum_acceleration_m_s2: -1.,
            ..good
        }
        .validate()
        .is_err());
        assert!(Limits {
            maximum_acceleration_m_s2: f64::NAN,
            ..good
        }
        .validate()
        .is_err());
        assert!(Limits {
            liftoff_altitude_m: 1e6,
            ..good
        }
        .validate()
        .is_err());
        assert!(Limits {
            liftoff_altitude_m: 200.,
            maximum_start_altitude_m: 100.,
            ..good
        }
        .validate()
        .is_err());
    }

    #[test]
    fn launch_ledger_evidence_validates() {
        let mut ledger = Ledger::new(1000., 0.);
        ledger.validate().unwrap();
        ledger.liftoff_elapsed_s = Some(10.);
        ledger.burnout_elapsed_s = Some(5.);
        assert!(ledger.validate().is_err());
        ledger.burnout_elapsed_s = None;
        ledger.liftoff_elapsed_s = None;
        ledger.burnout_elapsed_s = Some(5.);
        assert!(ledger.validate().is_err());
        let mut ledger = Ledger::new(1000., 0.);
        ledger.propellant_consumed_kg = -1.;
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn launch_definitions_require_gear_and_a_surface_start() {
        let world = launch_world(50., Limits::example(), None, 1.6);
        let flight = &world.vehicles[0];
        assert!(flight.launch_ledger.is_some());

        // Without contact gear the craft could not rest on a pad.
        let mut no_gear = flight.definition.clone();
        no_gear.contact = None;
        no_gear.model = crate::world_vehicle::MODEL.into();
        assert!(no_gear.validate_structure().is_err());

        // An orbital start is refused by the surface-relative start bound.
        let mut orbital = flight.definition.clone();
        orbital.initial_relative_position_m *= 1.05;
        assert!(crate::world_vehicle::Flight::new(99, orbital, &world).is_err());
    }

    #[test]
    fn surface_launch_liftoffs_and_records_trajectory_evidence() {
        let mut world = launch_world(200., Limits::example(), None, 1.6);
        for _ in 0..450 {
            world.advance_to(world.time + 1., None, None);
        }
        let flight = &world.vehicles[0];
        let ledger = flight.launch_ledger.as_ref().expect("launch ledger");
        let liftoff = ledger.liftoff_elapsed_s.expect("liftoff evidence");
        assert!(liftoff < 120., "liftoff at {liftoff}");
        let burnout = ledger.burnout_elapsed_s.expect("burnout evidence");
        assert!(
            burnout >= liftoff && burnout <= 260.,
            "burnout at {burnout}"
        );
        assert!(
            ledger.apogee_altitude_m > 5_000.,
            "apogee {}",
            ledger.apogee_altitude_m
        );
        assert!(
            ledger.peak_acceleration_m_s2 > 4.9 && ledger.peak_acceleration_m_s2 <= 10.1,
            "peak {}",
            ledger.peak_acceleration_m_s2
        );
        assert!(ledger.propellant_consumed_kg > 600.);
        assert!(ledger.substeps_observed > 100);
        assert!(matches!(flight.phase, Phase::Flying));
    }

    #[test]
    fn launch_stops_atomically_when_the_structural_limit_is_exceeded() {
        let mut world = launch_world(
            200.,
            Limits {
                maximum_acceleration_m_s2: 4.5,
                ..Limits::example()
            },
            None,
            1.6,
        );
        let mut stopped_at = None;
        for t in 0..300 {
            world.advance_to(world.time + 1., None, None);
            if matches!(world.vehicles[0].phase, Phase::Unsupported { .. }) {
                stopped_at = Some(t);
                break;
            }
        }
        let stopped_at = stopped_at.expect("craft stops on the over-g burn");
        assert!(stopped_at < 10, "stopped at {stopped_at} s");
        let reason = match &world.vehicles[0].phase {
            Phase::Unsupported { reason } => reason.clone(),
            other => panic!("expected Unsupported, got {other:?}"),
        };
        assert!(reason.contains("structural limit"), "reason: {reason}");
        assert!(
            world.vehicles[0]
                .launch_ledger
                .as_ref()
                .unwrap()
                .liftoff_elapsed_s
                .is_none(),
            "a pad-level over-g stop is not a liftoff"
        );
    }

    #[test]
    fn launch_ledgers_replay_exactly_after_reopen() {
        let mut world = launch_world(120., Limits::example(), None, 1.6);
        for _ in 0..80 {
            world.advance_to(world.time + 1., None, None);
        }
        let expected = world.vehicles[0].launch_ledger;
        assert!(expected.as_ref().unwrap().liftoff_elapsed_s.is_some());
        let json = crate::save::to_json(&world, "launch").unwrap();
        let mut tampered_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        tampered_value["universe"]["vehicles"][0]["launch_ledger"]
            .as_object_mut()
            .expect("ledger object")["peak_acceleration_m_s2"] = serde_json::json!(-1.);
        let tampered = serde_json::to_string(&tampered_value).unwrap();
        assert!(crate::save::from_json(&tampered).is_err());
        let mut reopened = crate::save::from_json(&json).unwrap();
        for _ in 0..40 {
            world.advance_to(world.time + 1., None, None);
            reopened.advance_to(reopened.time + 1., None, None);
        }
        assert_eq!(
            reopened.vehicles[0].launch_ledger,
            world.vehicles[0].launch_ledger
        );
    }

    #[test]
    fn launch_context_declares_its_models_and_limits() {
        let value = context();
        assert_eq!(value["model"], MODEL);
        assert!(value["limits"].is_string());
        assert!(value["limits_list"].is_array());
    }
}
