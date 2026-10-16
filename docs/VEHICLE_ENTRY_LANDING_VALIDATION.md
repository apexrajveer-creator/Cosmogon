# Entry heating and terminal landing limits — verification

Build-50 evidence (2026-10-16), companion to [VEHICLE_ENTRY_LANDING.md](VEHICLE_ENTRY_LANDING.md).

## Automated verification

* `cargo test --release --workspace`: **613 release tests plus one existing
  ignored pass** (130 app + 27 CLI + 32 core + 38 physics + 386 sim + 1
  doctest; +17 over 596).
* `cargo clippy --release -p cosmogon -p cosmogon_sim -p cosmogon_cli`: no
  warnings in touched files (remaining workspace warnings are the pre-existing
  nbody/save/complex-type set).
* Fresh `cargo build --release` of `cosmogon` and `cosmogon_cli`.

New automated cases:

`vehicle_entry::tests` (7):

1. `entry_limits_validate_their_bounds` — negative/NaN/out-of-range limits are
   refused.
2. `entry_ledger_evidence_validates` — non-finite/negative peak, total or
   substep evidence is refused.
3. `entry_requires_gear_and_an_atmosphere` — gear-less entry definitions fail
   structure validation; a Moon start fails with "Entry heating requires a
   starting body with a modeled atmosphere".
4. `entry_craft_records_interface_crossing_and_heating_evidence` — a descent
   start records an interface crossing and growing peak/total heat with
   substeps.
5. `entry_stops_atomically_when_the_thermal_limit_is_exceeded` — a 1 W/m²
   bound stops the craft in the first seconds with `Phase::Unsupported`
   containing "thermal limit".
6. `entry_ledgers_replay_exactly_after_reopen` — save/reopen with a tampered
   ledger refuses; a clean reopen steps in exact lockstep.
7. `entry_context_declares_its_models_and_limits` — study context carries the
   entry model, source and limits.

`vehicle_landing::tests` (8): `landing_limits_validate_their_bounds`,
`landing_ledger_evidence_validates`, `landing_requires_gear`,
`landing_stops_atomically_when_the_touchdown_limit_is_exceeded`,
`landing_ledgers_replay_exactly_after_reopen`,
`landing_context_declares_its_models_and_limits` plus the two
`landing_world` construction cases (descent start airborne from creation;
pad start never airborne until it rises the liftoff bound).

`save::tests`: `entry_pin_is_conditional_and_enforced`,
`landing_pin_is_conditional_and_enforced` — each conditional pin is absent for
plain worlds, present with limits/ledgers, and stripping or rewriting it
refuses reopen.

CLI: `entry_and_landing_arguments_are_explicit_and_refuse_ignored_flags` —
missing targets/outputs, out-of-range limits, ignored flags and
`flight-definition --entry` combined with geographic/plan options are refused;
valid combinations parse with the Mars defaults.

## End-to-end CLI loop (`/tmp/op50e2e/`)

Hand-authored fixtures from the default Solar System lab (seed 1, fingerprint
`fnv1a64:a5877f477368033f`): `geo.json` (Mars geographic start 0°N 0°E,
spherical altitude 150 km, −3000 m/s up velocity), `coast.json` (5000 s
throttle-0 plan, empty cargo), `gear.json` (four-foot contact hardware),
`entry.json` / `entry-tight.json` (interface 110 km, k = 1.7415e-4, flux bound
1e7 / 1 W/m²), `landing.json` / `landing-tight.json` (10/5 m/s with liftoff
0.05 m; tight 0.3 m/s vertical), each wrapped into a 31-sample 1 s study.

Positive cases:

1. `flight-study --definition entry-study.json --output entry-out` exits 0.
   Final entry ledger: interface crossing **13.234375 s**, peak flux
   **44117.097866207376 W/m²**, total heat **302775.0217576408 J/m²**. The
   craft later stops on the gear solver's ordinary structural refusal
   ("Foot normal load exceeds authored structural limit") at the ~55 km
   impact — an honest gear stop after complete heating evidence.
