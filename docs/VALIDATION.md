# Validation — native exploration, photography, sound and notifications

Host: macOS 26.3.1, arm64. This is local validation, not a claim that new Windows/Linux CI has run.
The release binary is built from the active branch; the public release remains 1.0.0.

## Automated checks

* Baseline: `cargo test --release --workspace` — 173 passed, 1 intentionally ignored.
* New coverage: profile bounds/default scenario; nearest-rank percentiles/outliers/invalid inputs;
  bounded buffers/reset between universes; absent metrics and overwrite protection; old settings
  and preference round trip; headless recording → JSON export after warmup without mutating Universe.
* New recording integration test also verifies paused simulation cost is zero and missing GPU/VRAM
  readings do not become fabricated zero measurements.
* Golden simulation fingerprint remains `0x579903fd05a28945`.
* `cargo clippy --release --workspace` completes; no warnings in the added performance code.
  Existing warnings remain in core math, physics, simulation, renderer and older UI/persistence code.
  Existing dependency `block 0.1.6` also reports a Rust future-compatibility warning.
* CI now runs release workspace tests (including these native-app boundary tests), the existing
  determinism check, and the release app build on its configured OS matrix.

Final workspace run: **192 passed, 1 ignored, 0 failed** (2026-10-06).
Nineteen added tests cover performance, fallback counters, settings, camera continuity, inertial
integration, finite motion/orientation across scales, viewpoint persistence and unchanged Universe
state after view/lens restoration. Audio/notification coverage includes realistic silence, independent
alert gain, old/new preference round trips, quiet/category routing, bounded bursts, pruning and
silent rewind reset. Clippy completes with the pre-existing warnings described above.

A native report inspection caught unsupported zero-valued Metal GPU and process-memory counters.
The export now filters unsupported GPU backends and treats zero resident-memory fallback as
unavailable process data. A regression test preserves legitimate zero CPU when memory is available.
Earlier pre-fix reports under `/tmp/cosmogon-native-foundation` must not be used as GPU/CPU evidence.

## Native verification protocol

1. Build `cargo build --release -p cosmogon` and stop compilation before measurements.
2. Run `scripts/profile-native.sh` with a new output directory. Inspect all three actual PNGs.
3. Inspect exported JSON sample counts, finite values, distribution ordering, scene context and
   unavailable GPU/VRAM values; retain reports with their capture/log companions.
4. Verify export failure returns a nonzero status and does not overwrite an earlier report.
5. Run an ordinary app session for at least 30 seconds and inspect logs. Separately exercise keyboard,
   viewpoint buttons, audio and the panel's export button manually. Profiling must preserve ordinary
   input/audio outside capture mode.

## Captures and recorded workloads

The native suite under `/tmp/cosmogon-final-verified` contains nine inspected captures: Navigate,
Photo, Viewpoints, free-flight Mars and galaxy, Inbox, Sound, Routing and Home Settings. The Photo
capture uses 35° FOV and +1.5 EV; the Realistic sound capture shows disabled exterior effects.
The three workload PNGs were also inspected. Each performance report has 600 raw samples matching
its exported means and nearest-rank percentiles; all values are finite. All associated photo
sidecars remain separate from the performance reports.

| Workload | Mean frame interval | p95 | p99 |
|---|---:|---:|---:|
| Saturn with Performance panel | 16.92 ms | 17.63 ms | 26.23 ms |
| Mars surface | 20.39 ms | 44.39 ms | 68.08 ms |
| Galaxy | 53.08 ms | 68.59 ms | 86.02 ms |

These are single local recordings on Apple M3 at 2940×1660 with Ultra settings, not repeated benchmarks or
evidence of a speedup. The backend is Metal; GPU timing is unavailable, VRAM is null and unsupported
zero process counters are omitted. Render CPU snapshots are not frame/GPU cost.

## Limits

* The sandboxed launch cannot access the GPU; native captures require approved host GPU access.
* `xctrace` is installed as a shim but full Xcode is not selected/installed here, so Instruments
  GPU/VRAM profiling is unavailable. Frame intervals are not substituted for GPU timing.
* This change does not validate new physics or claim performance improvement. It adds measurement
  capability. Simulation/save schemas and golden state are unchanged.
* No public release, Developer ID signing/notarization, remote CI run or cross-platform manual QA is claimed.

## Surface repair evidence

A native free-flight Mars capture exposed terrain precision artifacts at 2 m clearance. Free flight
now uses the existing orbit terrain envelope (60 m), matching near-plane/proximity bloom behavior.
The repaired 760-frame native capture `/tmp/cosmogon-mars-clearance.png` was inspected: grid artifacts
are absent. Its report records 60 m clearance, finite camera state, 600 frames and unavailable GPU
counters. This is endpoint protection, not swept collision or a physical walking/landing model.

## Interactive verification limit

The native UI automation binding stalled and was interrupted; no successful automated keyboard/button
journey is claimed. The ordinary earlier test app ran for over 30 seconds without a panic; its log also
contains an input-device force-feedback timing warning. Final captures and a fresh ordinary bounded
run use the final binary. Camera/viewpoint routing is covered by app boundary tests; real manual
keyboard, save/restore buttons and hardware audio listening remain follow-up checks.

## Final failure boundaries and package

The final packaged binary passed four native failure cases, each returning status **1**:
existing performance report, screenshot folder obstructed by a file, missing save during screenshot
startup, and missing save during profile startup. The earlier report's SHA-256 stayed unchanged;
failed loads created neither a PNG nor a report. Native verification found two defects: the entry
point discarded Bevy AppExit, and capture startup did not exit after a failed load. Both are repaired.

