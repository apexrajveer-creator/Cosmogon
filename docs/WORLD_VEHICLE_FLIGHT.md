# Physical finite vehicle flight

Verified local development checkpoint: **467 tests pass plus one existing ignored**, 63 CLI/native cases, nine inspected packaged layouts and a 48-second normal run. [Evidence and limits](WORLD_VEHICLE_FLIGHT_VALIDATION.md).

The same finite supply-vehicle plan now creates an optional authoritative spacecraft in the world. The ordinary create/remove edits, undo/redo, fixed task, saves and source pins retain its actual propellant, authored cargo, system-local position/velocity and modeled phase. Creating a craft supplies explicit initial conditions; it does not debit or grant habitat stocks. The previous propulsion design bench and its exact recordings remain separate.

Open **View → Supply vehicles · design & flight**, or search for that workspace in the command palette. **World flight** takes explicit relative position and velocity in metres and metres/second, adds the selected body's actual inertial translation and velocity, and creates the craft through the ordinary sandbox edit. The circular example starts 200 km above a spherical boundary with an assumed tangential circular speed. It is an initial-condition example, not a simulated surface launch or a validated orbit/mission. Copy design/burn controls explicitly into the staged flight before creating it. Reference worlds require cloning before live creation/removal.

The normal world clock drives finite burns and gravity. The craft resolves a one-second task, with at most 0.25-second gravity/powered-drift substeps split at command ends and represented fuel exhaustion. A user can observe between tasks: the world clock and last resolved craft clock are separate. Time warp resolves actual flight and can lag under the existing CPU budget; it never fast-forwards unresolved spacecraft. Cargo remains aboard when fuel runs out. After commands end, a supported craft coasts under gravity.

First modeled spherical contact stops motion and retains the actual partial time, position, velocity, consumed propellant, source position/radius, raw boundary residual and tolerance. It is **not a safe landing**, terrain contact or a delivery. Unsupported motion retains the last supported state and a reason. Abrupt gravity/frame edits and changed source membership/mass/radius currently stop unresolved flight: event-time source-discontinuity integration is not implemented. Historical terminal states do not keep moving or consume fuel.

## Recorded branches and exact origins

**Recorded flight** runs the staged definition from either the actual request-time live world or the exact original world retained in a completed recording. Both create a new craft in an independent branch. They do not rewind, replace or advance the live scene. A recorded origin loaded from disk is rechecked before rerunning; later draft edits do not replace a worker's frozen requested inputs or completed observations. A failed read/run cancels a queued export and preserves previous results.

**Flight models & files** reopens complete recordings, exports completed evidence to a fresh folder, or saves a setup folder containing both `definition.json` and its exact `origin.cosmo`. The setup retains the world at the instant the staged definition is written; users need not race the advancing clock to save a matching origin. Existing files/folders are protected.

A completed recording contains:

- `definition.json`: full flight/propulsion plan, sampling and exact starting-world pin.
- `origin.cosmo`: complete world before the ordinary create edit.
- `observations.json`: actual sampled world/craft clocks, finite craft state and retained gravity-source samples.
- `trajectories.csv`: 36 numerical columns, with unavailable initial/contact fields blank.
- `final.cosmo`: complete resulting world, including the studied craft and ordinary journal.
- `manifest.json`: completion marker, both state identities, source/model/frame context and limitations.

Reopening independently recomputes the actual world branch and compares every JSON observation, complete final world, manifest and CSV byte. It returns original stored observations after verification. State fingerprints use canonical FNV-1a identity and are not cryptographic integrity. The exported save headers use `saved_at_unix=0` for stable output bytes; ordinary user saves retain their real wall-clock metadata. All scientific times and state/source pins remain present.

Studies support 2–4096 observations, intervals from 0.001 to 86400 seconds, at most one day total two million nominal fleet substeps and twenty million nominal gravity-source visits. Observation payload size is estimated before allocation; large plans/source inventories may require fewer samples. Existing active craft also count toward the work bound. Input/output files are bounded to 32 MiB, with smaller definition/manifest limits. A study's original state must match its exact pin; changing its source world is a new experiment.

## Shared headless commands

