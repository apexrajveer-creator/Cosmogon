//! Sound, synthesised on the fly (no recordings, nothing to license).
//!
//! * An ambient drone: detuned low pads that swell and fade on slow, unrelated cycles over
//!   a filtered "solar wind" hiss — never quite repeating.
//! * Impacts: a deep thump and a rumble of brown noise, louder for bigger blasts.
//! * Supernovae: a long, rising roar.
//! * Milestones: a soft bell (at most one every few seconds).
//!
//! Space is silent, of course: this is a soundtrack, not a simulation.

use bevy::audio::{AddAudioSource, AudioPlayer, AudioSink, AudioSinkPlayback, Decodable, PlaybackSettings, Source, Volume};
use bevy::prelude::*;
use std::time::Duration;

use crate::persistence::UserSettings;
use crate::sim::Sim;
use crate::state::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SoundMode { Realistic, Cinematic, Hybrid }
impl SoundMode {
    pub const ALL: [Self; 3] = [Self::Realistic, Self::Cinematic, Self::Hybrid];
    pub fn label(self) -> &'static str { match self { Self::Realistic => "Realistic", Self::Cinematic => "Cinematic", Self::Hybrid => "Hybrid" } }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SoundMix { pub mode: SoundMode, pub music: f32, pub effects: f32, pub alerts: f32, pub environment: f32 }
impl Default for SoundMix { fn default() -> Self { Self { mode: SoundMode::Cinematic, music: 0.5, effects: 1.0, alerts: 0.5, environment: 0.5 } } }
impl SoundMix {
    pub fn sanitize(&mut self) { for v in [&mut self.music, &mut self.effects, &mut self.alerts, &mut self.environment] { *v = if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }; } }
    pub(crate) fn gain(&self, master: f32, channel: SoundChannel, quiet_alerts: bool) -> f32 {
        let master = if master.is_finite() { master.clamp(0.0, 1.0) } else { 0.0 };
        let gain = match channel {
            SoundChannel::Music => self.music,
            SoundChannel::Cinematic => self.effects * match self.mode { SoundMode::Realistic => 0.0, SoundMode::Cinematic => 1.0, SoundMode::Hybrid => 0.35 },
            SoundChannel::Alert => if quiet_alerts { 0.0 } else { self.alerts },
            SoundChannel::Environment => self.environment,
        };
        master * if gain.is_finite() { gain.clamp(0.0, 1.0) } else { 0.0 }
    }
}
#[derive(Component, Clone, Copy, Debug)]
pub(crate) enum SoundChannel { Music, Cinematic, Alert, Environment }
fn quiet_alerts(settings: &UserSettings, photo: bool) -> bool { settings.notifications.do_not_disturb || settings.notifications.mute_sounds || photo }

const SR: u32 = 44_100;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SynthKind {
    Ambient,
    Rumble,
    Roar,
    Bell,
}

#[derive(Asset, TypePath, Clone, Copy, Debug)]
pub struct Synth {
    pub kind: SynthKind,
    /// 0..1: size of the event.
    pub intensity: f32,
    pub seed: u32,
}

pub struct SynthDecoder {
    s: Synth,
    n: u64,
    rng: u32,
    brown: f32,
    lp: f32,
    lp2: f32,
}

