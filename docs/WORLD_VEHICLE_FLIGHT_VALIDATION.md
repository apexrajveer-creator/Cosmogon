# Physical finite world-flight verification

Local macOS arm64 development checkpoint, 2026-10-10. Public release remains 1.0.0; no remote push, release, notarization or cross-platform manual QA is claimed.

## Final checks

`cargo test --release --workspace`: **467 passed, one existing ignored, zero failed**. Release all-target clippy, app/CLI build, macOS packaging, strict ad-hoc signature verification and disk-image checksum verification all pass. Existing warnings remain; no new warnings occur in added flight implementation. Final check durations/logs and hashes are retained in ignored `dist/world-vehicle-flight-validation/`. All 147 frozen production Rust files match the source built for verification. The immutable master SHA-256 remains `03dcdfaf0a4fd5a5548cde2fb5d700fd67df648e8691c5322117ca652ff94711`.

## Behavioral evidence

63 distinct CLI/native cases cover actual gravity and finite burns, coasting, empty/exhausted fuel, opposing commands, fractional world/craft clocks, early swept spherical contact, unsupported numerical motion, bounded sampling/work/payload, protected files, strict origins/source pins and malformed/corrupted evidence. Complete flight reopening independently reproduces every retained observation, final world, manifest and exact CSV bytes. Native completed exports equal all six CLI files byte for byte. The previous propulsion bench reopens unchanged under the new binary. Unit coverage includes save/reload, ordinary edit undo/redo, reserved scheduler slots, terminal clock chunking, source-discontinuity stopping, frozen worker requests and disk rechecks.

Nine final packaged 2940×1660 PNGs were inspected: flight results, fractional clocks, spherical contact, unsupported motion, live flight, live contact, pinned-definition export, reopened export and legacy propulsion. Captures expose actual stocks/clocks/phase rather than declaring contact a landing. The selected numerical readings now precede the plot; the workspace leaves room for the main action dock.

The ordinary packaged saved-flight session ran **48.296151 seconds**, retained **900 samples** after **120 warmup frames**, and kept normal pipelined rendering enabled. It was deliberately terminated by the QA script (SIGTERM, exit -15); this is not a graceful-close test or a performance-speedup claim. GPU timing/VRAM remain unavailable rather than invented.

## Development failures and limits

An initial disk-image creation reported insufficient space despite substantial free storage; the identical packaging retry passed. Its failure trace is retained. Default sandbox native startup lacks macOS graphics access; final native checks ran with approved host access. A development review found terminal scheduler parking depended on the prior fractional caller clock; parking now occurs after the actual due time is assigned, and a whole-world chunking/save regression passes.

Contact and long unsupported-reason layouts can put the lower graph/range below the initial fold; controls, vectors and source details require scrolling or expansion. No successful manual pointer/keyboard/hover/clipboard, narrow-window, scrolling journey or assistive-technology audit is claimed. Captures and app boundary tests provide narrower evidence.

Enabled flight changes authoritative saved state and fingerprints with explicit model pins. Absent legacy worlds retain golden `0xf2c4140a45ae509b`. The model is a bounded Newtonian test-particle operator using existing sampled source motion, not calibrated launch/landing hardware. Contact is spherical and terminal; cargo remains aboard. Terrain landing, entry/heating, docking, actual stock loading/delivery, vessel graphics, uncertainty and full phase 10 remain incomplete. Graphics work stays queued.

[User guide](WORLD_VEHICLE_FLIGHT.md), [design/source memo](WORLD_VEHICLE_FLIGHT_DESIGN.md), [next landing/delivery mechanism](LANDING_AND_DELIVERY_RESEARCH.md).
