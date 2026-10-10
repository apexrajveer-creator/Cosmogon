# Terrain and rigid-body contact verification

Local macOS arm64 development checkpoint, 2026-10-10. Public release remains 1.0.0. No push, remote CI, notarization or cross-platform manual verification is claimed.

## Final evidence

`cargo test --release --workspace`: **492 passed, one existing ignored, zero failures**. Release all-target clippy, native app/CLI build, macOS package, strict ad-hoc signature and disk-image checks all pass. Existing clippy warnings remain; no warning-free claim. All **170 frozen Rust source/test files** match the checked packaged build. Master directive SHA256 remains `03dcdfaf0a4fd5a5548cde2fb5d700fd67df648e8691c5322117ca652ff94711`; legacy golden `0xf2c4140a45ae509b` passes unchanged. Enabled contact state deliberately uses additional separately pinned model/capability versions.

**57 CLI/native cases** verify actual below-spherical-base MOLA contact, signed new-mode geography, four-foot force/attitude/work, retained finite cargo, structural failure at the prior supported state, central finite burn/exhaust momentum, complete world/approach replay, protected outputs, strict invalid geometry/model/flag refusal and corrupted contact CSV/force/work/capability rejection. Native terrain/failure/orbit exports match every CLI file byte for byte. Original propulsion, world-flight and complete approach records still replay. Evidence, exact inputs/outputs, source hashes, checks and scripts are retained under ignored `dist/vehicle-contact-validation/`.

Six final packaged 2940×1660 PNGs were inspected: loaded terrain, structural failure, orbital finite burn, staged contact controls/geometry refusal, moving live contact and old flight. Status, source limitations and the main action dock remain visible. Detailed hardware, foot evidence and lower graphs require scrolling. No manual button/keyboard/scrolling, narrow-window or accessibility journey is claimed.

The normal live-contact session ran **48.261886 seconds**, retained **900 samples after 120 warmup frames**, preserved pipelined rendering, advanced actual craft/contact clocks and kept cargo aboard. QA deliberately sent SIGTERM to its own app (exit -15); this is not graceful-close or performance certification. No panic occurred. GPU timing/VRAM remain unavailable.

## Measured numerical limits

The same four-second terrain branch with requested maximum steps 0.02, 0.005 and 0.00125 seconds actually used **9293, 9239 and 9246** substeps: coupled contact bounds dominate those requested maxima. Final velocity differences were **0.000003052594** and **0.000009333110 m/s**; cumulative contact balance residuals were **0.366839, 0.795963 and 0.996826 J**. These are recorded measurements, not monotonic convergence, a universal error bound, or field calibration. Geometry changes and coordinate precision contribute to contact balances. Dedicated anisotropic free-Euler tests separately verify step convergence; they do not validate terrain contact.

Tests cover actual moving-foot torque/work, finite unilateral release, tilted rotating terrain, dyadic/elapsed-clock steps, save/chunk exactness, frozen worker hardware, finite exhaust assumptions, inertia triangle/bounding extent, unsupported low-speed friction resolution, source-edit discontinuities and refusal to reinterpret old spherical impact as new terrain mode. Independent complete readers check every retained observation, CSV, manifest and endpoint world.

## Development findings and remaining scope

Earlier fixtures advanced absolute second 1 instead of the actual dated origin; those apparent non-motion passes were rejected and repaired. The first integrated 491-test package was superseded by friction-resolution, inertia-extent, caption and terminal-semantics fixes. Its live-capture assertion also incorrectly assumed speed preset 0 paused the world; 0 means real time, so live capture legitimately advances while recorded branches remain immutable. Final 492-test checks and fresh runtime evidence supersede that package. No source changes followed the final frozen build.

Smooth sliding friction does not establish static sticking, footprint support, attachment, certified landing or cargo access. Actual contact continues evolving. Explicit same-site conservative cargo transfer, atmospheric entry, guidance, hardware/thermal/power calibration, full engine energy accounting and uncertainty remain incomplete. Modeled terrain is an overview/height-field approximation; the normal probe is not new measured resolution. Graphics remain queued. Phase 10 and the full master directive remain incomplete.

[Guide](VEHICLE_CONTACT.md), [mechanics/source memo](VEHICLE_CONTACT_DESIGN.md).
