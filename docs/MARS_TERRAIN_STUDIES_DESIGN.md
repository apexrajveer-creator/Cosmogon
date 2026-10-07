# Next connected build: Mars terrain studies

The exploration build links historical landing references, data, geographic camera travel and
viewpoints. The next useful research/learning mechanism is a measured topographic transect using
that same field, before adding habitat engineering or promising local terrain fidelity.

## Product mechanism

Choose two body-fixed coordinates (or historical landing references), calculate a bounded great-circle
section, inspect actual height/support values in a plot, select a sample to probe/visit, and export the
numeric section with a reproducible definition. A student can compare broad topography between sites;
a researcher can inspect the exact source cells and frame/radius assumptions. This is an overview
section of the implemented dataset, not a rover route, landing assessment or high-resolution survey.

## Implementation direction

- Engine-independent definition: endpoints with optional verified landing identity, dataset/version,
  spherical distance radius in metres and 2–1024 samples. The native path uses the selected body's
  actual radius; it rejects legacy procedural Mars. Do not put observer definitions in Universe.
- Use DMath and unit-vector interpolation along the unique shorter great-circle arc. Coincident
  endpoints produce a documented zero-length section. Reject ambiguous antipodal endpoints instead
  of silently inventing an azimuth. Preserve endpoint values exactly, including seam/pole semantics.
- Each row contains distance along the reference sphere, north/east coordinate, the shared bilinear
  areoid height, source-cell indices/medians/counts/weights and unavailable uncertainty. Export units,
  definition, dataset hashes and frame limitations alongside CSV. Distance is not a full areoid or
  ellipsoidal geodesic; a finely sampled curve does not add source spatial information.
- Native height-distance graph, clear units, measured min/max/relief and an actual row cursor. Mark
  samples that depend on zero-shot provider-interpolated cells without inventing confidence scores.
  Selecting a row fills the existing probe; travel remains explicit. Larger definitions calculate off
  the UI thread if measurement shows a need. No live external service or new dependency is required.
- Test known equatorial/polar distances, seam wrapping, identical and antipodal endpoints, limits,
  height agreement, endpoint/site validation, repeatability and protected export recovery. Inspect
  actual native graph/row/probe captures and run the ordinary app; retain unchanged Universe/golden.

## Data research / later refinement

The existing MOLA 4ppd labels and metadata establish the current overview grid, height datum and
support-count interpretation: [NASA PDS MGS MOLA derived topography](https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/), DOI 10.17189/1z1b-kv84. The section must retain those limits.

Primary USGS search results read 2026-10-07 identify the
[MOLA 463 m DEM](https://astrogeology.usgs.gov/search/map/mars_mgs_mola_dem_463m) as a finer global
candidate (128ppd). Full-page retrieval returned 403, so product metadata, exact downloadable payload,
rights, reference frame, checksums and missing-data behavior still require verification before ingestion.
Nothing from that candidate is included in the current product. A shaded-relief image is not an
interchangeable height grid. Local stereo/mission products and their rights/registration need separate
review; even a finer global DEM does not supply landing safety, engineering slopes or current rover paths.

The shared section engine, native graph/row/probe path, protected export and headless replay are now
implemented; see [MARS_TERRAIN_STUDIES](MARS_TERRAIN_STUDIES.md). Native definition import, durable
study preferences and finer terrain delivery remain unimplemented. Validation is recorded separately.
