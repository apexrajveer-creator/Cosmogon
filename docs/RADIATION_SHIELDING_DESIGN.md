# Radiation environment and cabin shielding dose — design (build 57A)

Closes half of the tracker gap "environmental hazard completeness (breach/radiation
propagation)" with an optional, explicit exterior-radiation and cabin-shielding dose
layer. Model: `cosmogon-habitat-radiation/1`. Optional and default-absent; legacy
worlds, saves, recordings and the golden stay byte-identical. Every new number is an
authored design input, classified as such in the report data states.

## A. Module layout

`crates/cosmogon_sim/src/habitat_radiation.rs` owns the whole model: the exterior law,
`Config`, `Budget`, `HazardBand` and `source()`. It depends only on
`crate::time::SECONDS_PER_DAY` and serde, keeping the physics-free bookkeeping layer
separable from `habitat.rs` wiring (same split as `habitat_calibration.rs`).

## B. Exterior law — authored anchor interpolation

```
exterior = 1.8 × (1 + 340 p)^{-0.89} × (1 − 0.3 B) × (1 + 4 f)
```

* `REFERENCE_DOSE_MSV_DAY = 1.8` — MSL/RAD cruise free-space anchor (Hassler 2014;
  Zeitlin 2013 en-route measurements cited in `source()` and the `msl-rad` registry
  source).
* `(1 + 340 p)^{-0.89}` — atmospheric overburden attenuation. 340/0.89 were **chosen so
  the curve passes through the published anchors**, not fitted: p = 0.006 bar (Mars)
  gives ≈ 0.669 mSv/day against the ~0.67 anchor; p = 1.01325 bar (Earth) gives
  ≈ 0.0069 before the magnetosphere term, consistent with the ~0.01 order-of-magnitude
  galactic component at sea level (the Earth anchor is a GCR-component order-of-magnitude
  figure, not a total-including-SPE value).
* `(1 − 0.3 B)` — authored dipole shielding, `MAGNETOSPHERE_REMOVAL = 0.3` (at most
  30 % removal at B = 1). Mars (B ≈ 0) is unshielded in this term.
* `(1 + 4 f)` — authored stellar-flare gain, `FLARE_GAIN = 4.0` at maximum flare
  activity. The existing `Star::flare_activity_at` scalar is reused; no SPE forecasting
  exists anywhere.

Inputs are clamped into their physical domains (p ≥ 0, B ∈ [0,1], f ≥ 0) before
evaluation; a non-finite result or a value > 1e4 mSv/day makes the environment
unavailable (the habitat falls back to `Environment::default()` and the budget is
absent) — unavailable is never recorded as zero dose.

## C. Shielding and interval application

`Config` (`deny_unknown_fields`): `model`, `wall_transmission` ∈ [0,1],
`storm_shelter_transmission` ∈ [0,1], `alert_dose_rate_msv_day` ∈ [0,1e4]. Defaults
0.5 / 0.25 / 1.0 — deliberate design midpoints, not measured values.

Per completed interval (after the existing thermal/air/water/maintenance/construction/
breach ordering, so dose bookkeeping never feeds back into any other process):

* `Budget::new(config, exterior, dt)` derives interior = exterior × wall, shelter =
  interior × shelter, `interval_dose_msv = interior × dt / 86400`.
* `totals.cumulative_dose_msv += interval_dose_msv` (skip-if-zero serialization).
* The budget joins `Budget.radiation: Option<...>` beside `calibration`/`breach`/etc.

`Budget::validate(&config, dt)` re-derives every product to 1e-9 relative tolerance,
refuses rate/interval domain breaks and a mismatched hazard band; habitat `validate`
refuses a budget without its configuration, cumulative outside [0, 1e9] mSv, and
cumulative below the last completed interval transfer.

## D. Cumulative dose in Totals

The cumulative crew dose is a running total across intervals, independent of any single
budget's configuration — exactly like `totals.leaked_gas` / `totals.maintenance_hours`.
It therefore lives in `Totals` (serde skip-if-zero), not `State` and not `Budget`; the
interval dose stays in the budget (configuration-bound evidence) and the cumulative
value is quoted by the budget validate, report, diagnostics, CSV and alert event.

