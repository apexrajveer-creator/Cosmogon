use crate::astro::{Quality, SOLAR_MASS};
use crate::sandbox::{BodyProperty, Edit, StarProperty};
use crate::{BodyRef, Scenario, Universe};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ResearchTarget {
    Star(u32),
    Body(BodyRef),
}
impl ResearchTarget {
    pub fn system(self) -> u32 {
        match self {
            Self::Star(s) => s,
            Self::Body(r) => r.system,
        }
    }
    pub fn name(self, u: &Universe) -> Option<&str> {
        let sys = u.systems.get(self.system() as usize)?;
        match self {
            Self::Star(_) => Some(&sys.star.name),
            Self::Body(r) => sys
                .bodies
                .get(r.body as usize)
                .filter(|b| b.exists())
                .map(|b| b.name.as_str()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataState {
    Measured,
    Derived,
    Estimated,
    Modelled,
    MlInference,
    Procedural,
    UserModified,
    Unknown,
}
impl DataState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Measured => "MEASURED",
            Self::Derived => "DERIVED",
            Self::Estimated => "ESTIMATED",
            Self::Modelled => "MODELLED",
            Self::MlInference => "ML INFERENCE",
            Self::Procedural => "PROCEDURAL",
            Self::UserModified => "USER MODIFIED",
            Self::Unknown => "UNKNOWN",
        }
    }
}
impl From<Quality> for DataState {
    fn from(q: Quality) -> Self {
        match q {
            Quality::Measured => Self::Measured,
            Quality::Derived => Self::Derived,
            Quality::Estimated => Self::Estimated,
            Quality::Procedural => Self::Procedural,
            Quality::UserModified => Self::UserModified,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterId {
    Mass,
    Radius,
    Rotation,
    Tilt,
    Albedo,
    Pressure,
    Co2,
    Water,
    Gravity,
    Density,
    Escape,
    Temperature,
    Equilibrium,
    Insolation,
    Habitability,
    StarMass,
    StarAge,
    Metallicity,
    StarRadius,
    Luminosity,
    StarTemperature,
    Steps,
    X,
    Y,
    Z,
    Vx,
    Vy,
    Vz,
    Eccentricity,
    SemiMajor,
    Inclination,
    OrbitalPeriod,
    SpecificEnergy,
    SpecificAngularMomentum,
    SystemEnergy,
    SystemAngularMomentum,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParameterDescriptor {
    pub id: ParameterId,
    pub key: &'static str,
    pub name: &'static str,
    pub section: &'static str,
    pub unit: &'static str,
    pub editable: bool,
    pub model: &'static str,
    pub formula: &'static str,
    pub dependencies: &'static [ParameterId],
    pub limits: &'static str,
    pub effects: &'static str,
}
macro_rules! p {
    ($id:ident, $key:literal, $name:literal, $section:literal, $unit:literal, $editable:literal, $model:literal, $formula:literal, [$($dep:ident),*], $limits:literal, $effects:literal) => {
        ParameterDescriptor { id: ParameterId::$id, key: $key, name: $name, section: $section, unit: $unit, editable: $editable, model: $model, formula: $formula, dependencies: &[$(ParameterId::$dep),*], limits: $limits, effects: $effects }
    }
}
pub static PARAMETERS: &[ParameterDescriptor] = &[
    p!(
        Mass,
        "mass_kg",
        "Mass",
        "Physical",
        "kg",
        true,
        "bulk",
        "Stored input",
        [],
        "1e-12–1e45 kg at this numeric control boundary; unusual densities are warned",
        "Gravity, density, escape speed and dynamics"
    ),
    p!(
        Radius,
        "radius_m",
        "Radius",
        "Physical",
        "m",
        true,
        "bulk",
        "Stored input",
        [],
        "1e-6–1e20 m at this numeric control boundary",
        "Gravity, density, contact geometry and surface rendering"
    ),
    p!(
        Gravity,
        "gravity_m_s2",
        "Surface gravity",
        "Physical",
        "m/s²",
        false,
        "bulk",
        "GM/R²",
        [Mass, Radius],
        "Spherical Newtonian relation",
        "Launch cost and existing habitability assessment"
    ),
    p!(
        Density,
        "density_kg_m3",
        "Mean density",
        "Physical",
        "kg/m³",
        false,
        "bulk",
        "M/(4πR³/3)",
        [Mass, Radius],
        "Uniform bulk sphere",
        "Unusual-density warnings"
    ),
    p!(
        Escape,
        "escape_m_s",
        "Escape speed",
        "Physical",
        "m/s",
        false,
        "bulk",
        "√(2GM/R)",
        [Mass, Radius],
        "Newtonian; not relativistic escape",
        "Existing launch cost estimates"
    ),
    p!(
        Rotation,
        "rotation_s",
        "Rotation period",
        "Rotation",
        "s",
        true,
        "bulk",
        "Stored input; negative is retrograde",
        [],
        "Finite, nonzero; disabled for tidally locked bodies",
        "Surface spin and existing climate/habitability context"
    ),
    p!(
        Tilt,
        "tilt_rad",
        "Axial tilt",
        "Rotation",
        "rad",
        true,
        "bulk",
        "Stored input",
        [],
        "Finite angle",
        "Seasons in terrain/habitability representation"
    ),
    p!(
        Albedo,
        "albedo",
        "Albedo",
        "Environment",
        "1",
        true,
        "climate",
        "Stored reflected fraction",
        [],
        "0–1; procedural climate may recompute it",
        "Equilibrium and surface temperature"
    ),
    p!(
        Pressure,
        "pressure_bar",
        "Surface pressure",
        "Environment",
        "bar",
        true,
        "climate",
        "Stored bulk atmosphere",
        [],
        "Finite, zero or positive",
        "Greenhouse and water stability; habitability"
    ),
    p!(
        Co2,
        "co2_fraction",
        "CO₂ volume fraction",
        "Environment",
        "1",
        true,
        "climate",
        "Stored atmosphere fraction",
        [],
        "0–1; other components are renormalized",
        "Greenhouse optical depth and resulting temperature"
    ),
    p!(
        Water,
        "water_earth_oceans",
        "Water inventory",
        "Environment",
        "Earth oceans",
        true,
        "climate",
        "Stored equivalent water inventory",
        [],
        "Finite, zero or positive",
        "Water phases, ocean/ice coverage and terrain sea level"
    ),
    p!(
        Temperature,
        "temperature_k",
        "Surface / reference temperature",
        "Environment",
        "K",
        false,
        "climate",
        "T_eq(1+0.75τ)^(1/4) + bias − winter",
        [Equilibrium, Pressure, Co2, Water],
        "Reduced global mean; giants use internal-heat approximation",
        "Water state, habitability, biosphere and civilization environment"
    ),
    p!(
        Equilibrium,
        "equilibrium_k",
        "Equilibrium temperature",
        "Environment",
        "K",
        false,
        "climate",
        "278.6 L^(1/4)(1−A)^(1/4)/√d_AU",
        [Albedo, Insolation],
        "Global radiative-equilibrium approximation",
        "Surface climate"
    ),
    p!(
        Insolation,
        "insolation_earth",
        "Incident flux",
        "Environment",
        "Earth flux",
        false,
        "climate",
        "L/L☉ divided by (d/AU)²",
        [Luminosity, X, Y, Z],
        "Current stellar distance; not a climate time average",
        "Radiative equilibrium and environment"
    ),
    p!(
        Habitability,
        "habitability_score",
        "Habitability score",
        "Environment",
        "1",
        false,
        "climate",
        "Existing multi-factor assessment",
        [Temperature, Pressure, Water, Gravity],
        "Heuristic score, not a measured probability",
        "Existing biosphere/civilization models"
    ),
    p!(
        StarMass,
        "star_mass_kg",
        "Stellar mass",
        "Star",
        "kg",
        true,
        "stellar",
        "Stored input; converted from solar masses",
        [],
        "Living stars: 0.08–150 M☉; compact remnants positive",
        "Stellar relations, gravity and planetary climate"
    ),
    p!(
        StarAge,
        "star_age_s",
        "Stellar age",
        "Star",
        "s",
        true,
        "stellar",
        "Simulation time − formation time",
        [],
        "Positive finite",
        "Evolution phase, luminosity and lifetime context"
    ),
    p!(
        Metallicity,
        "metallicity_dex",
        "Metallicity [Fe/H]",
        "Star",
        "dex",
        true,
        "stellar",
        "Stored input",
        [],
        "Finite; model metallicity response is limited",
        "Stored composition context and reconstructed stellar relations"
    ),
    p!(
        StarRadius,
        "star_radius_m",
        "Stellar radius",
        "Star",
        "m",
        false,
        "stellar",
        "Existing mass/phase relation",
        [StarMass, StarAge],
        "Illustrative stellar model",
        "Stellar rendering and engulfment context"
    ),
    p!(
        Luminosity,
        "luminosity_solar",
        "Luminosity",
        "Star",
        "L☉",
        false,
        "stellar",
        "Existing mass/age/phase relation",
        [StarMass, StarAge],
        "Not a detailed stellar structure solution",
        "Planetary flux and climate"
    ),
    p!(
        StarTemperature,
        "star_temperature_k",
        "Stellar effective temperature",
        "Star",
        "K",
        false,
        "stellar",
        "Existing mass/phase relation",
        [StarMass, StarAge],
        "Illustrative stellar model",
        "Star colour and environment context"
    ),
    p!(
        Steps,
        "steps_per_orbit",
        "Solver steps per orbit",
        "Solver",
        "steps/orbit",
        true,
        "nbody",
        "η = 2π / steps_per_orbit",
        [],
        "8–100000 (integrator clamp); dynamic systems only",
        "Integration resolution and runtime cost; no guaranteed error tolerance"
    ),
    p!(
        X,
        "position_x_m",
        "Position X",
        "State vector",
        "m",
        true,
        "nbody",
        "Current system-frame state",
        [],
        "Finite; |component| ≤ 1e24 m at this control boundary",
        "Orbit, stellar distance and future motion"
    ),
    p!(
        Y,
        "position_y_m",
        "Position Y",
        "State vector",
        "m",
        true,
        "nbody",
        "Current system-frame state",
        [],
        "Finite; |component| ≤ 1e24 m at this control boundary",
        "Orbit and future motion"
    ),
    p!(
        Z,
        "position_z_m",
        "Position Z",
        "State vector",
        "m",
        true,
        "nbody",
        "Current system-frame state",
        [],
        "Finite; |component| ≤ 1e24 m at this control boundary",
        "Orbit and future motion"
    ),
    p!(
        Vx,
        "velocity_x_m_s",
        "Velocity X",
        "State vector",
        "m/s",
        true,
        "nbody",
        "Current system-frame state",
        [],
        "Finite; |component| ≤ 1e8 m/s; high speeds exceed model validity",
        "Orbit and future motion"
    ),
    p!(
        Vy,
        "velocity_y_m_s",
        "Velocity Y",
        "State vector",
        "m/s",
        true,
        "nbody",
        "Current system-frame state",
        [],
        "Finite; |component| ≤ 1e8 m/s; high speeds exceed model validity",
        "Orbit and future motion"
    ),
    p!(
        Vz,
        "velocity_z_m_s",
        "Velocity Z",
        "State vector",
        "m/s",
        true,
        "nbody",
        "Current system-frame state",
        [],
        "Finite; |component| ≤ 1e8 m/s; high speeds exceed model validity",
        "Orbit and future motion"
    ),
    p!(Eccentricity, "eccentricity", "Orbital eccentricity", "Orbit", "dimensionless", false, "nbody", "|v × (r × v)/μ − r/|r||", [X,Y,Z,Vx,Vy,Vz,Mass], "Instantaneous parent-relative osculating orbit, μ=G(parent mass+body mass); not a future trajectory", "Orbit classification; 0 is circular, 0–1 elliptic, >1 hyperbolic"),
    p!(SemiMajor, "semi_major_axis_m", "Semi-major axis", "Orbit", "m", false, "nbody", "−μ/(2ε)", [SpecificEnergy], "Parent-relative osculating value; negative for hyperbolic motion, unavailable at parabolic singularity", "Instantaneous orbital scale"),
    p!(Inclination, "inclination_rad", "Orbital inclination", "Orbit", "rad", false, "nbody", "atan2(√(h_x²+h_y²), h_z)", [X,Y,Z,Vx,Vy,Vz], "Relative to simulation system XY plane; ecliptic J2000 orientation in Solar Lab", "Instantaneous orbital plane"),
    p!(OrbitalPeriod, "orbital_period_s", "Osculating orbital period", "Orbit", "s", false, "kepler", "2π√(a³/μ)", [SemiMajor], "Bound osculating ellipse only; perturbed systems do not repeat exactly after this period", "Two-body comparison timescale"),
    p!(SpecificEnergy, "specific_energy_j_kg", "Specific orbital energy", "Orbit", "J/kg", false, "nbody", "v_rel²/2 − μ/|r_rel|", [X,Y,Z,Vx,Vy,Vz,Mass], "Instantaneous two-body parent-relative quantity, not conserved in a perturbed system", "Bound/unbound diagnosis"),
    p!(SpecificAngularMomentum, "specific_angular_momentum_m2_s", "Specific angular momentum", "Orbit", "m²/s", false, "nbody", "|r_rel × v_rel|", [X,Y,Z,Vx,Vy,Vz], "Parent-relative magnitude; not a whole-system conservation test", "Orbital-plane and eccentricity diagnosis"),
    p!(SystemEnergy, "system_energy_j", "System Newtonian energy", "Diagnostics", "J", false, "nbody", "Σ m v²/2 − Σ_i<j G m_i m_j/r_ij", [], "Dynamic particles only, including active companion; excludes moons on rails, 1PN energy and internal/thermal energy. Available up to 256 particles", "Closed fixed-mass Newtonian runs may inspect drift; edits/collisions/evolution change the budget"),
    p!(SystemAngularMomentum, "system_angular_momentum_kg_m2_s", "System angular momentum", "Diagnostics", "kg·m²/s", false, "nbody", "|Σ m r × v|", [], "Magnitude about simulation system origin; dynamic particles only, excludes rails. Available up to 256 particles", "Diagnostic; not guaranteed conserved under edits/collisions/stellar evolution or incomplete rails coupling"),

];
impl ParameterId {
    pub fn descriptor(self) -> &'static ParameterDescriptor {
        PARAMETERS
            .iter()
            .find(|p| p.id == self)
            .expect("registered parameter")
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Reading {
    pub value: f64,
    pub state: DataState,
    pub source: String,
    pub uncertainty: Option<f64>,
}

/// Legacy star records have no provenance field. Use the loader's scenario and
/// initial IDs, never a user-visible name or invented planets, to identify its inputs.
pub fn star_reading_state(u: &Universe, s: u32, id: ParameterId) -> (DataState, &'static str) {
    use ParameterId::*;
    let changed = u.edits.iter().any(|e| match &e.edit {
        Edit::SetStar { system, property } if *system == s => matches!(
            (id, property),
            (StarMass, StarProperty::Mass(_) | StarProperty::Kind(_))
                | (StarAge, StarProperty::Age(_))
                | (Metallicity, StarProperty::Metallicity(_))
        ),
        _ => false,
    });
    if changed {
        return (DataState::UserModified, "Sandbox edit journal");
    }
    let solar = matches!(
        u.settings.scenario,
        Scenario::Sol | Scenario::SolarSystemLab
    ) && s == 0;
    let orion = u.settings.scenario == Scenario::SolarSystemLab
        && (1..=4).contains(&s)
        && u.nurseries.iter().any(|n| n.name == "Orion Nebula");
    if matches!(id, StarRadius | Luminosity | StarTemperature) {
        return (DataState::Modelled, "cosmogon-stellar");
    }
    if solar {
        (
            DataState::Estimated,
            "solar-facts (solar reference values; age/composition assumptions)",
        )
    } else if orion {
        (
            DataState::Estimated,
            "orion (catalogue estimates; assumed age/composition)",
        )
    } else {
        (DataState::Procedural, "Seeded stellar generation")
    }
}

pub fn read_parameter(u: &Universe, target: ResearchTarget, id: ParameterId) -> Option<Reading> {
    use ParameterId::*;
    target.name(u)?;
    let sys = u.systems.get(target.system() as usize)?;
    let (value, state, source) = if matches!(id, SystemEnergy | SystemAngularMomentum) {
        sys.dynamics.as_ref()?;
        // Bound observer work independently of how many bodies a sandbox can contain.
        if sys.bodies.len() > 256 {
            return None;
        }
        let (particles, _) = sys.particles(u.time);
        if particles.len() > 256 {
            return None;
        }
        let value = if id == SystemEnergy {
            cosmogon_physics::nbody::energy(&particles) / crate::astro::G
        } else {
            cosmogon_physics::nbody::angular_momentum(&particles).length() / crate::astro::G
        };
        (value, DataState::Derived, "Dynamic-particle Newtonian diagnostic; excludes rails and 1PN energy. Drift alone is not an accuracy certificate.".into())
    } else if id == Steps {
        let d = sys.dynamics.as_ref()?;
        let modified = u.edits.iter().any(
            |e| matches!(&e.edit, Edit::SetPhysics { system, .. } if *system == target.system()),
        );
        (
            d.settings.steps_per_orbit,
            if modified {
                DataState::UserModified
            } else {
                DataState::Modelled
            },
            "Current solver configuration".to_string(),
        )
    } else if let ResearchTarget::Star(s) = target {
        let star = &sys.star;
        let value = match id {
            StarMass => star.mass * SOLAR_MASS,
            StarAge => star.age(u.time),
            Metallicity => star.metallicity,
            StarRadius => star.current_radius(u.time),
            Luminosity => star.luminosity(u.time),
            StarTemperature => star.temperature_at(u.time),
            _ => return None,
        };
        let (state, source) = star_reading_state(u, s, id);
        (value, state, source.to_string())
    } else if let ResearchTarget::Body(r) = target {
        let b = sys.bodies.get(r.body as usize)?;
        let value = match id {
            Mass => b.mass,
            Radius => b.radius,
            Rotation if !b.tidally_locked => b.rotation_period,
            Tilt => b.axial_tilt,
            Albedo => b.albedo,
            Pressure if b.kind.has_surface() => b.atmosphere.pressure_bar,
            Co2 if b.kind.has_surface() => b.atmosphere.co2,
            Water if b.kind.has_surface() => b.hydro.water_inventory,
            Gravity => b.gravity(),
            Density => b.density(),
            Escape => b.escape_velocity(),
            Temperature if !b.kind.is_stellar() => b.temperature,
            Equilibrium if !b.kind.is_stellar() => b.equilibrium_temperature,
            Insolation => crate::planet::environment::insolation(
                &sys.star,
                u.time,
                (sys.body_state(r.body as usize, u.time).pos - sys.star_local_position(u.time))
                    .length(),
            ),
            Habitability if b.kind.has_surface() => u.habitability(r).score,
            Eccentricity
            | SemiMajor
            | Inclination
            | OrbitalPeriod
            | SpecificEnergy
            | SpecificAngularMomentum => {
                let state = sys.body_state(r.body as usize, u.time);
                let parent = sys.parent_state(r.body as usize, u.time);
                let relative = state.pos - parent.pos;
                let velocity = state.vel - parent.vel;
                let distance = relative.length();
                let mu = sys.parent_mu(r.body as usize) + b.mu();
                if !(distance > 0.0 && distance.is_finite() && mu > 0.0 && mu.is_finite()) {
                    return None;
                }
                if id == SpecificEnergy {
                    0.5 * velocity.length_squared() - mu / distance
                } else if id == SpecificAngularMomentum {
                    relative.cross(velocity).length()
                } else {
                    let el = sys.osculating(r.body as usize, u.time);
                    match id {
                        Eccentricity => el.eccentricity,
                        SemiMajor => el.semi_major_axis,
                        Inclination => el.inclination,
                        OrbitalPeriod if el.is_bound() => {
                            std::f64::consts::TAU
                                * el.semi_major_axis
                                * (el.semi_major_axis / mu).sqrt()
                        }
                        _ => return None,
                    }
                }
            }
            X | Y | Z | Vx | Vy | Vz => {
                let s = sys.body_state(r.body as usize, u.time);
                match id {
                    X => s.pos.x,
                    Y => s.pos.y,
                    Z => s.pos.z,
                    Vx => s.vel.x,
                    Vy => s.vel.y,
                    _ => s.vel.z,
                }
            }
            _ => return None,
        };
        let field = match id {
            Mass => "mass",
            Radius => "radius",
            Rotation => "rotation_period",
            Tilt => "axial_tilt",
            Albedo => "albedo",
            Pressure | Co2 => "atmosphere",
            Water => "water",
            X | Y | Z | Vx | Vy | Vz => "position",
            _ => "temperature",
        };
        let state = if matches!(
            id,
            Gravity
                | Density
                | Escape
                | Eccentricity
                | SemiMajor
                | Inclination
                | OrbitalPeriod
                | SpecificEnergy
                | SpecificAngularMomentum
        ) {
            DataState::Derived
        } else if matches!(id, Temperature | Equilibrium | Habitability | Insolation) {
            DataState::Modelled
        } else if matches!(id, X | Y | Z | Vx | Vy | Vz) {
            if b.quality("position") == Quality::UserModified
                || b.quality("velocity") == Quality::UserModified
            {
                DataState::UserModified
            } else {
                DataState::Modelled
            }
        } else {
            b.quality(field).into()
        };
        let source = if matches!(state, DataState::Modelled | DataState::Derived) {
            format!("{}; inputs: {}", id.descriptor().model, b.source_label())
        } else if state == DataState::UserModified {
            format!(
                "Sandbox edit journal; original source: {}",
                b.source_label()
            )
        } else {
            b.source_label()
        };
        (value, state, source)
    } else {
        return None;
    };
    value.is_finite().then_some(Reading {
        value,
        state,
        source,
        uncertainty: None,
    })
}

pub fn parameter_edit(
    u: &Universe,
    target: ResearchTarget,
    id: ParameterId,
    value: f64,
) -> Result<Edit, String> {
    use ParameterId::*;
    if !value.is_finite() {
        return Err("The value must be finite".into());
    }
    if !id.descriptor().editable || read_parameter(u, target, id).is_none() {
        return Err("This value is derived, locked or unavailable for this object".into());
    }
    let positive = matches!(id, Mass | Radius | StarMass | StarAge);
    if id == Mass && !(1e-12..=1e45).contains(&value) {
        return Err("Mass must be within this control boundary (1e-12–1e45 kg)".into());
    }
    if id == Radius && !(1e-6..=1e20).contains(&value) {
        return Err("Radius must be within this control boundary (1e-6–1e20 m)".into());
    }
    if positive && value <= 0.0 {
        return Err("The value must be positive".into());
    }
    if matches!(id, Pressure | Water) && value < 0.0 {
        return Err("The value must be zero or positive".into());
    }
    if matches!(id, Albedo | Co2) && !(0.0..=1.0).contains(&value) {
        return Err("The fraction must be in 0–1".into());
    }
    if id == Rotation && value == 0.0 {
        return Err("Rotation period must be nonzero".into());
    }
    if id == Steps {
        if !(8.0..=100_000.0).contains(&value) {
            return Err("Choose 8–100000 steps per orbit".into());
        }
        let mut settings = u
            .system(target.system())
            .dynamics
            .as_ref()
            .ok_or("Dynamic gravity is not active")?
            .settings;
        settings.preset = crate::astro::dynamics::PhysicsPreset::Custom;
        settings.steps_per_orbit = value;
        return Ok(Edit::SetPhysics {
            system: target.system(),
            nbody: true,
            settings,
        });
    }
    match target {
        ResearchTarget::Star(system) => {
            let property = match id {
                StarMass => StarProperty::Mass(value / SOLAR_MASS),
                StarAge => StarProperty::Age(value),
                Metallicity => StarProperty::Metallicity(value),
                _ => return Err("Not a stellar input".into()),
            };
            Ok(Edit::SetStar { system, property })
        }
        ResearchTarget::Body(body) => {
            let property = match id {
                Mass => BodyProperty::Mass(value),
                Radius => BodyProperty::Radius(value),
                Rotation => BodyProperty::RotationPeriod(value),
                Tilt => BodyProperty::AxialTilt(value),
                Albedo => BodyProperty::Albedo(value),
                Pressure => BodyProperty::SurfacePressure(value),
                Co2 => BodyProperty::Co2Fraction(value),
                Water => BodyProperty::WaterInventory(value),
                X | Y | Z | Vx | Vy | Vz => {
                    let max = if matches!(id, X | Y | Z) { 1e24 } else { 1e8 };
                    if value.abs() > max {
                        return Err(format!(
                            "Component magnitude exceeds this control's finite bound ({max:e})"
                        ));
                    }
                    let mut state = u.system(body.system).body_state(body.body as usize, u.time);
                    match id {
                        X => state.pos.x = value,
                        Y => state.pos.y = value,
                        Z => state.pos.z = value,
                        Vx => state.vel.x = value,
                        Vy => state.vel.y = value,
                        _ => state.vel.z = value,
                    }
                    return Ok(Edit::SetState { body, state });
                }
                _ => return Err("Not an editable body input".into()),
            };
            Ok(Edit::SetProperty { body, property })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orbital_readings_use_parent_relative_states_and_leave_universe_unchanged() {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let earth = ResearchTarget::Body(BodyRef { system: 0, body: 2 });
        let before = serde_json::to_string(&u).unwrap();
        assert!(
            (read_parameter(&u, earth, ParameterId::OrbitalPeriod)
                .unwrap()
                .value
                / 86400.0
                - 365.25)
                .abs()
                < 2.0
        );
        assert!(
            read_parameter(&u, earth, ParameterId::Eccentricity)
                .unwrap()
                .value
                < 0.02
        );
        assert!(
            read_parameter(&u, earth, ParameterId::SpecificEnergy)
                .unwrap()
                .value
                < 0.0
        );
        assert!(
            read_parameter(&u, earth, ParameterId::SystemEnergy)
                .unwrap()
                .value
                < 0.0
        );
        assert!(
            read_parameter(&u, earth, ParameterId::SystemAngularMomentum)
                .unwrap()
                .value
                > 0.0
        );
        let moon_index = u.systems[0]
            .bodies
            .iter()
            .position(|b| b.name == "Moon")
            .unwrap();
        let moon = ResearchTarget::Body(BodyRef {
            system: 0,
            body: moon_index as u32,
        });
        assert!(
            (read_parameter(&u, moon, ParameterId::OrbitalPeriod)
                .unwrap()
                .value
                / 86400.0
                - 27.3)
                .abs()
                < 2.0
        );
        assert_eq!(serde_json::to_string(&u).unwrap(), before);
        u.apply_edit(parameter_edit(&u, earth, ParameterId::Vx, 300_000.0).unwrap())
            .unwrap();
        assert!(
            read_parameter(&u, earth, ParameterId::Eccentricity)
                .unwrap()
                .value
                > 1.0
        );
        assert!(read_parameter(&u, earth, ParameterId::OrbitalPeriod).is_none());
        assert!(
            read_parameter(&u, earth, ParameterId::SemiMajor)
                .unwrap()
                .value
                < 0.0
        );
    }
    fn lab() -> Universe {
        Universe::new(crate::UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        })
    }
    #[test]
    fn reads_and_descriptors_leave_universe_bit_identical() {
        let u = lab();
        let before = serde_json::to_string(&u).unwrap();
        let mut keys = std::collections::BTreeSet::new();
        for p in PARAMETERS {
            assert!(keys.insert(p.key));
            assert!(super::super::PhysicsModelRegistry::get(p.model).is_some());
            for dep in p.dependencies {
                assert!(PARAMETERS.iter().any(|p| p.id == *dep));
            }
            for target in [
                ResearchTarget::Star(0),
                ResearchTarget::Body(BodyRef { system: 0, body: 2 }),
            ] {
                if let Some(r) = read_parameter(&u, target, p.id) {
                    assert!(r.value.is_finite());
                    assert!(r.uncertainty.is_none());
                }
            }
        }
        assert_eq!(serde_json::to_string(&u).unwrap(), before);
    }
    #[test]
    fn catalogue_stars_have_estimates_and_only_edited_field_is_modified() {
        let mut u = lab();
        assert_eq!(
            star_reading_state(&u, 1, ParameterId::StarMass).0,
            DataState::Estimated
        );
        assert_eq!(
            star_reading_state(&u, 1, ParameterId::Luminosity).0,
            DataState::Modelled
        );
        u.apply_edit(Edit::SetStar {
            system: 1,
            property: StarProperty::Metallicity(0.1),
        })
        .unwrap();
        assert_eq!(
            star_reading_state(&u, 1, ParameterId::Metallicity).0,
            DataState::UserModified
        );
        assert_eq!(
            star_reading_state(&u, 1, ParameterId::StarMass).0,
            DataState::Estimated
        );
        let u = Universe::new(crate::UniverseSettings {
            scenario: Scenario::EmptySystem,
            ..Default::default()
        });
        assert_eq!(
            star_reading_state(&u, 0, ParameterId::StarMass).0,
            DataState::Procedural
        );
    }
    #[test]
    fn edits_rederive_gravity_and_state_component_preserves_other_components() {
        let mut u = lab();
        let t = ResearchTarget::Body(BodyRef { system: 0, body: 2 });
        let radius = read_parameter(&u, t, ParameterId::Radius).unwrap().value;
        let g = read_parameter(&u, t, ParameterId::Gravity).unwrap().value;
        let command = parameter_edit(&u, t, ParameterId::Radius, radius * 2.0).unwrap();
        u.apply_edit(command).unwrap();
        assert!(
            (read_parameter(&u, t, ParameterId::Gravity).unwrap().value - g / 4.0).abs() < 1e-10
        );
        let state = u.system(0).body_state(2, u.time);
        let edit = parameter_edit(&u, t, ParameterId::Vx, state.vel.x + 100.0).unwrap();
        if let Edit::SetState { state: new, .. } = edit {
            assert_eq!(new.pos, state.pos);
            assert_eq!(new.vel.y, state.vel.y);
            assert_eq!(new.vel.z, state.vel.z);
            assert_eq!(new.vel.x, state.vel.x + 100.0);
        } else {
            panic!("state edit");
        }
    }
    #[test]
    fn unsafe_or_derived_inputs_are_rejected_before_edit() {
        let u = lab();
        let t = ResearchTarget::Body(BodyRef { system: 0, body: 2 });
        for (p, v) in [
            (ParameterId::Mass, f64::MAX),
            (ParameterId::Mass, f64::NAN),
            (ParameterId::Radius, 0.0),
            (ParameterId::Gravity, 20.0),
            (ParameterId::Pressure, -1.0),
            (ParameterId::Co2, 2.0),
            (ParameterId::Rotation, 0.0),
            (ParameterId::Vx, 1e10),
            (ParameterId::Steps, 4.0),
        ] {
            assert!(parameter_edit(&u, t, p, v).is_err());
        }
        assert!(read_parameter(
            &u,
            ResearchTarget::Body(BodyRef {
                system: 99,
                body: 99
            }),
            ParameterId::Mass
        )
        .is_none());
    }
}
