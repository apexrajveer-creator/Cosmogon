# End-to-end supply-mission closeout (build 51)

Status: **delivered and verified (2026-10-18)**. One scripted, deterministic
Mars supply mission — surface launch → orbit insertion → deorbit → atmospheric
entry → touchdown → supported hold → atomic habitat delivery — runs as a
single mission-span study whose consolidated `manifest["mission"]` reports
`complete: true`. Evidence: [MISSION_CLOSEOUT_VALIDATION](MISSION_CLOSEOUT_VALIDATION.md).

## What shipped

- `flight-definition --mission-span` admits raised (still bounded) work
  ceilings for one authored end-to-end study — 300 M nominal fleet substeps,
  3 G gravity-source visits, 256 MB recording — and requires an authored
  `--launch FILE`; `--transfer FILE` authors the atomic habitat transfer at an
  exact sample boundary. Admission only: no physics, sampling or determinism
  change; legacy studies serialize byte-exact.
- Consolidated `manifest["mission"]` block (`cosmogon-world-vehicle-mission/1`):
  per-phase evidence with actual retained values (launch, orbit insertion,
  entry, landing, hold, delivery) and the single completion gate
  `liftoff && launch_ok && orbit_ok && entry_ok && touchdown_ok && window_ok
  && delivered_positive`. Orbit evidence is the insertion summary (an
  elliptical state with periapsis above the entry interface, e < 1) from the
  retained osculating elements; the whole-span minimum periapsis is retained
  as informational `minimum_periapsis_above_entry_interface` and deliberately
  never gates completion, because a surface launch and a authored entry both
  cross the interface legitimately. Mission-span recordings also retain
  `orbital.csv`. `flight-inspect` replays every value.
- Native **Recorded flight** pane shows the mission evidence chain (phase rows
  plus completion status) for any recording carrying the block.
- Engine correctness fixes the full-span flight exposed:
  - foot shear resets to unloaded state whenever the normal load vanishes or
    the foot is separated, so free-flight arcs no longer transport contact
    normals and trip the 0.02 rad normal-turn refusal;
  - the hold window uses `Flight::mass_kg()` (cargo-ledger aware) instead of
    the plan-definition mass, so an accepted transfer no longer fakes
    post-transfer ground-relative acceleration;
  - retained `trajectories.csv` reports the current cargo-aware vehicle mass
    and held cargo after a transfer, and operation sampling tolerates equal
    representable craft-clock instants (task-tail ulp fragments) while still
    refusing a backwards clock.

## Verified mission (run-51r, 2026-10-18)

2700 one-second samples from a pinned world, replayed exactly by
`flight-inspect` (rc 0, transfer + habitat receipt confirmed):

| Phase | Evidence |
| --- | --- |
| Launch | liftoff 0.75 s · burnout 301.00390625 s · apogee 98362.95792136295 m · peak 256.4102564102564 m/s² · within limits |
| Orbit | insertion achieved (elliptical state above the 80 km interface) · within limits |
| Entry | interface crossing 1437.757812488079 s · peak flux 118930.29789614028 W/m² · total heat 40886865.39811703 J/m² · within limits |
| Landing | touchdown 2383.469531249964 s · vertical 1.3807864117485245 m/s · horizontal 0.01820571388295438 m/s (limits 10/5) · within limits |
| Hold | current window 237.6640625 s ≥ 5.0 s required · within limits |
| Delivery | transfer at sample 2460: 170.0 kg accepted (20 O₂ / 100 potable / 30 dry food / 20 food water) · 1 habitat receipt · within limits |
| **Complete** | **true** |

Checks: 618 release tests plus one existing ignored; no new clippy warnings in
touched files; inspected native capture of the mission readout.

## How the closeout was reached (authoring history)

- The open-loop drag-brake terminal descent originally lofted and ran dry in
  the engine while landing soft in the point-mass design. The fix was closing
  the loop: fly the replica against engine truth and freeze the commands that
  the engine itself reproduces (`/tmp/op51e2e/design.py` + `follower2.py`).
  Touchdown aligns to ~0.3 s of the designed stop.
- Terrain calibration: engine contact reads a slightly different vertical
  datum than the replica within one profile family (`gap@end = ground0 −
  693.88 m`, bias ≈ 1.54 m); `ground0 = 694.18` (TERRAIN_OFFSET 2030.58)
  puts the engine touch exactly at the design stop.
- A persistent hold needs `contact.shear` (`cosmogon-finite-foot-shear/1`)
  with per-foot `sliding_friction` at 0: plain sliding friction has no
  zero-speed shear support, so the window kept resetting.
- The landing footprint is wherever ballistic physics takes the craft; the
  habitat was re-sited to the actual touchdown site (lat −0.000004°, lon
  96.101386° E) so the transfer is genuinely co-located inside `reach_m`.
  Launch pad and original habitat share lat 0, lon 0.

## Goal

One **scripted, deterministic Mars supply mission** executed as a single
authoring-time scenario with complete retained evidence: surface launch →
orbit insertion → deorbit → atmospheric entry → touchdown → supported-window
accrual → atomic cargo transfer into the surface habitat. One recording, one
manifest, one independent replay (`mission-inspect` semantics via the existing
`flight-inspect` plus new consolidated checks).

