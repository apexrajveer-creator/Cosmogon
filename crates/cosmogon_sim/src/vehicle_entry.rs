//! Authored atmospheric-entry heating evidence from the parent body's
//! existing bulk atmosphere: stagnation-point convective heat flux through an
//! exponential height model, versioned heat-flux/total-heat limits, and an
//! atomic thermal stop at the last supported state when an authored bound is
//! exceeded. Entry-limited craft must rest on authored contact landing gear;
//! heating is integrated on the accepted contact substeps that already resolve
//! the gear, so free-flight recordings and the golden world state replay
//! exactly. The atmosphere is the existing modeled bulk composition — not a
//! calibrated entry corridor, radiation, or ablation model.

use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-vehicle-entry/1";
pub const SCOPE: &str =
    "Authored stagnation-point entry heating from an exponential parent atmosphere with versioned heat limits and retained thermal evidence";

/// Versioned thermal limits and geometry for an entry-limited flight.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// Height above the parent's reference sphere counted as the entry
    /// interface. A crossing is recorded evidence, not a physical boundary;
    /// heating accumulates at every accepted substep from the exponential
    /// atmosphere, which is never exactly zero above the interface.
    pub entry_interface_altitude_m: f64,
    /// Effective stagnation-point nose radius of the correlation.
    pub nose_radius_m: f64,
    /// Sutton-Graves heat-flux coefficient: q = k·sqrt(ρ/Rn)·v³ with q in
    /// W/m², ρ in kg/m³, Rn in m and v in m/s. The classic air value does not
    /// transfer to other gas mixtures; it is an authored assumption.
    pub heat_flux_coefficient: f64,
    /// Stagnation heat-flux bound; exceeding it stops the craft atomically at
    /// the last supported state.
    pub maximum_heat_flux_w_m2: f64,
    /// Integrated stagnation fluence bound (J/m²); exceeding it stops the
    /// craft atomically at the last supported state.
    pub maximum_total_heat_j_m2: f64,
}

