# Cosmogon

**A free universe sandbox that remembers.** Throw planets and black holes, watch stars live and die, pull
back from a city street to the cosmic web — and see what happens to the people. Start from the real Solar
System (NASA/JPL data); every change runs through orbits, climate, life and civilizations that discover
technology through causes, not timers.

![Home](docs/images/home.jpg)

| A feeding black hole: ray-traced lensing and a Doppler-beamed disk | The Milky Way from 150,000 light-years |
|---|---|
| ![Black hole](docs/images/black_hole.jpg) | ![Milky Way](docs/images/milky_way.jpg) |
| **Halley's Comet: ion and dust tails** | **The Orion Nebula, where stars are born** |
| ![Halley](docs/images/halley.jpg) | ![Orion](docs/images/orion_nebula.jpg) |
| **A Dyson swarm around the Sun** | **An asteroid strike: fireball and shock ring** |
| ![Dyson swarm](docs/images/dyson_swarm.jpg) | ![Impact](docs/images/impact_flash.jpg) |
| **Guided tours for a first look** | **Rival nations on the real Earth, 4000 CE** |
| ![Tour](docs/images/guided_tour.jpg) | ![Nations](docs/images/nations_real_earth.jpg) |

## Download
Get the latest **macOS `.dmg`** or **Windows `.exe`** from
[GitHub Releases](https://github.com/ashvyagni/SpaceRender/releases). Nothing else needs to be installed.
The builds are unsigned, so the first launch needs one extra click — see the release notes.

## Try it
* **macOS:** open `dist/Cosmogon-<version>-macOS.dmg`, drag Cosmogon to Applications, double-click.
  (Unsigned builds: right-click → *Open* the first time.) To build it yourself: `scripts/package-macos.sh`.
* **Windows / Linux:** push a `v*` tag; [`.github/workflows/release.yml`](.github/workflows/release.yml) builds a
  Windows installer and portable `.exe`, a universal macOS DMG and a Linux tarball.
* **From source:** `cargo run --release -p cosmogon` (see [BUILDING.md](BUILDING.md)).

**NEW SANDBOX** offers seven templates: the **Solar System Lab** (today's Solar System from JPL Horizons, with
N-body gravity), **Dawn of Humanity** (the real Solar System 200,000 years ago, with early humans), a
**habitable world**, a **procedural star system**, a **stellar neighbourhood**, an **empty system** to build
yourself, and **custom**. **SCENARIOS** has four **guided tours** and ready-made experiments (no Moon, 2× Jupiter,
Chicxulub today, Theia, a black hole flyby, a second sun, the Sun as a red giant or a dying supergiant,
Halley's Comet, a Kardashev II humanity and more). **REAL UNIVERSE** shows the real data
read-only; clone it to experiment. Time runs from real time to 100 million years per second.

## The sandbox
* **+ Create** a planet, moon, giant, dwarf planet, asteroid or comet: choose mass, radius, where it orbits and how fast.
* **Launch** an object at any body: the predicted trajectory and any impact are shown before you release it.
* **Edit anything** in the inspector — mass, radius, gravity, day length, tilt, orbit, atmosphere, water, the star's
  mass and age — in the units you like (typed values accept scientific notation). Every change is undoable.
* **Physics**: switch between fixed (Kepler) orbits and dynamic N-body gravity; choose Fast, Balanced, Accurate or
  Research. Collisions merge bodies; impacts leave craters, cause impact winters and extinctions.
* **Consequences**: change an orbit or the Sun and the climate, habitability and civilizations respond; the
  chronicle reports what happened and why.
* **Honest data**: every value is labelled MEASURED, DERIVED, ESTIMATED, PROCEDURAL or USER MODIFIED.
* Sandboxes save to their own folders with autosave, checkpoints, duplicates and thumbnails; **Continue** resumes.
* **Interfere** in a civilization — inspire it, send a signal, share knowledge, teach a technology it is ready
  for, or test it with hardship — once every 25 years, never skipping what its own history must earn.

## The whole universe (new in 1.0)
* **Throw anything**: the Throw tool flings planets, comets, stars and black holes from the shelf (drag;
  click for a circular orbit) with a live trajectory; Grab moves them.
* **Collisions that shatter worlds**: disruption scaling after Leinhardt & Stewart (2012), debris, molten
  surfaces and boiling oceans, impact flashes and shock rings; moons inside the Roche limit become rings.
* **Stars live and die**: red giants swallow planets; planetary nebulae and white dwarfs; supernovae leave
  neutron stars and black holes, and their radiation crosses the galaxy at light speed.
* **Star formation** in nurseries like the real Orion Nebula; **black holes** with gravitational lensing;
  **pulsars**; **comets** with tails.
* **Galaxy scale**: the Milky Way from outside, the Local Group, and thousands of galaxies and quasars.
* **Civilizations beyond their world**: Dyson swarms, terraforming, generation ships founding new branches.
* **Guided tours**, a **timeline you can rewind**, **photo mode** (P) and a synthesised **soundtrack**. Optional [habitat equipment listening](docs/HABITAT_AUDIO.md) follows live completed power budgets with explicitly designed textures and WAV/source exports.

## Worlds and civilizations you can see
* Every world has its own look, chosen from physics or spacecraft imagery: Io's volcanoes, Europa's cracked ice,
  Jupiter's belts and Great Red Spot, Saturn's hexagon and measured rings, Titan's haze, Venus's clouds — and
  exoplanets from lava worlds and mini-Neptunes to Sudarsky-class giants and glowing hot Jupiters.
* Atmospheric scattering, eclipses (moons' shadows), ring shadows, living stellar surfaces.
* The Solar System Lab starts with **humanity in 2026**: real cities and countries, 11 000 satellites, the real
  exploration record and missions in flight (BepiColombo, Europa Clipper, JUICE).
* Civilizations show their era, Kardashev rating and next breakthroughs; their satellites, stations, spacecraft
  and colonies are visible in space, their cities and farmland on the ground.

## What's simulated
Stars that brighten and die · Keplerian orbits exact at any time scale · procedural systems with statistically
plausible stars, planets, moons, belts and rings · climate with greenhouse, carbon cycle and ice ages ·
habitability as a breakdown of factors · life from prebiotic chemistry to intelligence, with oxygenation, fossil-fuel
formation and mass extinctions · civilizations with population, 13 knowledge domains, pressures, crises and collapse ·
a 59-technology causal graph with alternative routes and resource/environment gates · settlements, roads, rail and
sea lanes · satellites, colonies, probes and radio signals that other civilizations can detect · deterministic,
versioned saves.

Ask any civilization *why*: every discovery records its route and drivers, and the inspector explains what is still
missing for the next ones ("Bronze working — needs tin ≥ 0.15× Earth").

## Controls
Drag to orbit · scroll (or W/S) to zoom from metres to light-years · click to select · double-click or **F** to fly
there · **Home** for the whole system · **Space** pause · **, .** speed · **⌘/Ctrl+K** command palette ·
**⌘/Ctrl+Z / Shift+Z** undo / redo · **⌘/Ctrl+S** save · **Tab** hide UI · **P** photo · **F3** developer overlay · **F1** help ·
**Esc** menu.

## Documentation
[Master development directive](docs/MASTER_DEVELOPMENT_DIRECTIVE.md) · [Current audit](docs/CURRENT_STATE_AUDIT.md) · [Active phase plan](docs/MASTER_GAP_ANALYSIS.md) · [Performance measurement](docs/PERFORMANCE.md) · [Camera & photography](docs/NAVIGATION.md) · [Sound](docs/AUDIO_SYSTEM.md) · [Notifications](docs/NOTIFICATION_SYSTEM.md)

[HANDOFF (start here to contribute)](HANDOFF.md) · [1.0 product plan](docs/PRODUCT_PLAN_1_0.md) · [Sandbox vision](docs/SANDBOX_VISION.md) · [Milestone S1 status](docs/MILESTONE_S1.md) · [G1 + C1](docs/MILESTONE_G1_C1.md) ·
[Physics engine](docs/PHYSICS_ENGINE.md) · [Data sources](docs/DATA_SOURCES.md) · [Object model](docs/OBJECT_MODEL.md) ·
[Save format](docs/SAVE_FORMAT.md) · [UI system](docs/UI_SYSTEM.md) · [Gap analysis](docs/GAP_ANALYSIS.md) ·
[ARCHITECTURE](ARCHITECTURE.md) · [ROADMAP](ROADMAP.md) · [SIMULATION](SIMULATION.md) ·
[CIVILIZATION_MODEL](CIVILIZATION_MODEL.md) · [TECHNOLOGY_MODEL](TECHNOLOGY_MODEL.md) · [RENDERING](RENDERING.md) ·
[BUILDING](BUILDING.md) · [CHANGELOG](CHANGELOG.md) · [prototype audit](docs/AUDIT.md) ·
[engine assessment](docs/ENGINE_ASSESSMENT.md) · [assets & licensing](ASSETS.md)

## License
MIT OR Apache-2.0.

Developer research controls, recorded data and reproducible studies: [RESEARCH](docs/RESEARCH.md).
Scientific/product acceptance direction: [RESEARCH_DESIGN](docs/RESEARCH_DESIGN.md).

Habitat [WHY explanations](docs/HABITAT_DIAGNOSTICS.md) trace exact live or recorded power, water, heat and air/crew terms with units, formulas, missing-evidence gaps and protected source exports. These are model-accounting explanations, with unquantified field accuracy.

Authored native camera paths connect timed views, lens changes and explicit playback/scrub; [controls and limits](docs/CINEMATIC_PATHS.md). Presentation remains independent of simulation physics.

Native camera and everyday keyboard controls can now be customized with conflict checks, explicit saving and current-key help; see [keyboard controls](docs/INPUT_BINDINGS.md).

The controlled lettuce planner now exposes optional source-based transpiration, water/vapor curves, humidity assumptions and numerical/source evidence alongside biomass, gas exchange and lighting. Physical greenhouse resource coupling and empirical crop calibration remain incomplete; [details](docs/TRANSPIRATION_PLANNING.md).

The experimental habitat also offers optional finite [cabin humidity and condensate](docs/HABITAT_HUMIDITY.md), with explicit phase heat, equipment assumptions, source-supported RH and actual recorded water/power budgets. It does not establish safe cabin conditions or water quality.

Controlled moisture [studies](docs/MOISTURE_STUDIES.md) compare six water-phase/equipment assumptions from one exact starting world using native or CLI experiments, preserving all inventories and source-pinned trajectories. These are conditional model studies; field calibration and physical greenhouse exchange remain incomplete.

Paired habitat [experiments](docs/PAIRED_HABITAT_EXPERIMENTS.md) compare all four combinations of two settings, show actual conditional/interaction results and reopen complete source-pinned designs.

[Paired trajectory analysis](docs/PAIRED_TRAJECTORIES.md) shows recorded conditional/interaction effects over time and exports every supported observation with source identity.

[Saved-design replay verification](docs/HABITAT_REPLAY.md) independently reconstructs every paired observation and reports exact differences while preserving the original data.

[Recorded graph navigation](docs/RECORDED_GRAPHS.md) adds time windows, actual-point cursors, explicit scales and reusable layouts while keeping complete numerical exports unchanged.

[Finite cabin breach](docs/CABIN_BREACH.md) models outward open-orifice gas loss with an outside mass ledger, explicit assumptions/thermal limits and actual saved studies.

[Recorded pressure explanations](docs/RECORDED_PRESSURE.md) expose actual gas components and signed inventory, temperature and volume accounting, with exact retained inputs and protected evidence exports.

Controlled [cabin geometry/thermal studies](docs/THERMAL_STUDIES.md) compare fixed-inventory volume, effective heat capacity, envelope conductance and radiator assumptions through the same protected recordings, four-corner contrasts and independent replay.

[Recorded research figures](docs/RECORDED_FIGURES.md) export vector charts together with exact original values, support/scale omissions, source observations, paired methods and reusable display layouts.

Recorded cabin balances preserve actual endpoint stocks, cumulative electricity and reference temperature changes in shared native/CLI views and protected evidence: [guide](docs/RECORDED_BALANCES.md).

New habitat definitions retain actual interval heat counters for recorded span explanations; legacy history remains unavailable. [Cabin heat accounting](docs/CABIN_HEAT_ACCOUNTING.md).

Recorded cumulative heat curves: [guide](docs/HEAT_TRAJECTORIES.md); actual signed counters with explicit origin and coverage, selectable native/CLI research figures. Existing saved aggregate analyses remain immutable.

Crop research adds [finite supply studies](docs/FINITE_CROP_SUPPLIES.md): explicit water/CO₂/nutrient-equivalent/lighting budgets, actual daily forecast support, source recipe residuals and protected reproducibility files. Maintained-regime feasibility does not grant habitat inventories or predict stressed crops.

[Conservative crop chemistry](docs/CROP_CHEMISTRY.md) exposes explicit CHON-only surrogate composition, published-versus-derived oxygen, elemental residuals and protected exact evidence. Physical greenhouse coupling remains incomplete.

Finite installed crop engineering, actual material/electric/heat evidence and explicit model limits: [FINITE_GREENHOUSE](docs/FINITE_GREENHOUSE.md).

Actual installed crop recordings support selected resource/production/energy graphs, exact-time comparisons and protected research figures; [recorded crop guide](docs/RECORDED_CROP_TRAJECTORIES.md). Supported source progress remains separate from biological age.
