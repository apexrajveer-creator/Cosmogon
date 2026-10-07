use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ScientificSource {
    pub id: &'static str,
    pub title: &'static str,
    pub url: &'static str,
    pub version: &'static str,
    pub implementation: &'static str,
    pub limitations: &'static str,
}

pub static SCIENTIFIC_SOURCES: &[ScientificSource] = &[
    ScientificSource { id: "horizons", title: "NASA/JPL Horizons", url: "https://ssd.jpl.nasa.gov/horizons/manual.html", version: "embedded cache 1; epoch 2026-01-01 TDB", implementation: "crates/cosmogon_sim/data/horizons_sol.toml", limitations: "Cached initial/reference states, not a live ephemeris service. Frame: ecliptic J2000, barycentric. Per-vector uncertainties are not stored." },
    ScientificSource { id: "solar-facts", title: "Solar System physical reference values", url: "https://ssd.jpl.nasa.gov/astro_par.html", version: "sol.toml@1; reference values also use NASA planetary fact sheets", implementation: "crates/cosmogon_sim/data/sol.toml", limitations: "Hand-entered mixed sources and epochs; the JPL page documents constants, not every stored planet quantity. Per-field uncertainties and full source snapshots are not available." },
    ScientificSource { id: "yoshida", title: "Yoshida (1990), higher-order symplectic integrators", url: "https://doi.org/10.1016/0375-9601(90)90092-3", version: "Physics Letters A 150, 262–268", implementation: "crates/cosmogon_physics/src/nbody.rs", limitations: "The implementation includes encounter refinement and collision handling; validation of those extensions is local, not a claim from the paper." },
    ScientificSource { id: "relativity", title: "Will (2014), general relativity and experiment", url: "https://arxiv.org/abs/1403.7377", version: "Living Reviews in Relativity 17, 4", implementation: "crates/cosmogon_physics/src/nbody.rs", limitations: "Background for weak-field relativity. Cosmogon implements only a central-body 1PN correction, not the full relativistic N-body equations." },
    ScientificSource { id: "collisions", title: "Leinhardt & Stewart (2012), collision outcome scaling", url: "https://arxiv.org/abs/1106.6084", version: "ApJ 745, 79", implementation: "crates/cosmogon_sim/src/collision.rs", limitations: "Reduced gravity-regime scaling. Generated debris is not hydrodynamics or a resolved fragment calculation." },
    ScientificSource { id: "orion", title: "Trapezium catalogue estimates", url: "https://arxiv.org/abs/astro-ph/0510288", version: "Simón-Díaz et al. (2006); loader also cites Kraus et al. (2009)", implementation: "crates/cosmogon_sim/src/nursery.rs", limitations: "Loader uses approximate stellar masses, an assumed age and schematic cluster offsets. The cited paper does not validate every stored value; radii/luminosities/temperatures come from Cosmogon's stellar model." },
    ScientificSource { id: "cosmogon-climate", title: "Cosmogon reduced climate model", url: "https://github.com/ashvyagni/SpaceRender/blob/main/crates/cosmogon_sim/src/planet/environment.rs", version: "grey greenhouse / four-pass feedback, implementation version 1", implementation: "crates/cosmogon_sim/src/planet/environment.rs", limitations: "In-project parametric fit and real-world temperature calibration, not a published general circulation model. Uncertainty is unquantified." },
    ScientificSource { id: "cosmogon-stellar", title: "Cosmogon illustrative stellar evolution", url: "https://github.com/ashvyagni/SpaceRender/blob/main/crates/cosmogon_sim/src/astro/star.rs", version: "piecewise relations, implementation version 1", implementation: "crates/cosmogon_sim/src/astro/star.rs", limitations: "Coarse mass relations and fixed phase timing; remnant fits cite Kalirai/Fryer in source. Not a stellar structure solver." },
];

#[derive(Clone, Debug, Serialize)]
pub struct ModelDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub domain: &'static str,
    pub equations: &'static str,
    pub units: &'static str,
    pub assumptions: &'static str,
    pub validity: &'static str,
    pub limitations: &'static str,
    pub numerical_method: &'static str,
    pub expected_error: &'static str,
    pub validation_cases: &'static [&'static str],
    pub fidelity: &'static str,
    pub runtime_cost: &'static str,
    pub sources: &'static [&'static str],
    pub implementation: &'static str,
}

