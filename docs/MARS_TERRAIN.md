# Observed Mars overview terrain

The [source refinement build](MARS_REFINEMENT.md) additionally offers a separately pinned 16ppd grid through explicit source selection; existing worlds and new defaults preserve their original field.

New Solar System/reference/Lab worlds use the NASA MGS MOLA 4-pixels-per-degree MEGDR topography
and observation-count products. Simulation surface sampling and native terrain rendering use the
same bilinear height field. There is no additional generated Mars elevation noise. The existing
colour, climate, deposit and fine shading models remain modelled/generated.

## Read and inspect

Select Mars → Surface. The coordinate probe displays planetocentric latitude and east-positive
longitude in degrees, height above the GMM3 degree/order 50 areoid in metres, neighbouring source
cell medians, counts and blend weights. Zero observations marks a provider-interpolated cell.
Counts are not uncertainty/confidence; per-bin uncertainty is unavailable rather than zero.
The probe does not move the camera or edit the world.

```sh
python3 tools/process_mola.py --fetch
./target/release/cosmogon-cli datasets
./target/release/cosmogon-cli surface --target Mars --latitude 18.625 --longitude 226.125
./target/release/cosmogon --new lab --focus Mars --panel surface --quiet \
  --capture /tmp/mars-data.png --capture-after 180 --quit
```

The CLI accepts --initial to probe a saved world. Old worlds without the MOLA identifier retain their
original generated terrain and refuse a MOLA probe; importing this dataset does not replace their
state. The new terrain identifier and changed initial terrain-dependent state deliberately re-pin
the golden fingerprint to 0xf2c4140a45ae509b (previous 0x579903fd05a28945). Save schema stays unchanged;
older saves remain readable. Future grids must have distinct identifiers rather than silently replacing
this pinned source.

## Resolution, coordinates and sources

The 1440 × 720 grid has 0.25° pixels, about 14.82 km at the equator. Pixel centres begin at 89.875° N,
0.125° E. Heights are medians of MOLA ground returns; empty bins were filled by the provider.
The app blends neighbouring medians, so probes are derived values and cannot resolve subpixel terrain.
The renderer uses a spherical reference body radius and does not reconstruct the areoid shape.
It does not establish local terrain, current geology, safe landing slopes or engineering conditions.

Underlying MEGDR observations were acquired principally in 1999–2001 with additional polar coverage.
The 2003 grid is distributed with PDS4 label 1.1 (September 2026 projection-metadata correction).
The source labels are bundled. The converter makes an exact endian conversion; pinned SHA256
checks reject changed inputs. Source/output checksums, frame, units and attribution are in metadata.json.
The source range is -8068 to 21134 m and 9137 bins have zero direct observations.

Primary sources: [NASA PDS MOLA bundle](https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/),
[provider overview](https://pds-geosciences.wustl.edu/missions/mgs/megdr.html),
[public-domain archive FAQ](https://pds-geosciences.wustl.edu/documents/ViewingPDSImagesAndExportingToOtherFormats.pdf).
DOI: 10.17189/1z1b-kv84. Full redistribution/provenance inventory: [SOURCE_REGISTRY](../data/SOURCE_REGISTRY).

This completes an observed overview/coverage probe, not the full directive Mars phase. Higher-resolution
patch delivery, mission-site layers, image footprints, surface movement and settlement engineering remain.

## Checkpoint verification

The suite passes 216 tests with one existing ignored test on this macOS ARM host. Clippy reports
the existing warnings only. Grid checks verify extrema, missing-observation count, orientation,
known source centres, wrapped longitude, finite bounds, convex blending and shared terrain sampling.
Save checks preserve both the new MOLA identifier and a legacy procedural Mars without inventing a
dataset attachment. Native geometry checks compare Mars surface radius against observed heights
at a mountain, deep basin, seam and pole, preventing generated hills/craters in its measured geometry.

CLI probes reproduced exactly, returned the expected 20009 m/732-observation grid centre, exposed
a zero-count provider-interpolated cell, refused nonfinite/out-of-range coordinates and a legacy
procedural world, and reproduced both converted grids byte for byte. Native captures inspect the
Mars source-cell table, close relief, Earth-specific explanation and legacy generated-terrain label.
Manual pointer/keyboard, assistive technology and Windows/Linux native views remain unverified.
Higher-resolution patch accuracy and propagated uncertainty remain unavailable.

The final arm64 macOS package passes ad-hoc signature and disk-image CRC verification. A sustained
packaged Mars run completed in 36.59 seconds with 1800 measured frame samples, correct MOLA
dataset context and no panic. Profile timings from this correctness run are not a controlled speed comparison.
Local ignored evidence is retained in dist/mola-validation; this is not a published release.
