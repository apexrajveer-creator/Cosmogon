# Verification evidence — atomic co-located habitat transfer

Build 47 of the supply sequence, verified on 2026-10-16. Scope and limits are in
[VEHICLE_TRANSFER.md](VEHICLE_TRANSFER.md).

## Release tests

`cargo test --release --workspace`: **575 passed, 0 failed, 1 existing ignored**
(130 app + 24 CLI + 32 core + 38 physics + 350 sim + 1 doctest; +13 over the 562-test
build-46 checkpoint):

* `vehicle_transfer` (9): full acceptance with real solver window, zero separation and
  retained transfer/receipt/zero-relative cargo event; window refusal below the authored
  requirement; reach refusal beyond `reach_m`; rate trim (end-first species order, trimmed
  mass stays on the craft, accepted + rejected == requested); tank ulp clamp
  (`held + accepted ≤ tank` exactly); "accepts no material" refusal; no-ledgers and
  missing-equipment refusals (the build-46 preflight gates); save round-trip byte-exact with
  the conditional `cosmogon-vehicle-transfer` pin absent on legacy worlds; tampered retained
  records refused on reopen. The landed-world fixture places a sheared contact craft at the
  habitat site on real MOLA height; every asserted window is solver evidence.
* `world_vehicle_recording` (2 new): a study with an authored transfer applies the shared
  `Edit` at the sample boundary, records the manifest `transfers` block and `transfers.csv`,
  and replays exactly through `Bundle::run`/export; studies refuse missing habitats and
  out-of-range sample indices up front.
* `save` (1 new): `transfer_pin_is_conditional_and_enforced` — no pin on a plain world; pin
  present after a transfer record or receipt exists (or after the transfer `Edit` is journaled);
  reopen round-trips; stripped pin refuses reopen.

The released serialized-world golden remains **`0xf2c4140a45ae509b`** (unre-pinned: all new
fields are optional with skipped defaults, so legacy trajectories and fingerprints are
unchanged).

## Clippy / builds

`cargo clippy --release --workspace --all-targets`: no warnings in any touched file
(pre-existing warnings elsewhere unchanged: nbody.rs:428, save.rs:26 complex type, intervene.rs,
persistence.rs, dock.rs, block 0.1.6, "items after a test module" in terrain.rs/research.rs).
Fresh app and CLI release builds succeed.

## End-to-end CLI (`/tmp/op47e2e/`)

Fixture chain: `habitat-setup --target Mars --latitude 0 --longitude 0 --seed 1` →
`world.cosmo`; `contact-hardware --static-shear --operation-window 5` (build-46 fixture
`start.json`); `flight-definition --initial world.cosmo --contact-hardware … --geographic-start
… --cargo-ledgers --handling --samples 25 --interval-seconds 1` → `study47.json`; one authored
transfer injected at sample 23 (all held cargo: 20/100/30/20 kg).

1. `flight-study --definition study47.json --initial world.cosmo`: **full 170 kg accepted, 0
   kg rejected**, real supported window **12.38671875 s ≥ 5 s required**, surface separation
   0.0 m ≤ 2000 m reach, rate envelope 619.3359375 kg, craft mass 2170 → 2000 kg, ground
   absorbed momentum/KE evidence retained; manifest `transfers` block (authored/applied/
   habitats+receipts/scope) and `transfers.csv` present.
2. `flight-inspect --recording out47`: exact independent replay, `complete: true`, the
   transfer record verified.
3. Legacy stability: studies without `transfers` gain no `transfers` key and no
   `transfers.csv`; the handling manifest scope only switches to "consumed by the retained
   transfer records" when transfers exist.
4. Negatives, all refused with their exact messages (or the exact clamp):
   * transfer before the 5 s window exists → "Handling requires a continuous
     supported-operation window of 5 s (current 0 s)"
   * transfer to a missing habitat id → "Authored transfer references a missing habitat"
   * craft without `--handling` → "Handling preflight requires authored handling equipment"
   * handling without cargo ledgers → "Handling equipment requires cargo ledgers" (refused at
     definition time, before any study)
   * habitat 4 732 634.8 m away → "Transfer requires the craft ground site within 2000 m of
     the habitat site (current 4732634.805152617 m)"
   * request beyond held cargo → clamped to held (20 kg O₂ accepted, 4980 kg rejected, stays
     on the craft)
   * full potable tank (250 kg capacity, ~200 kg held, 100 kg requested) → accepted
     49.99914463425927 kg, rejected 50.00085536574073 kg; both sides record the same accepted
     split and the habitat passes its exact capacity validation.

## Boundaries

Transfer is instantaneous and co-moving; declared power/thermal values remain unmodeled
assumptions; reach is the modeled sphere only (no terrain or obstruction); ground-absorbed
momentum/energy is retained evidence, not structural physics. No attachment, phase change or
automatic delivery occurs. Detailed body/terrain/civilization graphics remain queued.
