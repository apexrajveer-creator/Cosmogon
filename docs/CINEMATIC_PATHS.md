# Authored camera paths

Open **View → Camera & Photo → Paths**. Opening the pane does not start motion. **New path**, then **Capture current view**, records a key from the actual camera and lens. Navigate to another composition and capture again; the spacing control assigns its presentation time. Saved viewpoints can also supply keys without moving the live camera. Drafts stay in memory until explicitly exported.

**Compose**, **Keys** and **Files** separate authoring, selected-key editing and protected file controls; playback/scrub remains above each view. Select a key to change its time, outgoing timing curve or vertical field of view. Replace its pose/lens with the current view, remove it, or preview it. The first key starts at zero; later times must increase. Removing the first key shifts remaining times to zero. A path holds 2–64 keys and lasts at most 3600 presentation seconds. Key spacing for authoring is 0.1–300 seconds.

**Play** advances presentation time using the real frame interval. **Pause** freezes that time while an object reference may continue moving with the simulation. **Start** previews zero; the slider scrubs through the same evaluator used by playback. At the endpoint playback pauses and holds the preview. **Stop & release** leaves camera/lens at their last values and restores ordinary camera control. Manual movement/look/scroll, a camera or lens change, missing objects, undo/world generation changes or returning to the menu stop path control. The pause menu and clean-photo preparation suspend playback. Opening a file preserves the camera until an explicit preview/play action. Simulation pause and acceleration remain independent.

All keys share the translating reference chosen from the current camera when creating the path. Positions are f64 render-frame metres relative to it; an absent reference means absolute render-frame positions. Saved viewpoints are converted to that common reference at authoring time. Body spin is not followed. **Look at selection** replaces authored orientation with a current-object look direction. **Use authored orientation** restores normalized shortest-rotation quaternion interpolation. A coincident target is refused; near the look-direction pole the evaluator uses a perpendicular alternate up axis. This avoids undefined orientation, but is not a continuous roll-constrained camera rig.

Position and all four lens/colour fields interpolate using the outgoing key's linear or smoothstep timing. Linear is constant parameter speed within each segment; different segment lengths/durations can change travel speed abruptly at keys. Smoothstep eases into/out of each segment and comes to rest at its endpoints. Neither promises spline continuity, arc-length parametrization, orbital flight dynamics or collision-free travel. Exposure, warmth and saturation can be authored through Photo controls and a replacement/captured view. Warmth remains a colour adjustment, not Kelvin.

## Files and evidence

**Open** validates a complete versioned definition before replacing a previous draft. Unsupported versions, unknown fields, invalid unit quaternions/lenses/times, missing references and mismatched worlds are refused. Regular files are bounded to 192 KiB. **Save new file** creates an exclusive file and protects existing ones; a failed write can leave an incomplete new file, which will fail later validation. Copy definition JSON also works for an incomplete draft, but playback/export require at least two valid keys.

`cosmogon-camera-path` schema 1 stores name, world identity, optional stable anchor/look-at targets and explicit keys (`time_s`, `position_m`, `orientation_xyzw`, complete presentation and easing). Persistent sandbox IDs and reference dataset identities follow existing viewpoints. Developer worlds use scenario/seed identity. Imported single-file states instead use the retained initial state's FNV-1a serialization fingerprint, so reopening the same source survives a new transient sandbox ID. This is a compatibility guard, not a cryptographic identity; a changed single-file state requires a new path or explicit re-authoring. Definition JSON is app presentation data, never Universe/save state.

Photo/capture and performance reports retain the entire path, presentation phase, simulation time, play/control state, exact evaluated pose and reference positions, actual camera position and positional correction distance. Ground rendering retains the existing 60 m endpoint clearance. An authored position inside terrain can be corrected; the intended path and actual image then differ. Frame metadata describes the capture request, not a physical measurement or video frame stream.

For a reproducible frozen view, use an exported definition matching the launched world:

```sh
./target/release/cosmogon --new lab --seed 1 --camera-path /tmp/shot.json \
  --camera-path-time 4 --panel paths --speed 0 --quiet \
  --capture /tmp/shot-at-four.png --capture-metadata --capture-after 240 --quit
```

`--panel paths-keys` and `--panel paths-files` capture the two additional editor views. `--camera-path-time` defaults to zero and must fall within the file's duration. Ordinary `--camera-path-play` opts into motion; captures require a frozen time and reject playback. Conflicting camera/surface/tour/menu/offline-export/time-advance flags are refused. Path and requested pose are checked before starting the renderer. A path flag without another launch source defaults to Solar System Lab.

Video export, depth-of-field, simulation-rate synchronization, spline/arc-length control, changing anchors during one path, swept collision and physical flight remain incomplete. Manual pointer/keyboard and assistive-technology acceptance remain separate from automated numerical/capture checks.
