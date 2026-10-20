# Life-support calibration and sensitivity envelopes (build 56)

Cosmogon habitats can now apply authored calibration multipliers to the four published
reference life-support assumptions and retain per-interval sensitivity envelopes as bounded
evidence. The layer is optional and default-absent, so legacy worlds, saves, recordings and
the world golden stay byte-identical.

Model pin: `cosmogon-life-support-calibration/1`
(`crates/cosmogon_sim/src/habitat_calibration.rs`).

## Calibration configuration

Enabled per habitat via the optional `calibration` configuration block with four
`Factor { central, relative_band }` entries:

| Factor | Calibrates |
| --- | --- |
| `crew_demand` | the nominal crew exchange cycle (NASA BVAD Table 3-31 draws/waste) |
| `scrubber` | the authored CO₂ capture rate |
| `recycler` | the authored water processing rate (wastewater, raw product, condensate) |
| `oxygen_makeup` | the regulator's stored-oxygen makeup rate |

Per factor:

* `central` ∈ [0.1, 10] — calibration multiplier applied to the reference expression
  (1.0 reproduces the published/nominal assumption exactly).
* `relative_band` ∈ [0, 1] — authored ± half-width for the retained envelope
  (0 keeps low = high = applied).

Validation refuses non-finite values, out-of-bounds factors/bands and a wrong model pin
through the ordinary `Config::validate` path, so `habitat define` authoring, native draft
Apply (`Edit::ConfigureHabitat`) and save reopen all refuse invalid calibration identically.

## How calibration is applied

Each completed interval the calibrated multiplier is applied in place:

* **Crew**: `demand = crew · crew_demand.central · dt / SECONDS_PER_DAY` before the
  existing stock-bounded cycle; every cycle-proportional draw (O₂, potable water, dry food,
  food water), waste, vapour, CO₂ and the 12.426 MJ/person-day metabolic heat follow the
  bounded cycle exactly as today. Crew-water reservations used by the electrolysis planner
  and the crop stock check scale with the same factor.
* **Oxygen makeup**: the regulator clamp bound becomes
  `oxygen_makeup_rate_kg_s · oxygen_makeup.central · dt`.
* **Scrubber**: the capture expression is multiplied by `scrubber.central`.
* **Recycler**: all three stream rate expressions (wastewater, raw product, condensate)
  are multiplied by `recycler.central`.

Central multiplies with — never replaces — equipment condition, power service fractions
and the build-54 maintenance capacity calibration; the authored layers compose.

## Retained envelopes

`budget.calibration` retains four `Bound { applied, low, high }` values:
`crew_cycle_person_days`, `oxygen_makeup_kg`, `co2_captured_kg` and `water_processed_kg`
(wastewater + raw product + condensate). `low = max(0, applied · (1 − relative_band))`,
`high = applied · (1 + relative_band)` from the matching factor. These are **authored
sensitivity bounds around the applied calibrated exchange** — not a re-simulation under
perturbed parameters, not measured variance and not a confidence interval. Open-time
validation recomputes the band arithmetic from the configuration captured with the budget
and refuses tampering.

## Surfaces

* Conditional `cosmogon-life-support-calibration` save pin whenever the configuration is
  present; the new fields skip-if-absent, so old worlds re-serialize byte-identically.
* `samples.csv` gains twelve calibration tail columns (applied/low/high for the four
  bounds), present only when calibration evidence exists.
* WHY/`habitat-diagnose` gains a "calibration" explanation (factors plus the last
  interval's applied values and band widths); the report gains a calibration observation,
  source entry and `source.calibration` diagnostics entry, a
  `data_states.life_support_calibration` entry and — only when calibration is configured —
  an `uncertainty_scope` stating the bands are authored sensitivity bounds, not measured
  variance or confidence intervals.
* Native: a "Life-support calibration" group in the shared habitat **Controls** config
  editor and in the **Settlement → Settings** pane (enable toggle, eight number fields),
  flowing through the existing draft-config Apply path — no new Edit variant.
* CLI: no new commands; calibration is authored in the `habitat define` configuration JSON
  (see below) or through the native editor.

## Authoring example

```json
"calibration": {
  "model": "cosmogon-life-support-calibration/1",
  "crew_demand":  { "central": 1.1, "relative_band": 0.1 },
  "scrubber":     { "central": 0.9, "relative_band": 0.2 },
  "recycler":     { "central": 1.0, "relative_band": 0.15 },
  "oxygen_makeup":{ "central": 1.0, "relative_band": 0.0 }
}
```

## Deliberate limitations

Calibration factors and bands are authored design inputs (ESTIMATED data state), not fitted
to measurements: no parameter identification, no observed-data validation, no probabilistic
propagation or Monte Carlo, no hardware reliability model. Envelopes bound the applied
interval exchange around its calibrated value and do not re-run physics, so they express
assumption sensitivity only. Numerical bounds do not establish predictive accuracy or
safety certification.
