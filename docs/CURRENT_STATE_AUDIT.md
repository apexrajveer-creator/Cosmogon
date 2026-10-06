# Current state audit — master directive baseline

Audited 2026-10-05 against `c28ab57` (the handoff immediately following release 1.0.0).
The controlling product specification is [MASTER_DEVELOPMENT_DIRECTIVE.md](MASTER_DEVELOPMENT_DIRECTIVE.md).
This audit supersedes older status claims in `AUDIT.md` and `GAP_ANALYSIS.md`; those remain historical records.

This table records the **entry baseline**, not the current branch. Subsequent navigation, photo
and performance work is tracked in [MASTER_GAP_ANALYSIS](MASTER_GAP_ANALYSIS.md) and
[VALIDATION](VALIDATION.md). The owner prioritizes integrated build batches followed by verification.

## Decision

Keep Rust, Bevy 0.18.1, bevy_egui 0.39 and the existing engine-independent simulation.
No engine or language migration is justified by this audit. The existing application is far beyond
an empty prototype, but it is not the full product in the master directive. A visual effect or enum
variant is not evidence that its scientific domain is complete.

Start with phase 1 performance instrumentation, then a camera/navigation slice from phase 3.
Much of phase 2 and phase 6 already exists; preserve it and close specific gaps rather than rebuild it.

## Baseline verification and checkpoint

* Clean `main` at entry. Local checkpoint: `checkpoint/master-directive-1.0` → `c28ab57`.
  Existing stable release tag `v1.0.0` retained. Work branch: `codex/native-performance-foundation`.
* `cargo test --release --workspace`: **173 passed**, **1 ignored** (the opt-in texture benchmark),
  zero failures, including the existing golden fingerprint and one physics doctest.
* Release build, clippy, instrumented captures and follow-up tests: recorded in [VALIDATION.md](VALIDATION.md).
* Native Saturn baseline capture inspected: rings, lighting, labels and orbit overlays render.
  `--hide-ui` hides panels but intentionally leaves labels/orbits (it is not the Photo action).
* GPU access requires launching outside the command sandbox on this host; a sandboxed launch
  reported “Unable to find a GPU”. The approved native launch succeeded.

## Architecture inventory

| Concern | Verified implementation | Limitation / implication |
|---|---|---|
| Languages / build | Rust 2021 Cargo workspace, five active crates; embedded WGSL and TOML | Python scripts are development-only dataset/packaging tools, not player runtime dependencies |
| Renderer | Bevy 0.18.1 / wgpu, native Metal/Vulkan/DX12; embedded shaders; HDR, AgX, bloom, MSAA | Engine capability does not imply every effect is integrated or physically validated |
| UI | egui via bevy_egui 0.39; Inter + Phosphor; home, search, inspectors, floating tool windows | No general docking/workspace persistence, keyboard rebinding or multi-monitor workflow |
| World ownership | `Universe` in `cosmogon_sim`; app `Sim` reads it and sends validated `Edit`s | Simulation advancement still runs on the main update schedule, nominal 9 ms budget |
| Scheduling | fixed-period tasks; DMath and keyed random streams | Wall-clock scheduling budget is not a scientific timestep or accuracy guarantee |
| Physics | analytic Kepler; opt-in per-system N-body; encounter substeps, optional 1PN; collisions | Reduced models and preset accuracy; not a general validated research instrument |
| Precision | f64 world/camera; per-frame camera-relative f32 transforms | Finite f64 resolution remains; arbitrary centimetre precision at cosmic coordinates is not guaranteed |
| Jobs / IO | asynchronous creation/loading, texture baking, terrain patches and predictions | Saving remains largely synchronous; no general data-pack/resource budget manager |
| Saves | versioned JSON `.cosmo`; sandbox manifests, autosaves, checkpoints, clone, bounded undo | Bookmark field exists but has no implemented camera bookmark workflow |
| Runtime / distribution | embedded assets; native app; macOS packaging and cross-platform CI release workflow | Unsigned; this audit does not certify a new Windows/Linux build or notarization |

## Current feature catalogue and real gaps

