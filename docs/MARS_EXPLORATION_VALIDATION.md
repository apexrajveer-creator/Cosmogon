# Mars exploration verification — 2026-10-07

This verifies the historical landing-reference and native geographic-navigation slice after MOLA
checkpoint 013a682. The development build remains version 1.0.0; it is not a new public release.

## Build and scientific boundary

Four cited historical landing references share an engine-independent registry, native selection,
markers and headless JSON. The camera uses the same f64 body frame as CPU terrain placement and
samples the shared MOLA height for radial altitude. Snapshot/tracked modes, ground/horizon views,
look adjustments, release behavior and geographic viewpoints are presentation state outside Universe.
Viewpoint version 2 reads released version 1 and protects invalid/newer libraries. Restoring a tracked
view updates the probe/selection and lens without rewinding the world. No dependency, Universe schema
or golden change; the golden remains `0xf2c4140a45ae509b`.

NASA source summaries substantiate historical landing coordinates/dates, not current rover positions,
mission status, complete frame registration or covariance. Unknown uncertainty remains null. MOLA is a
0.25° overview with source-cell observation counts; it cannot establish local slopes or landing safety.
Body orientation follows the implemented tilt/spin model, not a SPICE navigation solution. Generated
colours/fine shading and the spherical base are disclosed. Marker horizon visibility is spherical,
not detailed terrain occlusion. A snapshot can drift away from its placement coordinate as the planet
moves; ordinary free-camera clearance protection can also reposition it if terrain moves underneath.

## Automated verification

- Full `cargo test --release --workspace`: **223 passed, 1 existing ignored** (app 50, CLI 2,
  core 32, physics 38, simulation 100, documentation 1). Exact final source passed.
- New checks cover historical reference identity/provenance and unknown uncertainty; repeatable
  source-supported probe heights; conflicting named-coordinate requests; f64 frame/height agreement at
  seams/poles and later simulation times; finite horizon/overview orientations; snapshot/release and
  disappeared-body recovery; geometric marker horizon; tracked viewpoint/inspector/lens restoration
  without changing Universe; version-1 reading and newer-file protection.
- Release clippy succeeds with the existing core 2 / physics 1 / simulation 9 / app 11 warning baseline;
  no new warnings. Final native and CLI release builds succeed.
- Native/headless QA: 23 cases, all with expected status. Registry contains four references. Each site
  probe repeats byte-for-byte, preserves missing uncertainty and agrees with shared height. Unknown
  site, coordinate/target conflicts, non-finite altitude, and procedural legacy travel fail visibly
  with status 1. Legacy terrain is not migrated implicitly.
- Final native PNGs inspected: Perseverance ground/probe, horizon with hidden panels, Curiosity snapshot,
  Viking low view, Earth, legacy Mars and orbital historical landing labels. The first capture exposed a probe hidden behind the tool dock;
  the final overview angle places it above the dock and keeps the numerical height in the initial panel.
  Curiosity's snapshot is dark and can drift as Mars moves; no artificial lighting or following is added.
  Earth retains coasts/clouds/settlement markers; legacy Mars retains generated terrain and its explanation.
- Local app/disk image rebuilt. Ad-hoc signature and disk-image CRC verify. Signing changes the whole
  executable hash, so the 15 nonempty Mach-O code/data section hashes were compared and match the raw
  release binary. The app is local/native arm64; no new notarization or cross-platform evidence is claimed.

Exact logs: `/tmp/cosmogon-surface-tests-verified.log`,
`/tmp/cosmogon-surface-clippy-verified.log`, `/tmp/cosmogon-surface-build-verified.log`,
`/tmp/cosmogon-surface-package.log`, `/tmp/cosmogon-surface-dmg.log`.
Source-supported readings, capture sidecars, capture commands/results and payload hashes are retained
under ignored `dist/surface-exploration-validation` alongside the source checkpoint identity.

## Ordinary runtime and limits

The packaged app is also tested with ordinary input and pipelined rendering, without capture/quit
automation: **48.03 seconds**, completed **1800 measured frames** after 120 warmup frames, no panic,
and `pipelined_rendering: true` in the recorded report. The validation runner then intentionally stopped
the process. The actual runtime record and performance report are retained with validation.
Manual keyboard/mouse/accessibility and viewpoint-button interactions are not independently exercised;
unit tests and actual renders verify their shared data/pose paths. Marker picking and protected native
reading export are implemented but are not claimed as manual UI-test evidence. Small-window layout,
full keyboard ergonomics, swept collision, current rover paths and finer observed terrain remain future
work. No performance improvement or publication-grade validation is inferred from this runtime check.

Next connected direction: [numeric terrain sections](MARS_TERRAIN_STUDIES_DESIGN.md), followed by
verified finer source delivery and engineering consequences as their data/validation gates permit.
