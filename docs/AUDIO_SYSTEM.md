# Sound modes and mixing

Open **View → Sound & Alerts → Sound**, or search Sound in Cmd/Ctrl+K. Home Settings also provides
mix and quiet controls. Master volume combines with independent **Music / soundtrack**, **Cinematic
effects** and **Alert** levels. Zero mutes a channel. Changes update active sounds as well as new ones.

* **Realistic:** external cinematic impact/supernova effects are silent. Optional music and
  informational alert tones remain separately controlled. The Realistic preset also turns music off.
* **Cinematic:** preserves the existing designed score, impact rumbles and supernova effects.
* **Hybrid:** applies 35% of the configured cinematic effect gain, retaining the independent score
  and alerts. This is a softer presentation mix, not a physical atmosphere model.
* **Quiet preset:** soft music/effects and muted alert sounds. Do Not Disturb separately suppresses
  popups and configured pauses; it does not mute optional music.

The existing synthesized soundtrack and effects remain generated locally; no new recordings or
external assets are introduced. There is no claimed wind, interior, weather, spacecraft or acoustic
propagation model. Future environment channels will require real simulation/context inputs.

All mode/mix/notification preferences are app settings with compatibility defaults. Older master
volume settings retain their former mix: music ×0.5, effects ×1, alerts ×0.5. Save preferences reports
success or failure. Audio entities are removed when leaving the universe so one-shots cannot carry
into another session. Alert delivery is described in [NOTIFICATION_SYSTEM](NOTIFICATION_SYSTEM.md).

Developer `--sound-mode realistic|cinematic|hybrid` applies a mode for the launch; `--quiet` sets master
volume to zero and Do Not Disturb for that launch. Neither flag automatically saves preferences.
Automated screenshot capture continues to disable sound. Ordinary profiling keeps audio available.