The packaged application was launched from `/tmp` with free flight, Hybrid mode, the Sound panel
and ordinary input enabled (no `--capture`). It ran **49.39 seconds**, exported **2400** valid samples,
returned status **0**, and logged no panic. Quiet mode muted audio for the run; hardware listening and
button/keyboard journeys remain outside this evidence. The final source also passed all 192 release
workspace tests and clippy with existing warnings only.

`scripts/package-macos.sh` produced the native arm64 app and a 24 MiB disk image. The launchable
cache copy passed strict signature verification again after native runs. The disk image passed its
checksum verification; its standalone app was mounted read-only, strictly signature-verified and
unmounted. Temporary staging was removed. This remains an ad-hoc signed development build, not a
new public release or a notarized distribution.

Local evidence is retained in ignored `dist/validation/`: inspected PNGs, photo metadata, raw reports,
failure logs/results, and final check logs. The source instructions and evidence summary are committed;
generated binaries and captures are not tracked.

Frozen habitat starting-world selection: [verification](FROZEN_HABITAT_ORIGINS_VALIDATION.md), 435 passed plus one existing ignored, 39 cases, seven inspected packaged layouts and a 48-second normal run. Original and request-time live results independently reproduce; solver/saves/golden/graphics stay unchanged.


Finite supply-vehicle checkpoint (2026-10-10): **446 passed plus one existing ignored**, 59 distinct cases, seven inspected packaged layouts, six required checks and a 48-second normal packaged run. [Full evidence and limits](SUPPLY_VEHICLE_VALIDATION.md). No world/save/source/golden or graphics change; flight/delivery and the full directive remain incomplete.


Physical finite world-flight checkpoint (2026-10-10): **467 passed plus one existing ignored**, 63 distinct cases, nine inspected packaged layouts, six required checks and a 48.296151-second ordinary packaged run. [Evidence and limits](WORLD_VEHICLE_FLIGHT_VALIDATION.md). Enabled craft state changes with model pins; absent legacy golden remains `0xf2c4140a45ae509b`. Landing, delivery, graphics and the full directive remain incomplete.


Rotating vehicle arrival checkpoint (2026-10-10): **479 passed plus one existing ignored**, 50 distinct cases, six inspected packaged layouts, six required checks and a 48.312429-second ordinary packaged run. [Evidence and limits](VEHICLE_ARRIVAL_VALIDATION.md). Exact prior flight/propulsion evidence, world/save/physics and golden remain unchanged; landing/delivery, graphics and the full directive remain incomplete.

Terrain and rigid-body contact checkpoint (2026-10-10): **492 passed plus one existing ignored**, 57 cases, six inspected packaged captures and a 48.261886-second live-contact run. [Evidence and limits](VEHICLE_CONTACT_VALIDATION.md). Separate enabled-state pins; legacy golden/records unchanged. Static support, physical delivery, graphics and the master directive remain incomplete.

Persistent terrain-foot shear checkpoint (2026-10-10): **505 passed plus one existing ignored**, six release/package checks, 67 CLI/native cases, six inspected packaged captures and a 48.448818-second live-contact run. All 171 frozen Rust files match. [Evidence and numerical limits](VEHICLE_STATIC_SHEAR_VALIDATION.md). Optional new state has separate pins; legacy golden and old recordings remain unchanged. Footprint/stability, attachment, delivery, graphics and the master directive remain incomplete.

Actual footprint/ground-hold checkpoint (2026-10-10): **514 passed plus one existing ignored**, six release/package checks, 45 cases, eight inspected final captures and a 60.053035-second ordinary run. All 173 frozen Rust files match; new style warnings corrected. [Evidence and numerical/source limits](VEHICLE_FOOTPRINT_SUPPORT_VALIDATION.md). Existing flight formats/golden remain unchanged; surface operations, physical delivery, graphics and the full directive remain incomplete.

Cargo mass-epochs checkpoint (2026-10-12, commit `5943ead`): **532 passed plus one existing ignored**, no new clippy warnings, fresh app/CLI release builds, the full CLI loop with protected rerun and tamper refusals, native preflight refusals and three inspected captures. [Evidence and limits](CARGO_MASS_EPOCHS_VALIDATION.md). Mass-accounting bench only; flight integration, storage and delivery remain incomplete.

Surface-operation window checkpoint (2026-10-15): **539 passed plus one existing ignored**, no new clippy warnings in touched files, a fresh CLI release build, a seven-case end-to-end loop (millimeter-gap geographic start on real 4ppd MOLA height, coast plan, 20 s study `flying` throughout, window growing to 9.38671875 s ≥ 5 s required with 5800 supported samples, replay-exact `flight-inspect`) and two negative flag cases. [Evidence and limits](VEHICLE_OPERATION_VALIDATION.md). Enabled operation state uses a separate conditional save pin; absent legacy golden remains `0xf2c4140a45ae509b`. The window is a modeled envelope, not attachment, storage, handling or delivery; graphics and the full directive remain incomplete.

Cargo-mass-in-flight checkpoint (2026-10-15): **548 passed plus one existing ignored**, no new clippy warnings in touched files, a fresh CLI release build, and an eight-case end-to-end loop (ledger definition → study manifest `cargo` block → replay-exact `flight-inspect`, authored mid-study removal dropping held water 5000 → 4200 kg with an event at exactly 0.5 s, contact+operation composition, three negative cases). [Evidence and limits](VEHICLE_CARGO_FLIGHT_VALIDATION.md). Opt-in ledgers use a conditional `cosmogon-vehicle-cargo-flight` save pin; absent legacy golden remains `0xf2c4140a45ae509b`. Removal moves mass out of the craft with accounting; storage, recipient capacity, transfer and delivery remain incomplete, and graphics remain queued.
