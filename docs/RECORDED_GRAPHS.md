# Navigate recorded habitat graphs

Recorded habitat graphs now share time-window zoom/pan, original-value cursors, three display scales and reusable layout files. This is presentation of retained simulation data; graph choices never alter the live world, recorded observations or complete numerical exports. See the [design memo](RECORDED_GRAPH_DESIGN.md) and [verification evidence](RECORDED_GRAPH_VALIDATION.md).

Open Habitat engineering → Recordings. Ordinary curves/comparisons and paired Time curves show **Zoom in**, **Zoom out**, **Earlier**, **Later** and **Full range**. Zoom changes the visible interval about its middle; pan moves by half the current span and keeps that span at the boundaries. Right-drag pans continuously and double-click restores the complete time extent. Click/primary-drag selects the nearest actual visible A observation. The observation slider still accesses every original row. If the selected observation is outside the window, its original value remains selected and **Center selected** brings it back into view. A window without stored points says so explicitly. Snapshot points appear at their actual sole time.

Comparison extent includes both full aligned recordings. Elapsed alignment uses each recording's own initial world time; World alignment retains seconds since J2000. Graph-layout time-label choices convert those coordinates to seconds, terrestrial hours or terrestrial days. Real irregular timestamps determine placement and selection; display does not invent intermediate observations. Ordinary connecting lines remain visual guides and break at unavailable/omitted values. Paired effects use discrete points.

Choose a scale from the graph toolbar:

- **Linear** preserves negative, zero and positive values and fits supported values inside the current time window.
- **Log10 (positive)** displays positive values only. Nonpositive values remain in the selected row and complete exports; their omission count is explicit. Missing-model support is counted separately. A scale with no supported values shows no invented numeric y axis.
- **Signed log** has an explicit threshold `t` in the current metric's units. Display coordinates are `y/t` for `|y| ≤ t`, and `sign(y) × [1 + log10(|y|/t)]` outside. Tick labels invert that mapping to original units. This is a piecewise display scale, not a transformed-data analysis or predictive model.

Choose **Graph layout** for exact start/end fractions, scale, signed threshold, zero inclusion, time-label units, metric/effect and alignment. Fractions refer to the **current complete aligned time extent**, so a reusable layout adapts to a different recording without claiming a common epoch or source. The minimum span is one millionth of that extent. Imported layouts are strict, bounded, versioned JSON. Unknown properties, invalid numbers/enums/windows, future versions, oversized/nonregular inputs are refused while the current view remains intact. Export names a **new file** and protects existing files. **Save as my graph layout** explicitly saves separate app preferences; valid saved choices reopen on later launches. Invalid/newer preference files remain protected and are not automatically replaced. Import does not silently save preferences.

A native startup can apply a layout before opening a graph:

```bash
./target/release/cosmogon --new lab --focus Mars --speed 0 \
    --habitat-recording /tmp/my-paired-experiment \
    --habitat-graph /tmp/graph-layout.json --habitat-sample 12 \
    --panel habitat-paired --capture /tmp/graph.png \
    --capture-metadata --capture-after 240 --quit
```

`--habitat-graph` requires a layout file and a recording or paired/single-setting sweep launch. It is validated before graphics startup. Explicit `--habitat-metric`, `--habitat-effect` and compatible `--habitat-alignment` override imported defaults. `--panel habitat-graph-layout` opens the layout editor; paired inputs retain paired-effect context. The metadata records the effective layout, exact complete/visible time bounds, actual visible original values/indexes, transformed y bounds, missing support, scale omissions and whether the selected observation is visible. The original final analysis, selected-time table, replay report and CSV/JSON exports remain independent of scale/window.

Example file:

```json
{
  "format": "cosmogon-habitat-graph-layout",
  "schema_version": 1,
  "metric": "vapor",
  "effect": "average_a",
  "alignment": "elapsed",
  "window": [0.25, 0.75],
  "scale": "signed_log",
  "signed_threshold": 0.1,
  "include_zero": false,
  "time_unit": "hours"
}
```

The [official Matplotlib axis documentation](https://matplotlib.org/stable/users/explain/axes/axes_scales.html) gives primary software context for explicit scale/threshold choices; [NIST's transformation guidance](https://www.itl.nist.gov/div898/handbook/pmc/section5/pmc52.htm) explains ordinary logarithms' positive domain. This native renderer uses no Matplotlib dependency. Signed mapping and navigation are explicit engineering choices. Broader graph integration, manual input/assistive-technology verification, uncertainty, field calibration and the master directive remain incomplete.
