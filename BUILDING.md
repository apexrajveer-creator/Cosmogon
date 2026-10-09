# Building

## Requirements (developers only — players need nothing)
* Rust 1.85+ (`rustup`), a GPU with Metal / Vulkan / DX12.
* macOS: Xcode Command Line Tools (for `codesign`, `iconutil`, `hdiutil`).
* Linux: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.

## Common commands
```bash
cargo run --release -p cosmogon                      # the app
cargo run -p cosmogon --features dev                 # faster incremental builds (dynamic linking; not distributable)
cargo test -p cosmogon_core -p cosmogon_physics -p cosmogon_sim
cargo run --release -p cosmogon_cli -- run --scenario sol --seed 4 --years 202000
cargo run --release -p cosmogon_cli -- check --scenario garden --seed 9 --years 3000000   # determinism
cargo run --release -p cosmogon_cli -- survey --scenario garden --life hopeful --seeds 8 --years 2000000000
```

## Developer flags for the app
`--new sol|garden|neighbourhood --seed N --life 0|1|2 --advance-years Y --focus NAME --distance RADII
--yaw RAD --pitch RAD --speed INDEX --select-civ --hide-ui --load PATH --capture FILE.png --capture-after FRAMES --quit`

## Packaging
* **macOS:** `scripts/package-macos.sh` → `dist/Cosmogon.app` + `dist/Cosmogon-<v>-macOS.dmg`
  (ad-hoc signed). `UNIVERSAL=1` builds arm64 + x86_64. For public distribution set
  `SIGN_ID="Developer ID Application: …"`, then notarise:
  `xcrun notarytool submit dist/*.dmg --apple-id … --team-id … --wait && xcrun stapler staple dist/*.dmg`.
* **Windows:** built in CI (`release.yml`): portable zip and an Inno Setup installer
  (`packaging/windows/cosmogon.iss`); the icon is embedded by `crates/cosmogon/build.rs`. Release builds have
  no console window.
* **Linux:** tarball in CI; AppImage/Flatpak planned.
* Icon: regenerate with `python3 scripts/make_icon.py` (original artwork, stdlib only).

Everything the app needs (shaders, data files, fonts) is embedded in the binary.

## Where things are stored
Saves: `~/Library/Application Support/Cosmogon/saves` (macOS), `%APPDATA%\Cosmogon\saves` (Windows),
`~/.local/share/Cosmogon/saves` (Linux). Settings: the platform config directory, `Cosmogon/settings.toml`.

## Performance reports

Open View → Performance, or use Cmd/Ctrl+K and search Performance. Detailed counters are enabled
in Settings for the next launch. For an automatic run:

```sh
./target/release/cosmogon --new lab --focus Saturn --distance 5 --speed 0 \
  --profile /tmp/saturn-run1.json --profile-warmup 120 --profile-frames 600 --quit
```

`--panel performance` opens the panel for captures. Choose a new report path each time; export
refuses to overwrite existing files. See [PERFORMANCE.md](docs/PERFORMANCE.md) for measurement
meaning, sample bounds, screenshot coordination and GPU/VRAM limitations.

## Camera and photo verification

View → Camera & Photo opens the native camera, photo, viewpoint and path tabs. See [NAVIGATION](docs/NAVIGATION.md) for controls,
persistence and precision limits. Developer examples:

```sh
./target/release/cosmogon --new lab --focus Saturn --distance 5 --free-camera --panel camera \
  --capture /tmp/free-camera.png --capture-metadata --capture-after 240 --quit
./target/release/cosmogon --new lab --focus Saturn --distance 5 --panel photo --fov 35 --exposure 1.5 \
  --capture /tmp/photo-controls.png --capture-metadata --capture-after 240 --quit
```

`--free-camera` detaches after the initial view is established. `--fov` is vertical degrees (10–110);
`--exposure` is presentation EV (−6…6). `--panel camera|photo|bookmarks` selects a tab.
`--capture-metadata` writes `<image stem>.png.json` beside the image; a same-stem performance `.json`
remains separate. Automated `--capture` ignores keyboard input; exercise movement in an ordinary run.

