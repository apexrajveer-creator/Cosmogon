//! Continuous finite cabin construction: held processed material is consumed at an
//! authored build rate under ordinary bus power and becomes additional cabin volume.
//! Rates, power and the structural bulk ratio are user assumptions, not measured
//! construction hardware or field calibration.
use serde::{Deserialize, Serialize};

pub const CONSTRUCTION_MODEL: &str = "cosmogon-cabin-construction/1";

fn bounds(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be finite and between {min} and {max}"))
    }
}

/// Authored continuous construction assumptions for one habitat.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    /// Bound of held processed construction material.
    pub material_capacity_kg: f64,
    /// Material consumed per second at full served construction power.
    pub build_rate_kg_s: f64,
    /// Ordinary bus electricity drawn while construction runs.
    pub power_w: f64,
    /// Authored structural bulk ratio: cabin volume added per kilogram consumed.
    pub volume_per_kg_m3: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: CONSTRUCTION_MODEL.into(),
            material_capacity_kg: 1000.0,
            build_rate_kg_s: 1.0,
            power_w: 200.0,
            volume_per_kg_m3: 0.01,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != CONSTRUCTION_MODEL {
            return Err(format!("Unsupported cabin construction model {}", self.model));
        }
        bounds(
            self.material_capacity_kg,
            1.0e-6,
            1.0e7,
            "Construction material capacity (kg)",
        )?;
        bounds(
            self.build_rate_kg_s,
            1.0e-6,
            1.0e4,
            "Construction build rate (kg/s)",
        )?;
        bounds(self.power_w, 0.0, 1.0e6, "Construction power (W)")?;
        bounds(
            self.volume_per_kg_m3,
            1.0e-9,
            1.0,
            "Construction volume per kilogram (m³/kg)",
        )
    }
}

/// Last completed interval's construction accounting.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Budget {
    pub material_before_kg: f64,
    pub material_after_kg: f64,
    pub consumed_kg: f64,
    pub volume_added_m3: f64,
    pub requested_w: f64,
    pub delivered_w: f64,
    pub served_fraction: f64,
    pub constraints: Vec<String>,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            material_before_kg: 0.0,
            material_after_kg: 0.0,
            consumed_kg: 0.0,
            volume_added_m3: 0.0,
            requested_w: 0.0,
            delivered_w: 0.0,
            served_fraction: 0.0,
            constraints: Vec::new(),
        }
    }
}
impl Budget {
    pub fn validate(&self, c: &Config, material_kg: f64) -> Result<(), String> {
        for v in [
            self.material_before_kg,
            self.material_after_kg,
            self.consumed_kg,
            self.volume_added_m3,
            self.requested_w,
            self.delivered_w,
            self.served_fraction,
        ] {
            if !v.is_finite() || v < 0.0 {
                return Err("Invalid construction interval quantity".into());
            }
        }
        bounds(
            self.material_before_kg,
            0.0,
            c.material_capacity_kg,
            "Construction material before interval",
        )?;
        bounds(
            self.material_after_kg,
            0.0,
            c.material_capacity_kg,
            "Construction material after interval",
        )?;
        bounds(self.served_fraction, 0.0, 1.0, "Construction served fraction")?;
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0);
        if !near(
            self.consumed_kg,
            (self.material_before_kg - self.material_after_kg).max(0.0),
        ) || !near(self.volume_added_m3, self.consumed_kg * c.volume_per_kg_m3)
            || !near(self.material_after_kg, material_kg)
            || self.material_before_kg + 1e-9 < self.material_after_kg
        {
            return Err("Construction interval disagrees with retained configuration".into());
        }
        Ok(())
    }
}

