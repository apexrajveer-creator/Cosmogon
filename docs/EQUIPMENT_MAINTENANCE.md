# Equipment calibration and scheduled maintenance (build 54)

Cosmogon now lets every powered cabin life-support component wear down under use and be
restored in scheduled maintenance windows, with an authored as-found capacity multiplier
applied to its nominal rating. The mechanism is optional and default-absent, so existing
worlds, saves, recordings and the world golden stay byte-identical.

Model pin: `cosmogon-cabin-maintenance/1` (`crates/cosmogon_sim/src/habitat_maintenance.rs`).

## What is modelled

Enabled per habitat via the optional `maintenance` configuration block with one entry per
component — scrubber, recycler, heater, radiator, moisture condenser, electrolysis, Sabatier
(the canonical order shared by state, budgets and effects; crops keep their own maintained-
chamber assumption). Each component carries:

* `wear_per_hour` — condition fraction lost per operating hour, proportional to the whole
  slot's served power fraction.
* `repair_per_hour` — condition fraction restored per maintenance hour (must exceed zero, so
  a window of finite length always makes progress).
* `interval_hours` / `duration_hours` — a repeating cycle of operating time followed by a
  maintenance window. Each simulation interval intersects `[cycle, cycle+dt)` with the two
  phases exactly, including wrap-around.
* `maintenance_power_w` — ordinary bus electricity drawn only during the window.
* `capacity_calibration` ∈ (0, 1] — an authored as-found multiplier on the component's
  nominal rating. Values above 1 are refused because rated process ceilings are the modeled
  maximums (sabatier/moisture rate identities assume them).

## How a step runs

Two phases bracket the ordinary power allocation:

1. **Plan** (read-only, start-of-interval state): for each configured component,
   `scale = capacity_calibration × condition`, the operating/maintenance share of the
   interval, and the time-prorated maintenance power request. Process requests become
   `process × op_fraction + maintenance_power × maint_fraction`; a maintenance window
   therefore replaces process demand with maintenance demand in the same ordinary bus
   allocation, and its electricity becomes cabin heat through the existing ledger.
2. **Settle** (after allocation): each component's condition changes by
   `wear × served × operating_h` and `repair × maintenance_h`, clamped to [0, 1]; the cycle
   position advances. Component effects scale scrubber capture, recycler streams, moisture
   plan and heater/radiator limits for the interval; a windowed component produces nothing
   and draws only maintenance power.

Evidence: per-component held `state.maintenance` condition and cycle position, per-interval
`budget.maintenance` accounting (condition before/after, operating/maintenance seconds,
served fraction, scale, wear, restored), cumulative `totals.maintenance_hours`, save pins,
diagnostics ("maintenance" explanation), Maintenance-pane readouts and staging controls, and
four optional `samples.csv` tail columns present only when maintenance evidence exists.

## Using it

* Native: Habitat → **Maintenance** pane (Overview / Settings / Model & sources). Settings
  stages the `maintenance` block with per-component enable checkboxes and the six numbers
  above; disabling freezes held condition and cycle position, re-enabling resumes them.
* CLI: definitions are ordinary habitat config JSON (`maintenance` block;
  `habitat-definition` presets remain default-absent). `habitat` studies and
  `habitat-diagnose` explain the retained evidence; saves pin the model whenever any
  habitat uses it.
* Component-without-equipment configurations are refused (moisture/electrolysis/sabatier
  require their equipment; scrubber, recycler, heater and radiator are always present).

## Deliberate limitations

Wear, repair rates and calibration are explicit engineering assumptions, not measured
hardware reliability or field calibration. No stochastic failures, spares inventory, crew
labour, consumable mass, age curves beyond linear scaling, or maintenance of solar, battery,
regulator, crop or storage hardware. Numerical conservation does not establish predictive
accuracy.
