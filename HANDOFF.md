# Handoff

For anyone — human or AI agent — picking up Cosmogon. Read this first, then
[ARCHITECTURE.md](ARCHITECTURE.md) and [BUILDING.md](BUILDING.md).

## Active development directive (2026-10-05)

Read [docs/MASTER_DEVELOPMENT_DIRECTIVE.md](docs/MASTER_DEVELOPMENT_DIRECTIVE.md) for the owner's
long-term product specification. It supersedes conflicting older high-level plans. Current evidence:
[CURRENT_STATE_AUDIT](docs/CURRENT_STATE_AUDIT.md); active sequence:
[MASTER_GAP_ANALYSIS](docs/MASTER_GAP_ANALYSIS.md). Preserve the released features and native architecture.

The active build adds app-only free flight, camera/photo settings, persistent viewpoints and
performance measurement, configurable sound and a notification inbox; see [AUDIO_SYSTEM](docs/AUDIO_SYSTEM.md),
[NOTIFICATION_SYSTEM](docs/NOTIFICATION_SYSTEM.md), [NAVIGATION](docs/NAVIGATION.md),
[PERFORMANCE](docs/PERFORMANCE.md) and [VALIDATION](docs/VALIDATION.md). View → Performance or
Cmd/Ctrl+K → Performance opens the panel. `--profile /tmp/run.json --quit` records a bounded sample;
`--panel performance` supports capture QA. View → Camera & Photo opens navigation, photo and
viewpoint controls. G detaches; F returns to the selected object. No universe schema or golden
fingerprint change. Current branch verification is in VALIDATION; the 173-test figure below is
the released baseline.

The research build adds [research controls and recordings](docs/RESEARCH.md): searchable typed parameters,
model/source context, explicit data states, live plots and protected CSV/reproducibility bundles.
View → Control Panel & Research opens the workspace; developer --panel control|models|sources|plots
supports captures; experiments opens frozen-state studies. cosmogon-cli models, experiment,
sweep and replay expose the same engine-independent registry
and observer/definition runner. The owner additionally requires autonomous feature improvement
through primary-source research and product reasoning across student-to-researcher use; see
[RESEARCH_DESIGN](docs/RESEARCH_DESIGN.md). The following slice reopens recorded bundles, compares completed trajectories and persists guidance/favorites/graph choices; `--research-recording` and CLI inspect use the same bounded reader. Keep building successive connected slices. This does not change Universe/save state or the golden fingerprint. Research verification:
[RESEARCH_VALIDATION](docs/RESEARCH_VALIDATION.md), 208 passed plus 1 existing ignored at checkpoint 6e6da11. The following measured-validation slice adds cross-object studies and actual analytic/JPL reports; see [SCIENTIFIC_VALIDATION](docs/SCIENTIFIC_VALIDATION.md), 212 passed plus 1 existing ignored. Recorded-data verification: [RECORDED_DATA_VALIDATION](docs/RECORDED_DATA_VALIDATION.md).

