# Finite crop supply studies

Habitat engineering → Crop planner → Finite supplies adds a separate feasibility study to the unchanged maintained-regime forecast. It asks whether four explicit budgets cover the forecast's known input demands at each of its 31 daily ages. The [preimplementation design](FINITE_CROP_SUPPLIES_DESIGN.md) records source and scope decisions. This does not install crops or grant habitat food, oxygen or water.

## Inputs and numerical meaning

A version-1 supply definition wraps the original source-pinned crop definition, recipe identity and four user-assumed budgets: liquid water kg, CO₂ kg, HNO₃-equivalent nutrient kg and lighting energy J. Budgets can be zero; mass budgets are bounded at 1e9 kg and lighting at 1e16 J. These are supplied-cycle assumptions, independent of habitat stores. Original version-1 and version-2 crop definitions/forecast output remain unchanged. In Supplies, **Use current crop planner inputs** explicitly copies the prepared regime, area, efficacy and optional maintained humidity into the supply study; editing the ordinary planner does not silently rewrite an opened study.

At each age, metabolic water and material-recipe CO₂/nutrient/O₂ projections come from the original separate edible/inedible dry production. Total modeled liquid need is metabolic incorporation plus the original optional light-period vapor transfer. No condensate recovery, dark transpiration, irrigation losses or future recycling credit is assumed. Without the transpiration model, total liquid demand and complete supply support remain **unavailable**, even though other known constraints can independently be exceeded. Saturated-air mathematical zero vapor still requires positive metabolic incorporation.

Lighting energy is the original fixture power × 16-hour photoperiod × age in terrestrial days. Pumps, cooling and humidity regulation remain unavailable. Recipe oxygen is an external projection, separate from the original MEC gas calculation retained in the full forecast. Neither is an actual finite-stock grant. Dry lettuce mass does not establish calories or a complete diet.

Budget margin is supplied budget minus cumulative demand; negative means exceeded. Exact equality remains supported. For each constraint the report retains the last supported daily observation, first exceeded daily observation and their one-day bracket. It does not interpolate an exact depletion deadline. `supported_prefix_last_day` is available only when all four demand paths are available; `first_known_unsupported_day` can still report a known limit when water is missing. Day 30 within budget means only that these maintained-input demands fit these budgets, not proof of growth or hardware adequacy. Potential curves continue after a limit, explicitly retaining maintained-regime meaning rather than inventing stressed yield.

## Source and accounting

[Volk and Cullingford 1992, NASA 19930004806](https://ntrs.nasa.gov/citations/19930004806), printed p528 Table2, gives rounded lettuce coefficients per kg separate dry production. The report records the exact source PDF SHA-256 `98498e499f097489c294032611e2532ef70765d11c9b32e41c912945810fdfc5`, all coefficients, recipe model and original MEC source/coefficients/numerical evidence. Source ratios are held at their published precision; the inedible recipe has an explicit −0.01 kg/kg inputs-minus-outputs residual. Some inedible composition is author-assumed. The numeric residual is retained, never quietly corrected.

Combining that constant-composition recipe with the separate MEC growth forecast is an explicit engineering scenario, without empirical validation of the combined model. HNO₃-equivalent is a nutrient accounting basis, not an acid tank, complete fertilizer chemistry or purification specification. Physical crop coupling will require conservative elemental chemistry, finite supplies, independently retained growth state, allocated energy/heat/humidity exchange and actual harvest/storage conversion. Biological stress, chemistry and hardware calibration remain incomplete.

## Native views and files

Needs & coverage shows one selected resource demand with units, budget line, actual daily points, support colors and a selected-age table. Lines are guides; the precision resides in the retained numbers. Supplies edits four budgets and explicitly copies crop inputs. Recipe & limits exposes metabolic/vapor decomposition, source ratios, residual and limitations. Study files opens a definition or validated bundle, restores opened inputs, freezes/removes observation context and saves to a new folder.

Opening calculates/verifies before replacing valid prepared results. Original opened inputs and source path remain retained separately from modified prepared assumptions and unopened path text. A frozen world is observation context only; advancing the live app or selecting another object does not change that context. Explicit **Freeze current world as observation context** replaces it; it never fills budgets or mutates stocks. Protected export refuses existing folders and invalid/inconsistent bundle fields before writing. I/O failure reports an incomplete export rather than silently claiming a completed bundle.

Bundles contain `definition.json`, `report.json` including the exact original forecast, `manifest.json`, 20-column `samples.csv`, and optional `context.cosmo`. CSV numeric values use 17 digits in scientific notation, with empty cells for unavailable values. JSON uses null for missing values. Reopening checks pinned definitions/models/recipe/source identity, exact full calculated report, CSV agreement and optional context identity/selected habitat observation. The context fingerprint is the existing FNV state identity, not a cryptographic claim; source PDF identity uses SHA-256. Imports bound regular-file sizes and reject unknown fields and unsupported/nonfinite/out-of-range input.

## Reproduce

```sh
./target/release/cosmogon-cli greenhouse-supply-definition --output /tmp/supply-inputs.json
./target/release/cosmogon-cli greenhouse-supplies --definition /tmp/supply-inputs.json --output /tmp/supply-study
./target/release/cosmogon-cli greenhouse-supply-inspect --recording /tmp/supply-study --output /tmp/supply-copy
./dist/Cosmogon.app/Contents/MacOS/cosmogon --new lab --focus Mars \
    --greenhouse-supplies /tmp/supply-study --panel greenhouse-supplies \
    --greenhouse-supply-day 12 --greenhouse-supply-resource liquid_water \
    --greenhouse-supply-output /tmp/native-supply-study --capture /tmp/supplies.png \
    --capture-metadata --capture-after 240 --quiet --quit
```

CLI `greenhouse-supplies` accepts optional `--initial SAVE --habitat-id N` for read-only frozen context (the sole habitat is selected automatically). It refuses ignored scenario/time/sample requests. `greenhouse-supply-inspect` verifies retained files and can export a protected copy. Existing ordinary `greenhouse` behavior is unchanged. Native source can be a definition file or verified bundle folder. Developer capture panes: `greenhouse-supplies`, `greenhouse-supply-inputs`, `greenhouse-supply-model`, `greenhouse-supply-files`; resource keys: `liquid_water`, `carbon_dioxide`, `nutrient_hno3_equivalent`, `lighting_energy`. Age is an integer 0–30. Automatic output requires an explicit source and new nonempty folder. Photo/performance sidecars retain prepared/opened identities, complete report/manifest, selected resource/age and export status.

Simulation state, thermal laws, saves and golden `0xf2c4140a45ae509b` remain unchanged. Graphics stay queued. This is another connected phase9/16 component; the master directive remains incomplete.

[Conservative crop material chemistry](CROP_CHEMISTRY.md) now exposes a separately pinned CHON-only surrogate and explicit published-versus-derived oxygen. Original supply evidence and potential forecasts remain unchanged; physical greenhouse coupling remains incomplete.
