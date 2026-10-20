# Life-support calibration and sensitivity envelopes — design (build 56)

Closes the tracker gap "life-support validation beyond reference assumptions
(calibration/uncertainty)" with an optional, explicit calibration layer over the four
published reference life-support assumptions plus deterministic per-interval sensitivity
envelopes retained as bounded evidence. Model: `cosmogon-life-support-calibration/1`.
Optional and default-absent; legacy worlds, saves, recordings and the golden stay
byte-identical. Every new number is an authored design input, classified as such in the
report data states (master directive section 9).

## A. Calibration configuration

Optional `habitat::Config.calibration` (`habitat_calibration::Config`): the model pin plus
four `Factor { central, relative_band }` entries, one per reference assumption:

* `crew_demand` — the nominal crew exchange cycle (NASA BVAD Table 3-31 draws/waste).
* `scrubber` — the authored CO₂ capture rate.
* `recycler` — the authored water processing rate (wastewater, raw product, condensate).
* `oxygen_makeup` — the regulator's stored-oxygen makeup rate.

Per factor:

* `central` ∈ [0.1, 10] — calibration multiplier applied to the reference rate
  (1.0 reproduces the published/nominal assumption exactly).
* `relative_band` ∈ [0, 1] — authored ± half-width for the retained envelope
  (0 keeps low = high = applied).

Validation refuses non-finite or out-of-bounds values and a wrong model pin through the
existing `Config::validate` path, so both `habitat define` and `Edit::ConfigureHabitat`
refuse invalid calibration identically.

## B. Application in the completed-interval step

The calibrated value multiplies the reference expression in place:

* **Crew**: `demand = crew · crew_demand.central · dt / SECONDS_PER_DAY` before the
  existing stock-bounded cycle; every cycle-proportional draw, waste, vapour, CO₂ and
  metabolic heat follows the bounded cycle exactly as today.
* **Oxygen makeup**: the existing clamp bound becomes
  `oxygen_makeup_rate_kg_s · oxygen_makeup.central · dt`.
* **Scrubber**: the capture expression is multiplied by `scrubber.central`.
* **Recycler**: all three stream rate expressions (wastewater, raw product, condensate)
  are multiplied by `recycler.central`.

Central multiplies with — never replaces — equipment condition, power service fractions
and the build-54 maintenance capacity calibration; the authored layers compose. With no
calibration configured, or all factors at identity with zero band, physics is unchanged.

## C. Retained envelopes (budget evidence)

`Budget.calibration: Option<CalibrationBudget>` with four `Bound { applied, low, high }`:

* `crew_cycle_person_days` — the applied bounded cycle (person-days).
* `oxygen_makeup_kg` — oxygen transferred from reserve into the cabin this interval.
* `co2_captured_kg` — CO₂ captured into the finite store this interval.
* `water_processed_kg` — total recycled water this interval (wastewater + raw product +
  condensate streams).

`low = max(0, applied · (1 − relative_band))`, `high = applied · (1 + relative_band)`
using the matching domain's band. These are **authored sensitivity bounds around the
applied calibrated exchange** — not a re-simulation under perturbed parameters, not
measured variance, not a confidence interval. Open-time budget validation recomputes the
band arithmetic from the configuration captured with the budget and refuses tampering.

## Surfaces

* Conditional `cosmogon-life-support-calibration` save pin whenever the configuration is
  present; new fields skip-if-absent.
* `samples.csv` gains twelve calibration tail columns (applied/low/high for the four
  bounds) present only when any row has calibration evidence; accepted widths extend by
  one band.
* WHY/`habitat-diagnose` gains a "calibration" explanation (factors plus the last
  interval's applied values and band widths); the report gains a calibration observation,
  source entry and `source.calibration` diagnostics entry, and — only when calibration is
  configured — a `data_states.life_support_calibration` entry. The existing
  `uncertainty_scope` string is untouched when absent; when present it states that bands
  are authored sensitivity bounds around calibrated rates, not measured variance or
  confidence intervals.
* Native: a "Life-support calibration" group in the shared habitat config editor (enable
  toggle, eight number fields) flowing through the existing draft-config
  `Edit::ConfigureHabitat` Apply path — no new Edit variant.
* CLI: no new commands; calibration is authored through `habitat define` configuration
  JSON (and the native editor), documented in the guide.

## Deliberate limitations

Calibration factors and bands are authored design inputs (ESTIMATED), not fitted to
measurements: no parameter identification, no observed-data validation, no probabilistic
propagation or Monte Carlo, no hardware reliability model. Envelopes bound the applied
interval exchange around its calibrated value and do not re-run physics, so they express
assumption sensitivity only. Numerical bounds do not establish predictive accuracy or
safety certification.
