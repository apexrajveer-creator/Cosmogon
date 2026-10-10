# Original saved and current live starting worlds

A restored experiment plan now has an explicit starting-world choice. Open a complete single-setting or paired study in Habitat engineering → Recordings → Run experiment. **Original saved starting world** is the default. It reuses that study's actual `base.cosmo`, including the original habitat identity, finite stores, configuration, source versions, world clock and unfinished fixed-period remainder. **Current live world** uses the selected live habitat and captures its state when Run is pressed. Neither choice rewinds or edits the live universe. Choose a fresh output folder; the ordinary staged configuration edits apply to independent branches before sampling.

The saved-source caption identifies the retained folder, original habitat, clock and fingerprint. The live choice identifies the current clock and habitat. Changing unopened folder text or draft setting levels does not replace retained completed observations. Results switch only after a worker successfully completes and validates the new design. Invalid ordinary edits, changed original starting files and existing destinations preserve the previous recorded view. Completed folders and response catalogs keep their existing formats.

The original world is held read-only. Frozen-world plan preflight is cached by the exact typed plan, rather than reading files or copying a whole universe every frame. The worker rereads the bounded common file and checks its fingerprint, sources and habitat before any new output. This protects the opened source identity; it is not cryptographic authentication or empirical calibration. A newly opened study establishes its own original origin, and a completed live-origin rerun therefore retains that live starting state as its new saved origin.

Repeat an existing complete design exactly:

```bash
./target/release/cosmogon-cli habitat-rerun --recording /tmp/study --output /tmp/repeated-study
```

Use new typed settings from the same original stores:

```bash
./target/release/cosmogon-cli habitat-rerun --recording /tmp/study \
  --definition /tmp/new-plan.json --output /tmp/new-study
```

The optional definition is an ordinary bounded single or paired sweep plan. The original recorded habitat identity is retained. For a deliberately different saved world, use the existing `habitat-sweep --initial OTHER_SAVE --definition PLAN --output NEW_DIR` mechanism. The new rerun command refuses unrelated flags rather than silently ignoring them. Original input folders remain untouched.

Packaged capture of the source choices:

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/current.cosmo \
  --focus Mars --speed 0 --panel habitat-experiments --habitat-recording /tmp/study \
  --habitat-origin saved --capture /tmp/original-origin.png \
  --capture-metadata --capture-after 240 --quiet --quit
```

`--habitat-origin saved|live` chooses the staged origin after a complete design opens. Without it, reopening defaults to saved. `--habitat-rerun-output NEW_DIR` additionally reruns the restored original plan before capture/export. Automated live reruns require an explicit `--load CURRENT_WORLD`, use the recorded habitat ID, and validate its presence before starting the renderer. Interactive live experiments use the habitat selector. `--habitat-branches A,B` keeps selected single-study branches through an automated rerun; paired studies retain all four responses without that flag. The selected observation is retained within the plan's sample count. Dependent figures, balances and replay wait for the new design; a failed run cancels those queued actions rather than exporting the previous study as the new result.

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --load /tmp/current.cosmo \
  --focus Mars --speed 0 --panel habitat-figure --habitat-recording /tmp/study \
  --habitat-origin live --habitat-rerun-output /tmp/live-study \
  --habitat-branches 1,3 --habitat-sample 4 --habitat-graph /tmp/layout.json \
  --habitat-figure-output /tmp/live-figure --capture /tmp/live-origin.png \
  --capture-metadata --capture-after 240 --quiet --quit
```

Native metadata includes `study_origin`: the retained original snapshot/pins, staged source choice and exact worker request. Live requests capture the current state fingerprint/time/sources; saved requests retain original pins and recheck disk evidence before running. These are presentation/request records, not new Universe state or study schema. No simulation laws, save format, golden fingerprint, source constants or graphics change. Maintained crop conditions, chemical energy and equipment capacities remain engineering assumptions; physiology, chamber calibration, uncertainty and the master directive remain incomplete.
