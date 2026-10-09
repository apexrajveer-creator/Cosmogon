# Finite installed crops

View → Habitat engineering → Installed crops connects an **actual finite modelled process** to the cabin. It is separate from Crop planner: planner forecasts never grant stores. Overview shows supported source progress, retained dry material, actual feeds, electricity and completed heat transfers. Settings stages ordinary undoable edits. New crop setup creates an original source cycle with explicit finite inventories on the selected solid body. Import advanced definitions through Build & import.

## Operating assumption

The maintained-conditions checkbox starts **off**. Enabling it declares an ideal chamber boundary at the original source temperature, humidity, pressure and CO₂ regime. Cabin temperature or RH does not establish that a chamber maintains these conditions. This build does not simulate chamber HVAC, plant stress, survival, dark respiration or wet/root biomass.

One source cycle uses NASA's low-light lettuce MEC and original 31 daily cumulative edible/inedible dry and vapor values, with explicit linear interpolation between days. A 16-hour light window repeats in a terrestrial 24-hour day relative to installation time. Supported progress advances only under full allocated source light/control power. It is effective reference production progress, **not biological age**. Unsupported periods pause the reference calculation without claiming biological survival or later calibrated catch-up.

## Actual finite exchanges

Crops run after existing cabin services, reactors and moisture equipment in the explicit electrical priority. They consume available interval-start potable water and captured CO₂, plus the originally installed HNO₃-equivalent nutrient stock. Newly recycled water, newly captured CO₂ and newly freed product capacity wait until another interval. HNO₃-equivalent is a CHON bookkeeping basis, not literal fertilizer or a complete nutrient solution.

A separately pinned conservative material surrogate retains original rounded feed coefficients and infers CHON-only dry material and O₂. Oxygen enters bounded shared reserve. Dry edible/inedible material enters separate standing stores. Transpired water enters actual finite cabin vapor. Full fixture electricity is still consumed when material/storage constraints prevent growth; its unused chemical/phase portion enters sensible heat.

The original dry chemical-energy input defaults to **20 MJ/kg**, an explicit engineering assumption, not measured tissue enthalpy. Actual chemical storage and reference vapor phase energy must fit delivered light electricity. A bounded 48-iteration monotonic search limits each minute/remainder to available feeds, capacities and energy. These storage debits are retained in optional heat-ledger supplements; the original eleven-term array and legacy research catalogs remain intact.

## Harvest, disable and evidence

Harvest completed cycle is available only once the source progress reaches 30 days. It moves standing dry material to separate harvested edible/inedible stores, preserving mass and chemical storage. It does **not** credit the fixed complete crew diet or begin another cycle. Disable retains inventories, source curve and history. Runtime installation, rewriting the original crop source/chemical basis, or reducing capacity below stored material is refused.

Existing habitat studies include full JSON state and **crop.csv** when crops are present: 23 full-precision columns distinguish world/completed time, source progress, retained stores, cumulative transfers, electricity and completed interval storage/residuals. Missing interval evidence remains blank. The shared bounded reader checks CSV against original typed observations, exact saved endpoints, source/model pins and cumulative material/heat identities. Recorded Cabin balances include crop inventories and signed heat debits. WHY explains actual crop power/material/thermal paths.

## Command-line example

```bash
./target/release/cosmogon-cli habitat-crop-definition --target Mars --output /tmp/finite-crop.json
# Edit the original setup explicitly. The maintained-conditions assumption starts false.
./target/release/cosmogon-cli habitat --definition /tmp/finite-crop.json --samples 31 --interval-days 1 --output /tmp/finite-crop-study
./target/release/cosmogon-cli habitat-inspect --recording /tmp/finite-crop-study
./target/release/cosmogon-cli habitat-crop-harvest --initial /tmp/finite-crop-study/final.cosmo --habitat-id 0 --output /tmp/finite-crop-harvest
```

`--target` identifies an existing solid world. `--samples` includes the initial observation, with at most 1024 rows and 30 days total. `--interval-days` uses 86400 s per day. `--habitat-id` explicitly selects an installed experiment. Harvest requires completed supported progress; 30 chronological days alone does not guarantee completion. Existing output files/folders are protected. Harvest writes a verified ordinary snapshot plus the exact before-harvest world and action; snapshot initial/final refer to the same post-harvest world.

Packaged capture examples:

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/finite-crop-study/final.cosmo --focus Mars --speed 0 --panel habitat-crop --capture /tmp/finite-crop.png --capture-metadata --capture-after 240 --quiet --quit
```

Panels: `habitat-crop`, `habitat-crop-settings`, `habitat-crop-model`, `habitat-crop-build`. Crop capture sidecars retain the actual selected habitat/report, staged configuration and separate new setup. The app uses the same edits and engine as CLI. Untouched worlds and saves without crop state retain the existing golden `0xf2c4140a45ae509b`; explicitly installed state deliberately changes habitat save/state fingerprints and requires model/source pins.

[Preimplementation design](FINITE_GREENHOUSE_DESIGN.md). Original sources: [NASA BVAD](https://ntrs.nasa.gov/api/citations/20210024855/downloads/BVAD_2.15.22-final.pdf), [Volk and Cullingford](https://ntrs.nasa.gov/citations/19930004806), [CIAAW](https://ciaaw.org/abridged-atomic-weights.htm); the shared moisture model retains its NIST reference phase basis. Source snapshots and engineering limitations accompany exports. This conserves the selected engineering surrogate; it does not validate the combination as real crop biology. Phase 9, graphics and the wider master directive remain incomplete.
