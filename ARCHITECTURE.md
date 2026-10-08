# Architecture

![Overview](docs/images/sol_modern.jpg)

## Principle
**The simulation is authoritative and engine-independent. The application is a view.**

```
┌──────────────────────────── crates/cosmogon (Bevy app) ────────────────────────────┐
│  sim.rs        sessions (sandbox / read-only reference), create/load on task pool, │
│                advance under CPU budget, edits with undo/redo, predictions, saves  │
│  persistence   sandbox folders: manifest, state, autosave, checkpoints, thumbnail  │
│  camera.rs     f64 orbit/zoom/track rig → defines the floating origin each frame    │
│  render/       f64 WorldPos → camera-relative Transforms; planet & atmosphere WGSL; │
│                textures baked from the sim's own terrain; lights from settlements   │
│  ui/           egui: home, HUD, tools (create, launch, physics, palette), editable  │
│                inspector with units & provenance, civilization dashboard, markers   │
│  capture.rs    GPU screenshots for verification                                     │
└───────────────────────────────▲──────────────────────────────────────────────────────┘
                                │  reads state, issues commands (never edits the model)
┌───────────────────────────────┴──────── crates/cosmogon_sim (no engine deps) ───────┐
│  universe.rs   owns all state; advance_to(t, budget, stop_on_milestone)              │
│  scheduler.rs  fixed-period tasks: biospheres every 10 kyr, civilizations every year │
│  astro/        stars, Kepler orbits, dynamics (N-body state per system), objects &  │
│                provenance, JPL Horizons dataset, procedural systems, real Sol        │
│  sandbox.rs    Edit commands → validate, apply, journal, propagate consequences     │
│  impact.rs     impact energy, crater, blast, winter, extinction scaling             │
│  planet/       climate, terrain (shared with renderer), resources & deposits         │
│  habitability, life, civ/ (species, knowledge, tech graph, settlements), history     │
│  save.rs       versioned JSON with migrations          rng.rs  keyed RNG streams     │
│  data/         science.toml · technologies.toml · sol.toml   (embedded, data-driven) │
└───────────────────────────────▲──────────────────────────────────────────────────────┘
                 ┌──────────────┴──────────────┐
          cosmogon_physics               cosmogon_core           cosmogon_cli (headless)
          Kepler, elements,              f64 math, units,        run · check · survey
          integrators, gravity           constants
```

