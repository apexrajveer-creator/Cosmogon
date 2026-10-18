# Wall condensation and structural leak (build 53)

Cosmogon now couples cabin humidity to two passive/structural physical paths, both optional
and default-absent so existing worlds, saves, recordings and the world golden stay
byte-identical:

1. **Passive wall condensation / evaporation** (`cosmogon-cabin-wall-condensation/1`) —
   signed vapor↔wall-film exchange driven by the temperature difference between cabin air
   and the cabin walls, with explicit reference-phase heat.
2. **Nominal structural leak** (`cosmogon-cabin-structural-leak/1`) — slow authored air
   leakage that removes cabin gas proportionally to its composition and carries assumed
   sensible enthalpy out, independent of the discrete breach-orifice model.

## Wall condensation

Enabled per habitat via the optional `wall_condensation` configuration block. Each completed
interval, after powered dehumidification and crop vapor transfers:

* Wall temperature is an explicit engineering assumption:
  `T_wall = T_exterior + coupling × (T_cabin − T_exterior)` with authored `wall_coupling` ∈ [0,1].
* The exchange is a lumped surface mass transfer toward saturation at the wall temperature:
  `net = k × area × (p_vapor − p_sat(T_wall)) × dt`, signed (positive condenses vapor onto
  walls, negative evaporates held wall liquid), and bounded by available vapor, the held
  film's `liquid_capacity_kg` headroom, held film mass (evaporation) and a finite `rate_kg_s`
  cap.
* Saturation uses the existing NIST 273.15–303 K Antoine fit only. Outside this domain
  (e.g. cold Mars walls) no exchange occurs and the interval budget retains an explicit
  constraint note. No ice, fogging, drips or collection plumbing is modelled; wall liquid is
  a held store with no recycling path.
* Phase energy: `net × 44.004 kJ/mol reference` is released into (condensation) or debited
  from (evaporation) cabin sensible heat, matching the powered dehumidifier's accounting.
* Evidence: `state.wall_liquid_kg`, cumulative signed `totals.wall_condensed_kg`, per-interval
  `budget.wall`, save pins, WHY diagnostics ("wall"), humidity-pane readouts and staging
  controls, and two optional `samples.csv` tail columns (`wall_liquid_kg`,
  `total_wall_condensed_kg`) present only when wall evidence exists.

## Structural leak

Enabled per habitat via the optional `leak` block (`rate_kg_s`, assumed
`gas_specific_heat_j_kg`). Each interval the leak samples start-of-interval cabin gas and
removes `min(rate × dt, total gas)` proportionally to species masses; the mixed gas enthalpy
carried out (`leaked × cp × T_cabin`) cools the lumped cabin in the same thermal balance as
all other heat terms. Escaped masses join the cumulative `escaped_gas` outside ledger and
`totals.leaked_gas`; the breach substep still runs last, so both models compose inside one
interval with all retained identities intact. The leak is not the breach model: no pressure,
orifice, discharge coefficient or damage state; no seal degradation, inward or rarefied flow
is modelled.

## Using it

* Native: Habitat → Cabin moisture pane shows wall film, signed net exchange, wall
  temperature, cumulative leak and interval enthalpy removal; the Settings pane stages
  "Passive wall condensation" and "Structural leak" blocks alongside moisture.
* CLI: definitions are ordinary habitat config JSON (`wall_condensation`, `leak` blocks;
  `habitat-definition` presets remain default-absent). `habitat` studies and
  `habitat-diagnose` explain the new retained evidence ("wall", "leak" explanations,
  wall/leak heat and water terms).
* Heat-ledger worlds (`record_heat`) gain two optional supplements
  (`wall_condensation_heat`, `leak_cooling`) added to the span without changing the original
  eleven-term array.

Limits and assumptions are retained in `moisture::source()`, `moisture::leak_source()`,
the habitat report and `docs/WALL_CONDENSATION_LEAK_DESIGN.md`.
