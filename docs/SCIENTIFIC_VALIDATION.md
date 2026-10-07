# Measured orbit checks — 2026-10-07

This slice computes measurements from the installed deterministic engine. It changes no physical
algorithm, Universe field or golden fingerprint (0x579903fd05a28945). All results below were computed
from fixed cases, not inferred from test names or hardcoded into the interface.

## Conditions and coverage

Two massless circular test-particle fixtures start at 1 AU around GM = G × SOLAR_MASS, relativity
and collisions off, with 1000 or 2000 macro steps over exactly one analytic period. Their return
position errors are 7.93389420e-9 and 4.95852113e-10 of the initial radius. Energy and angular-momentum
changes stay below 1e-11 for these cases. Step doubling reduces this particular return error by
approximately sixteen; it does not establish convergence for every orbit or encounter.

The cached JPL case uses the embedded 2026-01-01 TDB states, the Accurate hybrid N-body/rails preset
(400 steps per shortest orbit, central-body 1PN enabled) and 365 days. It compares heliocentric,
ecliptic-J2000 endpoint positions with nine rounded locally cached 2027 coordinates:

| Quantity | Computed residual |
|---|---|
| Mercury heliocentric position residual | 94.5190195 km |
| Venus heliocentric position residual | 5.52391583 km |
| Earth heliocentric position residual | 590.841412 km |
| Mars heliocentric position residual | 3.13847074 km |
| Jupiter heliocentric position residual | 0.198132628 km |
| Saturn heliocentric position residual | 0.240557842 km |
| Uranus heliocentric position residual | 0.392722351 km |
| Neptune heliocentric position residual | 0.132487845 km |
| Moon heliocentric position residual | 786.781695 km |

The report includes 16 initial body vectors, nine endpoint coordinates, identifiers, source/retrieval
context, epoch, frame, integrator conditions and actual computed metrics. Its 1500 km / Moon 3000 km
limits are historical regression guards, not uncertainties or acceptable research precision.
Earth–Moon error is not independently attributed. J2, tides and small-body perturbations are missing.
No interval-wide bound, covariance propagation or external-integrator comparison is supplied. These
fixtures do not validate climate, stellar, collision or civilization models.

## Workflow evidence

The CLI report reproduced exactly on repeated computation. Existing report files were rejected
without replacement. Malformed numbers/flags and missing values returned status 1 without a panic.
Research creation defaults to the dynamic Solar Lab; explicit scenarios and complete saved bases
still take precedence.

A three-branch Jupiter-mass sweep measured Earth eccentricity at four states separated by ten days.
Earth mass stayed fixed at 5.97217e24 kg. Final eccentricities were 0.01587039997753068,
0.015870006479036327 and 0.015869616373502204 for input masses 9.49e26, 2.3725e27 and 3.796e27 kg.
Replay and native worker output matched every CSV row exactly. This is a short exploratory response
of the existing model, not a validated long-term planetary prediction. The primary measurement is
persisted and native comparisons display enough digits and a first-branch difference to distinguish
small computed effects; displayed digits do not imply observational precision.

Completed branches remain visible after cancellation/failure, while automation returns failure;
partial results never become a successful batch. Prepared definitions refuse a foreign pinned base
rather than silently rebinding it. Older definitions without primary_measurement remain loadable.

The workspace suite passes 212 tests, with one existing ignored test and no failures (app 44, CLI 1, core 32, physics 38, simulation 96, doctest 1). Clippy reports existing warnings only; none were added by this slice.
Artifacts are local and ignored; no public release, notarization or manual accessibility claim is made.

Final native captures show computed model residuals, populated 40-state Earth plots and completed cross-object results; the window ends above the tool dock and numeric rows remain unbroken. Scrolling is still needed for lower charts/reference rows. Direct pointer/keyboard and screen-reader use were not verified.

The final native arm64 macOS package passes ad-hoc signature and disk-image checksum verification. An ordinary packaged run lasted 39.81 seconds and recorded 100 Jupiter states without panic; packaged cross-object CSV also matches the headless study exactly. The final packaged results capture confirms the compact value/difference table. This is a local checkpoint, not a release.
