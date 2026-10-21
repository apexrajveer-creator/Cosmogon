# Radiation environment and cabin shielding dose (build 57A)

Cosmogon habitats can now model an authored exterior radiation environment at the
habitat site, transmit it through authored cabin and storm-shelter shielding fractions,
and retain per-interval interior dose rates, cumulative crew dose and hazard-band
classifications as explicit evidence. The layer is optional and default-absent, so
legacy worlds, saves, recordings and the world golden stay byte-identical.

Model pin: `cosmogon-habitat-radiation/1`
(`crates/cosmogon_sim/src/habitat_radiation.rs`).

## Radiation configuration

Enabled per habitat via the optional `radiation` configuration block:

| Field | Bounds | Meaning |
| --- | --- | --- |
| `wall_transmission` | [0, 1] | fraction of the exterior dose rate transmitted through the cabin structure (1.0 transmits everything, 0.0 is fully opaque) |
| `storm_shelter_transmission` | [0, 1] | additional fraction applied to the interior rate inside the designated storm shelter |
| `alert_dose_rate_msv_day` | [0, 1e4] | sustained interior dose-rate design threshold for the habitat alert |

Defaults: `wall_transmission` 0.5, `storm_shelter_transmission` 0.25,
`alert_dose_rate_msv_day` 1.0. The model pin is fixed and validated; `Config` keeps
`deny_unknown_fields`.

Validation refuses non-finite values and out-of-bounds fields through the ordinary
`Config::validate` path, so `habitat define` authoring, native draft Apply
(`Edit::ConfigureHabitat`) and save reopen all refuse invalid radiation configuration
identically.

## How the exterior dose rate is derived

The habitat environment sample (`habitat_environment`) evaluates one closed-form
authored law per site and time from three existing world inputs:

```
exterior [mSv/day] = 1.8 × (1 + 340 × p_bar)^(-0.89) × (1 − 0.3 × B_earth) × (1 + 4 × flare_activity)
```

* `p_bar` — the body's atmospheric pressure in bar (clamped ≥ 0),
* `B_earth` — the body's magnetic field in Earth-dipole units (clamped to [0, 1]),
* `flare_activity` — the primary star's current flare activity (clamped ≥ 0).

The constants interpolate published order-of-magnitude anchors: interplanetary space
near 1 AU ≈ 1.8 mSv/day (MSL/RAD cruise measurements), Mars surface ≈ 0.67 mSv/day,
Earth surface galactic component ≈ 0.01 mSv/day. The law is **authored**, not a fit;
the anchors are documented in `source()` and the registry. A non-finite result or a
value above 1e4 mSv/day marks the environment unavailable rather than inventing a dose.

## How the dose is applied each interval

When radiation is configured **and** the environment sample supplied a finite exterior
dose rate, the completed-interval step builds `Budget.radiation`:

* `interior_dose_rate_msv_day = exterior × wall_transmission`
* `storm_shelter_dose_rate_msv_day = interior × storm_shelter_transmission`
* `interval_dose_msv = interior × dt / 86400` (interior rate held constant over the interval)
* `totals.cumulative_dose_msv += interval_dose_msv`

The interval budget is retained with the ordinary budget and replay-revalidated
(`Budget::validate` re-derives every product, refuses tampering and domain breaks).
Cumulative dose lives in `Totals` (skip-if-zero: habitats that never recorded a dose
serialize byte-identically) and is bounded in [0, 1e9] mSv, at least the last completed
interval transfer.

## Hazard bands

The sustained interior dose rate is classified into authored design bands:

| Band | Interior rate | Context |
| --- | --- | --- |
| `nominal` | ≤ 50/365.25 ≈ 0.137 mSv/day | ICRP single-year occupational reference scaled to one day |
| `elevated` | ≤ 1.0 mSv/day | ISS-class sustained operational dose rate |
| `above_design` | > 1.0 mSv/day | above the sustained operational design envelope |
| `unavailable` | no interval dose | environment or interval missing; no dose claim is made |

Bands are modelled classifications, not safety determinations or medical guidance.

## Alert

The habitat alert list gains an eighth, appended condition (`RADIATION_ALERT`, bit 7 —
existing bit positions never shift): the configured radiation model is present, an
interval budget exists, and the interior dose rate exceeds `alert_dose_rate_msv_day`.
The matching Chronicle event carries a radiation-specific detail line (interior rate,
band, cumulative dose, threshold) and states the authored-model disclaimer.

## Evidence surfaces

* **Budget**: `budget.radiation` with the four values above plus the hazard band.
* **Totals**: `totals.cumulative_dose_msv` (skip-if-zero).
* **Report**: `radiation_observation` (all values, band label, model pin),
  `radiation_source` (law, anchors, bands, explicit limits), `source.radiation` and
  `data_states.radiation` (ESTIMATED design inputs, not measured dose); the `limits`
  text now describes radiation as optional and separately sourced.
* **CSV**: when any recorded row carries radiation evidence, `samples.csv` gains six
  trailing columns — `radiation_exterior_dose_rate_msv_day`,
  `radiation_interior_dose_rate_msv_day`, `radiation_shelter_dose_rate_msv_day`,
  `radiation_interval_dose_msv`, `radiation_cumulative_dose_msv`,
  `radiation_hazard_band`. Rows without a budget write empty cells (empty means
  unavailable, never zero); the band column always carries its label.
* **Diagnostics**: a `radiation` term group (five terms) and a `radiation` explanation;
  the group is serde-skipped while empty so old diagnostic exports keep their exact
  shape. `source.radiation` is added to the diagnostic report when the model is used.
* **Recordings**: conditional `radiation_source` / `radiation_model_descriptor` manifest
  keys, replay-validated against the live source and registry descriptor; a study that
  never used radiation must not declare them.
* **Save**: conditional `data_versions["cosmogon-habitat-radiation"]` pin, enforced on
  reopen (`SaveError::Model`) exactly like every other model pin.
* **Registry**: `msl-rad` and `icrp-dose-limits` scientific sources plus the
  `habitat-radiation` model descriptor.
* **Native UI**: a Habitat **Radiation shielding** group in the habitat configuration
  (right after life-support calibration) staging the checkbox, the two transmissions
  and the alert threshold through the ordinary draft Apply path, with the pinned model
  id and non-claims inline.

## Explicit limits

No solar-particle-event forecast, particle transport, secondary production, activation,
hull geometry/depth, material attenuation data, dust or altitude dependence,
time-varying shielding, individual dosimetry or medical/ALARA compliance claim. The
exterior law is an authored anchor-interpolating reduction; the shielding fractions are
authored constants, not measured attenuation; the band thresholds are authored design
categories informed by ICRP dose-limit context, not safety certification. Uncertainty
is unquantified. See `docs/RADIATION_SHIELDING_DESIGN.md` for the full mechanism and
`docs/RADIATION_SHIELDING_VALIDATION.md` for the verification evidence.
