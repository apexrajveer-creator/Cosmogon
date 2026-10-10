# Rotating vehicle approach and geographic starts

Verified local development checkpoint: **479 tests pass plus one existing ignored**, 50 CLI/native cases, six inspected packaged layouts and a 48-second ordinary run. [Evidence and limits](VEHICLE_ARRIVAL_VALIDATION.md).

Open Supply vehicles → **Approach**. Active craft observations show ground-relative up/horizontal speed, planetocentric coordinates, actual finite mass and radial/horizontal kinetic energy. Translation and surface rotation are included. The world observation clock, resolved craft clock and original source-frame observation clock are distinct. A high system-frame speed alone does not measure touchdown severity.

**Geographic initial conditions** specify north latitude, east longitude, altitude above the spherical flight radius and east/north/up velocity. Creating a geographic craft resolves those controls at the actual current world clock through the ordinary undoable create edit. This is an authored initial condition, not a surface launch or stocked loading operation. Staging controls instead freezes the converted inertial relative vectors at that clock: pause/reuse the same world for the same longitude, or explicitly recompute them before later creation. Reference worlds still require cloning.

The inverse transform uses the existing tilt/spin convention shared with terrain. Velocity subtracts the derivative of body translation and the rotational `omega × relative_position` term. Fixed-spin rates use the stored period, including retrograde signs; parent-facing tidally locked rotation differentiates the existing projected orbital azimuth. Dynamic on-rails position derivatives use the anomaly parameter that actually propagates the rail. The old N-body initialization velocity, original flight definition convention, physics and recording version remain unchanged.

## Complete recorded evidence

A completed original flight can be independently replayed into an approach report. Every flight sample, original context and complete endpoint must reproduce before deriving observations. Terminal contact retains the first observed source-frame reading; later rows keep that original reading and mark it historical instead of moving the craft or borrowing a later planet orientation. Unsupported motion and contact with a different selected source retain an unavailable reason. Live terminal states lack retained rotation evidence and require the original branch replay.

The report contains `definition.json`, `approach.json`, a 19-column `approach.csv`, a completion/context `manifest.json`, and all six original flight files inside `flight/`. Reopening replays the complete underlying flight and approach, comparing every reading, context and CSV byte. Exports protect existing folders and bound file sizes. The app rechecks the opened original flight on disk before deriving evidence, freezes the request and retains later geographic-control edits separately.

```bash
./target/release/cosmogon-cli arrival-start --output /tmp/descent-start.json
# Edit the explicit north/east/up velocity and spherical altitude in this file.
./target/release/cosmogon-cli flight-definition --target Moon \
  --geographic-start /tmp/descent-start.json --samples 121 \
  --interval-seconds 1 --output /tmp/descent-study.json
./target/release/cosmogon-cli flight-study \
  --definition /tmp/descent-study.json --output /tmp/descent-flight
./target/release/cosmogon-cli flight-approach \
  --recording /tmp/descent-flight --output /tmp/descent-approach
./target/release/cosmogon-cli approach-inspect --recording /tmp/descent-approach
```

The default start is an explicit 10 km spherical altitude and -10 m/s up velocity, with zero east/north velocity. It retains the currently selected propulsion plan; default thrust commands are not descent guidance. Use a zero-throttle authored plan for a coasting study, or explicit finite commands for powered evolution. `flight-approach --target BODY` can analyze another solid body in the same system, without pretending contact with a different source occurred there.

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --new lab --focus Moon --speed 0 \
  --flight-recording /tmp/descent-flight --flight-observation 10 \
  --arrival-output /tmp/native-descent-approach --panel vehicle-arrival \
  --capture /tmp/descent-approach.png --capture-metadata \
  --capture-after 240 --quiet --quit
```

`--arrival-output` requires an explicit pinned flight definition/origin or complete flight recording. It writes approach evidence to a fresh folder; `--flight-output` still writes the unchanged original flight format. These outputs need separate folders. Invalid source/frame/output inputs are refused before GPU startup; the worker rechecks disk inputs before writing.

## Support and next physical mechanisms

This is the application's modeled planetocentric frame, not an imported IAU/SPICE orientation kernel. The original gravity-source centre is retained; contact centres may be interpolated. Source velocity comes from the existing model's ephemeris derivative, not a recovered contact-substep chord slope. Frame phase precision and finite source/state bounds are validated. Undefined projected tidal azimuth, invalid parent chains, non-solid bodies and unsupported geometry are refused.

Longitude and east/north directions are unavailable at the pole. Terrain values also remain unavailable there rather than choosing an arbitrary overview meridian. Elsewhere, terrain context distinguishes observed-derived MOLA overview with contributing cells/counts/datum, Earth relief with procedural detail and generated terrain. Overview terrain clearance is separate from spherical contact; areoid shape, local hazard geometry and metre-level registration are unverified.

Kinetic energy uses actual current mass, including remaining propellant and retained cargo. It does not establish force, stopping distance, attitude, gear response, structural integrity, surface stability or a safe landing. No atmosphere/entry heating, attachment, automatic guidance, habitat cargo transfer, empirical calibration or uncertainty propagation is implemented by this build. Actual finite touchdown response and physically co-located delivery remain next. The graphics requirement stays queued.

[Research and design requirements](LANDING_AND_DELIVERY_RESEARCH.md), [original flight](WORLD_VEHICLE_FLIGHT.md). No authoritative world/save field or golden change is introduced by these read-only frames and converted initial conditions.
