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
