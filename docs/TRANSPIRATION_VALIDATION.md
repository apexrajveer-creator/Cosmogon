# Controlled transpiration verification

This build follows keyboard checkpoint `46f516e`. No Universe/save-state law changes; golden remains `0xf2c4140a45ae509b`. See [TRANSPIRATION_PLANNING](TRANSPIRATION_PLANNING.md) and the preimplementation [research memo](TRANSPIRATION_RESEARCH.md).

## Required checks

Release workspace tests: **308 passed, one existing ignored**. Five additional shared-engine tests cover independent published-equation arithmetic and fine quadrature across all five regimes, area scaling and lighting-efficiency independence, humidity/model/unknown-field refusal, bit-identical legacy biomass and blank unavailable water cells, and FIFO refusal before opening. A native planner test checks legacy/unavailable water, valid modern import and unchanged prepared result after failed import.

Release all-target workspace clippy passes with the existing warning baseline (native binary 11, test target 13); no new warning. Release app/CLI build, local macOS packaging, ad-hoc signature and disk-image checksum pass. This does not establish Developer ID/notarization or Windows/Linux behavior.

## Numeric, file and native evidence

**52 distinct cases**: 14 successful offline cases, 22 expected refusals and 16 native captures. Successful cases include all source regimes, dry/humid/saturated assumptions, legacy/water-disabled definitions, large area, exact read-only world context and replay. Refusals cover invalid/future/unknown definitions, unsupported pressure/temperature, negative source conductance, bounded regular-file/FIFO/directory inputs, protected existing export, invalid context and ambiguous duration. Five native launch refusals precede GPU creation. An independent 100,000-point quadrature and equation calculation check numeric JSON/CSV, all 21 columns, source hashes, unavailable legacy cells, exact context and unmodified input files. This is numerical verification, not crop calibration or biological validation.

Sixteen accepted final layouts were visually inspected: Forecast, total/daily water, Inputs, Compare, Water model, Files, legacy Forecast/Water model, disabled water, saturated/dry/source-300 regimes, existing keyboard Controls, live habitat WHY and authored Paths. Actual sidecars match the shared forecast and prepared definitions. Shortening the header exposes the last vapor row and input lamp power above the dock. Intermediate captures are retained; repeats do not increase case counts.

Those sixteen captures precede the final performance-resource gathering refactor, which removes a clippy warning without altering the planner or UI. Two additional packaged captures after that refactor (water and keyboard Controls) were inspected, and the latest ordinary run checks the exact forecast again. These repeat captures do not add distinct cases/layouts.

## Ordinary run and boundaries

The latest packaged app runs normally for 48 seconds, with the water graph and live native UI, no quiet/capture/quit flags. A 900-frame report after 120 warm-up frames checks pipelined rendering and the exact native definition/forecast against shared CLI export; the harness ends the observation. See retained `ordinary-run.json` for actual elapsed time and exit details. No panic occurred. This does not exercise manual edits or physical keys.

Ignored local evidence in `dist/transpiration-validation/` contains definitions, exact exports/context, commands, actual captures/sidecars, intermediate layouts, required checks, normal-run report and source/artifact hashes. No manual-input, assistive-technology, alternate-platform, facility calibration, physical greenhouse or complete master-directive claim. No public push/release. Finite cabin humidity and condensate handling follows.