impl SynthDecoder {
    fn white(&mut self) -> f32 {
        // xorshift32
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    fn duration_s(&self) -> Option<f32> {
        match self.s.kind {
            SynthKind::Ambient => None,
            SynthKind::Rumble => Some(5.5),
            SynthKind::Roar => Some(10.0),
            SynthKind::Bell => Some(3.0),
        }
    }
}

const TAU: f32 = std::f32::consts::TAU;

impl Iterator for SynthDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let t = self.n as f32 / SR as f32;
        if self.duration_s().is_some_and(|d| t >= d) {
            return None;
        }
        self.n += 1;
        let w = self.white();
        self.brown = (self.brown + 0.02 * w) / 1.02;
        let i = self.s.intensity.clamp(0.0, 1.0);
        let v = match self.s.kind {
            SynthKind::Ambient => {
                // Five voices of a suspended chord, each swelling on its own slow cycle.
                let voices = [(55.0, 23.0), (82.41, 31.0), (110.0, 41.0), (164.81, 53.0), (220.4, 67.0)];
                let mut pad = 0.0;
                for (k, (f, period)) in voices.iter().enumerate() {
                    let swell = 0.5 + 0.5 * (TAU * t / period + k as f32 * 1.7).sin();
                    let detune = 1.0 + 0.0015 * (TAU * t / 11.0 + k as f32).sin();
                    pad += swell * swell * ((TAU * f * detune * t).sin() + 0.3 * (TAU * f * 2.003 * t).sin());
                }
                // Wind: white noise through a slowly sweeping one-pole low-pass.
                let cut = 0.004 + 0.012 * (0.5 + 0.5 * (TAU * t / 37.0).sin());
                self.lp += cut * (w - self.lp);
                let wind = self.lp * (0.6 + 0.4 * (TAU * t / 19.0).sin());
                0.035 * pad + 0.9 * wind
            }
            SynthKind::Rumble => {
                let env = (-t * 0.9).exp() * (1.0 - (-t * 60.0).exp());
                let thump = (TAU * (42.0 - 14.0 * t.min(1.0)) * t).sin() * (-t * 3.0).exp();
                self.lp += 0.03 * (self.brown - self.lp);
                env * (0.7 * thump + 5.0 * self.lp) * (0.35 + 0.65 * i)
            }
            SynthKind::Roar => {
                let attack = (t / 1.8).min(1.0);
                let env = attack * attack * (-(t - 1.8).max(0.0) * 0.38).exp();
                self.lp += 0.05 * (self.brown - self.lp);
                self.lp2 += 0.2 * (w - self.lp2);
                let growl = (TAU * 31.0 * t + 3.0 * (TAU * 0.7 * t).sin()).sin();
                env * (6.0 * self.lp + 0.12 * self.lp2 + 0.35 * growl)
            }
            SynthKind::Bell => {
                // Inharmonic partials (a small bell), each decaying at its own rate.
                let f0 = [523.25, 587.33, 659.25, 783.99][(self.s.seed % 4) as usize];
                let mut b = 0.0;
                for (ratio, decay, amp) in [(1.0, 1.4, 1.0), (2.76, 2.6, 0.45), (5.40, 4.0, 0.25), (8.93, 6.0, 0.12)] {
                    b += amp * (TAU * f0 * ratio * t).sin() * (-t * decay).exp();
                }
                0.12 * b * (1.0 - (-t * 200.0).exp())
            }
        };
        Some((v * 1.2).tanh() * 0.8)
    }
}

impl Source for SynthDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SR
    }
    fn total_duration(&self) -> Option<Duration> {
        self.duration_s().map(Duration::from_secs_f32)
    }
}

impl Decodable for Synth {
    type DecoderItem = f32;
    type Decoder = SynthDecoder;
    fn decoder(&self) -> SynthDecoder {
        SynthDecoder { s: *self, n: 0, rng: 0x9E37_79B9 ^ self.seed.wrapping_mul(2_654_435_761), brown: 0.0, lp: 0.0, lp2: 0.0 }
    }
}

/// A one-shot sound requested by another system.
#[derive(Message, Clone, Copy)]
pub struct SoundCue {
    pub kind: SynthKind,
    pub intensity: f32,
}

#[derive(Component)]
struct AmbientSound;

#[derive(Resource, Default)]
struct SoundWatch {
    blasts: usize,
    generation: Option<u64>,
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Synth>()
            .add_plugins(crate::habitat_audio::HabitatAudioPlugin)
            .add_message::<SoundCue>()
            .init_resource::<SoundWatch>()
            .add_systems(OnEnter(AppState::Observing), start_ambient)
            .add_systems(OnExit(AppState::Observing), stop_ambient)
            .add_systems(Update, (watch_events, play_cues, mix_volume).chain().run_if(in_state(AppState::Observing).and(resource_exists::<Sim>)));
    }
}

fn start_ambient(mut commands: Commands, mut synths: ResMut<Assets<Synth>>, settings: Res<UserSettings>, args: Res<crate::args::Args>, mut watch: ResMut<SoundWatch>) {
    // Different universes may both start at generation zero; imported blasts are silent.
    *watch = SoundWatch::default();
    if args.capture.is_some() {
        return;
    }
    let h = synths.add(Synth { kind: SynthKind::Ambient, intensity: 1.0, seed: 1 });
    commands.spawn((AudioPlayer(h), PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.audio.gain(settings.sound_volume, SoundChannel::Music, false))), AmbientSound, SoundChannel::Music));
}

