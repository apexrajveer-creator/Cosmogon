# Finite carbon dioxide reduction and water recovery

Optional engine component `cosmogon-sabatier/1` connects captured CO₂ and isolated H₂ to finite separated methane and raw liquid product stores. Default equipment is absent; old habitats, the optional electrolysis model and the golden world remain unchanged. This is a reduced reference chemistry and engineering budget. [Research memo](SABATIER_RESEARCH.md) separates NASA's connected mechanism from assumed equipment ratings and the chosen NIST reference values.

Open Habitat engineering → Water recovery. Read actual held gases/products and the completed reaction interval. Enable CO₂ reduction, expand equipment assumptions and Apply configuration; the ordinary Controls pane also exposes the settings. Staged changes use normal read-only protection, validation and undo. Disabling retains methane, raw liquid and history; held raw liquid can still enter the existing powered recycler. Reducing either product capacity below held stock is rejected atomically, including old-configuration edit remainders.

The reaction consumes only captured CO₂ and isolated hydrogen already present at the start of the interval. Methane and raw water headroom, rate, electric intensity, equipment power and remaining bus power bound actual processing. Allocation follows base services, scrubber, recycler, heater, radiator and optional electrolysis. New gas capture, hydrogen production, raw liquid production and freed product headroom become usable in a later interval; there is no immediate recursive loop.

Raw liquid is separate from potable/process water. The existing powered recycler processes wastewater first and held raw product second, within its single rate/power budget. Its user-assumed recovery fraction splits processed mass into ordinary water and brine. This allocation does not establish contaminant removal or actual water quality. No methane venting, consumption or unlimited disposal is assumed; finite methane storage eventually stops the reaction.

## Definitions and reproducibility

Emit a normal definition using `cosmogon-cli habitat-definition --target Mars --output NEW_FILE`. Add the optional object under `config`:

```json
"sabatier": {
  "model": "cosmogon-sabatier/1",
  "power_limit_w": 1000.0,
  "co2_rate_kg_s": 0.000046296296296296294,
  "electricity_j_kg_co2": 2000000.0,
  "methane_capacity_kg": 2.0,
  "water_capacity_kg": 10.0
}
```

These illustrative ratings permit at most 4 kg CO₂/day. Hydrogen starts empty; enable [water electrolysis](HABITAT_ELECTROLYSIS.md) to produce feed. Zero equipment power or zero feed stops processing. `habitat --definition FILE --output NEW_DIR` records actual bounded trajectories; replay `initial.cosmo` for the same experiment. Native `--panel habitat-water` selects the pane, with `--habitat-definition FILE` or `--load SAVE`. Unknown fields, unsupported versions and malformed saves fail explicitly.

Protected exports retain original definition, current and last-interval settings, exact initial/final worlds, model/source manifest and CSV/JSON observations. New columns include held methane/raw water, reactor requests/supply, all reactant/product masses, electric/reference/total heat, downstream raw-water processing/recovery and cumulative ledgers. Reaction columns are blank without a completed active interval; an enabled but constrained interval records actual zero. Multiple constraints describe bounds below rated output without claiming unique causal attribution. Both model versions stay pinned when disabled histories still depend on them.

## Reference chemistry and heat

CO₂(g) + 4 H₂(g) → CH₄(g) + 2 H₂O(l), at a fixed 298.15 K reference. The selected formation enthalpies are CO₂ −393.51 kJ/mol and liquid water −285.830 kJ/mol (CODATA), methane −74.6 kJ/mol (Manion 2002 adopted recommendation). Overall reference ΔH = −252.750 kJ/mol CO₂, including condensation. This chosen methane value is distinct from the older Chase/JANAF −74.87 kJ/mol. Reported source uncertainties are retained separately and not propagated into model confidence intervals. Molar masses preserve internally consistent stoichiometry; rounded source methane/water display values are not mixed into the transfers.

Process electricity becomes heat and released reference reaction heat is added to the shared lumped thermal balance. Electrolysis retains its separate reference enthalpy debit. Cumulative reaction heat is historical transfer, not available energy or a claim that low-temperature catalysis runs instantaneously. Saved ledgers validate held hydrogen against electrolysis generation less reaction consumption, held methane against production, and raw water against production less powered processing.

No catalyst temperature/state, startup, equilibrium, gas separation, phase/purity or pressure correction, compression, tank structure, methane combustion/venting, reliability or field calibration. No closed-loop oxygen/water recovery percentage is inferred from stoichiometry. Physical greenhouse exchange, humidity/leaks, radiation, logistics, structures and habitat-specific audio remain incomplete master-directive requirements.
