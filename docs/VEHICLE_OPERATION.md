# Solver-integrated supported surface-operation window

Date: 2026-10-12. Model: `cosmogon-vehicle-surface-operation/1` (`crates/cosmogon_sim/src/vehicle_operation.rs`). This component closes the gap between instantaneous footprint-support readings and the eventual safe-unloading gate: it measures how long a craft has **continuously** satisfied an authored supported-operation envelope, using the actual accepted contact-solver states rather than low-speed heuristics or sparse recorded endpoints.

## Why solver states, not heuristics

A slow craft or a handful of recorded endpoint samples do not prove a craft can safely remain on the surface: transient contact, source discontinuities, force imbalance and stroke/structural saturation can hide between sparse observations. The contact solver in `vehicle_contact_dynamics.rs` already advances rigid terrain contact with explicit-midpoint substeps, and every one-second macro interval commits atomically only after all substeps validate. Each accepted substep yields two trustworthy rigid states: the **midpoint** (used as the integration velocity/acceleration sample) and the **endpoint** (committed). This component evaluates the authored support envelope at both states and accumulates a continuous window.

## Mechanism

* `vehicle_operation::Definition` (opt-in, authored on `Hardware.operation`): embeds the same `vehicle_support::Definition` envelope used by the instantaneous reading, plus `required_window_s`, `maximum_sample_gap_s` and per-foot structural/stroke/friction utilization caps.
* `vehicle_operation::Ledger` (retained on `Flight.operation_ledger`): origin and last sample craft clocks, `current_window_s`, `longest_window_s` with origins, sample/supported/exclusion counters, a bounded last-exclusion reason, and the `last_sample_supported` continuity flag. Windows extend only across consecutive supported samples; any excluded sample or continuity gap resets the current window to zero at the new sample clock. `longest_window_s` is historical evidence only and never grants current permission.
* Evaluation reuses the actual footprint/hold reading through a new crate-internal `Reading::from_flight`, which evaluates a retained state at its own craft clock **without** the public `Reading::live` assumption that `world.time` has advanced to the craft clock (the solver evaluates states mid-interval, before the scheduler clock moves).
* Samples are taken inside `substep` after the whole substep has passed its own acceptance gates (swept-hull refusal, work balance, finite bounds). The ledger lives on the previewed flight, so any later failure in the same one-second macro interval discards every sample from that interval: the ledger rolls back to the last committed macro state.
* The strictly increasing craft clock, monotonic continuity, finite windows and counter closure are validated on every sample, on retained-save validation and on save reopen.

## Scope and limits

* Checks at numerical solver endpoints and midpoints are a **modeled envelope**, not proof of continuous real hardware stability between samples; `maximum_sample_gap_s` bounds the assumed coverage.
* No phase change: the flight stays `Flying`; no attachment, freeze, landing grant or cargo permission.
* No storage, handling, power, thermal, delivery or habitat-exchange model. `required_window_s` is exposed for a future transfer gate; nothing consumes it yet.
* Legacy flights keep an absent ledger (`serde` default + skip-if-none); worlds without operation hardware gain no save pin, and the released serialized-world golden is unchanged.

## Versioning

Conditional save pin `cosmogon-vehicle-surface-operation` → `cosmogon-vehicle-surface-operation/1`, added to `save.rs::data_versions` when any vehicle (or pending vehicle-creation edit) carries operation hardware, and enforced on reopen like the existing shear pins. Strict tests remove the pin and require refusal; a legacy no-operation world must not contain the pin.

## Verification

See [the validation record](VEHICLE_OPERATION_VALIDATION.md): 539 release tests plus one existing ignored, a seven-case end-to-end loop on real 4ppd MOLA height (20 s `flying`, window 9.38671875 s ≥ 5 s required, 5800 supported samples, replay-exact inspect), negative flag cases and the retained thrust-sliding exclusion case.
