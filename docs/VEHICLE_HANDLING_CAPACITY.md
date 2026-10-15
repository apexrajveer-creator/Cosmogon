# Finite handling equipment and recipient storage capacity

Build 46 of the supply sequence. This build adds the two authorization inputs that a future
atomic transfer ([CONTINUATION_HANDOFF](CONTINUATION_HANDOFF.md) item 5) must respect: finite
craft-side handling equipment and finite habitat-side general storage tanks, with a pure
request/accept/reject preflight between them. **Nothing is transferred, received or delivered
in this build.** It follows the surface-operation window (build 44) and cargo-mass world flight
(build 45) and the delivery invariants in [VEHICLE_SUPPORT_NEXT](VEHICLE_SUPPORT_NEXT.md).

## Recipient side: habitat general storage tanks

Four optional `habitat::Config` fields bound the four transferable supply stocks:

| Field | Stock it tanks | Default |
| --- | --- | --- |
| `oxygen_reserve_capacity_kg` | `oxygen_reserve_kg` (reserve, never cabin pressure) | `1e7` |
| `potable_water_capacity_kg` | `potable_water_kg` (potable liquid, never food-bound water) | `1e7` |
| `food_dry_capacity_kg` | `food_dry_kg` (dry food, no energy credit) | `1e7` |
| `food_water_capacity_kg` | `food_water_kg` (food-bound water, not potable) | `1e7` |

The default `1e7` kg is exactly the historical general upper bound on these stocks, so
legacy definitions load with identical semantics and serialize byte-exactly (the fields are
skipped while they equal the default). Rules enforced everywhere definitions, states and
reconfigurations are validated:

* Each initial stock must fit its tank.
* Each held stock must fit the **active** configuration's tank (`check_config` refuses a
  reconfiguration below held stock with "no mass was discarded", matching the battery and
  CO₂-capture precedents).
* Internal oxygen production is gated only by installed equipment product capacities, so the
  O₂ tank must at least cover the crop/electrolysis oxygen capacity (both are bounded `0..=1e7`,
  so every legacy configuration satisfies the coupling at the default tank).
* Potable-water recovery (wastewater, Sabatier raw water, condensate recycling) now clamps
  recovered liquid to the tank's remaining free space; the un-recovered processed fraction
  stays as brine, never discarded. At the default tank the clamp never binds, so legacy
  simulation trajectories are unchanged.

New query API for the transfer bridge: `Habitat::held_cargo()`,
`Habitat::storage_capacity_cargo()` and `Habitat::available_cargo()` (free = capacity − held,
never negative), all in the same `supply_vehicle::Cargo` species mapping.

## Craft side: handling equipment

`world_vehicle::Definition.handling: Option<vehicle_handling::Equipment>` is opt-in
(`cosmogon-vehicle-handling/1`, absent legacy definitions serialize byte-exactly, conditional
save pin). One `Equipment` authors, per supported operation:

* `per_species_limit_kg` — bound on one species moved.
* `total_limit_kg` — bound on all species moved.
* `oxygen_interface_limit_kg` — **one shared finite oxygen-handling interface**; oxygen moves
  do not receive a separate allowance on top of it.
* `power_w` / `thermal_rejection_w` — declared assumptions. No craft power bus and no handling
  thermal integration are modeled.
* `required_operation_window_s` — minimum continuous supported-operation window that must
  exist at the retained craft clock (evidence from build 44's solver-integrated ledger).

Structural refusals prevent dead configuration: handling equipment requires cargo ledgers
(build 45) and authored contact surface-operation hardware (build 44).

## Preflight: request / accepted / rejected

`vehicle_handling::preflight(flight, recipient, requested) -> Result<Preflight>` is a pure
accounting evaluation. Strict refusals (authored equipment, flying phase, ledgers and retained
ledger evidence, same body, adequate continuous window, finite nonnegative request) change
nothing. Otherwise, per species in the fixed order O₂, potable water, dry food, food water:

```
accepted_i = min(requested_i, held_i, per_species_limit_kg, recipient_free_i)
```

then the accepted set is trimmed from the **end** of that order down to `total_limit_kg`
(food water first, then dry food, …), and the oxygen species is finally capped by the shared
`oxygen_interface_limit_kg`. `rejected = requested − accepted` per species; rejected stock
stays on the craft. The `Preflight` records requested, accepted, rejected and the qualifying
window, so requests, accepted amounts and rejected stock remain separate and explicit.

The preflight is **authorize-only**: it never mutates the flight, the recipient or any ledger.
Full co-location (site/frame/time geometry), species-specific atomic `Edit`s, undo/save/replay,
transfer momentum/energy/heat evidence and the recipient-capacity commit gate are build 47.

## Exposure

* Native: habitat "Finite supply storage" editors and free-tank display; world-vehicle pane
  readout of handling limits, declared assumptions and the current supported window.
* CLI: studies of definitions that carry `handling` gain a conditional `manifest["handling"]`
  block (authored limits, final window, scope) alongside the existing `operation_window` and
  `cargo` blocks, with exact `flight-inspect` replay. Habitat capacities travel with world
  saves (validated on reopen).
* Save format: a conditional `cosmogon-vehicle-handling` pin appears only when a world
  contains (or is about to create) a craft with handling equipment.

## Limits

Assumptions remain assumptions: instantaneous handling, declared-but-unmodeled power/thermal
draw, and the preflight's accounting scope. Oxygen reserve is never cabin pressure;
food-bound water is not potable liquid; dry food implies no energy credit. There is no
unlimited tank, discarded overflow, timer delivery, invisible grant or automatic arrival.
A preflight acceptance is not landing, attachment, transfer, delivery, hardware feasibility
or mission feasibility. Detailed body/terrain/civilization graphics remain queued under the
owner's supplement.
