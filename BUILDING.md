# Building

## Requirements (developers only — players need nothing)
* Rust 1.85+ (`rustup`), a GPU with Metal / Vulkan / DX12.
* macOS: Xcode Command Line Tools (for `codesign`, `iconutil`, `hdiutil`).
* Linux: `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`.

## Common commands
```bash
cargo run --release -p cosmogon                      # the app
cargo run -p cosmogon --features dev                 # faster incremental builds (dynamic linking; not distributable)
cargo test -p cosmogon_core -p cosmogon_physics -p cosmogon_sim
cargo run --release -p cosmogon_cli -- run --scenario sol --seed 4 --years 202000
cargo run --release -p cosmogon_cli -- check --scenario garden --seed 9 --years 3000000   # determinism
cargo run --release -p cosmogon_cli -- survey --scenario garden --life hopeful --seeds 8 --years 2000000000
```

## Developer flags for the app
`--new sol|garden|neighbourhood --seed N --life 0|1|2 --advance-years Y --focus NAME --distance RADII
--yaw RAD --pitch RAD --speed INDEX --select-civ --hide-ui --load PATH --capture FILE.png --capture-after FRAMES --quit`

## Packaging
* **macOS:** `scripts/package-macos.sh` → `dist/Cosmogon.app` + `dist/Cosmogon-<v>-macOS.dmg`
  (ad-hoc signed). `UNIVERSAL=1` builds arm64 + x86_64. For public distribution set
  `SIGN_ID="Developer ID Application: …"`, then notarise:
  `xcrun notarytool submit dist/*.dmg --apple-id … --team-id … --wait && xcrun stapler staple dist/*.dmg`.
* **Windows:** built in CI (`release.yml`): portable zip and an Inno Setup installer
  (`packaging/windows/cosmogon.iss`); the icon is embedded by `crates/cosmogon/build.rs`. Release builds have
  no console window.
* **Linux:** tarball in CI; AppImage/Flatpak planned.
* Icon: regenerate with `python3 scripts/make_icon.py` (original artwork, stdlib only).

Everything the app needs (shaders, data files, fonts) is embedded in the binary.

## Where things are stored
Saves: `~/Library/Application Support/Cosmogon/saves` (macOS), `%APPDATA%\Cosmogon\saves` (Windows),
`~/.local/share/Cosmogon/saves` (Linux). Settings: the platform config directory, `Cosmogon/settings.toml`.

## Performance reports

Open View → Performance, or use Cmd/Ctrl+K and search Performance. Detailed counters are enabled
in Settings for the next launch. For an automatic run:

```sh
./target/release/cosmogon --new lab --focus Saturn --distance 5 --speed 0 \
  --profile /tmp/saturn-run1.json --profile-warmup 120 --profile-frames 600 --quit
```

`--panel performance` opens the panel for captures. Choose a new report path each time; export
refuses to overwrite existing files. See [PERFORMANCE.md](docs/PERFORMANCE.md) for measurement
meaning, sample bounds, screenshot coordination and GPU/VRAM limitations.

## Camera and photo verification

View → Camera & Photo opens the three native tabs. See [NAVIGATION](docs/NAVIGATION.md) for controls,
persistence and precision limits. Developer examples:

```sh
./target/release/cosmogon --new lab --focus Saturn --distance 5 --free-camera --panel camera \
  --capture /tmp/free-camera.png --capture-metadata --capture-after 240 --quit
./target/release/cosmogon --new lab --focus Saturn --distance 5 --panel photo --fov 35 --exposure 1.5 \
  --capture /tmp/photo-controls.png --capture-metadata --capture-after 240 --quit
```

`--free-camera` detaches after the initial view is established. `--fov` is vertical degrees (10–110);
`--exposure` is presentation EV (−6…6). `--panel camera|photo|bookmarks` selects a tab.
`--capture-metadata` writes `<image stem>.png.json` beside the image; a same-stem performance `.json`
remains separate. Automated `--capture` ignores keyboard input; exercise movement in an ordinary run.

## Sound and notification verification

`--panel alerts|sound|routing` opens the respective Sound & Alerts tab for native captures.
`--sound-mode realistic|cinematic|hybrid` selects a launch mode; `--quiet` mutes master sound and
suppresses interruptions for the launch. See [AUDIO_SYSTEM](docs/AUDIO_SYSTEM.md) and
[NOTIFICATION_SYSTEM](docs/NOTIFICATION_SYSTEM.md) for live mixing, routing and current limits.

The macOS packager assembles/signs the app in a local temporary folder, then creates the disk image
and keeps a launchable copy in `~/Library/Caches/Cosmogon/builds/<version>/Cosmogon.app`.
`dist/Cosmogon.app` links to that copy; `COSMOGON_APP_OUTPUT_DIR` can override its directory.
The disk image contains a standalone app. This avoids Finder metadata being re-added by
file-provider folders during or after signing. Apple documents the forbidden metadata in
[QA1940](https://developer.apple.com/library/archive/qa/qa1940/_index.html). Temporary staging is
removed on success or failure. Local builds are ad-hoc signed unless a Developer ID is supplied.

## Research recordings

See [RESEARCH](docs/RESEARCH.md) for native controls, provenance, scientific limits and bundle layout.

```sh
./target/release/cosmogon-cli experiment --scenario lab --target Earth --samples 32 \
  --interval-days 1 --output /tmp/earth-research-1
./target/release/cosmogon --new lab --focus Earth --speed 0 --quiet --panel plots \
  --research-export /tmp/native-research-1 --research-samples 60 --research-every 20 \
  --capture /tmp/research.png --capture-after 1300 --quit
```

Use new output directories. Native cadence counts app frames; CLI cadence requests simulation days.
`--panel control|models|sources|plots` opens a research tab. Models prints registry JSON without
creating a universe. No additional installation or network service is required.

Measured orbit checks: `./target/release/cosmogon-cli validate --output /tmp/orbit-check-1.json`.
`--panel validation` opens Models and starts these fixed fixtures for native captures. CLI `--help` documents cross-object sweeps and primary measurements; existing export paths are protected.

Reopen saved numeric data: `./target/release/cosmogon-cli inspect --recording /tmp/earth-research-1`.
Native QA: `--research-recording /tmp/earth-research-1 --panel plots --research-workspace /tmp/research-choices.json`; the optional choices file is distinct from the simulation save.
