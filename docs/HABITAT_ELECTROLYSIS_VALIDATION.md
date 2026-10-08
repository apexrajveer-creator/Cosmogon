# Finite electrolysis verification

Date: 2026-10-08. Parent greenhouse checkpoint: `40267a4`.

Release workspace tests pass: 253, plus one existing ignored. Independent fixtures check water/oxygen/hydrogen stoichiometry, reaction enthalpy, electrical and thermal balance, all finite stock/storage/power bounds, crew water reservation and next-interval oxygen use. Integrated checks cover exact scheduler chunking, save continuation, optional model pins, corrupted budgets, atomic capacity refusal and configuration changes between fixed intervals. Golden `0xf2c4140a45ae509b` is unchanged; no new dependencies.

Release all-targets workspace clippy and native/CLI builds succeed with baseline warnings and no new warnings in changed code. The repository-wide formatting check remains non-green on longstanding compact formatting; it was read-only and no unrelated files were reformatted.

All 29 CLI/native cases pass: 19 CLI trajectories/input/replay/output-protection cases and ten native cases, including eight inspected captures. Isolated one-day production yields 6 kg O₂ and 0.7559833493756016 kg H₂, consumes their summed water mass, and debits 107190269.63511132 J reference reaction enthalpy. Independent mass/electric/thermal checks bound bookkeeping residuals, not predictive accuracy. Initial-state replay reproduces samples JSON/CSV and report byte for byte; the original non-reactor habitat trajectory remains byte-identical. Invalid future models, subthermoneutral assumptions, capacities and unknown fields fail explicitly; native invalid inputs fail before GPU startup without a PNG.

Native Oxygen captures display actual component flows and water/storage/power constraints; all seven chemistry/control/model sidecars match CLI reports. The eighth capture checks the crop planner with populated habitat context: the full curve, axes, selected age and numerical rows remain visible, and the forecast matches the previous greenhouse checkpoint. Manual pointer/keyboard and assistive-technology verification remains outstanding.

The final packaged app ran normally for 48.05 seconds with a complete 900-frame profile after 120 warmup frames, normal pipelined rendering and no panic. Validation stopped the application after this bounded run. Local ad-hoc signature and DMG checksum verify. No push, public release or notarization.

Evidence is retained under ignored `dist/electrolysis-validation/`: configurations, original/replay/legacy numeric bundles, eight PNGs and metadata, commands/statuses, test/build/package logs and ordinary run/profile. Its checkpoint manifest records source revision and artifact hashes. This verifies deterministic reduced chemistry and presentation, not hardware calibration, gas purity, tank pressure or physiological safety. Phase 9, physical greenhouse coupling and the full master directive remain incomplete.
