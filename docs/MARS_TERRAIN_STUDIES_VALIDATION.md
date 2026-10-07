# Terrain-section verification — 2026-10-07–08

Continuation after geographic exploration checkpoint `ccb6963`. The section engine, actual native
height/distance graph, row-to-probe/travel actions, protected numeric bundles and headless replay are
implemented. No Universe state/schema, dataset, dependency or golden change. This remains a local
unreleased development build, not a new public release or a calibrated planetary instrument.

## Checks and evidence

- Exact final full release workspace tests: **227 passed plus 1 existing ignored** (app 50, CLI 3,
  core 32, physics 38, simulation 103, documentation 1). The golden remains `0xf2c4140a45ae509b`.
- New tests cover known equatorial/polar arcs and seam crossing; uniform distance rows; exact endpoint
  coordinates; agreement with the authoritative height field; repeatability; zero-length sections;
  antipodal and coincident-coordinate ambiguity; sample/radius/version/site validation; bounded 64 KiB
  definition reading; unsupported definitions; exact export replay and existing/empty-folder protection.
  CLI checks reject silently ignored endpoint/target/sample overrides and accept supported definitions.
- Release clippy succeeds with the existing core 2 / physics 1 / simulation 9 / app 11 warnings and no
  new warning. Release native/CLI builds succeed.
- Headless/native QA: **18 cases**, all expected statuses. The four-file packet's numeric CSV and JSON
  agree; replay repeats byte-for-byte. Three selected rows independently agree with the coordinate
  probe. Existing packet hashes survive a refused export. Invalid samples/target/overrides/site and
  legacy terrain return status 1. Seam and coincident definitions calculate correctly; antipodal
  input reports ambiguity. A separate source-backed check at (89.875 N, 1.375 E) retains zero-shot
  provider-interpolation flags with null uncertainty.
- Native captures inspected: the actual section graph, the graph over a tracked geographic horizon
  view, and the explanation on a legacy procedural world. The graph/row/units match the exported
  Curiosity–InSight study: 64 rows, reference radius 3,389,500 m; **548.3961224114657 km**, sampled
  height range **−4335.925200000004 to −1540.58749040772 m**. These are samples of the coarse field,
  not continuous-path extrema or measured local precision. Both axis-scaling and source-resolution
  limits are visible; the legacy scene does not silently acquire observed terrain.
- Packaged ordinary native run: **48.03 seconds**, 1800 measured frames after 120 warmup frames,
  completed report, no panic, ordinary input and `pipelined_rendering: true`. The runner intentionally
  stops the process afterward. This is a sustained smoke test, not a controlled performance comparison.
- Local native arm64 app/DMG rebuilt. Ad-hoc signature and disk-image CRC verify. No new public
  release, notarization, remote push or cross-platform manual verification is claimed.

Logs: `/tmp/cosmogon-section-tests-exact.log`, `/tmp/cosmogon-section-clippy.log`,
`/tmp/cosmogon-section-build-exact.log`, `/tmp/cosmogon-section-package.log`,
`/tmp/cosmogon-section-dmg.log`. Captures, definitions, actual rows, sources/checksums, commands/results,
performance samples and checkpoint identity are retained in ignored `dist/terrain-section-validation`.

## Remaining limits

Manual keyboard/pointer/accessibility and native export/row-action button interaction have not been
independently exercised; source calculations, shared geographic travel and actual rendering are verified.
Native definition import/durable study preferences, an on-globe transect line, full geodesic modelling,
continuous extrema and uncertainty propagation remain future work. Current data is 0.25° overview;
finer sampling adds no spatial information and cannot establish local slopes, route safety or landing
safety. Current rover tracks and mission imagery remain absent. A source field with interpolated support
is labelled explicitly rather than acquiring a fabricated confidence score.