pub struct PhysicsModelRegistry;
impl PhysicsModelRegistry {
    pub fn models() -> &'static [ModelDescriptor] {
        MODELS
    }
    pub fn get(id: &str) -> Option<&'static ModelDescriptor> {
        MODELS.iter().find(|m| m.id == id)
    }
    pub fn source(id: &str) -> Option<&'static ScientificSource> {
        SCIENTIFIC_SOURCES.iter().find(|s| s.id == id)
    }
}

static MODELS: &[ModelDescriptor] = &[
    ModelDescriptor { id: "kepler", name: "Analytic Kepler propagation", domain: "Orbits", equations: "M = E − e sin(E); two-body orbital elements → Cartesian state", units: "m, m/s, s, rad, kg", assumptions: "Prescribed two-body orbits; moons follow parent rails.", validity: "Bound elliptic orbits; valid element inputs.", limitations: "No mutual perturbations, precession, drag or outgassing. Exact time evaluation of this model is not an exact ephemeris.", numerical_method: "Kepler solve and element transforms", expected_error: "Analytic checks cover periods and orbital transforms; general ephemeris error is unquantified.", validation_cases: &["cosmogon_physics::kepler::tests::orbital_period_earth", "cosmogon_physics::orbital_elements tests"], fidelity: "Reduced two-body", runtime_cost: "O(N) state evaluation", sources: &["solar-facts"], implementation: "crates/cosmogon_physics/src/kepler.rs" },
    ModelDescriptor { id: "nbody", name: "Newtonian N-body gravity", domain: "Dynamics", equations: "aᵢ = Σⱼ G mⱼ (rⱼ−rᵢ) / |rⱼ−rᵢ|³", units: "m, m/s, s, kg", assumptions: "Spherical point gravity outside contact; selected moons remain on rails.", validity: "Non-relativistic gravitational systems outside contact/strong-field regimes.", limitations: "No J2, tides, radiation pressure or non-gravitational forces. Collision/refinement steps interrupt pure fixed-step symplectic evolution.", numerical_method: "Fourth-order Yoshida composition with encounter substeps", expected_error: "Case-specific tests, not a universal bound. See PHYSICS_ENGINE: year-long JPL comparison has larger Mercury/Earth/Moon residuals.", validation_cases: &["nbody::tests::binary_star_conservation", "nbody::tests::high_eccentricity_comet", "nbody::tests::three_body_figure_eight_is_periodic", "horizons::tests::one_year_matches_jpl_ephemeris"], fidelity: "Numerical N-body / rails hybrid", runtime_cost: "O(N²) per force evaluation; refinement increases evaluations", sources: &["yoshida", "horizons"], implementation: "crates/cosmogon_physics/src/nbody.rs" },
    ModelDescriptor { id: "1pn", name: "Central-body relativistic correction", domain: "Dynamics", equations: "Leading 1/c² central-body acceleration correction", units: "m, m/s, s, kg", assumptions: "Weak field and slow motion around the most massive body.", validity: "Weak-field orbital precession; not horizon-scale dynamics.", limitations: "Not full Einstein–Infeld–Hoffmann N-body dynamics, gravitational waves or strong-field GR.", numerical_method: "Velocity-dependent correction in the existing integrator", expected_error: "Mercury regression checks 43 ± 1.5 arcsec/century for its fixture; no global bound.", validation_cases: &["nbody::tests::mercury_relativistic_precession"], fidelity: "First post-Newtonian approximation", runtime_cost: "Additional O(N) correction", sources: &["relativity"], implementation: "crates/cosmogon_physics/src/nbody.rs" },
    ModelDescriptor { id: "bulk", name: "Bulk spherical-body relations", domain: "Physical", equations: "g = GM/R²; ρ = M/(4πR³/3); v_escape = √(2GM/R)", units: "kg, m, m/s², kg/m³, m/s", assumptions: "Spherical body and Newtonian gravity.", validity: "Positive finite mass/radius outside relativistic regimes.", limitations: "No flattening, internal stratification or general relativity.", numerical_method: "Direct scalar evaluation", expected_error: "Floating-point and input uncertainty; propagated uncertainty is not calculated.", validation_cases: &["astro::sol::tests::sol_loads_with_real_values", "orbital_elements::tests::escape_velocity_earth"], fidelity: "Analytic bulk relations", runtime_cost: "O(1) per reading", sources: &["solar-facts"], implementation: "crates/cosmogon_sim/src/astro/body.rs" },
    ModelDescriptor { id: "climate", name: "Reduced planetary climate", domain: "Environment", equations: "T_eq ∝ L^(1/4)(1−A)^(1/4)/√d; T_s = T_eq(1+0.75τ)^(1/4) + calibration − winter", units: "K, AU, L☉, bar, volume fraction", assumptions: "Global mean, parameterized grey greenhouse and water phases.", validity: "Illustrative planetary response; real bodies have temperature calibration offsets.", limitations: "No resolved circulation/weather, spectral radiative transfer, regional forecast or quantified uncertainty.", numerical_method: "Four climate/albedo passes and scheduled environment refresh", expected_error: "Reference temperatures are calibrated; that is not independent predictive validation.", validation_cases: &["planet::environment::tests::model_reproduces_terrestrial_planets_without_calibration", "sandbox climate consequence tests"], fidelity: "Reduced 0D / parametric", runtime_cost: "O(1) per body refresh", sources: &["cosmogon-climate", "solar-facts"], implementation: "crates/cosmogon_sim/src/planet/environment.rs" },
    ModelDescriptor { id: "stellar", name: "Illustrative stellar evolution", domain: "Stars", equations: "Piecewise mass–luminosity/radius/lifetime relations; fixed giant phase and remnant fits", units: "M☉, L☉, m, K, s", assumptions: "Coarse single-star evolution; limited metallicity treatment.", validity: "Illustrative living stars and reduced remnant evolution.", limitations: "Not MESA or stellar structure. Binary interactions, detailed tracks and uncertainty are absent.", numerical_method: "Deterministic analytic relations plus scheduled transitions", expected_error: "Unquantified; tests check internal continuity/phase behavior, not a full observational fit.", validation_cases: &["astro::star::tests::present_day_sun_is_calibrated", "astro::star::tests::young_sun_was_fainter_and_old_sun_dies"], fidelity: "Reduced / educational", runtime_cost: "O(1) per star evaluation", sources: &["cosmogon-stellar"], implementation: "crates/cosmogon_sim/src/astro/star.rs" },
    ModelDescriptor { id: "collision", name: "Gravity-regime collision outcomes", domain: "Impacts", equations: "Specific impact energy / disruption threshold → largest-remnant fraction", units: "kg, m/s, J/kg", assumptions: "Spherical gravity-dominated bodies and reduced threshold scaling.", validity: "Gravity regime, not detailed material-strength or fluid impact calculations.", limitations: "Approximate geometry/debris; not a resolved SPH/hydrodynamic impact.", numerical_method: "Contact detection, scaling relations and generated remnant/debris edits", expected_error: "Unquantified outside published-scale regression inputs.", validation_cases: &["collision::tests::disruption_threshold_matches_published_scale", "collision::tests::gentle_impacts_merge_and_violent_ones_shatter"], fidelity: "Reduced scaling law", runtime_cost: "Contact-dependent; debris raises later N-body cost", sources: &["collisions"], implementation: "crates/cosmogon_sim/src/collision.rs" },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_links_are_unique_complete_and_resolved() {
        let mut ids = std::collections::BTreeSet::new();
        for m in MODELS {
            assert!(ids.insert(m.id));
            for text in [
                m.equations,
                m.units,
                m.assumptions,
                m.validity,
                m.limitations,
                m.numerical_method,
                m.expected_error,
                m.fidelity,
                m.runtime_cost,
                m.implementation,
            ] {
                assert!(!text.is_empty());
            }
            assert!(!m.validation_cases.is_empty());
            for s in m.sources {
                assert!(PhysicsModelRegistry::source(s).is_some(), "{s}");
            }
        }
        let mut sources = std::collections::BTreeSet::new();
        for s in SCIENTIFIC_SOURCES {
            assert!(sources.insert(s.id));
            assert!(s.url.starts_with("https://"));
        }
    }
}
