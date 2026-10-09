# Recorded pressure explanations — design before implementation

Following `89160c0`, build a read-only bridge between finite cabin gas stocks and the existing recorded pressure curves. The nominal breach example can gain pressure while losing gas: temperature and changing composition also matter. A student should be able to inspect the four gas components; a researcher should receive the exact inputs, formulas, units, identities and limitations behind each number.

NASA Glenn's [equation of state](https://www.grc.nasa.gov/www/k-12/airplane/eqstat.html), updated May 13, 2021 and checked October 9, 2026, supports ideal-gas `pV=nRT` with absolute temperature. The application assumes ideal, uniformly mixed gas at the retained lumped cabin temperature. Species molecular masses and R match the existing habitat model. This is modelled pressure, not a measured atmosphere or safety assessment.

For two actual saved observations, use the explicitly ordered algebra inventory → temperature → volume:

- Each species inventory term is `(n1_i - n0_i) R T0 / V0`.
- Temperature is `n1 R (T1 - T0) / V0`.
- Volume is `n1 R T1 (1/V1 - 1/V0)`.

The signed terms telescope to `p1-p0`; expose the numerical residual. Changing the ordering assigns interaction terms differently. This is a mathematical accounting convention, not unique physical attribution. Do not interpolate missing minutes, reconstruct crew/process start stocks, allocate total loss to a single cause, invent uncertainty, or turn structural consistency into predictive validation.

Default reference is the previous stored observation. Also offer first stored and explicitly chosen indices. Observation zero without a reference has actual components but unavailable change accounting; the same explicit index gives a real zero difference. Keep world observation time distinct from completed habitat time. Paired roots remain supported by the existing reader: pressure explains branch A (run-000), labelled explicitly, not a factorial effect or B−A comparison.

When an actual breach budget is retained, a separate pane accounts only for that resolved substep, using its retained initial gas/temperature, escaped quantities, final temperature and completed configuration volume. Preserve status, actual integrated seconds and requested seconds even when boundary/work/domain support is partial. Do not present that substep as the complete observation interval. Controls edited after a budget must not replace its volume or inputs.

Implement one engine-independent derivation shared by CLI, native tables, captures and protected exports. No simulated state/schema/law changes, new dependencies or golden re-pin. Exports contain exact selected/reference samples, source-pinned recording summary, numeric component and signed-term CSV, full substep evidence when available, and a completion manifest written last in a new folder. Native worker exports capture the report at request time and protect existing outputs. Pending export state is visible; invalid reference indices preserve loaded observations.

CLI `habitat-pressure --recording ORDINARY_BRANCH_FOLDER --observation INDEX [--reference-index INDEX] [--output NEW_DIR]` is strict, bounded and uses the same validated recording reader. Native Recordings → Pressure components offers components/change/substep/evidence panes, actual sample selection and reference choices. Capture aliases and optional explicit reference index permit reproducible preflight and layouts. Missing evidence remains unavailable.

Verification after the connected build: analytic isolated and combined terms, ordering dependence, zero gas/missing/explicit same reference, invalid indices/domains, exact source retention, protected exports, breach substep versus edited current volume, legacy ordinary and paired studies, exact native/CLI reports, inspected final PNGs, required workspace tests/clippy, macOS package/signature/DMG, and a normal 30–60 second packaged run. Manual input and assistive technology are separate unverified claims. Field calibration, uncertainty, full causal history and the master directive remain incomplete.