2. `flight-inspect --recording entry-out` replays exactly (`complete: true`).
3. `flight-study --definition landing-study.json --output landing-out`
   (liftoff bound 10 m against a 30 s pad coast): touchdown evidence stays
   `null` — pad settling never counts as a landing (design-intent artifact).
4. `flight-study --definition landing2-study.json --output landing2-out`
   exits 0. Final touchdown evidence: elapsed **0.22021484375 s**, vertical
   **0.8176573684913913 m/s**, horizontal **0.016955761270384735 m/s**.
5. `flight-study --definition entry-tight-study.json --output entry-tight-out`
   stops with `Phase::Unsupported`: "Entry heat flux exceeded the authored
   thermal limit (616.8209549357098 > 1 W/m²)" — flux accrues from the first
   accepted substep; the interface only timestamps the crossing.
6. `flight-study --definition landing-tight-study.json --output
   landing-tight-out` stops with `Phase::Unsupported`: "Touchdown vertical
   speed exceeded the authored landing limit (0.8176573684913913 > 0.3 m/s)".

Negative cases:

| Case | Result |
| --- | --- |
| `entry-definition --target Moon …` | "Entry heating requires a starting body with a modeled atmosphere" |
| `flight-definition --entry entry.json` without `--contact-hardware`/`--launch` | "Entry heating and terminal landing require contact gear (--contact-hardware or --launch FILE)" |
| `entry-definition --bogus 1 …` | "--bogus is not an entry-definition option" |
| `flight-study --definition … --entry entry.json` | "--entry is not a world flight study option" |

Legacy regressions: `flight-inspect` on the build-48 launch recording
(`/tmp/op48e2e/out48`) and all four build-49 free-flight recordings
(`/tmp/op49e2e/flightA`–`flightD`) replay `complete: true` unchanged.

## Native captures (inspected)

Captured with `--load <recording>/final.cosmo --panel vehicle-flight
--capture … --capture-after 240 --quit` (live) and `--flight-recording DIR
--flight-observation 30 --panel vehicle-flight-results` (recorded):

* `ui50-entry-live.png` — entry limits/evidence block above the flight
  details: interface 110000 m, nose radius 1 m, coefficient 1.7415e-4, bounds
  1e7 W/m² / 1e10 J/m², running peak 6.7410e5 W/m², total 4.8389e6 J/m²,
  12544 substeps, "Interface crossing at 13.234 s" and the Sutton-Graves scope
  line; the live world shows the honest gear stop after the study end.
* `ui50-landing-live.png` — conditional header "Contact does not establish
  safe landing on its own; authored touchdown limits add evidence only…", the
  landing block (bounds 10/5 m/s, liftoff +0.050 m, start clearance 1.600 m,
  airborne true, 7926759 substeps, touchdown 0.220 s / 0.818 m/s vertical /
  0.017 m/s horizontal) and the conditional finite-gear line (4/4 loaded
  feet).
* `ui50-entry-results.png` — recorded pane at observation 30: peak
  4.4117e4 W/m², total 3.0278e5 J/m², 7680 substeps, interface crossing
  13.234 s.
* `ui50-landing-files.png` — files pane renders; its conditional entry/landing
  disclaimer sits below the fixed scroll fold in headless capture (same
  conditionals are verified in the header and gear-line captures above).

## Limits of this evidence

Sutton-Graves stagnation convection on a bulk exponential atmosphere is an
authored assumption, not a calibrated entry corridor, radiation, ablation or
aerodynamic-force model; the classic air coefficient does not transfer to
other gas mixtures. Touchdown evidence is first-contact ground-relative speed
through the surface spin frame — not a certified landing, attachment, terrain
hazard or cargo delivery. Interface altitudes are evidence timestamps, not
physical boundaries. The study budget keeps CLI contact studies to ~39
one-second samples in SolarSystemLab; longer descents are covered by the
integration tests above, which step the world directly.