## Crates
| Crate | Responsibility | Depends on |
|---|---|---|
| `cosmogon_core` | f64 vectors, constants, units | — |
| `cosmogon_physics` | orbital mechanics, N-body integrator, collisions | core |
| `cosmogon_sim` | the universe model (everything that isn't pixels) | core, physics, serde, toml |
| `cosmogon_cli` | headless runs, determinism checks, surveys | sim |
| `cosmogon` | the desktop app | sim, Bevy 0.18, bevy_egui |

The brief's suggested modules map onto `cosmogon_sim` as: Astronomy → `astro`, Planet/Climate → `planet`,
Life → `life`, Civilization/Technology/Economy → `civ`, History → `history`, Simulation (time,
scheduler) → `time`, `scheduler`, `universe`, Persistence → `save`, Core → `rng`, `noise`, `params`.
Rendering/UI/Platform live in the app crate.

## Frame
```
Update:  Frame::Simulate  → advance universe (≤ 9 ms CPU budget)
         Frame::Positions → f64 world positions & rotations from the universe
         Frame::Camera    → camera rig; sets ViewInfo.origin = camera f64 position
         Frame::Apply     → Transform = world − origin; materials; texture LOD; lights; gizmos
EguiPrimaryContextPass:     markers, panels, toasts
```

## Sandbox edits and the consequence pipeline
The UI never mutates the model: it sends `cosmogon_sim::sandbox::Edit` commands through
`Sim::edit` (which keeps an undo snapshot). `Universe::apply_edit` validates, activates N-body
for the system, syncs the integrator exactly to the current time, applies, re-grids the step,
journals the edit and propagates it. Physics emits collisions (`ContactEvent`) and the
environment task notices insolation changes; `sandbox.rs` carries both down the chain:
orbit → insolation → climate → habitability → biosphere → civilization (see docs/SANDBOX_VISION.md).

## Ownership and concurrency
All simulation state lives in one `Universe` value owned by the `Sim` resource. Universe creation
and loading run on Bevy's async task pool; prehistory is parallelised internally with scoped threads
(biospheres are independent and draw from their own RNG streams, so results are deterministic).
Texture baking runs on the task pool from cloned body data. Moving `advance` itself to a worker
thread is planned (Phase 1 follow-up); the API was shaped for it (`advance_to` + budget).

## Determinism
See [SIMULATION.md](SIMULATION.md#determinism). Enforced by tests and by `cosmogon-cli check` in CI.

## Data-driven content
`science.toml` (rates and assumptions), `technologies.toml` (the technology graph) and `sol.toml`
(the real Solar System) are embedded in the binary — the app needs no files beside itself — and
validated at load (unknown references and dependency cycles are rejected by tests).

## App performance telemetry

`performance.rs` reads `Sim`, camera/view settings and Bevy diagnostics in the app's `Last` schedule.
It owns a bounded frame window (600 normally, at most 3600 for automated recordings), updates panel
summaries four times a second, and exports separate versioned JSON. Detailed render/process
measurement is opt-in at app startup. It never writes `Universe`, `Edit`, saves or the golden state.
The load task carries an app-only start instant; successful completion records elapsed time.
See [PERFORMANCE.md](docs/PERFORMANCE.md) for units, sampling limits and unavailable metrics.

## Navigation and presentation

`navigation.rs` owns app-only camera preferences, presentation values, free-flight math and the
versioned viewpoint library. `CameraRig` selects orbit or an f64 free pose. The camera is the floating
origin in either mode; free flight chooses the nearest solid body for terrain LOD independently of
selection. Controls integrate using real frame time; optional inertia uses the analytic exponential
response. Orbit return blends position and orientation from the detached view.

Presentation values apply to perspective projection and Bevy colour grading. They never issue an
`Edit` or change stellar output. Viewpoints store object-relative offsets plus stable object IDs and
universe identity; restoration follows current object position without rewinding simulation time.
`capture.rs` saves actual GPU readback images and optional versioned request-time metadata. Neither
photos, viewpoints nor performance telemetry enters the deterministic Universe serialization.

## Event presentation and audio

`notifications.rs` reads authoritative Chronicle events and maintains a bounded 500-entry derived
inbox. `sim::collect_toasts` routes new events after simulation advancement into inbox, bounded
popups, optional app pause and coalesced `SoundCue` messages. Universe replacement resets the cursor
and imports existing events silently. Quiet/photo policy suppresses interruptions while retaining
routed history. Routing and severity labels do not create a new simulation hazard model.

`audio.rs` tags sources by Music/Cinematic/Alert and applies master × channel × mode gain both at
spawn and to active sinks. Realistic mode disables external cinematic effects; Hybrid softens them.
Leaving Observing removes all session audio. Mode/mix/category preferences serialize only in app
settings. New simulated environment audio will require explicit context/model producers.

## Shared research adapters

cosmogon_sim::research owns static parameter/model/source descriptors, read-only readings,
validated adapters to existing Edit commands and bounded observer recordings. It has no Bevy
dependency and is shared with the CLI. Native research state/UI owns selection snapshots, real-time
recording cadence and export feedback; it edits only through Sim::edit. Registries/recordings are
not serialized into Universe. See [RESEARCH](docs/RESEARCH.md) for limits and the bundle contract.

The application-only `habitat_audio.rs` reads completed habitat interval configurations and power budgets into an explicit transient interior listener. A native streaming decoder uses bounded atomic presentation levels; ordinary mixer/capture/photo controls remain separate from simulation state. The same decoder exports protected fixed-interval WAV/source artifacts on a worker. No engine audio dependency or golden state change. See [HABITAT_AUDIO](docs/HABITAT_AUDIO.md).

### Habitat completed-interval explanations

`cosmogon_sim::habitat_diagnostics` derives typed unit/formula terms and priority allocations from retained completed budgets. Ordinary reports, bounded recording validation, CLI diagnostic exports and native live/historical WHY share the same result; app sonification reuses allocations. No start inventories or physical state are invented. Native protected source exports use a worker with a captured habitat snapshot.

Authored cinematic paths live in app-only `cinematic.rs`: bounded definitions, shared f64 pose evaluator and playback resource. Camera input → path evaluation → rendering/clearance → actual-pose recording is explicitly ordered. `ui/cinematic.rs` authors actual views and saved viewpoints; photo/performance reports expose intended/actual poses. `Sim.camera_source_fingerprint` retains app-only initial imported-state identity; no Universe/save fields, simulation laws or golden changes.

Native `input.rs` owns the validated physical-key map, capture/release gate and separately versioned explicit preferences. Camera, cinematic interruption and HUD read the same mapping; `ui/input.rs`, navigation/help labels and photo metadata expose it. No simulation or UserSettings schema mutation.

Crop planning version 2 retains the analytic biomass model and adds an optional separately pinned light-period transpiration component in `cosmogon_sim::greenhouse`. Humidity and fixed terrestrial pressure are assumptions; deterministic per-day Simpson integration includes a resolution diagnostic. Native Forecast/Inputs/Compare/Water model/Files and CLI share this engine. Legacy definitions remain biomass-only, and world context never transfers inventories. See [TRANSPIRATION_PLANNING](docs/TRANSPIRATION_PLANNING.md).

Optional cabin moisture is owned by `cosmogon_sim::moisture` and the habitat fixed-minute budget: finite vapor/condensate, explicit crew phase storage, start-state dehumidification and later recycler priority. RH uses a bounded NIST liquid-water fit; unsupported domains retain inventories. `uses_moisture` keeps source/model pins for disabled retained history. Native Humidity observes/stages ordinary edits, with exact photo/performance observations. CSV and independent recording graphs remain additive; legacy diagnostics remain identical when the mechanism is absent. See [HABITAT_HUMIDITY](docs/HABITAT_HUMIDITY.md).

Moisture sensitivity studies extend `habitat_sweep::Parameter` rather than introducing a second simulation. Native experiment preflight and the worker use `Plan::validate_world`/ordinary edits; common vapor/liquid stocks and source/version identities remain fixed. Existing exact-time/interval comparison metrics read actual exported observations.

Paired designs extend the existing ordinary-edit sweep plan with an optional second setting and four atomic configurations. Shared `habitat_factorial` computes conditional/interaction contrasts from actual supported final rows and validates root common-state identity, all branch recordings and recomputed analysis. Native workers/reopening and CLI inspection reuse that module; no presentation state enters the simulation.

Shared `habitat_comparison::pair_at` supplies exact point support to selected factorial reports and bounded typed curves. Native workers retain all four validated recordings plus ordinary first/last compatibility views, cache one metric curve and export from a separately retained loaded source path. Immutable final completion analysis remains distinct from selected-time metadata.

Engine-independent `habitat_replay` validates complete retained paired inputs, reconstructs ordinary edits from the common base and cooperatively advances the fixed scheduler at exact plan times. Complete canonical Sample bytes and initial/final world/source identities determine verification. Native replay uses a separate cancellable worker, immutable request/report sources and captured report export; the live world and recorded data remain independent.

App-only `ui::habitat_graph` validates reusable layouts and builds bounded actual-coordinate plots for ordinary and paired recordings. Missing support and scale-domain omissions remain separate; original selected/final analyses and full numerical exports are independent. Explicit app preferences and protected layout files never enter Universe or sample state.
