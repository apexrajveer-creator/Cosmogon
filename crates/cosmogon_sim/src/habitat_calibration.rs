//! Authored life-support calibration with retained sensitivity envelopes: each published
//! reference life-support assumption carries an authored calibration multiplier and an
//! authored ± band that is retained per completed interval as bounded exchange evidence.
//! Factors and bands are design inputs (ESTIMATED), not fitted measurements, measured
//! variance or confidence intervals.
use serde::{Deserialize, Serialize};

pub const CALIBRATION_MODEL: &str = "cosmogon-life-support-calibration/1";

fn bounds(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be finite and between {min} and {max}"))
    }
}

/// One authored calibration entry: a multiplier on the reference rate plus an authored
/// ± half-width retained around the applied interval exchange.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Factor {
    /// Calibration multiplier applied to the reference rate (1.0 = reference assumption).
    pub central: f64,
    /// Authored ± half-width for the retained envelope (0 keeps low = high = applied).
    pub relative_band: f64,
}
impl Default for Factor {
    fn default() -> Self {
        Self {
            central: 1.0,
            relative_band: 0.0,
        }
    }
}
impl Factor {
    pub fn validate(&self, name: &str) -> Result<(), String> {
        bounds(
            self.central,
            0.1,
            10.0,
            &format!("{name} calibration factor"),
        )?;
        bounds(
            self.relative_band,
            0.0,
            1.0,
            &format!("{name} relative band"),
        )
    }
    /// Authored sensitivity bound around one applied exchange value.
    pub fn bound(&self, applied: f64) -> Bound {
        Bound {
            applied,
            low: (applied * (1.0 - self.relative_band)).max(0.0),
            high: applied * (1.0 + self.relative_band),
        }
    }
}

/// Authored life-support calibration assumptions for one habitat.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    /// Calibration of the nominal crew exchange cycle (NASA BVAD Table 3-31).
    pub crew_demand: Factor,
    /// Calibration of the authored CO₂ capture rate.
    pub scrubber: Factor,
    /// Calibration of the authored water processing rate.
    pub recycler: Factor,
    /// Calibration of the regulator's stored-oxygen makeup rate.
    pub oxygen_makeup: Factor,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: CALIBRATION_MODEL.into(),
            crew_demand: Factor::default(),
            scrubber: Factor::default(),
            recycler: Factor::default(),
            oxygen_makeup: Factor::default(),
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != CALIBRATION_MODEL {
            return Err(format!(
                "Unsupported life-support calibration model {}",
                self.model
            ));
        }
        self.crew_demand.validate("Crew demand")?;
        self.scrubber.validate("CO₂ capture")?;
        self.recycler.validate("Water processing")?;
        self.oxygen_makeup.validate("O₂ makeup")
    }
}

/// One retained exchange bound: the applied calibrated value with its authored ± band.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Bound {
    pub applied: f64,
    pub low: f64,
    pub high: f64,
}
impl Bound {
    fn validate(&self, f: &Factor, name: &str) -> Result<(), String> {
        for v in [self.applied, self.low, self.high] {
            if !v.is_finite() || v < 0.0 {
                return Err(format!("{name} interval bound must be finite and non-negative"));
            }
        }
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0);
        if !near(self.low, (self.applied * (1.0 - f.relative_band)).max(0.0))
            || !near(self.high, self.applied * (1.0 + f.relative_band))
            || self.low > self.applied + 1e-9
            || self.applied > self.high + 1e-9
        {
            return Err(format!(
                "{name} interval disagrees with the retained calibration band"
            ));
        }
        Ok(())
    }
}

/// Last completed interval's calibrated exchanges with authored sensitivity bounds.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Budget {
    pub crew_cycle_person_days: Bound,
    pub oxygen_makeup_kg: Bound,
    pub co2_captured_kg: Bound,
    pub water_processed_kg: Bound,
}
impl Budget {
    pub fn new(c: &Config, crew_cycle: f64, oxygen_makeup: f64, co2_captured: f64, water_processed: f64) -> Self {
        Self {
            crew_cycle_person_days: c.crew_demand.bound(crew_cycle),
            oxygen_makeup_kg: c.oxygen_makeup.bound(oxygen_makeup),
            co2_captured_kg: c.scrubber.bound(co2_captured),
            water_processed_kg: c.recycler.bound(water_processed),
        }
    }
    pub fn validate(&self, c: &Config) -> Result<(), String> {
        self.crew_cycle_person_days
            .validate(&c.crew_demand, "Crew cycle")?;
        self.oxygen_makeup_kg
            .validate(&c.oxygen_makeup, "O₂ makeup")?;
        self.co2_captured_kg.validate(&c.scrubber, "CO₂ capture")?;
        self.water_processed_kg
            .validate(&c.recycler, "Water processed")
    }
}

