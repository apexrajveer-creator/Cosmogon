# Reopen and locate a terrain study

Connected native build after MOLA source-selection checkpoint `227a4b5`. Verification: [TERRAIN_STUDY_REPLAY_VALIDATION](TERRAIN_STUDY_REPLAY_VALIDATION.md). No simulation schema/state, dataset, dependency or golden change.

The Mars terrain section window can reopen definition.json from any protected section packet.
The shared bounded reader validates version, source, endpoints, sample count, radius and arc; the app
calculates every row before committing imported state. Failure preserves the previous definition/rows.
Source, radius and samples remain pinned. This includes a reference sphere different from the current
simulated Mars; distances remain study-sphere values, while geographic placement uses the app body.

A reopened field different from the current world can be graphed and exported independently, with
a visible mismatch explanation. Probe/travel and the on-globe trace require the same source as the
world. “Use current Mars source & radius” explicitly changes inputs, then requires recalculation.
Changing worlds/sources no longer silently overwrites imported definitions. Legacy worlds remain intact.

A checkbox shows the sampled trace and selected graph row in the same f64 body-fixed frame as landing
markers. Hidden/back-side rows break the trace rather than drawing across the planet. Coral segments
have a zero-shot endpoint; this is a sampled study trace, not a continuous rover route or safety model.
Stale inputs/different sources disable the trace. Photo sidecars retain last-calculated and pending
study definitions, actual selected row and whether inputs match; a recorded study may be hidden.

```sh
./target/release/cosmogon --new lab --mars-terrain 16 \
  --terrain-section /tmp/mars16-section/definition.json --focus Mars --panel section \
  --capture /tmp/reopened-section.png --capture-metadata --capture-after 240 --quit
```

`--terrain-section` defaults to a new Lab if no scene/load is selected. Automated invalid definitions
are rejected before starting the GPU. Ordinary opening reports failures in the window. No world/source
change is implicit in import. Graph view keeps the numeric profile prominent; setup/import controls collapse, and content scrolls on shorter displays. Map view keeps a compact row selector above the globe; `--panel section-map` opens it for captures. Durable study preferences,
a definition library, continuous extrema, precise geodesics and uncertainty propagation remain future.
