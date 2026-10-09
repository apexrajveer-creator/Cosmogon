# Controlled cabin geometry and thermal studies

Habitat engineering → Recordings → **Run experiment** includes four additional typed settings. Choose a live habitat, select the setting and enter distinct numeric levels in its displayed units. **Use illustrative levels** fills two editable values from half/double the current setting within existing bounds; zero conductance/area uses a nonzero example. These examples are not recommended equipment or optimized designs. Pair with a second setting to run all four combinations from the same captured world.

| Parameter | Unit | Valid range | Meaning |
|---|---|---|---|
| `volume_m3` | m³ | 1–1,000,000 | Assumed cabin volume at fixed actual initial gas |
| `heat_capacity_j_k` | J/K | 1,000,000–1,000,000,000,000 | Effective lumped cabin heat capacity |
| `conductance_w_k` | W/K | 0–1,000 | Total envelope heat conductance |
| `radiator_area_m2` | m² | 0–100,000 | Radiating area at fixed emissivity and pump limit |

Every branch uses an ordinary configuration edit before recording. Actual stocks, gas mass, initial temperature and original creation definition are preserved. The starting [ideal-gas pressure](https://www.grc.nasa.gov/www/k-12/airplane/eqstat.html) therefore changes inversely with volume. This is a different hypothetical cabin, **not simulated expansion, a moving wall or piston work**. No gas is added to restore pressure and other equipment does not scale automatically.

Changing effective heat capacity at the same temperature changes the model's implied reference sensible energy; this is not an equal-initial-energy experiment. [Heat capacity depends on material/process](https://www.grc.nasa.gov/www/k-12/airplane/specheat.html); these values are assumptions, not calibrated material data. Conductance is a whole-envelope W/K value, not material conductivity or insulation thickness. Radiator area alone does not guarantee removal: the existing thermostat, delivered pump power, emissivity and temperature determine actual heat rejection. Optional breach still refuses unsupported negative inferred structure heat capacity; no corrective energy is invented.

**What these changes mean** explains selected settings in native experiment and paired Files & method. Protected manifests, final/selected factorial reports, trajectory JSON and completion manifest retain the exact same source/method note. Total pressure is included in all new geometry/thermal designs: 19 metrics without breach, 22 when breach is used. Legacy designs retain their original 18/22 catalogs and exact reports. Numeric CSV observations are unchanged; paired contrast CSV keeps forty columns and includes actual pressure responses/effects. First observation flows may retain completed pre-edit settings; existing interval-match flags are explicit.

```sh
./target/release/cosmogon-cli habitat-sweep --initial /tmp/common.cosmo \
  --parameter volume_m3 --from 250 --to 1000 --runs 3 \
  --samples 25 --interval-days 0.041666666666666664 --output /tmp/volume-study-new
./target/release/cosmogon-cli habitat-compare \
  --recording /tmp/volume-study-new/run-000 --reference /tmp/volume-study-new/run-002 \
  --measure total_pressure --alignment elapsed --output /tmp/pressure-comparison-new
```

For a paired study save this bounded plan as `/tmp/volume-capacity-plan.json`, then run `habitat-sweep --initial /tmp/common.cosmo --definition /tmp/volume-capacity-plan.json --output /tmp/paired-thermal-new`:

```json
{"format":"cosmogon-habitat-sweep-plan","schema_version":2,"parameter":"volume_m3","values":[250,1000],"samples":25,"interval_s":3600,"secondary":{"parameter":"heat_capacity_j_k","values":[50000000,1000000000]}}
```

`habitat-inspect --recording /tmp/paired-thermal-new --observation 12 --output /tmp/thermal-trajectories-new` exports all actual paired trajectories; `habitat-verify --recording /tmp/paired-thermal-new --output /tmp/thermal-replay-new` independently recomputes every full sample/endpoint/source. [NIST factorial design](https://www.itl.nist.gov/div898/handbook/pri/section3/pri3331.htm) supports four-corner conditional/interaction arithmetic; deterministic sensitivity does not provide statistical confidence or field calibration. Use [pressure explanations](RECORDED_PRESSURE.md) on an individual branch for components/ordered endpoint accounting.

Native `--habitat-sweep PLAN --load SAVE --habitat-study-output NEW_DIR` uses the exact saved world on a worker. `--panel habitat-experiments` retains the restored editable inputs after automatic completion, clears the output field, and keeps completed evidence separate. Reopening a paired study also restores its typed controls. Invalid branches are refused before output/GPU launch. Captures embed separate staged geometry/thermal inputs and exact completed analyses. See [design](THERMAL_STUDIES_DESIGN.md) and [verification](THERMAL_STUDIES_VALIDATION.md). No new simulation law, save schema, dependency or golden change. Physical greenhouse, calibrated properties, physical expansion and the full directive remain incomplete.