```bash
# Explicit assumed circular start and default finite propulsion plan.
./target/release/cosmogon-cli flight-definition \
  --target Mars --samples 361 --interval-seconds 1 --output /tmp/flight.json
./target/release/cosmogon-cli flight-study \
  --definition /tmp/flight.json --output /tmp/flight-recording
./target/release/cosmogon-cli flight-inspect --recording /tmp/flight-recording

# Use an existing exact source world. --definition on flight-definition imports a
# propulsion plan; --definition on flight-study reads the complete pinned study.
./target/release/cosmogon-cli flight-definition \
  --initial /tmp/world.cosmo --definition /tmp/propulsion-plan.json \
  --target Mars --samples 121 --interval-seconds 1 --output /tmp/pinned-flight.json
./target/release/cosmogon-cli flight-study \
  --initial /tmp/world.cosmo --definition /tmp/pinned-flight.json \
  --output /tmp/pinned-flight-recording
```

Without `--initial`, these two author/run commands use the same default Solar System lab and seed 1. A custom world/seed requires its exact saved source for subsequent study execution. Inspection requires a complete folder. Unsupported/ignored option combinations are refused.

Free-flight recordings (no contact hardware) also carry derived orbital evidence: `orbital.csv` plus a conditional `manifest["orbital"]` summary block with per-observation osculating elements, verified byte-exact by `flight-inspect` (see [VEHICLE_ORBIT](VEHICLE_ORBIT.md)). Two complete recordings can be compared into a craft-to-craft closest-approach report:

```bash
./target/release/cosmogon-cli flight-rendezvous \
  --recording /tmp/flight-recording --reference /tmp/other-recording \
  --output /tmp/rendezvous-report
# Both recordings must share one origin fingerprint, one system and one exact
# observation grid; without --output the report manifest prints to stdout.
```

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --new lab --focus Mars --speed 0 \
  --flight-recording /tmp/flight-recording --flight-observation 360 \
  --panel vehicle-flight-results --capture /tmp/flight.png \
  --capture-metadata --capture-after 240 --quiet --quit

# View the actual live fleet in the saved final world.
./dist/Cosmogon.app/Contents/MacOS/cosmogon \
  --load /tmp/flight-recording/final.cosmo --focus Mars --speed 0 \
  --panel vehicle-flight
```

Native `--flight-definition STUDY` requires `--flight-origin WORLD`. `--flight-recording FOLDER` reopens a complete result. `--flight-output NEW_FOLDER` exports only validated completed results; `--flight-observation INDEX` uses actual bounds. Native inputs are read/validated before GPU startup, then workers recheck request-time disk evidence. `--panel vehicle-flight|vehicle-flight-results|vehicle-flight-files` selects the live, recorded or files/model pane. Existing `--vehicle-*` flags still control only the independent propulsion bench.

## Physical scope

The frame is `system-local-inertial-simulation-axes`, using f64 metres and metres/second and the world's existing seconds-since-J2000 coordinate. Galactic origin is excluded from physical state. Companion gravity uses the separate local position helper rather than subtracting a large absolute origin. Source order is primary star, companion and present bodies. The model uses the existing gravitational constants and analytic/N-body sampled source motion, with a test-particle initial parent-mass ratio at most 1e-6.

The operator integrates variable-mass displacement from the actual represented fuel removed; it does not use final speed times the whole burn. Applied ideal thrust capability and actual system-frame speed are different quantities. Contact source centres interpolate linearly across a substep; boundary residuals/tolerances remain visible. The supported regime is at most 1e15 m system-local position and 1e6 m/s speed. Per-row last-substep values are not integrated transfers over the whole observation span.

[Research and design](WORLD_VEHICLE_FLIGHT_DESIGN.md) records primary NASA Glenn/JPL references. Their role is model/frame guidance. There is no new JPL/SPICE precision, measured vehicle performance, uncertainty or field-calibration claim. Relativistic effects, atmosphere/entry heating, attitude/staging, power/thermal hardware, terrain-resolved landing, docking, cargo loading/delivery and spacecraft scene graphics remain incomplete. The graphics supplement remains queued separately.

Enabled craft state and fingerprints deliberately change with model pins. Absent legacy worlds omit the fleet/counter fields, keep their original state and retain the golden fingerprint `0xf2c4140a45ae509b`. The habitat task retains a finite `NEVER` reservation while a later vehicle task exists, so create/remove cannot shift fixed scheduler indices.
