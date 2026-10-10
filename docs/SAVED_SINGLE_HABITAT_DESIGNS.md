# Complete saved single-setting habitat designs

A completed single-setting sweep can now be reopened as a whole study. The shared reader checks its original common world, typed plan, source versions, every ordinary configuration edit, every recorded branch, exact planned observation times and actual final metric catalog. The completed study stays independent of the live world. Structural checks are separate from an explicit fresh replay and from empirical calibration.

In Habitat engineering → Recordings & comparisons, open the sweep folder containing `plan.json`, `base.cosmo` and the completed `manifest.json`. The initial comparison uses the first and last branches. The two branch menus select any retained independent recordings, including the same branch for a self-comparison. Source path, plan and branch identity stay separate from editable folder text and staged controls. WHY, pressure, graph layout, balances and figures use the selected actual recordings. A single-setting study has no four-response factorial effects.

Run experiment restores the typed setting, levels, number of observations and interval with an empty output destination. A new run still uses the explicitly supplied saved common world or current live starting world, following the existing experiment mechanism. To repeat the original common origin exactly, supply the study's `base.cosmo`; restoring controls alone does not rewind the live world.

CLI examples, using an existing completed study:

```bash
./target/release/cosmogon-cli habitat-inspect --recording /tmp/study
./target/release/cosmogon-cli habitat-inspect --recording /tmp/study --observation 4
./target/release/cosmogon-cli habitat-inspect --recording /tmp/study --output /tmp/new-trajectories
./target/release/cosmogon-cli habitat-verify --recording /tmp/study --output /tmp/new-verification
./target/release/cosmogon-cli habitat-figure --recording /tmp/study --branches 1,3 \
  --definition /tmp/graph-layout.json --output /tmp/new-figure
```

`--observation` is a zero-based original stored row, returned for every branch with the plan's metric catalog. Without it, inspection returns the unchanged completed manifest. The protected trajectory export contains all retained branch/time/catalog values in JSON and long-form CSV, complete design context and a last-written completion manifest. Missing metric values remain empty/null. Completed minute-flow metrics retain their interval meaning; they do not reconstruct intervening history. Original full samples and worlds remain in the source study. Each new trajectory file is bounded to 32 MiB before any output folder is created.

`--branches A,B` is a pair of zero-based saved branch indices used only for single-design figure export. Native `--habitat-branches A,B` supports complete single-design reopening and new single-plan launches. Bounds and source inputs are checked before starting the renderer. The figure retains the complete original plan/manifest, selected branch indices, exact selected samples, source/model context and separately editable graph layout. Without explicit choices, complete single figures select first/last. Ordinary recording figures and old paired report/figure serialization retain their prior formats.

For example, inspect the packaged selected-branch view:

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/study/base.cosmo \
  --focus Mars --speed 0 --panel habitat-figure --habitat-recording /tmp/study \
  --habitat-branches 1,3 --habitat-graph /tmp/graph-layout.json --habitat-sample 4 \
  --habitat-figure-output /tmp/new-native-figure --capture /tmp/single-figure.png \
  --capture-metadata --capture-after 240 --quiet --quit
```

`--habitat-verify` starts independent replay after the complete design opens. The same worker/progress/cancellation mechanism now reconstructs every planned branch at every exact stored time, rather than assuming four branches. It compares complete canonical samples, original/final fingerprints and source versions. Expand **Verify complete saved design** to see progress and retained mismatch evidence and export a fresh protected report. Changing the selected comparison does not change the replay scope: it always checks the whole saved design.

The reader bounds metadata to 1 MiB, common worlds to 32 MiB and uses the existing recording/source/endpoint reader for each branch. Existing plan limits allow 2–16 branches and at most 8192 total observations. Incomplete manifests, changed typed plans/source identity, lost branches, incorrect initial edits, altered final response catalogs and off-plan observation times are refused. A plausible interior observation edit can remain structurally readable; explicit replay detects it. These checks do not establish cryptographic authenticity, hardware validity, physiological calibration, uncertainty or optimization.

Maintained crop chamber conditions, chemical energy, capacity and auxiliary power remain stated engineering assumptions. No physics, saved state, data pins, legacy golden or graphics changed. Long paired setting descriptions now wrap within the native pane.
