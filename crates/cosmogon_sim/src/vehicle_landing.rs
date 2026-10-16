//! Authored terminal-landing completion for contact-gear craft: versioned
//! touchdown speed limits on the first touchdown after any airborne phase,
//! retained touchdown evidence on the craft propulsion clock, and an atomic
//! landing stop at the last supported state when an authored bound is
//! exceeded. Touchdown is the first accepted contact substep where a foot
//! meets the modeled terrain after all feet were clear; surface-start craft
//! never record a touchdown until they lift off and land again. Touchdown
//! speeds are ground-relative through the existing surface spin frame — not
//! a certified landing, attachment, or terrain-hazard model.

use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-vehicle-landing/1";
pub const SCOPE: &str =
    "Authored touchdown-speed limits with retained first-touchdown evidence on the existing contact gear solver";

/// Gap (m) at or below which a foot counts as meeting the modeled terrain.
pub(crate) const TOUCHDOWN_GAP_M: f64 = 0.01;

/// Versioned touchdown-speed limits for a landing-limited flight.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Ground-relative normal speed bound at touchdown; exceeding it stops
    /// the craft atomically at the last supported state.
    pub maximum_touchdown_vertical_speed_m_s: f64,
    /// Ground-relative tangential speed bound at touchdown; exceeding it
    /// stops the craft atomically at the last supported state.
    pub maximum_touchdown_horizontal_speed_m_s: f64,
    /// Height above the authored start clearance that counts as airborne
    /// evidence. A craft that starts above this clearance (a descent start)
    /// is airborne from creation; a surface-pad craft must rise this far
    /// after its start before any later touchdown can count, so gear
    /// settling onto the pad is never a landing.
    pub liftoff_altitude_m: f64,
}

impl Limits {
    pub fn example() -> Self {
        Self {
            maximum_touchdown_vertical_speed_m_s: 10.,
            maximum_touchdown_horizontal_speed_m_s: 5.,
            liftoff_altitude_m: 10.,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for (value, name, maximum) in [
            (
                self.maximum_touchdown_vertical_speed_m_s,
                "Landing maximum touchdown vertical speed",
                1e4,
            ),
            (
                self.maximum_touchdown_horizontal_speed_m_s,
                "Landing maximum touchdown horizontal speed",
                1e4,
            ),
            (self.liftoff_altitude_m, "Landing liftoff altitude", 1e5),
        ] {
            if !value.is_finite() || !(0.0..=maximum).contains(&value) {
                return Err(format!("{name} requires a finite value in 0–{maximum}"));
            }
        }
        Ok(())
    }

    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let value: Self = serde_json::from_str(&crate::habitat_recording::read(path, 1024 * 1024)?)
            .map_err(|e| e.to_string())?;
        value.validate()?;
        Ok(value)
    }

    /// Write these landing limits to a new protected file.
    pub fn write(&self, path: &std::path::Path) -> Result<(), String> {
        use std::io::Write;
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Existing definitions are protected: {e}"))?;
        file.write_all(&bytes).map_err(|e| e.to_string())
    }
}

/// Retained first-touchdown evidence for one landing-limited flight. Times
/// are on the craft propulsion clock; speeds are ground-relative at the
/// first touchdown after the last airborne phase.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    /// Terrain radial clearance at creation; the versioned liftoff bound is
    /// measured above this start, exactly like the launch liftoff baseline.
    pub start_clearance_m: f64,
    pub touchdown_elapsed_s: Option<f64>,
    pub touchdown_vertical_speed_m_s: Option<f64>,
    pub touchdown_horizontal_speed_m_s: Option<f64>,
    pub substeps_observed: u64,
    /// True once the craft has started above the versioned liftoff bound or
    /// has risen that far above its start clearance; retained so a pad-start
    /// craft never records a touchdown from gear settling.
    pub airborne: bool,
}

