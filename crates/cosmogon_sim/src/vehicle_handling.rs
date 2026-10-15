//! Opt-in finite surface handling equipment and a pure request/accept/reject preflight.
//! This module authorizes nothing and moves nothing: it bounds one supported operation
//! against authored equipment limits, retained craft cargo and recipient free space.
//! No transfer, timer delivery, invisible grant or discarded overflow exists here.
use crate::supply_vehicle::Cargo;
use crate::world_vehicle::Flight;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-vehicle-handling/1";
pub const LIMITS: &str = "Authored engineering limits for finite surface handling, not measured hardware. Per-species, total and shared oxygen-interface limits bound a single supported operation; the operation-window requirement is evaluated only from solver-integrated supported contact evidence. Reach bounds the modeled spherical surface arc between craft and recipient sites; the mass rate bounds one operation as rate times the current supported window. Declared electrical draw and thermal rejection are authored assumptions: no craft power bus and no handling thermal integration are modeled. This preflight is an accounting evaluation, not a transfer, delivery, landing or mission-feasibility claim.";

/// Authored finite handling equipment for one craft design.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equipment {
    pub model: String,
    /// Maximum of one species moved during one supported operation.
    pub per_species_limit_kg: f64,
    /// Maximum total mass across all species during one supported operation.
    pub total_limit_kg: f64,
    /// One shared finite oxygen-handling interface per supported operation; oxygen
    /// moves do not receive a separate per-species allowance on top of it.
    pub oxygen_interface_limit_kg: f64,
    /// Declared electrical draw while handling. Assumption only; no craft power bus.
    pub power_w: f64,
    /// Declared thermal rejection while handling. Assumption only; not thermally integrated.
    pub thermal_rejection_w: f64,
    /// Minimum continuous supported-operation window required before any handling.
    pub required_operation_window_s: f64,
    /// Maximum modeled spherical surface-arc separation between the craft ground
    /// site and the recipient site for one supported operation.
    #[serde(default = "default_reach_m")]
    pub reach_m: f64,
    /// Maximum total transferred mass per second of the current supported window.
    /// The rate envelope for one operation is mass_rate_kg_s × current_window_s.
    #[serde(default = "default_mass_rate_kg_s")]
    pub mass_rate_kg_s: f64,
}
fn default_reach_m() -> f64 {
    1.0e7
}
fn default_mass_rate_kg_s() -> f64 {
    1.0e6
}
impl Equipment {
    pub fn example() -> Self {
        Self {
            model: MODEL.into(),
            per_species_limit_kg: 200.0,
            total_limit_kg: 400.0,
            oxygen_interface_limit_kg: 100.0,
            power_w: 5000.0,
            thermal_rejection_w: 5000.0,
            required_operation_window_s: 5.0,
            reach_m: 2000.0,
            mass_rate_kg_s: 50.0,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err("Unsupported surface handling model version".into());
        }
        for (v, n) in [
            (self.per_species_limit_kg, "Handling per-species limit"),
            (self.total_limit_kg, "Handling total limit"),
            (
                self.oxygen_interface_limit_kg,
                "Handling oxygen interface limit",
            ),
        ] {
            if !v.is_finite() || !(0.0..=1.0e9).contains(&v) {
                return Err(format!("{n} must be finite and between 0 and 1e9 kg"));
            }
        }
        for (v, n) in [
            (self.power_w, "Handling power draw"),
            (self.thermal_rejection_w, "Handling thermal rejection"),
        ] {
            if !v.is_finite() || !(0.0..=1.0e6).contains(&v) {
                return Err(format!("{n} must be finite and between 0 and 1e6 W"));
            }
        }
        if !self.required_operation_window_s.is_finite()
            || !(0.0..=1.0e7).contains(&self.required_operation_window_s)
        {
            return Err(
                "Handling required operation window must be finite and between 0 and 1e7 s"
                    .into(),
            );
        }
        if !self.reach_m.is_finite() || !(0.0..=1.0e7).contains(&self.reach_m) {
            return Err("Handling reach must be finite and between 0 and 1e7 m".into());
        }
        if !self.mass_rate_kg_s.is_finite() || !(0.0..=1.0e6).contains(&self.mass_rate_kg_s) {
            return Err("Handling mass rate must be finite and between 0 and 1e6 kg/s".into());
        }
        Ok(())
    }
}