New Solar System worlds now use observed-derived Mars MOLA relief and coverage metadata; see [MARS_TERRAIN](docs/MARS_TERRAIN.md). This deliberate terrain state change re-pins the golden to `0xf2c4140a45ae509b`; old saves retain their stored terrain. The connected exploration slice adds four historical landing references, probe/landing markers, shared f64 body-fixed transforms, altitude travel, ground/horizon composition and snapshot/tracked geographic viewpoints. Native/headless exports retain source and registration limits; see [MARS_EXPLORATION_DESIGN](docs/MARS_EXPLORATION_DESIGN.md) and [MARS_EXPLORATION_VALIDATION](docs/MARS_EXPLORATION_VALIDATION.md). Current checks: 223 passed plus 1 existing ignored; the golden remains unchanged. The next connected build adds [numeric terrain sections](docs/MARS_TERRAIN_STUDIES.md) with shared source-supported samples, native graphs/probe actions and protected definition/CSV/JSON bundles. Current section verification: [MARS_TERRAIN_STUDIES_VALIDATION](docs/MARS_TERRAIN_STUDIES_VALIDATION.md), 227 passed plus 1 existing ignored. The connected [MOLA refinement](docs/MARS_REFINEMENT.md) build adds separately pinned 16ppd height/coverage, explicit undoable source selection, shared probes/travel/studies and source mismatch/cache invalidation. Existing worlds/defaults and the golden remain unchanged. Current verification: [MARS_REFINEMENT_VALIDATION](docs/MARS_REFINEMENT_VALIDATION.md), 232 passed plus 1 existing ignored. The next [study replay](docs/TERRAIN_STUDY_REPLAY.md) build reopens source-pinned native definitions without overwriting their radius/samples, shows an on-globe sampled trace/selected row, and retains independent mismatched-source graphs with disabled geography. Current checks: [TERRAIN_STUDY_REPLAY_VALIDATION](docs/TERRAIN_STUDY_REPLAY_VALIDATION.md), 233 passed plus 1 existing ignored. Optional streamed higher-resolution patches, mission imagery and current rover paths remain unimplemented.

The active [habitat engineering](docs/HABITAT_ENGINEERING.md) build begins phase 9: authoritative geographic experiments, finite O2/food/water/capture stores, primary-star PV/external supply, battery/power allocation, nominal NASA crew exchanges and lumped thermal budgets. Native View/command palette opens Habitat engineering; --panel habitat|habitat-controls|habitat-build|habitat-model supports capture, --habitat-definition builds an explicit configuration. CLI habitat-definition emits editable setups, habitat records/replays bounded actual trajectories. Optional state/ID counter stay absent in untouched worlds; the golden remains unchanged. New saved worlds pin/validate the model; skipped dynamic acceleration caches are restored on load. Current checks: 243 passed plus one existing ignored, 22 CLI/native QA cases, six inspected captures and a 48-second packaged run; evidence is in [HABITAT_VALIDATION](docs/HABITAT_VALIDATION.md). Phase 9 remains partial: greenhouse, leaks, humidity, radiation, maintenance, logistics, calibration and habitat meshes remain.

The connected [greenhouse planner](docs/GREENHOUSE_PLANNING.md) exposes NASA MEC low-light lettuce forecasts, protected numeric/source/context exports and native curves without crediting unconstrained crop yields to habitat stocks. Physical greenhouse installation and finite resource coupling remain incomplete; [research memo](docs/GREENHOUSE_RESEARCH.md). Current checks: 248 passed plus one existing ignored, 28 CLI/native cases and six inspected final layouts; see [GREENHOUSE_VALIDATION](docs/GREENHOUSE_VALIDATION.md).

The active [finite electrolysis](docs/HABITAT_ELECTROLYSIS.md) build connects bounded water/power consumption to stored O₂ and isolated H₂, shared heat accounting, optional source/version pins, native Oxygen controls and actual CSV/JSON trajectories. [Research memo](docs/ELECTROLYSIS_RESEARCH.md). Current checks: 253 passed plus one existing ignored, 29 CLI/native cases, eight inspected captures and a 48-second packaged run; see [HABITAT_ELECTROLYSIS_VALIDATION](docs/HABITAT_ELECTROLYSIS_VALIDATION.md). Phase 9 and physical greenhouse coupling remain incomplete.

The connected [CO₂ reduction](docs/HABITAT_SABATIER.md) build adds optional finite captured-gas/hydrogen consumption, methane/raw-liquid storage, downstream powered recycling and reference heat release. See [SABATIER_RESEARCH](docs/SABATIER_RESEARCH.md). Current checks: 258 passed plus one existing ignored, 40 CLI/native cases, eleven accepted inspected layouts and a 48-second packaged run; [HABITAT_SABATIER_VALIDATION](docs/HABITAT_SABATIER_VALIDATION.md). Phase 9 remains partial.

