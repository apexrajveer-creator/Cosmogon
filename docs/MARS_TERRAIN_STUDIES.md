# Mars terrain sections

Open **View → Mars terrain section**, select **Measure a terrain section** in Mars → Surface,
or find terrain section in the command palette. This is a numeric study of the shared MOLA overview
field; it does not change the universe or calculate a rover route.

Select two historical landing references or enter north latitude and east longitude. **Calculate
section** takes 2–1024 uniformly spaced samples along the unique shorter great-circle arc. It uses the
current simulated Mars radius for distance on a reference sphere. Height remains the shared MOLA
areoid-referenced value. Coincident identical endpoints give zero distance; ambiguous antipodal or
numerically coincident but differently expressed polar endpoints report an explanation. The angular
coincidence/antipodal tolerances are 1e-10/1e-9 rad; they do not establish source coordinate accuracy.

The graph shows actual height/distance rows, sampled minimum/maximum/relief and sample spacing.
Hover or use the sample slider to inspect a row. A coloured support point means at least one contributing
cell has no direct observations and was interpolated by the provider. Counts are not confidence scores.
**Inspect this coordinate** fills the existing probe. **Visit this coordinate** uses the Surface
altitude/tracking/view controls. Neither edits physics. Endpoint landing identity is retained at the
exact endpoint; intermediate rows are custom coordinates, not current rover positions.

Inputs and computed results are separate: changing an input marks the graph stale until recalculated
and disables row actions/export. The initial section is calculated from Curiosity's and InSight's
historical coordinates using the same implementation as the headless tool. This is actual dataset
sampling, not a static demo. A world without the pinned observed Mars field shows an explanation and
retains its stored terrain. No network or new dependency is required.

## Export and replay

A new folder contains `definition.json`, `section.json`, `samples.csv` and `sources.json`.
CSV has 35 explicit columns: index, distance m, latitude north/east longitude degrees, height above
areoid m, unavailable height uncertainty (empty), a provider-interpolation flag and seven fields for
each of four supporting cells (row, column, centre coordinates, median height m, observations, weight).
JSON retains null uncertainty, method limits and the definition; sources retain dataset checksums and
historical landing references/frame limitations. Existing folders, including empty folders, are protected.
An I/O failure reports an incomplete export and preserves that directory for inspection.

```bash
./target/release/cosmogon-cli section --from-site curiosity --to-site insight \
    --samples 64 --output /tmp/new-mars-section
./target/release/cosmogon-cli section --definition /tmp/new-mars-section/definition.json
```

Named sections use the selected stored/new Mars radius (`--initial` can supply a complete saved world).
A definition keeps its explicit distance sphere and sample count; overrides that would be silently
ignored are rejected. Custom endpoint definitions use the same schema as the exported file; `landing_id`
is null for arbitrary coordinates. Definitions are limited to 64 KiB and require the supported schema
and exact terrain version. Replaying the same valid definition produces the same numerical rows.
Native definition import/persistent study preferences are not yet provided; headless replay is available.

## Interpretation

The current field has 0.25° source cells, about 14.82 km equatorial sampling on the provider's sphere.
Extra section samples interpolate that field, adding no source resolution. Plot axes scale independently,
so visual angles are not physical slopes. Sample extrema do not bound every point on the continuous
arc. Distances on the chosen sphere are not ellipsoidal/full-areoid geodesics. Unknown per-cell height
uncertainty and endpoint covariance are not estimated. Colours, fine shading and the rendered spherical
base remain separate presentation/model assumptions. These results support broad terrain exploration,
not local geology, engineering slopes, safe landing or rover traversal decisions.

Verification: [MARS_TERRAIN_STUDIES_VALIDATION](MARS_TERRAIN_STUDIES_VALIDATION.md).
