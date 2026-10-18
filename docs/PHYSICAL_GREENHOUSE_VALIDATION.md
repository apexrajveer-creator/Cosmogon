# Physical greenhouse installation and finite habitat-stock exchange: verification

Build 52 verification, 2026-10-19. Commands use the release CLI built from this tree.
Evidence lives in protected create-new folders; every study folder is an ordinary
export (definition, initial/final worlds, samples, crop.csv when crops exist,
manifest, report).

## Unit and integration coverage

`cargo test --release -p cosmogon_sim crop_process` covers the new mechanism:
`runtime_install_is_bounded_refused_twice_and_never_disablable` (moisture-less
candidate refused with the state unchanged; out-of-bounds nutrient charge refused;
a successful 2 kg install adds exactly 2 kg of accounted mass, pins origin and
initial nutrient, arms heat-ledger crop debits; a second install is refused;
lowering the O₂ product capacity below held reserve is refused; disabling the
runtime crop is refused with "cannot be disabled"; the grown save round-trips and
contains no `food_stored_kg` text), `store_conversion_is_bounded_conserves_mass_and_feeds_the_nominal_diet`
(pre-harvest store refused; zero request refused; a store moves exactly
min(request, held, headroom) with counter, harvested remainder and accounted mass
consistent; a tightened food tank bounds the next store; a full tank refuses with
"headroom"; the save round-trips with the counter present; a crew-4 day draws the
exact 3.200 kg nominal diet from the prefilled tank), and
`recording_spanning_install_and_conversion_validates_with_optional_food_column`
(a leading cropless row validates; the optional `food_stored_kg` column appears
only after a positive conversion; an ordinary post-install recording re-derives
byte-exactly with a 26-stock balance and a 13-term crop heat span).
`crates/cosmogon/src/sim.rs::crop_edit_tests::runtime_greenhouse_install_and_food_store_are_exactly_undoable`
proves install, harvest and store are exact undo/redo JSON round trips.

`cargo test --release -p cosmogon_cli` argument tests refuse incomplete
install/store invocations, negative charges, unrelated flags (`--samples` on
install, `--definition` on store) and accept the complete forms.

## End-to-end loop (release CLI, Mars surface, crew 1)

```bash
# cropless-with-moisture definition (habitat-definition output plus the moisture
# block from habitat-crop-definition), maintained-chamber assumption enabled in
# the crop config (it starts false by design)
cosmogon-cli habitat --definition /tmp/b52/cropless-wet.json --output /tmp/b52/world
cosmogon-cli habitat-crop-install --initial /tmp/b52/world/final.cosmo --habitat-id 0 \
  --definition /tmp/b52/crop-config.json --nutrient-kg 2 --output /tmp/b52/installed
cosmogon-cli habitat --initial /tmp/b52/installed/final.cosmo --habitat-id 0 \
  --samples 31 --interval-days 1 --output /tmp/b52/grown
cosmogon-cli habitat-crop-harvest --initial /tmp/b52/grown/final.cosmo \
  --habitat-id 0 --output /tmp/b52/harvested
cosmogon-cli habitat-crop-store --initial /tmp/b52/harvested/final.cosmo \
  --habitat-id 0 --output /tmp/b52/stored
cosmogon-cli habitat --initial /tmp/b52/stored/final.cosmo --habitat-id 0 \
  --samples 6 --interval-days 1 --output /tmp/b52/diet
```

Observed retained values:

- Install: action `cosmogon-crop-install/1` with `before_state_fingerprint`
  `fnv1a64:b00da7bf`, edit `InstallHabitatCrop { nutrient_kg: 2 }`, before-world
  copy `before-habitat-crop-install.cosmo`; crop state origin pins to the
  completed install clock.
- Grow (30-day span, 31 rows): source progress **30.0 days**, standing dry
  **[0.7036212416499155, 0.03703770526983985] kg** edible/inedible, CO₂ consumed
  1.3428140046561066 kg, metabolic water 0.42143484563889394 kg; crop.csv has the
  legacy 23-column header (no `food_stored_kg`, counter still zero).
- Harvest: standing material moves to the separate harvested stores, no diet
  credit, no restart.
- Store (full held amount requested by default): action `cosmogon-crop-store/1`
  with `requested_kg: 0.7036212416499155`; harvested edible → 0, crop counter
  `food_stored_kg: 0.7036212416499155`, habitat `food_dry_kg: 75.90362124188393`
  (75.20000000023401 held + the exact conversion); crop.csv gains the 24th
  `food_stored_kg` column.
- Diet (5-day span): `food_dry_kg` draws the exact **0.800 kg per day** nominal
  crew diet at served fraction 1: 75.90362124188393 → 71.90362124192167.
- Balance on the grown recording: component-sum residual
  **−6.417089082333405e-14 kg**, 26 stocks; crop standing
  **+0.7036212416499155 / +0.03703770526983985 kg**, crop nutrients
  **−0.14368726475222515 kg**, food dry −23.999999999773536 kg (31 crew-days),
  potable water +14.964681605447794 kg, oxygen reserve −25.68272283252361 kg.
  Balance on the stored export (both endpoints post-edit, the established harvest
  evidence shape) closes at residual 0.0 with the same 26-stock set.
- `habitat-inspect` on the stored recording returns the standard bounded-format
  checks (`cosmogon-habitat-recording-inspection`).

## Refusals observed

- Second install into the same habitat: "A greenhouse is already installed in
  this habitat".
- Store before a completed harvest: "Food storage requires an explicit
  completed-cycle harvest".
- Store into a cropless habitat: "No installed crop".
- `--nutrient-kg -1`: argument refused before any world is read ("a finite
  charge between 0 and 1e7 kg").
- Moisture-less install candidates, O₂ ceiling conflicts, capacity reductions
  below held stock, and disabling a runtime crop are covered in the unit tests
  above.

## Native capture QA

Two inspected release captures (1920×1080, `--panel habitat-crop --capture-after 240 --quiet --quit`):

- `/tmp/b52/capture-install.png` — the cropless world's Installed crops overview shows
  "No crop is installed in this habitat…", the staged "Installation nutrient charge
  (kg) 1.00" and the enabled "Install greenhouse" button with the grants-only note.
- `/tmp/b52/capture-crop.png` — the stored world shows harvest and store controls
  correctly disabled (cycle complete, held edible 0), Supported source progress
  30.000000 / 30 days, Standing 0 / 0, Harvested 0.000000 / 0.037038 kg, **Cabin
  dry-food tank 75.903621 kg** (matching the CLI value), and the crop CO₂/water,
  vapor/O₂ and light/auxiliary electricity readouts.

## Regressions

Legacy cropless worlds and legacy crop recordings serialize byte-identically
(`food_stored_kg` is skipped when zero; `crop.csv` gains the optional column only
after a positive conversion; the shared reader accepts both). The world golden
stays `0xf2c4140a45ae509b` (absent-crop state is untouched by design; confirmed by
the full workspace release run). Full `cargo test --release --workspace`,
clippy with zero new warnings in touched files, fresh release app/CLI builds and
an inspected native capture complete the gate.