The active [habitat recording viewer](docs/HABITAT_RECORDINGS.md) reopens validated source-pinned trajectories/snapshots, shows actual native metric graphs and selected rows independently of the live world, and shares bounded inspection with CLI `habitat-inspect`. [Design memo](docs/HABITAT_RECORDINGS_DESIGN.md). Checks: 264 passed plus one existing ignored, 40 distinct CLI/native cases, ten accepted inspected layouts and a 48-second packaged run; [verification](docs/HABITAT_RECORDINGS_VALIDATION.md). Full directive remains incomplete.

The connected [controlled habitat experiments](docs/HABITAT_COMPARISONS.md) build adds typed single-setting sweeps, exact common-state branches, protected complete manifests and native/CLI time/interval-supported comparisons. [Design and methodological memo](docs/HABITAT_COMPARISON_DESIGN.md). Checks: 271 passed plus one existing ignored, fifty distinct CLI/native cases, eleven final inspected layouts and a 48-second packaged run; [verification](docs/HABITAT_COMPARISONS_VALIDATION.md). Hardware calibration, interactions/uncertainty and physical greenhouse exchange remain incomplete.

The connected [habitat equipment audio](docs/HABITAT_AUDIO.md) build adds explicit interior listening, five actual completed-power-driven textures, independent environment mixing and protected WAV/source exports. It is designed sonification rather than measured acoustics or a physical cabin. Checks: 277 passed plus one existing ignored, 32 distinct offline/native/regression cases, thirteen inspected accepted layouts and a 48-second packaged run; [verification](docs/HABITAT_AUDIO_VALIDATION.md). Simulation state and golden remain unchanged. The next connected build explains completed habitat results through shared WHY diagnostics; the full directive remains incomplete.

The connected [habitat WHY diagnostics](docs/HABITAT_DIAGNOSTICS.md) build explains completed power allocations, signed heat/water/air transfers and retained evidence through the same native/CLI/export implementation. Recorded intervals remain independent of the live world, and missing evidence remains unavailable. Checks: 283 passed plus one existing ignored, forty distinct offline/native/regression cases, sixteen accepted inspected layouts and a 48-second packaged run; [verification](docs/HABITAT_DIAGNOSTICS_VALIDATION.md). No simulation law, save schema or golden change. Full causal history, hardware calibration and the master directive remain incomplete. The next connected build adds authored camera paths.

The connected [authored camera paths](docs/CINEMATIC_PATHS.md) build adds f64 timed poses, orientation/look-at, interpolated lens values, actual/saved-view authoring, explicit playback/scrub, protected definitions and request-time intended/actual metadata. Checks: 294 passed plus one existing ignored, 33 distinct preflight/native/regression cases, seventeen inspected accepted layouts and a 48-second packaged playback run; [verification](docs/CINEMATIC_PATHS_VALIDATION.md). No simulation, save or golden change. Video, DOF, splines and simulation-rate synchronization remain incomplete. The next connected build customizes native keyboard controls.

The connected [native keyboard controls](docs/INPUT_BINDINGS.md) build maps 24 camera/everyday actions through shared physical-key input, conflict/cancel/release handling, explicit protected persistence/import/export and dynamic help. Checks: 302 passed plus one existing ignored, 28 distinct refusal/native/regression cases, thirteen final inspected layouts and a 48-second packaged run; [verification](docs/INPUT_BINDINGS_VALIDATION.md). No simulation, save or golden change. Mouse/controller/chord customization, international logical keys and full accessibility verification remain. Controlled greenhouse transpiration planning follows; the master directive remains incomplete.

The connected [controlled transpiration](docs/TRANSPIRATION_PLANNING.md) build adds optional source-pinned lettuce water/vapor forecasts, explicit humidity, conductance and pressure assumptions, deterministic quadrature, native total/daily plots and complete protected numeric exports. Legacy definitions retain identical biomass and unavailable water. Checks: 308 passed plus one existing ignored, 52 distinct offline/refusal/native cases, sixteen inspected accepted layouts plus two latest packaged repeats and a 48-second normal packaged run; [verification](docs/TRANSPIRATION_VALIDATION.md). No Universe/save or golden change. Physical greenhouse supply, humidity dynamics, facility energy and crop calibration remain incomplete. Finite cabin moisture and condensate handling follows; the master directive remains incomplete.