/// Accounting result of one handling request: nothing was moved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preflight {
    pub requested: Cargo,
    pub accepted: Cargo,
    pub rejected: Cargo,
    pub current_window_s: f64,
    pub required_operation_window_s: f64,
}

/// Evaluate one handling request against authored equipment limits, retained craft
/// cargo and recipient free space. Strict refusals change nothing; partial acceptance
/// keeps the rejected remainder explicit so it stays on the craft.
pub fn preflight(
    flight: &Flight,
    recipient: &crate::habitat::Habitat,
    requested: &Cargo,
) -> Result<Preflight, String> {
    let equipment = flight
        .definition
        .handling
        .as_ref()
        .ok_or("Handling preflight requires authored handling equipment")?;
    equipment.validate()?;
    if flight.phase != crate::world_vehicle::Phase::Flying {
        return Err("Handling preflight requires a craft in flight".into());
    }
    if !flight.definition.cargo_ledgers {
        return Err("Handling preflight requires cargo ledgers".into());
    }
    let ledger = flight
        .cargo
        .as_ref()
        .ok_or("Handling preflight requires a retained cargo mass ledger")?;
    if flight.definition.parent != recipient.definition.body {
        return Err("Handling requires the same body".into());
    }
    let window = flight
        .operation_ledger
        .as_ref()
        .ok_or("Handling preflight requires surface-operation window evidence")?;
    if window.current_window_s < equipment.required_operation_window_s {
        return Err(format!(
            "Handling requires a continuous supported-operation window of {} s (current {} s)",
            equipment.required_operation_window_s, window.current_window_s
        ));
    }
    if !crate::vehicle_cargo_mass::finite_cargo(requested) {
        return Err(
            "Handling requests must contain finite nonnegative supported quantities".into(),
        );
    }
    let held = &ledger.held;
    let free = recipient.available_cargo();
    let requested_values = [
        requested.oxygen_reserve_kg,
        requested.potable_water_kg,
        requested.food_dry_kg,
        requested.food_water_kg,
    ];
    let held_values = [
        held.oxygen_reserve_kg,
        held.potable_water_kg,
        held.food_dry_kg,
        held.food_water_kg,
    ];
    let free_values = [
        free.oxygen_reserve_kg,
        free.potable_water_kg,
        free.food_dry_kg,
        free.food_water_kg,
    ];
    let mut accepted_values = [0.0; 4];
    for i in 0..4 {
        accepted_values[i] = requested_values[i]
            .min(held_values[i])
            .min(equipment.per_species_limit_kg)
            .min(free_values[i]);
    }
    // Trim to the shared total limit from the end of the fixed species order so the
    // accepted split is deterministic and never exceeds the request.
    let mut total: f64 = accepted_values.iter().sum();
    if total > equipment.total_limit_kg {
        for i in (0..4).rev() {
            if total <= equipment.total_limit_kg {
                break;
            }
            let excess = total - equipment.total_limit_kg;
            let take = excess.min(accepted_values[i]);
            accepted_values[i] -= take;
            total -= take;
        }
    }
    if accepted_values[0] > equipment.oxygen_interface_limit_kg {
        accepted_values[0] = equipment.oxygen_interface_limit_kg;
    }
    let accepted = Cargo {
        oxygen_reserve_kg: accepted_values[0],
        potable_water_kg: accepted_values[1],
        food_dry_kg: accepted_values[2],
        food_water_kg: accepted_values[3],
    };
    let rejected = Cargo {
        oxygen_reserve_kg: requested.oxygen_reserve_kg - accepted.oxygen_reserve_kg,
        potable_water_kg: requested.potable_water_kg - accepted.potable_water_kg,
        food_dry_kg: requested.food_dry_kg - accepted.food_dry_kg,
        food_water_kg: requested.food_water_kg - accepted.food_water_kg,
    };
    Ok(Preflight {
        requested: requested.clone(),
        accepted,
        rejected,
        current_window_s: window.current_window_s,
        required_operation_window_s: equipment.required_operation_window_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::Edit;
    use crate::supply_vehicle::{Command, Plan};
    use crate::vehicle_arrival::{KinematicState, SurfaceFrame};
    use crate::vehicle_contact::Surface;
    use crate::vehicle_contact_dynamics::Hardware;
    use crate::world_vehicle::Definition as FlightDefinition;
    use crate::{universe::UniverseSettings, BodyRef, Scenario, Universe, Vec3d};

    fn operation_envelope() -> crate::vehicle_operation::Definition {
        crate::vehicle_operation::Definition {
            model: crate::vehicle_operation::MODEL.into(),
            support: crate::vehicle_support::Definition::default(),
            required_window_s: 5.,
            maximum_sample_gap_s: 1.000001,
            maximum_structural_utilization: 1.,
            maximum_stroke_utilization: 1.,
            maximum_friction_utilization: 1.,
        }
    }

    fn zero_cargo() -> Cargo {
        Cargo {
            oxygen_reserve_kg: 0.,
            potable_water_kg: 0.,
            food_dry_kg: 0.,
            food_water_kg: 0.,
        }
    }

    /// Mars world with a descending-then-resting contact craft carrying cargo ledgers
    /// and handling equipment, plus a habitat on the same body. The world is advanced
    /// so the operation window is real solver evidence, never manufactured state.
    fn landed_world(
        equipment: Equipment,
        advance_s: f64,
        tweak_habitat: impl FnOnce(&mut crate::habitat::Definition),
    ) -> (Universe, u64) {
        let mut world = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mars = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut habitat_definition = crate::habitat::Definition::at(mars, -4.59, 137.44);
        tweak_habitat(&mut habitat_definition);
        world
            .apply_edit(Edit::CreateHabitat {
                definition: Box::new(habitat_definition),
            })
            .unwrap();
        let frame = SurfaceFrame::at(&world, mars, world.time).unwrap();
        let direction = Vec3d::new(0.8, 0.6, 0.);
        let above = frame
            .to_inertial(&KinematicState {
                position_m: direction * (frame.radius_m + 10000.),
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let surface = Surface::sample(&world, mars, world.time, above.position_m, 1.).unwrap();
        let position = surface.position_m + surface.outward_normal * 1.6;
        let fixed = frame.rotate_to_fixed(position - frame.centre_m);
        let motion = frame
            .to_inertial(&KinematicState {
                position_m: fixed,
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let velocity = motion.velocity_m_s - surface.outward_normal * 0.1;
        let mut plan = Plan::example();
        plan.commands = vec![Command {
            duration_s: 60.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        let mut hardware = Hardware::example(&world, mars, position).unwrap();
        hardware = hardware
            .with_shear(crate::vehicle_shear::Definition::example())
            .unwrap();
        hardware.operation = Some(operation_envelope());
        let body = world.systems[0].body_state(mars.body as usize, world.time);
        let definition = FlightDefinition {
            model: crate::world_vehicle::SHEAR_MODEL.into(),
            frame: crate::world_vehicle::FRAME.into(),
            parent: mars,
            plan,
            initial_relative_position_m: position - body.pos,
            initial_relative_velocity_m_s: velocity - body.vel,
            contact: Some(hardware),
            cargo_ledgers: true,
            cargo_removals: vec![],
            handling: Some(equipment),
        };
        definition.validate(&world).unwrap();
        world
            .apply_edit(Edit::CreateVehicle {
                definition: Box::new(definition),
            })
            .unwrap();
        let id = world.vehicles[0].id;
        if advance_s > 0. {
            world.advance_to(world.time + advance_s, None, None);
        }
        (world, id)
    }

    fn rest_world(equipment: Equipment) -> (Universe, u64) {
        landed_world(equipment, 30., |_| {})
    }

    fn flight(world: &Universe, id: u64) -> &crate::world_vehicle::Flight {
        world.vehicles.iter().find(|v| v.id == id).unwrap()
    }

    fn habitat(world: &Universe) -> &crate::habitat::Habitat {
        &world.habitats[0]
    }

    #[test]
    fn equipment_validation_bounds_are_explicit() {
        assert!(Equipment::example().validate().is_ok());
        let mut bad = Equipment::example();
        bad.model = "cosmogon-vehicle-handling/9".into();
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.per_species_limit_kg = -1.;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.total_limit_kg = f64::NAN;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.oxygen_interface_limit_kg = 1.0e10;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.power_w = -5.;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.thermal_rejection_w = 1.0e9;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.required_operation_window_s = 1.0e9;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.reach_m = -1.;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.reach_m = 1.0e8;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.mass_rate_kg_s = f64::NAN;
        assert!(bad.validate().is_err());
        let mut bad = Equipment::example();
        bad.mass_rate_kg_s = 1.0e9;
        assert!(bad.validate().is_err());
        // Definitions authored before reach/rate existed load at unconstrained defaults.
        let legacy: Equipment = serde_json::from_str(
            r#"{"model":"cosmogon-vehicle-handling/1","per_species_limit_kg":200.0,"total_limit_kg":400.0,"oxygen_interface_limit_kg":100.0,"power_w":5000.0,"thermal_rejection_w":5000.0,"required_operation_window_s":5.0}"#,
        )
        .unwrap();
        assert_eq!(legacy.reach_m, 1.0e7);
        assert_eq!(legacy.mass_rate_kg_s, 1.0e6);
        assert!(legacy.validate().is_ok());
    }

    #[test]
    fn structure_requires_cargo_ledgers_and_operation_hardware() {
        let (world, id) = landed_world(Equipment::example(), 0., |_| {});
        let mut definition = flight(&world, id).definition.clone();
        definition.handling = None;
        definition.validate_structure().unwrap();
        definition.handling = Some(Equipment::example());
        definition.cargo_ledgers = false;
        assert_eq!(
            definition.validate_structure().unwrap_err(),
            "Handling equipment requires cargo ledgers"
        );
        definition.cargo_ledgers = true;
        let operation = definition.contact.as_mut().unwrap().operation.take();
        assert_eq!(
            definition.validate_structure().unwrap_err(),
            "Handling equipment requires authored contact surface-operation hardware"
        );
        definition.contact.as_mut().unwrap().operation = operation;
        let mut bad = Equipment::example();
        bad.model = "cosmogon-vehicle-handling/9".into();
        definition.handling = Some(bad);
        assert!(definition.validate_structure().is_err());
        definition.handling = Some(Equipment::example());
        definition.validate_structure().unwrap();
    }

    #[test]
    fn preflight_refuses_missing_window_evidence() {
        let (world, id) = landed_world(Equipment::example(), 0., |_| {});
        let error = preflight(flight(&world, id), habitat(&world), &zero_cargo()).unwrap_err();
        assert!(
            error.contains("surface-operation window evidence"),
            "{error}"
        );
    }

    #[test]
    fn preflight_refuses_short_window() {
        let mut equipment = Equipment::example();
        equipment.required_operation_window_s = 1000.;
        let (world, id) = rest_world(equipment);
        let error = preflight(flight(&world, id), habitat(&world), &zero_cargo()).unwrap_err();
        assert!(
            error.contains("continuous supported-operation window"),
            "{error}"
        );
    }

    #[test]
    fn preflight_refuses_a_different_body() {
        let (mut world, id) = rest_world(Equipment::example());
        let earth = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Earth").unwrap() as u32,
        };
        world
            .apply_edit(Edit::CreateHabitat {
                definition: Box::new(crate::habitat::Definition::at(earth, 10., 20.)),
            })
            .unwrap();
        let error = preflight(flight(&world, id), &world.habitats[1], &zero_cargo()).unwrap_err();
        assert!(error.contains("same body"), "{error}");
    }

    #[test]
    fn preflight_refuses_missing_capability_and_unsupported_requests() {
        let (world, id) = rest_world(Equipment::example());
        let mut unsupported = flight(&world, id).clone();
        unsupported.definition.handling = None;
        let error = preflight(&unsupported, habitat(&world), &zero_cargo()).unwrap_err();
        assert!(error.contains("authored handling equipment"), "{error}");

        let mut no_ledger = flight(&world, id).clone();
        no_ledger.definition.cargo_ledgers = false;
        let error = preflight(&no_ledger, habitat(&world), &zero_cargo()).unwrap_err();
        assert!(error.contains("cargo ledgers"), "{error}");

        let mut no_ledger_evidence = flight(&world, id).clone();
        no_ledger_evidence.cargo = None;
        let error = preflight(&no_ledger_evidence, habitat(&world), &zero_cargo()).unwrap_err();
        assert!(error.contains("retained cargo mass ledger"), "{error}");

        let mut not_flying = flight(&world, id).clone();
        not_flying.phase = crate::world_vehicle::Phase::Unsupported {
            reason: "test refusal".into(),
        };
        let error = preflight(&not_flying, habitat(&world), &zero_cargo()).unwrap_err();
        assert!(error.contains("craft in flight"), "{error}");

        let mut negative = zero_cargo();
        negative.potable_water_kg = -1.;
        let error = preflight(flight(&world, id), habitat(&world), &negative).unwrap_err();
        assert!(error.contains("finite nonnegative"), "{error}");
        let mut non_finite = zero_cargo();
        non_finite.food_dry_kg = f64::NAN;
        let error = preflight(flight(&world, id), habitat(&world), &non_finite).unwrap_err();
        assert!(error.contains("finite nonnegative"), "{error}");
    }

    #[test]
    fn preflight_splits_request_across_held_equipment_and_total_limits() {
        let mut equipment = Equipment::example();
        equipment.per_species_limit_kg = 60.;
        equipment.total_limit_kg = 100.;
        equipment.oxygen_interface_limit_kg = 25.;
        equipment.required_operation_window_s = 5.;
        let required_window = equipment.required_operation_window_s;
        let (world, id) = rest_world(equipment);
        let flight = flight(&world, id);
        let window = flight.operation_ledger.as_ref().unwrap().current_window_s;
        assert!(window >= 5., "real supported window {window}");
        let requested = Cargo {
            oxygen_reserve_kg: 100.,
            potable_water_kg: 100.,
            food_dry_kg: 30.,
            food_water_kg: 20.,
        };
        let held_before = flight.cargo.as_ref().unwrap().held.clone();
        let pre = preflight(flight, habitat(&world), &requested).unwrap();
        assert_eq!(
            pre.accepted,
            Cargo {
                oxygen_reserve_kg: 20.,
                potable_water_kg: 60.,
                food_dry_kg: 20.,
                food_water_kg: 0.,
            }
        );
        assert_eq!(
            pre.rejected,
            Cargo {
                oxygen_reserve_kg: 80.,
                potable_water_kg: 40.,
                food_dry_kg: 10.,
                food_water_kg: 20.,
            }
        );
        assert!((pre.accepted.mass_kg() - 100.).abs() < 1e-9);
        assert_eq!(pre.requested, requested);
        assert_eq!(pre.current_window_s, window);
        assert_eq!(pre.required_operation_window_s, required_window);
        // The preflight is pure: rejected stock stays on the craft, nothing moved.
        assert_eq!(flight.cargo.as_ref().unwrap().held, held_before);
        assert!((pre.accepted.oxygen_reserve_kg + pre.rejected.oxygen_reserve_kg
            - requested.oxygen_reserve_kg)
            .abs()
            < 1e-12);
        assert!((pre.accepted.potable_water_kg + pre.rejected.potable_water_kg
            - requested.potable_water_kg)
            .abs()
            < 1e-12);
    }

    #[test]
    fn preflight_never_exceeds_recipient_free_space() {
        let (world, id) = landed_world(Equipment::example(), 30., |definition| {
            definition.config.potable_water_capacity_kg = 150.;
            definition.initial.potable_water_kg = 140.;
        });
        let requested = Cargo {
            potable_water_kg: 100.,
            ..zero_cargo()
        };
        let pre = preflight(flight(&world, id), habitat(&world), &requested).unwrap();
        assert_eq!(pre.accepted.potable_water_kg, 10.);
        assert_eq!(pre.rejected.potable_water_kg, 90.);
        assert_eq!(pre.accepted.mass_kg(), 10.);
    }

    #[test]
    fn preflight_accepts_fully_within_all_limits() {
        let (world, id) = rest_world(Equipment::example());
        let requested = Cargo {
            oxygen_reserve_kg: 5.,
            potable_water_kg: 50.,
            food_dry_kg: 10.,
            food_water_kg: 5.,
        };
        let pre = preflight(flight(&world, id), habitat(&world), &requested).unwrap();
        assert_eq!(pre.accepted, requested);
        let empty = zero_cargo();
        assert_eq!(pre.rejected, empty);
    }
}
