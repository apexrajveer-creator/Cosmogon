# Vehicle footprint and ground-hold verification

Local macOS arm64 development checkpoint, 2026-10-10. Public release remains 1.0.0. No push, remote CI, notarization or cross-platform manual verification is claimed.

## Final evidence

**514 release workspace tests pass, one existing ignored, zero failures.** Release all-target clippy, app/CLI build, macOS packaging, strict ad-hoc signature and disk-image verification pass. Three newly introduced style warnings were fixed before the final pipeline; existing unrelated warnings remain. All **173 frozen Rust source/test files** match this package. The unchanged legacy golden is `0xf2c4140a45ae509b`; immutable master SHA256 is `03dcdfaf0a4fd5a5548cde2fb5d700fd67df648e8691c5322117ca652ff94711`.

**45 CLI/native cases** cover independent original-world support derivation, actual per-foot force sums and 29-column CSV readings, default and strict authored envelopes, unavailability/terminal history, corrupted numbers/CSV/definition/manifest/nested flight, protected destinations, strict input conflicts and five pre-GPU refusals. Three native terrain/failure/orbit reports match all original command-line files byte for byte. Original spherical/approach and smooth-contact reports still replay; smooth contact is explicitly excluded from persistent static-support interpretation.

All **eight final 2940×1660 packaged PNGs** were inspected: terrain, structural refusal, orbital burn, reopened support, initially unloaded feet, strict limits, advancing live contact and old smooth contact. The footprint now precedes detailed numbers following inspection of the first build's clipped default viewport; the live capture shows the full diagram and legend. Some recorded views still require scrolling for the legend and lower values. An isolated copy separately demonstrated scrolling down to numeric values and back to the full hull; its additional command-line PNG is retained separately. A later drag attempt found that bounded test app already closed. No complete manual button/keyboard, narrow-window or accessibility journey is claimed.

An ordinary session ran **60.053035 seconds**, retained **900 samples after 120 warmup frames**, preserved pipelined rendering, advanced actual craft/contact clocks and kept cargo aboard. QA sent SIGTERM to its own process (exit -15); no graceful-close or performance certification is claimed. An earlier 48-second run overlapped the separate interactive inspection and did not finish its 900-frame profile before QA terminated it. Its log is retained; the isolated 60-second retry completed. No panic occurred. GPU timing/VRAM remain unavailable. Checked app SHA256: `e1c6726ae7b2c0a075dce439e3e52a56eb93882eb7656f92715e49de24c30c35`.

Ignored `dist/vehicle-footprint-support-validation/` retains the final pipeline, all cases/captures/recordings, external inputs, frozen source copies/hashes, earlier verification and scripts. Its checkpoint manifest hashes the retained evidence and records the local commit separately.

## Numerical and verification findings

Independent Python summation differed from Rust accumulation by at most approximately **9.1e-13 N** in these fixtures. The independent cross-language check uses `math.fsum` and a declared 16-epsilon bound scaled by the actual sum of absolute terms. Rust replay, recorded JSON and raw CSV comparisons remain exact. This is a rounding comparison, not a force-accuracy allowance.

The source-frame centred velocity derivative differs from an independent Kepler acceleration reference by approximately **1.05e-6 m/s²** at the tested legacy Mars rail clock. The test declares a **0.1% relative source-operator budget**; half-step refinement and roundoff indicators are distinct from physical/source accuracy. The existing DMath orbital state operator and golden were preserved. No rigorous acceleration bound or improved ephemeris is claimed.

An early package attempt reported “No space left on device”; the inspected filesystem had about 100 GiB available and a packaging-only retry succeeded. No owner cache, mounted image or process was removed. A later permission-review timeout on a combined command was resolved by running compilation normally and packaging separately. This was not evidence that the build was unsafe.

Six simulation tests and three app/CLI boundary/worker tests add geometry, independent compliant-wrench moments, actual coupled readings, source/edit refusal, exact replay and frozen-definition support. Applied force points and modeled ground sites remain distinct; positive weighted-site margin is not proof of stable contact. Stopped unsupported history retains an unavailable reason and is never reconstructed from the later live world.

## Remaining mechanism

Instantaneous authored envelope inclusion is neither attachment nor future stability, landing permission or cargo access. Actual supported operating duration, finite handling/storage, changing-cargo mass/inertia/heat/momentum, physical same-habitat transfer, launch/entry/guidance and empirical calibration remain unfinished. Graphics stay queued. Phase 10 and the full master directive remain incomplete.

[Guide](VEHICLE_FOOTPRINT_SUPPORT.md), [next support/delivery requirements](VEHICLE_SUPPORT_NEXT.md).
