# Controlled moisture equipment studies

Prepared before the next connected implementation. Finite moisture checkpoint adds state, phase heat, source-domain limits and actual recordings. Researchers also need to compare equipment/partition assumptions from the same world, not merely inspect separate hand-built runs.

Extend the existing typed ordinary-edit sweep engine with airborne fraction, humidity setpoint, dehumidifier power, condensation rate, electricity per kilogram and condensate capacity. Require already enabled optional moisture equipment. Reuse all-branch validation, held initial inventory, protected manifests, complete source-pinned runs and exact-time/interval comparisons. Do not auto-enable equipment or reinterpret a retained initial vapor stock when changing its future emission fraction.

Native Experiments selects the same typed parameters and existing worker/compare path. Add actual vapor, condensate, humidity and phase-heat metrics already provided by moisture recordings. Compare one setting at a time; no automatic optimization, interaction inference, empirical uncertainty or recommended RH. The source of each numerical relationship is the same bounded NIST/NASA model, with explicit assumed equipment. A downstream optimum depends on unmodeled purification, constraints and crop physiology.

Check a controlled emitted-vapor versus sensible-heat tradeoff, rate/power limitation, full-store validation before any output, disabled equipment rejection, repeated exact trajectories, unchanged common initial world and actual native comparison metadata/captures.

Clarify native completed-interval duration and whole-model disable semantics: disabling the optional mechanism restores future crew output to legacy liquid collection while retaining held phase stocks. Zero power/rate keeps emissions active with an idle collector. Put optional phase heat terms ahead of aggregate WHY sums so the new driver is visible in initial views.

Methodological source: [NIST/SEMATECH one-factor-at-a-time limitations](https://www.itl.nist.gov/div898/handbook/pri/section2/pri212.htm), checked 2026-10-09. Such studies do not account for factor interactions. The existing NASA/NIST source equations and domains remain as documented in HUMIDITY_RESEARCH; no scientific law or source coefficient changes here.
