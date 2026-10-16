//! Atomic co-located cargo transfer from a supported craft into an existing
//! habitat, consuming the pure handling preflight. One edit commits both sides
//! or refuses and changes nothing; the rejected remainder stays on the craft.
//! Momentum the receiving ground structure must absorb and the declared
//! handling power/thermal draw are retained evidence and assumptions, not
//! modeled structural, thermal or attachment physics.
use crate::habitat::Habitat;
use crate::supply_vehicle::Cargo;
use crate::vehicle_arrival::{KinematicState, SurfaceFrame};
use crate::vehicle_handling::Preflight;
use crate::world_vehicle::Flight;
use crate::{Universe, Vec3d};
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-vehicle-transfer/1";
pub const SCOPE: &str = "Atomic same-body surface transfer between retained craft cargo and finite habitat storage, gated by authored handling limits, a solver-integrated supported-operation window, modeled spherical surface reach and a window-scaled mass-rate envelope. The rejected remainder stays on the craft. Ground-absorbed momentum/kinetic energy and declared power/thermal values are retained evidence and assumptions: no structural, thermal, attachment or power-bus integration is modeled.";
pub const MAX_RECEIPTS: usize = 4096;
pub const MAX_TRANSFERS: usize = 4096;

/// Flight-side retained record of one applied transfer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transfer {
    pub model: String,
    pub world_time_s: f64,
    pub habitat_id: u64,
    pub requested: Cargo,
    pub accepted: Cargo,
    pub rejected: Cargo,
    pub current_window_s: f64,
    pub required_operation_window_s: f64,
    /// Modeled spherical surface arc between craft ground site and habitat site.
    pub surface_separation_m: f64,
    pub reach_m: f64,
    /// mass_rate_kg_s × current_window_s actually available to this operation.
    pub rate_envelope_kg: f64,
    /// Craft velocity relative to the co-rotating surface frame at the site.
    pub ground_relative_velocity_m_s: Vec3d,
    pub ground_absorbed_linear_momentum_kg_m_s: Vec3d,
    pub ground_absorbed_kinetic_energy_j: f64,
    pub declared_power_w: f64,
    pub declared_thermal_rejection_w: f64,
    pub craft_mass_before_kg: f64,
    pub craft_mass_after_kg: f64,
}

/// Habitat-side receipt of one applied transfer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub model: String,
    pub world_time_s: f64,
    pub vehicle_id: u64,
    pub requested: Cargo,
    pub accepted: Cargo,
}

/// Pure co-location and rate evaluation layered on the handling preflight.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub preflight: Preflight,
    /// After the window-scaled rate trim; identical on both commit sides.
    pub accepted: Cargo,
    /// Requested minus rate-trimmed accepted; stays on the craft.
    pub rejected: Cargo,
    pub surface_separation_m: f64,
    pub reach_m: f64,
    pub rate_envelope_kg: f64,
    pub ground_relative_velocity_m_s: Vec3d,
}

#[cfg(test)]
fn zero_cargo() -> Cargo {
    Cargo {
        oxygen_reserve_kg: 0.,
        potable_water_kg: 0.,
        food_dry_kg: 0.,
        food_water_kg: 0.,
    }
}

