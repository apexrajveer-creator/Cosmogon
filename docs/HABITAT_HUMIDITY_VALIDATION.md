# Finite cabin moisture verification

This build follows transpiration checkpoint `0e0d3f0`. Optional vapor/condensate state and model pins preserve old saves and untouched-world serialization. Golden remains `0xf2c4140a45ae509b`. See [HABITAT_HUMIDITY](HABITAT_HUMIDITY.md) and [HUMIDITY_RESEARCH](HUMIDITY_RESEARCH.md).

## Required checks

Release workspace tests: **316 passed, one existing ignored**. Seven shared-engine tests cover independent saturation/phase arithmetic and domain limits, mass/electric/thermal closure, start-stock delays and powered downstream recovery, unavailable power/storage/domain cases, strict input/history refusal, diagnostic closure and missing evidence, and exact chunking/save-resume/source-pinned recording behavior. One native test verifies atomic invalid-capacity refusal, retained stocks, dirty state and exact undo/redo.

Release all-target workspace clippy passes with the existing warning baseline; no new warning. Release app/CLI build, local macOS packaging, ad-hoc signature and disk-image checksum pass. No Developer ID/notarization or alternate-platform claim.

## Numeric, file and native evidence

**57 distinct cases**: 24 successful offline cases, 19 expected refusals and 14 native captures. The independent harness checks ideal vapor pressure, source-supported saturation/RH, all 85 CSV columns, finite store/history closure, phase transfers, allocated electricity and thermal closure. Cases include nominal, zero-power/rate/space/airborne/setpoint, unsupported temperature, coupled electrolysis/CO₂ reduction, held-state disabled/no-bus continuations, exact replay, protected existing output, malformed/future/unknown/oversized/missing/directory/FIFO inputs and corrupt phase/history/model pins. Native definition refusals occur before GPU creation. Older recording reopening and legacy samples/CSV-prefix equality pass.

Fourteen final layouts were visually inspected: Humidity Overview, Settings, Model & sources, supersaturation, unsupported temperature, disabled mechanism, legacy habitat, WHY Heat/Water, recorded humidity/condensate, coupled reactions, existing crop water and keyboard Controls. Capture metadata matches actual shared reports. Settings Apply/Discard and all six inputs and the model source links fit after a layout correction. Six preliminary layouts were inspected before correction; other preliminary images are retained without an inspection claim. Repeated captures do not add cases. WHY phase terms and additional water rows are scrollable but offscreen in these initial captures; no claim of visual verification for offscreen content.

## Ordinary run and boundaries

The latest packaged app runs normally for 48 seconds with live humidity evolution and native UI, no capture/quiet/quit flags. A 900-frame report after 120 warm-up frames verifies pipelined rendering, actual model metadata, completed-time advancement, increasing emitted/condensed totals and mass closure. The harness ends the observation; see retained `ordinary-run.json` for actual elapsed time and exit details. No panic occurred. No manual edits/keys or assistive-technology exercise is claimed.

Ignored local evidence in `dist/humidity-validation/` retains commands, exact source-pinned exports, captures/sidecars, independent harnesses, check logs, ordinary-run data, source images and artifact/source hashes. One preliminary harness expected current configuration rather than the retained completed-interval configuration after disable; that harness was corrected and its intermediate output retained. No production defect is inferred from that expectation failure.

Numerical verification does not calibrate the inferred airborne partition, crew reference rates, assumed equipment, heat law or habitat against a real cabin. Physical greenhouse coupling, leaks, ice/wall condensation, condensate purification and broader settlement fidelity remain incomplete. No push or public release. Controlled moisture studies follow.