/// Bounded material draw for one interval at the served fraction, further bounded so
/// the live volume never exceeds the authored upper volume bound. Returns
/// (consumed_kg, volume_added_m3, constraint).
pub fn draw(
    c: &Config,
    material_kg: f64,
    volume_m3: f64,
    dt: f64,
    served_fraction: f64,
) -> (f64, f64, Option<String>) {
    let mut consumed = (c.build_rate_kg_s * dt * served_fraction).min(material_kg);
    let mut constraint = None;
    if volume_m3 >= 1.0e6 {
        consumed = 0.0;
        constraint = Some("cabin volume at its authored upper bound".into());
    } else if consumed * c.volume_per_kg_m3 > 1.0e6 - volume_m3 {
        consumed = (1.0e6 - volume_m3) / c.volume_per_kg_m3;
        constraint = Some("cabin volume approaching its authored upper bound".into());
    }
    (consumed.max(0.0), consumed.max(0.0) * c.volume_per_kg_m3, constraint)
}

pub fn source() -> serde_json::Value {
    serde_json::json!({"model":CONSTRUCTION_MODEL,
        "version":"Explicit engineering assumption; no measured construction hardware dataset claimed",
        "description":"Held processed construction material is consumed at an authored build rate while ordinary bus power is served, and each kilogram becomes an authored bulk ratio of additional live cabin volume at unchanged gas mass.",
        "law":"consumed = min(material, build_rate · dt · served); volume += consumed · volume_per_kg; power is an ordinary bus slot",
        "uncertainty":null,"limits":"No structural strength, thermal-mass growth, dust, EVA, crew labour, assembly physics, stochastic delays or vehicle-mediated material delivery. Field calibration and predictive uncertainty remain unquantified."})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            model: CONSTRUCTION_MODEL.into(),
            material_capacity_kg: 1000.0,
            build_rate_kg_s: 1.0,
            power_w: 200.0,
            volume_per_kg_m3: 0.01,
        }
    }

    #[test]
    fn draw_consumes_at_rate_bounds_material_and_volume() {
        let c = config();
        // One fully served minute at 1 kg/s.
        let (consumed, volume, constraint) = draw(&c, 500.0, 500.0, 60.0, 1.0);
        assert_eq!(consumed, 60.0);
        assert_eq!(volume, 0.6);
        assert!(constraint.is_none());
        // Held material binds exactly.
        let (consumed, volume, _) = draw(&c, 10.0, 500.0, 60.0, 1.0);
        assert_eq!(consumed, 10.0);
        assert_eq!(volume, 0.1);
        // Unserved power stops the build.
        let (consumed, volume, _) = draw(&c, 500.0, 500.0, 60.0, 0.0);
        assert_eq!((consumed, volume), (0.0, 0.0));
        // Volume bound binds and leaves a retained constraint.
        let (consumed, volume, constraint) = draw(&c, 500.0, 1.0e6 - 0.05, 60.0, 1.0);
        assert!((volume - 0.05).abs() < 1e-8);
        assert!((consumed - 5.0).abs() < 1e-6);
        assert!((volume - consumed * 0.01).abs() < 1e-12);
        assert!(constraint.is_some());
        let (consumed, volume, constraint) = draw(&c, 500.0, 1.0e6, 60.0, 1.0);
        assert_eq!((consumed, volume), (0.0, 0.0));
        assert!(constraint.is_some());
    }

    #[test]
    fn validation_refuses_bad_bounds_and_models() {
        let mut c = config();
        c.material_capacity_kg = 0.0;
        assert!(c.validate().is_err());
        c = config();
        c.build_rate_kg_s = -1.0;
        assert!(c.validate().is_err());
        c = config();
        c.volume_per_kg_m3 = 0.0;
        assert!(c.validate().is_err());
        c = config();
        c.model = "future/2".into();
        assert!(c.validate().is_err());
        config().validate().unwrap();
    }

    #[test]
    fn budget_rejects_inconsistent_arithmetic() {
        let c = config();
        let mut b = Budget {
            material_before_kg: 100.0,
            material_after_kg: 90.0,
            consumed_kg: 10.0,
            volume_added_m3: 0.1,
            requested_w: 200.0,
            delivered_w: 200.0,
            served_fraction: 1.0,
            constraints: Vec::new(),
        };
        b.validate(&c, 90.0).unwrap();
        b.consumed_kg = 5.0;
        assert!(b.validate(&c, 90.0).is_err());
    }
}
