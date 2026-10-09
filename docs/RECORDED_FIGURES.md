# Recorded research figures

Open Habitat engineering → Recordings, reopen an actual ordinary recording or four-branch design, then choose **Research figure**. Choose the measured quantity, actual paired contrast if applicable, elapsed/world time, seconds/hours/days, zero policy and scale. Zoom/pan the same recorded graph; **Save figure to new folder** captures the current layout and immutable observations before starting its worker. The displayed choices can subsequently change without relabeling an export already requested. A failed export leaves the loaded evidence available; choose a new folder and retry.

These are figures of model observations. They support comparing assumptions and carrying evidence into reports; they do not establish real-hardware accuracy, uncertainty or safety. Read [geometry/thermal method notes](THERMAL_STUDIES.md), [paired methods](PAIRED_TRAJECTORIES.md) and [recording provenance](HABITAT_RECORDINGS.md). Broader research workflows and the master directive remain incomplete.

## Bundle

| File | Meaning |
|---|---|
| `figure.svg` | Static vector chart with metric/unit/time coordinates, scale caption, actual sample markers, omission counts and source-file references. No external fonts, assets or scripts. |
| `points.csv` | All original observations, including points outside the selected window. Full-precision original values, actual observation index, world/completed time, completed interval, chosen coordinate, unit, support and display state. |
| `figure.json` | Exact typed input samples and source summaries, request-time layout, plotted series and ordinary-comparison or paired-contrast analysis. Paired designs retain all four branches and applicable geometry/thermal assumptions. |
| `layout.json` | Existing strict version-1 graph layout, reusable independently of the recording. |
| `manifest.json` | Completion marker written after all other files. A folder without this marker is incomplete. Existing folders/files are protected. |

CSV series index maps to `figure.json.series`; ordinary A/B retain independent stored times. Elapsed coordinates begin at each recording's first observation; world coordinates preserve absolute simulation time. Paired effects use all four actual branches with shared support checks, rather than treating the first/last corner as one isolated setting change. Source/model identity is retained from the recording; a path typed into an unopened field never becomes the source identity.

Markers are actual discrete samples. Connecting lines are visual guides and break at unsupported or scale-omitted points; they do not reconstruct intermediate states. Positive log10 omits zero/negative values as **scale_omitted**, distinct from unavailable observations. Signed log preserves zero/negative values and names its threshold in original metric units. Ticks retain original units. Linear bounds use the actual nonzero range, and interval-aware labels resolve small changes. Constant, empty and all-unavailable views show their actual support without inventing data. A zero-only linear chart uses a one-unit display baseline with 5% padding; those bounds are canvas choices, not measured variation or uncertainty. There is no smoothing or invented uncertainty band.

## Command line

```sh
./target/release/cosmogon-cli habitat-figure --recording /tmp/cabin-study \
  --definition /tmp/graph-layout.json --output /tmp/new-cabin-figure
./target/release/cosmogon-cli habitat-figure --recording /tmp/cabin-a \
  --reference /tmp/cabin-b --definition /tmp/graph-layout.json --output /tmp/new-comparison-figure
./dist/Cosmogon.app/Contents/MacOS/cosmogon --habitat-recording /tmp/paired-study \
  --habitat-graph /tmp/graph-layout.json --habitat-figure-output /tmp/new-native-figure \
  --panel habitat-figure --quiet --speed 0 --capture /tmp/figure-controls.png \
  --capture-metadata --capture-after 240 --quit
```

CLI requires all three primary paths and refuses unrelated options. A paired root already supplies its four sources and rejects an extra `--reference`. Native `--habitat-figure-output` requires a recording or automatic sweep and a nonempty new folder; capture waits for the export worker. The same protected readers, graph definition and numerical/vector exporter serve both interfaces. No image generation, simulation edits, save-schema changes or new dependencies are involved. Rendering transforms are presentation calculations and never enter the deterministic simulation.

The implementation follows explicit SVG path geometry from the [W3C SVG paths specification](https://www.w3.org/TR/SVG2/paths.html). All labels and embedded metadata are XML-escaped. Independent replay of the simulation remains a separate explicit action; retaining inputs in a figure is not proof that the model is empirically correct. See [design](RECORDED_FIGURES_DESIGN.md) and [verification](RECORDED_FIGURES_VALIDATION.md).
