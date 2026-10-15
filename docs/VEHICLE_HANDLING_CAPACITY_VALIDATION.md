# Verification evidence — finite handling equipment and recipient storage capacity

Build 46 of the supply sequence, verified on 2026-10-16. Scope and limits are in
[VEHICLE_HANDLING_CAPACITY.md](VEHICLE_HANDLING_CAPACITY.md).

## Release tests

`cargo test --release --workspace`: **562 passed, 0 failed, 1 existing ignored**
(130 app + 23 CLI + 32 core + 38 physics + 338 sim + 1 doctest; +14 over the 548-test
build-45 checkpoint):

* `vehicle_handling` (9): equipment bounds (model, per-species/total/oxygen-interface,
  power, thermal, window), structural refusals (ledgers, operation hardware, equipment
  version), preflight refusals (missing window evidence, short window, different body,
  missing equipment/ledgers/ledger evidence/flight phase, nonnegative finite requests),
  the deterministic acceptance matrix (held/per-species/total-trim/shared-oxygen split
  accepted 20/60/20/0 kg from a 100/100/30/20 kg request with rejected remainder and a
  pure, unmoved ledger), recipient free-space clamping (150 kg tank at 140 kg held accepts
  only 10 of 100 kg requested water) and full acceptance within all limits. The rest-world
  setup lands a sheared contact craft on Mars beside a habitat and advances the real solver;
  the asserted window (≥ 5 s) is genuine supported-operation evidence, never manufactured
  ledger state.
* `habitat` (4): default tanks skipped in serialization and restored by serde default;
  authored non-default tanks round-trip; initial stock above tank refused; held stock above
  active tank refused at validate; reconfiguration below held stock refused with "no mass was
  discarded"; `available_cargo` = capacity − held; O₂ tank below installed electrolysis
  oxygen capacity refused.
* `save` (1): `handling_pin_is_conditional_and_enforced` — no handling pin on a plain world;
  pin present after creating a handling craft; reopen round-trips; stripped pin refuses
  reopen; mismatched pin refuses reopen.

The released serialized-world golden remains **`0xf2c4140a45ae509b`** (unre-pinned: all new
fields are optional with skipped defaults, and the potable-recovery clamp never binds at the
default 1e7 kg tank, so legacy trajectories and fingerprints are unchanged).

## Clippy / builds

`cargo clippy --release --workspace --all-targets`: no warnings in any touched file
(pre-existing warnings elsewhere unchanged: nbody.rs:428, save.rs:26 complex type, dock.rs,
block 0.1.6, "items after a test module" in terrain.rs/research.rs). Fresh app and CLI
release builds succeed.

## End-to-end CLI (`/tmp/op46e2e/`)

1. `def46.json` (build-45 contact+operation study plus an authored `handling` block):
   `flight-study` succeeds; `manifest["handling"]` carries the model, all authored limits,
   the final supported window and the authorize-only scope; `operation_window` and `cargo`
   blocks remain.
2. `def46_land.json` (the build-43 geographic-landing definition plus `cargo_ledgers` and
   `handling`): `flight-study` succeeds with a **real supported window of 9.38671875 s ≥ 5 s
   required** (5800 supported samples), handling window evidence ≥ 5 s and the retained
   cargo ledger (20/100/30/20 kg). `flight-inspect` replays and retains the handling
   equipment and cargo ledger exactly.
3. Legacy stability: the original build-45 `def_contact.json` still studies byte-stably and
   its manifest gains **no** `handling` key.
4. Negatives, all refused with their exact messages:
   * handling without cargo ledgers → "Handling equipment requires cargo ledgers"
   * handling without operation hardware → "Handling equipment requires authored contact
     surface-operation hardware"
   * wrong equipment model → "Unsupported surface handling model version"
   * window requirement of 1e9 s → "Handling required operation window must be finite and
     between 0 and 1e7 s"

## Boundaries

No transfer, receipt, delivery, attachment or phase change occurs; the preflight is pure
accounting and is exercised in engine tests (CLI has no combined habitat+craft command yet —
that arrives with the build-47 transfer `Edit`). Declared power/thermal values remain
unmodeled assumptions. Co-location beyond same-body, species-specific atomic edits, undo/
save/replay of transfers and momentum/energy/heat transfer evidence remain build 47.
Graphics remain queued.
