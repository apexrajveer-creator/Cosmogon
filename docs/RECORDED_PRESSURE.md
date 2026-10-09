# Recorded pressure components and change accounting

Open Habitat engineering → Recordings → recorded-view menu → **Pressure components** after loading an ordinary study or paired design. This workspace reads retained observations independently of the live world. With a paired design it explains **A, run-000**; choose the individual branch directory to inspect another corner. It does not display a factorial pressure effect or B−A comparison.

**Components** lists actual cabin O₂, N₂, CO₂ and tracked water vapor: mass kg, amount mol, mole fraction and partial pressure Pa. Liquid stores, reserve cylinders, captured CO₂ and isolated products are excluded. Total pressure is also shown in kPa. Mole fractions describe molecules, not mass. Zero total gas has real zero pressure and unavailable fractions.

**Change accounting** compares two stored states. Reference defaults to Previous stored; First stored and Choose index are explicit alternatives. Observation zero without an explicit reference has unavailable accounting. Choosing the same actual sample gives a real zero difference. Reverse index order is allowed and its signed world/completed time difference is shown. These are actual endpoints; no missing minute is interpolated or reconstructed.

The [ideal-gas law](https://www.grc.nasa.gov/www/k-12/airplane/eqstat.html) uses absolute K and m³. Four species inventory changes are evaluated at reference T/V, then temperature at selected moles/reference volume, then volume at selected moles/temperature. Hover terms for exact formulas and context for exact world/completed times. Expand **Read this result** for a plain-language signed summary. Positive terms raise pressure; negative lower it. The terms telescope to the actual pressure difference, with an explicit numerical residual. **Inventory → temperature → volume is a convention.** Another order assigns interaction terms differently; this is mathematical accounting, not unique physical cause attribution. Pressure may rise while actual breach loss remains positive.

**Breach substep** uses the actual retained initial gas/temperature, escaped quantities, final temperature and the completed configuration volume. It shows the resolved seconds/status separately from requested seconds. It explains only the breach step following the minute's process/thermal update, not the full gap between observations. Current controls edited after the budget do not replace its inputs. Absent evidence remains unavailable; boundary/domain/work statuses do not become supported full-interval flow.

**Evidence & export** captures a complete report at request time in a worker. The new folder contains report.json with exact selected/reference Sample inputs and source-pinned recording summary, components.csv (14 columns, including parent observation index/world/completed times), terms.csv with signed Pa/formulas and a completion manifest written last. CSV parent times identify the stored observation that holds the substep; they are not invented substep timestamps. Missing changes produce no terms rather than fake zero terms. Existing folders/files are protected. The copied full report and captures use the same derivation as CLI. Original studies remain read-only.

```bash
./target/release/cosmogon-cli habitat-pressure --recording /tmp/study \
  --observation 12 --reference-index 0 --output /tmp/pressure-evidence-new
./dist/Cosmogon.app/Contents/MacOS/cosmogon --new lab --focus Mars --speed 0 \
  --habitat-recording /tmp/study --habitat-sample 12 --habitat-pressure-reference 0 \
  --panel habitat-recording-pressure-change --capture /tmp/pressure.png \
  --capture-metadata --capture-after 240 --quit
```

CLI requires an ordinary branch folder and observation index. Without `--reference-index`, it selects the previous stored observation if available; with the flag, it requires that exact index. Native reference flag likewise requires a recording and validates availability before rendering. Indices are 0–1023 and must exist. Capture aliases: `habitat-recording-pressure` (components), `-change`, `-breach`, `-files`; the last three append to `habitat-recording-pressure`. Metadata embeds the exact shared report and loaded source, separate from unopened folder text.

Ideal uniform mixing/lumped temperature are model assumptions. Molecular masses and R match the habitat model. Source consistency, numerical residuals and exact replay do not establish measured atmosphere, predictive uncertainty, human safety or equipment calibration. No new simulation/save state, law, dependency or golden change. Full causal history, physical greenhouse and the master directive remain incomplete. See [design](RECORDED_PRESSURE_DESIGN.md) and [verification](RECORDED_PRESSURE_VALIDATION.md).