/// Evaluate one transfer request against the handling preflight, modeled
/// spherical surface reach and the window-scaled rate envelope. Strict
/// refusals change nothing; the accepted/rejected split stays deterministic
/// and conservative on both sides.
pub fn plan(
    u: &Universe,
    flight: &Flight,
    recipient: &Habitat,
    requested: &Cargo,
) -> Result<Plan, String> {
    let preflight = crate::vehicle_handling::preflight(flight, recipient, requested)?;
    let equipment = flight
        .definition
        .handling
        .as_ref()
        .expect("preflight requires equipment");
    let frame = SurfaceFrame::at(u, flight.definition.parent, u.time)?;
    let craft = frame.to_fixed(&KinematicState {
        position_m: flight.position_m,
        velocity_m_s: flight.velocity_m_s,
    })?;
    let craft_radius = craft.position_m.length();
    if !craft_radius.is_finite() || craft_radius <= 0. {
        return Err("Transfer requires a finite craft ground position".into());
    }
    let craft_dir = craft.position_m / craft_radius;
    let fixed = crate::planet::terrain::dir_from_lat_lon(
        recipient.definition.latitude_deg.to_radians(),
        recipient.definition.longitude_east_deg.to_radians(),
    );
    let habitat_dir = Vec3d::new(fixed[0], fixed[1], fixed[2]);
    let cosine = (craft_dir.dot(habitat_dir)).clamp(-1.0, 1.0);
    let separation_m = frame.radius_m * cosine.acos();
    if separation_m > equipment.reach_m {
        return Err(format!(
            "Transfer requires the craft ground site within {} m of the habitat site (current {} m)",
            equipment.reach_m, separation_m
        ));
    }
    let rate_envelope_kg = equipment.mass_rate_kg_s * preflight.current_window_s;
    if !rate_envelope_kg.is_finite() {
        return Err("Handling rate envelope is not finite".into());
    }
    let mut accepted_values = [
        preflight.accepted.oxygen_reserve_kg,
        preflight.accepted.potable_water_kg,
        preflight.accepted.food_dry_kg,
        preflight.accepted.food_water_kg,
    ];
    let mut rejected_values = [
        preflight.rejected.oxygen_reserve_kg,
        preflight.rejected.potable_water_kg,
        preflight.rejected.food_dry_kg,
        preflight.rejected.food_water_kg,
    ];
    let mut total: f64 = accepted_values.iter().sum();
    if total > rate_envelope_kg {
        for i in (0..4).rev() {
            if total <= rate_envelope_kg {
                break;
            }
            let take = (total - rate_envelope_kg).min(accepted_values[i]);
            accepted_values[i] -= take;
            rejected_values[i] += take;
            total -= take;
        }
    }
    // Guarantee held + accepted never exceeds a tank by even one ulp, so the
    // committed habitat state passes its exact capacity comparison.
    let capacity = recipient.storage_capacity_cargo();
    let held = recipient.held_cargo();
    let held_values = [
        held.oxygen_reserve_kg,
        held.potable_water_kg,
        held.food_dry_kg,
        held.food_water_kg,
    ];
    let capacity_values = [
        capacity.oxygen_reserve_kg,
        capacity.potable_water_kg,
        capacity.food_dry_kg,
        capacity.food_water_kg,
    ];
    for i in 0..4 {
        while held_values[i] + accepted_values[i] > capacity_values[i] && accepted_values[i] > 0. {
            accepted_values[i] = accepted_values[i].next_down();
        }
    }
    if accepted_values.iter().all(|v| *v <= 0.) {
        return Err(
            "Transfer request accepts no material within equipment, held-cargo, free-space and rate limits"
                .into(),
        );
    }
    let accepted = Cargo {
        oxygen_reserve_kg: accepted_values[0],
        potable_water_kg: accepted_values[1],
        food_dry_kg: accepted_values[2],
        food_water_kg: accepted_values[3],
    };
    let rejected = Cargo {
        oxygen_reserve_kg: rejected_values[0],
        potable_water_kg: rejected_values[1],
        food_dry_kg: rejected_values[2],
        food_water_kg: rejected_values[3],
    };
    let ground_relative_velocity_m_s = craft.velocity_m_s;
    if ![
        ground_relative_velocity_m_s.x,
        ground_relative_velocity_m_s.y,
        ground_relative_velocity_m_s.z,
    ]
    .iter()
    .all(|v| v.is_finite())
        || ground_relative_velocity_m_s.length() > 1.0e6
    {
        return Err("Transfer ground-relative craft velocity lacks finite support".into());
    }
    Ok(Plan {
        preflight,
        accepted,
        rejected,
        surface_separation_m: separation_m,
        reach_m: equipment.reach_m,
        rate_envelope_kg,
        ground_relative_velocity_m_s,
    })
}

