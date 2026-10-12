# Cargo inventory and changing-mass propulsion — foundation component

The next physical-delivery mechanism must remove actual held cargo without rewriting the original vehicle design or previously consumed fuel. Surface-operation evidence, co-location, finite recipient storage/handling and habitat external-material/heat ledgers are still required before any world transfer can be enabled. This document records the architecture being implemented; it does not claim delivery exists.

## Evidence and scope

NASA Glenn's [ideal rocket equation](https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/ideal-rocket-equation/) derives the constant-effective-exhaust-velocity result from momentum conservation and changing mass. Its constant empty-mass assumption applies within an interval, not across a separate removal of payload. The [specific impulse reference](https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/specific-impulse/) gives the thrust/mass-flow relationship used by the existing propulsion model. Checked 2026-10-10. Neither source validates our cargo handling, gear, storage or mission feasibility.

Engineering inference: partition a flight's ideal propulsion accounting into cargo-mass epochs. Sum the fuel-driven logarithmic velocity changes within each epoch. Removing cargo is a distinct material event; it neither exhausts propellant nor generates rocket impulse. The old single initial-to-final logarithm is invalid once cargo is removed between burns.

## Implementation requirements

Keep immutable initial Cargo and vehicle Definition alongside separate held and removed four-species inventories. Record event clock, propellant at that clock and actual per-species removed amounts; reject negative, nonfinite, out-of-order, unrepresented or overdrawn changes atomically. Food-bound water, potable water, dry food and reserve oxygen remain distinct. No untracked disappearance or unsolicited grants.

Future burns use the current held mass while prior epochs retain their original denominators. Validate both ordinary finite-fuel counters and the independently recomputed sum across epochs. Dry hardware/packaging stays aboard. A stated central, comoving removal carries material momentum/kinetic energy away without imparting a free velocity change; angular momentum, inertia distribution, rotational energy, plumbing impulses and articulated handling require explicit later coupling. This component accounts only for carried linear momentum and translational kinetic energy; it cannot yet update a rotating craft.

Connect this ledger to authoritative contact flight only after versioned save/replay support, the supported operation interval and habitat exchange ledgers are ready. Legacy untouched flights must retain identical serialization, dynamics and golden state. A standalone mass-accounting component is not a completed supply chain.

## Implemented component status — 2026-10-12

`crates/cosmogon_sim/src/vehicle_cargo_mass.rs` implements the bounded four-species held-cargo ledger and piecewise ideal propulsion accounting described above. It retains the exact original `supply_vehicle::Definition`, actual requested/removed species quantities and event mass, fuel, clock, comoving linear momentum and translational kinetic-energy evidence. Future burns use current held cargo; earlier intervals retain their historical mass denominators.

The same module now provides the shared research workflow used by both the CLI and the native application. An authored `Plan` (immutable vehicle definition, observation interval, ordered burn/removal steps, validated against 256 steps and 8192 observations) runs into a `Bundle` of 39-column observations. Protected export writes `plan.json`, `observations.json`, `epochs.csv` and `manifest.json` (format `cosmogon-cargo-epochs-recording`) and refuses existing folders. Reopening via `Bundle::read` re-runs the plan and compares every observation byte-for-byte against the stored recording before displaying or re-exporting anything; any drift is refused. The research registry registers source `nasa-grc-ideal-rocket-equation` and model `vehicle-cargo-mass-epochs` (`cosmogon-vehicle-cargo-mass-epochs/1`).

User-facing surfaces:

* CLI: `cargo-epochs-definition` emits an editable example plan; `cargo-epochs-study --definition PLAN --output NEW_DIR` runs and exports; `cargo-epochs-inspect --recording DIR` independently replays. Each refuses to overwrite existing outputs.
* Native: the Supply vehicles window gained a ninth pane, "Cargo epochs", which edits the step sequence in place, calculates in a background worker, shows per-epoch denominators, actual-versus-requested species removal, mass residual and carried momentum/kinetic-energy evidence, and exports through the same protected worker. `--cargo-plan`/`--cargo-recording`/`--cargo-output`/`--cargo-observation`/`--panel cargo-epochs` drive it headlessly; preflight rejects missing plan/observation inputs and existing output folders before the app starts.

Scope limits are unchanged: this is a mass-accounting bench. It is not a field on `world_vehicle::Flight`, grants no impulse on removal, touches no world body, creates no settlement stocks and implements no habitat exchange. Surface-operation windows, flight mass/inertia integration, finite handling/storage and atomic co-located transfer remain the next dependencies.

Verification for this 175-file source tree: `cargo test --release --workspace` passes **532 tests with one existing ignored** (simulation 308 including 15 cargo-mass tests, app 130 including 2 pane tests, CLI 23, core 32, physics 38 plus one doctest); `cargo clippy --release --workspace --all-targets` exits 0 with no warnings in touched files; fresh release builds of `cosmogon` and `cosmogon-cli` succeed; the full CLI definition/study/inspect loop, protected rerun refusal and tampered-observation refusal were executed; native `--capture` images for the pane, the export path and an ordinary run were inspected; the native export replayed exactly via `cargo-epochs-inspect`. Full evidence is in [the validation record](CARGO_MASS_EPOCHS_VALIDATION.md). No physical cargo delivery is implemented. Continue with solver-integrated supported surface-operation duration and the [next vehicle-support work](VEHICLE_SUPPORT_NEXT.md).
