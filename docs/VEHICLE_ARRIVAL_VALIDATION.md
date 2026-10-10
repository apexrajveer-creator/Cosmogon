# Rotating vehicle arrival verification

Local macOS arm64 development checkpoint, 2026-10-10. Public release remains 1.0.0. No push, remote CI, release, notarization or cross-platform manual verification is claimed.

## Final evidence

The final `cargo test --release --workspace` passes **479 tests plus one existing ignored test**, zero failures. Release all-target clippy, native app/CLI build, macOS package, strict ad-hoc signature and disk-image checksum checks pass. Existing warnings remain; clippy also suggests a style-only initializer change in one new test fixture. No new production-code warning is reported. All **168 frozen Rust source/test files** match the final checked and packaged build. The master directive SHA-256 remains `03dcdfaf0a4fd5a5548cde2fb5d700fd67df648e8691c5322117ca652ff94711`.

50 distinct CLI/native workflow cases verify geographic source conversion, fractional craft/world clocks, rotating ground-relative velocity, finite mass/energy, terrain source support, original terminal-frame retention, pole unavailability, complete exact replay, protected output and strict input/source corruption refusals. Native approach exports match all ten CLI files byte for byte, including all six original flight files. The previous flight and propulsion records still verify unchanged. Cases/logs/check durations/hashes are retained in ignored `dist/vehicle-arrival-validation/`.

Six final packaged 2940×1660 PNGs were inspected: active recorded approach, historical Moon contact, Mars pole, actual live approach, unavailable historical live contact, and the previous Mars flight analyzed in the new frame. Numerical readings remain visible before expandable controls/source detail, and the main action dock stays clear. Contact shows the original craft/source/world clocks; negative overview-terrain clearance is visible separately from the old spherical boundary. This exposes the terrain-contact gap, rather than declaring a landing.

The ordinary packaged live-Moon session ran **48.312429 seconds**, retained **900 samples after 120 warmup frames**, and preserved normal pipelined rendering. The QA script deliberately terminated its own app process with SIGTERM (exit -15). This is neither a graceful-close test nor a benchmark/speedup claim. No panic occurred; unavailable GPU timing/VRAM remain unavailable.

Unit coverage establishes transform inverse/terrain orientation, tilted/retrograde and tidally locked rotation, position derivatives including dynamic rails and excluded galactic origins, a stationary surface point, geographic velocity, pole/source/mass evidence, invalid geometry and pending edits, retained contact frames, source corruption and native frozen worker/disk/output boundaries. An initial CLI boundary test found required-input checks were placed before flag parsing; those checks now run after parsing, and the repaired complete suite passes. Initial failure/repair logs are retained separately from final checks.

## Limits and continuation

Readings use the application's existing modeled orientation/source motion. Spherical-contact centres can be interpolated; velocity is an ephemeris derivative and does not recover a historical contact chord slope. The first terminal source-frame observation is retained by the approach replay, not by old live vehicle state. No IAU/SPICE orientation precision, gear force/stopping distance/stability, field calibration or measured uncertainty is claimed.

Expandable geographic/file controls and complete source JSON require scrolling. Long energy numbers can wrap units onto another line; east longitude very close to the wrap boundary can display as 360° after rounding. No manual button/keyboard/hover/scrolling/clipboard, narrow-window or accessibility journey was completed. No new approach graph is implemented; original flight graphs remain available.

No authoritative world/save/physics schema, existing flight format or golden change is introduced. Golden `0xf2c4140a45ae509b` passes. Geographic creation remains an explicit authored initial condition through existing ordinary edits; cargo remains aboard and no habitat is granted. Terrain-resolved contact response, touchdown/attachment, entry, guidance, cargo delivery, graphics and the full directive remain incomplete. The next build must evolve real contact forces on modeled terrain, not reinterpret old spherical impacts or attach a reference fixture on a timer.

[Guide](VEHICLE_ARRIVAL.md), [research and next contact mechanism](LANDING_AND_DELIVERY_RESEARCH.md).
