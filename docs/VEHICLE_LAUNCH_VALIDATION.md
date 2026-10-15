# Authored finite surface launch — verification

Build-48 evidence (2026-10-16), companion to [VEHICLE_LAUNCH.md](VEHICLE_LAUNCH.md).

## Automated verification

* `cargo test --release --workspace`: **584 release tests plus one existing
  ignored pass** (130 app + 25 CLI + 32 core + 38 physics + 358 sim + 1
  doctest; +9 over 575).
* `cargo clippy --workspace --release --all-targets`: no warnings in touched
  files (the four new warnings raised during development — a `let`-block
  return in the CLI dispatch, an unused pad probe and a manual assign/clone in
  the launch fixtures — were fixed; remaining workspace warnings are the
  pre-existing nbody/save/complex-type set).
* Fresh `cargo build --release` of `cosmogon_cli` and `cosmogon`.

New automated cases (all in `vehicle_launch::tests` unless noted):

1. `launch_limits_validate_their_bounds` — negative/NaN/out-of-range limits
   and liftoff-above-start inconsistencies are refused.
2. `launch_ledger_evidence_validates` — burnout-before-liftoff, burnout
   without liftoff, negative propellant/peak/apogee are refused.
3. `launch_definitions_require_gear_and_a_surface_start` — gear-less launch
   definitions fail structure validation; a ×1.05 orbital start fails
   `Flight::new` with the surface-start bound.
4. `surface_launch_liftoffs_and_records_trajectory_evidence` — a 200 s full-
   throttle Mars hop: liftoff < 120 s, burnout in [liftoff, 260] s, apogee >
   5 km above local terrain, peak specific force in (4.9, 10.1] m/s²
   (consistent with the example bench mass decay), > 600 kg consumed, phase
   remains Flying at the observation horizon.
5. `launch_stops_atomically_when_the_structural_limit_is_exceeded` — a 4.5
   m/s² bound against a 5 m/s² ignition specific force stops the craft in the
   first seconds with `Phase::Unsupported` containing "structural limit"; no
   liftoff is recorded for the pad-level stop.
6. `launch_ledgers_replay_exactly_after_reopen` — save/reopen with a
   structurally invalid tampered ledger (negative peak) refuses; a clean
   reopen steps in exact lockstep with the original.
7. `save::tests::launch_pin_is_conditional_and_enforced` — the
   `cosmogon-vehicle-launch` data-version pin is absent for plain worlds,
   present with a launch-limited craft, and stripping or rewriting it refuses
   reopen.
8. CLI `launch_arguments_are_explicit_and_refuse_ignored_flags` — missing
   thrust/time/output, negative thrust, ignored flags (`--samples`,
   `--habitat-id`-style), initial+seed conflicts, and
   `flight-definition --launch` combined with geographic/contact/plan options
   are all refused; valid combinations parse with the Mars defaults.

## End-to-end CLI loop (`/tmp/op48e2e/`)

Positive path (Mars pad at 0°N 0°E, seed 1, 15 kN / 10 s burn, limit 15
m/s², 39 one-second samples):

1. `launch-definition` writes `launch.json` and reports the fixed-inertial-
   zenith burn direction and limits.
2. `flight-definition --launch launch.json` pins `study48.json`
   (fingerprint `fnv1a64:a5877f477368033f`).
3. `flight-study` completes with the conditional
   `cosmogon-vehicle-launch: cosmogon-vehicle-launch/1` data version and the
   launch model/limits context in the manifest. Final flight ledger:

   | Evidence | Value |
   | --- | --- |
   | Liftoff | 0.79296875 s |
   | Burnout | 10.00390625 s |
   | Apogee (local terrain) | 309.48951319931075 m |
   | Peak specific force | 12.820512820512821 m/s² (= 15000 / 1170 inert kg) |
   | Propellant consumed | 50.98581064899918 kg |
   | Start clearance | 1.600013529881835 m |
   | Substeps observed | 7936 |

   The study ends with the craft's gear refusing the ~48 m/s touchdown
   ("Foot resultant force exceeds authored structural limit") — an honest
   gear-model stop after complete launch evidence; the launch structural
   bound itself was never exceeded.
4. `flight-inspect --recording out48` replays the study exactly
   (`complete: true`, identical liftoff).

Negative cases:

| Case | Result |
| --- | --- |
| `launch-definition --thrust-newtons 25000 --maximum-acceleration 15` | Authoring refusal: "Authored thrust 25000 N on inert mass 1170 kg senses 21.368 m/s², above the structural limit 15 m/s²; lower --thrust-newtons or raise --maximum-acceleration" |
| Hand-edited definition with `contact: null` | "Launch limits require authored contact landing gear" |
| Hand-edited definition with position ×1.05 | "Launch definitions require a surface start at or below 100 m above the local terrain (current 169409.78002674738 m)" |
| Hand-edited thrust above the bound at runtime (pre-fix run) | "Launch acceleration exceeded the authored structural limit (21.367521367521366 > 15 m/s²)" with the ledger retained at the last supported state |

## Limits of this evidence

Accepted-substep geometry on the propulsion clock is not telemetry or
navigation truth. The structural bound is an engine specific-force bound, not
measured load paths. Local-terrain altitudes are the contact sampler's radial
clearance. No atmospheric ascent losses, attitude control, plume/pad
interaction, weather, or areoid hazards are modeled; entry/heating limits are
a separate future model. The study budget keeps CLI launch studies to ~39
one-second samples in SolarSystemLab; longer ascents are covered by the
integration tests above, which step the world directly.