| Domain | Existing source / behavior | Not yet established |
|---|---|---|
| Home / library | `ui/home.rs`, `persistence.rs`; Continue, New, Real Universe, Scenarios, save browser | New directive's full workspace architecture and layout persistence |
| Sandbox control | `sandbox.rs`, `ui/tools.rs`, `interact.rs`; edits, create/throw/grab, undo/redo, predictions, unit inputs | Universal parameter registry, dependency graphs, comparison, batch edits, parameter sweeps |
| Camera | `camera.rs`; target orbit, tracking, scale-aware zoom, focus transitions, terrain-aware altitude, galactic pivot | Detached six-degree free flight, speed/inertia controls, bookmarks, cinematic paths, input configuration |
| Photo | `capture.rs`; clean screenshot action; reproducible developer capture | Persistent photo workspace; independent exposure/FOV/DOF/grading UI; high-res/tiled output and metadata |
| Audio | `audio.rs`; synthesized ambience, impacts, supernovae and milestone bell; master volume | Realistic/cinematic selector, category mixer, atmosphere/interior/environment models, configurable alert routing |
| Notifications | toast/history pipeline, Chronicle filters, automatic milestone slowdown | Persistent searchable notification centre, severity/category settings, Do Not Disturb |
| Solar System | `astro/horizons.rs`, `sol.rs`; Horizons initial vectors and physical parameter data, read-only reference | Large catalogue streaming, small-body ingest, uncertainty per imported quantity |
| Terrain | `render/terrain_lod.rs`; CPU-generated cube-sphere patches; Earth ETOPO5 relief | Real Mars/Moon terrain, mission imagery placement; photographic surface rendering |
| Climate / biosphere | `planet/`, `habitability.rs`, `life.rs`; reduced global climate, resources, stage hazards | Regional weather/ecosystems; quantitatively validated agricultural/climate forecasts |
| Civilization | `civ/`, `present_day.rs`; population, sites, polities, causal technologies, Earth initial estimates | Detailed infrastructure/economy; engineering-scale settlement balances |
| Space capability | `civ/space.rs`, `expansion.rs`; missions, colonies, generation ships, terraforming, Dyson state | User-designed staged spacecraft, detailed vehicle thermal/power/flight, physical cargo supply chain |
| Stellar / exotic | `stellar.rs`, `nursery.rs`, rendering shaders; evolution, compact objects, supernovae, nurseries | Full model registry/error bounds; every exotic process listed in the directive |
| Galaxies | `render/galaxy.rs`; galaxy, Local Group, deep-field visuals | Survey-backed authoritative galaxy population/dynamics; galaxy collision simulation |
| Research | CLI runs/checks/surveys, solver diagnostics, provenance, histories, charts | PhysicsModelRegistry, generalized data export/experiments, uncertainty/validation panel |
| Data truth | object provenance and field overrides; real vs sandbox distinction | All eight requested states, consistent star provenance and uncertainty display |

## Save inspection

Reviewed `save.rs` and `persistence.rs`, manifest fields and dataset-version pinning, migration tests,
JSON round trips and golden fingerprint. Save version is 2. New state fields have compatibility
defaults. The app keeps settings outside the universe and bounded undo snapshots in memory.
No user save was rewritten during this audit. Performance records belong in separate JSON files;
they must never enter the simulation serialization or change the golden fingerprint.

## Profiling evidence and limits

Baseline native Saturn launch (High settings, 2940×1660 physical capture, lab, speed 0,
240 capture frames plus shutdown delay) measured by `/usr/bin/time -l`:
14.51 s wall, 19.48 s user CPU, 3.28 s system CPU, maximum RSS 302,481,408 bytes,
peak footprint 985,236,728 bytes. These are whole-process startup/capture measurements,
not steady-state frame times or a controlled before/after comparison; a release compilation was
also running. RSS and footprint are different OS counters and must not be conflated with VRAM.

The baseline F3 overlay only exposes smoothed FPS/frame and latest simulation timings.
There is no retained distribution/export or explicit universe-load duration. This blocks credible
regression comparisons before camera and terrain work. The first slice remedies that; see
[PERFORMANCE.md](PERFORMANCE.md) for methodology and measured runs.

GPU command timing and dedicated VRAM usage are **not measured** on this host. Bevy 0.18.1 documents
CPU-only render diagnostics on Metal. Instruments cannot be used here: `xctrace` requires full
Xcode and this host has Command Line Tools selected. No inferred frame-time-as-GPU-time numbers
are substituted. Full GPU/VRAM profiling remains an explicit phase 0 follow-up on a supported setup.

## Shortcuts and document drift

* Existing roadmaps contain older unchecked entries for features subsequently implemented. Use the
  new [gap analysis](MASTER_GAP_ANALYSIS.md), not historical checkbox counts, to choose work.
* Initial inventories now live in `data/SOURCE_REGISTRY` and `assets/ASSET_REGISTRY`, with unknown metadata explicit. Root licence text files were not found despite the workspace licence declaration.
* `ASSETS.md` predates the bundled Inter/Phosphor fonts and synthesized audio. Asset and dataset
  registries need an evidence-backed rights/provenance reconciliation before adding new data packs.
* Planet close-up detail and many material features are generated presentation detail. Earth relief
  is observed; a convincing Mars render is not MOLA terrain.
* Audio is deliberately a soundtrack, not acoustic propagation. A single master volume is not the
  requested environment engine or notification customization.
* Star provenance can still say PROCEDURAL for catalogue objects (already noted in HANDOFF).
* Physics/biology/civilization contain reduced models and heuristics; sources exist in scattered
  comments/docs, not in a machine-readable model registry with all required validation metadata.
* The black-hole secondary lens image, milestone slowdown and synchronous saves remain known issues.

## Architecture for the next slice

Add an app-only `Performance` resource. Keep a bounded frame window, distinct wall-time labels,
explicit absent metrics, and opt-in Bevy process/render diagnostics. Provide a calm optional panel
through the existing View menu and command palette. Export local versioned JSON with scene,
settings, camera, seed, data versions, sample count, quantiles and measurement limitations.
Record async universe-load duration; preserve simulation/save schemas. Build and test native
captures before deciding on threading, cache sizes or shader optimization.
