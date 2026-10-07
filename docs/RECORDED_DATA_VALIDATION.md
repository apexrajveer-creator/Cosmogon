# Recorded data and comparison workspace verification

This checkpoint extends scientific-validation checkpoint 535d327. It reopens exported numeric
recordings, displays synchronized plots and branch differences, and stores presentation choices
separately from the universe. No physical algorithm, save schema or golden state changes.

## Shared numeric path

Native reopening and `cosmogon-cli inspect --recording DIR` use the same bounded reader. The reader
checks format/schema, canonical parameter keys/units, finite values, chronological states/edit counts,
row count and input sizes. It reads manifest, numeric samples and events without loading initial.cosmo
or evolving the selected universe. These checks establish format consistency, not authenticity.

Difference plots and `inspect --recording DIR --reference BASE --measure eccentricity` use the same
subtraction routine. They require exactly matching elapsed sample times and do no interpolation.
The first completed branch is a reference, not necessarily an unmodified control. Displayed digits,
relative changes and branch differences do not establish physical accuracy or observational precision.
Initial provenance and uncertainty labels apply to the initial state; no covariance propagation is supplied.

The three-branch Jupiter-mass study from SCIENTIFIC_VALIDATION is the retained fixture. Opening its
third branch preserves all four recorded states and every numeric CSV value. The final Earth
eccentricity difference from its first branch is -7.836040284772694e-7 at elapsed 2592000 seconds.
Native batch curves read the computed branch recordings, not generated presentation data. Up to eight
branches appear as curves; all completed branches retain their numeric output/table. Up to four native
plots share elapsed range, cursor and time units; manual navigation and assistive-technology behavior
remain unverified.

## Preferences and recovery

Explore/Learn/Research changes guidance rather than physical models. Favorites, graph parameters,
scales, time units, comparison mode and normalized time range have a separately versioned workspace
file. Changes require explicit Save workspace. Invalid/newer files remain untouched with writes disabled.
The cursor is transient. The reader and preferences round trips have regression tests, including
malformed nonfinite data, duplicate states, incompatible units and foreign preference versions.

Automated import rejects invalid recordings before GPU initialization. Short imported-recording jobs
with --quit use rendering on the main thread: repeated early-exit trials exposed an intermittent
pipelined-renderer shutdown hang. Performance reports state `measurement.pipelined_rendering`; timing
comparisons must use matching scheduling. Interactive sessions keep the existing pipelined renderer.

## Evidence

Final commands, process statuses, native captures, logs and checksums are retained locally under
ignored dist/recorded-data-validation. Captures exercise reopening Earth while Mars remains selected,
completed Earth trajectories with Jupiter mass changed, favorites and a preserved newer workspace.
No public release, push, notarization, screen-reader or manual pointer/keyboard claim is made.

The final workspace suite passes 214 tests with one existing ignored test (app 45, CLI 1, core 32,
physics 38, simulation 97, doctest 1). Clippy reports the existing warning counts only (core 2,
physics 1, simulation 9, app 11). The golden fingerprint remains 0x579903fd05a28945.

Six invalid automated imports returned status 1 and four valid imports returned status 0 with
bounded timeouts. Conflicting import/export arguments returned 1 before app initialization.
Combined import/profile completed with the scheduling marker false. Native three-branch CSVs match
the headless fixture byte for byte. The comparison graph’s lower axis/legend and the table require
scrolling in the initial window; full multi-panel visual layout remains a polish task.

The arm64 macOS package passes ad-hoc signature and disk-image CRC verification. An ordinary
packaged recording completed in 46.54 seconds with 100 states and no panic. This is a local
checkpoint, not a public release.
