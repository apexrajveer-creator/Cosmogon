# Solver-integrated surface-operation window — validation

Date: 2026-10-15 (Asia/Kolkata). Source: uncommitted working tree on `codex/finite-world-flight` (baseline `5943ead` plus the operation-window module, ledger/recording wiring, CLI flag and detailed shear refusal diagnostics). This record covers the continuous supported-operation measurement only. It does not claim attachment, phase change, storage, handling or delivery.

## What was verified

1. **Simulation engine and tests.** `crates/cosmogon_sim/src/vehicle_operation.rs` defines `Definition` (envelope plus `required_window_s`, `maximum_sample_gap_s`, per-foot utilization caps) and `Ledger` (continuous window with origin clocks, counters and bounded last-exclusion). Seven focused tests cover explicit bounds, a resting craft accumulating then resetting the window, same-clock and definition-mismatch refusals, whole-macro-step rollback on solver failure, save-roundtrip pin enforcement, protected operation-CSV export and legacy absent-ledger/pin behavior.
2. **Solver sampling.** `vehicle_contact_dynamics.rs` evaluates the authored envelope at both the accepted midpoint and endpoint of every passing substep through `Reading::from_flight`; any later failure in the same one-second macro interval discards that interval's samples with the committed state.
3. **Recording and reopen.** `world_vehicle_recording.rs` writes `operation.csv` (21-column per-sample ledger growth), pins `manifest.operation_window`, and `flight-inspect` replays the recording and reports the window independently.
4. **CLI workflow.** `contact-hardware --operation-window SECONDS --geographic-start FILE` authors the envelope on the hardware; `flight-definition --contact-hardware HW --geographic-start FILE --definition coast.json` builds a world flight at the geographic start; `flight-study` runs it; `flight-inspect` verifies. Four CLI arg tests cover the flag value, missing value, negative value and the flight-definition refusal.

## Release checks

| Check | Result |
| --- | --- |
| `cargo test --release --workspace` | exit 0 — **539 passed, 0 failed, 1 existing ignored** (simulation 315, app 130, CLI 23, core 32, physics 38, physics doctest 1) |
| `cargo clippy --release --workspace --all-targets` | exit 0 — no warnings in touched files; remaining warnings are pre-existing in untouched code |
| `cargo build --release -p cosmogon_cli -p cosmogon_sim` | release CLI built fresh |

## Executed end-to-end cases

Fixture chain in `/tmp/op2e2e`: `start.json` (lat 0/lon 0, ENU velocity zero, `spherical_altitude_m = -1336.494657` giving a **5.03 mm initial foot gap** at the 4ppd MOLA site whose authoritative height is −1338.0 m), a coast supply-vehicle plan (single zero-throttle 3600 s command) and fresh output paths.

| Case | Expected | Observed |
| --- | --- | --- |
| `contact-hardware --static-shear --operation-window 5 --geographic-start start.json --output hw3.json` | hardware carries the operation envelope | exit 0, `operation` block with required window 5 s |
| `flight-definition --contact-hardware hw3.json --geographic-start start.json --definition coast.json --samples 21 --interval-seconds 1 --output def4.json` | definition at the geographic start with retained coast plan | exit 0, fingerprint `fnv1a64:a5877f477368033f` |
| `flight-study --definition def4.json --output study6` (fresh folder) | craft lands, stays `flying`, window accumulates | exit 0 — **phase `flying` throughout; `supported_samples` 5800 of 10240; `current_window_s` `longest_window_s` 9.38671875 ≥ 5.0 required; `last_exclusion` null; manifest `operation_window` non-null** |
| `flight-inspect --recording study6` | exact independent replay, window reported | exit 0, `complete: true`, window 9.38671875 s, `operation.csv` in the verified file set |
| `contact-hardware --operation-window -1` | refuses non-finite window | exit 1, "requires a finite window between 0 and 1e7 seconds" |
| `flight-definition --operation-window 5` | flag is contact-hardware-only | exit 1, "not a world flight definition option" |
| earlier study with the default burning plan (150 s full throttle) | envelope must detect thrust-driven sliding | every sample excluded by "Ground-relative translation exceeds the declared limit"; ledger stayed at zero — correct exclusion of a craft pushed at 10 kN against ~4 kN friction capacity |

The accepted run's first ≈9 s are the underdamped vertical landing transient (modeled damping ratio ≈ 0.09); exclusions during that interval are rotation/acceleration envelope trips, after which the window opens and grows monotonically to the end of the study. Contact CSV at rest shows **1961–2067 N per foot** and **−0.133 m compression**, matching the independent static prediction (mass ≈ 2185 kg, g = 3.71 m/s², stiffness 15000 N/m ⇒ ≈2013 N and ≈0.134 m per foot) to within the slope-driven load redistribution.

## Diagnosis retained as a diagnostics improvement

`State::validate` in `vehicle_shear.rs` now reports the specific failing quantities (`shear_length_m=… max=…`, `coulomb_load_n=… friction_capacity_n=…`, …) inside the unchanged refusal prefix, so an unsupported flight's stored reason states why. This was the instrument that identified the earlier 0.5028 m shear trip (foot load ≥ 5028 N ⇒ friction radius > 0.5 m). Failure conditions are unchanged; no test or doc asserted the previous bare message.

## Numerical and scenario findings

* The initial-gap sensitivity is real and physical: a 10 cm drop reaches ≈5 kN peak per-foot load (mass·(compression + v/ωn), ωn ≈ 5.24 rad/s), straddling the 5028 N shear-stability boundary; a 5 mm start reaches ≈2.4 kN and cannot trip it. The e2e fixture therefore starts at millimeter clearance rather than asserting engine changes.
* Frame semantics were audited before the retune: `Surface::sample`, `Foot::sample` and the `substep` half-step all compare inertial-minus-inertial velocities; the earlier phantom-slip hypothesis was ruled out and the 0.5028 m strain was real physical slip under excessive load.
* The operation envelope rejected the default thrust-accelerated scenario completely (see cases), confirming it measures actual support rather than trusting the commanded plan.

## Known limits

The window is a modeled numerical envelope at solver sample clocks, not proof of continuous hardware stability; `maximum_sample_gap_s` bounds the assumed coverage. `longest_window_s` is historical evidence and grants nothing. The flight stays `flying`: no attachment, freeze, landing grant, cargo permission, storage, handling, power, thermal or delivery model exists, and nothing consumes `required_window_s` yet. The e2e fixture is a flat-site engineering scenario on real MOLA height with authored zero ENU start, not a landing trajectory or hazard survey. Legacy flights keep an absent ledger and no save pin; the released serialized-world golden remains `0xf2c4140a45ae509b`. No push, tag or release was made for this source.
