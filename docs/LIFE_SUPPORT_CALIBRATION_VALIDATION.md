# Life-support calibration and sensitivity envelopes — verification (build 56)

## Automated suite

* `cargo test --release --workspace`: **662 passed, 0 failed, 1 existing ignored**
  (build 55 baseline 651 + 1). Breakdown: cosmogon 131, cosmogon_cli 28, cosmogon_core 32,
  cosmogon_physics 38, cosmogon_sim 432, integration 1, doctests 0. New coverage: the six
  `habitat_calibration` unit tests (identity default config, factor/band range refusal,
  `deny_unknown_fields` extra-key refusal, band wrapping of applied exchanges, budget
  band-arithmetic validation with tamper refusal, budget JSON round trip) and five habitat
  tests (identity calibration preserves physics and records bounds, raised crew demand
  draws faster with bands wrapping applied, calibrated processors scale capture and
  processing exchanges, calibration world deterministic with additive CSV tail, calibration
  pin lifecycle plus invalid-configuration refusal).
* `cargo clippy`: zero warnings in touched files (fresh log after the final habitat.rs
  edit; all remaining warnings pre-date this build: `save.rs` complex-type const — byte-identical
  at `7442d8e` — maintenance loop-index, etc.).
* Fresh release builds of `cosmogon` and `cosmogon-cli` after all edits.

## End-to-end loop (`/tmp/b56`)

1. `cosmogon-cli habitat --definition def.json` with four authored factors injected
   (`crew_demand` 1.3 / ±0.15, `scrubber` 1.0 / ±0.10, `recycler` 1.1 / ±0.10,
   `oxygen_makeup` 1.0 / ±0.05) → 25-sample study completed; the output carries
   `definition.json`, `final.cosmo`, `report.json` and `samples.csv`.
2. Study CSV is **122 columns** (110 base + the 12-column `calibration_*` tail), 26 lines
   (header + 25 rows); the t=0 row is all zeros and 24/25 rows carry nonzero calibration
   values.
3. First nonzero row band arithmetic is exact: `crew_cycle` applied 0.003611111111111111
   (= 1.3 × the published reference expression), low 0.0030694444444444444 (applied × 0.85),
   high 0.004152777777777777 (applied × 1.15); oxygen makeup 0.0021793142627757334
   [0.0020703485496369467, 0.00228827997591452]; CO₂ captured 0.004166666666666667
   [0.00375, 0.004583333333333333]; water processed 0.01613083333333333
   [0.014517749999999998, 0.017743916666666665].
4. Export `report.json` carries `calibration_observation` (model
   `cosmogon-life-support-calibration/1`, the four factors, per-interval applied/low/high),
   `calibration_source` (law `applied = reference expression × central; low = max(0,
   applied·(1−band)); high = applied·(1+band)` with the authored-bounds limits text),
   `data_states.life_support_calibration` (ESTIMATED design inputs) and
   `uncertainty_scope` (bands are not measured variance or confidence intervals).
5. `final.cosmo` header carries exactly
   `data_versions["cosmogon-life-support-calibration"] = "cosmogon-life-support-calibration/1"`.
6. `habitat-diagnose` on the final world: one "calibration" explanation (id `calibration`)
   with the retained factor/band numbers and the authored-bounds caveat; 16 "calibration"
   hits in the diagnostic report, 0 in `terms.csv`.
7. Refusals (both exit 1, nothing applied), via `cosmogon-cli habitat --definition …`:
   `central: 15.0` → "Crew demand calibration factor must be finite and between 0.1 and 10";
   `relative_band: 2.0` → "Crew demand relative band must be finite and between 0 and 1".

## Native captures (inspected)

* `calibration.png` — Habitat window, **Budgets** tab: NASA reference crew rates header,
  tab bar, world/site line, fixed-minute model at 1 min/s, cabin 293.15 K, total pressure
  101.10 kPa, O₂ partial 21.20 kPa, battery 20/20 kWh, power 5318.9/1931.5 W, stored
  oxygen 27.139 kg, potable/wastewater 265.399/0.000 kg, dry food 34.870/33.126 kg.
* `calibration-controls.png` — **Controls** tab: staged-change hint, the collapsed
  structural/moisture/leak/chemistry headers and the open "Crew and cabin" section
  (nominal crew demand 4). The "Air and water processors" and "Life-support calibration"
  headers sit below the fold at the default 480 px window height — the calibration group
  is verified by unit tests, the end-to-end study and the export report rather than by
  capture pixels (same below-fold precedent as build 55).

## Regression guarantees

* Worlds without a calibration config skip the model entirely: legacy serialization stays
  byte-identical (unit covered) and the golden remains `0xf2c4140a45ae509b` (unchanged in
  the suite).
* No new `Edit` variant and no new CLI commands were added; the pin appears only when a
  calibration config is present, and `Config` structs keep `deny_unknown_fields`
  (bad central/band configurations are refused before any step runs).
* The calibration CSV tail is purely additive (110 → 122 columns) and the calibration world
  is deterministic across reruns (unit covered).

## Limits of this verification

Calibration factors and ± bands are authored sensitivity bounds (ESTIMATED design inputs),
not measured hardware, fitted parameters or predictive statistics; bands are not confidence
intervals and no parameter identification, observed-data validation, probabilistic
propagation or hardware reliability is claimed (as stated in `calibration_source.limits`).
The study was verified on one site and one configuration; other factor combinations are
exercised only by the unit-test matrix.
