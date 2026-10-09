# Conservative crop material chemistry

Habitat engineering → Crop planner → Finite supplies → Chemistry gives a separate CHON-only engineering surrogate over the exact original supply study. [Design](CROP_CHEMISTRY_DESIGN.md) records the source reasoning. It is a prerequisite for later finite greenhouse material exchange, not an installed crop, measured tissue composition or validated physiology.

## Evidence and assumptions

[Volk and Cullingford 1992, printed p528 Table 2](https://ntrs.nasa.gov/citations/19930004806) supplies the original rounded CO₂, metabolic water, HNO₃-equivalent and O₂ ratios. Original recipe coefficients, their −0.01 kg/kg inedible mass residual, MEC growth/gas forecasts, finite supply support and source hashes remain unchanged. Some inedible source composition was assumed by the authors.

[CIAAW abridged standard atomic weights 2024](https://ciaaw.org/abridged-atomic-weights.htm) supplies a separately pinned nominal atomic basis: C 12.011, H 1.0080, O 15.999, N 14.007 g/mol. Consistently derived compound masses in CO₂/H₂O/HNO₃/O₂ order are retained in the report. Listed source atomic-weight variations are not propagated biological or predictive uncertainty. Existing habitat reaction and pressure constants remain unchanged.

For exactly 1 kg dry edible or inedible material, this engineered surrogate retains the published CO₂/metabolic-water/HNO₃-equivalent inputs, assigns their carbon/hydrogen/nitrogen to dry matter, infers dry oxygen as the remaining dry mass, and derives O₂ as input oxygen minus dry oxygen. All quantities must be finite and nonnegative. This CHON-only/no-ash assumption conserves four elements and total mass to retained floating roundoff. The derived inedible O₂ is approximately 1.31 kg/kg, versus the separately retained published 1.32. This difference is explicit; it is not a corrected source measurement.

No P/K/S/trace nutrients, ash, ions, pH, salts, root exudates, alternative gases or isotope variability are modeled. HNO₃-equivalent is an accounting basis, not a literal fertilizer recommendation. No food calories, nutritional adequacy, stressed yield, habitat inventory grant or hardware calibration is implied.

## Actual-age report

Thirty-one daily ages use the original maintained-regime edible and inedible dry masses. The report retains separate published and conservative O₂, their signed difference, original recipe and surrogate mass residuals, inferred dry C/H/O/N, actual input/output element projections and all four residuals. Day-zero production is genuinely zero. Transpiration is retained separately; it is never reaction water or an oxygen source. Legacy absent vapor remains unavailable. Supply excess preserves potential source outputs and the original unsupported state without inventing stress physiology.

Element balance shows selected-day gas products and four complete element rows with units. Assumptions shows separate source/derived coefficients and inferred dry fractions, with complete limits and formula/atomic details. Chemistry files protects existing folders and reopens a verified chemistry study independently of the live world and unopened path text. Prepared supply changes recalculate the separate chemistry; Study files can restore the retained opened inputs. Cached chemistry is invalidated only when its source changes, not recalculated every frame.

## Protected files and CLI

```sh
./target/release/cosmogon-cli crop-chemistry \
  --recording /tmp/supply-study --output /tmp/new-chemistry
./target/release/cosmogon-cli crop-chemistry-inspect \
  --recording /tmp/new-chemistry --output /tmp/new-verified-copy
```

The first command accepts an original supply definition file or verified supply folder. The second requires a complete chemistry folder. Both print the same full chemistry report and refuse unrelated options. A new output folder holds `chemistry.json`, `elements.csv` and the full protected `original/` supply bundle, including optional frozen context. The 26-column CSV uses 17-digit scientific values; unavailable vapor is an empty cell. Existing folders are refused, and an I/O error reports an incomplete export.

Reopening bounded regular files recomputes original evidence and the pinned chemistry model, verifies the full typed report and exact CSV, and rejects source, weight, composition or numerical changes. Frozen context is observation-only and remains independent of the app's live world.

```sh
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/world.cosmo \
  --panel greenhouse-chemistry --greenhouse-supplies /tmp/supply-study \
  --greenhouse-supply-day 30 --crop-chemistry-output /tmp/native-chemistry \
  --capture /tmp/chemistry.png --capture-metadata --capture-after 240 --quit
```

`--crop-chemistry CHEMISTRY_FOLDER` reopens verified chemistry instead of the original supply source. The age flag selects an integer 0–30. `greenhouse-chemistry-model` and `greenhouse-chemistry-files` select assumptions/files. Source and output preflight run before GPU startup. Only one source and export type can launch at once. Definitions and old report schemas remain unchanged.
