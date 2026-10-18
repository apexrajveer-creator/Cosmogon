# Validation — end-to-end supply-mission closeout (build 51)

Host: macOS (arm64), release profile. Date: 2026-10-18. Local validation, not
a CI claim. Evidence for [MISSION_CLOSEOUT](MISSION_CLOSEOUT.md).

## Automated checks

* `cargo test --release --workspace` — **618 passed, 1 existing ignored, 0
  failed** (app 130, CLI 27, core 32, physics 38, simulation 390, one physics
  doctest).
* `cargo clippy --workspace --all-targets --release` — completes; **no
  warnings in any touched file** (`world_vehicle_recording.rs`,
  `vehicle_support.rs`, `vehicle_operation.rs`, `vehicle_contact_dynamics.rs`,
  `cosmogon_cli/src/main.rs`, `ui/world_vehicle.rs`). Pre-existing warnings
  elsewhere (core math/physics/renderer/older UI, `block 0.1.6` future-compat)
  are unchanged.
* Fresh release builds of `cosmogon-cli` and `cosmogon` succeed.
* Golden serialized-world fingerprint remains `0xf2c4140a45ae509b`: all
  mission state is authored/tmp-side (pinned study files and a re-sited
  habitat in the protected origin world), no universe schema change.

## End-to-end mission loop (case folder `/tmp/op51e2e`)

Pinned world `world0.cosmo` (Mars + habitat 0 re-sited to the ballistic
landing footprint), authored `mission-{launch,entry,landing}.json`,
`transfer.json` (at_sample 2460) and the 502-command inertial plan:

1. `flight-definition --initial world0.cosmo --launch … --entry … --landing …
   --transfer transfer.json --cargo-ledgers --handling --mission-span --samples
   2700 --interval-seconds 1` → pinned study (refuses without `--launch`).
2. `flight-study` → recording `run-51r` (rc 0, no diagnostic output).
3. `manifest["mission"]` — **`complete: true`**, with every gate green:

   | Gate | Value |
   | --- | --- |
   | launch | liftoff 0.75 s, burnout 301.00390625 s, apogee 98362.95792136295 m, peak 256.4102564102564 m/s², within limits |
   | orbit | `achieved: true`, `within_limits: true` (insertion above the 80 km interface; `orbital.csv` retained) |
   | entry | interface 1437.757812488079 s, peak 118930.29789614028 W/m², total 40886865.39811703 J/m², within limits |
   | landing | touchdown 2383.469531249964 s, 1.3807864117485245 / 0.01820571388295438 m/s (limits 10 / 5), within limits |
   | hold | current window 237.6640625 s ≥ 5.0 s required |
   | delivery | 170.0 kg accepted (20 / 100 / 30 / 20), 1 habitat receipt, 1 transfer |

4. `flight-inspect --recording run-51r` exits 0 and replays the ledgers plus
   the transfer (accepted equals habitat receipt; habitat receipt
   `world_time_s 820500060.0` = sample 2460 boundary).
5. Determinism: a post-DIAG-removal rerun produced a byte-identical
   `mission` block and the same origin fingerprint.

## Native evidence

Captured with
`cosmogon --flight-recording /tmp/op51e2e/run-51r --panel vehicle-flight-results --capture … --capture-after 240 --quit`
and inspected: the Recorded flight pane renders the "Mission evidence chain
(whole span)" section — seven green phase rows with the values above and
"Mission complete: every authored phase gate is satisfied." Non-mission
recordings show no section (the block is conditional).

## Defects found and fixed during the build (each with a failing run first)

1. **Shear transported across free flight** — after an unsupported gap the
   carried foot-shear normal rotated past the 0.02 rad guard and refused the
   flight (run-51k, "Foot shear normal turns more than the supported 0.02 rad
   increment"). `sample()` now resets shear to unloaded when
   `normal_load_n <= 0`, and `evolve_shear` resets when the foot is separated;
   the stuck-friction and 8 shear tests pass.
2. **Hold used stale mass after transfer** — `vehicle_support::hold()` read
   `propulsion.mass_kg(plan definition)` (2314.1 kg, initial cargo included)
   while the dynamics used the cargo-ledger mass (2144.1 kg), fabricating
   0.27 m/s² of "ground-relative acceleration" forever (run-51o: every phase
   green, hold never opened). It now uses `Flight::mass_kg()`; run-51p opened
   a 237 s continuous window through the transfer.
3. **Orbit gate made `complete` unreachable** — the completion conjunct
   required the *minimum* periapsis over the whole span to stay above the
   entry interface, which a surface launch and a deliberate entry can never
   satisfy. Completion now uses the documented insertion evidence
   (`orbit_achieved`); the whole-span statistic remains as informational
   `minimum_periapsis_above_entry_interface`.
4. **Operation sampling refused equal clocks** — task-tail substep fragments
   can collapse to the same representable craft instant at real-world epochs;
   equal-clock samples are now skipped (backwards clocks still refuse), with
   regression tests including a non-dyadic-command flight that must keep
   flying.
5. **Recorder showed pre-transfer mass/cargo** — `trajectories.csv` now
   writes `Flight::mass_kg()` and the held cargo ledger, so post-transfer rows
   read 2144.10 kg and 0 kg held O₂.

## Authoring-side evidence (not engine claims)

* Touchdown aligns with the closed-loop design stop (~0.3 s); landing speeds
  are within the authored 10 / 5 m/s limits at first contact.
* Terrain datum bias within one profile family: `gap@end = ground0 − 693.88 m`
  (≈ 1.54 m engine-vs-replica), calibrated to `ground0 = 694.18`
  (`TERRAIN_OFFSET = 2030.58`).
* Hold requires `contact.shear` with per-foot `sliding_friction = 0`;
  plain sliding friction has no zero-speed shear support.
* The habitat is co-located by authoring (re-sited to lat −0.000004°, lon
  96.101386° E after observing the ballistic footprint); launch pad and the
  original habitat share lat 0, lon 0.

## Explicit limits

The mission is an authored ballistic scenario closeout on the example bench:
no guidance, navigation, attitude control, aerodynamic lift, plume/pad
interaction, soil hazard, landing certification, docking or mission
feasibility for a real vehicle. `complete` means every authored evidence gate
in one retained recording is satisfied and replayable — nothing more.