## E. Hazard bands and alert

`HazardBand::of(Option<f64>)` classifies the sustained interior rate against the two
published-context boundaries (`ELEVATED_DOSE_RATE_MSV_DAY = 50/365.25`, the ICRP
single-year occupational reference scaled to one day; `ABOVE_DESIGN_DOSE_RATE_MSV_DAY =
1.0`, a sustained ISS-class operational envelope). `label()` produces the stable
snake_case token (`above_design`) used identically by CSV, JSON report observations and
event text.

The habitat alert list grows from seven to eight entries; the new bit is **appended**
(`RADIATION_ALERT = 7`) so every existing bit position keeps its meaning. The alert
fires when the configured model produced an interior rate above the authored threshold.
Only the new event carries the radiation-specific detail string; the seven existing
event details are untouched.

## F. Evidence, pins and replay

* Report: `radiation_observation`, `radiation_source`, `source.radiation`,
  `data_states.radiation`; the `limits` strings (base and crop variants) now describe
  radiation as optional and separately sourced and drop "radiation" from the
  model-absence list.
* `samples_csv`: a `uses_radiation` flag over the rows decides six conditional tail
  columns; rows without a budget emit empty cells (empty = unavailable), and the band
  label is always written. Existing accepted-width consumers (the header/row equality
  tests, the breach sweep's fixed 110-column CSV and the factorial's 40-column CSV)
  are unaffected because those recordings never carry radiation evidence.
* `habitat_recording`: conditional `radiation_source` / `radiation_model_descriptor`
  manifest keys accumulated per recording like crop; replay validation compares them
  against the live `source()` and registry descriptor and refuses both mismatches and
  declarations on radiation-free studies (old manifests lack the keys and read as null,
  which passes).
* `habitat_diagnostics`: `Diagnostics.radiation: Vec<Term>` with
  `#[serde(skip_serializing_if = "Vec::is_empty")]` so pre-build-57 diagnostic exports
  (which have no such key) keep their exact serialized shape; five terms plus one
  explanation appear only under `uses_radiation()`. The export CSV gains the
  `radiation` group automatically through the shared terms loop.
* `save`: `data_versions["cosmogon-habitat-radiation"]` only when some habitat uses the
  model; `from_json` enforces the pin with `SaveError::Model` beside the calibration
  pin.
* `research/registry.rs`: `msl-rad` (Hassler 2014 DOI + Zeitlin 2013) and
  `icrp-dose-limits` (ICRP Publication 103) scientific sources and the
  `habitat-radiation` model descriptor with real validation-case names; the base
  `habitat` descriptor's limitations drop "radiation" from the absence list.

## G. Native UI

`ui/habitat_radiation.rs` mirrors `ui/habitat_calibration.rs`: a checkbox
("Enable radiation shielding model") that instantiates the default config, then a
collapsible group staging the two transmissions and the alert threshold through the
same `super::habitat::number` widgets and the ordinary draft Apply path. It is mounted
in the habitat configuration form directly after the life-support calibration group.
The model id and the non-claims (constant fractions, not measured attenuation; bands
are design categories, not dosimetry) are rendered inline.

## H. Deliberate non-changes

* No new `Edit` variant, no new CLI command, no new CSV in the recording bundle
  (dose rides the ordinary `samples.csv` tail).
* Breach (`cosmogon-cabin-breach/1`) is untouched in this half; inward/reverse flow is
  the separate 57B half.
* The golden fingerprint is expected to be unchanged: radiation is default-absent, the
  `Totals` field is skip-if-zero, and `Environment.exterior_dose_rate_msv_day` only
  appears in derived samples, never in serialized state. Confirmed by the suite.
* Crew physiology, EVA suits, dust, altitude and medical outcomes remain outside the
  model entirely — dose is a whole-cabin engineering estimate, nothing more.
