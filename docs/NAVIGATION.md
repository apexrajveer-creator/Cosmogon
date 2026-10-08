# Native navigation and photography

Open **View → Camera & Photo**, or find Camera, Photo or Viewpoints in Cmd/Ctrl+K.
These are app controls; they do not alter simulation physics. The simulation only pauses when the
explicit pause control is selected.

## Explore

| Control | Action |
|---|---|
| G | Detach from orbit; press again to return to the selected object |
| F | Orbit the selected object |
| Home | Frame the selected system |
| W / S, A / D | Forward/backward and sideways in free flight |
| R / C | Up/down in camera-local coordinates |
| Q / E | Roll |
| Right drag / arrow keys | Look around |
| Wheel | Change free-flight speed multiplier |
| Shift / Alt (Option) | Fast ×10 / fine ×0.1 movement |
| P | Save a clean PNG |
| Tab | Hide/show the interface |

Navigate offers automatic speed based on nearby objects or a manual metres-per-second base, a
speed multiplier, optional inertia, response, mouse sensitivity and vertical inversion. “Stop” clears
velocity. Save preferences makes these defaults for later launches. Typing in fields does not fly the
camera. The camera is a navigation tool, not a spacecraft model.

## Compose

Photo provides vertical field of view, exposure EV, cool/warm colour adjustment and saturation.
Warmth is a colour-grading adjustment, not a physical Kelvin temperature. “Reset lens & colour”
restores neutral defaults; “Save as defaults” persists the lens settings. A viewpoint also preserves
these settings. Star luminosity, temperature, climate and solar-power calculations are untouched.

“Take clean PNG” temporarily hides interface, labels and gizmos, then restores their previous state.
The image is saved at native window pixel resolution to `~/Pictures/Cosmogon`; the toast confirms
actual completion or reports failure. Optional `.png.json` metadata records camera pose, lens,
seed, dataset versions and capture-request simulation time. Two images taken within one second
receive distinct filenames.

## Return to a view

Viewpoints stores up to 128 named entries across universes in the app data directory. Only entries
for the open universe are listed. An ordinary saved view follows the current position of its reference object, retaining the stored world orientation and lens. A tracked surface view instead restores its body-fixed coordinate, radial altitude, local look direction, landing identity and lens at the current world time. A snapshot restores its stored object-relative world pose once and continues in free flight. Restoring never rewinds time.
The UI reports missing objects and write errors; unsupported or corrupt libraries cannot be overwritten.

## Current limits

* f64 positioning and a camera-relative renderer preserve the existing precision architecture.
  Resolution still decreases at enormous absolute distances: centimetre accuracy at galactic
  coordinates is not promised. Tests exercise motion from metre to ~10²¹ m scales.
* Solid surfaces have 60 m endpoint clearance protection, matching the existing terrain precision
  envelope of orbit mode. This is not swept collision. Very high speeds can pass
  through bodies between frames. Camera motion has no simulated mass or physical consequences.
* Terrain follows the nearest solid body. New Solar System worlds use the embedded MOLA overview on Mars; legacy saves preserve their procedural terrain.
* Surface bookmarks require Mars with the pinned MOLA terrain. The body frame follows the implemented axial tilt/spin model, not a SPICE navigation solution. Registration of the source coordinates at metre scale remains unverified.
* Key rebinding, cinematic paths/video, depth of field and tiled high-resolution export remain future work.
* A photograph is a rendered presentation, not a calibrated instrument exposure. GPU readback is
  asynchronous; metadata describes the request state rather than claiming exact frame synchronization.

## Mars surface exploration

Select Mars → Surface. Choose a historical landing reference or enter north latitude and east longitude.
The selection fills the shared MOLA probe and its visible marker; **Visit coordinate** moves the camera.
Altitude is radial metres above the same field on the app's spherical base, bounded to 60 m–100,000 km.
**Ground overview** keeps the footprint in view; **Horizon** composes toward local north and the limb. The probe may be outside a horizon composition. **Follow the planet’s motion and spin** keeps the camera in the body-fixed frame. Right drag/arrow/Q/E
look adjustments persist relative to that frame. Translational movement releases tracking. **Release to
free flight** preserves the current world pose and clears velocity; F/G can return to orbit. If the
reference body disappears or its terrain no longer applies, tracking releases with a visible explanation.

The four initial references are Perseverance, Curiosity, InSight and Viking 1. They mark historical
UTC touchdown positions, not current rover locations or current mission status. Rounded coordinates
are not uncertainty estimates. Click a visible landing marker to open its probe; double-click visits it
using the Surface altitude/tracking controls. Labels obey the overlay/photo preferences, label spacing
and geometric horizon; this is not detailed terrain occlusion. Probe exports include source dates,
coordinate interpretation, null uncertainty and the actual supporting MOLA cells. Existing export files
are protected. The new viewpoint library writes version 2, reads released version 1, and protects newer
or invalid files from overwrite.

Developer capture: `--surface-site perseverance --surface-altitude 100000 --panel surface` defaults
to a new Solar System Lab, selects Mars and visits the historical coordinate. `--surface-snapshot`
disables following; `--surface-horizon` composes toward the limb. Other focus/latlon flags conflict explicitly. Ordinary `--latlon` and its `noon`
lighting helper remain available for orbit captures. Sidecars distinguish tracked/free/orbit mode and
include surface anchor/source information; snapshot coordinates describe the placement request, not a
claim that the camera remains above that point later. Headless `cosmogon-cli sites` emits the registry;
`cosmogon-cli surface --site curiosity` emits a source-supported reading. No network is needed at runtime.

## Authored shots

Camera & Photo → Paths connects current views and saved viewpoints to explicit timed keys, interpolation, look-at, lens changes and Play/Pause/Stop/scrub. See [CINEMATIC_PATHS](CINEMATIC_PATHS.md) for file identity, clearance and timing limits. No simulation or golden changes.
