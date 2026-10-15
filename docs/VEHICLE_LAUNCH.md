# Authored finite surface launch

A launch-limited flight is an ordinary contact-capable craft that starts on a
modeled surface pad and opts into versioned structural limits plus retained
trajectory evidence (`cosmogon-vehicle-launch/1`). The engine thrust is the
existing authored inertial central thrust through the craft centre; what the
launch model adds is a **structural specific-force bound**, **surface-relative
liftoff/burnout/apogee evidence** on the craft propulsion clock, and an
**atomic structural-failure stop** at the last supported macro state when the
bound is exceeded.

The parent surface is the existing body-fixed tilt/spin height field via the
contact surface sampler. There is no pad, weather, plume, attitude controller,
gimbal, aerodynamic drag/lift, or areoid launch-site hazard model. Local
altitudes are the sampler's radial clearance, not certified geodetic
altitudes. Atmospheric ascent losses beyond gravity and the modeled thrust are
not computed; entry/heating limits are a separate future model.

## Limits

`Definition.launch` (`Option<Limits>`) is opt-in; absent legacy definitions
serialize byte-exactly.

| Field | Meaning | Bounds | Example |
| --- | --- | --- | --- |
| `maximum_acceleration_m_s2` | Sensed engine specific force (throttle × rated thrust / inert mass) bound | `0..=1e4`, finite | 15 |
| `liftoff_altitude_m` | Height above the authored start position that counts as liftoff | `0..=1e5`, ≤ start bound | 1 |
| `maximum_start_altitude_m` | Maximum surface-relative start height above the local terrain | `0..=1e5` | 100 |

Structural rules (enforced in `validate_structure` / `Flight::new`):

* Launch limits require authored contact landing gear (the craft must rest on
  a pad; the free-flight integrator is not a launch path).
* `Flight::new` resolves the start clearance with the contact surface sampler
  and refuses starts above `maximum_start_altitude_m` (orbital starts are
  rejected with the actual clearance in the message).
* The liftoff rule is *height above the authored start plus the liftoff
  bound*, so settling onto the gear never counts as liftoff.

## Evidence

`Flight.launch_ledger` (created with launch-limited definitions) retains:

| Field | Meaning |
| --- | --- |
| `initial_propellant_kg`, `start_altitude_m` | Ledger anchors at creation |
| `liftoff_elapsed_s`, `burnout_elapsed_s` | First crossings on the propulsion clock (burnout requires observed thrust after liftoff) |
| `apogee_altitude_m` | Running post-liftoff maximum of the local-terrain radial clearance |
| `peak_acceleration_m_s2` | Running maximum of the sensed specific force |
| `propellant_consumed_kg` | Initial minus remaining propellant |
| `substeps_observed`, `powered` | Accepted-substep count and thrust-observed flag |

Exceeding the structural bound returns an error from the accepted substep; the
world keeps the last supported macro state and the craft is marked
`Phase::Unsupported` with the actual/limit values in the reason. On reopen,
`validate_saved` requires the ledger to exist exactly when limits exist and
revalidates every field; saves gain a conditional `cosmogon-vehicle-launch`
data-version pin when any flight carries limits or a ledger.

## CLI

```
cosmogon-cli launch-definition --thrust-time SECONDS --thrust-newtons N \
    [--maximum-acceleration M_S2 --liftoff-altitude M --maximum-start-altitude M] \
    [--initial WORLD | --scenario S --seed N] --target Mars \
    --latitude N --longitude E --output NEW_FILE
cosmogon-cli flight-definition --launch FILE [--samples N --interval-seconds S] --output NEW_FILE
cosmogon-cli flight-study --definition PINNED_STUDY --output NEW_DIR
cosmogon-cli flight-inspect --recording COMPLETE_DIR
```

`launch-definition` resolves the pad on the actual local terrain (a probe at
the geographic site, then a fixed clearance above the terrain along the local
radial), authors a single full-throttle burn in the fixed inertial local
zenith at ignition on the example propulsion bench at the given thrust, and
refuses at authoring time when the sensed specific force would exceed the
structural limit (the message gives both numbers). `flight-definition
--launch` consumes the whole definition file; it is mutually exclusive with
`--geographic-start`, `--contact-hardware` and `--definition`.

Study budgets still apply: a contact-craft study in SolarSystemLab allows
roughly 39 one-second samples (`duration × work × sources ≤ 20_000_000`);
long ascents belong in unit/integration tests or smaller-source scenarios.

## Native UI

The world-vehicle pane shows a "Launch limits and trajectory evidence"
readout for launch-limited craft: the structural bound, liftoff/burnout times,
apogee above local terrain, peak acceleration, consumed propellant and the
accepted-substep count, plus the scope reminder (accepted-substep geometry, no
atmospheric losses or attitude controller).

## Scope

This is explicit authored mechanics on the existing bounded solvers. It does
not establish safe launch, mission feasibility, structural load paths,
telemetry-grade trajectory truth, atmospheric ascent performance, pad
interaction or landing. A craft that returns to the surface too fast for its
gear ends in the gear solver's ordinary structural refusal — the launch
evidence recorded before that point remains valid.
