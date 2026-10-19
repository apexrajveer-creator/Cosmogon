# Equipment calibration and maintenance — preimplementation design

Prepared 2026-10-19 after wall-condensation/leak checkpoint `2c7054f`. Build 54 of the master
settlement sequence ([BUILD_TRACKER](BUILD_TRACKER.md) row 54, phase-9 remainder). The directive's
settlement list includes maintenance; phase-9 gap tables name equipment calibration/reliability as
unimplemented. This build adds the first reduced, deterministic wear/maintenance/calibration model
for powered cabin equipment. It does not claim empirical hardware calibration (build 56's subject).

## Problem

Scrubber, recycler, heater, radiator, moisture collector, electrolysis and Sabatier currently run
at fixed rated capacity forever. There is no condition state, no scheduled servicing and no way to
author an as-found capacity derating versus the datasheet rating. Settlement engineering needs
equipment that degrades with use, is restored by explicit maintenance windows with a real power
cost, and carries an authored calibration factor — all deterministic and save-pinned.

## Model

New module `habitat_maintenance.rs`, pinned `cosmogon-cabin-maintenance/1`. Optional
`Config.maintenance: Option<MaintenanceConfig>`; absent config means today's behavior exactly
(golden-safe). Seven component slots in canonical order: scrubber, recycler, heater, radiator,
moisture, electrolysis, sabatier. Crop keeps its separate maintained-chamber assumption.

`ComponentMaintenance` (per configured component):
- `wear_per_hour` — condition lost per operating hour at full served power fraction,
  in [0, 1000].
- `repair_per_hour` — condition restored per maintenance hour, in (0, 1e6].
- `interval_hours` — operating hours between maintenance windows, in
  [60 s, 1e6 h] expressed in hours (>= one 60 s scheduler interval, so a window boundary
  crosses at most one interval edge per cycle).
- `duration_hours` — maintenance window length, in (0, same upper bound].
- `maintenance_power_w` — power drawn only during the window, in [0, 1e6].
- `capacity_calibration` — authored as-found capacity multiplier on the nominal rating,
  in (0, 1]. Overperformance (>1) is refused because rate identities elsewhere assume
  rated ceilings.

`MaintenanceState` holds `Option<ComponentCondition { condition, cycle_s }>` per component;
`condition` in [0, 1], `cycle_s` the position inside the repeating cycle
`interval_hours*3600 + duration_hours*3600`. Fresh state starts at condition 1.0, cycle_s 0.

Each 60 s interval, per configured component, `advance()` splits `dt` exactly into operating
time `op` and maintenance time `maint` by intersecting `[cycle_s, cycle_s+dt)` with the two
phases (wrap handled; interval validation guarantees at most one wrap for accepted dt). Then:
- effective scale = `capacity_calibration * condition` (start-of-interval condition; ≤ 1).
- wear = `wear_per_hour * served_fraction * op / 3600`, clamped so condition ≥ 0.
- restore = `repair_per_hour * maint / 3600`, clamped so condition ≤ 1.
- cycle_s advances by dt modulo the cycle length.

Step wiring (process power slots keep their priority order; each slot's request is prorated
`process*(op/dt) + maintenance_power*(maint/dt)`):
- Scrubber capture rate, recycler throughput, moisture plan, electrolysis planned oxygen and
  Sabatier planned CO₂ are scaled by the component scale; process energy consistency holds
  because a slot's served fraction applies uniformly to its prorated request.
- Heater limit and radiator rejection are multiplied by their scales; the radiator pump
  request is prorated the same way.
- Maintenance power is ordinary delivered bus electricity and therefore becomes cabin heat
  through the existing ledger; no new heat path, mass path or Totals mass counter is added.
- `Totals.maintenance_hours` accumulates total component maintenance time (hours).

Budget: `Budget.maintenance: Option<MaintenanceBudget>` with one `Option<ComponentBudget>` per
component (`condition_before/after`, `operating_s`, `maintenance_s`, `served_fraction`,
`effective_scale`, `wear`, `restored`). Save pins `cosmogon-cabin-maintenance` conditionally via
`uses_maintenance()`; readers reject missing/mismatched pins. Recordings accept the four new
aggregate CSV tail columns (`maintenance_operating_s`, `maintenance_active_s`,
`worst_equipment_condition`, `total_maintenance_hours`) → widths 114/116/119/121 in the
wall/leak combination matrix. Diagnostics gain per-component condition/phase explanations;
native UI gains a Maintenance tab (Overview / Settings / Model & sources) following the
humidity/breach sub-pane pattern.

## Explicit non-goals

No stochastic failures, no spares inventory, no crew labor mass, no consumables, no
performance-vs-age curves beyond linear condition scaling, no maintenance of solar array,
battery, oxygen regulator, crop equipment or storage tanks, no claim of measured hardware
reliability or field calibration. Condition affects achievable throughput only; worn equipment
still draws its nominal process power until the maintenance window replaces it.

## Acceptance

Independently computed wear/restore/cycle arithmetic against `ComponentBudget`; prorated
power requests and scaled process outputs with identities (mass/electric/thermal residuals)
unchanged; refused invalid parameters and component-without-equipment configs leave
configuration and state untouched; absent maintenance config preserves legacy behavior and
golden `0xf2c4140a45ae509b`; exact save/replay with model-pin tamper refusal; deterministic
repeat; CSV tail widths; actual native Maintenance-tab capture; full release tests, clippy and
ordinary packaged run. No uncertainty, field-calibration or master-completion claim.
