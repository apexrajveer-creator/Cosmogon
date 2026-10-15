# Cargo mass in authoritative world flight

Date: 2026-10-15. Model: `cosmogon-vehicle-cargo-mass-epochs/1` (`crates/cosmogon_sim/src/vehicle_cargo_mass.rs`), surfaced in world flight through strictly versioned per-flight cargo ledgers. This closes gap entry 45: cargo mass now affects the **authoritative simulation**, not only the standalone cargo-epochs study.

## Why this matters

Until now a world flight's mass was dry structure plus propellant only; held cargo was authored inventory (`supply_vehicle::Definition.cargo`) with no effect on world dynamics, and nothing could remove it during flight. With cargo ledgers, held cargo is part of the craft's real mass at every integration step, and removals — interactive (UI/undo edit) or authored (timed in the definition) — change that mass through one shared, validated ledger with exact momentum accounting.

## Mechanism

* **Opt-in ledger.** `world_vehicle::Definition.cargo_ledgers` (default off, serde skip-if-false) makes `Flight::new` attach a `vehicle_cargo_mass::Ledger`. Legacy flights carry no ledger, serialize byte-identically to before, and keep the golden `0xf2c4140a45ae509b`.
* **Epoch-correct mass.** Every substep of both world-flight paths (spherical `advance_system` and rigid contact `substep`) binds an *effective definition* from `Ledger::current_definition`, so thrust, mass-flow rate, `inert_mass_kg`, force/mass, inertia and interval evidence all use the current held-cargo epoch. `Flight::mass_kg()` is the single authoritative mass accessor (dry + held cargo + remaining propellant).
* **One shared removal path.** `Flight::remove_cargo(requested, velocity_relative_m_s)` is used by the interactive `Edit::RemoveVehicleCargo` (validate/apply, undoable snapshot) and by authored `Definition.cargo_removals` (applied at the first represented propulsion-clock boundary at or after `at_elapsed_s`; substeps clamp to the remaining gap so the boundary is exact, and any pending removal at a macro end is drained before the clock advances).
* **Momentum closure.** Linear momentum closes exactly: `v_after = (m_before·v_before − m_material·v_after_material)/m_after`. The material velocity is craft velocity plus the bounded relative ejection velocity (`MAX_CARGO_EJECTION_SPEED_M_S = 1000 m/s`); co-moving removal (zero relative) is impulse-free, any non-zero relative velocity is an accounted dissipative ejection.
* **Uniform-gyration layout rule.** Removed material shares the craft's co-rotating per-mass inertia (`gyration_radii_m²·m_removed`), so under this layout the craft's angular velocity is **exactly unchanged**; the removed material carries its angular-momentum magnitude and rotational energy, recorded per event. Eccentric/offset cargo layouts are not representable and no spin-up or spin-down is ever implied.
* **Evidence.** Every removal appends a `CargoEvent` (removal, relative/before/after velocities, angular before/after, carried angular magnitude/energy, mass before/after, authored index). `validate_saved` re-derives momentum closure, mass closure, the angular rule, ledger counters (piecewise capability, propellant closure, interval mass with end-boundary removals added back) and authored-index coverage; tampering fails save validation.
* **Recordings.** Studies with ledgers add a conditional `manifest["cargo"]` block (model, final held, serialized events, authored removals, scope note). `flight-inspect` replays the bundle byte-exactly and prints the same manifest. Interactive removals are save-level evidence only (studies replay authored removals), stated in the manifest scope string.
* **CLI.** `flight-definition --cargo-ledgers` opts a definition into ledgers; the flag is whitelisted for `flight-definition` only.
* **Save pin.** Conditional data-version pin `cosmogon-vehicle-cargo-flight` → `cosmogon-vehicle-cargo-mass-epochs/1` (same strictly-versioned ledger model), added only when a flight or pending creation edit carries ledgers; mismatch or stripped pin refuses reopen.

## Scope and limits

* Removal species mapping is simple for now: the interactive UI draft removes potable water; `Edit::RemoveVehicleCargo` supports any species combination within held inventory and payload capacity.
* Removal changes velocity; it does **not** reorient attitude, change spin direction, attach/detach hardware, or grant landing/delivery. Craft stays `Flying`.
* Co-rotating uniform-gyration layout only; offset cargo centers cannot be represented (refused by construction, not modeled).
* Storage/recipient capacity (who the cargo goes *to*) and habitat transfer are separate future entries; this entry only moves mass out of the flying craft with full accounting.
* Authored removals need `cargo_ledgers` (structural refusal otherwise) and a bounded relative velocity; times must be finite inside `0–1e7 s`.

## Verification

See [the validation record](VEHICLE_CARGO_FLIGHT_VALIDATION.md): 548 release tests plus one existing ignored, no new clippy warnings, fresh CLI release build, end-to-end definition → study → inspect loop with the manifest cargo block and authored mid-study removal, plus negative flag/case checks.
