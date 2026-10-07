# Mars source refinement verification — 2026-10-08

Connected build after terrain-section checkpoint `ad6f5b4` and source research `6a166f7`.
The 16ppd field, coverage, explicit edit/undo, shared native/headless reads, source-version protection
and cache invalidation are implemented. Existing worlds/defaults retain their field and the golden.
This is an unreleased local arm64 build; no remote push/public release or notarization is claimed.

## Evidence

- Final release workspace suite: **232 passed plus 1 existing ignored** (app 51, CLI 4, core 32,
  physics 38, simulation 106, documentation 1). Golden `0xf2c4140a45ae509b` passes unchanged.
- Release clippy: existing core 2 / physics 1 / simulation 9 / app 11 warnings only. Native/CLI builds pass.
- Public PDS binaries and both XML labels match the provider archive MD5 inventory. Raw/label/output
  SHA256 pins are stored in metadata; inventory itself is pinned and retained. Range retrieval checked
  each requested offset, content range and byte count, then checked whole-file hashes. Raw topography
  range −8177..21171 m; observation counts 0..326 with **903558 zero-shot bins**.
- Conversion took about 1.11 seconds in the local processing run; reprocessing yielded byte-identical
  grids/labels/metadata. A changed label was refused before any output directory/write. Both products
  total 66,355,200 source bytes. No resampling, generated elevations or invented coverage.
- Semantic tests verify independent raw-pixel fixtures, extrema/zero-count totals, axes/centres,
  poles/seam, source weights, shared geometry/heights, unknown uncertainty, source-dependent cache
  identity, deterministic edits, preserved orbital/dynamical state, exact app undo, read-only refusal,
  tracking release, version-pinned bookmarks, source-specific section exports/replay and save failures.
- **42 expected headless/native case statuses**, plus two explicit Lab captures. Probes repeat exactly;
  exported CSV/JSON and replay agree; three section rows independently agree with probes. Source
  mismatches, unsupported versions, missing 16ppd manifest version, wrong body, ignored/invalid flags,
  legacy probes and existing output overwrite are refused. Explicit legacy attachment succeeds.
- Inspected seven native captures: 4/16ppd saved-world and Lab surface views, 16ppd section graph,
  low-altitude horizon, legacy explanation. Lab sidecars pin the actual source and tracked coordinate.
  Historical Sol saves can show physically dark terrain; the Lab pair separately verifies a lit view.
  The 64-row Curiosity–InSight section remains **548.3961224114657 km** on the 3,389,500 m sphere;
  the 16ppd sampled height range is **−4496.613600000001 to −729.9778154688512 m**. This is an
  implemented-field comparison, not measured error relative to truth or continuous-path extrema.
- Warm source-query microbenchmark: median **44.39 ns/query** (4ppd) and **58.20 ns/query** (16ppd),
  five 200000-query runs after warmup. Example `mola_profile` is retained for reproduction. These
  numbers exclude load, simulation, textures and GPU; they do not establish app FPS or memory budgets.
- Packaged ordinary 16ppd Mars/section run: **48.07 seconds**, 1800 measured frames after 120 warmup,
  correct dataset context and pipelined renderer, completed report, no panic. Runner then intentionally
  terminates the process. This is a sustained smoke run, not a controlled performance comparison.
- Updated local app/DMG pass ad-hoc signature and disk-image CRC. Binary is about 124 MiB; disk image
  about 68 MiB. The finer pair is immutable embedded bytes and is not duplicated into a heap grid.

Logs: `/tmp/cosmogon-mola16-{tests-exact,clippy,build,package,dmg,qa,sustained}.log`;
query measurement `/tmp/cosmogon-mola16-query-profile.log`. Source-backed readings, packets, commands,
captures/sidecars, run samples and checkpoint identity are retained in `dist/mola-refinement-validation`.

## Remaining limits

Manual source-button/undo, native export interactions, keyboard/pointer/assistive technology and
Windows/Linux native behavior remain unverified. Automated source edits, model calculations,
app undo/tracking tests and actual rendering are verified. The Surface panel scrolls to controls
below the current viewport; not every control is simultaneously visible.

The 16ppd grid remains an overview with provider-interpolated support and missing covariance.
Its 3.704 km pixel spacing is not independent measured ground resolution; provider profile gaps
reach 12 km. The app sphere is not a full areoid/ellipsoid or SPICE registration. No current rover
track, mission image, local engineering slope or landing-safety claim. Optional pack/download delivery,
native section import/durable study preferences and on-globe transects remain follow-on work.
