# Physical greenhouse installation and finite habitat-stock exchange: design draft before implementation

Next connected phase9 build after the end-to-end supply-mission closeout. Do not add a
presentation-only layer. Two missing physical couplings become actual undoable edits:
(1) runtime installation of a finite greenhouse process into an existing habitat, and
(2) an explicit bounded conversion of harvested edible dry material into the habitat's
finite dry-food stock. Both are ordinary sandbox edits with preview-before-commit,
journal summaries, undo/redo and no silent grants. Existing cropless worlds keep
byte-identical state; the world golden for absent crop state stays unchanged.

## Research and limits

Today crops exist only through an original explicit habitat definition
(`Habitat::new` + `Definition.initial.crop_nutrient_equivalent_kg`); runtime
installation is refused (`check_config` refuses toggling because it would grant
stock), and harvest moves material only into crop-local harvested stores that never
reach `state.food_dry_kg`. The nominal crew cycle already draws dry food at
0.800 kg per reference person-day from the finite habitat store, so a conservative
mass-conserving store conversion closes the material loop without touching diet
bookkeeping, oxygen, water, CO₂ or power accounting (those exchanges already exist).

A runtime installation cannot rewrite `Habitat.definition`: recordings demand
constant definitions across retained observations, and `habitat_balance` pins
identity to the original definition. Therefore the installed crop lives in the live
`Habitat.config.crop` + `Habitat.state.crop` pair exactly like configuration edits do,
and `Habitat::validate` learns a second pin path: authored crops stay pinned to the
original definition; runtime-installed crops pin to the retained live configuration
(source crop and dry chemical basis must match) plus the state's own self-pins
(model/chemistry/source snapshots, recomputed curve). Disabling a runtime-installed
greenhouse would destroy the only carrier of its operational parameters
(assume-maintained flag, auxiliary power, capacities), so `check_config` refuses
disabling it; authored crops keep the established disable-retains-inventory behavior.

Installation grants one explicit bounded finite nutrient-equivalent charge (the same
quantity original setups author), because no habitat-side nutrient tank exists. The
charge is an explicit edit parameter, validated `0..1e7` kg, journaled in the edit
record, included in crop state (`initial_nutrient_equivalent_kg`), and added to the
escaped-gas mass identity for habitats whose original definition has no crop. Water,
captured CO₂, oxygen headroom, power and food tanks are never granted by installation.

The store conversion is deliberately separate from harvest: harvest keeps its old
semantics (completed-cycle transfer to separate stores, no diet credit), and
`Edit::StoreHabitatCropFood` moves `min(requested, held edible, food tank headroom)`
kilograms of harvested edible dry material 1:1 into `state.food_dry_kg`. The rejected
remainder stays in the harvested store; refusals change neither side. Inedible dry
material is never food and never converts. The cumulative `food_stored_kg` crop
counter makes the crop-side ledger checkable: harvested edible + stored equals the
pinned 30-day source curve edible total, and `chemical_energy_j()` counts stored
material so heat-ledger crop debits stay consistent after mass leaves the crop pools.
The assumed 20 MJ/kg crop chemical basis is not crew metabolizable energy and the
nominal crew metabolic rate is unchanged; this is an explicit limitation, not a
calorie claim.

## Selected mechanism (2026-10-18, before Rust changes)

1. `Edit::InstallHabitatCrop { system, id, config, nutrient_kg }`. Validation: habitat
   exists, no crop state, crop `Config::validate` (source pins, transpiration model),
   whole candidate configuration `Config::validate` (crop vapor requires enabled
   moisture accounting; crop oxygen ceiling must fit the reserve tank capacity), state
   oxygen reserve must fit the crop product capacity, finite nutrient bounds. Apply
   completes the current interval first (preview clone, same pattern as harvest),
   then sets live config crop, builds `crop_process::State::new(config, nutrient, t)`
   at the completed clock, and arms heat-ledger crop debits at zero when a ledger
   exists. Post-mutation `check_config` re-checks the full configuration/state pair.
2. `Edit::StoreHabitatCropFood { system, id, requested_kg }`. Validation: crop state,
   explicit completed harvest, positive bounded request, held edible, and food tank
   headroom all present with a positive transfer. Apply completes the interval, then
   performs the exact bounded move and records the cumulative counter.
3. `crop_process::State` gains `#[serde(default, skip_serializing_if = zero)]
   food_stored_kg`, so legacy saves, legacy recordings and crop worlds that never
   store food serialize byte-identically to today. `validate` gains bounds, the
   harvested-ledger identity and the not-harvested/zero coupling; the standing plus
   harvested ledger check adds stored edible material back to the edible side.
4. `habitat_crop_trajectory::validate_series` allows leading rows without crop state
   (an explicit installation origin at the first crop row) while keeping the existing
   "crop never disappears and origin never changes" rules from that row onward, and
   counts `dry + stored` in the monotone counters. Recordings that span an
   installation therefore validate; balance ranges over changing crop stock sets stay
   refused by the existing endpoint-length guard.
5. `crop.csv` gains one optional tail column `food_stored_kg`, present only when any
   retained row has a positive counter. Old stored CSVs re-derive byte-identically.
   `samples.csv` (habitat) already carries `food_dry_kg` for every row and needs no
   header change.
6. `crop_process::LIMITS` and the `finite-crop` registry descriptor texts state the
   new facts: installation is an explicit finite edit, store conversion is a bounded
   1:1 mass-parity transfer feeding nominal dry-food accounting, inedible material
   never becomes food, and neither diet adequacy nor metabolizable energy is
   validated. Stored manifests pin the previous text; superseding evidence is
   re-recorded (no test reads committed study folders).
7. CLI: `habitat-crop-install --initial SAVE --habitat-id ID --crop-config FILE
   --nutrient-kg N --output NEW_DIR` and
   `habitat-crop-store --initial SAVE --habitat-id ID [--kg N] --output NEW_DIR`,
   mirroring `habitat-crop-harvest` (before-world copy, action JSON with state
   fingerprints, ordinary export, report print).
8. Native: the Installed crops overview gains the install control (staged crop config
   and nutrient charge come from the existing New crop setup staging) when the
   selected habitat has no crop, and the store control when harvested edible material
   is held with food headroom; the settings pane points cropless habitats at the
   install control instead of implying toggling could install.

## Verification requirements

Edit refusals (missing moisture accounting, oxygen ceiling conflicts, double
install, pre-harvest store, empty tank/held store, nonpositive request) leave both
sides unchanged; happy paths conserve mass exactly (nutrient charge equals accounted
mass delta; stored food equals edible delta), heat debits stay consistent after
installation and storage, save/import round trips and undo/redo restore exact JSON,
a recording spanning install + grow + harvest + store validates and re-derives both
CSVs byte-exactly, legacy crop recordings without the new column still re-derive
byte-exactly, the nominal crew cycle visibly draws the stored food, and absent-crop
world state plus the world golden stay unchanged. All release tests, clippy, release
builds, inspected native captures and `git diff --check` remain green.
