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
for the open universe are listed. A saved view follows the current position of its reference object,
retaining the stored world orientation and lens. Restoring enters free flight and does not rewind time.
The UI reports missing objects and write errors; unsupported or corrupt libraries cannot be overwritten.

## Current limits

* f64 positioning and a camera-relative renderer preserve the existing precision architecture.
  Resolution still decreases at enormous absolute distances: centimetre accuracy at galactic
  coordinates is not promised. Tests exercise motion from metre to ~10²¹ m scales.
* Solid surfaces have 60 m endpoint clearance protection, matching the existing terrain precision
  envelope of orbit mode. This is not swept collision. Very high speeds can pass
  through bodies between frames. Camera motion has no simulated mass or physical consequences.
* Terrain follows the nearest solid body; procedural detail remains procedural. This work adds no
  observed Mars terrain or other new astronomical datasets.
* Saved views follow translation, not body rotation; they are not geographic surface bookmarks.
* Key rebinding, cinematic paths/video, depth of field and tiled high-resolution export remain future work.
* A photograph is a rendered presentation, not a calibrated instrument exposure. GPU readback is
  asynchronous; metadata describes the request state rather than claiming exact frame synchronization.
