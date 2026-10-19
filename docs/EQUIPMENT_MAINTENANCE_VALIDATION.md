# Equipment calibration and scheduled maintenance: verification evidence (build 54)

All commands run in the repository root on 2026-10-19 unless noted.

## Release tests

`cargo test --release --workspace` → **639 passed, 0 failed, 1 existing ignored**
(app 131, CLI 27, core 32, physics 38 + one doctest, simulation 410 = 639; one pre-existing
ignored app test). New simulation tests (10):

* `habitat_maintenance::tests::cycle_split_is_exact_across_phase_boundaries_and_wrap` —
  operating-only, boundary-crossing and wrap-around splits intersect `[cycle, cycle+dt)`
  exactly and always sum to `dt`.
* `habitat_maintenance::tests::wear_and_restore_follow_authored_rates_and_clamp` — wear
  equals `wear_per_hour · served · op_h` (0.36 over one full served hour), restoration in a
  window follows `repair_per_hour · maint_h` and clamps to remaining headroom, condition
  never leaves [0, 1].
* `habitat_maintenance::tests::validation_refuses_bad_parameters_and_empty_config` —
  calibration above rated, zero repair rate, zero interval, an empty maintenance config and
  an unknown model string are all refused.
* `habitat_maintenance::tests::component_budget_rejects_inconsistent_wear_arithmetic` —
  `ComponentBudget::validate` accepts the exact retained arithmetic and refuses a tampered
  wear value.
* `habitat::tests::scheduled_maintenance_wears_scrubber_then_restores_in_window` — a 60 s
  interval/60 s cycle world: minute one wears 1.0 condition with capture at rated scale;
  minute two is a full window (zero capture, full restoration, real maintenance power,
  `maintenance_hours` = 60/3600); mass/electric residuals and `validate()` pass every step.
* `habitat::tests::authored_calibration_halves_effective_scrubber_capacity` — calibration
  0.5 halves `co2_captured_kg` against an otherwise identical rated control exactly.
* `habitat::tests::maintenance_refusals_leave_configuration_and_state_unchanged` — a
  moisture-free habitat with an electrolysis maintenance component, a negative wear rate
  and an empty maintenance config are all refused at creation.
* `habitat::tests::legacy_world_serializes_without_maintenance_state` — a 10-interval
  plain habitat has `uses_maintenance() == false`, no `maintenance` key in State JSON, no
  `maintenance_hours` key in Totals JSON and no maintenance budget (legacy byte-shape
  proxy).
* `habitat::tests::maintenance_world_is_deterministic_and_csv_tail_is_additive` — two
  identical 30-interval worlds compare `==`; the maintenance `samples.csv` header has 114
  columns ending in `total_maintenance_hours` with matching row widths.
* `habitat::integration_tests::maintenance_save_pin_is_conditional_and_enforced` — a plain
  world has no `cosmogon-cabin-maintenance` pin; a maintenance habitat's save carries
  `cosmogon-cabin-maintenance/1`, reopens with identical habitats, and a tampered or
  stripped pin is refused.

Golden fingerprint: the workspace golden test passes unchanged — `0xf2c4140a45ae509b`
(maintenance is optional and default-absent; absent worlds never enter the new code paths).

## Clippy and builds

`cargo clippy -p cosmogon_sim -p cosmogon` → zero warnings in all touched files
(`habitat_maintenance.rs`, `habitat.rs`, `save.rs`, `habitat_recording.rs`,
`habitat_diagnostics.rs`, `ui/habitat_maintenance.rs`, `ui/habitat.rs`, `ui/mod.rs`, tests).
Remaining workspace warnings are pre-existing in untouched files. Fresh release builds of
`cosmogon` and `cosmogon-cli` succeed.

## End-to-end study (case folder `/tmp/b54`)

1. `habitat-setup --seed 7 --output /tmp/b54/world` then a hand-authored **live `config`**
   maintenance block (scrubber: wear 0.36, repair 0.5, interval 0.05 h, duration 0.02 h,
   maintenance power 40 W, calibration 0.5; recycler: wear 0.05, repair 1.0, interval
   0.1 h, duration 0.03 h, 25 W, calibration 1.0) plus the explicit header pin
   `cosmogon-cabin-maintenance/1`. (The mechanism lives in the live `config` block, exactly
   like the build-53 wall/leak world; the first attempt authoring only
   `definition.config` loaded cleanly but never activated — recorded here as a trap.)
2. `habitat --initial /tmp/b54/world --samples 12 --interval-days 0.5 --output /tmp/b54/run`
   succeeds and reopens (recording reader verifies the 114-column `samples.csv` against the
   typed rows, including the maintenance tail `maintenance_operating_s`,
   `maintenance_active_s`, `worst_equipment_condition`, `total_maintenance_hours`).
3. Real wear/restore evidence across the 5.5-day study: per-sample operating/maintenance
   seconds split cycles exactly (120/0, 84/36, 60/60, 96/24 s …), the worst condition
   oscillates between 0.0 and 0.01, and cumulative maintenance grows ~6.18 h per 12 h
   sample to `68.14999999999857` h at the final world; final state holds scrubber condition
   `0.0` at cycle 180 s (wear 0.36/h outruns 0.5/h × 36 s windows) and recycler condition
   `0.9974999999999999`; the last interval budget records scrubber 60 s operating with
   `effective_scale = 0.0` (calibration 0.5 × condition 0).
4. `habitat-diagnose --initial /tmp/b54/run/final.cosmo --habitat-id 0` explains the
   mechanism (`maintenance` explanation: "scrubber condition 0.000→0.000, operating 60 s,
   maintaining 0 s, …" with evidence `budget.maintenance`,
   `budget.configuration.maintenance`, `state.maintenance`, `totals.maintenance_hours`).
5. Native: inspected capture of the Maintenance pane
   (`/tmp/b54/capture-maintenance.png`) after a release app build: cumulative 141.6600 h
   under the live 1 day/s clock, held-condition table (scrubber 0.0% at cycle 144 s,
   recycler 100.0%), and the interval table (scrubber 0.2 → 0.0% over 60.0/0.0 s at scale
   0.001 with wear 0.00160; recycler 100.0 → 100.0% over 0.0/60.0 s in window at scale
   1.000).

## Regression surface

* Legacy saves: new config/state/totals/budget fields are all `skip_serializing_if`
  absent/zero, verified by the no-keys serialization test; the conditional save pin appears
  only for habitats that use maintenance.
* Legacy recordings: `samples.csv` header lengths accept
  `38 | 49 | 66 | 85 | 110 | 112 | 114 | 115 | 116 | 117 | 119 | 121`; the maintenance tail
  is appended only when any retained row has maintenance evidence, so old studies re-derive
  byte-exactly (the crop/moisture/wall recording regression tests cover the unchanged
  historical layouts).
* Power allocation: the legacy no-maintenance path is unchanged (`fx` defaults to
  scale 1, op_fraction 1, maintenance 0); disabling maintenance freezes held state and
  returns components to rated capacity without discarding history.
* Heat: maintenance electricity is ordinary delivered bus power and enters the existing
  heat ledger; no new thermal path was added.

## Deliberate limitations retained

Wear, repair rates and capacity calibration are explicit engineering assumptions, not
measured hardware reliability or field calibration. No stochastic failures, spares,
crew labour, consumable mass, age curves beyond linear scaling, or maintenance of solar,
battery, regulator, crop or storage hardware. Numerical conservation does not establish
predictive accuracy.
