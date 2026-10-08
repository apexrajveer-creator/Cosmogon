# Controlled moisture studies

Habitat engineering → Recordings → Run experiment exposes six additional settings in the existing deterministic experiment runner. Enable finite cabin moisture in ordinary Humidity Settings first. Native all-branch validation shows invalid assumptions or capacity below retained liquid before offering Run; the worker previews every ordinary edit again before creating output.

| Parameter | Unit | Changed future assumption |
|---|---|---|
| `moisture_airborne_fraction` | fraction | Existing nominal crew water partition and reference phase-energy debit |
| `moisture_humidity_setpoint` | fraction | Supported saturation-based vapor target |
| `moisture_power_w` | W | Collector electric limit, after existing bus priorities |
| `moisture_rate_kg_s` | kg/s | Maximum collected liquid rate |
| `moisture_electricity_j_kg` | J/kg | Assumed process electricity per mass collected |
| `moisture_condensate_capacity_kg` | kg | Raw-liquid storage; held mass is fixed |

Every branch starts from the same exact world; new ratings do not grant water, erase vapor or refill storage. The original saved state, source/version pins, all 85-column trajectories and completion manifest are protected. Initial observations can retain the last completed interval under the old configuration. Humidity Overview labels that interval duration/end explicitly. Full mechanism disable restores future crew output to legacy liquid collection while retaining held phase stocks; zero collector power/rate keeps emissions active.

Choose recorded vapor, condensate, relative humidity or condensation heat alongside existing power/temperature/water metrics. Comparisons use exact stored times; interval heat additionally requires aligned equal completed intervals. Unsupported-temperature RH remains unavailable. Independent arbitrary recordings do not establish a controlled experiment; use one sweep's branches for one-setting sensitivity.

```sh
./target/release/cosmogon-cli habitat-sweep --initial /tmp/moisture.cosmo \
  --parameter moisture_power_w --from 0 --to 500 --runs 3 \
  --samples 25 --interval-days 0.041666666666666664 --output /tmp/power-study
./target/release/cosmogon-cli habitat-compare \
  --recording /tmp/power-study/run-000 --reference /tmp/power-study/run-002 \
  --measure humidity --alignment elapsed --output /tmp/humidity-comparison
```

See [HABITAT_COMPARISONS](HABITAT_COMPARISONS.md) for native worker launch/replay and bounded plans, [HABITAT_HUMIDITY](HABITAT_HUMIDITY.md) for source laws/limits and [MOISTURE_STUDIES_DESIGN](MOISTURE_STUDIES_DESIGN.md) for the preimplementation design. This measures conditional model sensitivity, without hardware calibration, predictive uncertainty, interaction analysis, recommended cabin humidity or global optimization. Physical greenhouse exchange remains incomplete.
