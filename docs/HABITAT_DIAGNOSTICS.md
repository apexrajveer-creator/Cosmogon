# Habitat WHY: completed-interval explanations

Open Habitat engineering → WHY, or choose **WHY this result** from Budgets. Power, Water, Heat and Air & crew trace actual recorded quantities; Evidence & export exposes the underlying source snapshot and protected files. Recordings → WHY selected interval explains any exact saved observation independently of the live world. Selecting another observation does not advance or rewind the simulation. Return to Curves & comparison for time/interval-supported A/B differences.

This is a completed-interval accounting explanation. It does not claim a universal historical cause graph, a unique equipment fault or real-world predictive accuracy. Every term carries a unit and an evidence/formula string. Hover numeric rows to inspect it; the complete JSON/CSV retains it. Unquantified uncertainty is explicit. Missing evidence is unavailable (JSON null / blank CSV), never assumed zero.

The interval ends at the retained habitat time, which may precede the displayed world clock. Current and staged controls do not rewrite the configuration stored with that interval. Waiting, unavailable environment, unsupported temperature domain and missing interval configuration suppress decomposition. A configured reactor with an absent budget leaves its watts/flow/heat and dependent aggregate sums unavailable while retaining other supported terms. Disabling a reactor in the completed configuration establishes an actual zero contribution.

Power explains firm/PV supply, battery bus discharge and charging before losses, conversion losses, unused supply, total requested/served/unserved demand and stored-energy change. Served base/air/water watts follow the engine's actual priority order, then thermal equipment and optional reactors. Separate heater/pump requests were not retained, so their demand is a reconstructed aggregate. Tiny nonzero aggregate remainders can be floating-point subtraction roundoff; the raw derived value is retained rather than represented as measured equipment operation. Reactor allocation and constraint lists are recorded; constraints may overlap. Powered idle equipment does not prove material processing or an equipment fault.

Water separates reference crew potable/food-water use, wastewater generation, electrolysis draw, wastewater versus held raw-product processing, potable recovery and brine. New raw reactor product waits until a later interval's recycling. Recovery is an assumed mass split, not measured water quality. Air & crew shows regulator/capture transfers, actual oxygen/CO₂ store changes, oxygen/hydrogen/methane/captured-gas changes and nominal food/waste flows. Newly generated oxygen waits until a later regulator interval. End stocks do not reveal the unique start stock that limited a crew cycle. No health/mortality outcome is inferred.

Heat lists signed bus electricity, battery/metabolic heat, electrolysis reference enthalpy debit, Sabatier reference heat, radiator rejection, lumped conduction and actual stored heat. **Do not add the heater twice:** its watts are already included in delivered bus electricity. Electrical/thermal/whole-mass residuals check model accounting, not physical calibration. Zero sampled horizontal irradiance cannot establish local dust/weather or a specific night without those inputs.

The shared calculations live in `cosmogon_sim::habitat_diagnostics`; native views, ordinary habitat reports, equipment sonification and CLI use them. Simulation trajectories, optional save fields, source versions and golden `0xf2c4140a45ae509b` are unchanged. Older study reports without the additive diagnostics key still load; a present key must match the exact shared completed result. Existing CSV study columns are unchanged.

```sh
./target/release/cosmogon-cli habitat-diagnose --initial /path/to/state.cosmo \
  --habitat-id 0 --output /tmp/new-habitat-why
./target/release/cosmogon --load /path/to/state.cosmo --focus Mars --speed 0 \
  --panel habitat-why-heat --capture /tmp/why-heat.png --capture-metadata \
  --capture-after 240 --quit
./target/release/cosmogon --new lab --habitat-recording /path/to/study \
  --habitat-sample 1 --panel habitat-recording-why-water --speed 0 \
  --capture /tmp/recorded-why.png --capture-metadata --capture-after 240 --quit
```

`habitat-diagnose` requires an explicit saved world and habitat ID, refuses unrelated options and does not advance. Omit output to print JSON. A new export folder contains `report.json` (exact habitat/configuration/interval/source/diagnostics), `terms.csv` (units/formulas, blanks for unavailable quantities) and `manifest.json`, written last as the completion marker. Existing folders are protected; an interrupted new folder may remain incomplete. This derived snapshot is not a full replay bundle; use ordinary saves and habitat studies for replay. Native export captures the chosen interval and writes off the main thread; the live clock can continue.

Developer live panels: `habitat-why`, `habitat-why-water`, `habitat-why-heat`, `habitat-why-air`, `habitat-why-evidence`. Recorded panels use `habitat-recording-why` with the same suffixes. PNG sidecars retain selected live/recorded scope and exact shared reports. Captures cannot establish manual interaction or acoustic-device behavior.
