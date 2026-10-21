//! Optional exterior radiation environment, authored habitat shielding, interior
//! whole-body dose rate, cumulative crew dose and hazard bands. Reduced deterministic
//! authored model: an authored pressure power law passes through published anchor dose
//! rates (MSL cruise ≈1.8 mSv/day, Mars surface ≈0.67 mSv/day, Earth surface cosmic-ray
//! component ≈0.01 mSv/day) with authored magnetosphere and stellar flare factors;
//! shielding is two authored constant transmission fractions. No particle transport,
//! solar-particle-event forecasting, activation or personal dosimetry.
use crate::time::SECONDS_PER_DAY;
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-habitat-radiation/1";
/// Unshielded ~1 AU galactic cosmic-ray dose-rate anchor (MSL cruise).
pub const REFERENCE_DOSE_MSV_DAY: f64 = 1.8;
/// Authored pressure response of the exterior power law (anchor-interpolating).
pub const PRESSURE_RESPONSE: f64 = 340.0;
/// Authored pressure exponent of the exterior power law (anchor-interpolating).
pub const PRESSURE_EXPONENT: f64 = 0.89;
/// Authored maximum exterior-dose removal by an Earth-like dipole.
pub const MAGNETOSPHERE_REMOVAL: f64 = 0.3;
/// Authored exterior-dose gain at maximum stellar flare activity.
pub const FLARE_GAIN: f64 = 4.0;
/// Hazard band boundary: ICRP single-year occupational reference scaled to one day.
pub const ELEVATED_DOSE_RATE_MSV_DAY: f64 = 50.0 / 365.25;
/// Hazard band boundary: sustained ISS-class operational crew dose rate.
pub const ABOVE_DESIGN_DOSE_RATE_MSV_DAY: f64 = 1.0;

fn bounds(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be finite and between {min} and {max}"))
    }
}
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

/// Exterior (outside-habitat) dose rate at the body surface in mSv/day. Inputs are the
/// body's atmospheric pressure and magnetic field plus the primary star's current flare
/// activity; each input is clamped to its physical model domain before evaluation.
pub fn exterior_dose_rate_msv_day(
    flare_activity: f64,
    magnetic_field: f64,
    atmosphere_pressure_bar: f64,
) -> f64 {
    let pressure = atmosphere_pressure_bar.max(0.0);
    let magnetic = magnetic_field.clamp(0.0, 1.0);
    let flare = flare_activity.max(0.0);
    REFERENCE_DOSE_MSV_DAY
        * (1.0 + PRESSURE_RESPONSE * pressure).powf(-PRESSURE_EXPONENT)
        * (1.0 - MAGNETOSPHERE_REMOVAL * magnetic)
        * (1.0 + FLARE_GAIN * flare)
}

/// Hazard band of the sustained interior whole-body dose rate. Boundaries are published
/// reference values; the band is a modelled classification, not a safety determination.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HazardBand {
    Nominal,
    Elevated,
    AboveDesign,
    Unavailable,
}
impl HazardBand {
    /// Exterior environment or interval budget missing; no dose claim is made.
    pub fn of(interior_dose_rate_msv_day: Option<f64>) -> Self {
        match interior_dose_rate_msv_day {
            None => Self::Unavailable,
            Some(v) if v <= ELEVATED_DOSE_RATE_MSV_DAY => Self::Nominal,
            Some(v) if v <= ABOVE_DESIGN_DOSE_RATE_MSV_DAY => Self::Elevated,
            Some(_) => Self::AboveDesign,
        }
    }
    /// Stable single-token label used by CSV exports and report observations.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Elevated => "elevated",
            Self::AboveDesign => "above_design",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Authored radiation-shielding design inputs for one habitat. Absent definitions stay
/// byte-identical and record no dose.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    /// Authored fraction of the exterior dose rate transmitted through the cabin
    /// structure (1.0 transmits everything, 0.0 is fully opaque).
    pub wall_transmission: f64,
    /// Authored additional transmission fraction inside the designated storm shelter,
    /// applied on top of the wall transmission (1.0 adds no shelter benefit).
    pub storm_shelter_transmission: f64,
    /// Sustained interior dose-rate design threshold for the habitat alert (mSv/day).
    pub alert_dose_rate_msv_day: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            wall_transmission: 0.5,
            storm_shelter_transmission: 0.25,
            alert_dose_rate_msv_day: 1.0,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err(format!("Unsupported habitat radiation model {}", self.model));
        }
        bounds(
            self.wall_transmission,
            0.0,
            1.0,
            "Wall dose transmission",
        )?;
        bounds(
            self.storm_shelter_transmission,
            0.0,
            1.0,
            "Storm-shelter dose transmission",
        )?;
        bounds(
            self.alert_dose_rate_msv_day,
            0.0,
            1.0e4,
            "Radiation dose-rate design alert",
        )
    }
}

