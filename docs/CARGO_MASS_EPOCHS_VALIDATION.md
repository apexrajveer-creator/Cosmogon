# Cargo mass epochs — research workflow validation

Date: 2026-10-12 (Asia/Kolkata). Source: uncommitted 175-Rust-file working tree on `codex/finite-world-flight` (baseline `c668692` plus the cargo-mass module, sequence workflow, CLI commands, native pane and research registration). This record covers the burn → finite species removal → later burn research workflow only. It does not claim world delivery, flight integration or surface attachment.

## What was verified

1. **Simulation engine and tests.** `crates/cosmogon_sim/src/vehicle_cargo_mass.rs` implements `Plan` (validate/run/export/read), `Bundle` (exact replay), the bounded removal ledger, piecewise ideal propulsion accounting and 39-column `epochs.csv`. Fifteen focused tests cover piecewise rocket-equation comparisons, species and total-mass closure, atomic refusals, serialization corruption, unchanged-ledger behavior, exhaustion, sequence bounds, protected export, exact independent replay and tamper refusal.
2. **Research registration.** `research/registry.rs` registers source `nasa-grc-ideal-rocket-equation` (NASA Glenn ideal rocket equation and specific-impulse references) and model `vehicle-cargo-mass-epochs` with id `cosmogon-vehicle-cargo-mass-epochs/1`.
3. **CLI workflow.** `cargo-epochs-definition` emits an editable example plan; `cargo-epochs-study` runs and exports a protected recording; `cargo-epochs-inspect` independently replays it. Workspace exclusivity rules and path validation reject malformed invocations before dispatch.
4. **Native workflow.** The Supply vehicles window's "Cargo epochs" pane edits steps, calculates in a background worker, renders per-epoch denominators, requested-versus-actual species removal, mass residual and carried momentum/kinetic energy, and exports through the same protected worker. Headless arguments `--cargo-plan`, `--cargo-recording`, `--cargo-output`, `--cargo-observation` and `--panel cargo-epochs` drive it; preflight refuses missing plan/observation inputs and existing output folders.

## Release checks

| Check | Result |
| --- | --- |
| `cargo test --release --workspace` | exit 0 — **532 passed, 0 failed, 1 existing ignored** (simulation 308, app 130, CLI 23, core 32, physics 38, physics doctest 1) |
| `cargo clippy --release --workspace --all-targets` | exit 0 — no warnings in any touched file (`vehicle_cargo_mass.rs`, `vehicle_cargo.rs`, `vehicle.rs`, `args.rs`, `ui/mod.rs`, CLI `main.rs`, registry, `supply_vehicle.rs`); remaining warnings are pre-existing in untouched code |
| `cargo build --release -p cosmogon -p cosmogon_cli` | both release binaries built fresh |

## Executed cases

CLI (against `/tmp/cargo-epochs-run`):

| Case | Expected | Observed |
| --- | --- | --- |
| `cargo-epochs-definition --output NEW_FILE` | writes editable plan | exit 0, plan written |
| `cargo-epochs-study --definition plan --output NEW_DIR` | runs 5-step example (2 removals), writes `plan.json`, `observations.json`, `epochs.csv`, `manifest.json` | exit 0, manifest reports 5 steps / 2 removals / 19 observations |
| `cargo-epochs-study` rerun into the same folder | refuses (protected) | exit 1, "Existing cargo-epochs recordings are protected; choose a new folder" |
| `cargo-epochs-inspect --recording DIR` | exact independent replay | exit 0 |
| `cargo-epochs-inspect` after incrementing one stored `vehicle_mass_kg` | refuses (replay mismatch) | exit 1, "Stored cargo-epochs trajectory differs from an independent exact replay" |

Native (`target/release/cosmogon`, headless `--capture`):

| Case | Expected | Observed |
| --- | --- | --- |
| `--cargo-output` without plan/recording | preflight error, no capture | exit 1, "Cargo-epochs output/observation requires an explicit --cargo-plan or --cargo-recording" |
| `--cargo-plan` into existing output folder | preflight error, no capture | exit 1, "Cannot open cargo mass epochs: Existing cargo-epochs outputs are protected; choose a new folder" |
| `--panel cargo-epochs` | pane opens, example plan auto-calculates | exit 0, `/tmp/cargo-pane.png` inspected: tab active, steps and species fields rendered |
| `--cargo-plan plan --cargo-output NEW_DIR` | worker exports protected recording | exit 0, `/tmp/cargo-export.png` inspected, all four export files written; `cargo-epochs-inspect` replays the native export exactly |
| ordinary `--capture` without cargo arguments | no regression | exit 0, `/tmp/normal-run.png` written |

## Known limits

This workflow is an independent mass-accounting bench. Removal is central and comoving: it carries material linear momentum and translational kinetic energy out of the ledger and never grants a free velocity change, exhausts propellant or edits a world. The model has no angular momentum, inertia distribution, plumbing, heat, storage, handling, recipient or habitat-exchange coupling, is not integrated into `world_vehicle::Flight`, and creates no settlement stocks. It cannot support a delivery claim. The next dependencies remain: solver-integrated supported surface-operation duration, flight mass/inertia integration, finite handling and recipient capacity, and atomic co-located habitat transfer ([next vehicle-support work](VEHICLE_SUPPORT_NEXT.md)).

No push, tag or release was made for this source.
