# Orbital evidence and craft rendezvous

Build 49. Free-flight studies now retain derived orbital evidence, and two complete flight
recordings can be compared into an explicit craft-to-craft rendezvous report. Both features are
derived from retained states: no universe state, simulation law, save schema or golden changes.

## Osculating orbital evidence (`cosmogon-orbital-evidence/1`)

`crates/cosmogon_sim/src/vehicle_orbit.rs` computes instantaneous two-body elements from one
retained N-body state:

* `Elements` — semi-major axis, eccentricity, periapsis, apoapsis and period (both `None` for
  hyperbolic states, never zero), inclination to the system inertial plane, specific energy and
  specific angular momentum.
* `osculating(rel_pos, rel_vel, mu)` — `a = −μ/(2E)`, eccentricity vector from `e = (v×h)/μ − r̂`,
  inclination from the `z`-clamped `acos` of `h`.
* `observe(world, flight)` — rebuilds exactly what the craft felt: parent position from the
  retained `GravitySample` the craft recorded (no origin-time reconstruction), parent velocity
  from `system.body_state(parent, flight.last_time_s)`. Refuses on sample/body/`radius_m`/`mu`
  mismatch, so evidence can never silently come from the wrong body.

Every retained observation of a **free-flight** study (no contact hardware) appends one
`OrbitSample { world_time_s, elements }` to the recording:

* `orbital.csv` — one row per observation at full `f64` precision (hyperbolic apoapsis/period
  columns are blank, not zero),
* `manifest["orbital"]` — a conditional summary block (`model`, parent, scope, observation
  count, initial/final elements, minimum periapsis, maximum apoapsis, `closed_at_final`),
* both are verified byte-exact against replay by `flight-inspect` and by every reader.

Contact studies omit the block entirely; a stray `orbital.csv` on a contact recording is
refused. The research registry gains a `vehicle-orbital-evidence` descriptor (sources
`horizons`, `solar-facts`).

## Craft-to-craft rendezvous (`cosmogon-vehicle-rendezvous/1`)

`crates/cosmogon_sim/src/world_vehicle_rendezvous.rs` compares two complete flight recordings:

* Both must share one origin-world fingerprint, one system and one exact observation grid
  (count and times); each side is independently replayed through `Bundle::run`.
* Evidence over the shared times: closest index/time, closest distance, relative speed and
  relative position at that sample, plus the exact scope string.
* `Report::export` writes a protected fresh folder (`definition.json`, `rendezvous.json`,
  `rendezvous.csv`); readers re-verify the report by recomputing from the two recordings.

## Headless commands

```bash
./target/release/cosmogon-cli flight-rendezvous \
    --recording /tmp/flight-a --reference /tmp/flight-b --output /tmp/rendezvous
# without --output the full report manifest is printed to stdout

./target/release/cosmogon-cli flight-inspect --recording /tmp/flight-a   # now includes orbital.csv
```

`--recording` and `--reference` are both required; unknown flags are refused.

## Native view

The World flight pane shows an "Osculating orbit about the parent body" readout for any live
craft without contact hardware (semi-major axis, eccentricity, peri/ apoapsis, period,
inclination, specific energy and angular momentum). It observes the current state once per
frame — instantaneous evidence, never a propagated orbit.

## Scope

* Elements are instantaneous two-body evidence of the retained N-body state. They are not an
  orbit propagator, a navigation plan or a maneuvering guarantee; real N-body perturbations make
  them drift over time (that drift is retained honestly in the CSV).
* Inclination is measured to the system inertial plane, not the parent's equator.
* Rendezvous evidence reports closest approach among exactly shared observation times. It does
  not plan transfers, command craft, prove capture/docking or imply contact.
* No atmosphere, drag, third-body station-keeping or maneuver optimization is modeled here;
  entry/heating limits remain build 50.

See [validation](VEHICLE_ORBIT_VALIDATION.md) for the recorded evidence.