pub fn source() -> serde_json::Value {
    serde_json::json!({"model":CALIBRATION_MODEL,
        "version":"Authored design inputs (ESTIMATED); not fitted to measurements",
        "description":"Each published reference life-support rate carries an authored calibration multiplier applied in place, and each completed interval retains applied exchanges with authored ± sensitivity bands around them.",
        "law":"applied = reference expression × central; low = max(0, applied·(1−band)); high = applied·(1+band)",
        "uncertainty":null,"limits":"Bands are authored sensitivity bounds around applied calibrated exchanges, not re-run physics, measured variance or confidence intervals. No parameter identification, observed-data validation, probabilistic propagation or hardware reliability is claimed."})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factor(central: f64, band: f64) -> Factor {
        Factor {
            central,
            relative_band: band,
        }
    }

    #[test]
    fn default_config_is_identity_and_valid() {
        let c = Config::default();
        assert_eq!(c.model, CALIBRATION_MODEL);
        c.validate().unwrap();
        assert_eq!(c.crew_demand, factor(1.0, 0.0));
        assert_eq!(c.scrubber, factor(1.0, 0.0));
        assert_eq!(c.recycler, factor(1.0, 0.0));
        assert_eq!(c.oxygen_makeup, factor(1.0, 0.0));
    }

    #[test]
    fn validation_refuses_bad_factors_and_models() {
        let mut c = Config::default();
        c.crew_demand.central = 0.05;
        assert!(c.validate().is_err());
        c = Config::default();
        c.scrubber.central = 10.5;
        assert!(c.validate().is_err());
        c = Config::default();
        c.recycler.relative_band = -0.1;
        assert!(c.validate().is_err());
        c = Config::default();
        c.oxygen_makeup.relative_band = 1.1;
        assert!(c.validate().is_err());
        c = Config::default();
        c.crew_demand.central = f64::NAN;
        assert!(c.validate().is_err());
        c = Config::default();
        c.model = "future/2".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn deny_unknown_fields_rejects_extra_keys() {
        let json = r#"{"model":"cosmogon-life-support-calibration/1","crew_demand":{"central":1.0,"relative_band":0.0},"scrubber":{"central":1.0,"relative_band":0.0},"recycler":{"central":1.0,"relative_band":0.0},"oxygen_makeup":{"central":1.0,"relative_band":0.0},"mystery":1}"#;
        assert!(serde_json::from_str::<Config>(json).is_err());
    }

    #[test]
    fn bands_wrap_applied_exchanges() {
        let f = factor(1.2, 0.25);
        let b = f.bound(8.0);
        assert_eq!(b, Bound { applied: 8.0, low: 6.0, high: 10.0 });
        // Zero band collapses to the applied value; zero applied stays zero.
        let b = factor(1.0, 0.0).bound(3.5);
        assert_eq!(b, Bound { applied: 3.5, low: 3.5, high: 3.5 });
        let b = factor(1.0, 0.5).bound(0.0);
        assert_eq!(b, Bound { applied: 0.0, low: 0.0, high: 0.0 });
        // Non-negative clamp holds for any authored band.
        let b = factor(1.0, 1.0).bound(2.0);
        assert_eq!(b.low, 0.0);
        assert_eq!(b.high, 4.0);
    }

    #[test]
    fn budget_validates_band_arithmetic_and_refuses_tampering() {
        let c = Config {
            crew_demand: factor(1.5, 0.1),
            scrubber: factor(0.9, 0.2),
            recycler: factor(1.0, 0.0),
            oxygen_makeup: factor(1.1, 0.5),
            ..Config::default()
        };
        let b = Budget::new(&c, 4.0, 0.5, 0.2, 25.0);
        b.validate(&c).unwrap();
        let mut bad = b.clone();
        bad.co2_captured_kg.high += 0.01;
        assert!(bad.validate(&c).is_err());
        let mut bad = b.clone();
        bad.crew_cycle_person_days.low = 0.0;
        assert!(bad.validate(&c).is_err());
        let mut bad = b.clone();
        bad.oxygen_makeup_kg.applied = f64::INFINITY;
        assert!(bad.validate(&c).is_err());
        let mut bad = b.clone();
        bad.water_processed_kg.high = -1.0;
        assert!(bad.validate(&c).is_err());
    }

    #[test]
    fn budget_round_trips_through_json() {
        let c = Config::default();
        let b = Budget::new(&c, 4.0, 0.25, 0.5, 20.0);
        let json = serde_json::to_string(&b).unwrap();
        let back: Budget = serde_json::from_str(&json).unwrap();
        assert_eq!(back, b);
        back.validate(&c).unwrap();
    }
}
