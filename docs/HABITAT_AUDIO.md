# Habitat equipment listening preview

Open **View → Sound & Alerts → Sound → Habitat equipment**, choose a live habitat, and press **Listen inside selected habitat**. Habitat engineering's Budgets tab also opens this preview with **Listen to equipment**. **Return to exterior / stop** removes the equipment sound. The listening context is explicit: it does not enter a rendered cabin, move the camera, or send sound through exterior vacuum. The default is exterior space with no habitat equipment audio.

Five locally synthesized textures follow the actual last completed habitat interval:

| Designed texture | Actual control | Normalization |
|---|---|---|
| Base services fan | Served base-load watts | Interval base-load rating |
| Air processor | Served scrubber watts | Interval scrubber rating |
| Water pump | Served recycler watts | Interval recycler rating |
| Electrolysis equipment | Reactor delivered watts | Interval electrolysis power limit |
| CO₂ reduction equipment | Reactor delivered watts | Interval Sabatier power limit |

The first three allocations are reconstructed from the engine's explicit priority order and total delivered power. The configuration retained with that completed interval controls the mapping; current or staged edits cannot rewrite its history. Reactor watts come directly from their actual budgets. Levels range from zero to one. These normalize electricity into designed sound controls, not sound pressure, measured decibels, motor speed or a physical fan law.

Powered idle scrubbers/recyclers can hum even without material processing because the existing engine supplies their configured electrical loads. Reactors request no power when inputs or product headroom are exhausted, so their sounds stop. Chemical stocks alone do not trigger sound. Lack of a completed interval, exterior environment, supported temperature domain or retained configuration produces silence with an explicit reason; unavailable watts remain unavailable in the source report. Configured reactors lacking their interval budget also stay silent with unavailable watts; an absent budget is not a measured zero.

Pausing holds the last completed interval. Source time and duration remain visible. Loading, undoing or rewinding clears the listener; deletion and leaving the universe also stop it. Historical recordings and parameter sweeps do not automatically become live audio sources. Listening is presentation only and works in reference sessions if actual habitats exist.

**Habitat equipment** has its own saved mixer level, default 0.5 for old settings. Master mute, quiet launch, photo mode and developer captures suppress playback. Realistic mode allows the selected interior preview while external cinematic effects stay silent. Alert routing and Do Not Disturb remain independent. The Quiet preset mutes equipment. Listener choice is transient and does not restart on the next launch.

An indefinite mono native stream reads bounded atomic targets every 64 samples and smooths level changes with an 80 ms time constant. Oscillator phases stay bounded in f64; presentation noise does not consume simulation RNG. One stream avoids restarting assets each frame. No recordings, dependencies or simulation fields are added.

## Inspectable sound export

Expand **Export eight-second sound preview**, enter a new folder and choose **Save WAV and source report**. A worker captures the exact selected habitat and world time at click, then exports:

- `preview.wav`: eight seconds, 44,100 Hz, mono signed 16-bit PCM, using the same decoder and mapping as live playback, with an initial fade.
- `source.json`: complete marker, mapping version, actual world/completed times, each source's watts/normalization/level, exact habitat definition/configuration/state/budget and presentation limitations.

Export bypasses mixer settings, as recorded in its metadata. An unpowered or unsupported preview exports silence; unsupported watts remain null. Existing folders are protected. On I/O failure the new incomplete folder may remain without a completed report; choose a new folder to retry. This is a fixed interval presentation artifact, not trajectory replay, acoustic observation or uncertainty estimate. The habitat state can be compared with its research export; this two-file artifact does not claim whole-world replay.

Developer tooling exports the same data without initializing a renderer:

```sh
./target/release/cosmogon --load /tmp/habitat-run/final.cosmo \
  --habitat-audio 0 --habitat-audio-export /tmp/new-habitat-sound
./target/release/cosmogon --load /tmp/habitat-run/final.cosmo \
  --habitat-audio 0 --panel sound --sound-mode realistic --quiet \
  --capture /tmp/habitat-sound.png --capture-metadata --capture-after 240 --quit
```

`--habitat-audio` is the exact live identifier in the loaded save. `--habitat-audio-export` chooses a new folder and exits after writing. Renderer/simulation launch requests cannot be mixed into this fixed saved-state export. Photo metadata includes actual listener/source context and records playback suppression; screenshots do not verify audible playback.

NASA's [Acoustics and Noise Control](https://www.nasa.gov/directorates/esdmd/hhp/acoustics-and-noise-control/) and [Quiet Space Fan research](https://www.nasa.gov/general/nasa-quiet-space-fan-research-to-benefit-commercial-space-stations/) establish the relevance of mechanical sources and the need for hardware measurements. They do not calibrate these textures. See the [pre-implementation memo](HABITAT_AUDIO_DESIGN.md). Cabin geometry/materials, measured spectra, propagation, spatial listening, wind/weather, footsteps/airlocks, equipment faults and the full audio environment engine remain incomplete.