The connected [finite cabin moisture](docs/HABITAT_HUMIDITY.md) build adds optional vapor/isolated condensate, supported saturation/RH, explicit phase heat and allocated dehumidification power, later powered recycling, source pins and actual native/recorded diagnostics. [Research memo](docs/HUMIDITY_RESEARCH.md). Checks: 316 passed plus one existing ignored, 57 distinct offline/refusal/native cases, fourteen final inspected layouts and a 48-second packaged run; [verification](docs/HABITAT_HUMIDITY_VALIDATION.md). Untouched worlds/old saves and the golden remain unchanged. Physical greenhouse, leaks, ice/wall condensation and equipment calibration remain incomplete. Controlled moisture equipment studies follow; the master directive remains incomplete.

The connected [controlled moisture studies](docs/MOISTURE_STUDIES.md) build adds six typed single-setting experiments with retained vapor/liquid inventories, shared native/CLI preflight and exact recorded phase/water comparisons. Completed interval timing and disable semantics are explicit; WHY phase drivers appear before aggregates. Checks: 320 passed plus one existing ignored, 36 distinct offline/refusal/native cases, eight inspected packaged layouts and a 48-second normal run; [verification](docs/MOISTURE_STUDIES_VALIDATION.md). No simulation law, save schema or golden change. Paired-setting studies follow; calibration, physical greenhouse and the master directive remain incomplete.

The connected [paired habitat experiments](docs/PAIRED_HABITAT_EXPERIMENTS.md) build adds two settings at two levels, all four atomic common-state branches, actual final conditional/average/interaction contrasts, protected source-pinned analysis and shared native/CLI reopening. Snapshots retain explicit pre-edit flow warnings. Checks: 324 passed plus one existing ignored, 35 distinct offline/refusal/native cases, nine inspected packaged layouts and a 48-second normal run; [verification](docs/PAIRED_HABITAT_VALIDATION.md). Single-setting behavior, simulation laws, saves and the golden remain unchanged. Selectable-time paired trajectory analysis follows; uncertainty, calibration, physical greenhouse and the master directive remain incomplete.

The connected [paired trajectories](docs/PAIRED_TRAJECTORIES.md) build adds selectable actual stored-time tables, signed discrete effect curves, four retained recordings, explicit support gaps and protected complete JSON/40-column CSV analyses. Loaded design identity stays separate from unopened folder text. Checks: 328 passed plus one existing ignored, 46 distinct cases, thirteen inspected final packaged layouts and a 48-second normal run; [verification](docs/PAIRED_TRAJECTORIES_VALIDATION.md). Original final analyses, simulation laws, saves and the golden remain unchanged. Explicit independent replay verification follows; empirical calibration, physical greenhouse and the master directive remain incomplete.

The connected [recorded design replay](docs/HABITAT_REPLAY.md) build independently reconstructs four retained branches at every stored time, compares complete observations/endpoints/source versions, and exposes worker progress/cancellation with protected mismatch reports. Checks: 333 passed plus one existing ignored, 34 distinct cases, twelve inspected packaged layouts and a 48-second normal run; [verification](docs/HABITAT_REPLAY_VALIDATION.md). Original recordings, analyses, simulation laws, saves and the golden remain unchanged. Recorded graph navigation follows; empirical calibration, physical greenhouse and the master directive remain incomplete.

The connected [recorded graph navigation](docs/RECORDED_GRAPHS.md) build adds actual-time zoom/pan/cursors, truthful linear/positive-log/signed scales and protected reusable layouts for ordinary and paired habitat recordings. Checks: 342 passed plus one existing ignored, 28 distinct cases, fourteen inspected packaged layouts and a 48-second normal run; [verification](docs/RECORDED_GRAPH_VALIDATION.md). Original observations/analyses, simulation laws, saves and the golden remain unchanged. Finite cabin gas loss follows; accessibility, empirical calibration, physical greenhouse and the master directive remain incomplete.

