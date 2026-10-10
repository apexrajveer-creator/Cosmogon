# Persistent terrain-foot shear verification

Local macOS arm64 development checkpoint, 2026-10-10. Public release remains 1.0.0. No push, remote CI, notarization or cross-platform manual verification is claimed.

## Final evidence

`cargo test --release --workspace`: **505 passed, one existing ignored, zero failures**. Release all-target clippy, app/CLI build, macOS package, strict ad-hoc signature and disk-image checks pass. Existing clippy warnings remain. All **171 frozen Rust source/test files** match the package. The immutable master SHA256 remains `03dcdfaf0a4fd5a5548cde2fb5d700fd67df648e8691c5322117ca652ff94711`; the legacy golden `0xf2c4140a45ae509b` passes unchanged. Optional enabled state deliberately uses world-flight version 3, terrain-contact version 2 and separately pinned foot-shear version 1.

**67 CLI/native cases** check actual rotating terrain contact, retained per-foot shear/storage/work, finite cargo, structural refusal at the prior supported state, finite orbital burn/exhaust closure, full branch and nested approach replay, strict flags, protected outputs and corrupted histories/counters/CSV/pins. Native terrain, failure and orbital exports match every original CLI file byte for byte. Original propulsion, spherical world-flight, rotating approach and smooth-sliding terrain-contact recordings still replay. Raw strain CSV and independently computed tangent storage agree; full Rust readers separately recompute all observations and numeric exports. Evidence, inputs/outputs, scripts, checks and source hashes are retained under ignored `dist/vehicle-static-shear-validation/` (305 hashed files).

All six final packaged **2940×1660** PNGs were inspected: loaded terrain shear, structural refusal, finite orbital burn, staged persistent-shear controls/geometry refusal, advancing live contact and old flight graphs. Status, model limits and the main dock are readable. Expanded hardware/foot detail and lower graphs require scrolling. No manual button/keyboard/scrolling, narrow-window or accessibility journey is claimed.

The ordinary live-contact session ran **48.448818 seconds**, retained **900 samples after 120 warmup frames**, preserved pipelined rendering, advanced actual craft/contact clocks and kept cargo aboard. QA sent SIGTERM to its own process (exit -15); this is not graceful-close or a performance certification. No panic occurred. GPU timing/VRAM remain unavailable. The checked app SHA256 is `d4ba29b630bbd0b0dc3a92e4ac42b82da86249d96327da367b65c0c8ce276edb`.

## Measured numerical limits

The same four-second terrain branch with requested maximum steps **0.02, 0.005 and 0.00125 seconds** used **1024, 1024 and 4096 substeps**. The first two settings produced identical endpoints because a stronger coupled bound dominated. Final velocity differences were **0** and **0.002766116 m/s**; cumulative actual contact balance residuals were **-0.236071, -0.236071 and -0.241068 J**. These are retained measurements, not monotonic convergence, a universal error bound or field calibration. Endpoint constitutive balance and actual midpoint mechanical balance remain separate. Endpoint numerical damping is not measured ground friction.

The eight preceding component tests cover objective history transport, static force retention, independent supported-mass equilibrium, changing directions, separation and one-dimensional constitutive refinement. Four integrated simulation tests cover actual coupled motion/storage, strict versions/pins/history, exact save/chunk/full-record replay and exclusion of pre-contact displacement. A native frozen-request test preserves the selected law and exact exports after later draft edits. Those checks do not establish footprint stability or certified landing.

## Findings and remaining scope

Development fixtures initially used nonexistent APIs and were corrected to the actual advance/study interfaces. Full checked development tests exposed an existing south-hemisphere rendering seed overflow; explicit wrapping now preserves the released modulo-u64 values without adding a visual feature. Final development and release suites pass. No Rust changes followed the frozen release pipeline.

Persistent shear is now connected to motion and saved/replayed native evidence. Stable footprint/load analysis, attachment, supported surface operations and physically co-located cargo transfer remain incomplete. Cargo stays aboard. Entry/guidance, changing-cargo mass epochs, handling/thermal/power hardware, calibration and uncertainty remain unfinished. Overview terrain and fixed-offset feet are authored reduced models. Graphics remain queued; phase 10 and the complete master directive remain incomplete.

[Guide](VEHICLE_STATIC_SHEAR.md), [mechanics and source memo](STATIC_FOOT_SUPPORT_DESIGN.md), [next support/delivery mechanism](VEHICLE_SUPPORT_NEXT.md).
