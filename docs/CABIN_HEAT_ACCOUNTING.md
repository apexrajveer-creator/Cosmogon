# Recorded cabin heat accounting

Newly generated habitat definitions explicitly set `record_heat: true`. Each executed interval retains eleven signed joule counters without changing the existing temperature solver. Original definitions with the field absent remain disabled, preserving old saves and exact independent replay. There is no invented history or automatic adoption into older habitats.

Open Habitat engineering → Recordings → Cabin balances → Heat ledger. Select two ordered actual observations; choose any of all four paired branches. The report shows before/after/change, executed seconds, separately excluded environment/starting-temperature seconds, actual effective-capacity transitions, summed heat and storage and two residual conventions. Latest sampled watts are never stretched over the experiment.

```sh
./target/release/cosmogon-cli habitat-definition --target Mars --output /tmp/heat-definition.json
./target/release/cosmogon-cli habitat --definition /tmp/heat-definition.json --samples 25 --interval-days 0.041666666666666664 --output /tmp/heat-study
./target/release/cosmogon-cli habitat-balance --recording /tmp/heat-study --from-index 0 --observation 24 --output /tmp/heat-balance
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/heat-study/final.cosmo --focus Mars --panel habitat-recording-balance-heat --habitat-recording /tmp/heat-study --habitat-balance-output /tmp/native-heat-balance --capture /tmp/heat.png --capture-metadata --capture-after 240 --quiet --quit
```

Choose new output names each time. A balance of an enabled recording adds `heat.csv` to the existing protected report/stocks/counters bundle; manifest is written last. Heat CSV uses seven columns: branch, id, unit, initial, final, change, scope. Values retain original SI joules with 17 digits. The full report retains exact endpoint samples, source identity, paired method context and coverage. Legacy reports show an unavailable reason and omit heat CSV. Balance worker requests freeze the actual loaded evidence and selected range.

Positive heat paths enter the lumped cabin: bus electricity, conversion losses, nominal metabolic heat, reduction reference heat and condensation. Electrolysis enthalpy, crew vapor phase energy, radiator rejection and breach lumped cooling are debits. Conduction is signed. Heater power already appears in delivered bus electricity. Full gas outflow enthalpy is separately reported by breach accounting and is not counted again here.

Storage is the sum of each executed interval's assumed C×ΔT. It is not absolute thermodynamic energy. This sum remains defined when capacity changes; endpoint C×ΔT still requires constant retained capacity. Capacity edits model different assumptions, not construction or added energy. The displayed span residual is heat minus storage; accumulated saved interval residual uses storage minus heat. Floating-point summation can leave small differences, which stay visible.

Unavailable environment and unsupported starting cabin temperature skip the existing thermal computation and accumulate excluded seconds. A computed interval ending outside the domain is still recorded as executed; subsequent unsupported intervals are excluded. Zero-length and rejected steps add nothing. A partial ledger describes only executed model intervals. Complete model coverage does not establish hardware accuracy, whole-system thermodynamic closure or predictive uncertainty.

New worlds pin `cosmogon-cabin-heat-accounting/1`; recording manifests pin the shared method/source and model descriptor. Readers reject missing/mismatched pins and definition/ledger disagreement. [Design and primary-source context](CABIN_HEAT_ACCOUNTING_DESIGN.md). The unchanged simulation laws and absent legacy states preserve the untouched-universe golden. Physical greenhouse coupling, equipment calibration and the master directive remain incomplete.
