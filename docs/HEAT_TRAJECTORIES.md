# Recorded cumulative heat trajectories

Choose cumulative heat metrics in Habitat engineering → Recordings, paired curves/tables, graph layouts or Research figure. The same explicitly selected quantities work in CLI comparisons and SVG/CSV/JSON figures. Existing immutable recorded summaries, sweep/factorial metric catalogs and final analyses stay unchanged, including heat-enabled version-one recordings. Full-catalog trajectory exports therefore keep their existing metrics; select a heat metric and export a research figure to retain its four responses and contrasts.

Eleven signed energy paths/storage counters are available in joules: `heat_bus`, `heat_battery`, `heat_metabolic`, `heat_electrolysis`, `heat_sabatier`, `heat_crew_vapor`, `heat_condensation`, `heat_radiator`, `heat_conduction`, `heat_breach`, `heat_storage`. `heat_paths` sums ten paths; `heat_residual` is that sum minus interval sensible storage. `heat_executed` and `heat_excluded` are cumulative coverage seconds. Cooling/debits stay negative. A valid enabled ledger initially has actual zero counters; absent legacy ledgers remain unavailable. These are cumulative quantities from the stored origin, never powers or a sum of sparse sampled minute budgets.

The selected native view displays origin, executed and excluded time. Hover the coverage line to inspect the individual branches/comparison inputs and the complete method scope. Figure/comparison JSON carries every retained observation's origin, completed end, coverage and capacity transitions, together with the original heat source/method and full inputs. SVG includes the cumulative/executed-subset scope. Original numerical values remain in the existing ten-column figure CSV with 17-digit formatting. Signed log/linear scales support debits; positive log omits nonpositive values with explicit omission counts. Connecting lines remain display guides.

Partial model execution retains the actual counters accumulated in executed intervals. A flat counter during excluded time is **not zero physical heat**. Such figure rows are `available_executed_subset`. Numerical branch responses remain retained, but comparisons/paired contrasts require matching aligned ledger origins and completed habitat ends, equal executed duration and no excluded interval since the origin. Different origins, incomplete coverage and missing ledgers have explicit support reasons and empty differences/effects. Elapsed alignment may place different physical epochs together; it does not make them physically equivalent. Readers refuse a reset/discontinuous ledger inside an otherwise plausible recording.

```bash
./target/release/cosmogon-cli habitat-compare --recording /tmp/cabin-a \
  --reference /tmp/cabin-b --measure heat_storage --alignment world \
  --output /tmp/new-heat-comparison
./target/release/cosmogon-cli habitat-figure --recording /tmp/paired-cabin \
  --definition /tmp/heat-layout.json --output /tmp/new-heat-figure
./dist/Cosmogon.app/Contents/MacOS/cosmogon --panel habitat-figure \
  --habitat-recording /tmp/paired-cabin --habitat-graph /tmp/heat-layout.json \
  --habitat-figure-output /tmp/new-native-heat-figure \
  --capture /tmp/heat-figure.png --capture-after 240 --quit
```

A complete reusable layout:

```json
{"format":"cosmogon-habitat-graph-layout","schema_version":1,"metric":"heat_radiator","effect":"average_a","alignment":"world","window":[0,1],"scale":"signed_log","signed_threshold":1,"include_zero":true,"time_unit":"hours"}
```

For paired input, `effect` selects `average_a`, `average_b` or `difference_in_differences`; actual independent four-branch responses accompany the chosen effect in `figure.json`. Ordinary input uses its recorded response(s). Existing output is protected and the complete figure manifest is written last. No new solver, state/save schema, dependency or golden change. Temperature/effective capacity are reduced model assumptions; summed interval C×ΔT is not absolute energy, hardware calibration, full-system thermodynamic closure or predictive uncertainty. [Accounting basis](CABIN_HEAT_ACCOUNTING.md), [preimplementation design](HEAT_TRAJECTORIES_DESIGN.md).
