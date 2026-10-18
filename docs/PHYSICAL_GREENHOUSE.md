# Physical greenhouse installation and finite habitat-stock exchange

Build 52 closes the last missing physical couplings between the finite installed crop
process and the cabin. Two ordinary undoable sandbox edits join the existing crop
toolbox: an explicit runtime **installation** of a greenhouse process into a cropless
habitat, and a bounded 1:1 **store conversion** of harvested edible dry material into
the habitat's finite dry-food tank that feeds the nominal crew diet. Nothing is
granted silently; every transfer is previewed, journaled, undoable and mass-accounted.

## Runtime greenhouse installation

A habitat created without a crop can receive one later through
`habitat-crop-install` (CLI) or the Installed crops overview install control (native).
The edit `Edit::InstallHabitatCrop` grants exactly one explicit bounded finite
nutrient-equivalent charge, validated 0…1e7 kg and journaled in the action record.
Water, captured CO₂, oxygen headroom, power and dry food are never granted.

Validation refuses installation when a crop already exists, the crop configuration
fails its source pins or transpiration-model checks, the whole candidate
configuration fails (crop vapor requires enabled moisture accounting; the crop O₂
product ceiling must fit the shared reserve tank), the habitat's current oxygen
reserve exceeds the crop product capacity, or the nutrient charge is out of bounds.
Applying completes the current cabin interval first, then arms heat-ledger crop
debits at zero when a ledger exists, installs the live configuration crop and builds
fresh crop state at the completed clock. Post-mutation configuration checks re-run.

Because recordings require a constant original definition, a runtime-installed crop
pins to the retained live configuration (source crop and dry chemical basis must
match the installed setup) plus the crop state's own self-pins (model, chemistry and
source snapshots, recomputed curve). Disabling a runtime-installed greenhouse is
refused: the live configuration is the only carrier of its operational parameters,
and its material and history stay in use. Authored crops keep the established
disable-retains-inventory behavior. Settings points cropless habitats at the install
control instead of implying that toggling could install.

## Bounded store conversion

`habitat-crop-store` (CLI) or the store control (native) issues
`Edit::StoreHabitatCropFood`: after an explicit completed harvest, up to
`min(requested, held harvested edible, food-tank headroom)` kilograms move 1:1 from
the crop's harvested edible store into `state.food_dry_kg`. The rejected remainder
stays in the harvested store; refusals change neither side. Inedible dry material is
never food and never converts. The nominal crew cycle already draws the finite tank
at 0.800 kg per reference person-day, so stored food visibly feeds the crew without
touching diet bookkeeping.

Harvest semantics are unchanged: it still transfers standing dry material into
separate harvested stores with no diet credit and no new cycle. The crop state gains
a cumulative `food_stored_kg` counter (serialized only when positive, so legacy
saves and recordings stay byte-identical): harvested edible plus stored equals the
pinned 30-day source-curve edible total, and crop chemical energy counts stored
material so heat-ledger debits stay consistent after mass leaves the crop pools. The
assumed 20 MJ/kg crop chemical basis is not crew metabolizable energy and the
nominal crew metabolic rate is unchanged — this is an explicit limitation, not a
calorie claim.

## Recording and trajectory evidence

Installations may span recorded observations: a trajectory now allows leading rows
without crop state, with the explicit installation origin at the first crop row, and
keeps the existing crop-never-disappears and origin-never-changes rules from that row
onward. Crop counters count standing plus stored material in their monotone checks.
`crop.csv` gains one optional column, `food_stored_kg`, present only when any
retained row has a positive counter; stored CSVs and legacy crop recordings re-derive
byte-exactly. Habitat `samples.csv` already carries `food_dry_kg` per row. Balance
bundles still refuse ranges whose stock sets change length, so an install-spanning
range must be analysed from its first complete post-installation row. The escaped-mass
identity adds the runtime crop's initial nutrient charge for habitats whose original
definition had no crop.

## Command-line examples

```bash
# Install a finite greenhouse into a cropless habitat, granting only its nutrient charge.
./target/release/cosmogon-cli habitat-crop-install \
  --initial /tmp/cropless-world/final.cosmo --habitat-id 0 \
  --definition /tmp/finite-crop-config.json --nutrient-kg 2 \
  --output /tmp/crop-installed

# Grow, harvest, then convert held harvested edible dry material into cabin dry food.
./target/release/cosmogon-cli habitat --initial /tmp/crop-installed/final.cosmo \
  --habitat-id 0 --samples 31 --interval-days 1 --output /tmp/crop-grown
./target/release/cosmogon-cli habitat-crop-harvest \
  --initial /tmp/crop-grown/final.cosmo --habitat-id 0 --output /tmp/crop-harvested
./target/release/cosmogon-cli habitat-crop-store \
  --initial /tmp/crop-harvested/final.cosmo --habitat-id 0 --store-kg 1 \
  --output /tmp/crop-food

# Continue cabin time; the nominal crew diet now draws the stored dry food.
./target/release/cosmogon-cli habitat --initial /tmp/crop-food/final.cosmo \
  --habitat-id 0 --samples 6 --interval-days 1 --output /tmp/crop-diet
```

`--definition` is finite crop process JSON (the `habitat-crop-definition` output
shape). `--nutrient-kg` is the explicit installation charge; `--store-kg` defaults to
the full held harvested edible amount when omitted. Both commands copy the
before-world, write an action JSON with state fingerprints, export an ordinary
recording folder and print the habitat report. Each write target is create-new
protected.

Native QA:

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/crop-food/final.cosmo \
  --focus Mars --speed 0 --panel habitat-crop \
  --capture /tmp/physical-greenhouse.png --capture-metadata \
  --capture-after 240 --quiet --quit
```
