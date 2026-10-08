# Controlled lettuce water forecast

Habitat engineering → Crop planner shares the same deterministic calculation with CLI `greenhouse`. Forecast plots and age inspection now include optional cumulative liquid-equivalent liters, daily liters, vapor mass, gross/light-cycle net carbon and leaf/canopy conductance. This is a controlled planning instrument for phase 9/16. [Primary-source research and domain decisions](TRANSPIRATION_RESEARCH.md).

## Explicit assumptions and compatibility

New templates use `cosmogon-lettuce-mec/2` and optional `transpiration: {"model":"cosmogon-lettuce-transpiration-mec/1","relative_humidity":0.7}`. Humidity is a user assumption expressed as a fraction. Reference air pressure is fixed at an assumed 101.325 kPa; source leaf/light temperature remains 23°C, with 16 hours light per terrestrial day. The five pinned light/CO₂ regimes and all biomass calculations are unchanged. Humidity affects the water equation only; it does not implement crop stress or change biomass.

Old `cosmogon-lettuce-mec/1` definitions reopen with bit-identical biomass and unavailable water. No default humidity is silently added, and no imported file is overwritten. Explicitly enabling transpiration in Inputs changes the runtime definition to version 2; exporting writes a new protected study. Version 1 with a water extension, future crop/water versions, unknown fields and arbitrary pressure/temperature inputs are refused. Version 2 can disable water by omitting/nulling the optional assumption.

Humidities must be finite fractions from zero to one and give nonnegative conductance in the published planophile equation. At 23°C sufficiently dry air makes its coefficient negative; this unsupported domain is refused rather than clipped to a plausible-looking answer. This is a mathematical guard, not empirical validation. Saturated-air zero deficit gives mathematical zero light-period flux; it is not an operating recommendation. Other physiological, condensation and disease effects are unmodeled.

## Quantities, methods and product views

The engine calculates saturation/air vapor pressure and deficit in kPa, source gross and light-cycle net photosynthesis in µmol carbon/(m² s), stomatal and series canopy conductance in mol water/(m² s), and daily liquid-equivalent water flux. The light-cycle net factor accounts for the source equation's respiration convention; it differs from daily net carbon. The water equation covers the light period only. Multiplying liters by source liquid density yields equivalent vapor mass; it does not describe liquid collected by hardware.

Age-dependent canopy conductance is nonlinear, so water is integrated with deterministic composite Simpson quadrature at 256 subdivisions per terrestrial day. Exports show the absolute harvest difference from 512 subdivisions/day. This diagnostic is numerical evidence, not a certified error bound or biological uncertainty. Biomass continues to use its exact analytic canopy antiderivative. Both curves connect 31 calculated integer-age observations through day 30, not measured crop samples.

Forecast offers separate unit-labelled total/daily water graphs and selected-age values. Inputs changes humidity/area/fixture assumptions explicitly. Compare holds those assumptions fixed across five published regimes; some vary both light and CO₂, so it is not a single-variable causal experiment. Water model shows vapor pressures, gross/light-cycle carbon, conductance, constants and numerical evidence. Files keeps source-pinned import and protected export accessible. Invalid imports preserve prepared inputs/results.

```sh
./target/release/cosmogon-cli greenhouse-definition --output /tmp/crop-water.json
./target/release/cosmogon-cli greenhouse --definition /tmp/crop-water.json \
    --output /tmp/crop-water-study
./target/release/cosmogon --new lab --focus Mars --greenhouse-definition /tmp/crop-water.json \
    --panel greenhouse-water --quiet --capture /tmp/crop-water.png \
    --capture-metadata --capture-after 240 --quit
```

`greenhouse-daily-water`, `greenhouse-inputs`, `greenhouse-compare`, `greenhouse-model` and `greenhouse-files` select other native capture views. Ordinary use opens the same five views without developer flags.

Schema-2 `forecast.json` includes the exact optional water model, assumed humidity/pressure, source constants/coefficients/equations, numeric rows and null predictive uncertainty. Schema-2 manifest retains actual crop model, optional water model, complete descriptor and optional exact context. `samples.csv` keeps the thirteen original biomass/lighting columns followed by eight gas/conductance/water columns. Empty cells mean unavailable for water-disabled/legacy definitions, not zero. Comparison JSON includes each actual harvest water row. Existing folders/files are protected and incomplete exports report failure. Regular input reads are bounded to 64 KiB; FIFO/directory inputs are refused before opening.

## Limits and next connection

Transpired vapor is not irrigation demand, potable water production, condensate recovery or total facility consumption. Dark-period flux, nutrient consumption, humidity dynamics, crop stress, cooling/pump/dehumidifier power, arbitrary pressure/gravity response, hardware calibration and uncertainty propagation remain unavailable. Source implementation and independent quadrature checks are numerical verification, not independent empirical crop validation.

Optional saved habitat/world context remains an exact read-only observation; no food, water, gas or energy inventory is changed. A physical greenhouse still needs finite supply/storage, actual crop state, lighting allocation, humidity/heat exchange and harvest semantics. Dry edible lettuce does not establish complete crew nutrition. The master directive remains incomplete.
