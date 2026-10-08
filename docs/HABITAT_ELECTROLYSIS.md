# Finite habitat oxygen generation

Optional engine component `cosmogon-water-electrolysis/1` connects reaction chemistry to the same habitat experiment state. Default habitats keep the reactor absent. Untouched worlds and the existing golden fingerprint retain their state. This is a reduced engineering experiment, not calibrated flight equipment or a physical greenhouse installation.

## Native use

Open Habitat engineering → Oxygen. The pane shows actual held oxygen/hydrogen and the last completed reaction interval. Enable water electrolysis, expand equipment assumptions if needed, then Apply configuration. Existing Controls also exposes these settings. All changes use normal validated edits with read-only protection and undo. Staged changes do not alter the simulation. Disabling preserves products/history. Reducing capacity below already held oxygen or hydrogen is rejected atomically, including any old-configuration edit-time remainder.

Power is allocated after base services, CO₂ capture, recycling, heater and radiator. The reactor reserves nominal crew water before planning consumption and draws only for bounded production. Start-of-interval available water, both product stores, maximum rate and power limit constrain demand. Fresh recycled water and newly freed capacity wait until next interval. Produced oxygen enters the existing reserve; the regulator can use it next interval and retains its flow limit. This can reduce unserved nominal inventory demand, but cannot establish breathable/safe conditions.

## Reproducible definitions and studies

Emit the ordinary habitat definition with `cosmogon-cli habitat-definition --target Mars --output NEW_FILE`. Add this optional object under `config` before importing or running the definition:

```json
"electrolysis": {
  "model": "cosmogon-water-electrolysis/1",
  "power_limit_w": 2000.0,
  "oxygen_rate_kg_s": 0.00006944444444444444,
  "electricity_j_kg_oxygen": 25000000.0,
  "oxygen_capacity_kg": 200.0,
  "hydrogen_capacity_kg": 2.0
}
```

These are illustrative assumptions (6 kg O₂/day rate ceiling), not spacecraft ratings. Initial oxygen must fit the optional shared reserve capacity; hydrogen starts empty. Use `habitat --definition FILE --output NEW_DIR` for 25 hourly actual observations, or explicit bounded samples/intervals. Replay from exported `initial.cosmo`; the optional model/version is pinned and missing/unsupported source model versions are rejected. Native developer `--panel habitat-oxygen` opens the pane; `--habitat-definition FILE` builds an explicit experiment, while `--load SAVE` preserves exact state. Automated invalid definitions/saves fail before GPU startup.

Protected habitat exports include original definition, current/interval configurations, exact initial/final worlds, source/model manifest and CSV/JSON observations. CSV appends held `hydrogen_kg`, reactor requested/delivered W, O₂/H₂/water kg, electricity/reaction-enthalpy/waste-heat J and cumulative reaction enthalpy J. Reaction columns are blank when no completed active interval exists, not fabricated zero observations. An active reactor that is resource constrained records actual zero production. Reports and CSV retain the interval’s production constraints (water after crew reserve, product headroom, equipment power or bus supply). Multiple bounds can be below the configured rate; the list does not claim a unique counterfactual attribution. JSON retains optional absent state and the distinction between current equipment and the last interval.

## Chemistry and heat accounting

[Research memo](ELECTROLYSIS_RESEARCH.md) records primary sources. The reference reaction is 2 H₂O(l) → 2 H₂(g) + O₂(g). Water withdrawal equals both separated products; hydrogen remains isolated and enters the total mass ledger. No automatic disposal creates unlimited hydrogen capacity.

Electrical energy is split into reference reaction enthalpy and waste heat. The first is subtracted from the habitat's otherwise all-electrical-load-as-heat budget. Reference enthalpy is fixed at 298.15 K; electrical intensity must be at least that thermoneutral value. Cumulative reference enthalpy transfer is a historical ledger, not recoverable chemical energy currently stored after oxygen participates in crew exchange. Source constants and reported source uncertainty accompany exports; uncertainty is not propagated into a predictive confidence interval.

No water phase/purity model, electrochemical voltage/current, process temperature/pressure correction, compression, tank structure, leakage/ignition, venting, fuel-cell recovery, maintenance or hardware calibration. Water is a modeled process stock, not a measured supply meeting purity or phase requirements. This model excludes below-thermoneutral operation requiring external reaction heat. Optional [CO₂ reduction](HABITAT_SABATIER.md) is a separately pinned reaction; its consumption is retained in hydrogen history validation. Physical greenhouse resource coupling, radiation, logistics, site structures and habitat audio remain separate master-directive acceptance gates.
