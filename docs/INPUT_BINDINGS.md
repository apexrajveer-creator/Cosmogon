# Native keyboard controls

View → Camera & Photo → Controls opens the editor; the help window also links there. Camera and Everyday views select an action, show its current key and offer either **Press a new key…** or an unused-key dropdown. Search narrows the action list; All current mappings shows the complete configuration.

Changes apply for this run immediately. Assignments are unique across 24 camera and everyday actions. A conflict names the existing action and preserves the previous mapping. Escape cancels capture. Modifier chords, multiple simultaneous keys and unsupported keys are refused. After capture, cancellation, reset or import, keyboard actions wait until all held keys are released. Closing/hiding the editor or entering the menu cancels capture. Camera velocity is stopped while assignment is gated.

The mapping controls held camera movement/look/roll, orbit zoom/yaw, orbit/free switching, orbit selection/system view, pause, two speed-up/down keys, interface visibility, clean photos, help and the developer overlay. Defaults preserve existing keys. Mouse gestures, Shift boost, Alt/Option fine movement, Escape recovery and Cmd/Ctrl palette/save/undo/redo shortcuts retain fixed behavior. Command chords do not also fire a rebound unmodified action. Camera/HUD controls still respect text-field focus; the mouse continues to be handled by existing interfaces. Cinematic manual interruption uses rebound camera actions.

## Physical key meaning and files

These are physical key positions, labeled by US layout names; printed letters can differ on other keyboards. They are not logical typed characters. Supported keys are A–Z positions, digits, arrows, Home/End/PageUp/PageDown, Space/Tab/Enter/Backspace, comma/period/equal/minus and F1–F12. Escape and modifier keys cannot be assigned. International logical-key entry, mouse/controller/chord rebinding and global OS shortcuts remain separate work.

**Save for later launches** explicitly writes `input-bindings.json` under the platform Cosmogon data directory (next to the app's saves, independent of Universe and settings.toml). Valid current primary files are replaced using a new temporary file and rename after comparing the previously loaded bytes. Files changed externally are refused until reloaded. This comparison is not a multi-process locking protocol. Missing primary files use defaults; invalid/future/unreadable/nonregular files stay protected with runtime defaults and a warning. Runtime edits and new-file export remain available. Reload discards unsaved edits; reset applies runtime defaults and still requires explicit saving.

Files view imports a complete strict version-1 definition for this run, exports to a new protected file or copies current JSON. Definitions require all 24 typed actions, supported unique keys and the `cosmogon-input-bindings` format. Unknown fields/actions, duplicate action entries, future versions, missing actions and unsupported keys are refused. Regular-file reads are bounded to 16 KiB; FIFOs are refused before opening. Existing exports are never overwritten. No UserSettings or simulation save migration is added; the deterministic golden remains unchanged.

Developer import is explicit and validated before GPU startup:

```sh
./target/release/cosmogon --input-bindings /path/to/controls.json --panel controls \
  --capture /tmp/controls.png --capture-metadata --capture-after 240 --quit
./target/release/cosmogon --new lab --input-bindings /path/to/controls.json --panel help \
  --capture /tmp/controls-help.png --capture-metadata --capture-after 240 --quit
```

`--panel controls-everyday|controls-files` selects the other editor views. Developer captures ignore ordinary physical input and expose the actual mapping in photo metadata; they do not demonstrate manual key presses. No launch/capture automatically saves controls. `--input-bindings` without another world source defaults to Solar System Lab and conflicts with offline audio-only export. See [INPUT_BINDINGS_DESIGN](INPUT_BINDINGS_DESIGN.md) for evidence and [INPUT_BINDINGS_VALIDATION](INPUT_BINDINGS_VALIDATION.md) for actual verification.