/// Last completed interval's modelled doses with their hazard band. Exterior rate comes
/// from the environment, interior/shelter rates from the authored transmissions and the
/// interval dose from the interior rate over the completed interval.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    pub exterior_dose_rate_msv_day: f64,
    pub interior_dose_rate_msv_day: f64,
    pub storm_shelter_dose_rate_msv_day: f64,
    pub interval_dose_msv: f64,
    pub hazard_band: HazardBand,
}
impl Budget {
    pub fn new(c: &Config, exterior_dose_rate_msv_day: f64, dt: f64) -> Self {
        let interior = exterior_dose_rate_msv_day * c.wall_transmission;
        Self {
            exterior_dose_rate_msv_day,
            interior_dose_rate_msv_day: interior,
            storm_shelter_dose_rate_msv_day: interior * c.storm_shelter_transmission,
            interval_dose_msv: interior * dt / SECONDS_PER_DAY,
            hazard_band: HazardBand::of(Some(interior)),
        }
    }
    pub fn validate(&self, c: &Config, dt: f64) -> Result<(), String> {
        for (v, n) in [
            (
                self.exterior_dose_rate_msv_day,
                "Recorded exterior dose rate",
            ),
            (
                self.interior_dose_rate_msv_day,
                "Recorded interior dose rate",
            ),
            (
                self.storm_shelter_dose_rate_msv_day,
                "Recorded storm-shelter dose rate",
            ),
        ] {
            bounds(v, 0.0, 1.0e4, n)?;
        }
        bounds(self.interval_dose_msv, 0.0, 10.0, "Interval crew dose")?;
        let interior = self.exterior_dose_rate_msv_day * c.wall_transmission;
        if !near(self.interior_dose_rate_msv_day, interior) {
            return Err("Interior dose disagrees with exterior rate and wall transmission".into());
        }
        if !near(
            self.storm_shelter_dose_rate_msv_day,
            interior * c.storm_shelter_transmission,
        ) {
            return Err("Storm-shelter dose disagrees with shelter transmission".into());
        }
        if !near(
            self.interval_dose_msv,
            interior * dt / SECONDS_PER_DAY,
        ) {
            return Err("Interval crew dose disagrees with interior rate and interval".into());
        }
        if self.hazard_band != HazardBand::of(Some(self.interior_dose_rate_msv_day)) {
            return Err("Hazard band disagrees with the recorded interior dose rate".into());
        }
        Ok(())
    }
}

pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MODEL,
        "version":"Authored design inputs (ESTIMATED); not fitted measurements, particle transport or dosimetry",
        "description":"An authored pressure power law passes through published anchor dose rates for the exterior environment; authored magnetosphere and stellar-flare factors scale it; authored constant wall and storm-shelter transmission fractions yield interior, shelter and interval whole-body dose rates with published-reference hazard bands.",
        "exterior_law":"exterior_mSv_day = 1.8 × (1 + 340·p_bar)^(-0.89) × (1 − 0.3·B_earth) × (1 + 4·flare_activity), with p_bar ≥ 0, B_earth and flare clamped to [0,1]/[0,∞)",
        "shielding_law":"interior = exterior × wall_transmission; shelter = interior × storm_shelter_transmission; interval_dose_msv = interior × interval_s / 86400",
        "anchors":"MSL cruise ≈1.8 mSv/day (Zeitlin et al. 2013), Mars surface ≈0.67 mSv/day (Hassler et al. 2014), Earth surface cosmic-ray component ≈0.01 mSv/day (ICRP Publication 103); the authored law passes through these order-of-magnitude anchors, it is not a fit or environment reconstruction",
        "bands":"nominal ≤ 50 mSv/365.25 d ≈ 0.137 mSv/day (ICRP single-year occupational reference); elevated ≤ 1.0 mSv/day (sustained ISS-class crew rate); above_design beyond; the habitat alert uses a separate authored design threshold",
        "uncertainty":null,
        "limits":"Reduced deterministic whole-body dose-rate bookkeeping under authored shielding fractions. No particle transport, solar-particle-event forecasting, secondary production, activation, geometry/orientation, self-shielding by stores, spatial or individual variation, career dose redistribution, measured hardware attenuation or medical/safety interpretation. Published anchors are order-of-magnitude references, not validation of any specific habitat, environment or crew."})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid_and_authored() {
        let c = Config::default();
        assert_eq!(c.model, MODEL);
        c.validate().unwrap();
        assert_eq!(c.wall_transmission, 0.5);
        assert_eq!(c.storm_shelter_transmission, 0.25);
        assert_eq!(c.alert_dose_rate_msv_day, 1.0);
    }

    #[test]
    fn validation_refuses_bad_models_transmissions_and_thresholds() {
        let bad_model = Config {
            model: "future/2".into(),
            ..Config::default()
        };
        assert!(bad_model.validate().is_err());
        let bad_wall_low = Config {
            wall_transmission: -0.01,
            ..Config::default()
        };
        assert!(bad_wall_low.validate().is_err());
        let bad_wall_high = Config {
            wall_transmission: 1.01,
            ..Config::default()
        };
        assert!(bad_wall_high.validate().is_err());
        let bad_shelter_low = Config {
            storm_shelter_transmission: -1.0,
            ..Config::default()
        };
        assert!(bad_shelter_low.validate().is_err());
        let bad_shelter_nan = Config {
            storm_shelter_transmission: f64::NAN,
            ..Config::default()
        };
        assert!(bad_shelter_nan.validate().is_err());
        let bad_alert_low = Config {
            alert_dose_rate_msv_day: -1.0,
            ..Config::default()
        };
        assert!(bad_alert_low.validate().is_err());
        let bad_alert_high = Config {
            alert_dose_rate_msv_day: 1.0e5,
            ..Config::default()
        };
        assert!(bad_alert_high.validate().is_err());
    }

    #[test]
    fn deny_unknown_fields_rejects_extra_keys() {
        let json = format!(
            r#"{{"model":"{MODEL}","wall_transmission":0.5,"storm_shelter_transmission":0.25,"alert_dose_rate_msv_day":1.0,"mismatch":1}}"#
        );
        assert!(serde_json::from_str::<Config>(&json).is_err());
    }

    #[test]
    fn exterior_dose_matches_authored_anchors() {
        // Vacuum, quiet star: the unshielded MSL cruise anchor exactly.
        assert_eq!(exterior_dose_rate_msv_day(0.0, 0.0, 0.0), REFERENCE_DOSE_MSV_DAY);
        // Mars-like surface: authored law passes through the ≈0.67 mSv/day anchor.
        let mars = exterior_dose_rate_msv_day(0.0, 0.0, 0.006);
        assert!((mars - 0.67).abs() < 0.01, "{mars}");
        // Earth-like surface: order of the ≈0.01 mSv/day cosmic-ray component.
        let earth = exterior_dose_rate_msv_day(0.0, 1.0, 1.01325);
        assert!((0.001..=0.02).contains(&earth), "{earth}");
        // Maximum authored flare activity multiplies the quiet exterior rate by five.
        let flaring = exterior_dose_rate_msv_day(1.0, 0.0, 0.0);
        assert!((flaring - 5.0 * REFERENCE_DOSE_MSV_DAY).abs() < 1e-12, "{flaring}");
    }

    #[test]
    fn exterior_dose_is_monotonic_in_each_authored_input() {
        let base = exterior_dose_rate_msv_day(0.0, 0.0, 0.0);
        // Thicker atmosphere and stronger dipole only attenuate; flares only add.
        assert!(
            exterior_dose_rate_msv_day(0.0, 0.0, 1.0)
                < exterior_dose_rate_msv_day(0.0, 0.0, 0.006)
                && exterior_dose_rate_msv_day(0.0, 0.0, 0.006) < base
        );
        assert!(
            exterior_dose_rate_msv_day(0.0, 1.0, 0.0)
                < exterior_dose_rate_msv_day(0.0, 0.5, 0.0)
                && exterior_dose_rate_msv_day(0.0, 0.5, 0.0) < base
        );
        assert!(exterior_dose_rate_msv_day(0.5, 0.0, 0.0) > base);
        // Inputs outside their physical domains clamp instead of inverting the law.
        assert_eq!(
            exterior_dose_rate_msv_day(0.0, -3.0, -1.0),
            exterior_dose_rate_msv_day(0.0, 0.0, 0.0)
        );
        assert_eq!(
            exterior_dose_rate_msv_day(-1.0, 2.0, 0.0),
            exterior_dose_rate_msv_day(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn hazard_bands_follow_published_thresholds() {
        assert_eq!(HazardBand::of(None), HazardBand::Unavailable);
        assert_eq!(HazardBand::of(Some(0.0)), HazardBand::Nominal);
        assert_eq!(HazardBand::of(Some(ELEVATED_DOSE_RATE_MSV_DAY)), HazardBand::Nominal);
        assert_eq!(
            HazardBand::of(Some(ELEVATED_DOSE_RATE_MSV_DAY + 1.0e-9)),
            HazardBand::Elevated
        );
        assert_eq!(
            HazardBand::of(Some(ABOVE_DESIGN_DOSE_RATE_MSV_DAY)),
            HazardBand::Elevated
        );
        assert_eq!(
            HazardBand::of(Some(ABOVE_DESIGN_DOSE_RATE_MSV_DAY + 1.0e-9)),
            HazardBand::AboveDesign
        );
        assert_eq!(HazardBand::Nominal.label(), "nominal");
        assert_eq!(HazardBand::Elevated.label(), "elevated");
        assert_eq!(HazardBand::AboveDesign.label(), "above_design");
        assert_eq!(HazardBand::Unavailable.label(), "unavailable");
        for band in [
            HazardBand::Nominal,
            HazardBand::Elevated,
            HazardBand::AboveDesign,
            HazardBand::Unavailable,
        ] {
            let json = serde_json::to_string(&band).unwrap();
            let back: HazardBand = serde_json::from_str(&json).unwrap();
            assert_eq!(back, band);
            assert!(json.contains(band.label()));
        }
    }

    #[test]
    fn budget_applies_authored_transmissions_over_the_interval() {
        let c = Config::default();
        let b = Budget::new(&c, 2.0, 60.0);
        b.validate(&c, 60.0).unwrap();
        assert_eq!(b.exterior_dose_rate_msv_day, 2.0);
        assert_eq!(b.interior_dose_rate_msv_day, 1.0);
        assert_eq!(b.storm_shelter_dose_rate_msv_day, 0.25);
        assert!((b.interval_dose_msv - 60.0 / SECONDS_PER_DAY).abs() < 1e-15);
        assert_eq!(b.hazard_band, HazardBand::Elevated);
        // Zero exterior dose stays zero everywhere and bands as nominal.
        let quiet = Budget::new(&c, 0.0, 60.0);
        quiet.validate(&c, 60.0).unwrap();
        assert_eq!(quiet.interval_dose_msv, 0.0);
        assert_eq!(quiet.hazard_band, HazardBand::Nominal);
    }

    #[test]
    fn budget_validation_refuses_tampering_and_domain_breaks() {
        let c = Config::default();
        let b = Budget::new(&c, 2.0, 60.0);
        let mut bad = b;
        bad.interior_dose_rate_msv_day += 0.01;
        assert!(bad.validate(&c, 60.0).is_err());
        let mut bad = b;
        bad.storm_shelter_dose_rate_msv_day = 1.0;
        assert!(bad.validate(&c, 60.0).is_err());
        let mut bad = b;
        bad.interval_dose_msv += 1.0e-9;
        assert!(bad.validate(&c, 60.0).is_err());
        let mut bad = b;
        bad.hazard_band = HazardBand::AboveDesign;
        assert!(bad.validate(&c, 60.0).is_err());
        let mut bad = b;
        bad.exterior_dose_rate_msv_day = -1.0;
        assert!(bad.validate(&c, 60.0).is_err());
        let mut bad = b;
        bad.exterior_dose_rate_msv_day = f64::INFINITY;
        assert!(bad.validate(&c, 60.0).is_err());
        let mut bad = b;
        bad.interval_dose_msv = 11.0;
        assert!(bad.validate(&c, 60.0).is_err());
        // The interval argument participates in the retained arithmetic.
        assert!(b.validate(&c, 30.0).is_err());
    }

    #[test]
    fn config_and_budget_round_trip_through_json() {
        let c = Config::default();
        let back: Config =
            serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        back.validate().unwrap();
        let b = Budget::new(&c, 1.5, 60.0);
        let back: Budget =
            serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
        assert_eq!(back, b);
        back.validate(&c, 60.0).unwrap();
    }

    #[test]
    fn source_declares_law_anchors_bands_and_limits() {
        let s = source();
        assert_eq!(s["model"], MODEL);
        assert!(s["exterior_law"].as_str().unwrap().contains("p_bar"));
        assert!(s["anchors"].as_str().unwrap().contains("1.8 mSv/day"));
        assert!(s["bands"].as_str().unwrap().contains("0.137"));
        assert!(s["uncertainty"].is_null());
        assert!(s["limits"].as_str().unwrap().contains("No particle transport"));
    }
}
