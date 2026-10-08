# Native keyboard control verification

This connected build follows camera checkpoint `1f0f940`. Simulation/save state and golden `0xf2c4140a45ae509b` are unchanged. See [INPUT_BINDINGS](INPUT_BINDINGS.md) for behavior and scope.

## Required checks

`cargo test --release --workspace`: **302 passed, one existing ignored**. Eight added tests cover held versus pressed mapped input driving actual free-flight integration, defaults, duplicate/reserved/unknown keys and actions, capture conflict/modifiers/Escape/release, cancellation on closing the editor, bounded regular-file/FIFO import, protected export/persistence/reload/external changes, future-file fallback and launch validation.

`cargo clippy --release --workspace --all-targets` passes with existing warnings and no new implementation warning. Release app/CLI build and local macOS app/DMG packaging succeed. The initial disk-image creation reported no space; the filesystem had 120 GiB available and a retry succeeded without deleting files. Ad-hoc signature and disk image verification pass; this is local macOS evidence, not Developer ID/notarization or alternate-platform verification.

## Actual native and file cases

Twenty-eight distinct cases: fifteen pre-GPU refusals and thirteen native captures. Refusals cover future/unknown fields, duplicate keys/action JSON, reserved Escape/modifiers, unsupported keys/actions, missing actions/files, oversized files, FIFO/directory, empty launch option and offline-export conflict. FIFO refusal precedes opening. Fixture hashes and the owner's primary controls file remain unchanged.

Thirteen accepted final layouts were visually inspected: default/custom Camera, Everyday, Files and dynamic help, rebound navigation/photo labels, path keys, historical Mars landing travel and live habitat WHY. Sidecars confirm exact loaded definitions, unsaved state and inactive capture/release gating; existing-feature contexts are checked. Earlier help had its customization button under the dock. Moving it above the list, adding bounded scrolling and compact key labels corrected this. Intermediate captures are retained but not all were visually inspected; repeats do not increase distinct counts.

## Ordinary run and limits

The packaged app ran normally for **48.08 seconds**, with custom mapping, live simulation, the Controls panel and advancing authored playback. A 900-frame report after 120 warm-up frames confirms pipelined rendering, the exact custom definition, no capture/release gate and **17.7726674 presentation seconds** of active playback. The harness stops the process after observation; no panic occurred. It does not press physical keys.

Ignored local evidence in `dist/input-bindings-validation/` contains commands, fixtures, actual PNG/sidecars, refusal logs, required checks, ordinary report and source/artifact hashes. Engine-level tests verify capture/release and mapped input; these captures and metadata do not establish actual manual keyboard gestures, controller interaction, international layouts, assistive technology or global OS shortcut behavior. Those remain acceptance gates. No push or public release occurred. The master directive remains incomplete; controlled greenhouse transpiration planning follows.
