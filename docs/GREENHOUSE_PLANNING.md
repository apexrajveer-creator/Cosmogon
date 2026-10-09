# Controlled greenhouse crop planning

This source-based research instrument follows habitat checkpoint `d5d2cf6`. Open Habitat engineering → Crop planner. The same deterministic engine drives native curves, JSON/CSV exports and CLI `greenhouse`. It is a controlled-regime forecast component for phase 9, not an installed greenhouse or closed-loop resource solver. [Research and source transcription](GREENHOUSE_RESEARCH.md).

## Inputs and outputs

Five exact low-light Waldmann’s Green lettuce cases from NASA BVAD Rev2 Table 4-123 are selectable. For each, incident photon flux and CO₂ mixing ratio are pinned together. Planted area and fixture efficacy are independent user assumptions. Fixed source conditions: 23°C light/dark, 16 hours light per 24-hour terrestrial day, density 19.2 plants/m², unlimited maintained CO₂/water/nutrients. Thirty days are ages after emergence, not Mars sols.

The engine calculates the published quantum-yield polynomial and integrates the corrected canopy curve analytically. Every supported low-light case reaches its reduced maximum absorption at day 32, after harvest; it never assumes complete canopy closure. Carbon use efficiency accounts for net daily respiration once. Edible allocation begins after day 1. Curves/31 integer-day rows report cumulative/daily net carbon, CO₂ uptake, oxygen and total/edible/inedible **dry** biomass. Lamp power equals incident PPF × area / assumed fixture efficacy; fixture and optical losses are included in that assumed efficacy. Lamp energy assumes the 16-hour light period. Full facility energy, irrigation demand and nutrient requirements are unavailable. New version-2 definitions can explicitly estimate light-period transpiration under user-assumed maintained humidity at fixed assumed 101.325 kPa; see [TRANSPIRATION_PLANNING](TRANSPIRATION_PLANNING.md).

Click a graph or change the integer-age slider to inspect the actual computed row. Edible mass, oxygen and lamp energy use separate axes/units. A comparison table holds area/fixture efficacy constant across all five source cases; some change both light and CO₂, so it is not automatically a single-variable experiment. Forecast, Inputs, Compare, Water model and Files views separate plots from assumptions, comparisons and reproducibility. Water-disabled/legacy definitions show unavailable water and disable those graphs.

Habitat context shows the actual selected habitat’s current temperature, CO₂ mixing ratio and generated/served electricity. It does not replace the planner’s controlled inputs, forecast future power availability or reserve any demand. No crop yield is credited to food, O₂ or water inventories. Dry edible lettuce is not a complete diet: fresh mass, calories, micronutrients and crew-days are not inferred.

## Reproduction

```sh
./target/release/cosmogon-cli greenhouse-definition --output /tmp/crop-definition.json
./target/release/cosmogon-cli greenhouse --definition /tmp/crop-definition.json \
    --output /tmp/crop-study-1
./target/release/cosmogon --new lab --greenhouse-definition /tmp/crop-definition.json \
    --focus Mars --panel greenhouse --quiet --capture /tmp/crop.png \
    --capture-metadata --capture-after 240 --quit
./target/release/cosmogon-cli greenhouse --definition /tmp/crop-study-1/definition.json \
    --output /tmp/crop-replay-1
```

New definition JSON contains model `cosmogon-lettuce-mec/2`, typed regime, area in m², assumed efficacy in µmol/J and an optional separately versioned transpiration assumption. Version-1 definitions remain readable with identical biomass and no invented humidity or water values. Generate a template to see exact regime spelling; supported enum variants are listed in the engine source. Inputs are limited to 64 KiB, reject unknown fields/versions/regimes/nonfinite/out-of-range values. Native imports calculate before replacing existing inputs/results. Automated invalid input fails before GPU startup with nonzero exit.

`greenhouse` can use `--initial WORLD.cosmo` for exact read-only context; `--habitat-id ID` selects an observation (automatically the sole habitat if there is one). Without a context world it remains a standalone controlled forecast. Ignored scenario, seed, simulation-time or sample-count options are refused; this instrument always computes 31 rows from emergence to harvest. No initial world is manufactured implicitly.

Exports protect existing directories/files. definition.json holds prepared inputs; forecast.json includes conditions, numeric rows, coefficients, source URL/version/SHA-256, data states, null uncertainty and limits; samples.csv retains all 31 unit-labelled numeric rows and eight additional water/gas-flux fields (empty cells mean unavailable, never zero); comparison.json retains all five computed harvest cases at the same area/efficacy; manifest.json ties model/source/units/context semantics together. Optional context.cosmo is the exact current world, with FNV identity and selected habitat report in the manifest; FNV is not a cryptographic checksum. Incomplete I/O is an explicit error. Native photo sidecars record planner draft/result, selected age, input/result agreement and hidden-view semantics.

## Scientific limits and next connections

These equations describe a specific controlled cultivar/regime, not Mars open-air crops or a certified greenhouse. No arbitrary-regime interpolation is supported. The snapshot source identity does not establish empirical calibration. Numerical quadrature/carbon/scaling verification is separate from experimental crop validation; prediction uncertainty is unquantified. Biological stress, humidity dynamics/dark transpiration, nutrient chemistry, pressure/gravity effects, disease and full energy/mass closure remain absent. The optional source-based water forecast is not an empirical calibration or physical habitat exchange.

The next physical connections require finite water/nutrient/CO₂ supply, independent crop state, allocated lighting/processing power, humidity/heat exchange and actual harvest conversion. Unlimited-boundary forecasts must not be turned into automatic resource bonuses. Greenhouse installation and phase 9 remain incomplete.

A separate [finite supply study](FINITE_CROP_SUPPLIES.md) now estimates source-based material needs and lighting budget support at original daily ages. Original forecasts/schemas remain unchanged. Physical greenhouse supply/harvest/stress remains incomplete.
