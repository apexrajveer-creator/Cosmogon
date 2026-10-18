# Physical humidity/leak/wall-condensation coupling: design draft before implementation

Next connected phase-9 build after the physical greenhouse exchange. Two passive/structural
physical couplings join the existing finite cabin moisture system: (1) **wall condensation**
— signed vapor↔wall-liquid exchange driven by the temperature difference between cabin air
and the cabin walls, with explicit phase energy — and (2) **nominal structural leak** — a
slow authored air leakage that removes cabin gas proportionally to its composition and
carries sensible enthalpy out, independent of the discrete breach-orifice model. Both are
optional, versioned, default-absent, and never silently activate. Existing worlds, old
saves, recordings without the new evidence and the world golden stay byte-identical.

## Research and limits

The finite moisture build (build 17) deliberately retained supersaturation "rather than
inventing passive wall condensation". This build adds that missing physical path honestly:
saturation uses the existing NIST 273.15–303 K fit only (no ice, no high-temperature
extrapolation — outside the domain no wall exchange occurs and stores stay unchanged);
the reference phase enthalpy 44.004 kJ/mol (REFERENCE_PHASE_J_KG) moves with the exchanged
mass exactly as the powered dehumidifier already accounts it (condensation releases it into
sensible cabin heat, evaporation debits it). Wall surface temperature is an explicit
engineering assumption: T_wall = T_exterior + f·(T_cabin − T_exterior) with an authored
wall thermal coupling fraction f ∈ [0,1] (f=1 means walls track cabin air; f=0 means walls
sit at exterior temperature). The exchange flux is signed and linear in the vapor-pressure
difference: rate = k·area·(p_vapor − p_sat(T_wall)) kg/s (k in kg/(s·m²·Pa)), bounded by
available vapor (condensation) or held wall liquid (evaporation), by a finite rate limit,
and by a wall liquid film capacity. This is a lumped surface mass-transfer assumption, not
a CFD film, fogging or structural icing model.

The structural leak is a user-assumed nominal leakage rate (kg/s of cabin gas at cabin
conditions) applied to the mixed cabin gas proportionally to each species' current mass
fraction — it is not the breach orifice model (which keeps its pressure-driven
composition-weighted outflow) and never replaces it; both may coexist and add to the same
retained outside-mass store (`escaped_gas`). Leaking gas carries its sensible enthalpy out
of the lumped cabin using an explicit assumed constant cabin-gas specific heat (same
engineering-assumption class as the existing lumped thermal model). No seal degradation,
pressure-driven flow, inward/rarefied flow, humidity-dependent leak rate, wall icing,
drips, collection plumbing or EVA is modelled; wall liquid is a held store with no
recycling path in this build (deliberate limitation, like raw wastewater before a
recycler). The leak rate does not depend on cabin pressure or damage state.

## Selected mechanism (2026-10-19, before Rust changes)

1. `moisture::WallConfig` (`cosmogon-cabin-wall-condensation/1`): bounded `area_m2`
   (0…1e5), `mass_transfer_kg_s_m2_pa` (0…1e-6), `wall_coupling` (0…1),
   `liquid_capacity_kg` (0…1e7), `rate_kg_s` (0…100, finite per-interval cap). Installed as
   optional `habitat::Config.wall_condensation`. Passive: no bus power, no electrical
   priority slot. Applied after powered moisture equipment within the completed interval,
   using start-of-application vapor, wall liquid, the current lumped cabin temperature and
   the interval's exterior temperature; signed bounded exchange; phase energy
   (condensed−evaporated)×REFERENCE_PHASE_J_KG enters the thermal `gross` balance exactly
   like the equipment's condensation heat. Outside the NIST saturation domain the exchange
   is zero with a retained constraint note. New cumulative totals counter
   `wall_condensed_kg` (net) and per-interval `Budget.wall: Option<moisture::WallBudget>`;
   the vapor-store identity gains the wall term.
2. `habitat::LeakConfig` (`cosmogon-cabin-structural-leak/1`): bounded `rate_kg_s`
   (0…100) and `gas_specific_heat_j_kg` (1…1e5). Optional `habitat::Config.leak`. Applied
   each completed interval on start-of-interval cabin gas: leaked =
   min(rate·dt, total gas); each species leaves by its mass fraction; the mixed gas enthalpy
   carried out (leaked × cp × T_cabin) cools the lumped cabin (added to the thermal balance
   as a signed gross debit before the conduction solve). Escaped masses join
   `state.escaped_gas`; new cumulative totals counters `leaked_*_kg` and per-interval
   `Budget.leak: Option<BudgetLeak>`; the breach identity is unaffected because breach
   integration still runs last on the post-leak cabin state. Leak is not breach: no
   pressure, no orifice, no temperature-driven flow curve.
3. State/serialization: new `state.wall_liquid_kg` (`#[serde(default, skip_serializing_if
   is_zero)]`) so legacy saves and zero-wall worlds serialize byte-identically. Optional
   config sections skip when absent (existing pattern). Conditional registry descriptors
   and source pins follow retained optional state (existing humidity pattern).
4. Recordings: `samples.csv` gains optional tail columns `wall_liquid_kg` and
   `total_wall_condensed_kg` plus `leaked_oxygen_kg`, `leaked_nitrogen_kg`, `leaked_co2_kg`,
   `leaked_vapor_kg` and `total_leaked_gas_kg` (cumulative), present only when any retained
   row has wall/leak evidence, so old recordings re-derive byte-exactly (shipped refinement
   of the draft's two-column sketch). `Budget` grows serde-defaulted optional blocks;
   WHY explains wall exchange and leak with the same bounded reader. `habitat-diagnose`
   and the native Humidity pane surface the new retained evidence and staged controls.
5. Validation: config bounds; wall liquid ≤ capacity; totals identity
   `vapor = crew + crop − equipment_condensed − wall_net − breach_escaped` (wall_net =
   condensed−evaporated, signed, added to totals) and
   `wall_liquid = wall_net` (single source); leak totals consistency with
   `escaped_gas` deltas; phase-energy balance entries; save/import round trips; undo/redo;
   refusals (capacity below held wall liquid, invalid bounds, unknown model) leave both
   sides unchanged.

## Verification requirements

Unit tests: signed wall exchange both directions, domain refusal outside 273.15–303 K wall
temperature, film capacity bound, rate bound, phase energy closure, leak proportional
escape, enthalpy cooling closure, leak+breach same interval (identities hold), combined
with equipment moisture. Refusals leave state unchanged; happy paths conserve total mass
exactly (wall exchange internal; leak appears in escaped_gas); save round trips; legacy
recordings re-derive byte-exactly; a recording spanning wall/leak activity validates;
undo/redo restores exact JSON; absent-optional worlds keep the golden `0xf2c4140a45ae509b`.
All release tests, clippy, release builds, inspected native captures and `git diff --check`
remain green.