The connected [finite cabin breach](docs/CABIN_BREACH.md) build adds optional source-pinned outward orifice flow, actual escaped O₂/N₂/CO₂/vapor, finite outside mass history, explicit split thermal/enthalpy identities and bounded supported integration. Native controls/WHY/graphs, exact saves and controlled studies share the same state. Checks: 349 passed plus one existing ignored, 49 distinct cases, thirteen inspected packaged layouts and a 48-second normal run; [verification](docs/CABIN_BREACH_VALIDATION.md). Absent legacy worlds/golden stay unchanged. Seal/rarefied/inward flow, EVA, field calibration and physical greenhouse remain incomplete. Recorded pressure explanations follow.

## Where things stand (2026-10-05)
* **Released: 1.0.0** ("The whole universe"), tag `v1.0.0`, published on
  [GitHub Releases](https://github.com/ashvyagni/SpaceRender/releases) with macOS, Windows and Linux builds.
  0.5.0 and 0.3.0 are published too. `main` = `universe-1.0` = the 1.0.0 commit.
* What 1.0 contains and what was deliberately left out: [docs/PRODUCT_PLAN_1_0.md](docs/PRODUCT_PLAN_1_0.md)
  (section "Status at 1.0.0"). Milestone table: [ROADMAP.md](ROADMAP.md). History: [CHANGELOG.md](CHANGELOG.md).
* All 173 tests pass across the workspace; CI is green on macOS, Linux and Windows.

## The shape of the project
* `crates/cosmogon_sim` — the simulation. **No engine dependencies, deterministic.** It owns all state
  (`Universe`), advanced by a fixed-period task scheduler. The app may read it and issue `Edit`s; it never
  mutates the model behind the simulation's back.
* `crates/cosmogon` — the Bevy 0.18 app (bevy_egui 0.39, WGSL shaders, Metal/Vulkan/DX12): camera, rendering,
  UI, persistence, audio. Shaders are embedded; `render/shaders/common.wgsl` is imported as `cosmogon::common`.
* `crates/cosmogon_physics`, `cosmogon_core` — orbits, integrators, f64 math, constants.
  `crates/cosmogon_cli` — headless runs and determinism checks.
* Data is embedded TOML in `crates/cosmogon_sim/data/` (technologies, science parameters, the Solar System,
  2025 cities, JPL Horizons state vectors).

Map of the 1.0 features (start here when changing one):

| Feature | Simulation | App |
|---|---|---|
| Stellar evolution, supernovae, blast fronts | `stellar.rs`, `astro/star.rs` | `render/stellar.rs`, `shaders/{star,black_hole,nebula,beam}.wgsl` |
| Star formation (nurseries, Orion Nebula) | `nursery.rs` | `render/nurseries.rs`, `shaders/emission_nebula.wgsl` |
| Collisions, disruption, Roche rings | `collision.rs`, `sandbox.rs` (`on_contact`) | `render/impacts.rs` (flashes) |
| Comets | (bodies with `ObjectClass::Comet`) | `render/comets.rs`, `shaders/comet.wgsl` |
| Galaxy, Local Group, deep field | — | `render/galaxy.rs`, `shaders/{galaxy,deep_field}.wgsl`; galactic pivot in `camera.rs` |
| Dyson swarms, terraforming, generation ships | `expansion.rs` | `ui/markers.rs`, `ui/inspect.rs` (space programme) |
| Civilizations, technology | `civ/`, `data/technologies.toml`, `present_day.rs` | `ui/inspect.rs` |
| Experiments ("what ifs") | `sandbox.rs` (`WHAT_IFS`, `apply_what_if`) | `ui/home.rs` (Scenarios) |
| Throw / Grab tools | — | `interact.rs`, `sim.rs` (`Tool`), `ui/dock.rs` |
| Guided tours | — | `ui/tour.rs` |
| Timeline and rewind | — | `timeline.rs` |
| Historical Mars landing references / geographic travel | `planet/landing.rs`, `planet/mola.rs` | `surface.rs`, `camera.rs`, `ui/inspect.rs`, `ui/markers.rs` |
| Authored camera paths | — | `cinematic.rs`, `ui/cinematic.rs`; photo/performance metadata |
| Native keyboard controls | — | `input.rs`, `ui/input.rs`; camera/HUD/help and photo/performance metadata |
| Camera, free flight, viewpoints | — | `camera.rs`, `navigation.rs`, `ui/navigation.rs` |
| Photo mode, lens/colour, dev capture | — | `capture.rs`, `navigation.rs`, `ui/navigation.rs` |
| Research controls/models/sources/CSV | `research/` | `research.rs`, `ui/research.rs`; CLI `experiment`, `models` |
| Performance recordings | — | `performance.rs`, `ui/performance.rs` |
| Finite cabin moisture / condensate | `moisture.rs`, `habitat.rs` | `ui/habitat_humidity.rs`; actual recordings/WHY and CLI habitat exports |
| Paired habitat experiments / contrasts | `habitat_sweep.rs`, `habitat_factorial.rs` | `ui/habitat_recordings.rs`; CLI sweep/inspect, native Paired results |
| Saved paired-design replay verification | `habitat_replay.rs`, `habitat_factorial.rs` | `ui/habitat_replay.rs`; CLI `habitat-verify` |
| Recorded habitat graph navigation | — | `ui/habitat_graph.rs`, `ui/habitat_recordings.rs`; protected layouts and native `--habitat-graph` |
| Finite outward cabin breach | `breach.rs`, `habitat.rs` | `ui/habitat_breach.rs`; native/CLI studies, WHY and recordings |
| Habitat WHY / diagnostics exports | `habitat_diagnostics.rs` | `ui/habitat_why.rs`, `ui/habitat_recordings.rs`; CLI `habitat-diagnose` |
| Habitat equipment listening / WAV | completed `habitat.rs` budgets | `habitat_audio.rs`, `audio.rs`, `ui/alerts.rs` |
| Sound modes/mixer | — | `audio.rs`, `ui/alerts.rs` (synthesised; no audio files) |
| Notification inbox/routing | — | `notifications.rs`, `ui/alerts.rs`, `sim.rs` |

## How to verify a change
1. `cargo test --release --workspace` — must stay green. Run it with output to a file if it seems to hang
   (a test that advances the N-body Solar System lab by millions of years *will* take forever; test the
   function directly instead, see `nursery.rs` tests).
2. `cargo clippy --release --workspace` — no new warnings in the code you touched.
3. **Look at it.** The app renders straight to PNG without any screen-recording permission:
   ```bash
   cargo build --release -p cosmogon
   ./target/release/cosmogon --new lab --focus Saturn --distance 5 --speed 0 --hide-ui \
       --capture /tmp/shot.png --capture-after 240 --quit
   ```
   * `--new lab|sol|garden|system|empty` (anything else = neighbourhood), `--what-if ID` (any `WHAT_IFS` id),
     `--tour ID`, `--focus NAME` (body or star name), `--distance` **in radii of the focused object**,
     `--yaw/--pitch` (rad; pass `--yaw` too or the default lighting angle overrides `--pitch`),
     `--speed INDEX` into `cosmogon_sim::time::SPEEDS` (0 = real time … 11 = 1 Myr/s).
   * Galaxy-scale views: `--focus Sun --distance 2e12` ≈ 150 000 ly.
   * Without `--hide-ui` you see the full interface; with it, panels are hidden (labels and orbits stay).
4. For anything that runs for a while, also launch it normally for 30–60 s and watch the log for panics.

## Rules and conventions
* **Determinism.** The simulation must give bit-identical results on every platform. Use the `DMath`
  functions (`dsin`, `dexp`, `dpowf`, …) from `cosmogon_core::dmath` in simulation code, keyed RNG streams
  (`Rng::stream(seed, domain, keys)`), and never iterate a `HashMap` where order affects results.
* **Golden fingerprint** (`save.rs`, `golden_universe_fingerprint`): any change to the serialised state —
  even a new field — changes it. If the change is deliberate, re-pin the constant and say so in the
  changelog. Old saves must still load: new fields get `#[serde(default)]`.
* **Present-day Earth** gets every technology except those in `present_day::AHEAD` *and anything that
  depends on them*. New future technologies therefore stay in the future automatically.
* **Real vs procedural.** Real-data scenarios contain only catalogued objects (the Solar System, the
  Trapezium stars); every quantity carries a provenance label. Don't invent bodies around real stars.
* **Style.** Match the surrounding code: comments explain *why* (often with the physics and a reference),
  names are plain words, units in names or comments (SI internally). No new dependencies without need.
* **Git.** Commits by `ashvyagni`, **no `Co-Authored-By` trailers**. Push only to
  `git@github.com:ashvyagni/SpaceRender.git` as ashvyagni (check with `ssh -T git@github.com`); if you can't,
  hand back to the owner to push. Branch per milestone (1.0 was `universe-1.0`), fast-forward into `main`.

## Traps we already fell into
* **f32 overflow at galaxy scale.** Render-space vectors reach ~1e21 m and `length²` overflows f32
  (max 3.4e38) → `normalize` returns NaN/zero silently. Divide by a scale (ly, the object's radius) *before*
  normalising in shaders, and normalise in f64 on the CPU (`camera.rs`).
* **Metal: `smoothstep(a, b, x)` with `a ≥ b` is undefined.** Write `1.0 - smoothstep(b, a, x)`.
* **Expensive shaders can time out the GPU** (the first nebula shader did). Keep ray-march steps ~40 and
  noise octaves ≤ 4.
* **Fonts:** Inter's private-use code points shadowed Phosphor icons; they were stripped from the bundled
  Inter (see `crates/cosmogon/assets/fonts/README.md`). Don't swap in a stock Inter file.
* **Scheduler placeholders** use `scheduler::NEVER` (1e290), not `f64::INFINITY` — infinity serialises as
  JSON `null` and breaks loading.
* **Bevy query conflicts:** two queries touching the same component mutably in one system need
  `Without<…>` filters or separate systems.
* **UI layering:** bottom panels stack upwards in system order; floating areas (tool dock, tour card) are
  positioned above the timeline strip — adjust both if the bottom bar grows.
* `--capture` disables keyboard input and sound; photo mode is the in-app path.

## Releasing
1. Bump `version` in the root `Cargo.toml`; update `CHANGELOG.md`, `RELEASE_NOTES.md` (it becomes the
   release text), README images if relevant.
2. Commit, tag `vX.Y.Z`, push the tag. `.github/workflows/release.yml` builds macOS (universal DMG),
   Windows (installer, portable exe and zip) and Linux (tarball) and creates a **draft** release.
3. The owner publishes the draft on GitHub (needs a signed-in browser; the `gh` CLI isn't installed here).
   Builds are unsigned — first-launch instructions are in the release notes.

## What's next (candidates for 1.x)
From the plan's "Not yet" list, roughly by value:
* Real nearby stars (Gaia) and known exoplanets; more real nebulae.
* The Chronicle as a shareable book; challenges ("save Earth from the asteroid"); observatory mode.
* GPU N-body for thousands of particles; galaxy collisions; kilonovae, tidal disruption events, auroras,
  atmospheric entry fireballs.
* Video capture and a cinematic camera; signed/notarised installers; update check; localisation;
  accessibility options; a mod folder for data files.
* Known rough edges: auto-slow caps the clock at milestones, so star formation (tens of kyr per star) needs
  auto-slow off and 1 Myr/s to watch; the black hole's lensed secondary image draws as a large loop below
  the disk; catalogue stars show PROCEDURAL provenance tags in the inspector.
