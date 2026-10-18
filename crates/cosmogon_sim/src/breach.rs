//! Finite well-mixed outward orifice flow; not seal leakage or rarefied flow.
//! NASA Glenn/GUNNS supply the nozzle law. Thermal decomposition/operator splitting
//! and integration policy are explicit Cosmogon assumptions (see source()).
use cosmogon_core::dmath::DMath;
use serde::{Deserialize, Serialize};
pub const MODEL: &str = "cosmogon-cabin-breach/1";
pub const SOURCE: &str = "https://www.grc.nasa.gov/www/k-12/BGP/mflchk.html";
pub const THEORY_SOURCE: &str = "https://github.com/nasa/gunns/wiki/Standard_Flow_Equation";
pub const LIMITS_SOURCE: &str =
    "https://ntrs.nasa.gov/api/citations/20190033458/downloads/20190033458.pdf";
pub const R: f64 = 8.31446261815324;
const FRACTION_STEP: f64 = 0.01;
const MAX_STEPS: u32 = 1024;
fn bound(v: f64, lo: f64, hi: f64, name: &str) -> Result<(), String> {
    if v.is_finite() && (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(format!("{name} must be finite within {lo}–{hi}"))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: String,
    pub area_m2: f64,
    pub discharge_coefficient: f64,
    pub gamma: f64,
    pub outside_pressure_pa: f64,
    pub minimum_pressure_pa: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: MODEL.into(),
            area_m2: 1e-6,
            discharge_coefficient: 0.65,
            gamma: 1.4,
            outside_pressure_pa: 600.0,
            minimum_pressure_pa: 1000.0,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err("Unsupported cabin breach model".into());
        }
        bound(self.area_m2, 0.0, 1.0, "Breach area (m²)")?;
        bound(
            self.discharge_coefficient,
            0.0,
            1.0,
            "Assumed discharge coefficient",
        )?;
        bound(self.gamma, 1.05, 1.67, "Assumed constant gas gamma")?;
        bound(
            self.outside_pressure_pa,
            0.0,
            2e6,
            "Assumed outside receiver pressure (Pa)",
        )?;
        bound(
            self.minimum_pressure_pa,
            100.0,
            2e6,
            "Supported pressure floor (Pa)",
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gas {
    pub oxygen_kg: f64,
    pub nitrogen_kg: f64,
    pub co2_kg: f64,
    pub vapor_kg: f64,
}
impl Gas {
    pub fn mass_kg(self) -> f64 {
        self.oxygen_kg + self.nitrogen_kg + self.co2_kg + self.vapor_kg
    }
    pub fn moles(self) -> f64 {
        self.oxygen_kg / 0.0319988
            + self.nitrogen_kg / 0.0280134
            + self.co2_kg / 0.0440095
            + self.vapor_kg / crate::moisture::WATER_MOLAR_KG
    }
    pub fn validate(self) -> Result<(), String> {
        for v in [self.oxygen_kg, self.nitrogen_kg, self.co2_kg, self.vapor_kg] {
            bound(v, 0.0, 1e12, "Cabin/escaped gas stock")?;
        }
        Ok(())
    }
    pub(crate) fn fraction(self, f: f64) -> Self {
        Self {
            oxygen_kg: self.oxygen_kg * f,
            nitrogen_kg: self.nitrogen_kg * f,
            co2_kg: self.co2_kg * f,
            vapor_kg: self.vapor_kg * f,
        }
    }
    pub fn add(&mut self, g: Self) {
        self.oxygen_kg += g.oxygen_kg;
        self.nitrogen_kg += g.nitrogen_kg;
        self.co2_kg += g.co2_kg;
        self.vapor_kg += g.vapor_kg;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Complete,
    Closed,
    Equilibrium,
    ReverseUnsupported,
    PressureUnsupported,
    TemperatureUnsupported,
    CapacityUnsupported,
    PressureBoundary,
    TemperatureBoundary,
    WorkLimit,
}
impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Complete => "outward flow integrated",
            Self::Closed => "orifice closed",
            Self::Equilibrium => "receiver equilibrium; no outward flow",
            Self::ReverseUnsupported => "receiver above cabin; inward flow unsupported",
            Self::PressureUnsupported => "cabin pressure outside supported domain",
            Self::TemperatureUnsupported => "temperature outside 100–500 K",
            Self::CapacityUnsupported => "configured heat capacity below modeled gas capacity",
            Self::PressureBoundary => "flow stopped at pressure support/equilibrium boundary",
            Self::TemperatureBoundary => "flow stopped at 100 K boundary",
            Self::WorkLimit => "integration work limit; remaining time unresolved",
        }
    }
    pub fn supported(self) -> bool {
        matches!(self, Self::Complete | Self::Closed | Self::Equilibrium)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub initial_gas: Gas,
    pub initial_temperature_k: f64,
    pub final_temperature_k: f64,
    pub initial_pressure_pa: f64,
    pub final_pressure_pa: f64,
    pub initial_rate_kg_s: f64,
    pub final_rate_kg_s: f64,
    pub initial_choked: bool,
    pub integrated_seconds: f64,
    pub integration_steps: u32,
    pub status: Status,
    pub escaped: Gas,
    pub structure_capacity_j_k: f64,
    /// Temperature-equivalent change for the existing fixed-C interval report, not enthalpy.
    pub lumped_cooling_j: f64,
    pub structure_energy_change_j: f64,
    pub gas_energy_change_j: f64,
    pub phase_energy_change_j: f64,
    pub outflow_enthalpy_j: f64,
    pub energy_residual_j: f64,
}
impl Budget {
    pub fn validate(&self, c: &Config, dt: f64, volume: f64, capacity: f64) -> Result<(), String> {
        self.initial_gas.validate()?;
        bound(
            self.initial_temperature_k,
            1.0,
            1e8,
            "Breach start temperature",
        )?;
        if *self
            != integrate(
                c,
                self.initial_gas,
                self.initial_temperature_k,
                volume,
                capacity,
                dt,
            )
        {
            return Err(
                "Breach interval disagrees with retained initial gas/configuration/integration"
                    .into(),
            );
        }
        Ok(())
    }
}
pub fn rate(c: &Config, p: f64, t: f64, r_mix: f64) -> (f64, bool) {
    if c.area_m2 == 0.0 || c.discharge_coefficient == 0.0 || p <= c.outside_pressure_pa || p <= 0.0
    {
        return (0.0, false);
    }
    let critical = (2.0 / (c.gamma + 1.0)).dpowf(c.gamma / (c.gamma - 1.0));
    let r = c.outside_pressure_pa / p;
    let choked = r <= critical;
    let factor = if choked {
        (c.gamma / (r_mix * t)).sqrt()
            * (2.0 / (c.gamma + 1.0)).dpowf((c.gamma + 1.0) / (2.0 * (c.gamma - 1.0)))
    } else {
        (2.0 * c.gamma / (r_mix * t * (c.gamma - 1.0))
            * r.dpowf(2.0 / c.gamma)
            * (1.0 - r.dpowf((c.gamma - 1.0) / c.gamma)))
        .max(0.0)
        .sqrt()
    };
    (c.area_m2 * c.discharge_coefficient * p * factor, choked)
}
pub fn integrate(c: &Config, gas: Gas, t0: f64, volume: f64, capacity: f64, dt: f64) -> Budget {
    integrate_resolution(c, gas, t0, volume, capacity, dt, FRACTION_STEP)
}
fn integrate_resolution(
    c: &Config,
    gas: Gas,
    t0: f64,
    volume: f64,
    capacity: f64,
    dt: f64,
    fraction: f64,
) -> Budget {
    let n0 = gas.moles();
    let p0 = n0 * R * t0 / volume;
    let mut b = Budget {
        initial_gas: gas,
        initial_temperature_k: t0,
        final_temperature_k: t0,
        initial_pressure_pa: p0,
        final_pressure_pa: p0,
        initial_rate_kg_s: 0.0,
        final_rate_kg_s: 0.0,
        initial_choked: false,
        integrated_seconds: 0.0,
        integration_steps: 0,
        status: Status::Complete,
        escaped: Gas::default(),
        structure_capacity_j_k: 0.0,
        lumped_cooling_j: 0.0,
        structure_energy_change_j: 0.0,
        gas_energy_change_j: 0.0,
        phase_energy_change_j: 0.0,
        outflow_enthalpy_j: 0.0,
        energy_residual_j: 0.0,
    };
    if c.area_m2 == 0.0 || c.discharge_coefficient == 0.0 {
        b.status = Status::Closed;
        b.integrated_seconds = dt;
        return b;
    }
    if !(100.0..=500.0).contains(&t0) {
        b.status = Status::TemperatureUnsupported;
        return b;
    }
    if n0 <= 0.0 || !(c.minimum_pressure_pa..=2e6).contains(&p0) {
        b.status = Status::PressureUnsupported;
        return b;
    }
    if p0 < c.outside_pressure_pa {
        b.status = Status::ReverseUnsupported;
        return b;
    }
    if p0 == c.outside_pressure_pa {
        b.status = Status::Equilibrium;
        b.integrated_seconds = dt;
        return b;
    }
    let cv = R / (c.gamma - 1.0);
    let cs = capacity - n0 * cv;
    if cs < 0.0 {
        b.status = Status::CapacityUnsupported;
        return b;
    }
    b.structure_capacity_j_k = cs;
    let molar = gas.mass_kg() / n0;
    let r_mix = R / molar;
    let temperature = |n: f64| t0 * ((cs + n * cv) / capacity).max(0.0).dpowf(c.gamma - 1.0);
    let pressure = |n: f64| n * R * temperature(n) / volume;
    let p_floor = c.minimum_pressure_pa.max(c.outside_pressure_pa);
    let mut low = 0.0;
    let mut high = n0;
    for _ in 0..64 {
        let mid = (low + high) * 0.5;
        if pressure(mid) < p_floor || temperature(mid) < 100.0 {
            low = mid;
        } else {
            high = mid;
        }
    }
    let floor = high;
    let derivative = |n: f64| {
        let n = n.max(floor);
        -rate(c, pressure(n), temperature(n), r_mix).0 / molar
    };
    let rk4 = |n: f64, h: f64| {
        let a = derivative(n);
        let d = derivative(n + 0.5 * h * a);
        let e = derivative(n + 0.5 * h * d);
        let f = derivative(n + h * e);
        n + h / 6.0 * (a + 2.0 * d + 2.0 * e + f)
    };
    (b.initial_rate_kg_s, b.initial_choked) = rate(c, p0, t0, r_mix);
    let mut n = n0;
    while b.integrated_seconds < dt && b.integration_steps < MAX_STEPS {
        if n <= floor {
            b.status = if temperature(floor) <= 100.0 * (1.0 + 1e-12) {
                Status::TemperatureBoundary
            } else {
                Status::PressureBoundary
            };
            break;
        }
        let dn = -derivative(n);
        if dn <= 0.0 {
            b.integrated_seconds = dt;
            b.status = Status::Equilibrium;
            break;
        }
        let mut h = (dt - b.integrated_seconds).min(fraction * n / dn);
        let accurate_step = |h: f64| rk4(rk4(n, h * 0.5), h * 0.5);
        let mut next = accurate_step(h);
        // Step doubling bounds local numerical disagreement, especially near the
        // square-root approach to receiver equilibrium. Attempts count toward work.
        loop {
            b.integration_steps += 1;
            let coarse = rk4(n, h);
            if (coarse - next).abs() <= 1e-11 * n0 || b.integration_steps >= MAX_STEPS {
                break;
            }
            h *= 0.5;
            next = accurate_step(h);
        }
        if b.integration_steps >= MAX_STEPS && (rk4(n, h) - next).abs() > 1e-11 * n0 {
            b.status = Status::WorkLimit;
            break;
        }
        if next <= floor {
            // Resolve the boundary within the attempted numerical step, rather than
            // claim a full interval of valid flow or remove gas below support.
            let mut a = 0.0;
            let mut z = h;
            for _ in 0..48 {
                let mid = (a + z) * 0.5;
                if accurate_step(mid) > floor {
                    a = mid;
                } else {
                    z = mid;
                }
            }
            h = z;
            next = floor;
        }
        if h <= 0.0 || b.integrated_seconds + h == b.integrated_seconds {
            b.status = Status::WorkLimit;
            break;
        }
        n = next.max(floor).min(n);
        b.integrated_seconds = (b.integrated_seconds + h).min(dt);
    }
    if n <= floor {
        b.status = if temperature(n) <= 100.0 * (1.0 + 1e-12) {
            Status::TemperatureBoundary
        } else {
            Status::PressureBoundary
        };
    } else if b.integrated_seconds < dt {
        b.status = Status::WorkLimit;
    }
    let t1 = temperature(n);
    b.escaped = gas.fraction(((n0 - n) / n0).clamp(0.0, 1.0));
    b.final_temperature_k = t1;
    b.final_pressure_pa = pressure(n);
    b.final_rate_kg_s = rate(c, b.final_pressure_pa, t1, r_mix).0;
    b.lumped_cooling_j = capacity * (t0 - t1);
    b.structure_energy_change_j = cs * (t1 - t0);
    b.gas_energy_change_j = cv * (n * (t1 - t0) - (n0 - n) * t0);
    b.phase_energy_change_j = -b.escaped.vapor_kg * crate::moisture::REFERENCE_PHASE_J_KG;
    b.outflow_enthalpy_j = b.lumped_cooling_j + (n0 - n) * cv * t1 - b.phase_energy_change_j;
    b.energy_residual_j = b.structure_energy_change_j
        + b.gas_energy_change_j
        + b.phase_energy_change_j
        + b.outflow_enthalpy_j;
    b
}
pub fn source() -> serde_json::Value {
    serde_json::json!({"model":MODEL,"orifice_source":SOURCE,"subcritical_source":THEORY_SOURCE,"regime_limits_source":LIMITS_SOURCE,"universal_gas_constant_j_mol_k":R,"maximum_fraction_per_step":FRACTION_STEP,"maximum_steps":MAX_STEPS,"step_doubling_mole_tolerance_fraction_initial":1e-11,"numerical_method":"Bounded deterministic step-doubled RK4 in total gas moles; analytic variable-gas thermal relation; fixed bisection support/equilibrium events; existing minute thermal/process law followed by breach operator split","assumptions":"Well-mixed ideal O2/N2/CO2/vapor; constant assumed gamma/discharge/receiver pressure; inferred structure C = configured effective C - initial gas cv; shared temperature; no inward flow; continuum assumed, not inferred from area","thermal_scope":"Separate finite substep enthalpy/structure/gas/reference-vapor-phase identity. Fixed-C main report uses temperature-equivalent cooling debit, not full enthalpy. Split coupling and constant properties are uncalibrated approximations.","limits":"Not seal permeation, tortuous/molecular leak paths, validated geometry, jet/shock forces, liquid entrainment, wall/ice condensation, varying receiver, EVA or physiological predictions; pressure floor and 100–500 K support are explicit modelling choices","uncertainty":null})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn air() -> Gas {
        Gas {
            oxygen_kg: 1.0,
            nitrogen_kg: 3.5,
            co2_kg: 0.01,
            vapor_kg: 0.02,
        }
    }
    #[test]
    fn primary_orifice_formula_choke_transition_and_receiver() {
        let mut c = Config {
            area_m2: 1e-4,
            discharge_coefficient: 0.65,
            outside_pressure_pa: 725000.0,
            ..Default::default()
        };
        // Independent substitution of NASA GUNNS dry-air example (rho=8.616,p=2 MPa).
        let r = 2e6 / (8.616 * 293.15);
        let m = rate(&c, 2e6, 293.15, r);
        assert!(m.1);
        assert!((m.0 - 0.18475724570510083).abs() < 1e-12);
        let critical = (2.0 / (c.gamma + 1.0)).dpowf(c.gamma / (c.gamma - 1.0));
        c.outside_pressure_pa = 1e5 * critical;
        let sonic = rate(&c, 1e5, 293.15, 287.0).0;
        c.outside_pressure_pa *= 1.0 + 1e-8;
        assert!(!rate(&c, 1e5, 293.15, 287.0).1);
        assert!((rate(&c, 1e5, 293.15, 287.0).0 / sonic - 1.0).abs() < 1e-12);
        c.outside_pressure_pa = 1e5;
        assert_eq!(rate(&c, 1e5, 293.15, 287.0), (0.0, false));
        c.outside_pressure_pa = 1.1e5;
        assert_eq!(rate(&c, 1e5, 293.15, 287.0), (0.0, false));
    }
    #[test]
    fn finite_blowdown_matches_closed_form_and_resolution() {
        let g = air();
        let t = 293.15;
        let n = g.moles();
        let cv = R / (Config::default().gamma - 1.0);
        let cap = n * cv;
        let v = n * R * t / 100000.0;
        let c = Config {
            area_m2: 1e-4,
            outside_pressure_pa: 0.0,
            minimum_pressure_pa: 100.0,
            ..Default::default()
        };
        let b = integrate(&c, g, t, v, cap, 60.0);
        assert_eq!(b.status, Status::Complete);
        let rate0 = rate(&c, 1e5, t, R / (g.mass_kg() / n)).0 / g.mass_kg();
        let remaining = (1.0 + 0.5 * (c.gamma - 1.0) * rate0 * 60.0).dpowf(-2.0 / (c.gamma - 1.0));
        assert!((b.escaped.mass_kg() / g.mass_kg() - (1.0 - remaining)).abs() < 1e-10);
        assert!((b.final_temperature_k / t - remaining.dpowf(c.gamma - 1.0)).abs() < 1e-10);
        assert!(b.energy_residual_j.abs() < 1e-7);
        let fine = integrate_resolution(&c, g, t, v, cap, 60.0, 0.0025);
        assert!((fine.escaped.mass_kg() - b.escaped.mass_kg()).abs() < 1e-9);
        let mut sub = c.clone();
        sub.outside_pressure_pa = 80000.0;
        let a = integrate(&sub, g, t, v, 50e6, 60.0);
        let z = integrate_resolution(&sub, g, t, v, 50e6, 60.0, 0.0025);
        assert!(
            (a.escaped.mass_kg() - z.escaped.mass_kg()).abs() < 1e-8,
            "coarse {:?}, fine {:?}, delta {}",
            a,
            z,
            a.escaped.mass_kg() - z.escaped.mass_kg()
        );
    }
    #[test]
    fn domain_boundaries_preserve_stocks_and_phase_energy() {
        let g = air();
        let n = g.moles();
        let t = 293.15;
        let v = n * R * t / 100000.0;
        let c = Config {
            area_m2: 1.0,
            ..Default::default()
        };
        let b = integrate(&c, g, t, v, 50e6, 60.0);
        assert_eq!(b.status, Status::PressureBoundary);
        assert!(b.integrated_seconds < 60.0);
        assert!(b.final_pressure_pa >= c.minimum_pressure_pa * (1.0 - 1e-12));
        assert!(b.escaped.mass_kg() < g.mass_kg());
        assert!(b.energy_residual_j.abs() < 1e-6);
        assert_eq!(
            b.phase_energy_change_j,
            -b.escaped.vapor_kg * crate::moisture::REFERENCE_PHASE_J_KG
        );
        for (e, total) in [
            (b.escaped.oxygen_kg, g.oxygen_kg),
            (b.escaped.nitrogen_kg, g.nitrogen_kg),
            (b.escaped.co2_kg, g.co2_kg),
            (b.escaped.vapor_kg, g.vapor_kg),
        ] {
            assert!((e / total - b.escaped.mass_kg() / g.mass_kg()).abs() < 1e-12);
        }
        let cold = integrate(&c, g, 101.0, v, n * R / (c.gamma - 1.0), 60.0);
        assert_eq!(cold.status, Status::TemperatureBoundary);
        assert!(cold.final_temperature_k >= 100.0 * (1.0 - 1e-12));
        for (status, cc, tt, cap) in [
            (Status::CapacityUnsupported, c.clone(), t, 1.0),
            (Status::TemperatureUnsupported, c.clone(), 600.0, 50e6),
            (
                Status::ReverseUnsupported,
                Config {
                    outside_pressure_pa: 200000.0,
                    ..c.clone()
                },
                t,
                50e6,
            ),
            (Status::Closed, Config { area_m2: 0.0, ..c }, t, 50e6),
        ] {
            let b = integrate(&cc, g, tt, v, cap, 60.0);
            assert_eq!(b.status, status);
            assert_eq!(b.escaped, Gas::default());
            assert_eq!(b.final_temperature_k, tt);
        }
    }
    #[test]
    fn strict_model_and_budget_checks() {
        let mut c = Config {
            gamma: 1.0,
            ..Default::default()
        };
        assert!(c.validate().is_err());
        c = Config {
            model: "future".into(),
            ..Default::default()
        };
        assert!(c.validate().is_err());
        c = Config::default();
        let b = integrate(&c, air(), 293.15, 5.0, 50e6, 60.0);
        b.validate(&c, 60.0, 5.0, 50e6).unwrap();
        let mut bad = b.clone();
        bad.escaped.vapor_kg += 0.001;
        assert!(bad.validate(&c, 60.0, 5.0, 50e6).is_err());
        assert_eq!(b, integrate(&c, air(), 293.15, 5.0, 50e6, 60.0));
    }
}

#[cfg(test)]
mod coupling_tests {
    use super::*;
    use crate::{habitat, sandbox::Edit, BodyRef, Scenario, Universe, UniverseSettings};
    fn world() -> Universe {
        let mut u = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut d = habitat::Definition::at(BodyRef { system: 0, body: 3 }, 0.0, 0.0);
        d.config.breach = Some(Config::default());
        d.config.moisture = Some(crate::moisture::Config::default());
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        u
    }
    #[test]
    fn actual_cabin_losses_conserve_accounted_mass_and_keep_isolated_stocks() {
        let mut u = world();
        let initial = u.habitats[0].state.accounted_mass_kg();
        u.advance_to(u.time + 86400.0, None, None);
        let h = &u.habitats[0];
        h.validate(u.time).unwrap();
        let escaped = h.state.escaped_gas.unwrap();
        assert!(
            escaped.oxygen_kg > 0.0
                && escaped.nitrogen_kg > 0.0
                && escaped.co2_kg > 0.0
                && escaped.vapor_kg > 0.0
        );
        assert!((h.state.mass_kg() + escaped.mass_kg() - initial).abs() < 1e-8);
        assert!(h.state.mass_kg() < initial);
        assert_eq!(h.state.hydrogen_kg, 0.0);
        assert_eq!(h.state.methane_kg, 0.0);
        assert!(
            (h.state.water_vapor_kg + escaped.vapor_kg - h.totals.crew_vapor_kg
                + h.totals.condensed_kg)
                .abs()
                < 1e-9
        );
        assert!(h.budget.mass_residual_kg.abs() < 1e-9 && h.budget.thermal_residual_j.abs() < 1e-4);
        let d = crate::habitat_diagnostics::explain(h);
        let sum = d
            .heat
            .iter()
            .find(|t| t.id == "sum")
            .unwrap()
            .value
            .unwrap();
        assert!((sum - h.budget.thermal_storage_w).abs() < 1e-5);
        assert!(d.air_and_crew.iter().any(|t| t.id == "breach_nitrogen"));
    }
    #[test]
    fn save_resume_disable_and_source_pins_preserve_exact_history() {
        let mut u = world();
        u.advance_to(u.time + 3671.0, None, None);
        let serialized = crate::save::to_json(&u, "Breach fixture").unwrap();
        let mut resumed = crate::save::from_json(&serialized).unwrap();
        let end = u.time + 7219.0;
        u.advance_to(end, None, None);
        resumed.advance_to(end, None, None);
        assert_eq!(
            crate::research::state_fingerprint(&u).unwrap(),
            crate::research::state_fingerprint(&resumed).unwrap()
        );
        let mut altered: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        altered["universe"]["habitats"][0]["state"]["escaped_gas"]["nitrogen_kg"] = 12345.0.into();
        assert!(crate::save::from_json(&altered.to_string()).is_err());
        let mut bad: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        bad["header"]["data_versions"]["cosmogon-cabin-breach"] = "future/2".into();
        assert!(crate::save::from_json(&bad.to_string()).is_err());
        bad["header"]["data_versions"]
            .as_object_mut()
            .unwrap()
            .remove("cosmogon-cabin-breach");
        assert!(crate::save::from_json(&bad.to_string()).is_err());
        let escaped = u.habitats[0].state.escaped_gas;
        let mut c = u.habitats[0].config.clone();
        c.breach = None;
        u.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        let retained = u.habitats[0].state.escaped_gas;
        // The ordinary edit first completes its old-configuration remainder.
        assert!(retained.unwrap().mass_kg() >= escaped.unwrap().mass_kg());
        u.advance_to(u.time + 3600.0, None, None);
        assert_eq!(u.habitats[0].state.escaped_gas, retained);
        assert_eq!(
            crate::save::data_versions(&u)["cosmogon-cabin-breach"],
            MODEL
        );
        crate::save::from_json(&crate::save::to_json(&u, "Disabled retained").unwrap()).unwrap();
    }
    #[test]
    fn protected_paired_breach_studies_reopen_and_replay_every_actual_row() {
        let u = world();
        let root =
            std::env::temp_dir().join(format!("cosmogon-breach-paired-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut p = crate::habitat_sweep::Plan::linear(
            crate::habitat_sweep::Parameter::BreachAreaM2,
            0.0,
            2e-6,
            2,
            3,
            3600.0,
        )
        .unwrap();
        p.schema_version = 2;
        p.secondary = Some(crate::habitat_factorial::Secondary {
            parameter: crate::habitat_sweep::Parameter::BreachOutsidePressurePa,
            values: [0.0, 90000.0],
        });
        crate::habitat_sweep::execute(&u, 0, &p, &root).unwrap();
        let d = crate::habitat_factorial::read(&root).unwrap();
        assert_eq!(d.analysis["metrics"].as_object().unwrap().len(), 22);
        assert!(
            d.analysis["metrics"]["escaped_gas"]["contrasts"]["average_a"]
                .as_f64()
                .unwrap()
                > 0.0
        );
        assert!(
            d.analysis["metrics"]["escaped_gas"]["contrasts"]["difference_in_differences"]
                .as_f64()
                .unwrap()
                < 0.0
        );
        let replay = crate::habitat_replay::verify(&root).unwrap();
        assert_eq!(replay["verified"], true);
        assert!(crate::habitat_sweep::execute(&u, 0, &p, &root).is_err());
        let csv = std::fs::read_to_string(root.join("run-003/samples.csv")).unwrap();
        assert_eq!(csv.lines().next().unwrap().split(',').count(), 110);
        let mut tampered = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(root.join("run-003/samples.json")).unwrap(),
        )
        .unwrap();
        tampered[1]["habitat"]["budget"]["breach"]["escaped"]["oxygen_kg"] = 1.0.into();
        std::fs::write(root.join("run-003/samples.json"), tampered.to_string()).unwrap();
        assert!(crate::habitat_factorial::read(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
