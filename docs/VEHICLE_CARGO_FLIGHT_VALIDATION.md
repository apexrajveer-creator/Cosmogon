# Cargo mass in world flight — verification record

Date: 2026-10-15. Companion to [VEHICLE_CARGO_FLIGHT](VEHICLE_CARGO_FLIGHT.md) (gap entry 45).
Host: macOS 26.3.1, arm64.

## Release workspace suite

* `cargo test --release --workspace` exited 0: **548 passed, 0 failed, 1 existing ignored**
  (build 2 checkpoint was 539; +9 net: 7 new `world_vehicle` tests, 1 new contact test,
  1 new save-pin test).
* `cargo clippy --release --workspace --all-targets` exited 0; no warnings in any touched file
  (remaining warnings pre-existing: core math, physics, older sim/UI/persistence, `block 0.1.6`
  future-compat, `nbody.rs:428` unused `mut`).
* Fresh `cargo build --release -p cosmogon_cli` succeeded.
* Golden simulation fingerprint unchanged: `0xf2c4140a45ae509b` (golden test in suite).

## New automated coverage (9 tests)

1. `co_moving_cargo_removal_is_impulse_free_and_closes_momentum` — zero-relative removal keeps
  craft velocity (within 1e-9 relative), mass drops by exactly the removed amount, ledger held
  and event count correct, full save validation passes.
2. `bounded_ejection_closes_linear_momentum_exactly` — non-zero relative ejection: craft velocity
  changes, linear momentum `m_after·v_after + m_removed·v_material` closes to `m_before·v_before`
  within 1e-6.
3. `cargo_removal_refuses_missing_ledgers_overdraw_unbounded_velocity_and_non_flight` — four
  refusals: no ledger, overdraw beyond held inventory, relative speed above
  `MAX_CARGO_EJECTION_SPEED_M_S`, and non-`Flying` phase.
4. `tampered_cargo_evidence_fails_saved_validation` — flipping a `CargoEvent` velocity makes
  `validate_vehicle_state` refuse the world.
5. `authored_cargo_removals_apply_at_represented_boundaries` — authored removal at 0.5 s lands at
  a represented craft-clock boundary (elapsed ≥ 0.5), held drops 3000 → 2600 kg,
  `authored_index` recorded, save validation passes.
6. `ledger_flights_expose_cargo_keys_and_legacy_flights_stay_byte_exact` — a plain flight's
  serialized object has no `cargo`/`cargo_events` keys; a ledger flight has `cargo` (and
  `cargo_events` once non-empty); ledger worlds round-trip through save and revalidate.
7. `definition_structure_rejects_removals_without_ledgers_and_unsupported_inputs` — structural
  refusals: authored removals without ledgers, NaN time, relative speed above bound, NaN cargo.
8. `co_rotating_cargo_removal_preserves_angular_velocity_and_closes` (contact) — spinning craft
  (0.2 rad/s z): co-rotating removal leaves `angular_velocity_body_rad_s` exactly unchanged,
  carried angular magnitude/energy positive, spin still unchanged after one second of contact
  advance, save validation passes.
9. `cargo_flight_pin_is_conditional_and_enforced` (save) — plain world header has no
  `cosmogon-vehicle-cargo-flight` pin; ledger world has pin = `cosmogon-vehicle-cargo-mass-epochs/1`;
  stripped or mismatched pin refuses reopen.

## End-to-end CLI loop (`/tmp/op3e2e`)

All run against a fresh `target/release/cosmogon-cli`:

| Case | Command | Result |
| --- | --- | --- |
| Ledger definition | `flight-definition --target Mars --cargo-ledgers --samples 4 --interval-seconds 1 --output def1.json` | exit 0, `"cargo_ledgers": true` |
| Plain study | `flight-study --definition def1.json --output study1` | exit 0; manifest `cargo` block: model `cosmogon-vehicle-cargo-mass-epochs/1`, held example cargo, 0 events |
| Authored removal study | `flight-study --definition def2.json --output study2` (removal at 0.5 s, 800 kg water, co-moving) | exit 0; held water 5000 → 4200 kg, 1 event at `elapsed_s` 0.5, `authored_index` 0, masses differ by exactly 800.000000 kg |
| Replay inspect | `flight-inspect --recording study1` | exit 0; manifest byte-equal to `study1/manifest.json` |
| Contact composition | `flight-definition --contact-hardware hw3.json --geographic-start start.json --cargo-ledgers …` then `flight-study` | both exit 0; manifest has `cargo` block **and** `operation_window` together |
| Negative: removals without ledgers | `flight-study --definition def3_noledger.json` | exit 1: `Authored cargo removals require cargo ledgers` |
| Negative: unbounded relative velocity | `flight-study --definition def4_unbounded.json` (4000 m/s) | exit 1: `…bounded relative ejection velocity` |
| Negative: flag misuse | `flight-study --definition def1.json --cargo-ledgers` | exit 1: `--cargo-ledgers is not a world flight study option` |

## What this does not claim

* No storage, recipient, habitat-transfer or delivery model; removal takes material out of the
  flying craft with accounting, nothing receives it.
* No reorientation or attitude control: angular velocity is unchanged by the uniform-gyration
  layout rule, and eccentric cargo layouts are not representable.
* Interactive removals live in save-level evidence only; studies replay authored removals and say
  so in the manifest `scope` string.
* The e2e loop exercises spherical and contact flights; hostile-path removal coverage beyond the
  listed negatives lives in the unit suite.
