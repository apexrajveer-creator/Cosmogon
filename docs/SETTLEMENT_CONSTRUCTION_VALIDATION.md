# Finite settlement construction and logistics — verification (build 55)

## Automated suite

* `cargo test --release --workspace`: **651 passed, 0 failed, 1 existing ignored**
  (build 54 baseline 639 + 1). Breakdown: cosmogon 131, cosmogon_cli 28, cosmogon_core 32,
  cosmogon_physics 38, cosmogon_sim 421, integration 1, doctests 0. New coverage: the
  construction `Budget::draw` bounds unit test, construction consumption/growth,
  power-off and volume-bound stopping, legacy serialization without construction keys,
  deterministic replay with the additive CSV tail, disable/re-enable freeze, capacity
  below held-stock refusal, and integration tests for the bounded journaled supply edit
  (pin + built volume) and the conserved paired stock transfer (pairing tamper refusal,
  cross-body refusal).
* `cargo clippy`: zero warnings in touched files (all remaining warnings pre-date this
  build: `save.rs` complex-type const, maintenance loop-index, etc.).
* Fresh release builds of `cosmogon` and `cosmogon-cli` after all edits.

## End-to-end loop (`/tmp/b55`)

1. `habitat-setup --target Earth --seed 7` twice (`--initial` on the second) → habitats
   #0 and #1 on Earth (same body).
2. Hand-edited `config.construction` (capacity 1000 kg, rate 0.05 kg/s, 200 W, 0.01
   m³/kg) into habitat #0's live config plus the `cosmogon-cabin-construction` header pin.
3. `habitat-construction-supply --habitat-id 0 --material-kg 400` → held 400.000 kg,
   `construction_observation` in the report, output folder with before-world, action JSON
   and full export.
4. Construction study (`--samples 13 --interval-days 1`): **115-column CSV** with the
   five-column construction tail. Material 400 → 0 kg, cumulative volume +4.000000 m³
   (live cabin 500 → 504 m³), requested/delivered 200.0 W every interval, served 100%.
   The 400 kg held + 400 kg structure = mass conserved exactly.
5. Mid-construction study (`--interval-days 0.005`): material declines 400 → 142 kg over
   the run with per-interval `construction_volume_added_m3 = 0.030000` (= 0.05 kg/s × 60 s
   × 0.01 m³/kg) and cumulative volume 0 → 2.58 m³ — linear, rate-exact growth.
6. `habitat-stock-transfer --from-id 0 --to-id 1 --oxygen-kg 10 --water-kg 30
   --food-dry-kg 5 --food-water-kg 4`: moved == requested on every species; habitat #0
   decreases equal habitat #1 increases exactly (10.0/30.0/5.0/4.0) and both worlds'
   species totals are unchanged (0.0 residual each). Retained records pair: outbound on
   #0 (counterparty 1) and inbound on #1 (counterparty 0), same `world_time_s`
   (821534400.0), equal `moved` masses.
7. Refusals (all exit 1, nothing applied): no species flags → required-args usage error;
   supply 5000 kg > capacity → "Construction material supply exceeds held capacity";
   `from-id == to-id` → "Source and recipient habitats must differ".
8. Save pins in the post-transfer world are exactly the two conditional ones added:
   `cosmogon-cabin-construction` (construction in use) and
   `cosmogon-habitat-stock-transfer` (records present) — no pin appears without evidence.
9. `habitat-diagnose` on the final world: a "construction" explanation with the retained
   numbers (consumed, served fraction, requested/delivered W, volume added, 0 held,
   400 kg structure, cumulative 4 m³) and `source.construction` (law/limits) in the
   diagnostic report; the export `report.json` carries `construction_observation` and
   `construction_source`.

## Native captures (inspected)

* `settlement.png` — Habitat window, **Settlement** tab selected (Overview): held
  0.000/1000.000 kg · structure 400.000 kg · cumulative +4.000000 m³ · live cabin
  504.000000 m³, the interval grid (60 s, 200.0/200.0 W, 100.0% served), material-supply
  authoring and the transfer section header.
* `settlement-settings.png` — Settings pane: disabled Apply (no staged change), staged
  hint, "Enable continuous cabin construction" checked, and the four authored numbers
  (1000 / 0.050 / 200 / 0.010) matching the hand-edited world exactly.

## Regression guarantees

* Legacy worlds without construction/transfer serialize without any new keys (unit
  covered); conditional pins never fire without evidence; the golden remains
  `0xf2c4140a45ae509b` (unchanged in the suite); the transfer/supply edits are ordinary
  undoable journaled edits preserving before-world fingerprints in action JSON.

## Limits of this verification

Rates, bulk ratio and power are authored; conservation and pinning are numerical
bookkeeping, not predictive accuracy. Transfers were verified on one body at one site;
vehicle-mediated material delivery and inter-settlement transport remain unmodeled (as
documented). The transfer section of the Overview pane sits below the fold in the
capture's window height — content verified by rendering plus the readouts above.
