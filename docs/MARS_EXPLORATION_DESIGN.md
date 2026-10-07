# Next Mars exploration build

Continuation from observed-terrain checkpoint 013a682. The landing registry, shared f64 frame,
probe markers, snapshot/tracked travel and geographic viewpoints are now implemented. Validation
is recorded in [MARS_EXPLORATION_VALIDATION](MARS_EXPLORATION_VALIDATION.md). Higher-resolution
terrain, mission imagery/paths and landing engineering remain future work.

## Connected user journey

An explorer selects a documented landing location, sees what source supports its coordinates,
reads the shared MOLA coverage/height, then moves the camera above the same place. A student can
compare relief and coverage at different sites. A researcher can export the coordinate/frame/source
record and inspect support cells, while coarse-grid and coordinate limitations remain explicit.

The landing record is historical. Do not present it as the current rover position or mission status.
Live rover tracks require their own dated, verified ingestion and permissions. No mission photograph
should be pasted onto terrain without a registered footprint, projection, acquisition context and
rights evidence. Existing generated worlds must not acquire observed terrain behind the user's back.

## Initial verified reference points

| Landing reference | Latitude N (°) | Longitude E (°) | Landing UTC date | Primary reference |
|---|---:|---:|---|---|
| Perseverance | 18.44463 | 77.45088 | 2021-02-18 | [NASA Langley NTRS 20210024644](https://ntrs.nasa.gov/citations/20210024644) |
| Curiosity | -4.59 | 137.44 | 2012-08-06 | [NASA GISS Mars24 landing references](https://www.giss.nasa.gov/tools/mars24/help/landers.html) |
| InSight | 4.50 | 135.62 | 2018-11-26 | [NASA GISS Mars24 landing references](https://www.giss.nasa.gov/tools/mars24/help/landers.html) |
| Viking 1 | 22.27 | 312.05 | 1976-07-20 | [NASA GISS Mars24 landing references](https://www.giss.nasa.gov/tools/mars24/help/landers.html) |

Sources read 2026-10-07. The GISS mission narrative is dated 2023-12-09; it cannot substantiate
present mission status. Rounded reference coordinates are not uncertainty estimates. NTRS identifies
its record as public and US Government work with public use permitted. Retain per-record references,
source date, coordinate convention and missing positional covariance; investigate any mapping-frame
mismatch before promising physical registration at metre scale.

## Implementation and acceptance

1. Add a versioned, bounded, engine-independent landing-reference registry with stable IDs, dates,
   east-longitude/planetocentric-coordinate declarations, sources and unavailable uncertainty. Expose
   native selection and headless metadata. Selecting a site fills the coordinate probe, without edits.
2. Share an f64 body-fixed-to-render transform between surface camera placement and geometry. The
   current body_rotation returns an f32 quaternion, and current viewpoints follow body translation
   without following rotation; neither supports a precise surface anchor as-is.
3. Add explicit altitude above the shared height field and snapshot/tracked modes. Track body spin
   and translation only when requested. Reset flight velocity during travel, avoid zero-length look
   vectors at poles and honor the existing terrain clearance. A probe location should have a visible
   marker, with occlusion and label layout, so the numeric reading can be related to the scene.
4. Store camera anchors/presentation preferences outside Universe. Restore site identity, coordinates,
   height reference and lens; do not rewind the world or fabricate a physics result. Preserve existing
   viewpoint files and protect unknown versions.
5. Verify source records, frame round trips, seam/pole behavior, altitude and shared height agreement.
   Inspect native captures for surface placement, marker/probe correspondence and horizon composition;
   include sustained ordinary runs and legacy/reference-world behavior.

The MOLA layer remains a 0.25° overview. Navigation must not turn it into a claim of safe landing,
accurate local slopes or high-resolution geometry. Higher-resolution patch delivery needs actual
source products, pinned checksums, a bounded cache, invalid/missing-tile recovery and datum checks.
