//! Measured regression fixtures, distinct from blanket claims of scientific accuracy.
use crate::{
    astro::{self, dynamics::PhysicsPreset, horizons, sol},
    save, Vec3d,
};
use cosmogon_physics::nbody::{self, Particle, Settings};
use serde::Serialize;

mod reference {
    include!("../../data/horizons_validation_2027.rs");
}

#[derive(Clone, Debug, Serialize)]
pub struct ValidationMetric {
    pub quantity: String,
    pub value: f64,
    pub unit: String,
    pub regression_limit: f64,
    pub passes_regression: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct ValidationCase {
    pub id: String,
    pub name: String,
    pub model: String,
    pub inputs: serde_json::Value,
    pub reference: String,
    pub metrics: Vec<ValidationMetric>,
    pub limitations: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct ValidationReport {
    pub format: String,
    pub schema_version: u32,
    pub app_version: String,
    pub physics_engine: String,
    pub cases: Vec<ValidationCase>,
    pub passes_regression: bool,
    pub limitations: Vec<String>,
}
fn metric(quantity: impl Into<String>, value: f64, unit: &str, limit: f64) -> ValidationMetric {
    ValidationMetric {
        quantity: quantity.into(),
        value,
        unit: unit.into(),
        regression_limit: limit,
        passes_regression: value.is_finite() && value <= limit,
    }
}

/// Fixtures use local copies and no external download or current live-universe state.
pub fn validation_report() -> ValidationReport {
    let mut cases = Vec::new();
    let gm = astro::G * astro::SOLAR_MASS;
    let radius = astro::AU;
    let speed = (gm / radius).sqrt();
    let period = std::f64::consts::TAU * radius * (radius / gm).sqrt();
    for steps in [1000, 2000] {
        let mut particles = vec![
            Particle::new(Vec3d::ZERO, Vec3d::ZERO, gm, 0.0),
            Particle::new(
                Vec3d::new(radius, 0.0, 0.0),
                Vec3d::new(0.0, speed, 0.0),
                0.0,
                0.0,
            ),
        ];
        let dt = period / steps as f64;
        let energy0 = 0.5 * speed * speed - gm / radius;
        let h0 = radius * speed;
        for k in 0..steps {
            nbody::step(&mut particles, k as f64 * dt, dt, &Settings::default());
        }
        let p = particles[1];
        let energy1 = 0.5 * p.vel.length_squared() - gm / p.pos.length();
        cases.push(ValidationCase{id:format!("circular-orbit-{steps}"),name:format!("Circular test-particle orbit · {steps} steps"),model:"Newtonian point-mass, Yoshida-4, relativity off".into(),
            inputs:serde_json::json!({"central_gm_m3_s2":gm,"initial_radius_m":radius,"initial_speed_m_s":speed,"period_s":period,"macro_steps":steps,"dt_s":dt,"eta":Settings::default().eta,"particle_count":2,"orbiting_particle_gm":0,"collision_radii_m":0}),
            reference:"Analytic circular two-body solution; no observed ephemeris fitted".into(),
            metrics:vec![metric("return_position_error",(p.pos-Vec3d::new(radius,0.0,0.0)).length()/radius,"fraction of initial radius",if steps==2000{1e-9}else{2e-8}),metric("specific_energy_change",(energy1-energy0).abs()/energy0.abs(),"fraction of |initial specific energy|",1e-11),metric("specific_angular_momentum_change",(p.pos.cross(p.vel).length()-h0).abs()/h0,"fraction of initial specific angular momentum",1e-11)],
            limitations:"A simple fixed-mass circular fixture. These regression limits do not bound eccentric, encounter, many-body or dissipative regimes. Numerical agreement does not validate missing physical forces.".into()});
    }
    let mut sys = sol::sol_system();
    let preset = PhysicsPreset::Accurate;
    let epoch = horizons::apply_to_sol(&mut sys, preset.settings());
    let end = epoch + 365.0 * 86400.0;
    sys.sync_to(end);
    let sun = sys.star_local_position(end);
    let mut metrics = Vec::new();
    for (name, km) in reference::HELIO_2027 {
        if let Some(body) = sys.find_body(name) {
            let actual = (sys.body_local_position(body, end) - sun) / 1000.0;
            let expected = Vec3d::new(km[0], km[1], km[2]);
            metrics.push(metric(
                format!("{name} heliocentric position residual"),
                (actual - expected).length(),
                "km",
                if *name == "Moon" { 3000.0 } else { 1500.0 },
            ));
        }
    }
    let data = horizons::Dataset::embedded();
    cases.push(ValidationCase{id:"solar-lab-horizons-365-days".into(),name:"Solar System · 365 days against cached JPL positions".into(),model:"Accurate N-body / rails hybrid, central-body 1PN enabled".into(),inputs:serde_json::json!({"initial_epoch":data.dataset.epoch,"initial_epoch_jd_tdb":data.dataset.epoch_jd_tdb,"initial_dataset":data.id(),"retrieved_utc":data.dataset.retrieved_utc,"provider":data.dataset.provider,"attribution":data.dataset.attribution,
            "initial_vectors":data.body.iter().map(|r|serde_json::json!({"name":r.name,"horizons_id":r.horizons_id,"ephemeris":r.ephemeris,"barycentric_position_km":r.position_km,"barycentric_velocity_km_s":r.velocity_km_s})).collect::<Vec<_>>(),
            "reference_heliocentric_positions_km":reference::HELIO_2027.iter().map(|(name,position)|serde_json::json!({"name":name,"position_km":position})).collect::<Vec<_>>(),
            "reference_epoch":"2027-01-01 00:00 TDB","duration_s":365.0*86400.0,"preset":"Accurate","steps_per_orbit":preset.settings().steps_per_orbit,"relativity":true,"frame":"heliocentric positions, ecliptic J2000 orientation","reference_file":"data/horizons_validation_2027.rs"}),
        reference:"https://ssd.jpl.nasa.gov/horizons/manual.html · locally cached 2027 endpoint, fetched by tools/fetch_horizons.py".into(),metrics,
        limitations:"One endpoint for one starting epoch, compared with rounded locally cached reference coordinates; no covariance/error propagation or interval-wide bound. Giant-planet states use system barycentres. Missing J2, tides and small-body perturbations matter; the Earth–Moon residual is not independently attributed. Regression thresholds are historical acceptance guards, not uncertainties or a research-precision target.".into()});
    let passes = cases
        .iter()
        .all(|c| !c.metrics.is_empty() && c.metrics.iter().all(|m| m.passes_regression));
    ValidationReport{format:"cosmogon-model-validation".into(),schema_version:1,app_version:env!("CARGO_PKG_VERSION").into(),physics_engine:save::PHYSICS_ENGINE.into(),cases,passes_regression:passes,limitations:vec!["Covers only these explicitly described orbital fixtures. Climate, stellar, collision and civilization models are not validated by this report.".into(),"Results are calculated at request time using the installed engine. They do not certify every sandbox, mission planning, observational reduction or long-term accuracy.".into(),"No measured uncertainty/covariance is available for these exported cached reference fixtures.".into()]}
}

impl ValidationReport {
    /// New file only, so a previous scientific report cannot be silently replaced.
    pub fn export(&self, path: &std::path::Path) -> Result<(), String> {
        use std::io::Write;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_contain_computed_residuals_and_reproducible_conditions() {
        let a = validation_report();
        let b = validation_report();
        assert!(a.passes_regression);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert_eq!(a.cases.len(), 3);
        assert_eq!(a.cases[2].metrics.len(), 9);
        assert!(a.cases[0].metrics[0].value > a.cases[1].metrics[0].value);
        assert!(a.cases[2]
            .metrics
            .iter()
            .any(|m| m.quantity.contains("Earth") && m.value > 1.0));
        assert!(a.cases.iter().all(|c| !c.limitations.is_empty()));
    }
}
