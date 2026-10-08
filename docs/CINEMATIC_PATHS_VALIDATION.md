# Authored camera path verification

This is the partial camera build described in [CINEMATIC_PATHS](CINEMATIC_PATHS.md), following habitat WHY checkpoint `bb8aae5`. The simulation, save schema and golden `0xf2c4140a45ae509b` are unchanged.

## Required checks

`cargo test --release --workspace`: **294 passed, one existing ignored**. Eleven added tests cover finite/schema/context validation, exact astronomical endpoints, easing/lens interpolation, shortest quaternion rotation, negative-Z look-at/poles, moving/missing anchors, world replacement/manual changes, authoring and protected files, imported-save identity and launch conflicts. The editor itself is rendered without input while the fractional playback clock advances: none of its three views may scrub or pause it.

`cargo clippy --release --workspace --all-targets` passes with existing warnings; no new warning in cinematic implementation. Release app/CLI and macOS app/DMG packaging succeed. The ad-hoc app signature and disk image are verified locally; this is not signing/notarization or Windows/Linux runtime evidence.

## Native and refusal cases

Thirty-three distinct cases: sixteen invalid pre-GPU launches, fourteen native path captures and three existing-feature regressions. Invalid future/unknown fields, missing world/object, invalid lens/quaternion/timing, degenerate look-at, oversized/nonregular files, missing file, nonfinite/out-of-duration phase and ambiguous launch combinations are refused. Source hashes are unchanged. Pipe refusal occurs before opening the FIFO.

Actual GPU images cover empty editor, start/intermediate/boundary/smooth/end poses, Keys and Files, authored orientation, pole look-at, absolute frame, different look-at object, below-ground authored position and reopened saved state. Sidecars are compared with independently calculated interpolation, lens and negative-Z viewing direction; intended and actual camera agree except the explicitly reported existing ground-clearance correction. Saved habitat input is byte-identical. No physical trajectory or landscape registration is inferred from these images.

Seventeen accepted final layouts are inspected, including Mars landing travel, live habitat WHY and ordinary Saturn photo controls. Earlier captures revealed buried editing/file controls; Compose/Keys/Files views, a compact key selector and a taller bounded panel corrected this. Intermediate screenshots are retained but not all were visually inspected, and repeated captures do not increase the distinct case count.

## Ordinary playback and limits

The packaged app ran for 48.04 seconds with ordinary pipelined rendering, a 120-second authored path, live simulation, the path panel and a 900-frame report after 120 warm-up frames. The report recorded 17.2881935 advancing presentation seconds. The validation harness stops the process after the observation period and verifies an active advancing presentation clock, unchanged definition and actual camera agreement in the report. This specifically caught an untouched slider rounding the clock and pausing playback; clamping only user edits plus the editor regression prevents it. The failed run is retained separately.

Ignored local evidence: `dist/cinematic-validation/` contains definitions, actual PNG/JSON/logs, commands, required-check logs, packaged playback report and source/artifact hashes. Full keyboard/mouse interaction, assistive technology, alternate platforms, video, DOF, spline/arc-length speed and simulation-rate synchronization remain unverified or unimplemented. The master directive remains incomplete; native keyboard customization follows. No push or public release occurred.
