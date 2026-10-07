# Next Mars refinement: verified 16ppd labels

Primary label research after terrain-section checkpoint `ad6f5b4`, read 2026-10-08.
The web reader could not fetch these archive pages; bounded direct HTTPS retrieval obtained the
provider's actual XML labels. This is research/build direction; no finer grid has been ingested or
made available in the app yet. Current worlds still use their stored 4ppd/procedural identifier.

## Verified primary product facts

- [Topography label](https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/meg016/megt90n000eb.xml): `megt90n000eb`, primary label version 1.1; 2880 north-to-south lines × 5760 samples, SignedMSB2, 0.0625° / 16 pixels per degree. Derived median heights with provider interpolation where bins have no shots.
- [Observation-count label](https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/meg016/megc90n000eb.xml): matching dimensions/type/resolution, number of observations per pixel. These are counts, not uncertainty or confidence.
- Equirectangular map; planetocentric latitude, positive-east longitude, IAU2000. GMM3/mgm1025 degree/order 50 areoid, topography = radius minus areoid radius. Source reference sphere 3396 km; equatorial pixel resolution 3.7044613374 km. The app's current Mars radius is 3389.5 km; do not conflate those references.
- Label 1.1 corrects swapped resolution/scale metadata and recomputes the map corner from precise ENVI scale. It does not describe new observations. Primary/extended mapping observations remain February 1999–June 2001 plus specified 1998/mapping polar observations.
- The topography label reports roughly 55% of equatorial bins with at least one shot and gaps up to 12 km between profiles. A 3.7 km pixel is not a promise of 3.7 km independent ground measurements. Retain coverage and missing uncertainty through probes, plots and exports.
- Each raw grid would occupy 33,177,600 bytes (5760 × 2880 × 2), a combined 66,355,200 bytes before packaging/cache overhead. Raw payload hashes, archive checksum inventory, actual binary ranges and count validity are not verified yet; no binary was downloaded in this research step.

Retrieved label SHA256:

| Product | Label SHA256 |
|---|---|
| topography | `c393221db2de31916b30113ce28841dd967456dadb9bf2735eb30496eb37afa2` |
| counts | `845eda1aa007144821f3915e3c45dada4fbccb4f862de08f761c80406a156c28` |

Local label copies: `/tmp/cosmogon-mola-refinement-research`. Rights/attribution are governed by
the same PDS bundle and the provider FAQ already retained in the source registry; a separately
processed version still requires its own pinned product/label/checksum metadata.

## Connected implementation requirements

1. Verify both raw products against the provider archive inventory and pin SHA256. Extend processing
   without resampling/fictional detail, validate axis order/type/label version, reject unexpected missing
   or negative count values and retain reproducible processed checksums. Measure conversion/data-size
   and query overhead before choosing embedded vs a verified optional pack.
2. Provide a shared dataset-aware read contract for terrain, coverage, probe, section and camera.
   Renderer-only finer height would misalign scientific values and travel. Preserve existing dataset
   versions; changing a world's source must be explicit and explain consequences. Saves must require
   the exact field/version, and a missing optional pack must fail recoverably rather than invent relief.
3. Centralize fixed provenance/resolution/frame/coverage metadata. A finer overview remains an overview;
   no local engineering slope, rover path or landing-safety claim. Carry unknown uncertainty unchanged.
   Native source selection and dataset/cache availability should explain what is ready, missing or failed.
4. Geographic anchors and section definitions must pin their dataset. Restore/replay against a changed
   field must report the mismatch. Source changes must invalidate visual terrain/bakes and dependent
   sampling correctly, reset/release incompatible tracking and follow ordinary simulation edit/undo rules.
5. Verify byte-exact processing, representative pixels/source weights, seams/poles, counts/interpolated
   support, old saves, explicit source changes and failure recovery. Preserve required deterministic tests,
   deliberately repin the golden only if initial serialized state changes and document the decision.
   Inspect native comparisons and measure an ordinary run before claiming completion.

This is the next connected phase-8 build. Native section-definition import/preferences and a visible
transect line remain useful follow-ons; habitat engineering follows with actual shared state, sources,
conservation/balance checks and honest model bounds rather than decorative panels.

## Subsequent implementation

The connected [MARS_REFINEMENT](MARS_REFINEMENT.md) build now implements the separately pinned 16ppd field. Both payloads and primary labels match the provider inventory; raw height range is −8177..21171 m and coverage contains 903558 zero-shot bins. This supersedes the initial binary/availability unknowns above; overview/coverage/frame limitations remain.
