# Finite supply-vehicle design

View → **Supply vehicle design**, or the command palette, opens an independent engineering bench. It begins the master directive's spacecraft design phase. It does not launch a craft, calculate a gravitational trajectory, land or deliver supplies. Cabin temperature, greenhouse and study-origin systems continue to use their existing authoritative state. Graphics work remains queued.

## Author and calculate

**Design** separates dry hardware/containers, tank capacity, actually held propellant, payload capacity and four cargo inventories: reserve oxygen, potable water, dry food and food-bound water. All values, including thrust and effective specific impulse, are authored assumptions. The default example is not measured hardware. Capacity reductions never silently discard mass; correct invalid held stocks before calculating.

**Burn plan** specifies duration, throttle and an explicit unit direction for each command. Zero throttle coasts. Axis buttons and custom components use the same input validation; invalid directions are refused rather than normalized. Choose an observation interval; at most 8192 retained observations and 1024 commands are supported. Initial state, actual regular interval endpoints, every command endpoint and an exact fuel-depletion boundary are retained. A depletion coincident with a regular endpoint uses that endpoint's depletion phase; its original command index/definition remains available.

**Calculate this design** pins the exact requested plan in a worker. Later control edits remain staged and do not change the completed observations. **Results** shows completed-plan identity, mass closure, selectable actual observations, scalar capability spent and separate X/Y/Z velocity changes. Opposing burns can cancel vector velocity change while still spending capability and fuel. Recorded lines are display guides between retained points; no continuous measurements or uncertainty bands are invented. Elapsed time is bench time, independent of the live universe clock.

## Physical and numerical scope

With the constant assumed effective nozzle condition, mass flow is `F/(Isp*g0)` and interval impulse uses the actual represented inventory removed. The deterministic ideal velocity increment is `Isp*g0*log1p(removed_propellant/final_mass)`. Dry mass and all cargo remain aboard throughout. Changing mass changes the retained powered acceleration. Fuel depletion splits powered and fuel-limited time; empty tanks grant no impulse, acceleration or velocity change. Coasts and zero thrust have no powered acceleration; unavailable values are not reported as zero measurements.

The exact conventional `g0 = 9.80665 m/s²` converts specific impulse; it is not local planetary gravity. [NASA Glenn's specific impulse](https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/specific-impulse/) and [ideal rocket equation](https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/ideal-rocket-equation/) support these equations; [NIST's SI conversion reference](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b9) supports standard acceleration. No source calibrates the authored hardware values.

Raw interval mass residuals and cumulative propellant closure remain visible. Different observation intervals can accumulate slightly different floating-point residuals; the same typed plan replays exactly. `DMath::dln1p` avoids losing small mass-ratio velocity changes. A requested positive burn below held-inventory representation precision is refused before mutation rather than granting impulse without mass removal or pretending fuel shortage.

The model `cosmogon-supply-vehicle-reference/1` excludes gravity, aerodynamic/nozzle-pressure variation, attitude dynamics, staging, thermal/power limits, boiloff, navigation, launch, landing and cargo transfer. Burn capability alone does not establish mission feasibility or safety. No Universe fields, simulation laws, source constants, save schema or golden fingerprint change in this build.

## Protected files and exact reopening

**Models & files** displays assumptions and primary sources. Save a fresh typed plan file, load a typed plan into controls, calculate into a fresh output folder or verify/reopen an existing complete recording. Failed reads/runs preserve the previous completed inputs, source and observations. Loading a plan only stages controls; reopening a verified recording restores its controls and retains its original stored rows. Unopened path text is separate from retained source identity.

A complete folder contains:

- `plan.json`: full typed design, commands and observation interval.
- `observations.json`: all original actual states, original commands, phases and preceding intervals.
- `trajectories.csv`: 24 numeric/phase columns, with explicit units in names. Command indices are zero-based; initial interval columns are blank because no preceding interval exists. Powered acceleration is blank when no powered duration exists.
- `manifest.json`: completion written last, full plan, source/model context and final mass/result summary.

The bounded shared reader verifies schema, exact source context, manifest/plan/result consistency, independently replays every retained numerical observation and checks CSV byte-for-byte. Incomplete, changed-source, altered-number and altered-CSV bundles are refused. Existing files/folders are protected, including partially written folders. Definitions are limited to 1 MiB; recordings to 32 MiB per trajectory file. Definition-file parent directories must exist; recording exports create their parent directories.

```bash
./target/release/cosmogon-cli vehicle-definition --output /tmp/vehicle-plan.json
./target/release/cosmogon-cli vehicle-study --definition /tmp/vehicle-plan.json --output /tmp/vehicle-study
./target/release/cosmogon-cli vehicle-inspect --recording /tmp/vehicle-study
./dist/Cosmogon.app/Contents/MacOS/cosmogon --new lab --focus Mars --speed 0 \
  --vehicle-recording /tmp/vehicle-study --vehicle-observation 300 --panel vehicle-results \
  --capture /tmp/vehicle-results.png --capture-metadata --capture-after 240 --quiet --quit
```

Use fresh output paths. The default example includes more than 300 observations, including an exact cutoff. `vehicle-study` without output prints the actual result; `vehicle-definition` without output prints the editable plan. Native `--vehicle-plan FILE` calculates that exact plan; `--vehicle-recording FOLDER` reopens original rows; optional `--vehicle-output NEW_DIR` reruns the selected plan into protected files. These two input forms are mutually exclusive. `--vehicle-observation INDEX` must lie inside the actual plan. Native input/output/index checks run before GPU startup and workers recheck disk when opening. Invalid dependent exports are canceled, never redirected to older retained results.

Capture panels: `vehicle`/`vehicle-design`, `vehicle-burns`, `vehicle-results`, `vehicle-model`/`vehicle-files`. Screenshot/performance metadata includes staged controls, request-time plan, actual retained source/manifest and selected row. Ordinary native launches keep pipelined rendering; short vehicle captures disable it to avoid the known teardown trap.

World flight, physically valid delivery, calibrated vehicle hardware, research figure exports, manual keyboard/pointer/accessibility coverage and the full spacecraft/settlement phases remain incomplete. Next, connect this finite mechanism to deterministic world-relative flight rather than adding automatic supply grants.