/// Commit the recipient side of a planned transfer: finite tank stocks grow by
/// the exact accepted cargo and a receipt retaining the original request is
/// kept. Callers must have stepped the habitat to the transfer clock and must
/// validate afterwards.
pub fn receive(
    habitat: &mut Habitat,
    time: f64,
    vehicle_id: u64,
    requested: &Cargo,
    accepted: &Cargo,
) -> Result<Receipt, String> {
    if habitat.receipts.len() >= MAX_RECEIPTS {
        return Err("Habitat transfer receipt ledger is full".into());
    }
    habitat.state.oxygen_reserve_kg += accepted.oxygen_reserve_kg;
    habitat.state.potable_water_kg += accepted.potable_water_kg;
    habitat.state.food_dry_kg += accepted.food_dry_kg;
    habitat.state.food_water_kg += accepted.food_water_kg;
    let receipt = Receipt {
        model: MODEL.into(),
        world_time_s: time,
        vehicle_id,
        requested: requested.clone(),
        accepted: accepted.clone(),
    };
    habitat.receipts.push(receipt.clone());
    Ok(receipt)
}

pub(crate) fn validate_receipts(habitat: &Habitat, time: f64) -> Result<(), String> {
    if habitat.receipts.len() > MAX_RECEIPTS {
        return Err("Habitat holds more transfer receipts than supported".into());
    }
    let mut previous = f64::NEG_INFINITY;
    for receipt in &habitat.receipts {
        if receipt.model != MODEL {
            return Err("Unsupported retained transfer receipt model".into());
        }
        if !receipt.world_time_s.is_finite()
            || receipt.world_time_s < previous
            || receipt.world_time_s > time + 0.001
        {
            return Err("Retained transfer receipts lack ordered finite clocks".into());
        }
        previous = receipt.world_time_s;
        if !crate::vehicle_cargo_mass::finite_cargo(&receipt.requested)
            || !crate::vehicle_cargo_mass::finite_cargo(&receipt.accepted)
        {
            return Err("Retained transfer receipts lack finite cargo".into());
        }
        let capacity = habitat.storage_capacity_cargo();
        let accepted = [
            receipt.accepted.oxygen_reserve_kg,
            receipt.accepted.potable_water_kg,
            receipt.accepted.food_dry_kg,
            receipt.accepted.food_water_kg,
        ];
        let capacity_values = [
            capacity.oxygen_reserve_kg,
            capacity.potable_water_kg,
            capacity.food_dry_kg,
            capacity.food_water_kg,
        ];
        if accepted.iter().zip(capacity_values).any(|(a, c)| *a > c) {
            return Err("Transfer receipt exceeds a storage capacity".into());
        }
    }
    Ok(())
}

