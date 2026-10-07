# Mars MOLA source refinement

Build on `codex/mola-refinement`, 2026-10-08. Verification: [MARS_REFINEMENT_VALIDATION](MARS_REFINEMENT_VALIDATION.md).
Primary evidence and requirements: [MARS_REFINEMENT_RESEARCH](MARS_REFINEMENT_RESEARCH.md).

## Mechanism and source choice

The new separately pinned 16ppd product is 5760 × 2880 with 0.0625° cells (3.704 km equatorial
spacing). It adds finer overview relief, not local rover mapping or independent ground measurement
at every pixel. Counts travel with each height read. Provider profile gaps can reach 12 km;
zero-shot cells remain explicitly interpolated and uncertainty remains unavailable.

Both datasets are embedded immutable source bytes, decoded only for the four contributing cells.
This avoids a duplicated heap grid and a mutable external-cache dependency in deterministic physics.
The 16ppd pair adds 66,355,200 uncompressed bytes. No extra dependency, network at runtime or missing
pack fallback is introduced. An optional verified pack/download manager is still future work;
this build does not claim to implement that directive requirement. The local arm64 binary is about 124 MiB; the packaged disk image is about 68 MiB. Warm query medians on this host are 44.39 ns (4ppd) and 58.20 ns (16ppd), five 200000-query runs after warmup. These exclude loading, textures, GPU and simulation and are not an app speed claim.

Existing worlds and new defaults keep 4ppd. Surface → Choose terrain source exposes the available
4ppd and 16ppd fields; changing one uses an ordinary simulation edit with undo/redo. Read-only
references must be cloned first. The edit refreshes normal derived environmental values, records its
source choice and preserves the existing gravity/orbital phase. It is not an automatic save migration.
The initial golden fingerprint stays `0xf2c4140a45ae509b` and passes the full required suite. There is no initial state change.

Terrain geometry, climate/resource terrain queries, numeric probes, landing markers, surface camera
altitude and terrain sections use the same selected field. Meshes and texture bakes include terrain
source/seed in their cache identity; completed old-source bakes are discarded. Existing geographic
anchors and bookmarks pin their source. Changing the field releases incompatible tracking and rejects
restoring a mismatched bookmark. Existing sections retain their rows but become stale until recalculated.

Save manifests record the 16ppd version separately. Unsupported/mismatched versions refuse to load
with an explanation; legacy procedural and 4ppd state remain supported. Numeric section definitions
pin the exact dataset, and CLI replay against a different world refuses to silently reinterpret them.
Probe JSON and section source packets include the actual dataset metadata, hashes, coverage and frame.

## Developer examples

```sh
python3 tools/process_mola.py --ppd 16 --fetch
./target/release/cosmogon-cli datasets
./target/release/cosmogon-cli surface --site curiosity --mars-terrain 16
./target/release/cosmogon-cli terrain --mars-terrain 16 --save /tmp/mars16.cosmo
./target/release/cosmogon-cli section --initial /tmp/mars16.cosmo --output /tmp/mars16-section
./target/release/cosmogon-cli section --initial /tmp/mars16.cosmo --definition /tmp/mars16-section/definition.json
./target/release/cosmogon --new lab --mars-terrain 16 --surface-site curiosity --panel surface --quiet \
  --capture /tmp/mars16.png --capture-metadata --capture-after 240 --quit
```

`--mars-terrain 4|16` is an explicit source edit. The CLI `terrain` command refuses existing output
files; section/probe commands only change their in-memory world unless a separate terrain save is
requested. Definitions keep their chosen source/sphere/samples; no hidden interpolation across versions.
The dataset registry now lists both embedded grids rather than a single metadata object.