impl Entry {
    pub fn example() -> Self {
        Self {
            entry_interface_altitude_m: 125_000.,
            nose_radius_m: 1.,
            heat_flux_coefficient: 1.7415e-4,
            maximum_heat_flux_w_m2: 1e7,
            maximum_total_heat_j_m2: 1e10,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for (value, name, minimum, maximum) in [
            (
                self.entry_interface_altitude_m,
                "Entry interface altitude",
                0.,
                1e7,
            ),
            (self.nose_radius_m, "Entry nose radius", 1e-3, 1e4),
            (
                self.heat_flux_coefficient,
                "Entry heat-flux coefficient",
                0.,
                1e-1,
            ),
            (
                self.maximum_heat_flux_w_m2,
                "Entry maximum heat flux",
                1e-30,
                1e12,
            ),
            (
                self.maximum_total_heat_j_m2,
                "Entry maximum total heat",
                1e-30,
                1e18,
            ),
        ] {
            if !value.is_finite() || !(minimum..=maximum).contains(&value) {
                return Err(format!("{name} requires a finite value in {minimum}–{maximum}"));
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

    /// Write these entry limits to a new protected file.
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

/// Retained thermal evidence for one entry-limited flight. Times are on the
/// craft propulsion clock; the interface crossing is the first accepted
/// substep at or below the authored interface altitude after any substep
/// above it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    /// True once any accepted substep has been above the authored interface;
    /// retained so a later below-interface substep records a crossing across
    /// substep boundaries in the saved ledger.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub above_interface: bool,
    pub interface_elapsed_s: Option<f64>,
    pub peak_heat_flux_w_m2: f64,
    pub total_heat_j_m2: f64,
    pub substeps_observed: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            above_interface: false,
            interface_elapsed_s: None,
            peak_heat_flux_w_m2: 0.,
            total_heat_j_m2: 0.,
            substeps_observed: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.peak_heat_flux_w_m2.is_finite()
            || self.peak_heat_flux_w_m2 < 0.
            || !self.total_heat_j_m2.is_finite()
            || self.total_heat_j_m2 < 0.
            || self.substeps_observed > 10_000_000_000_000
        {
            return Err("Entry ledger evidence exceeds its supported finite bounds".into());
        }
        if self
            .interface_elapsed_s
            .is_some_and(|t| !t.is_finite() || !(0.0..=1e7).contains(&t))
        {
            return Err("Entry ledger interface time requires a finite 0–1e7 s clock".into());
        }
        Ok(())
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

/// Mean molar mass (kg/mol) of the existing bulk composition. The collapsed
/// H₂/He fraction is treated as 0.004 kg/mol (helium-like); that blend is an
/// authored assumption, not a calibrated composition.
fn mean_molar_mass(atmosphere: &crate::astro::body::Atmosphere) -> f64 {
    atmosphere.n2 * 0.028
        + atmosphere.o2 * 0.032
        + atmosphere.co2 * 0.044
        + atmosphere.h2o * 0.018
        + atmosphere.ch4 * 0.016
        + atmosphere.h2he * 0.004
}

/// Observe one accepted post-substep state of an entry-limited flight. The
/// parent atmosphere is the existing bulk composition through an exponential
/// height model off the reference sphere; the freestream velocity excludes
/// the parent's rigid spin via the existing surface frame.
pub(crate) fn observe(
    flight: &mut crate::world_vehicle::Flight,
    world: &crate::Universe,
    sample_time: f64,
    elapsed_s: f64,
    step_s: f64,
) -> Result<(), String> {
    let entry = flight
        .definition
        .entry
        .ok_or("Entry observation requires authored entry limits")?;
    entry.validate()?;
    if !elapsed_s.is_finite() || !step_s.is_finite() || step_s <= 0. {
        return Err("Entry observation requires a finite positive substep and clock".into());
    }
    let parent = flight.definition.parent;
    let body = world
        .systems
        .get(parent.system as usize)
        .and_then(|s| s.bodies.get(parent.body as usize))
        .ok_or("Entry heating requires a retained modeled parent body")?;
    let parent_sample = flight
        .last_sources
        .iter()
        .find(|s| matches!(&s.source, crate::world_vehicle::Source::Body(b) if *b == parent.body))
        .ok_or("Entry heating requires a retained parent gravity sample")?;
    let radius = (flight.position_m - parent_sample.position_m).length();
    let altitude = radius - parent_sample.radius_m;
    if !altitude.is_finite() {
        return Err("Entry altitude above the parent reference sphere is not finite".into());
    }
    // Exponential height model: ρ0 = p0/(R·T0), H = R·T0/g0 off the reference
    // sphere, with the existing bulk composition and surface temperature.
    let atmosphere = &body.atmosphere;
    let molar_mass = mean_molar_mass(atmosphere);
    let t0 = body.temperature;
    let g0 = parent_sample.mu_m3_s2 / (parent_sample.radius_m * parent_sample.radius_m);
    let (rho, scale_height) = if molar_mass > 0. && atmosphere.pressure_bar > 0. && t0 > 0. && g0 > 0.
    {
        let r_specific = 8.314462618 / molar_mass;
        let rho0 = atmosphere.pressure_bar * 1e5 / (r_specific * t0);
        (rho0, r_specific * t0 / g0)
    } else {
        (0., 1.)
    };
    let density = if scale_height > 0. {
        rho * (-altitude.max(0.) / scale_height).exp()
    } else {
        0.
    };
    let frame = crate::vehicle_arrival::SurfaceFrame::at(world, parent, sample_time)?;
    // The atmosphere translates and rotates with the parent: remove the
    // body-centre orbital velocity and the rigid-spin wind together.
    let wind = frame.centre_velocity_m_s
        + frame
            .angular_velocity_inertial_rad_s
            .cross(flight.position_m - frame.centre_m);
    let speed = (flight.velocity_m_s - wind).length();
    if !speed.is_finite() {
        return Err("Entry freestream speed is not finite".into());
    }
    let heat_flux_w_m2 =
        entry.heat_flux_coefficient * (density / entry.nose_radius_m).sqrt() * speed.powi(3);
    if !heat_flux_w_m2.is_finite() || heat_flux_w_m2 < 0. {
        return Err("Entry heat flux is not finite".into());
    }
    let ledger = flight
        .entry_ledger
        .get_or_insert_with(crate::vehicle_entry::Ledger::new);
    ledger.substeps_observed += 1;
    if altitude > entry.entry_interface_altitude_m {
        ledger.above_interface = true;
    } else if ledger.above_interface && ledger.interface_elapsed_s.is_none() {
        ledger.interface_elapsed_s = Some(elapsed_s);
    }
    ledger.peak_heat_flux_w_m2 = ledger.peak_heat_flux_w_m2.max(heat_flux_w_m2);
    ledger.total_heat_j_m2 += heat_flux_w_m2 * step_s;
    if heat_flux_w_m2 > entry.maximum_heat_flux_w_m2 {
        return Err(format!(
            "Entry heat flux exceeded the authored thermal limit ({heat_flux_w_m2} > {} W/m²)",
            entry.maximum_heat_flux_w_m2
        ));
    }
    if ledger.total_heat_j_m2 > entry.maximum_total_heat_j_m2 {
        return Err(format!(
            "Entry total heat exceeded the authored thermal limit ({} > {} J/m²)",
            ledger.total_heat_j_m2, entry.maximum_total_heat_j_m2
        ));
    }
    Ok(())
}

pub fn context() -> serde_json::Value {
    serde_json::json!({
        "model":MODEL,
        "operator":"Stagnation-point convective heating q = k·sqrt(ρ/Rn)·v³ integrated on the accepted contact substeps of the existing bounded gear solver; freestream velocity excludes the parent's orbital translation and rigid spin",
        "atmosphere":"Existing bulk composition and surface temperature through an exponential height model ρ(h) = ρ0·exp(−h/H) off the parent's reference sphere; ρ0 = p0/(R·T0) and H = R·T0/g0 with a composition-weighted mean molar mass",
        "limits":"Authored bounds on stagnation heat flux and integrated fluence; exceeding a bound stops the craft atomically at the last supported macro state",
        "evidence":"Entry-interface crossing on the craft propulsion clock, peak heat flux, integrated fluence and observed substep count",
        "scope":SCOPE,
        "limits_list":["No radiative heating, ablation, wall-temperature feedback, catalytic surface, aerodynamic drag/lift or attitude controller","The heat-flux coefficient and nose radius are authored assumptions; the air correlation value does not transfer to other gas mixtures","The interface altitude is an authored evidence marker, not a physical boundary or certified entry corridor","H₂/He blend treated as 0.004 kg/mol; bulk composition is not a calibrated local atmosphere profile","Trajectory evidence is accepted-substep geometry, not telemetry or navigation truth"],
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

    /// A Mars gear craft above the authored entry interface with a coasting
    /// (zero-throttle) command and fast tangential motion for measurable
    /// stagnation heating.
    pub(crate) fn entry_world(entry: Entry) -> Universe {
        let mut world = Universe::new(crate::UniverseSettings {
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
        let clearance = 5_000.;
        let position = surface.position_m + surface.outward_normal * clearance;
        let tangential = Vec3d::new(-0.6, 0.8, 0.);
        let mut plan = Plan::example();
        plan.commands = vec![Command {
            duration_s: 5_000.,
            throttle: 0.,
            direction: [0., 0., 1.],
        }];
        // Interface three quarters of the way to the start altitude, so any
        // terrain elevation still leaves a measurable above→below crossing.
        let start_altitude = ((position - frame.centre_m).length() - frame.radius_m).max(1.);
        let mut entry = entry;
        entry.entry_interface_altitude_m = (start_altitude * 0.75).max(0.);
        let definition = Definition {
            model: crate::world_vehicle::CONTACT_MODEL.into(),
            frame: crate::world_vehicle::FRAME.into(),
            parent: target,
            plan,
            initial_relative_position_m: position - body.pos,
            initial_relative_velocity_m_s: tangential * 1_000. + outward * -80.,
            contact: Some(Hardware::example(&world, target, position).expect("gear is finite")),
            cargo_ledgers: false,
            cargo_removals: Vec::new(),
            handling: None,
            launch: None,
            entry: Some(entry),
            landing: None,
        };
        definition.validate(&world).expect("entry definition validates");
        world
            .apply_edit(Edit::CreateVehicle {
                definition: Box::new(definition),
            })
            .expect("entry craft starts above the terrain");
        world
    }

    #[test]
    fn entry_limits_validate_their_bounds() {
        let good = Entry::example();
        good.validate().unwrap();
        assert!(Entry {
            entry_interface_altitude_m: -1.,
            ..good
        }
        .validate()
        .is_err());
        assert!(Entry {
            nose_radius_m: f64::NAN,
            ..good
        }
        .validate()
        .is_err());
        assert!(Entry {
            heat_flux_coefficient: 1e6,
            ..good
        }
        .validate()
        .is_err());
        assert!(Entry {
            maximum_heat_flux_w_m2: 0.,
            ..good
        }
        .validate()
        .is_err());
        assert!(Entry {
            maximum_total_heat_j_m2: -1e9,
            ..good
        }
        .validate()
        .is_err());
    }

    #[test]
    fn entry_ledger_evidence_validates() {
        let mut ledger = Ledger::new();
        ledger.validate().unwrap();
        ledger.interface_elapsed_s = Some(-1.);
        assert!(ledger.validate().is_err());
        let mut ledger = Ledger::new();
        ledger.peak_heat_flux_w_m2 = -1.;
        assert!(ledger.validate().is_err());
        let mut ledger = Ledger::new();
        ledger.total_heat_j_m2 = f64::INFINITY;
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn entry_requires_gear_and_an_atmosphere() {
        let world = entry_world(Entry::example());
        let flight = &world.vehicles[0];
        assert!(flight.entry_ledger.is_some());

        let mut no_gear = flight.definition.clone();
        no_gear.contact = None;
        no_gear.model = crate::world_vehicle::MODEL.into();
        assert!(no_gear.validate_structure().is_err());

        // A vacuum parent is refused: there is no modeled atmosphere to heat in.
        let mut vacuum = flight.definition.clone();
        vacuum.parent = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Moon").unwrap() as u32,
        };
        assert!(vacuum.validate(&world).is_err());
    }

    #[test]
    fn entry_craft_records_interface_crossing_and_heating_evidence() {
        let mut world = entry_world(Entry::example());
        for _ in 0..25 {
            world.advance_to(world.time + 1., None, None);
        }
        let flight = &world.vehicles[0];
        let ledger = flight.entry_ledger.as_ref().expect("entry ledger");
        let crossing = ledger.interface_elapsed_s.expect("interface crossing evidence");
        assert!((0.0..25.).contains(&crossing), "crossing at {crossing}");
        assert!(ledger.peak_heat_flux_w_m2 > 100., "peak {}", ledger.peak_heat_flux_w_m2);
        assert!(ledger.total_heat_j_m2 > 0., "total {}", ledger.total_heat_j_m2);
        assert!(ledger.substeps_observed > 10);
        assert!(matches!(flight.phase, Phase::Flying));
    }

    #[test]
    fn entry_stops_atomically_when_the_thermal_limit_is_exceeded() {
        let mut world = entry_world(Entry {
            maximum_heat_flux_w_m2: 100.,
            ..Entry::example()
        });
        let mut stopped_at = None;
        for t in 0..30 {
            world.advance_to(world.time + 1., None, None);
            if matches!(world.vehicles[0].phase, Phase::Unsupported { .. }) {
                stopped_at = Some(t);
                break;
            }
        }
        let stopped_at = stopped_at.expect("craft stops on the thermal limit");
        assert!(stopped_at < 10, "stopped at {stopped_at} s");
        let reason = match &world.vehicles[0].phase {
            Phase::Unsupported { reason } => reason.clone(),
            other => panic!("expected Unsupported, got {other:?}"),
        };
        assert!(reason.contains("thermal limit"), "reason: {reason}");
    }

    #[test]
    fn entry_ledgers_replay_exactly_after_reopen() {
        let mut world = entry_world(Entry::example());
        for _ in 0..30 {
            world.advance_to(world.time + 1., None, None);
        }
        let expected = world.vehicles[0].entry_ledger;
        assert!(expected.as_ref().unwrap().peak_heat_flux_w_m2 > 0.);
        let json = crate::save::to_json(&world, "entry").unwrap();
        let mut tampered_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        tampered_value["universe"]["vehicles"][0]["entry_ledger"]
            .as_object_mut()
            .expect("ledger object")["total_heat_j_m2"] = serde_json::json!(-1.);
        let tampered = serde_json::to_string(&tampered_value).unwrap();
        assert!(crate::save::from_json(&tampered).is_err());
        let mut reopened = crate::save::from_json(&json).unwrap();
        for _ in 0..20 {
            world.advance_to(world.time + 1., None, None);
            reopened.advance_to(reopened.time + 1., None, None);
        }
        assert_eq!(
            reopened.vehicles[0].entry_ledger,
            world.vehicles[0].entry_ledger
        );
    }

    #[test]
    fn entry_context_declares_its_models_and_limits() {
        let value = context();
        assert_eq!(value["model"], MODEL);
        assert!(value["limits"].is_string());
        assert!(value["limits_list"].is_array());
    }
}