impl Ledger {
    pub fn new(airborne: bool, start_clearance_m: f64) -> Self {
        Self {
            start_clearance_m,
            touchdown_elapsed_s: None,
            touchdown_vertical_speed_m_s: None,
            touchdown_horizontal_speed_m_s: None,
            substeps_observed: 0,
            airborne,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.start_clearance_m.is_finite()
            || !(0.0..=1e5).contains(&self.start_clearance_m)
        {
            return Err("Landing ledger start clearance requires a finite 0–1e5 m value".into());
        }
        if self.substeps_observed > 10_000_000_000_000 {
            return Err("Landing ledger substeps exceed their supported finite bounds".into());
        }
        if self
            .touchdown_elapsed_s
            .is_some_and(|t| !t.is_finite() || !(0.0..=1e7).contains(&t))
        {
            return Err("Landing ledger touchdown time requires a finite 0–1e7 s clock".into());
        }
        for value in [self.touchdown_vertical_speed_m_s, self.touchdown_horizontal_speed_m_s] {
            if value.is_some_and(|v| !v.is_finite() || !(0.0..=1e4).contains(&v)) {
                return Err("Landing ledger touchdown speeds require finite 0–1e4 m/s values".into());
            }
        }
        match (
            self.touchdown_elapsed_s,
            self.touchdown_vertical_speed_m_s,
            self.touchdown_horizontal_speed_m_s,
        ) {
            (Some(_), None, _) | (Some(_), _, None) | (None, Some(_), _) | (None, _, Some(_)) => {
                return Err("Landing touchdown speeds require touchdown time evidence".into())
            }
            _ => {}
        }
        Ok(())
    }
}

/// Observe one accepted post-substep state of a landing-limited flight. The
/// craft must already have contact hardware; touchdown fires once, on the
/// first substep where any foot meets the modeled terrain after all feet
/// were clear, and the ground-relative speeds are resolved through the
/// parent's existing spin frame.
pub(crate) fn observe(
    flight: &mut crate::world_vehicle::Flight,
    world: &crate::Universe,
    sample_time: f64,
    elapsed_s: f64,
) -> Result<(), String> {
    let limits = flight
        .definition
        .landing
        .ok_or("Landing observation requires authored landing limits")?;
    limits.validate()?;
    if !elapsed_s.is_finite() {
        return Err("Landing observation requires a finite clock".into());
    }
    let ledger = flight
        .landing_ledger
        .as_mut()
        .ok_or("Landing observation requires a retained landing ledger")?;
    ledger.substeps_observed += 1;
    let contact = flight
        .contact
        .as_ref()
        .ok_or("Landing observation requires contact hardware")?;
    let all_clear = contact
        .feet
        .iter()
        .all(|f| f.surface.tangent_gap_m > TOUCHDOWN_GAP_M);
    let clearance = crate::vehicle_contact::Surface::sample(
        world,
        flight.definition.parent,
        sample_time,
        flight.position_m,
        1.,
    )?
    .radial_clearance_m;
    if all_clear {
        if clearance.is_finite()
            && clearance > ledger.start_clearance_m + limits.liftoff_altitude_m
        {
            ledger.airborne = true;
        }
        return Ok(());
    }
    if !ledger.airborne || ledger.touchdown_elapsed_s.is_some() {
        return Ok(());
    }
    let touching = contact
        .feet
        .iter()
        .find(|f| f.surface.tangent_gap_m <= TOUCHDOWN_GAP_M)
        .ok_or("Landing touchdown requires a foot at or below the terrain gap bound")?;
    let normal = touching.surface.outward_normal;
    let frame = crate::vehicle_arrival::SurfaceFrame::at(world, flight.definition.parent, sample_time)?;
    let arm = flight.position_m - frame.centre_m;
    // Ground-relative: the terrain translates and rotates with the parent.
    let ground_relative = flight.velocity_m_s
        - (frame.centre_velocity_m_s + frame.angular_velocity_inertial_rad_s.cross(arm));
    let vertical = (ground_relative.dot(normal)).abs();
    let horizontal = (ground_relative - normal * ground_relative.dot(normal)).length();
    if !vertical.is_finite() || !horizontal.is_finite() {
        return Err("Landing touchdown speeds are not finite".into());
    }
    ledger.touchdown_elapsed_s = Some(elapsed_s);
    ledger.touchdown_vertical_speed_m_s = Some(vertical);
    ledger.touchdown_horizontal_speed_m_s = Some(horizontal);
    if vertical > limits.maximum_touchdown_vertical_speed_m_s {
        return Err(format!(
            "Touchdown vertical speed exceeded the authored landing limit ({vertical} > {} m/s)",
            limits.maximum_touchdown_vertical_speed_m_s
        ));
    }
    if horizontal > limits.maximum_touchdown_horizontal_speed_m_s {
        return Err(format!(
            "Touchdown horizontal speed exceeded the authored landing limit ({horizontal} > {} m/s)",
            limits.maximum_touchdown_horizontal_speed_m_s
        ));
    }
    Ok(())
}

pub fn context() -> serde_json::Value {
    serde_json::json!({
        "model":MODEL,
        "operator":"First touchdown after any airborne phase on the accepted contact substeps of the existing bounded gear solver; ground-relative speeds resolved through the parent's translation and spin",
        "limits":"Authored bounds on touchdown vertical and horizontal ground-relative speed plus a versioned liftoff altitude above the start clearance; exceeding a speed bound stops the craft atomically at the last supported macro state",
        "evidence":"First-touchdown time on the craft propulsion clock, ground-relative vertical and horizontal touchdown speeds, start clearance and observed substep count",
        "scope":SCOPE,
        "limits_list":["No landing-site hazards, terrain roughness beyond the modeled height field, wind, thrust-off timing, attachment or static certification","Touchdown is geometric foot contact within the solver gap bound, not a structural loads or tip-over model","Gear settling onto a pad below the versioned liftoff altitude is never a landing; a descent start above that bound is airborne from creation","Trajectory evidence is accepted-substep geometry, not telemetry or navigation truth"],
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
        vehicle_arrival::SurfaceFrame,
        vehicle_contact::Surface,
        vehicle_contact_dynamics::Hardware,
        world_vehicle::{Definition, Phase},
        BodyRef, Vec3d,
    };

    /// A Mars gear craft at a fixed clearance above the local terrain with a
    /// downward (or zero) initial velocity, a constant body-up thrust along
    /// the local outward normal (or a coast when `hover_fraction` is zero),
    /// and no cargo. A positive `hover_fraction` arms 100 kg of propellant
    /// and tunes the constant throttle to that fraction of the local hover
    /// thrust, so the descent speed is set by real gravity rather than a
    /// hard-coded throttle. Landing mass is `dry_mass_kg` plus that
    /// propellant.
    pub(crate) fn landing_world(
        limits: Limits,
        clearance_m: f64,
        downward_m_s: f64,
        dry_mass_kg: f64,
        hover_fraction: f64,
    ) -> Universe {
        let world = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let frame = SurfaceFrame::at(&world, target, world.time).unwrap();
        let outward = Vec3d::new(0.8, 0.6, 0.);
        let body = world.systems[0].body_state(target.body as usize, world.time);
        let probe = body.pos + outward * (frame.radius_m + 30_000.);
        let surface = Surface::sample(&world, target, world.time, probe, 1.).unwrap();
        let position = surface.position_m + surface.outward_normal * clearance_m;
        let mut plan = Plan::example();
        plan.definition.dry_mass_kg = dry_mass_kg;
        plan.definition.cargo = crate::supply_vehicle::Cargo {
            oxygen_reserve_kg: 0.,
            potable_water_kg: 0.,
            food_dry_kg: 0.,
            food_water_kg: 0.,
        };
        let throttle = if hover_fraction > 0. {
            plan.definition.initial_propellant_kg = 100.;
            let g = 6.674_30e-11 * world.body(target).mass / frame.radius_m.powi(2);
            let hover = (dry_mass_kg + 100.) * g / plan.definition.rated_thrust_n;
            (hover_fraction * hover).clamp(f64::EPSILON, 1.)
        } else {
            plan.definition.initial_propellant_kg = 0.;
            0.
        };
        plan.definition.propellant_capacity_kg = plan
            .definition
            .propellant_capacity_kg
            .max(plan.definition.initial_propellant_kg);
        plan.commands = vec![Command {
            duration_s: 5_000.,
            throttle,
            direction: [
                surface.outward_normal.x,
                surface.outward_normal.y,
                surface.outward_normal.z,
            ],
        }];
        let definition = Definition {
            model: crate::world_vehicle::CONTACT_MODEL.into(),
            frame: crate::world_vehicle::FRAME.into(),
            parent: target,
            plan,
            initial_relative_position_m: position - body.pos,
            // Co-rotate with the terrain so the descent is ground-relative.
            initial_relative_velocity_m_s: frame
                .angular_velocity_inertial_rad_s
                .cross(position - frame.centre_m),
            contact: Some(Hardware::example(&world, target, position).expect("gear is finite")),
            cargo_ledgers: false,
            cargo_removals: Vec::new(),
            handling: None,
            launch: None,
            entry: None,
            landing: Some(limits),
        };
        definition
            .validate(&world)
            .expect("landing definition validates");
        let mut world = world;
        world
            .apply_edit(Edit::CreateVehicle {
                definition: Box::new(definition),
            })
            .expect("landing craft starts above the terrain");
        if downward_m_s != 0. {
            let flight = &mut world.vehicles[0];
            flight.velocity_m_s += surface.outward_normal * -downward_m_s;
        }
        world
    }

    #[test]
    fn landing_limits_validate_their_bounds() {
        let good = Limits::example();
        good.validate().unwrap();
        assert!(Limits {
            maximum_touchdown_vertical_speed_m_s: -1.,
            ..good
        }
        .validate()
        .is_err());
        assert!(Limits {
            maximum_touchdown_horizontal_speed_m_s: f64::NAN,
            ..good
        }
        .validate()
        .is_err());
        assert!(Limits {
            maximum_touchdown_vertical_speed_m_s: 1e6,
            ..good
        }
        .validate()
        .is_err());
    }

    #[test]
    fn landing_ledger_evidence_validates() {
        let mut ledger = Ledger::new(true, 0.);
        ledger.validate().unwrap();
        ledger.touchdown_elapsed_s = Some(10.);
        assert!(ledger.validate().is_err());
        ledger.touchdown_vertical_speed_m_s = Some(3.);
        ledger.touchdown_horizontal_speed_m_s = Some(1.);
        ledger.validate().unwrap();
        ledger.touchdown_vertical_speed_m_s = Some(-1.);
        assert!(ledger.validate().is_err());
        let ledger = Ledger::new(true, -1.);
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn landing_requires_gear() {
        let world = landing_world(Limits::example(), 2_000., 0., 1_000., 0.);
        let flight = &world.vehicles[0];
        let ledger = flight.landing_ledger.as_ref().expect("landing ledger");
        // A 2 km descent start is airborne from creation by the versioned bound.
        assert!(ledger.airborne);
        assert!(ledger.start_clearance_m > 1_900.);

        let mut no_gear = flight.definition.clone();
        no_gear.contact = None;
        no_gear.model = crate::world_vehicle::MODEL.into();
        assert!(no_gear.validate_structure().is_err());
    }

    #[test]
    fn surface_start_craft_records_no_false_touchdown() {
        let mut world = landing_world(Limits::example(), 1.6, 0., 1_000., 0.);
        // Starts just above the pad: below the versioned liftoff bound, so it
        // must never see a touchdown when its gear settles onto the terrain.
        assert!(!world.vehicles[0]
            .landing_ledger
            .as_ref()
            .unwrap()
            .airborne);
        for _ in 0..20 {
            world.advance_to(world.time + 1., None, None);
        }
        let ledger = world.vehicles[0].landing_ledger.as_ref().unwrap();
        assert!(ledger.touchdown_elapsed_s.is_none(), "false touchdown");
        assert!(ledger.substeps_observed > 0);
    }

    #[test]
    fn airborne_craft_touches_down_with_retained_evidence() {
        // A heavy gear craft released 1.8 m above the terrain — just above a
        // 0.8 m versioned liftoff bound — free-falls its last 0.3 m (gear
        // height) and touches down at ≈ 1.5 m/s, within the gear solver's
        // step, stroke and load budget.
        let mut world = landing_world(
            Limits {
                maximum_touchdown_vertical_speed_m_s: 60.,
                maximum_touchdown_horizontal_speed_m_s: 60.,
                liftoff_altitude_m: 0.8,
            },
            1.8,
            0.,
            2_000.,
            0.,
        );
        let mut touched = None;
        for t in 0..30 {
            world.advance_to(world.time + 1., None, None);
            if matches!(world.vehicles[0].phase, Phase::Unsupported { .. }) {
                break;
            }
            if world.vehicles[0]
                .landing_ledger
                .as_ref()
                .unwrap()
                .touchdown_elapsed_s
                .is_some()
            {
                touched = Some(t);
                break;
            }
        }
        let touched = touched.expect("touchdown evidence");
        assert!(touched < 10, "touched down at {touched} s");
        let ledger = world.vehicles[0].landing_ledger.as_ref().unwrap();
        let vertical = ledger
            .touchdown_vertical_speed_m_s
            .expect("vertical touchdown speed");
        assert!(vertical > 0.5 && vertical < 5., "vertical {vertical}");
        let horizontal = ledger
            .touchdown_horizontal_speed_m_s
            .expect("horizontal touchdown speed");
        assert!(horizontal < 2., "horizontal {horizontal}");
    }

    #[test]
    fn landing_stops_atomically_when_the_touchdown_limit_is_exceeded() {
        // 400 m of fall reaches ≈ 54 m/s, above the 10 m/s authored limit.
        let mut world = landing_world(Limits::example(), 400., 0., 1_000., 0.);
        let mut stopped_at = None;
        for t in 0..60 {
            world.advance_to(world.time + 1., None, None);
            if matches!(world.vehicles[0].phase, Phase::Unsupported { .. }) {
                stopped_at = Some(t);
                break;
            }
        }
        let stopped_at = stopped_at.expect("craft stops on the touchdown limit");
        assert!(stopped_at < 30, "stopped at {stopped_at} s");
        let reason = match &world.vehicles[0].phase {
            Phase::Unsupported { reason } => reason.clone(),
            other => panic!("expected Unsupported, got {other:?}"),
        };
        assert!(reason.contains("landing limit"), "reason: {reason}");
    }

    #[test]
    fn landing_ledgers_replay_exactly_after_reopen() {
        let mut world = landing_world(
            Limits {
                maximum_touchdown_vertical_speed_m_s: 60.,
                maximum_touchdown_horizontal_speed_m_s: 60.,
                liftoff_altitude_m: 0.8,
            },
            1.8,
            0.,
            2_000.,
            0.,
        );
        for _ in 0..30 {
            world.advance_to(world.time + 1., None, None);
        }
        let expected = world.vehicles[0].landing_ledger;
        assert!(expected.as_ref().unwrap().touchdown_elapsed_s.is_some());
        let json = crate::save::to_json(&world, "landing").unwrap();
        let mut tampered_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        tampered_value["universe"]["vehicles"][0]["landing_ledger"]
            .as_object_mut()
            .expect("ledger object")["touchdown_vertical_speed_m_s"] = serde_json::json!(-1.);
        let tampered = serde_json::to_string(&tampered_value).unwrap();
        assert!(crate::save::from_json(&tampered).is_err());
        let mut reopened = crate::save::from_json(&json).unwrap();
        for _ in 0..10 {
            world.advance_to(world.time + 1., None, None);
            reopened.advance_to(reopened.time + 1., None, None);
        }
        assert_eq!(
            reopened.vehicles[0].landing_ledger,
            world.vehicles[0].landing_ledger
        );
    }

    #[test]
    fn landing_context_declares_its_models_and_limits() {
        let value = context();
        assert_eq!(value["model"], MODEL);
        assert!(value["limits"].is_string());
        assert!(value["limits_list"].is_array());
    }
}
