# Paired responses over recorded time

A complete paired design now keeps all four validated branch recordings in the native viewer. In **Paired results**, choose **Time curves**, **Four-response table** or **Files & method**. The shared observation index selects an actual stored world time independently of the live universe. The original protected `factorial.json` remains the final observation; selected-time analysis does not rewrite it.

Time curves show discrete recorded marks for average A change, average B change or the difference of conditional A changes. Signed values retain a zero reference; unavailable values remain gaps. Click/drag chooses the nearest actual observation, and the labeled index slider offers another control. The full table exposes four responses and all eight contrasts at that same selected time. A0/A1 and B0/B1 denote first/second supplied levels, even for descending values. A snapshot's previous completed flows remain marked as pre-edit when appropriate.

**Files & method** exports a protected new folder containing `trajectories.json`, `contrasts.csv` and a completion `manifest.json` written last. JSON contains the original plan, common-state fingerprint/data versions, all branch summaries/source descriptors, formulas, limitations, immutable final analysis and all 18 metric trajectories. CSV contains 40 columns: metric/unit/index/time, four responses, eight contrasts, four support states, matched branch indices/world times, completed interval ends/durations and four configuration-match flags. Missing numeric values are empty CSV cells and JSON nulls; never zeros. Existing output is refused. An incomplete folder has no completion marker. Native export revalidates the saved design on a worker and retains the current view if it fails.

CLI defaults remain compatible:

```sh
./target/release/cosmogon-cli habitat-inspect --recording /tmp/paired-design
./target/release/cosmogon-cli habitat-inspect --recording /tmp/paired-design --observation 12
./target/release/cosmogon-cli habitat-inspect --recording /tmp/paired-design --output /tmp/new-trajectory-analysis
```

`--observation` is an explicit zero-based stored index (0–1023 and within the design). `--output` exports every stored time/metric while stdout remains the selected or default final report. Both options require a complete paired root, not an ordinary branch or snapshot study. Validation precedes output creation. Existing branch inspection and single-setting studies remain unchanged.

Packaged capture equivalents:

```sh
./dist/Cosmogon.app/Contents/MacOS/cosmogon --habitat-recording /tmp/paired-design \
  --habitat-metric vapor --habitat-effect difference_in_differences --habitat-sample 12 \
  --panel habitat-paired --focus Mars --speed 0 --capture /tmp/paired-time.png \
  --capture-metadata --capture-after 240 --quit
```

`--habitat-effect` accepts `average_a`, `average_b` or `difference_in_differences`; it requires a recording/sweep input. Panels `habitat-paired-table` and `habitat-paired-files` select the other views. Capture/performance metadata separately records immutable final sweep analysis, selected-time analysis and the actual curve selection/support counts. Curve data is cached per metric and recalculated only when the loaded design/metric changes.

[NIST's full factorial design](https://www.itl.nist.gov/div898/handbook/pri/section3/pri3331.htm) supplies the experimental ordering/contrast method. Conditioning on stored time, exact interval support and graph/export rules are Cosmogon engineering choices. No interpolation, statistical significance, predictive confidence, independent noisy replications, global optimization or field-hardware calibration is implied. This extends phase 9/16; physical greenhouse exchange and the wider master directive remain incomplete.
