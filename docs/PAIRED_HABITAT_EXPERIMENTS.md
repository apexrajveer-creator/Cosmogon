# Paired habitat experiments

Habitat engineering → Recordings → Run experiment can pair two settings with exactly two distinct levels each. Select the first setting, enter its two values, enable **Pair with a second setting**, choose a distinct second setting and two levels in its displayed unit. Four independent clones start from one exact common world; both settings change through one atomic ordinary edit per branch. No inventories are filled, discarded or reinterpreted. All four edits validate before output; old-configuration remainders finish before the changes.

The standard order is A0B0, A1B0, A0B1, A1B1. First/second supplied levels define 0/1, even for descending values. Native **Paired results** shows four actual final responses for the chosen metric, each conditional change, average main changes and the difference of conditional A changes. The standard AB effect is half that difference. These retain the metric's unit; RH changes are percentage-point differences. Missing values or unsupported interval alignment make contrasts unavailable. Results always identify the final stored time/index, independently of a selected graph observation. A one-observation snapshot can retain pre-edit flow settings; the result flags each interval/configuration match and warns before interpreting equipment changes.

The first/last trajectory graph changes both settings and cannot isolate either effect. Use the four-response table for conditional comparisons; all four complete source-pinned trajectories remain available for ordinary reopening/WHY or independent pair comparison. Physical hardware response, predictive accuracy, significance, p-values, confidence intervals and a global optimum are not established. Deterministic repetition checks implementation, not independent measurement noise. See [NIST factorial design methods](https://www.itl.nist.gov/div898/handbook/pri/section3/pri3331.htm) and the [preimplementation design](PAIRED_HABITAT_DESIGN.md).

## Protected files and reopening

Saved paired plans use schema 2 (single-setting plans retain schema 1):

```json
{
  "format": "cosmogon-habitat-sweep-plan",
  "schema_version": 2,
  "parameter": "moisture_power_w",
  "values": [0, 500],
  "secondary": {"parameter": "moisture_airborne_fraction", "values": [0.2, 1]},
  "samples": 25,
  "interval_s": 3600
}
```

```sh
./target/release/cosmogon-cli habitat-sweep --initial /tmp/moisture.cosmo \
  --definition /tmp/paired-plan.json --output /tmp/new-paired-study
./target/release/cosmogon-cli habitat-inspect --recording /tmp/new-paired-study
./target/release/cosmogon --habitat-recording /tmp/new-paired-study \
  --habitat-metric humidity --panel habitat-paired
```

Native recording-folder opening accepts a completed paired study root as well as ordinary branch folders. Root reopening validates bounded plan/metadata, the common saved world/fingerprint/source versions, all four ordinary-edit starting worlds, every branch recording, final metrics and recomputed analysis. This is structural numerical consistency, not cryptographic authenticity or independent trajectory replay. Malformed/incomplete/future/tampered results preserve previous native views. CLI inspection returns the same actual analysis.

Protected output includes the original common world, paired plan, all four ordinary recording bundles, `factorial.json` with source/formulas/limits and the completion manifest written last. No overwrite. Metadata is bounded to regular files within 1 MiB; plan bounds remain 64 KiB and 1–1024 samples within 30 days. The immutable master, simulation/save schema and golden remain unchanged. No statistical noise model, center points/curvature, higher-factor designs, uncertainty propagation, calibration or physical greenhouse exchange.
