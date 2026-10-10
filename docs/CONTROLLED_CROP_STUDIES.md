# Controlled installed crop studies

Habitat engineering → Recordings → Controlled experiment includes four typed installed-crop controls. Run a single-setting sweep or pair two settings at two levels. Every branch starts from the same actual saved world through one ordinary configuration edit; the live world is independent.

| Parameter | Unit / levels | Meaning |
|---|---|---|
| `crop_maintained_conditions` | exactly 0 or 1 | Off or maintained chamber **by assumption**, not a working HVAC controller or measured sensor |
| `crop_auxiliary_power_w` | W | Assumed full auxiliary electrical demand alongside the original source lighting |
| `crop_dry_capacity_kg` | kg | Combined standing and harvested dry product capacity |
| `crop_oxygen_capacity_kg` | kg | Existing shared oxygen reserve capacity; enabled electrolysis can impose a smaller limit |

An original installed crop and an enabled crop configuration are required. Original source curves, chemical basis, original nutrients, actual held materials/counters, installation clock and harvest state remain fixed. Capacity cannot discard material; a larger limit supplies no biomass, oxygen, nutrients, energy or new cycle. Turning the ideal assumption off pauses supported reference progress without predicting plant stress or survival. Full auxiliary allocation is required by this reduced model; levels are user assumptions, not calibrated equipment ratings.

Illustrative levels are editable. Crop capacity examples respect actual held material. Boolean assumptions accept only 0/1, including in a paired plan; a three-level 0-to-1 linear sweep is refused. Full ordinary-edit preflight checks every branch before output is created. Existing destinations stay protected.

New plans containing a typed crop parameter retain all thirteen actual crop state metrics in final and aggregate paired analyses. Previous plans retain their original catalogs, even if they contain installed crops. Explicit selected crop graphs/figures continue working for those older plans. A crop-only plan does not acquire geometry/pressure responses merely because it contains an assumption note. Mixed thermal/crop plans retain both scopes and their plan-dependent pressure response.

Completed design identity stays separate from editable inputs for a rerun. Reopen paired designs at their root, or select an individual single-setting `run-000`/`run-001` recording; complete single-setting root reopening follows in the next connected build. Inspect actual trajectories and support-aware four-response contrasts, independently replay saved branches, or export a complete protected research figure. Recorded context preserves original configuration, progress/elapsed time, counters, source/chemical pins and actual completed budgets. Lines connect stored points; no interpolation of missing operating history or biological observation is claimed.

```json
{"format":"cosmogon-habitat-sweep-plan","schema_version":2,"parameter":"crop_maintained_conditions","values":[0,1],"samples":5,"interval_s":43200,"secondary":{"parameter":"crop_auxiliary_power_w","values":[0,10000]}}
```

```bash
./target/release/cosmogon-cli habitat-sweep --initial /tmp/installed-crop.cosmo \
  --habitat-id 0 --definition /tmp/crop-plan.json --output /tmp/new-crop-design
./target/release/cosmogon-cli habitat-verify --recording /tmp/new-crop-design
```

The [NASA material-flow reference](https://ntrs.nasa.gov/citations/19930004806) and original [NASA MEC conditions](https://ntrs.nasa.gov/api/citations/20210024855/downloads/BVAD_2.15.22-final.pdf) support the retained reference inputs, not our auxiliary watts/storage capacities or field calibration. Four branches follow [NIST's standard factorial order](https://www.itl.nist.gov/div898/handbook/pri/section3/pri3331.htm); deterministic contrasts are not p-values, predictive uncertainty or noise replications.

[Finite process](FINITE_GREENHOUSE.md), [actual crop trajectories](RECORDED_CROP_TRAJECTORIES.md), [preimplementation design](CONTROLLED_CROP_STUDIES_DESIGN.md). No solver, save schema, graphics or dependency change. Calibration, chamber hardware, repeated cycles, nutrition and the master directive remain incomplete.
