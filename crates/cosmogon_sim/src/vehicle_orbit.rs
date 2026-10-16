//! Two-body osculating elements of retained free-flight states about the craft's
//! parent source, plus craft-to-craft rendezvous evidence over exactly shared
//! retained observation times. These observations do not change flight, grant
//! maneuverability, or claim a propagated orbit: every value is instantaneous
//! evidence computed from retained actual N-body states.

use crate::{
    world_vehicle::{Flight, GravitySample, Source},
    BodyRef, Universe, Vec3d,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MODEL: &str = "cosmogon-orbital-evidence/1";
pub const SCOPE: &str = "Instantaneous two-body osculating elements about the craft's parent body, evaluated from the retained actual N-body craft state, the retained parent gravity sample the craft actually felt, and the same parent ephemeris resolver the integrator uses; inclination is measured to the system inertial reference plane, not the parent's equator; evidence only, never a propagated orbit or navigation solution";

fn finite(v: Vec3d) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

/// Instantaneous two-body osculating elements of one retained craft state
/// relative to a parent gravitational source.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Elements {
    /// Two-body semi-major axis (m); negative for hyperbolic states.
    pub semi_major_axis_m: f64,
    pub eccentricity: f64,
    pub periapsis_m: f64,
    /// Absent for hyperbolic (e >= 1) states.
    pub apoapsis_m: Option<f64>,
    /// Absent for hyperbolic (e >= 1) states.
    pub period_s: Option<f64>,
    /// To the system inertial XY reference plane, not the parent equator.
    pub inclination_rad: f64,
    pub specific_energy_j_kg: f64,
    pub specific_angular_momentum_m2_s: f64,
}

/// Osculating elements of a parent-relative state vector under gravitational
/// parameter `mu`. Refuses non-finite, parabolic, radial and inside-zero inputs.
pub fn osculating(rel_pos: Vec3d, rel_vel: Vec3d, mu: f64) -> Result<Elements, String> {
    if !mu.is_finite() || mu <= 0. {
        return Err(
            "Osculating elements require a positive finite gravitational parameter".into(),
        );
    }
    if !finite(rel_pos) || !finite(rel_vel) {
        return Err("Osculating elements require finite parent-relative state vectors".into());
    }
    let radius = rel_pos.length();
    if !radius.is_finite() || radius <= 0. {
        return Err("Osculating elements require a finite nonzero parent-relative radius".into());
    }
    let speed2 = rel_vel.dot(rel_vel);
    let energy = 0.5 * speed2 - mu / radius;
    if !energy.is_finite() || energy == 0. {
        return Err("Osculating elements require a finite nonparabolic specific energy".into());
    }
    let semi_major_axis = -mu / (2. * energy);
    let e_vec = (rel_pos * (speed2 - mu / radius) - rel_vel * rel_pos.dot(rel_vel)) * (1. / mu);
    let eccentricity = e_vec.length();
    if !eccentricity.is_finite() || !semi_major_axis.is_finite() {
        return Err("Osculating elements are outside supported finite bounds".into());
    }
    let angular_momentum = rel_pos.cross(rel_vel);
    let h_magnitude = angular_momentum.length();
    if !h_magnitude.is_finite() || h_magnitude <= 0. {
        return Err("Osculating inclination requires a nonzero specific angular momentum".into());
    }
    let inclination = (angular_momentum.z / h_magnitude).clamp(-1., 1.).acos();
    let periapsis = semi_major_axis * (1. - eccentricity);
    if !periapsis.is_finite() || periapsis < 0. {
        return Err("Osculating periapsis is outside supported finite bounds".into());
    }
    let (apoapsis, period) = if eccentricity < 1. {
        if semi_major_axis <= 0. {
            return Err("Osculating closed elements require a positive semi-major axis".into());
        }
        let apoapsis = semi_major_axis * (1. + eccentricity);
        let period = std::f64::consts::TAU * (semi_major_axis * semi_major_axis * semi_major_axis / mu).sqrt();
        if !apoapsis.is_finite() || !period.is_finite() {
            return Err("Osculating closed elements are outside supported finite bounds".into());
        }
        (Some(apoapsis), Some(period))
    } else {
        (None, None)
    };
    Ok(Elements {
        semi_major_axis_m: semi_major_axis,
        eccentricity,
        periapsis_m: periapsis,
        apoapsis_m: apoapsis,
        period_s: period,
        inclination_rad: inclination,
        specific_energy_j_kg: energy,
        specific_angular_momentum_m2_s: h_magnitude,
    })
}

