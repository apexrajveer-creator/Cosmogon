# Recorded actual crop trajectories

Habitat engineering → Recordings exposes actual installed crop values in Curves & comparison, paired curves/tables, reusable graph layouts and Research figure. These selections read retained model state, independently of the live world. Crop planner forecasts remain separate and never stand in for actual stocks.

| Selection | Quantity | Unit |
|---|---|---|
| `crop_progress` | Supported reference production progress, not biological age | days |
| `crop_nutrient` | Retained original nutrient-equivalent stock | kg |
| `crop_standing_edible`, `crop_standing_inedible` | Standing dry stores | kg |
| `crop_harvested_edible`, `crop_harvested_inedible` | Separately harvested dry stores | kg |
| `crop_co2`, `crop_metabolic_water` | Actual cumulative model feeds consumed | kg |
| `crop_vapor`, `crop_oxygen` | Actual cumulative vapor and conservative surrogate oxygen generated | kg |
| `crop_light`, `crop_auxiliary` | Actual cumulative allocated light/auxiliary electricity | J |
| `crop_chemical` | Retained dry chemical storage under the original assumed energy basis | J |
| `heat_crop_chemical`, `heat_crop_vapor` | Retained signed storage debits in an enabled cabin heat ledger | J |

An installed cycle has real initial values: nutrient stock and zero dry production/electricity. A cycle without acknowledged maintained conditions can remain at zero production while chronological time passes. Disable retains material and counters. Absent legacy crop state remains unavailable; no zero is manufactured from a missing process. Original cumulative counters continue across study restarts; they are not sums of sparse sampled minute budgets. Harvest transfers standing to harvested stores without changing total dry chemical storage or feeding nominal crew diet.

Native scope labels distinguish supported progress from elapsed installation time and disclose the assumed chamber. Full figure/comparison JSON retains every original source curve, configuration, chemical-energy basis, original nutrient stock, installation/completed clocks, harvest state and actual completed crop budget. SVG carries model scope in its description/metadata; original ten-column figure CSV retains all numerical values at 17-digit precision. Connecting lines are guides between samples, not reconstructed biological or operating history. Positive log explicitly omits zero/nonpositive values; signed log can display negative storage debits.

Comparisons and four-branch contrasts require exact stored timestamps plus aligned crop installation origins and completed habitat ends. Different origins, ends or absent evidence produce explicit support reasons and empty differences/effects while retaining available responses. Elapsed alignment may align different epochs; it does not establish physical equivalence. Different original source/chemical/chamber assumptions remain declared engineering scenarios, not calibrated biological predictions. The two heat debit selections additionally require the existing complete heat coverage/origin support. Harvest is a material action, not sudden biological production; retained harvest flags accompany samples.

Reopening also refuses a changing installation origin, reversed cumulative crop counters, disappearing crop state or reversed harvest inside a trajectory. Per-row source/material validation remains separate from these continuity checks; neither establishes cryptographic authenticity or independent replay.

Existing recorded summaries, sweep/factorial metric catalogs and aggregate trajectory exports stay unchanged. Select a new crop metric explicitly and export a research figure to retain its original responses and selected paired effect. No hidden rewrite of previous analyses.

A complete reusable layout:

```json
{"format":"cosmogon-habitat-graph-layout","schema_version":1,"metric":"crop_progress","effect":"average_a","alignment":"world","window":[0,1],"scale":"linear","signed_threshold":1,"include_zero":true,"time_unit":"days"}
```

```bash
./target/release/cosmogon-cli habitat-compare --recording /tmp/finite-crop-study \
  --reference /tmp/limited-crop-study --measure crop_progress --alignment world \
  --output /tmp/new-crop-comparison
./target/release/cosmogon-cli habitat-figure --recording /tmp/finite-crop-study \
  --definition /tmp/crop-layout.json --output /tmp/new-crop-figure
./dist/Cosmogon.app/Contents/MacOS/cosmogon --panel habitat-figure \
  --habitat-recording /tmp/finite-crop-study --habitat-graph /tmp/crop-layout.json \
  --habitat-figure-output /tmp/new-native-crop-figure \
  --capture /tmp/crop-figure.png --capture-metadata --capture-after 240 --quiet --quit
```

`--recording` also accepts an existing four-branch paired design. `effect` selects the retained conditional-average A/B change or interaction; it does not fit a statistical model or supply uncertainty. Existing destinations are protected. Figure exports write their complete manifest last; habitat studies now write the optional crop CSV before their manifest as well. This is a completion marker, not an atomic replacement of a whole directory.

[Finite process and primary sources](FINITE_GREENHOUSE.md), [preimplementation design](RECORDED_CROP_TRAJECTORIES_DESIGN.md), [heat scope](HEAT_TRAJECTORIES.md). No new solver/state/save/dependency/graphics change; golden remains `0xf2c4140a45ae509b`. Crop physiology, chamber control, survival, repeated cycles and diet coupling remain incomplete.