## Why one continuous study (not chained legs)

Every flight study creates a new craft through `Flight::new`; there is no
"continue the existing craft" study, so multi-leg chains would describe
several vehicles, not one mission. A single study with one definition keeps
one craft, one propulsion clock and one ledger set for the whole mission.

## The budget problem and its honest fix

Contact-craft studies are charged a nominal `duration × 30000 × sources ≤
20_000_000` work bound, capping simulated time near 66.7 s — enough for
touchdown transents, not for a ~1–2 h mission. The real CPU cost is actual
accepted substeps (`maximum_step_s` 0.005 → 200 Hz even in vacuum), so a
~1-hour mission is ~720 k substeps (minutes of CPU), not a hang. The shipped
fix is the **opt-in `--mission-span` flag** on `flight-definition`, which
raises the bound for that study only (300 M nominal fleet substeps, 3 G
gravity-source visits, 256 MB recording) and requires an authored launch.
Determinism is unaffected: the flag changes only an admission bound, never
physics.

## Orbit evidence for contact studies

Build-49 `orbital.csv` evidence applies to free-flight studies only; contact
studies deliberately omit it. The mission does not need per-sample CSV: the
mission summary derives **insertion evidence** from the retained observations
(periapsis above the authored entry interface, eccentricity < 1, apoapsis
bound) using the parent's μ and the exact retained states, with mismatch
refusals mirroring `vehicle_orbit`. This is a conditional
`manifest["mission"]` block, not a change to the free-flight orbital evidence.

## Mission definition (authored, not guided)

`mission-definition` authors on the existing example bench at a geographic
pad (default: Mars 0°N 0°E or explicit site):

1. cargo ledgers + handling equipment (required for delivery),
2. launch limits (build 48) and entry + landing limits (build 50),
3. an explicit inertial command sequence: vertical rise → pitch/insertion
   burn → coast to apoapsis → deorbit burn → ballistic descent (fixed
   authored directions; no guidance or attitude controller is claimed),
4. one timed authored transfer (build 47) at a sample boundary well after the
   expected touchdown, gated by the real supported operation window,
5. a habitat-bearing origin world (`habitat-setup`) so the transfer has a
   same-body recipient within the modeled great-circle reach.

The landing site is wherever ballistic physics takes the craft; the habitat
site is chosen so the great-circle separation stays inside `reach_m`
(default 1e7 m). Predicting the impact footprint is authoring work (run,
observe, re-site the habitat); the scenario does not steer.

## Evidence chain (all existing ledgers, one manifest)

| Phase | Evidence |
| --- | --- |
| Launch | launch ledger: liftoff/burnout/peak force/propellant (build 48) |
| Orbit | `manifest["mission"]` insertion summary (periapsis > interface, e < 1) |
| Entry | entry ledger: interface crossing, peak flux, fluence, no limit stop (build 50) |
| Landing | landing ledger: touchdown time/speeds within limits (build 50) |
| Hold | operation window ≥ `required_window_s` at the transfer boundary (build 44) |
| Delivery | transfer record + receipt: cargo leaves the craft, habitat stocks rise (build 47) |

`flight-inspect` already replays every ledger and the transfer CSV. The
mission build adds the consolidated `manifest["mission"]` block (phase
checklist with actual values, insertion summary, final habitat receipt
totals) verified against replay, so one command proves the whole chain.

## Explicit non-claims

No guidance, navigation, attitude control, aerodynamic lift, plume/pad
interaction, soil/terrain hazard, landing certification, docking or
interplanetary transfer. A ballistic Mars mission authored on the example
bench is a scenario closeout, not mission feasibility for a real vehicle.
The 200 Hz contact stepping in vacuum is a modeling cost, not a physical
requirement.

## Implementation checklist (all delivered in build 51)

1. ✅ Opt-in mission work bound in `Study::new_mission` / `mission_span`
   (admission only, hard ceilings, CLI `--mission-span` + validation and tests
   for refusal beyond the ceiling).
2. ✅ `manifest["mission"]` consolidation in `world_vehicle_recording` (phase
   evidence, insertion summary from retained states + parent μ, receipt totals)
   with `flight-inspect` verification; `complete` is the conjunct of all gates.
3. ✅ CLI surface: `flight-definition --mission-span` (requires `--launch
   FILE`) and `--transfer FILE` compose the mission from the existing
   `launch-definition`/`entry-definition`/`landing-definition`/`habitat-setup`
   authoring commands rather than one monolithic `mission-definition`.
4. ✅ Tests: admission refusals, operation same-clock task-tail fragments,
   shear free-flight resets, manifest replay equality, full e2e mission run
   (every gate non-null, `complete: true`, habitat receipt equals the
   transfer).
5. ✅ Native: mission-context readout in the Recorded flight pane for studies
   carrying `manifest["mission"]` (inspected capture).
6. ✅ Docs: `MISSION_CLOSEOUT.md`, `MISSION_CLOSEOUT_VALIDATION.md`, gap entry
   51, tracker, changelog.
