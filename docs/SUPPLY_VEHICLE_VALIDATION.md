# Finite supply-vehicle verification — 2026-10-10

Local macOS arm64 checkpoint; public release remains 1.0.0. Graphics implementation remains deferred. Evidence: ignored `dist/supply-vehicle-validation/`, with source/file hashes, exact commands/results, complete authored studies, seven final PNGs/metadata and normal app profile. No push or release performed. The full master directive and phase 10 remain incomplete.

## Required checks

- Release workspace tests: **446 passed, one existing ignored, zero failed**, including the unchanged absent-world golden `0xf2c4140a45ae509b`.
- Release all-target workspace clippy completes, with no warnings in the new vehicle modules. Existing core/physics/simulation/native warnings and dependency `block 0.1.6` future-compatibility warning remain.
- Release native/CLI build, macOS packaging, strict ad-hoc app signature and disk-image verification all pass.
- Frozen production file hashes match the tested/captured code. The master directive's SHA-256 stays `03dcdfaf0a4fd5a5548cde2fb5d700fd67df648e8691c5322117ca652ff94711`.

## Numerical and boundary evidence

Eleven added tests cover actual finite depletion, retained cargo, analytic ideal capability/impulse, coasts/zero thrust/zero duration/empty tanks, exact aligned/fractional depletion phases, sampling/chunking tolerance, stable tiny mass ratios and explicit below-inventory-precision refusal, atomic invalid inputs, vector versus scalar capability, protected full numeric/source reopening and native request-time pinning/preserved later draft edits. Failed reopen/run and out-of-range requested rows cancel dependent output rather than writing previous retained results. Native automation validates actual plan rows and existing outputs before GPU startup; CLI refuses unrelated world flags.

The first development core test expected accumulated interval exhaust to equal initial fuel bit-for-bit. Actual residual was approximately 4.55e-13 kg. The model retains that raw residual; the analytic mass identity uses a numerical tolerance and independent same-plan replay stays exact. This was a test expectation correction, not hidden normalization of mass or counters. Initial development clippy findings were fixed before final required checks.

**59 distinct workflow cases** cover complete definitions/studies/inspection, exact summary consistency, analytic fuel/capability/cutoff checks, every 24-column CSV row, eight distinct valid designs and exact reopening, nine invalid typed/precision cases with absent destinations, four changed/incomplete recording cases, protected outputs, parser context refusals, nine pre-GPU native refusals, seven actual packaged captures, native-plan export/recorded-plan rerun byte equality and the prior completed frozen cabin design reader. The example retains 362 actual observations, with exact fuel depletion at row 355 / 354.1995 bench seconds. Cargo remains 170 kg and final vehicle mass is 1170 kg; no habitat delivery occurs.

## Actual packaged presentation

Seven final 2940×1660 app PNGs inspected: Design, Burn plan, selected depletion, empty tank, opposing burns, Models & files/export, and reopened-plan rerun. Labels, units, raw residuals, source URLs, scope, actual acceleration availability and completed source identity are visible. Long panes scroll; later burn/custom/import controls and some bottom status text require scrolling, and the existing floating tool dock can overlap the window's lower edge. This remains a presentation rough edge, not claimed manual interaction coverage.

The normal packaged run lasted **48.331491 seconds**, retained 900 measured frames after 120 warmup, and kept pipelined rendering enabled. It completed without a logged panic and retained the exact checked manifest in performance metadata. Only this task's own process received SIGTERM after the planned run. Short automated captures disable pipelining for the known renderer teardown issue. The packaged app SHA-256 is `39f76eb69d5c38b570a2d72f3136afd9bf75a584e28f67bfd07adf11b3fac4c8`.

No new Universe/save state, simulation law, dataset/source constants, shader, generated imagery, dependency or golden re-pin. No manual pointer/keyboard/hover/clipboard/hyperlink, narrow viewport, accessibility, Windows/Linux, calibrated hardware, independent empirical validation, actual gravitational flight, launch/landing or cargo-transfer claim.

## Next connected build

Attach finite vehicle state to deterministic f64 world-relative dynamics, explicit reference frames and real gravity, then physically supported arrival/transfer into the same finite settlement stores. Keep the design bench and its exact existing recordings intact. Full spacecraft hardware, construction/maintenance, cabin calibration, detailed graphics and the remaining directive continue separately; this checkpoint does not complete those phases.