fn stop_ambient(mut commands: Commands, q: Query<Entity, With<SoundChannel>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn mix_volume(settings: Res<UserSettings>, ui: Res<crate::ui::UiState>, mut q: Query<(&mut AudioSink, &SoundChannel)>) {
    // Existing one-shots must respond immediately to mute/mode changes too.
    for (mut sink, &channel) in &mut q {
        let gain=if matches!(channel,SoundChannel::Environment) && ui.photo_mode {0.0} else {settings.audio.gain(settings.sound_volume, channel, quiet_alerts(&settings, ui.photo_mode))};
        sink.set_volume(Volume::Linear(gain));
    }
}

/// Supernovae are cinematic presentation cues; milestone alerts are routed by notifications.
fn watch_events(sim: Res<Sim>, mut w: ResMut<SoundWatch>, mut cues: MessageWriter<SoundCue>) {
    let u = &sim.universe;
    // Fresh universe, rewind or undo: start counting from here, silently.
    if w.generation != Some(sim.generation) {
        w.generation = Some(sim.generation);
        w.blasts = u.blasts.len();
        return;
    }
    if u.blasts.len() > w.blasts {
        cues.write(SoundCue { kind: SynthKind::Roar, intensity: 1.0 });
    }
    w.blasts = u.blasts.len();

}

fn play_cues(mut commands: Commands, mut cues: MessageReader<SoundCue>, mut synths: ResMut<Assets<Synth>>, settings: Res<UserSettings>, args: Res<crate::args::Args>, ui: Res<crate::ui::UiState>, mut seed: Local<u32>) {
    if args.capture.is_some() || settings.sound_volume <= 0.0 {
        cues.clear();
        return;
    }
    for c in cues.read() {
        *seed = seed.wrapping_add(1);
        let channel = if c.kind == SynthKind::Bell { SoundChannel::Alert } else { SoundChannel::Cinematic };
        let gain = settings.audio.gain(settings.sound_volume, channel, quiet_alerts(&settings, ui.photo_mode));
        if gain <= 0.0 { continue; }
        let h = synths.add(Synth { kind: c.kind, intensity: c.intensity, seed: *seed });
        commands.spawn((AudioPlayer(h), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)), channel));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(kind: SynthKind) -> Vec<f32> {
        Synth { kind, intensity: 1.0, seed: 3 }.decoder().take(SR as usize * 12).collect()
    }

    #[test]
    fn mixer_keeps_realistic_space_silent_and_alerts_independent() {
        let mut mix = SoundMix::default();
        assert_eq!(mix.gain(0.6, SoundChannel::Music, false), 0.3);
        assert_eq!(mix.gain(0.6, SoundChannel::Alert, true), 0.0);
        assert_eq!(mix.gain(0.0, SoundChannel::Cinematic, false), 0.0);
        mix.mode = SoundMode::Realistic;
        assert_eq!(mix.gain(1.0, SoundChannel::Cinematic, false), 0.0);
        assert_eq!(mix.gain(1.0, SoundChannel::Environment, true), 0.5);
        assert_eq!(mix.gain(0.0, SoundChannel::Environment, false), 0.0);
        assert_eq!(mix.gain(1.0, SoundChannel::Alert, false), 0.5);
        mix.mode = SoundMode::Hybrid;
        assert_eq!(mix.gain(1.0, SoundChannel::Cinematic, false), 0.35);
        mix.effects = f32::NAN; mix.music = f32::INFINITY; mix.alerts = -10.0; mix.environment=f32::NAN; mix.sanitize();
        for c in [SoundChannel::Music, SoundChannel::Cinematic, SoundChannel::Alert, SoundChannel::Environment] { assert_eq!(mix.gain(1.0, c, false), 0.0); }
    }
    #[test]
    fn old_preferences_preserve_the_mix_and_new_preferences_round_trip() {
        let mut settings: UserSettings = toml::from_str("sound_volume = 0.4\n").unwrap();
        assert_eq!(settings.audio.mode, SoundMode::Cinematic);
        assert_eq!(settings.audio.environment,0.5);
        assert_eq!(settings.audio.gain(settings.sound_volume, SoundChannel::Music, false), 0.2);
        settings.audio.mode = SoundMode::Realistic; settings.audio.music = 0.0;
        settings.notifications.do_not_disturb = true;
        settings.notifications.rules[4].pause = true;
        let restored: UserSettings = toml::from_str(&toml::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.audio.mode, SoundMode::Realistic); assert_eq!(restored.audio.music, 0.0);
        assert_eq!(restored.audio.environment,0.5);
        assert!(restored.notifications.do_not_disturb); assert!(restored.notifications.rules[4].pause);
    }

    #[test]
    fn one_shots_end_and_nothing_clips() {
        for k in [SynthKind::Rumble, SynthKind::Roar, SynthKind::Bell] {
            let s = render(k);
            assert!(s.len() < SR as usize * 11, "{k:?} must end");
            let peak = s.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            assert!(peak > 0.05 && peak <= 0.8, "{k:?} peak {peak}");
        }
        let amb = render(SynthKind::Ambient);
        assert_eq!(amb.len(), SR as usize * 12, "ambient never ends");
        assert!(amb.iter().all(|x| x.is_finite() && x.abs() <= 0.8));
    }
}
