//! Opt-in supported surface-operation window evaluated from actual contact-solver states.
//!
//! This is a modeled envelope, not proof of continuous real hardware stability: every
//! evidence sample is a numerical solver state (an accepted explicit-midpoint midpoint
//! or a committed substep endpoint) checked against an authored support envelope.
//! `current_window_s` is the only current permission signal; `longest_window_s` is
//! historical evidence and never grants eligibility. The ledger participates in the
//! atomic one-second macro preview, so any solver failure rolls it back with the rest
//! of the step, and any excluded sample resets the current window to zero.

use serde::{Deserialize, Serialize};

use crate::{universe::Universe, vehicle_support, world_vehicle::Flight};

/// Versioned opt-in model for solver-integrated operation-window accounting.
pub const MODEL: &str = "cosmogon-vehicle-surface-operation/1";
/// Bounded stored exclusion reason; diagnostics stay finite and comparable.
const MAX_EXCLUSION_REASON_CHARS: usize = 240;

/// Authored envelope and window requirements for counting supported operation time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    /// The same authored footprint/hold envelope used by the instantaneous support reading.
    pub support: vehicle_support::Definition,
    /// Continuous currently-supported duration a future consumer may require.
    pub required_window_s: f64,
    /// Largest tolerated gap between consecutive evidence samples inside one window.
    pub maximum_sample_gap_s: f64,
    pub maximum_structural_utilization: f64,
    pub maximum_stroke_utilization: f64,
    pub maximum_friction_utilization: f64,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.support.validate()?;
        if self.model != MODEL {
            return Err("Unsupported surface-operation model version".into());
        }
        if !self.required_window_s.is_finite()
            || !(0.0..=1.0e7).contains(&self.required_window_s)
            || !self.maximum_sample_gap_s.is_finite()
            || !(0.0..=1.000001).contains(&self.maximum_sample_gap_s)
            || !self.maximum_structural_utilization.is_finite()
            || !(0.0..=100.).contains(&self.maximum_structural_utilization)
            || !self.maximum_stroke_utilization.is_finite()
            || !(0.0..=100.).contains(&self.maximum_stroke_utilization)
            || !self.maximum_friction_utilization.is_finite()
            || !(0.0..=100.).contains(&self.maximum_friction_utilization)
        {
            return Err("Surface-operation window and utilization bounds require finite supported values".into());
        }
        Ok(())
    }
}