## Sound and notification verification

Saved habitat previews use `--load SAVE --habitat-audio ID --panel sound`; capture metadata retains the source interval while playback stays disabled. `--load SAVE --habitat-audio ID --habitat-audio-export NEW_DIR` exports a WAV and exact source report without starting the GPU. See [HABITAT_AUDIO](docs/HABITAT_AUDIO.md) for context and presentation limits.

`--panel alerts|sound|routing` opens the respective Sound & Alerts tab for native captures.
`--sound-mode realistic|cinematic|hybrid` selects a launch mode; `--quiet` mutes master sound and
suppresses interruptions for the launch. See [AUDIO_SYSTEM](docs/AUDIO_SYSTEM.md) and
[NOTIFICATION_SYSTEM](docs/NOTIFICATION_SYSTEM.md) for live mixing, routing and current limits.

The macOS packager assembles/signs the app in a local temporary folder, then creates the disk image
and keeps a launchable copy in `~/Library/Caches/Cosmogon/builds/<version>/Cosmogon.app`.
`dist/Cosmogon.app` links to that copy; `COSMOGON_APP_OUTPUT_DIR` can override its directory.
The disk image contains a standalone app. This avoids Finder metadata being re-added by
file-provider folders during or after signing. Apple documents the forbidden metadata in
[QA1940](https://developer.apple.com/library/archive/qa/qa1940/_index.html). Temporary staging is
removed on success or failure. Local builds are ad-hoc signed unless a Developer ID is supplied.

## Research recordings

See [RESEARCH](docs/RESEARCH.md) for native controls, provenance, scientific limits and bundle layout.

```sh
./target/release/cosmogon-cli experiment --scenario lab --target Earth --samples 32 \
  --interval-days 1 --output /tmp/earth-research-1
./target/release/cosmogon --new lab --focus Earth --speed 0 --quiet --panel plots \
  --research-export /tmp/native-research-1 --research-samples 60 --research-every 20 \
  --capture /tmp/research.png --capture-after 1300 --quit
```

Use new output directories. Native cadence counts app frames; CLI cadence requests simulation days.
`--panel control|models|sources|plots` opens a research tab. Models prints registry JSON without
creating a universe. No additional installation or network service is required.

Measured orbit checks: `./target/release/cosmogon-cli validate --output /tmp/orbit-check-1.json`.
`--panel validation` opens Models and starts these fixed fixtures for native captures. CLI `--help` documents cross-object sweeps and primary measurements; existing export paths are protected.

Reopen saved numeric data: `./target/release/cosmogon-cli inspect --recording /tmp/earth-research-1`.
Native QA: `--research-recording /tmp/earth-research-1 --panel plots --research-workspace /tmp/research-choices.json`; the optional choices file is distinct from the simulation save.

Observed Mars relief: `python3 tools/process_mola.py --fetch` verifies pinned PDS products and
converts their endianness. The application needs no runtime network. `cosmogon-cli datasets` shows
MOLA source/checksum metadata; `surface --target Mars --latitude 18.625 --longitude 226.125` samples
the shared field. Native `--new lab --focus Mars --panel surface` opens the source-cell probe.
See [MARS_TERRAIN](docs/MARS_TERRAIN.md).

### Historical landing references and surface travel

```bash
./target/release/cosmogon-cli sites
./target/release/cosmogon-cli surface --site perseverance
./target/release/cosmogon --surface-site curiosity --surface-altitude 100000 --speed 0 \
    --panel surface --capture /tmp/curiosity.png --capture-after 300 --capture-metadata --quit
```

`--surface-site` selects a historical reference and new Solar System Lab unless a world is supplied.
Altitude uses radial metres above the pinned MOLA field (60–100000000); `--surface-snapshot` leaves
camera pose fixed after placement rather than following body translation/spin. `--surface-horizon`
uses a horizon composition instead of the footprint overview. The automation renderer
runs on the main thread for `--surface-site --quit`, as for imported-recording captures, to avoid early
GPU teardown waits. Ordinary interactive starts retain pipelined rendering. Historical coordinates,
coarse terrain and unverified registration do not provide current rover position or landing safety.

### Numeric Mars terrain sections

```bash
./target/release/cosmogon-cli section --from-site curiosity --to-site insight --samples 64 \
    --output /tmp/new-mars-section
./target/release/cosmogon-cli section --definition /tmp/new-mars-section/definition.json
./target/release/cosmogon --new lab --focus Mars --panel section --quiet \
    --capture /tmp/mars-section.png --capture-after 300 --quit
```

`section` requires the pinned observed Mars field, rejects ambiguous endpoints and protects existing
output folders. An exported definition retains its chosen reference sphere/sample count. Native
`--panel section` shows the actual graph, row values and probe/travel/export controls; see
[MARS_TERRAIN_STUDIES](docs/MARS_TERRAIN_STUDIES.md) for schema and limitations.

## Finer Mars source selection

See [MARS_REFINEMENT](docs/MARS_REFINEMENT.md). Both verified grids are embedded; no runtime data download is needed. `--mars-terrain 4|16` applies an explicit source edit for native development starts and CLI surface/section commands. CLI `terrain --mars-terrain 16 --save NEW_FILE [--initial OLD_SAVE]` creates a protected saved world. `cargo run --release -p cosmogon_cli --example mola_profile` measures warm source queries only; it is not a graphics benchmark.

## Native terrain study replay

`--terrain-section /path/to/definition.json` reopens a bounded terrain section without changing the world. It preserves the exact dataset, sphere radius and samples. Different-source study graphs/export remain available; geographic actions require a matching world. See [TERRAIN_STUDY_REPLAY](docs/TERRAIN_STUDY_REPLAY.md). Invalid automated definitions exit before GPU startup.

`--panel section-map` opens the compact geographic trace view; Graph view keeps the profile and row values prominent, with setup/import collapsed.

## Habitat engineering

See [HABITAT_ENGINEERING](docs/HABITAT_ENGINEERING.md) for the reduced model, complete controls, advanced JSON and limitations. CLI `habitat-definition --target Mars --latitude -4.59 --longitude 137.44 --output /tmp/habitat.json` emits a protected editable setup for the chosen world. `habitat --definition /tmp/habitat.json --output /tmp/habitat-day-1` defaults to 25 actual hourly samples on a Lab world. `habitat --initial /tmp/habitat-day-1/initial.cosmo --output /tmp/habitat-replay-1` replays without adding another habitat. --initial restores an exact world; --habitat-id selects among multiple experiments.

Native `--load /tmp/habitat-day-1/final.cosmo --focus Mars --panel habitat --quiet --capture /tmp/habitat.png --capture-metadata --capture-after 240 --quit` shows completed budgets. --panel habitat-controls|habitat-build|habitat-model opens other tabs; --habitat-definition FILE explicitly builds a new experiment. Automated invalid definitions and worlds are rejected before GPU startup. Ordinary loading remains asynchronous. Photo metadata retains actual reports. The same engine owns each fixed minute; high cosmic clock speeds can be CPU limited. Exports protect existing files/folders and retain exact worlds, source/model versions, numerical values and unavailable uncertainty.

## Greenhouse planning

See [GREENHOUSE_PLANNING](docs/GREENHOUSE_PLANNING.md). CLI `greenhouse-definition --output NEW_FILE` emits inputs, `greenhouse --definition FILE --output NEW_DIR` calculates source-based crop/lighting rows and `--initial SAVE --habitat-id ID` optionally retains exact observation context. Native --panel greenhouse and --greenhouse-definition FILE open the controlled crop planner. Photos retain actual inputs/results. Physical greenhouse installation and habitat resource coupling remain unimplemented.

Optional habitat oxygen generation: see [HABITAT_ELECTROLYSIS](docs/HABITAT_ELECTROLYSIS.md). Native `--panel habitat-oxygen` shows actual reaction stocks/flows and staged equipment controls; imported definitions can explicitly enable `config.electrolysis`. Existing CLI habitat studies and native photos retain the same optional chemistry, source, power/thermal budget and model pin. Legacy defaults keep the reactor absent.

Optional CO₂ reduction: `--panel habitat-water` opens actual reaction/storage/recycler results. See [HABITAT_SABATIER](docs/HABITAT_SABATIER.md) for typed definitions, source assumptions and finite-product limits.

Habitat recordings: `cosmogon-cli habitat-inspect --recording STUDY_DIR`; native `--habitat-recording STUDY_DIR --habitat-metric water --habitat-sample 24 --panel habitat-recordings`. Graphs and photo metadata retain actual independent recorded rows; see [HABITAT_RECORDINGS](docs/HABITAT_RECORDINGS.md).

Controlled habitat sweeps and exact-time comparisons: [HABITAT_COMPARISONS](docs/HABITAT_COMPARISONS.md), including shared headless and native capture examples.

## Habitat WHY verification

See [HABITAT_DIAGNOSTICS](docs/HABITAT_DIAGNOSTICS.md) for live/recorded panels and `cosmogon-cli habitat-diagnose --initial SAVE --habitat-id ID [--output NEW_DIR]`. Native captures retain exact shared source terms and selected observation scope; no world advancement by diagnosis.

## Authored camera paths

`--camera-path FILE --camera-path-time SECONDS --panel paths` opens a validated matching-world definition and freezes a pose for native capture. `--camera-path-play` explicitly starts ordinary playback and conflicts with captures. Paths/default launch use Solar System Lab; existing camera/surface/tour/menu/time-advance conflicts are refused. See [CINEMATIC_PATHS](docs/CINEMATIC_PATHS.md) for protected files and complete examples.

## Keyboard customization

`--input-bindings FILE --panel controls|controls-everyday|controls-files` validates a complete physical-key mapping before GPU startup and opens native controls. `--panel help` opens mapping-aware help. Capture metadata retains actual controls; input remains ignored in automated captures. Runtime imports require explicit saving to persist. See [INPUT_BINDINGS](docs/INPUT_BINDINGS.md).

Crop planning `--panel greenhouse-water|greenhouse-daily-water|greenhouse-inputs|greenhouse-compare|greenhouse-model|greenhouse-files` selects the actual water/assumption/comparison/source layouts for captures. `--greenhouse-definition` reads bounded regular version-1/2 inputs and fails invalid definitions before rendering. See [TRANSPIRATION_PLANNING](docs/TRANSPIRATION_PLANNING.md).

Finite cabin moisture: explicit `config.moisture` enables the model; `--panel habitat-humidity|habitat-humidity-settings|habitat-humidity-model` captures Overview/Settings/Model. Native and CLI invalid definitions are bounded and refused before GPU startup. `habitat-inspect` and native `--habitat-metric vapor|condensate|humidity|condensation_heat` reopen actual observations. See [HABITAT_HUMIDITY](docs/HABITAT_HUMIDITY.md); predictive accuracy remains uncalibrated.

Controlled moisture studies reuse the existing native/CLI habitat sweeps; [MOISTURE_STUDIES](docs/MOISTURE_STUDIES.md) lists canonical settings/units and example commands. Humidity settings must already be enabled; all ordinary branch edits validate before output.

[PAIRED_HABITAT_EXPERIMENTS](docs/PAIRED_HABITAT_EXPERIMENTS.md) documents schema-2 paired plans, CLI sweep/inspection, native root reopening and `--panel habitat-paired` capture equivalents.

[PAIRED_TRAJECTORIES](docs/PAIRED_TRAJECTORIES.md) documents CLI `--observation` and protected analysis `--output`, native curve/table/files capture panels and explicit effect selection.

[HABITAT_REPLAY](docs/HABITAT_REPLAY.md) documents `cosmogon-cli habitat-verify`, native `--habitat-verify`, replay captures and protected report outputs.

[RECORDED_GRAPHS](docs/RECORDED_GRAPHS.md) documents native `--habitat-graph FILE`, layout imports/exports/preferences and `--panel habitat-graph-layout` captures.

[CABIN_BREACH](docs/CABIN_BREACH.md) documents optional `config.breach`, protected CLI studies, new metrics/settings and native `--panel habitat-breach|habitat-breach-settings|habitat-breach-model` captures.

## Recorded pressure explanations

`cosmogon-cli habitat-pressure --recording ORDINARY_BRANCH_DIR --observation 12 --reference-index 0 --output NEW_DIR` exports actual components and ordered change terms. Omit reference-index for the previous stored observation; an initial observation has unavailable change accounting. Native `--habitat-recording DIR --habitat-sample 12 --habitat-pressure-reference 0 --panel habitat-recording-pressure-change --capture /tmp/pressure.png --capture-metadata --capture-after 240 --quit` uses the exact same report. Panels `habitat-recording-pressure`, `-change`, `-breach`, `-files` cover components/accounting/resolved substep/evidence. See [guide](docs/RECORDED_PRESSURE.md) and [verification](docs/RECORDED_PRESSURE_VALIDATION.md).

Geometry/thermal studies use four typed settings and plan-dependent pressure catalogs; [guide](docs/THERMAL_STUDIES.md), [verification](docs/THERMAL_STUDIES_VALIDATION.md). Capture `--panel habitat-experiments` with a paired recording or automatic sweep to retain its actual editable controls; use `habitat-paired-files`, `habitat-paired-table`, and existing pressure/WHY aliases for completed results.

Recorded figures use `cosmogon-cli habitat-figure --recording FOLDER --definition GRAPH_LAYOUT --output NEW_DIR [--reference ORDINARY_B]`. Native `--panel habitat-figure --habitat-figure-output NEW_DIR` captures actual loaded evidence through the shared worker; add `--habitat-graph FILE` for an exact reusable layout. See [guide](docs/RECORDED_FIGURES.md) and [verification](docs/RECORDED_FIGURES_VALIDATION.md).

Recorded balances: CLI `habitat-balance --recording FOLDER [--from-index N] [--observation N] [--output NEW_DIR]`; native `--panel habitat-recording-balance[-electricity|-temperature|-counters|-files]`, `--habitat-balance-from N`, `--habitat-sample N` and `--habitat-balance-output NEW_DIR`. See [guide](docs/RECORDED_BALANCES.md). Both indices refer to actual retained rows; export waits for recording/sweep workers.

Cumulative cabin heat capture: use `--panel habitat-recording-balance-heat` with an enabled recording; generated definitions set `record_heat: true`. Balance outputs add protected `heat.csv` only when actual counters exist. [Guide](docs/CABIN_HEAT_ACCOUNTING.md).

Cumulative heat trajectories use existing `habitat-compare --measure heat_storage`, reusable graph layouts and `habitat-figure`/native `--habitat-metric` selections. See [guide](docs/HEAT_TRAJECTORIES.md). Heat metrics are explicit choices; immutable existing aggregate catalogs stay unchanged.

Crop supplies: `cosmogon-cli greenhouse-supply-definition`, `greenhouse-supplies --definition FILE --output NEW_DIR`, `greenhouse-supply-inspect --recording DIR`. Native `--greenhouse-supplies FILE_OR_DIR --panel greenhouse-supplies` opens the same calculation; `--greenhouse-supply-output NEW_DIR`, `--greenhouse-supply-day 0..30`, `--greenhouse-supply-resource liquid_water|carbon_dioxide|nutrient_hno3_equivalent|lighting_energy` support reproducible captures. See [guide](docs/FINITE_CROP_SUPPLIES.md).

Crop chemistry uses `crop-chemistry` / `crop-chemistry-inspect` and native `--panel greenhouse-chemistry[-model|-files]`, with original `--greenhouse-supplies` or verified `--crop-chemistry` source and protected `--crop-chemistry-output`; [commands and units](docs/CROP_CHEMISTRY.md).
