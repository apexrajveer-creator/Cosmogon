# Validation — orbital evidence and craft rendezvous

Build 49 end-to-end evidence, run against the fresh release CLI (`./target/release/cosmogon-cli`)
and the release app on macOS arm64, 2026-10-16.

## Automated checks

* `cargo test --release --workspace` — **596 passed, 0 failed, 1 intentionally ignored**
  (130 app + 26 CLI + 32 core + 38 physics + 369 sim + 1; +12 over 584: `vehicle_orbit` 6,
  rendezvous 3, recording 2, CLI 1).
* `cargo clippy --release --workspace --all-targets` — no warnings in any touched file
  (`vehicle_orbit.rs`, `world_vehicle_rendezvous.rs`, `world_vehicle_recording.rs`,
  `world_vehicle.rs` UI, CLI `main.rs`, registry). Pre-existing warnings elsewhere untouched.
* Golden simulation fingerprint remains `0xf2c4140a45ae509b` — build 49 adds no saved-universe
  fields (same precedent as approach recording: derived evidence carries no save pin).
* Fresh `cargo build --release -p cosmogon -p cosmogon_cli`.

## End-to-end loop (13 cases)

Setup: a coast propulsion plan (single 86400 s zero-throttle command; observation interval 30 s
to respect the 8192-actual-observation plan budget) and a geographic start file
(400 km equatorial altitude, `east_velocity_m_s 3093.219816855398`).

1. **Circular study A** — `flight-definition --samples 1000 --interval-seconds 60` pins
   `fnv1a64:a5877f477368033f` (lab/seed 1 default), `flight-study` records 1000 observations
   (59 940 s of coast). Manifest `orbital`: initial `a = 3589500.000 m`, `e = 3.12e-13`,
   `T = 6529.26495041638 s`, `i = 0`; final `a = 3589523.661 m`, `e = 0.0000115`
   (honest N-body drift over 16.6 h); `minimum_periapsis_m 3588539.823`,
   `maximum_apoapsis_m 3589600.810`, `closed_at_final true`, 1000 observations.
   `orbital.csv` = 1001 lines; row 0 `a = 3589500.000000001`, `i = 0`.
2. **Geographic study B** — craft authored via `--geographic-start` in Mars's equatorial plane at
   400 km. The surface-spin term (`268.608701673340875 m/s` = authored-speed residual after
   `SurfaceFrame::to_inertial` adds `ω×r`) was subtracted once, giving relative speed
   `3361.8285185287414 m/s` against the exact circular `3361.8285185237064 m/s` (agreement
   `5e-9 m/s`, tangency dot `0.008`) and initial `a = 3789500.000 m`, `e ≈ 0`,
   `T = 7082.494 s`, **`i = 25.188600°` = Mars's axial tilt** (equatorial orbit seen in system
   inertial coordinates — an independent physical cross-check), final `e = 0.000216326`,
   `closed_at_final true`.
3. **Rendezvous A×B** — `flight-rendezvous --recording A --reference B --output out49`:
   1000 common samples, closest at sample 381 / `world_time_s 820520460.0`,
   distance `342909.446772 m`, relative speed `1509.798785 m/s`, relative position
   `[-191360.11, 262473.166, 109891.01]`. Distance genuinely varies across the shared grid
   (first `5403694.900 m`, min `342909.447 m`, max `7258470.863 m`, last `7150547.161 m`).
   Report files: `definition.json`, `rendezvous.json`, `rendezvous.csv`.
4. **Manifest print** — same invocation without `--output` prints the report manifest
   (`complete true`, same evidence).
5. **Replay inspect** — `flight-inspect` on study A: `complete true`,
   `orbital model cosmogon-orbital-evidence/1`, 1000 observations, all CSV bytes match replay.
6. **Negative: grid mismatch** — A (1000×60 s) vs a 999-sample sibling from the same origin →
   `Rendezvous requires both recordings to share one exact observation grid`, exit 1.
7. **Negative: origin mismatch** — `habitat-setup` world + `flight-definition --initial` +
   `flight-study --initial` produces a recording with a different origin fingerprint →
   `Rendezvous requires both flight recordings to share the same exact origin world`, exit 1.
8. **Negative: non-recording path** — `--recording /tmp/op49e2e` → missing-manifest error,
   exit 1.
9. **Negative: unknown flag** — `--samples 5` → `--samples is not a flight-rendezvous option`,
   exit 1.
10. **Negative: tampered CSV** — copy of study A with row 1 `semi_major_axis_m` edited →
    `Stored orbital CSV differs from exact retained osculating evidence`, exit 1.
11. **Negative: stray CSV on contact recording** — build-48 contact recording + added
    `orbital.csv` → `Contact flight recording must not carry an orbital CSV`, exit 1.
12. **Regression: old contact recording** — the pre-build-49 launch recording still reads:
    `complete true`, **no `orbital` key**, files end in `contact.csv`. Legacy free-flight
    recordings likewise read (missing `orbital.csv` = absent evidence, not an error).
13. **Native capture inspected** — `./target/release/cosmogon --load flightA/final.cosmo
    --panel vehicle-flight --capture ui49.png --capture-after 240 --quit`: the World flight
    pane shows the "Osculating orbit about the parent body" block with live values
    (`a ≈ 3588996.204 m`, `e = 0.000111`, peri `3588599.006 m`, apo `3589393.402 m`,
    `T = 6527.890 s`, `i = 0.000°`, energy `−5966642.685011 J/kg`,
    `h = 123980393325.587002 m²/s` after ~42.6 h of live coast at 1 day/s — consistent with the
    recorded CSV drift). The first capture attempt exposed the readout nested inside the
    `flight.cargo.is_some()` gate (free-flight craft have no cargo entry); the block was moved
    to flight level and re-captured.

## Limits

* Elements are instantaneous two-body evidence of retained N-body states, not propagation,
  guidance or a maneuver plan; drift between samples is real physics and is retained.
* Inclination is to the system inertial plane, not the parent equator.
* Rendezvous measures closest approach on exactly shared observation times only; it plans
  nothing, commands nothing and proves no capture, docking or contact.
* No atmosphere, drag, lift, entry or heating model; those remain build 50.
* Local macOS arm64 run; no cross-platform CI claim.