pub(crate) fn validate_transfers(
    flight: &Flight,
    u: &Universe,
) -> Result<(), String> {
    if flight.transfers.len() > MAX_TRANSFERS {
        return Err("Flight holds more transfer records than supported".into());
    }
    if flight.transfers.len() > flight.cargo_events.len() {
        return Err("Transfer records outnumber retained cargo events".into());
    }
    let mut event_index = 0usize;
    for transfer in &flight.transfers {
        if transfer.model != MODEL {
            return Err("Unsupported retained transfer model".into());
        }
        if !transfer.world_time_s.is_finite()
            || !transfer.rate_envelope_kg.is_finite()
            || transfer.rate_envelope_kg < 0.
            || !transfer.surface_separation_m.is_finite()
            || transfer.surface_separation_m < 0.
            || !transfer.reach_m.is_finite()
            || !transfer.ground_absorbed_kinetic_energy_j.is_finite()
            || transfer.ground_absorbed_kinetic_energy_j < 0.
            || !transfer.craft_mass_before_kg.is_finite()
            || !transfer.craft_mass_after_kg.is_finite()
            || transfer.craft_mass_after_kg <= 0.
            || !crate::vehicle_cargo_mass::finite_cargo(&transfer.requested)
            || !crate::vehicle_cargo_mass::finite_cargo(&transfer.accepted)
            || !crate::vehicle_cargo_mass::finite_cargo(&transfer.rejected)
        {
            return Err("Retained transfer evidence lacks finite supported values".into());
        }
        for (accepted, requested, rejected) in [
            (
                transfer.accepted.oxygen_reserve_kg,
                transfer.requested.oxygen_reserve_kg,
                transfer.rejected.oxygen_reserve_kg,
            ),
            (
                transfer.accepted.potable_water_kg,
                transfer.requested.potable_water_kg,
                transfer.rejected.potable_water_kg,
            ),
            (
                transfer.accepted.food_dry_kg,
                transfer.requested.food_dry_kg,
                transfer.rejected.food_dry_kg,
            ),
            (
                transfer.accepted.food_water_kg,
                transfer.requested.food_water_kg,
                transfer.rejected.food_water_kg,
            ),
        ] {
            if accepted < 0. || rejected < 0. || (accepted + rejected - requested).abs() > 1e-9 * requested.abs().max(1.) {
                return Err("Transfer requested, accepted and rejected cargo do not close".into());
            }
        }
        // Every transfer consumes exactly one co-moving (zero relative velocity)
        // cargo event, in order, carrying the accepted material.
        loop {
            let event = flight
                .cargo_events
                .get(event_index)
                .ok_or("Transfer records lack matching co-moving cargo events")?;
            event_index += 1;
            if event.velocity_relative_m_s == Vec3d::ZERO {
                break;
            }
        }
        let event = &flight.cargo_events[event_index - 1];
        if event.removal.actual_removed != transfer.accepted
            || event.removal.requested != transfer.accepted
        {
            return Err("Transfer accepted cargo disagrees with its cargo event".into());
        }
        if (event.vehicle_mass_before_kg - transfer.craft_mass_before_kg).abs() > 1e-6
            || (event.vehicle_mass_after_kg - transfer.craft_mass_after_kg).abs() > 1e-6
        {
            return Err("Transfer craft masses disagree with its cargo event".into());
        }
        let habitat = u
            .habitats
            .iter()
            .find(|h| h.id == transfer.habitat_id)
            .ok_or("Transfer references a missing habitat")?;
        if habitat.definition.body != flight.definition.parent {
            return Err("Transfer habitat left the original body".into());
        }
        let receipt = habitat
            .receipts
            .iter()
            .find(|r| {
                r.world_time_s == transfer.world_time_s
                    && r.vehicle_id == flight.id
                    && r.accepted == transfer.accepted
                    && r.requested == transfer.requested
            })
            .ok_or("Transfer lacks a matching habitat receipt")?;
        if receipt.model != MODEL {
            return Err("Matching habitat receipt uses an unsupported model".into());
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::sandbox::Edit;
    use crate::supply_vehicle::{Command, Plan as SupplyPlan};
    use crate::vehicle_contact::Surface;
    use crate::vehicle_contact_dynamics::Hardware;
    use crate::world_vehicle::Definition as FlightDefinition;
    use crate::{universe::UniverseSettings, BodyRef, Scenario};

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

    const SITE_LAT: f64 = -4.59;
    const SITE_LON: f64 = 137.44;

    /// Habitat-only Mars world at the transfer site.
    pub(crate) fn habitat_world(tweak: impl FnOnce(&mut crate::habitat::Definition)) -> (Universe, BodyRef) {
        let mut world = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mars = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut habitat_definition = crate::habitat::Definition::at(mars, SITE_LAT, SITE_LON);
        tweak(&mut habitat_definition);
        world
            .apply_edit(Edit::CreateHabitat {
                definition: Box::new(habitat_definition),
            })
            .unwrap();
        (world, mars)
    }

    /// Craft definition resting at the habitat site, positioned at the world's
    /// *current* clock so ordinary recreation lands on the same ground.
    pub(crate) fn site_definition(
        world: &Universe,
        mars: BodyRef,
        equipment: crate::vehicle_handling::Equipment,
    ) -> FlightDefinition {
        let frame = SurfaceFrame::at(world, mars, world.time).unwrap();
        let radial = crate::planet::terrain::dir_from_lat_lon(
            SITE_LAT.to_radians(),
            SITE_LON.to_radians(),
        );
        let radial = Vec3d::new(radial[0], radial[1], radial[2]);
        let above = frame
            .to_inertial(&KinematicState {
                position_m: radial * (frame.radius_m + 10000.),
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let surface = Surface::sample(world, mars, world.time, above.position_m, 1.).unwrap();
        let position = surface.position_m + surface.outward_normal * 1.6;
        let fixed = frame.rotate_to_fixed(position - frame.centre_m);
        let motion = frame
            .to_inertial(&KinematicState {
                position_m: fixed,
                velocity_m_s: Vec3d::ZERO,
            })
            .unwrap();
        let velocity = motion.velocity_m_s - surface.outward_normal * 0.1;
        let mut plan = SupplyPlan::example();
        plan.commands = vec![Command {
            duration_s: 60.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        let mut hardware = Hardware::example(world, mars, position).unwrap();
        hardware = hardware
            .with_shear(crate::vehicle_shear::Definition::example())
            .unwrap();
        hardware.operation = Some(operation_envelope());
        let body = world.systems[0].body_state(mars.body as usize, world.time);
        FlightDefinition {
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
            launch: None,
            entry: None,
            landing: None,
        }
    }

    /// Mars world with the craft resting at the habitat site so a real solver
    /// window and true co-location are both present. The window is always
    /// solver evidence; nothing here manufactures ledger state.
    pub(crate) fn landed_world(
        equipment: crate::vehicle_handling::Equipment,
        advance_s: f64,
        tweak: impl FnOnce(&mut crate::habitat::Definition),
    ) -> (Universe, u64) {
        let (mut world, mars) = habitat_world(tweak);
        let definition = site_definition(&world, mars, equipment);
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

    fn rest_world(equipment: crate::vehicle_handling::Equipment) -> (Universe, u64) {
        landed_world(equipment, 30., |_| {})
    }

    fn flight(world: &Universe, id: u64) -> &Flight {
        world.vehicles.iter().find(|v| v.id == id).unwrap()
    }

    fn habitat(world: &Universe) -> &Habitat {
        &world.habitats[0]
    }

    fn request() -> Cargo {
        Cargo {
            oxygen_reserve_kg: 10.,
            potable_water_kg: 40.,
            food_dry_kg: 5.,
            food_water_kg: 5.,
        }
    }

    fn transfer_edit(id: u64, requested: Cargo) -> Edit {
        Edit::TransferVehicleCargoToHabitat {
            system: 0,
            vehicle_id: id,
            habitat_id: 0,
            requested,
        }
    }

    #[test]
    fn transfer_commits_both_sides_and_leaves_craft_velocity_unchanged() {
        let (mut world, id) = landed_world(crate::vehicle_handling::Equipment::example(), 30., |d| {
            d.config.crew = 0;
        });
        let held_before = flight(&world, id).cargo.as_ref().unwrap().held.clone();
        let stocks_before = habitat(&world).held_cargo();
        let velocity_before = flight(&world, id).velocity_m_s;
        let mass_before = flight(&world, id).mass_kg();
        world.apply_edit(transfer_edit(id, request())).unwrap();
        let after = flight(&world, id);
        let accepted = request();
        let held_after = &after.cargo.as_ref().unwrap().held;
        assert!((held_before.oxygen_reserve_kg - held_after.oxygen_reserve_kg - accepted.oxygen_reserve_kg).abs() < 1e-9);
        assert!((held_before.potable_water_kg - held_after.potable_water_kg - accepted.potable_water_kg).abs() < 1e-9);
        let stocks_after = habitat(&world).held_cargo();
        assert!((stocks_after.potable_water_kg - stocks_before.potable_water_kg - accepted.potable_water_kg).abs() < 1e-9);
        assert!((stocks_after.food_dry_kg - stocks_before.food_dry_kg - accepted.food_dry_kg).abs() < 1e-9);
        assert!((after.velocity_m_s - velocity_before).length() < 1e-9);
        assert!((mass_before - after.mass_kg() - accepted.mass_kg()).abs() < 1e-6);
        assert_eq!(after.transfers.len(), 1);
        let transfer = &after.transfers[0];
        assert_eq!(transfer.accepted, accepted);
        assert_eq!(transfer.rejected, zero_cargo());
        assert_eq!(transfer.habitat_id, 0);
        assert!(transfer.surface_separation_m < 10., "{}", transfer.surface_separation_m);
        assert!(transfer.ground_absorbed_linear_momentum_kg_m_s.length() > 0.);
        assert!(transfer.ground_absorbed_kinetic_energy_j > 0.);
        assert_eq!(habitat(&world).receipts.len(), 1);
        assert_eq!(habitat(&world).receipts[0].accepted, accepted);
        world.validate_vehicle_state().unwrap();
        habitat(&world).validate(world.time).unwrap();
    }

    #[test]
    fn plan_refuses_a_craft_beyond_reach() {
        let mut equipment = crate::vehicle_handling::Equipment::example();
        equipment.reach_m = 1.0;
        let (world, id) = landed_world(equipment, 30., |d| {
            d.latitude_deg = 10.;
            d.longitude_east_deg = 20.;
        });
        let error = plan(&world, flight(&world, id), habitat(&world), &request()).unwrap_err();
        assert!(error.contains("within 1 m of the habitat site"), "{error}");
    }

    #[test]
    fn plan_trims_to_the_rate_envelope_deterministically() {
        let mut equipment = crate::vehicle_handling::Equipment::example();
        equipment.per_species_limit_kg = 100.;
        equipment.total_limit_kg = 400.;
        equipment.oxygen_interface_limit_kg = 100.;
        equipment.mass_rate_kg_s = 2.;
        let (world, id) = landed_world(equipment, 30., |d| {
            d.config.crew = 0;
        });
        let flight = flight(&world, id);
        let window = flight.operation_ledger.as_ref().unwrap().current_window_s;
        let requested = Cargo {
            oxygen_reserve_kg: 10.,
            potable_water_kg: 20.,
            food_dry_kg: 30.,
            food_water_kg: 40.,
        };
        let planned = plan(&world, flight, habitat(&world), &requested).unwrap();
        let envelope = 2. * window;
        assert!((planned.accepted.mass_kg() - envelope).abs() < 1e-9, "{} vs {envelope}", planned.accepted.mass_kg());
        assert_eq!(planned.accepted.food_water_kg, 0.);
        assert_eq!(planned.accepted.food_dry_kg, (envelope - 30.).clamp(0., 30.));
        // Trim order is the fixed end-first rule: water species first.
        assert_eq!(planned.rejected.food_water_kg, 40.);
        assert!((planned.accepted.mass_kg() + planned.rejected.mass_kg() - requested.mass_kg()).abs() < 1e-9);
    }

    #[test]
    fn plan_refuses_requests_that_accept_nothing() {
        let (world, id) = rest_world(crate::vehicle_handling::Equipment::example());
        let error = plan(&world, flight(&world, id), habitat(&world), &zero_cargo()).unwrap_err();
        assert!(error.contains("accepts no material"), "{error}");
    }

    #[test]
    fn refusal_changes_neither_side() {
        let mut equipment = crate::vehicle_handling::Equipment::example();
        equipment.required_operation_window_s = 1000.;
        let (mut world, id) = rest_world(equipment);
        let flight_before = flight(&world, id).clone();
        let habitat_before = habitat(&world).clone();
        let error = world.apply_edit(transfer_edit(id, request())).unwrap_err();
        assert!(error.contains("continuous supported-operation window"), "{error}");
        assert_eq!(flight(&world, id), &flight_before);
        assert_eq!(habitat(&world), &habitat_before);
    }

    #[test]
    fn partial_acceptance_keeps_the_rejected_remainder_on_the_craft() {
        let (mut world, id) = landed_world(crate::vehicle_handling::Equipment::example(), 30., |d| {
            d.config.crew = 0;
            d.config.potable_water_capacity_kg = 150.;
            d.initial.potable_water_kg = 140.;
        });
        let requested = Cargo {
            potable_water_kg: 30.,
            ..zero_cargo()
        };
        world.apply_edit(transfer_edit(id, requested.clone())).unwrap();
        let transfer = &flight(&world, id).transfers[0];
        assert_eq!(transfer.accepted.potable_water_kg, 10.);
        assert_eq!(transfer.rejected.potable_water_kg, 20.);
        assert_eq!(habitat(&world).state.potable_water_kg, 150.);
        let held = flight(&world, id).cargo.as_ref().unwrap().held.potable_water_kg;
        assert!(held > 0., "rejected stock stays on the craft: {held}");
        world.validate_vehicle_state().unwrap();
    }

    #[test]
    fn transfer_save_roundtrip_is_exact_and_legacy_stays_unchanged() {
        let (mut world, id) = rest_world(crate::vehicle_handling::Equipment::example());
        world.apply_edit(transfer_edit(id, request())).unwrap();
        let json = crate::save::to_json(&world, "transfer roundtrip").unwrap();
        let reloaded = crate::save::from_json(&json).unwrap();
        assert_eq!(reloaded, world);
        reloaded.validate_vehicle_state().unwrap();
        reloaded.habitats[0].validate(reloaded.time).unwrap();
        // A world with no transfers serializes without the new fields at all.
        let (plain, _) = rest_world(crate::vehicle_handling::Equipment::example());
        let plain_json = crate::save::to_json(&plain, "no transfers").unwrap();
        assert!(!plain_json.contains("\"transfers\""));
        assert!(!plain_json.contains("\"receipts\""));
    }

    #[test]
    fn tampered_transfer_records_are_refused() {
        let (mut world, id) = rest_world(crate::vehicle_handling::Equipment::example());
        world.apply_edit(transfer_edit(id, request())).unwrap();
        world.vehicles[0].transfers[0].accepted.potable_water_kg = 1.;
        assert!(world.validate_vehicle_state().is_err());
        let (mut world, id) = rest_world(crate::vehicle_handling::Equipment::example());
        world.apply_edit(transfer_edit(id, request())).unwrap();
        world.habitats[0].receipts.clear();
        assert!(world.validate_vehicle_state().is_err());
        let (mut world, id) = rest_world(crate::vehicle_handling::Equipment::example());
        world.apply_edit(transfer_edit(id, request())).unwrap();
        world.vehicles[0].transfers[0].model = "cosmogon-vehicle-transfer/9".into();
        assert!(world.validate_vehicle_state().is_err());
    }

    #[test]
    fn transfer_requires_the_same_handling_gates_as_preflight() {
        let mut world = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mars = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        world
            .apply_edit(Edit::CreateHabitat {
                definition: Box::new(crate::habitat::Definition::at(mars, SITE_LAT, SITE_LON)),
            })
            .unwrap();
        // A world with a habitat but no craft cannot transfer.
        let error = world
            .apply_edit(transfer_edit(7, request()))
            .unwrap_err();
        assert!(error.contains("No such finite vehicle"), "{error}");
    }
}