/// Running window evidence recorded from actual accepted solver samples.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub model: String,
    /// Craft clock of the first evaluated sample after this ledger was created.
    pub origin_time_s: f64,
    pub last_sample_time_s: f64,
    /// Continuous currently-supported duration ending at `last_sample_time_s`.
    pub current_window_s: f64,
    pub current_window_origin_s: f64,
    /// Historical longest continuous window. Evidence only; never current permission.
    pub longest_window_s: f64,
    pub longest_window_origin_s: f64,
    pub samples: u64,
    pub supported_samples: u64,
    pub exclusion_samples: u64,
    /// Whether the previous accepted sample was supported; windows extend only across
    /// consecutive supported samples.
    pub last_sample_supported: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_exclusion: Option<String>,
}
impl Ledger {
    fn first(sample_time_s: f64) -> Self {
        Self {
            model: MODEL.into(),
            origin_time_s: sample_time_s,
            last_sample_time_s: sample_time_s,
            current_window_s: 0.,
            current_window_origin_s: sample_time_s,
            longest_window_s: 0.,
            longest_window_origin_s: sample_time_s,
            samples: 0,
            supported_samples: 0,
            exclusion_samples: 0,
            last_sample_supported: false,
            last_exclusion: None,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let times = [
            self.origin_time_s,
            self.last_sample_time_s,
            self.current_window_s,
            self.current_window_origin_s,
            self.longest_window_s,
            self.longest_window_origin_s,
        ];
        if self.model != MODEL
            || times.iter().any(|v| !v.is_finite())
            || self.origin_time_s > self.last_sample_time_s
            || !(self.origin_time_s..=self.last_sample_time_s)
                .contains(&self.current_window_origin_s)
            || !(self.origin_time_s..=self.last_sample_time_s)
                .contains(&self.longest_window_origin_s)
            || self.current_window_s < 0.
            || self.longest_window_s < self.current_window_s
            || self.samples == 0
            || self.supported_samples + self.exclusion_samples != self.samples
            || self
                .last_exclusion
                .as_ref()
                .is_some_and(|r| r.is_empty() || r.len() > MAX_EXCLUSION_REASON_CHARS)
        {
            return Err("Surface-operation ledger counters, clocks or windows are inconsistent".into());
        }
        Ok(())
    }
}

fn clamp_reason(reason: String) -> String {
    // Commas would break the flat recording CSVs; keep stored reasons CSV-safe.
    let reason = reason.replace(',', ";");
    if reason.len() <= MAX_EXCLUSION_REASON_CHARS {
        reason
    } else {
        let mut cut = MAX_EXCLUSION_REASON_CHARS;
        while !reason.is_char_boundary(cut) {
            cut -= 1;
        }
        reason[..cut].to_string()
    }
}

/// Evaluate one actual solver state against the authored envelope and extend or reset
/// the flight's opt-in operation ledger. `flight.last_time_s` is the sample's craft clock.
pub fn observe(world: &Universe, flight: &mut Flight, d: &Definition) -> Result<(), String> {
    d.validate()?;
    let authored = flight
        .definition
        .contact
        .as_ref()
        .and_then(|h| h.operation.as_ref())
        .ok_or("Operation sampling requires authored contact operation hardware")?;
    if authored != d {
        return Err("Sampled operation definition differs from the authored hardware".into());
    }
    if flight.contact.is_none() {
        return Err("Operation sampling requires retained rigid contact state".into());
    }
    let t = flight.last_time_s;
    if !t.is_finite() {
        return Err("Operation sample craft clock is not finite".into());
    }
    let reading = vehicle_support::Reading::from_flight(world, flight, &d.support)?;
    let mut reason: Option<String> = None;
    let mut exclude = |candidate: String| {
        if reason.is_none() {
            reason = Some(candidate);
        }
    };
    for line in &reading.envelope_exclusions {
        exclude(line.clone());
    }
    for foot in &reading.feet {
        if foot.structural_utilization > d.maximum_structural_utilization {
            exclude("Foot structural utilization exceeds the operation envelope".into());
        }
        if foot.stroke_utilization > d.maximum_stroke_utilization {
            exclude("Foot stroke utilization exceeds the operation envelope".into());
        }
        if foot
            .friction_utilization
            .is_some_and(|u| u > d.maximum_friction_utilization)
        {
            exclude("Foot friction utilization exceeds the operation envelope".into());
        }
    }
    let ledger = match flight.operation_ledger.take() {
        Some(mut ledger) => {
            ledger.validate()?;
            // Both clocks are validated finite above, so `<=` is exact here.
            if t <= ledger.last_sample_time_s {
                return Err("Operation samples require a strictly increasing craft clock".into());
            }
            let gap = t - ledger.last_sample_time_s;
            let mut previous_supported = ledger.last_sample_supported;
            if gap > d.maximum_sample_gap_s {
                ledger.current_window_s = 0.;
                ledger.current_window_origin_s = t;
                previous_supported = false;
            }
            ledger.last_sample_time_s = t;
            ledger.samples += 1;
            match reason {
                Some(reason) => {
                    ledger.exclusion_samples += 1;
                    ledger.current_window_s = 0.;
                    ledger.current_window_origin_s = t;
                    ledger.last_sample_supported = false;
                    ledger.last_exclusion = Some(clamp_reason(reason));
                }
                None => {
                    ledger.supported_samples += 1;
                    if previous_supported {
                        ledger.current_window_s += gap;
                        ledger.last_exclusion = None;
                        ledger.last_sample_supported = true;
                        if ledger.current_window_s > ledger.longest_window_s {
                            ledger.longest_window_s = ledger.current_window_s;
                            ledger.longest_window_origin_s = ledger.current_window_origin_s;
                        }
                    } else {
                        ledger.current_window_s = 0.;
                        ledger.current_window_origin_s = t;
                        ledger.last_exclusion = None;
                        ledger.last_sample_supported = true;
                    }
                }
            }
            ledger
        }
        None => {
            let mut ledger = Ledger::first(t);
            ledger.samples = 1;
            if let Some(reason) = reason {
                ledger.exclusion_samples = 1;
                ledger.last_exclusion = Some(clamp_reason(reason));
            } else {
                ledger.supported_samples = 1;
                ledger.last_sample_supported = true;
            }
            ledger
        }
    };
    ledger.validate()?;
    flight.operation_ledger = Some(ledger);
    Ok(())
}

/// Documented model scope for native/CLI research surfaces.
pub fn context() -> serde_json::Value {
    serde_json::json!({
        "model": MODEL,
        "unit": "s",
        "meaning": "Continuous currently-supported surface-operation window evaluated from actual accepted contact-solver midpoint and endpoint states",
        "limits": [
            "Modeled numerical envelope at solver sample clocks, not proof of continuous real hardware stability",
            "Longest historical window is evidence only and never grants current permission",
            "Any excluded sample resets the current window; solver failures roll the ledger back with the whole one-second macro step",
            "No attachment, phase change, cargo handling, storage, delivery or hardware thermal/power calibration",
        ],
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
    use crate::{universe::UniverseSettings, BodyRef, Scenario, Vec3d};

    fn operation_definition() -> Definition {
        Definition {
            model: MODEL.into(),
            support: vehicle_support::Definition::default(),
            required_window_s: 5.,
            maximum_sample_gap_s: 1.000001,
            maximum_structural_utilization: 1.,
            maximum_stroke_utilization: 1.,
            maximum_friction_utilization: 1.,
        }
    }

    fn setup(op: &Definition, altitude: f64, descent: f64) -> (Universe, FlightDefinition) {
        setup_with(op, altitude, descent, true)
    }
    fn setup_with(
        op: &Definition,
        altitude: f64,
        descent: f64,
        shear: bool,
    ) -> (Universe, FlightDefinition) {
        let world = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = BodyRef {
            system: 0,
            body: world.systems[0].find_body("Mars").unwrap() as u32,
        };
        let frame = SurfaceFrame::at(&world, target, world.time).unwrap();
        let direction = Vec3d::new(0.8, 0.6, 0.);
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
            duration_s: 60.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        let body = world.systems[0].body_state(target.body as usize, world.time);
        let mut hardware = Hardware::example(&world, target, position).unwrap();
        if shear {
            hardware = hardware
                .with_shear(crate::vehicle_shear::Definition::example())
                .unwrap();
        }
        hardware.operation = Some(op.clone());
        let d = FlightDefinition {
            model: if shear {
                crate::world_vehicle::SHEAR_MODEL.into()
            } else {
                crate::world_vehicle::CONTACT_MODEL.into()
            },
            frame: crate::world_vehicle::FRAME.into(),
            parent: target,
            plan,
            contact: Some(hardware),
            initial_relative_position_m: position - body.pos,
            initial_relative_velocity_m_s: velocity - body.vel,
            cargo_ledgers: false,
            cargo_removals: vec![],
            handling: None,
            launch: None,
        };
        d.validate(&world).unwrap();
        (world, d)
    }

    #[test]
    fn definition_and_ledger_bounds_are_explicit() {
        assert!(operation_definition().validate().is_ok());
        let mut bad = operation_definition();
        bad.required_window_s = f64::NAN;
        assert!(bad.validate().is_err());
        let mut bad = operation_definition();
        bad.maximum_sample_gap_s = 2.;
        assert!(bad.validate().is_err());
        let mut ledger = Ledger::first(10.);
        ledger.samples = 1;
        ledger.exclusion_samples = 1;
        assert!(ledger.validate().is_ok());
        ledger.current_window_s = 1.;
        assert!(ledger.validate().is_err());
        let mut ledger = Ledger::first(10.);
        ledger.samples = 2;
        ledger.supported_samples = 1;
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn resting_craft_accumulates_and_resets_the_current_window() {
        let op = operation_definition();
        let (mut world, d) = setup(&op, 1.6, 0.1);
        world.apply_edit(Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        world.advance_to(world.time + 8., None, None);
        let flight = &world.vehicles[0];
        assert_eq!(flight.phase, crate::world_vehicle::Phase::Flying);
        let ledger = flight.operation_ledger.as_ref().expect("ledger");
        assert!(ledger.supported_samples > 0, "{ledger:?}");
        assert!(ledger.current_window_s > 0., "{ledger:?}");
        assert!(ledger.longest_window_s >= ledger.current_window_s);
        assert_eq!(
            ledger.samples,
            ledger.supported_samples + ledger.exclusion_samples
        );
        world.validate_vehicle_state().unwrap();
        // A directly observed over-speed state is an exclusion that resets the current
        // window while the historical longest window is retained as evidence only.
        let mut fast = flight.clone();
        fast.last_time_s += 1e-3;
        fast.velocity_m_s += flight.position_m.normalize() * 100.;
        observe(&world, &mut fast, &op).expect("over-speed sample");
        let after = fast.operation_ledger.as_ref().unwrap();
        assert_eq!(after.current_window_s, 0.);
        assert!(after.longest_window_s >= ledger.current_window_s);
        assert_eq!(after.exclusion_samples, ledger.exclusion_samples + 1);
        assert!(after.last_exclusion.is_some());
    }

    #[test]
    fn same_clock_is_refused_and_definition_mismatch_is_refused() {
        let op = operation_definition();
        let (mut world, d) = setup(&op, 1.6, 0.1);
        world.apply_edit(Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        world.advance_to(world.time + 3., None, None);
        let flight = &world.vehicles[0];
        assert!(flight.operation_ledger.is_some());
        let mut cloned = flight.clone();
        assert!(observe(&world, &mut cloned, &op).is_err());
        let mut other = op.clone();
        other.required_window_s = 1.;
        assert!(observe(&world, &mut cloned, &other).is_err());
        cloned.definition.contact.as_mut().unwrap().operation = None;
        assert!(observe(&world, &mut cloned, &op).is_err());
    }

    #[test]
    fn solver_failure_rolls_back_the_ledger_with_the_whole_macro_step() {
        let op = operation_definition();
        let (mut world, mut d) = setup_with(&op, 1.6, 0.1, false);
        for foot in &mut d.contact.as_mut().unwrap().feet {
            foot.response.sliding_friction = 10.;
            foot.response.slip_speed_scale_m_s = 1e-6;
        }
        world.apply_edit(Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        let prior = world.vehicles[0].clone();
        world.advance_to(world.time + 1., None, None);
        let failed = &world.vehicles[0];
        assert!(matches!(
            failed.phase,
            crate::world_vehicle::Phase::Unsupported { .. }
        ));
        assert_eq!(failed.operation_ledger, prior.operation_ledger);
        assert_eq!(failed.position_m, prior.position_m);
        assert_eq!(failed.contact, prior.contact);
    }

    #[test]
    fn save_roundtrip_pins_the_operation_model_and_refuses_its_removal() {
        let op = operation_definition();
        let (mut world, d) = setup(&op, 1.6, 0.1);
        world.apply_edit(Edit::CreateVehicle {
            definition: Box::new(d),
        })
        .unwrap();
        world.advance_to(world.time + 4., None, None);
        let json = crate::save::to_json(&world, "operation").unwrap();
        assert!(json.contains("cosmogon-vehicle-surface-operation"));
        assert!(json.contains(MODEL));
        let reopened = crate::save::from_json(&json).unwrap();
        assert_eq!(
            reopened.vehicles[0].operation_ledger,
            world.vehicles[0].operation_ledger
        );
        let mut stripped: serde_json::Value = serde_json::from_str(&json).unwrap();
        stripped["header"]["data_versions"]
            .as_object_mut()
            .unwrap()
            .remove("cosmogon-vehicle-surface-operation");
        assert!(crate::save::from_json(&stripped.to_string()).is_err());
    }

    #[test]
    fn recording_exports_and_protects_the_operation_window_csv() {
        let op = operation_definition();
        let (world, d) = setup(&op, 1.6, 0.1);
        let study = crate::world_vehicle_recording::Study::new(&world, d, 8, 1.).unwrap();
        let root = std::env::temp_dir()
            .join(format!("cosmogon-operation-recording-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let bundle = crate::world_vehicle_recording::Bundle::run(&world, study).unwrap();
        bundle.export(&root).unwrap();
        assert!(bundle.manifest["operation_window"].is_object());
        assert!(bundle.manifest["operation_window"]["current_window_s"].is_number());
        let csv = std::fs::read_to_string(root.join("operation.csv")).unwrap();
        assert!(csv.starts_with("world_time_s,craft_time_s,current_window_s"));
        let reopened = crate::world_vehicle_recording::Bundle::read(&root).unwrap();
        assert_eq!(reopened.manifest, bundle.manifest);
        let mut altered = csv.clone();
        altered.push_str("0,0,1,1,0,0,1,1,0,false,\n");
        std::fs::write(root.join("operation.csv"), altered).unwrap();
        assert!(crate::world_vehicle_recording::Bundle::read(&root).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn legacy_flights_keep_an_absent_ledger_and_unchanged_pins() {
        let (mut world, d) = setup(&operation_definition(), 1.6, 0.1);
        world
            .apply_edit(Edit::CreateVehicle {
                definition: {
                    let mut d = d;
                    d.contact.as_mut().unwrap().operation = None;
                    Box::new(d)
                },
            })
            .unwrap();
        world.advance_to(world.time + 2., None, None);
        assert!(world.vehicles[0].operation_ledger.is_none());
        let json = crate::save::to_json(&world, "legacy").unwrap();
        assert!(!json.contains("cosmogon-vehicle-surface-operation"));
    }
}
