# Vehicle footprint, loads and ground-hold evidence

The shared simulation calculation, exact original-world reports, native controls/footprint diagram and headless commands now expose actual supporting terrain feet. This is the next mechanism after persistent shear; it does not attach a craft, declare a completed landing or grant habitat supplies. The master directive's complete landing/delivery/hardware scope remains unfinished.

## Actual quantities

Every reading retains the actual craft/world clocks, source frame, mass, ground-relative translation/rotation, individual loads, structural/stroke/friction utilization, normal/tangential storage and aggregate applied force/torque. Unloaded feet remain explicit. Zero friction capacity has no invented utilization ratio.

A planar footprint requires at least three distinct loaded modeled surface sites, bounded normal dispersion/plane departure, resolved coordinates and a nondegenerate convex hull. Sorting and hull ordering are deterministic; at most eight feet are accepted. Geometric centre projection, force-weighted site centroid and applied reaction wrench centre have separate coordinates and signed edge margins. Positive site weights normally put their centroid inside the hull; this fact is not a stability certificate.

Ground sites and compliant force-application points differ. The actual wrench centre uses `n × (torque_about_centre − plane_origin × force) / (n · force)` on the declared plane. Remaining normal torque is retained separately. The force-weighted site centroid can therefore differ from that wrench centre; the implementation does not silently move actual forces to the terrain surface.

The hold comparison asks which contact force/torque would follow the ground at the actual centre with zero relative motion. It uses retained gravity sources, current scheduled thrust only while finite propellant remains, actual mass/inertia and material-frame acceleration. Ground acceleration includes translation, changing spin and centripetal terms. Relative acceleration additionally includes the Coriolis term. Rotation torque is checked separately from the force-only hold line through the centre.

Frame acceleration comes from centred derivatives of the original replayed source velocity/spin operator, a half-step comparison and roundoff indicators. Those are numerical indicators, not rigorous bounds or a new accurate ephemeris. An independent Kepler comparison exposed approximately `1.05e-6 m/s²` difference at the tested legacy Mars rail clock; the check declares a 0.1% relative source-operator budget. This discrepancy is separate from derivative refinement. The existing orbital operator, world state and golden fingerprint are preserved.

## Native workflow

Supply vehicles → **Footprint & loads** shows live contact evidence or independently replayed original-world observations. Select a retained observation to inspect the equal-scale footprint diagram. The diagram distinguishes geometric centre, weighted sites, applied wrench and required hold line. Missing plane/hold support has a reason and no fabricated margin.

**Authored comparison limits for a new report** exposes every geometric, motion, force/torque and derivative threshold. A completed report retains its original definition while new controls are edited. Derive from the original recorded world, optionally writing a fresh protected folder; **Verify and reopen support report** restores the retained definition and exact original flight. No later live world supplies historical acceleration or terminal geometry.

An instantaneous envelope checks persistent shear, geometry, coordinate support, motion/acceleration, load margins, force/torque balance and absence of powered thrust. Inclusion does not prove future stability, attachment or cargo access. A stopped unsupported flight yields retained unavailable evidence, not a reconstructed landed state. Original smooth-contact recordings can produce reports but are excluded from static-support interpretation. Original spherical recordings lack feet and are refused.

## Headless workflow

```sh
./target/release/cosmogon-cli support-definition --output /tmp/support-limits.json
./target/release/cosmogon-cli flight-support --recording /tmp/complete-contact-flight --definition /tmp/support-limits.json --output /tmp/support-report
./target/release/cosmogon-cli support-inspect --recording /tmp/support-report
```

Reports contain `definition.json`, `support.json`, `support.csv`, `manifest.json` and the complete original `flight/` bundle. The 29-column CSV keeps unavailable quantities blank; full JSON retains vectors, per-foot detail, exclusions and exact source frame. Reopening independently recomputes all observations, manifests and numeric files. Existing output destinations and corrupted evidence are refused.

Native automation: `--flight-recording COMPLETE --panel vehicle-support --support-definition LIMITS --support-output NEW`, or `--support-recording REPORT --panel vehicle-support`. `--flight-observation INDEX` selects the original row. Live inspection uses `--load WORLD --panel vehicle-support`. Definition/report/source/output conflicts and protected destinations are checked before GPU startup.

## Sources, verification and continuation

[MIT contact/planar support notes](https://underactuated.csail.mit.edu/humanoids.html) provide the reduced force/moment context; the actual source/model registry identifies assumptions and absent empirical validation/uncertainty. Our footprint and envelope are authored analyses of the existing simulation, not calibrated gear, soil or mission operations. [Mechanism and inventory constraints](VEHICLE_SUPPORT_NEXT.md).

Six new simulation tests check deterministic hull geometry, independent compliant-wrench moments, actual coupled readings, source/edit support, bounded independent Kepler comparison and exact complete report replay/corruption/terminal refusal. Native worker and command-boundary tests supplement them. Final verification passes **514 release tests plus one existing ignored**, six release/package checks, 45 CLI/native cases, eight inspected packaged captures and a 60.053035-second ordinary live run. The new style warnings were corrected before the final pipeline. All 173 frozen Rust files match; existing unrelated warnings remain. [Evidence and limitations](VEHICLE_FOOTPRINT_SUPPORT_VALIDATION.md).

The next build must model a supported surface-operation interval, finite handling/storage capacities and actual changing cargo mass/energy/momentum, then transfer held consumables to the same co-located habitat stocks. No timer, low-speed freeze or stock grant is allowed. Graphics remain queued.
