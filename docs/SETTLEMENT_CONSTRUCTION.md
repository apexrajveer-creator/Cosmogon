# Finite settlement construction and logistics (build 55)

Cosmogon cabins can now consume finite held material into structure and settlements can move
finite supplies between habitats of the same world, with paired retained evidence and exact
conservation. Both mechanisms are optional and default-absent, so legacy worlds, saves,
recordings and the world golden stay byte-identical.

Model pins: `cosmogon-cabin-construction/1`
(`crates/cosmogon_sim/src/habitat_construction.rs`) and
`cosmogon-habitat-stock-transfer/1` (`crates/cosmogon_sim/src/habitat.rs`).

## A. Continuous cabin construction

Enabled per habitat via the optional `construction` configuration block:

* `material_capacity_kg` ∈ (0, 1e7] — bound of held processed construction material.
* `build_rate_kg_s` ∈ (0, 1e4] — material consumed per second at full served power.
* `power_w` ∈ [0, 1e6] — ordinary bus electricity for construction.
* `volume_per_kg_m3` ∈ (0, 1] — authored structural bulk ratio (cabin volume added per
  kilogram consumed).

Each completed interval, a tenth power slot (lowest priority, after the crop slot) requests
`power_w`; the served fraction scales the build. `consumed = min(held, build_rate · dt ·
served)` is further bounded so the resulting volume never exceeds the authored 1e6 m³
volume bound; volume added = `consumed · volume_per_kg_m3`. The volume increase is applied
to the **live configuration at interval end**, after the retained budget configuration has
been captured — so the next interval's gas pressures reflect the larger cabin at unchanged
gas mass (real thinning), while each interval's budget keeps the pre-add volume it actually
ran at. Consumed material becomes `construction_structure_kg`; held + structure are both in
`mass_kg()` so total mass is conserved exactly. Delivered construction power is ordinary
bus electricity and becomes cabin heat through the existing ledger; thermal mass
deliberately stays unchanged.

Material arrives only through `Edit::SupplyHabitatConstructionMaterial {system, id,
requested_kg}` (CLI `habitat-construction-supply`) — one explicit bounded journaled charge
that refuses to exceed capacity ("Construction material supply exceeds held capacity"). No
stock is granted by building the habitat; the starting held material is zero.

Evidence: held `state.construction_material_kg` / `construction_structure_kg`,
per-interval `budget.construction` (material before/after, consumed, volume added,
requested/delivered W, served fraction, constraints), cumulative
`totals.constructed_volume_m3` and `totals.construction_material_used_kg`, conditional save
pins, a "construction" diagnostics explanation plus report observation/source, and five
optional `samples.csv` tail columns (`construction_material_kg`,
`construction_requested_w`, `construction_delivered_w`, `construction_volume_added_m3`,
`total_constructed_volume_m3`) present only when construction evidence exists.

## B. Habitat-to-habitat stock transfer

`Edit::TransferHabitatStock {system, from, to, requested: supply_vehicle::Cargo}` (CLI
`habitat-stock-transfer`) moves finite supplies between two habitats of the **same body**.
Both habitats are stepped to the edit clock first (preview-clone pattern), then per species
`accepted = min(requested, sender held, recipient free)` with an exact
`held + accepted ≤ capacity` clamp; a transfer that accepts nothing is refused ("Habitat
stock transfer accepts no material") and nothing partial is applied. Conservation is exact:
sender decreases equal receiver increases, species by species.

Each side stores a retained `StockTransfer {model, counterparty, world_time_s, moved,
outbound}` record; `validate_stock_transfers` re-derives ordered outbound↔inbound pairing
(time and masses; the counterparty is implied by the filter) on reopen and refuses tampered
or orphaned records. The save pins `cosmogon-habitat-stock-transfer` whenever any habitat
has records or any recorded edit is a stock transfer.

## Surfaces

* Native: Habitat → **Settlement** tab (index 13, Overview / Settings / Model & sources).
  Overview shows construction state, interval accounting and retained transfer history, and
  authors supply charges and transfers (source/recipient combo boxes plus four species
  amounts); Settings stages the `construction` block; Model shows source equations and
  assumptions. Panels: `habitat-settlement`, `habitat-settlement-settings`,
  `habitat-settlement-model`.
* CLI: `habitat-construction-supply --initial … --habitat-id … --material-kg …` and
  `habitat-stock-transfer --initial … --from-id … --to-id … [--oxygen-kg … --water-kg …
  --food-dry-kg … --food-water-kg …]` (at least one species flag required). Both refuse
  ignored/incomplete invocations, write the protected before-world, an action JSON with
  before-state fingerprints, and the updated final world — the crop-edit pattern.
* Configuration refusals: construction bounds outside (0,1e7] kg / (0,1e4] kg/s / [0,1e6] W
  / (0,1] m³/kg; capacity below already-held material ("no material discarded"); supply
  above remaining capacity; transfers across different bodies, with the source and
  destination equal, or accepting nothing.

## Deliberate limitations

Construction rates, bulk ratio and power are authored assumptions, not measurements. No
structural strength, thermal-mass growth, dust, EVA, crew labour, crane or assembly physics,
or stochastic delays; supply is an authored charge, not a modeled delivery. Stock transfer
is instantaneous within one settlement site; no roads, vehicles, warehouses, markets or
inter-settlement caravans. Numerical conservation does not establish predictive accuracy.