/// Osculating elements of a retained craft state about its parent body. The
/// parent position is the retained gravity sample the craft actually felt;
/// the parent velocity comes from the same body-state resolver at the craft's
/// retained state time. The craft state itself is never re-integrated.
pub fn observe(world: &Universe, flight: &Flight) -> Result<Elements, String> {
    let parent = flight.definition.parent;
    let sample: &GravitySample = flight
        .last_sources
        .iter()
        .find(|s| s.source == Source::Body(parent.body))
        .ok_or("Parent body absent from retained craft gravity sources")?;
    let system = world
        .systems
        .get(parent.system as usize)
        .ok_or("Craft parent system is absent")?;
    let body = system
        .bodies
        .get(parent.body as usize)
        .ok_or("Craft parent body is absent")?;
    if !body.exists() {
        return Err("Craft parent body no longer exists".into());
    }
    if sample.mu_m3_s2 != body.mu() || sample.radius_m != body.radius {
        return Err(
            "Parent body radius or mass differs from the retained craft gravity source".into(),
        );
    }
    if !flight.last_time_s.is_finite() {
        return Err("Craft state clock is not finite".into());
    }
    let centre = system.body_state(parent.body as usize, flight.last_time_s);
    if !finite(centre.vel) {
        return Err("Parent body ephemeris velocity is not finite".into());
    }
    osculating(
        flight.position_m - sample.position_m,
        flight.velocity_m_s - centre.vel,
        sample.mu_m3_s2,
    )
}

/// One retained observation's osculating elements at its exact world time.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrbitSample {
    pub world_time_s: f64,
    pub elements: Elements,
}

/// Exact deterministic CSV of retained osculating evidence; hyperbolic
/// apoapsis and period fields are blank, never zero.
pub fn csv(samples: &[OrbitSample]) -> String {
    let mut output = String::from("observation_index,world_time_s,semi_major_axis_m,eccentricity,periapsis_m,apoapsis_m,period_s,inclination_rad,specific_energy_j_kg,specific_angular_momentum_m2_s\n");
    let empty = String::new();
    for (index, sample) in samples.iter().enumerate() {
        let e = &sample.elements;
        let apoapsis = e
            .apoapsis_m
            .map(|v| v.to_string())
            .unwrap_or_else(|| empty.clone());
        let period = e
            .period_s
            .map(|v| v.to_string())
            .unwrap_or_else(|| empty.clone());
        output.push_str(
            &[
                index.to_string(),
                sample.world_time_s.to_string(),
                e.semi_major_axis_m.to_string(),
                e.eccentricity.to_string(),
                e.periapsis_m.to_string(),
                apoapsis,
                period,
                e.inclination_rad.to_string(),
                e.specific_energy_j_kg.to_string(),
                e.specific_angular_momentum_m2_s.to_string(),
            ]
            .join(","),
        );
        output.push('\n');
    }
    output
}

fn elements_json(e: &Elements) -> Value {
    json!({
        "semi_major_axis_m": e.semi_major_axis_m,
        "eccentricity": e.eccentricity,
        "periapsis_m": e.periapsis_m,
        "apoapsis_m": e.apoapsis_m,
        "period_s": e.period_s,
        "inclination_rad": e.inclination_rad,
        "specific_energy_j_kg": e.specific_energy_j_kg,
        "specific_angular_momentum_m2_s": e.specific_angular_momentum_m2_s,
    })
}

/// Conditional study-manifest summary of a free-flight recording's osculating
/// evidence: first/last elements, running minimum periapsis, maximum apoapsis
/// over closed samples, and whether the final state remains closed.
pub fn summary(parent: BodyRef, samples: &[OrbitSample]) -> Result<Value, String> {
    let first = samples
        .first()
        .ok_or("Orbital evidence requires at least one retained observation")?;
    let last = samples
        .last()
        .ok_or("Orbital evidence requires at least one retained observation")?;
    let minimum_periapsis = samples
        .iter()
        .map(|s| s.elements.periapsis_m)
        .fold(f64::INFINITY, f64::min);
    let maximum_apoapsis = samples
        .iter()
        .filter_map(|s| s.elements.apoapsis_m)
        .fold(None, |acc: Option<f64>, v| {
            Some(acc.map_or(v, |a| a.max(v)))
        });
    if !minimum_periapsis.is_finite() {
        return Err("Orbital summary requires finite retained periapsis evidence".into());
    }
    Ok(json!({
        "model": MODEL,
        "parent": {"system": parent.system, "body": parent.body},
        "scope": SCOPE,
        "observations": samples.len(),
        "initial": elements_json(&first.elements),
        "final": elements_json(&last.elements),
        "minimum_periapsis_m": minimum_periapsis,
        "maximum_apoapsis_m": maximum_apoapsis,
        "closed_at_final": last.elements.apoapsis_m.is_some(),
    }))
}

