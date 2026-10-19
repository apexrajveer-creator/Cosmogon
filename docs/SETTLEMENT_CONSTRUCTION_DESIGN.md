# Finite settlement construction and logistics — design (build 55)

Closes two remaining phase-9 gaps ("resources/construction … logistics") with physically
grounded, finite, journaled mechanisms. Both are optional and default-absent; legacy worlds,
saves, recordings and the golden stay byte-identical. Models:
`cosmogon-cabin-construction/1`, `cosmogon-habitat-stock-transfer/1`.

## A. Continuous cabin construction

Optional `habitat::Config.construction` (`construction::Config`):

* `material_capacity_kg` ∈ (0, 1e7] — bound of held processed construction material.
* `build_rate_kg_s` ∈ (0, 1e4] — material consumed per second at full served power.
* `power_w` ∈ [0, 1e6] — ordinary bus electricity for construction.
* `volume_per_kg_m3` ∈ (0, 1] — authored structural bulk ratio (cabin volume added per
  kilogram consumed).

State: `construction_material_kg` (skip-if-zero, bounded by capacity). Totals:
`constructed_volume_m3` and `construction_material_used_kg` (both skip-if-zero). Budget:
`construction` block (material before/after, consumed, volume added, requested/delivered W,
served fraction, constraints).

Each completed interval: a tenth power slot requests `construction.power_w` (lowest
priority, after crop); the served fraction scales the build; `consumed = min(material,
build_rate · dt · fraction)` further bounded so the resulting volume never exceeds the
authored `volume_m3` upper bound (1e6 m³); volume added = `consumed · volume_per_kg_m3`.
The volume increase is applied to the **live configuration at interval end** (the same
channel as an ordinary volume edit), so the next interval's gas pressures reflect the
larger cabin at unchanged gas mass — real thinning — while each interval's retained
budget configuration keeps the pre-add volume it actually ran at. Delivered construction
power is ordinary bus electricity and becomes cabin heat through the existing ledger.

Material supply: `Edit::SupplyHabitatConstructionMaterial {system, id, requested_kg}`
(CLI `habitat-construction-supply`) — one explicit bounded journaled charge (0…1e7 kg,
refusing to exceed capacity), the same pattern as the build-52 crop nutrient charge. This
is an authored supply, not a modeled delivery; vehicle-mediated delivery of construction
material remains future work and is labelled as such.

Refusals: empty/invalid configuration bounds; supply above remaining capacity; consumption
never leaves negative material. Disabling the configuration freezes the held material
(retained, re-enable resumes); no mass is discarded.

## B. Habitat-to-habitat stock transfer (settlement logistics)

`Edit::TransferHabitatStock {system, from, to, requested: supply_vehicle::Cargo}`
(CLI `habitat-stock-transfer`) moves finite supplies between
two habitats of the **same body**: both are stepped to the edit clock first (preview-clone
pattern), then per species `accepted = min(requested, held_from, headroom_to)` with an
exact `held + accepted ≤ tank` clamp; zero total accepted refuses ("accepts no material");
nothing partial is applied on refusal. Conservation is exact: sender decreases equal
receiver increases, species by species, in the same ordinary state update.

Retained evidence: each side stores a `StockTransfer { counterparty, world_time_s, moved,
outbound }` record (positive masses, `outbound` distinguishes sender/receiver);
`validate_stock_transfers` re-derives ordered outbound↔inbound pairing (same counterparty,
time and masses) on reopen and refuses tampering or orphans. Conditional
`cosmogon-habitat-stock-transfer` save pin whenever any habitat has records. Receipts from
vehicle deliveries are unchanged; this is intra-settlement logistics only.

## Surfaces

* Save pins conditional on use; new fields skip-if-absent/zero.
* `samples.csv` gains five optional construction tail columns
  (`construction_material_kg`, `construction_requested_w`, `construction_delivered_w`,
  `construction_volume_added_m3`, `total_constructed_volume_m3`) present only when any row
  has construction evidence; accepted widths extend by one band.
* `habitat-diagnose`/WHY gains a "construction" explanation (material, consumed, volume
  added, power, constraints); report gains a construction observation + source.
* Native: new **Settlement** tab (index 13, Overview/Settings/Model & sources) —
  construction readouts + staging controls, stock-transfer authoring (from/to + four
  amounts) and retained transfer history; panel names `habitat-settlement[-settings|-model]`.
* CLI: `habitat-construction-supply`, `habitat-stock-transfer` (protected before-world,
  action JSON with fingerprints, ordinary recording-folder output like crop edits).

## Deliberate limitations

Construction rates, bulk ratio and power are authored assumptions; no structural strength,
thermal mass growth, dust, EVA, crew labour, crane or assembly physics; no stochastic
delays. Stock transfer is instantaneous within one settlement site; no roads, vehicles,
warehousing, markets or inter-settlement caravans; no construction-material delivery by
craft yet. Numerical conservation does not establish predictive accuracy.
