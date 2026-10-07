# Performance measurements

The Performance panel is available from **View (eye icon) → Performance** or by searching
**Performance** in Cmd/Ctrl+K. It stays closed by default and follows interface/photo hiding.
It records the most recent 600 frames, shows average/p95/p99 and the slowest frame, and exports
local JSON to the platform's `Cosmogon/reports` data folder. Exports never change universe state.

## What the numbers mean

* **Frame ms:** real elapsed frame intervals, including VSync and scheduling/presentation waits.
  Frame time is not CPU utilization and is not GPU execution time. FPS = 1000 / mean frame ms.
* **Simulation ms:** elapsed wall time inside `Universe::advance_to`, reported by the existing
  simulation driver. Paused frames record zero, not the previous frame's work.
* **Percentiles:** nearest rank over the bounded recorded window; p95 means 95% of sampled
  frame intervals were no larger. Startup/warmup is excluded from automated frame statistics.
* **Last universe load:** from scheduling creation/loading to the main thread receiving the result;
  includes initial simulation and requested save/prehistory work. It excludes subsequent GPU
  pipelines and async surface bakes. **Startup to observing** starts at `main` and ends when the
  app enters Observing. Neither number claims the fully refined scene has finished loading.
* **Process metrics:** opt-in Bevy measurements, updated asynchronously. CPU is normalized over
  all logical CPUs; memory is process resident memory in GiB, not percentage and not VRAM.
  If process lookup is denied and the backend supplies zero fallback memory, both process counters
  are omitted. An unavailable CPU counter is not reported as 0%.
* **Render passes:** opt-in snapshots from Bevy. CPU values measure command recording spans;
  GPU values appear only when timestamps are supplied. Spans can overlap: do not sum them.
  Exports include the age of each counter; repeated readings are not treated as new samples.
* **VRAM:** currently unmeasured. On unified-memory hardware a process RSS/footprint counter
  must not be presented as dedicated GPU memory usage.

Enable **Detailed measurements at next launch** in the panel or Settings for process and
render counters. The preference persists in user settings; old settings files default to off.
The developer `--profile` flag enables details for that launch without altering the preference.

Implementation uses the pinned Bevy 0.18.1 APIs and their source:
[render diagnostics](https://docs.rs/bevy/0.18.1/bevy/render/diagnostic/struct.RenderDiagnosticsPlugin.html),
[process diagnostics](https://docs.rs/bevy/0.18.1/bevy/diagnostic/struct.SystemInformationDiagnosticsPlugin.html).
Bevy documents timestamp support on Vulkan/DX12 and CPU-only spans on Metal. The report checks
both the backend and actual supplied metrics. Unsupported Metal bloom timestamp placeholders are
omitted, even if Bevy publishes an `elapsed_gpu` key. No GPU timing is inferred from frame duration.

## Reproducible native recordings

Build first. Run one app instance at a time, with no compilation or competing workload. Use the
same seed, scene, focus, camera orientation, speed, graphics settings, window size/display scaling
and presentation mode for comparisons. Reports include these scene/view fields and dataset versions.
The initial frame of Observing is always discarded because its interval belongs to loading.

```sh
cargo build --release -p cosmogon
./target/release/cosmogon --new lab --seed 1 --focus Saturn --distance 5 --speed 0 \
  --profile /tmp/saturn-run1.json --profile-warmup 120 --profile-frames 600 --quit
```

`--profile FILE` defaults to Solar System Lab if no `--new`, `--load` or `--continue` is given.
`--profile-frames` defaults to 600 and is bounded to 1–3600; `--profile-warmup` defaults to 120
and is bounded to 0–36000 (at least one transition frame is discarded). `--quit` exits after
writing the report. An export failure gives a nonzero exit status. Existing report files are
never overwritten; use a new path for each run.

To visually inspect the panel, capture after the measurement window. Capture mode disables
keyboard/audio as it did before; ordinary profiling leaves them active.

```sh
./target/release/cosmogon --new lab --focus Mars --distance 3.2 --speed 0 \
  --panel performance --profile /tmp/mars-run1.json --profile-warmup 120 --profile-frames 600 \
  --capture /tmp/mars-performance.png --capture-after 760 --quit
```

When combining capture and profiling, exit waits for both. A screenshot during the sampling
window can distort frame times; schedule it afterward. A zero-work paused simulation is not an
N-body throughput benchmark. Use `--nbody balanced` with an explicitly selected speed for that.
For close-up terrain use a small distance above the radius and record outstanding bake/terrain
jobs; fixed warmup frames alone do not prove convergence.

For a repeatable three-scene smoke pass, `scripts/profile-native.sh /tmp/cosmogon-run1` records
Saturn with the panel, close-up Mars and a galactic view sequentially, keeping reports/captures/logs.
Use a fresh directory. This is a functional/performance observation set, not a controlled benchmark
comparing those different views against one another.

## Report format

`format = cosmogon-performance`, `schema_version = 1`. Includes app/platform/adapter,
startup/load timing, sample policy, scene/seed/time/dataset/physics versions, camera pose,
resolution/graphics/presentation mode, frame/simulation distributions, lagging frames,
raw bounded samples, asynchronous diagnostic snapshots and explicit limitations.
Unavailable VRAM is JSON `null`; absent counters are omitted. This is a measurement report,
not a replacement for a simulation save or a reproducible experiment definition. Camera focus,
speed and view context describe the export moment; moving/editing during a run invalidates a
fixed-scene comparison. No absolute source/save path or sandbox title is exported.

In-memory records reset on entering a different universe. Export is local, explicit and bounded;
no uploads or notification sounds are added. The report's one-time file write happens after
collecting its samples. In-memory serialization/diagnostics still have overhead; this slice
establishes measurement capability and makes no optimization claim.

## Host observations (2026-10-05)

See [VALIDATION.md](VALIDATION.md) for measured runs and retained evidence. Baseline whole-process
numbers and tool limitations are in [CURRENT_STATE_AUDIT.md](CURRENT_STATE_AUDIT.md).
A controlled cross-platform performance benchmark and full GPU/VRAM traces remain future work.

Automated saved-recording runs (`--research-recording … --quit`) render on the main thread to
avoid an early pipelined-renderer shutdown hang. Reports include `measurement.pipelined_rendering`;
compare performance only between runs with matching renderer scheduling. Ordinary interactive
sessions and other automated workloads retain pipelined rendering.