pub fn context() -> Value {
    json!({
        "model": MODEL,
        "operator": "Two-body osculating elements from each retained craft state relative to its parent source: parent position from the retained gravity sample the craft actually felt; parent velocity from the same body-state resolver at the craft's retained state time; the craft state is never re-integrated",
        "evidence": "Per-observation osculating element CSV plus a conditional study-manifest summary block for free-flight studies without authored contact hardware",
        "scope": SCOPE,
        "limits_list": [
            "Osculating elements describe one instant of an actual N-body trajectory; they are not integrated forward and say nothing about future position",
            "Inclination is relative to the system inertial XY plane, not the parent body's equator or an ecliptic plane",
            "Hyperbolic states have no apoapsis and no period; those CSV fields are blank, never zero",
            "No maneuverability, fuel, navigation, stationkeeping, rendezvous or mission-feasibility claim is granted by the element evidence"
        ],
        "empirical_validation": null,
        "uncertainty": null
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_osculating_elements_are_exact_and_deterministic() {
        let mu = 4.282837e13_f64;
        let radius = 3.4e6_f64;
        let speed = (mu / radius).sqrt();
        let pos = Vec3d::new(radius, 0., 0.);
        let vel = Vec3d::new(0., speed, 0.);
        let e = osculating(pos, vel, mu).unwrap();
        assert!(e.eccentricity < 1e-9, "{:?}", e);
        assert!((e.semi_major_axis_m - radius).abs() < 1e-3, "{:?}", e);
        assert!((e.periapsis_m - radius).abs() < 1e-3, "{:?}", e);
        assert!((e.apoapsis_m.unwrap() - radius).abs() < 1e-3, "{:?}", e);
        let period = std::f64::consts::TAU * (radius * radius * radius / mu).sqrt();
        assert!((e.period_s.unwrap() - period).abs() < 1e-6, "{:?}", e);
        assert!(e.inclination_rad.abs() < 1e-12, "{:?}", e);
        assert!((e.specific_energy_j_kg + mu / (2. * radius)).abs() < 1e-6, "{:?}", e);
        assert!((e.specific_angular_momentum_m2_s - radius * speed).abs() < 1e-6, "{:?}", e);
        assert_eq!(e, osculating(pos, vel, mu).unwrap());
    }

    #[test]
    fn elliptical_and_hyperbolic_states_report_their_supported_elements() {
        let mu = 4.282837e13_f64;
        let radius = 4.4e6_f64;
        let apoapsis_speed = (mu / radius * 0.5).sqrt();
        let e = osculating(
            Vec3d::new(radius, 0., 0.),
            Vec3d::new(0., apoapsis_speed, 0.),
            mu,
        )
        .unwrap();
        assert!(e.eccentricity > 0.2 && e.eccentricity < 1., "{:?}", e);
        assert!(e.semi_major_axis_m < radius, "{:?}", e);
        assert!((e.apoapsis_m.unwrap() - radius).abs() < 1e-3, "{:?}", e);
        assert!(e.apoapsis_m.unwrap() > e.periapsis_m, "{:?}", e);
        assert!(e.period_s.unwrap().is_finite(), "{:?}", e);
        let escape = (2. * mu / radius).sqrt() * 1.2;
        let h = osculating(
            Vec3d::new(radius, 0., 0.),
            Vec3d::new(0., escape, 0.),
            mu,
        )
        .unwrap();
        assert!(h.eccentricity > 1., "{:?}", h);
        assert!(h.semi_major_axis_m < 0., "{:?}", h);
        assert!(h.apoapsis_m.is_none() && h.period_s.is_none(), "{:?}", h);
        assert!(
            h.periapsis_m > 0. && (h.periapsis_m - radius).abs() < 1e-3,
            "{:?}",
            h
        );
    }

    #[test]
    fn polar_inclination_and_unsupported_inputs_are_refused() {
        let mu = 4.282837e13_f64;
        let radius = 3.4e6_f64;
        let speed = (mu / radius).sqrt();
        let polar = osculating(
            Vec3d::new(radius, 0., 0.),
            Vec3d::new(0., 0., speed),
            mu,
        )
        .unwrap();
        assert!((polar.inclination_rad - std::f64::consts::FRAC_PI_2).abs() < 1e-9, "{:?}", polar);
        let radial = Vec3d::new(speed, 0., 0.);
        assert!(osculating(Vec3d::new(radius, 0., 0.), radial, mu)
            .unwrap_err()
            .contains("angular momentum"));
        assert!(osculating(Vec3d::new(radius, 0., 0.), radial, 0.)
            .unwrap_err()
            .contains("gravitational parameter"));
        assert!(osculating(Vec3d::ZERO, radial, mu)
            .unwrap_err()
            .contains("radius"));
        assert!(osculating(
            Vec3d::new(f64::NAN, 0., 0.),
            radial,
            mu
        )
        .unwrap_err()
        .contains("finite"));
    }

    #[test]
    fn observe_reports_the_authored_circular_flight_and_refuses_missing_sources() {
        let u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let parent = crate::BodyRef { system: 0, body: 3 };
        let mut plan = crate::supply_vehicle::Plan::example();
        plan.commands = vec![crate::supply_vehicle::Command {
            duration_s: 10.,
            throttle: 0.,
            direction: [1., 0., 0.],
        }];
        let d = crate::world_vehicle::Definition::circular_example(&u, parent, plan).unwrap();
        let flight = crate::world_vehicle::Flight::new(0, d, &u).unwrap();
        let elements = observe(&u, &flight).unwrap();
        assert!(elements.eccentricity < 1e-9, "{:?}", elements);
        let body = &u.systems[0].bodies[3];
        let radius = body.radius + 200_000.;
        assert!((elements.semi_major_axis_m - radius).abs() < 1e-3, "{:?}", elements);
        let period = std::f64::consts::TAU * (radius * radius * radius / body.mu()).sqrt();
        assert!((elements.period_s.unwrap() - period).abs() < 1e-4, "{:?}", elements);
        assert_eq!(elements, observe(&u, &flight).unwrap());
        let mut orphan = flight.clone();
        orphan.last_sources.retain(|s| s.source != Source::Body(3));
        assert!(observe(&u, &orphan)
            .unwrap_err()
            .contains("absent from retained craft gravity sources"));
    }

    #[test]
    fn csv_blanks_hyperbolic_apoapsis_and_repeats_exactly() {
        let mu = 4.282837e13_f64;
        let radius = 3.4e6_f64;
        let closed = osculating(
            Vec3d::new(radius, 0., 0.),
            Vec3d::new(0., (mu / radius).sqrt(), 0.),
            mu,
        )
        .unwrap();
        let open = osculating(
            Vec3d::new(radius, 0., 0.),
            Vec3d::new(0., (2. * mu / radius).sqrt() * 1.3, 0.),
            mu,
        )
        .unwrap();
        assert!(open.apoapsis_m.is_none());
        let samples = vec![
            OrbitSample { world_time_s: 0., elements: closed },
            OrbitSample { world_time_s: 120., elements: open },
        ];
        let first = csv(&samples);
        assert_eq!(first, csv(&samples));
        let closed_line = first.lines().nth(1).unwrap();
        let closed_fields: Vec<&str> = closed_line.split(',').collect();
        assert!((closed_fields[2].parse::<f64>().unwrap() - 3.4e6).abs() < 1e-3);
        assert!(!closed_fields[5].is_empty() && !closed_fields[6].is_empty());
        let hyperbolic_line = first.lines().nth(2).unwrap();
        let fields: Vec<&str> = hyperbolic_line.split(',').collect();
        assert_eq!(fields[5], "");
        assert_eq!(fields[6], "");
    }

    #[test]
    fn summary_reports_minimum_periapsis_closure_and_maximum_apoapsis() {
        let parent = crate::BodyRef { system: 0, body: 3 };
        let mu = 4.282837e13_f64;
        let closed = osculating(
            Vec3d::new(3.4e6, 0., 0.),
            Vec3d::new(0., (mu / 3.4e6).sqrt(), 0.),
            mu,
        )
        .unwrap();
        let open = osculating(
            Vec3d::new(4.0e6, 0., 0.),
            Vec3d::new(0., (2. * mu / 4.0e6).sqrt() * 1.1, 0.),
            mu,
        )
        .unwrap();
        let value = summary(
            parent,
            &[
                OrbitSample { world_time_s: 0., elements: closed },
                OrbitSample { world_time_s: 60., elements: open },
            ],
        )
        .unwrap();
        assert_eq!(value["model"], MODEL);
        assert_eq!(value["observations"], 2);
        assert_eq!(value["closed_at_final"], false);
        assert_eq!(value["maximum_apoapsis_m"], json!(closed.apoapsis_m));
        assert!((value["minimum_periapsis_m"].as_f64().unwrap() - closed.periapsis_m).abs() < 1e-9);
        assert!(summary(parent, &[]).unwrap_err().contains("at least one"));
    }
}
