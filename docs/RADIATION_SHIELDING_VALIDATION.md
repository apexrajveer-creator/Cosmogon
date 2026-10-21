# Radiation environment and cabin shielding dose — verification (build 57A)

## Automated suite

* `cargo test --release --workspace`: **674 passed, 0 failed, 1 existing ignored**
  (build 56 baseline 662 + 12). Breakdown: cosmogon 131, cosmogon_cli 28, cosmogon_core
  32, cosmogon_physics 38, cosmogon_sim 444, doctests 1. New coverage: the ten
  `habitat_radiation` unit tests (default config valid and authored, factor/band/threshold
  range refusal, `deny_unknown_fields` extra-key refusal, exterior dose matches the three
  published anchors — vacuum exactly 1.8, Mars ≈ 0.67, Earth in [0.001, 0.02], flare
  activity ×5 at f=1 — exterior dose monotonic in every input with clamping, hazard bands
  at the published thresholds, budget applies the authored transmissions over the
  interval, budget validation refuses tampering and domain breaks, config/budget JSON
  round trip, source declares law/anchors/bands/limits) and two integration tests
  (`radiation_budget_applies_authored_law_alerts_and_extends_csv` in `habitat.rs`:
  exterior 3.0 → interior 1.5 → shelter 0.375, interval dose = interior·dt/86400,
  cumulative accumulation across intervals, alert bit 7 set at interior > 1.0 mSv/day,
  disabled twin records nothing, report keys present only when used, CSV grows by exactly
  six columns ending in `radiation_hazard_band` with header/row widths equal;
  `save::tests::radiation_pin_is_conditional_and_enforced`: pin absent on plain worlds,
  `cosmogon-habitat-radiation/1` present when configured, reopen OK, stripped and
  tampered pins refused with `SaveError::Model`).
* `cargo check --release --workspace --all-targets`: clean. The 20 test-only
  `Environment` literals across sim and one in the app UI gained
  `exterior_dose_rate_msv_day: None` (cfg(test) is invisible to plain `cargo check`).
* `cargo clippy --release --workspace --all-targets`: zero warnings in touched files.
  One new lint (`field_reassign_with_default` in the radiation validation test) was fixed
  by switching to struct-update construction. All remaining warnings pre-date this build:
  `save.rs:26` complex-type const — byte-identical at `7442d8e` — `nbody.rs` test-only
  `unused_mut` (present on stashed HEAD), maintenance/dock loop-index lints, etc.
* Fresh release builds of `cosmogon` and `cosmogon-cli` after all edits.
* Two inspected app captures (`--capture`): the Saturn lab scene with and without
  `--hide-ui` — the app boots, renders and autosaves with the new UI module compiled in.
  The Radiation shielding group sits directly below the life-support calibration group
  in the habitat configuration form, below the fold at the default pane height (same
  verified-by-tests precedent as build 56's calibration group).

## End-to-end loop (`/tmp/b57a`)

1. `habitat-definition` (SolarSystemLab, Mars) → inject
   `"radiation": {"model": "cosmogon-habitat-radiation/1", "wall_transmission": 0.5,
   "storm_shelter_transmission": 0.25, "alert_dose_rate_msv_day": 1.0}` →
   `habitat --definition def.json` completes a 25-sample study.
2. Study CSV is **116 columns** (110 base + the six-column radiation tail). The t=0 row
   carries empty dose cells, cumulative `0` and band `unavailable` (empty means
   unavailable, never zero); the final row reads exterior **0.6713548408966546 mSv/day**
   (the authored law at Mars's actual surface pressure — the ~0.67 published anchor,
   reproduced from world inputs, not hard-coded), interior **0.3356774204483273**
   (= exterior × 0.5), shelter **0.08391935511208183** (× 0.25), interval
   **0.00023310931975578287 mSv** (= interior × 60 s / 86400), cumulative
   **0.33567742044833243 mSv** (= 1440 fixed-minute intervals × the interval dose,
   exactly 24 hours of accumulated interior dose) and band **elevated**
   (0.137 < 0.336 ≤ 1.0).
3. `report.json` carries `radiation_observation` (model pin, five values, band label,
   cumulative), `radiation_source` (law, anchors, bands, limits),
   `source.radiation` and `data_states.radiation` (ESTIMATED design inputs).
4. `manifest.json` carries `radiation_source` and `radiation_model_descriptor`;
   `final.cosmo` header carries
   `data_versions["cosmogon-habitat-radiation"] = "cosmogon-habitat-radiation/1"`.
5. `habitat-diagnose` reports the five `radiation` terms with units and a radiation
   explanation (exterior → interior → shelter with the authored percentages, interval
   and cumulative dose, band, threshold and the reduced-law disclaimer);
   `source.radiation` is present in the diagnostic report.
6. `habitat-inspect` reopens and validates the recording (conditional manifest keys
   compared against the live source and registry descriptor).
7. Authoring refusal: `wall_transmission: 1.5` → "Wall dose transmission must be finite
   and between 0 and 1" before any study output is written.
8. Legacy control: the same definition without the radiation block produces a
   **110-column** CSV, no radiation report/manifest keys and no save pin.

## Regression guarantees

* Worlds without a radiation config skip the model entirely: `Budget.radiation` stays
  `None`, `Totals.cumulative_dose_msv` serializes skip-if-zero, and pre-build-57
  recordings, reports and diagnostic exports read unchanged (the diagnostics field is
  serde-skipped while empty and the manifest keys read as null on old folders).
* The golden remains **`0xf2c4140a45ae509b`** (unchanged in the suite — radiation is
  default-absent and never touches serialized state).
* No new `Edit` variant and no new CLI commands; the pin appears only when a radiation
  config is present, and `Config` keeps `deny_unknown_fields`.
* The alert list is appended to (bit 7), so the seven existing alert bit positions and
  their event detail strings are untouched (covered by the existing alert and event
  tests).
