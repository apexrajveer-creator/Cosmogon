# Wall condensation and structural leak: verification evidence (build 53)

All commands run in the repository root on 2026-10-19 unless noted.

## Release tests

`cargo test --release --workspace` → **629 passed, 0 failed, 1 existing ignored**
(app 131, CLI 27, core 32, physics 38 + one doctest, simulation 400 = 629; one pre-existing
ignored app test). New simulation tests (7):

* `moisture::tests::wall_exchange_is_signed_bounded_and_domain_refused` — signed exchange
  condenses supersaturated vapor toward a cold in-domain wall (positive phase heat, rate cap),
  film headroom binds exactly (1.0e-9 kg), a warm wall with a held film evaporates (negative
  phase heat) and refuses evaporation from a dry film, and a wall temperature outside the NIST
  273.15–303 K fit yields exactly zero exchange with the retained saturation-domain
  constraint; budgets validate.
* `moisture::tests::wall_and_leak_config_validation_refuses_invalid_models_and_bounds` —
  unknown wall/leak model strings, NaN transfer coefficient and zero specific heat refused.
* `habitat::tests::passive_wall_condensation_moves_vapor_to_film_and_keeps_identities` — a
  4-day moisture world with a wall-coupled exterior (coupling 0, exterior 275.15 K) grows
  `wall_liquid_kg > 0` exactly equal to cumulative signed `totals.wall_condensed_kg`, holds
  vapor below the moisture-only control, and keeps mass/electric residuals and full
  `validate()` identities each interval.
* `habitat::tests::structural_leak_removes_proportional_gas_cools_and_keeps_accounted_mass` —
  a 60-interval leak world (1e-4 kg/s) leaks > 0 kg proportionally to retained interval
  species masses, accumulates `totals.leaked_gas` exactly equal to the `escaped_gas` ledger,
  conserves accounted mass (residual < 1e-8), and the interval cooling equals
  `leaked × cp × T` exactly.
* `habitat::tests::leak_and_breach_compose_within_one_interval` — leak (1e-4 kg/s) plus the
  default breach orifice over 120 intervals: both budgets retained every interval, escaped
  vapor ≥ leaked vapor, and full `validate()` passes (breach identities hold on the post-leak
  cabin state).
* `habitat::tests::wall_and_leak_refusals_leave_configuration_and_state_unchanged` — a held
  film above a new capacity is refused by `check_config`; tampering `totals.wall_condensed_kg`
  or zeroing `totals.leaked_gas` after an interval fails `validate()`; a plain habitat
  serializes with **no** `wall_condensation`/`wall_liquid_kg`/`wall_condensed_kg`/`leak`/
  `leaked_gas`/`"wall"` keys (legacy byte-shape proxy).
* `habitat::tests::heat_ledger_wall_and_leak_supplement_closes_span_residual` — a 240-interval
  `record_heat` world with moisture + wall + leak: the ledger supplement
  `wall_leak_storage_j` exists, span terms `wall_condensation_heat` and `leak_cooling` are
  present, and the span heat-path sum closes against storage within 1.0 J.

Golden fingerprint: the workspace golden test passes unchanged — `0xf2c4140a45ae509b`
(wall/leak are optional and default-absent; absent worlds never enter the new code paths).

## Clippy and builds

`cargo clippy -p cosmogon_sim -p cosmogon -p cosmogon_cli --all-targets` → zero warnings in
all touched files (`moisture.rs`, `habitat.rs`, `breach.rs`, `habitat_heat.rs`,
`habitat_recording.rs`, `habitat_diagnostics.rs`, `save.rs`, `ui/habitat_humidity.rs`,
`ui/habitat.rs`, tests). Remaining workspace warnings are pre-existing in untouched files.
Fresh release builds of `cosmogon` and `cosmogon-cli` succeed.

## End-to-end study (case folder `/tmp/b53`)

1. `habitat-setup --seed 7 --output /tmp/b53/world` then hand-authored config blocks
   (full `moisture::Config` fields, `wall_condensation` with coupling 0 and `leak` at
   1e-5 kg/s) plus explicit header pins for `cosmogon-habitat-budget/1`,
   `cosmogon-cabin-moisture/1`, `cosmogon-cabin-wall-condensation/1` and
   `cosmogon-cabin-structural-leak/1`.
2. `habitat --initial /tmp/b53/world --samples 8 --interval-days 0.5 --output /tmp/b53/run`
   succeeds and reopens (recording reader verifies the 117-column `samples.csv` against the
   typed rows, including the wall/leak tail columns).
3. Final state evidence: vapor 4.325101 kg, condensate 0.006382 kg; cumulative leak
   `{'oxygen_kg': 0.6999226232293277, 'nitrogen_kg': 2.3036150350585065, 'co2_kg': 5.6804366217555494e-05, 'vapor_kg': 0.02040553734594482}` kg.
   Last-interval leak budget: initial N₂ 456.8510006976438 kg → leaked 0.0004565970696… kg
   (exact start-composition share), cooling 175.89075746689286 J at 293.1512624448214 K.
4. Wall exchange is correctly **domain-refused** on Mars (exterior ≈ 210 K →
   `wall_temperature_k: 209.9994925720218`, `saturation_pa: None`, retained constraint
   "wall temperature outside the 273.15–303 K saturation fit", net 0.0) — the NIST no-ice
   domain boundary is enforced exactly; condensation on warm worlds is covered by the unit
   and world tests above.
5. `habitat-diagnose --initial /tmp/b53/run/final.cosmo --habitat-id 0` explains both
   mechanisms (`wall`/`leak` explanations; `leaked_oxygen` air term; wall/leak heat and water
   terms from `habitat_diagnostics::explain`).
6. Native: inspected capture of the humidity pane with wall/leak evidence
   (`/tmp/b53/capture-humidity.png`) after a release app build.

## Regression surface

* Legacy saves: new config/state/totals/budget fields are all `skip_serializing_if`
  absent/zero, verified by the no-keys serialization test; conditional save pins appear only
  for habitats that use wall/leak.
* Legacy recordings: `samples.csv` header lengths accept `112 | 115 | 117` in addition to
  historical `38 | 49 | 66 | 85 | 110`; the tail columns are appended only when any retained
  row has wall/leak evidence, so old studies re-derive byte-exactly (reopen verified in the
  e2e run for the new study; the crop/moisture recording regression tests cover the
  unchanged historical layouts).
* Heat ledger: the original eleven-term `terms_j` array is untouched; wall/leak heat lives in
  the optional `wall_leak_storage_j` supplement exactly like crop debits.
* Breach model: unchanged; leak applies before the breach substep and the breach retained
  identities pass every interval in the combined test.

## Deliberate limitations retained

Wall temperature, mass-transfer coefficient, film capacity and leak rate/specific heat are
explicit engineering assumptions. No ice condensation, fogging, drips, collection plumbing,
humidity-dependent leak rate, seal degradation, inward/rarefied flow or wall recycling is
modelled. Numerical conservation does not establish predictive accuracy.
