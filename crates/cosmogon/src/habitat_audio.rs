//! Optional interior equipment sonification. Never acoustic measurements or simulation input.
use crate::{audio::SoundChannel, persistence::UserSettings, sim::Sim, state::AppState};
use bevy::{
    audio::{AddAudioSource, AudioPlayer, Decodable, PlaybackSettings, Source, Volume},
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_egui::egui;
use cosmogon_sim::{habitat::Habitat, Universe};
use std::{
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

const SR: u32 = 44_100;
const VOICES: usize = 5;
const LABELS: [&str; VOICES] = [
    "Base services · fan texture",
    "CO₂ scrubber · air texture",
    "Water recycler · pump texture",
    "Electrolysis · equipment hum",
    "CO₂ reduction · equipment hum",
];
const MAPPING: &str = "cosmogon-habitat-equipment-sound/1";
const LIMITATIONS: &str = "Designed sonification of the last completed interval. No measured SPL, dB, motor RPM, cabin geometry, propagation, fault prediction or acoustic safety assessment. Pausing holds that interval. Powered idle processors can hum without material flow. Chemical stocks alone do not produce sound.";

#[derive(Clone, Debug, serde::Serialize)]
pub struct Equipment {
    label: &'static str,
    normalization_w: Option<f64>,
    served_w: Option<f64>,
    designed_level: f32,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Snapshot {
    mapping: &'static str,
    habitat_id: u64,
    world_time_s: f64,
    completed_habitat_time_s: f64,
    interval_s: f64,
    support: &'static str,
    equipment: [Equipment; VOICES],
    limitations: &'static str,
}
impl Snapshot {
    fn levels(&self) -> [f32; VOICES] {
        self.equipment.each_ref().map(|e| e.designed_level)
    }
}
fn fraction(served: f64, rating: f64) -> f32 {
    if !served.is_finite() || !rating.is_finite() || rating <= 0.0 {
        0.0
    } else {
        (served / rating).clamp(0.0, 1.0) as f32
    }
}
fn snapshot(h: &Habitat, world_time_s: f64) -> Snapshot {
    let b = &h.budget;
    let support = if b.seconds <= 0.0 {
        "waiting_for_completed_interval"
    } else if !b.environment_available {
        "environment_unavailable"
    } else if !b.temperature_domain_supported {
        "temperature_domain_unsupported"
    } else if b.configuration.is_none() {
        "interval_configuration_unavailable"
    } else {
        "supported"
    };
    let mut result = Snapshot {
        mapping: MAPPING,
        habitat_id: h.id,
        world_time_s,
        completed_habitat_time_s: h.last_time,
        interval_s: b.seconds,
        support,
        equipment: std::array::from_fn(|i| Equipment {
            label: LABELS[i],
            normalization_w: None,
            served_w: None,
            designed_level: 0.0,
        }),
        limitations: LIMITATIONS,
    };
    if support != "supported" {
        return result;
    }
    let c = b.configuration.as_ref().expect("supported configuration");
    // The engine serves these first, in this exact order. Use the completed interval's
    // configuration, not a newly staged/applied configuration or remaining inventory.
    let mut remaining = b.delivered_w;
    for (i, request) in [c.base_load_w, c.scrubber_power_w, c.recycler_power_w]
        .into_iter()
        .enumerate()
    {
        let supplied = remaining.min(request).max(0.0);
        remaining = (remaining - supplied).max(0.0);
        result.equipment[i].normalization_w = Some(request);
        result.equipment[i].served_w = Some(supplied);
        result.equipment[i].designed_level = fraction(supplied, request);
    }
    for (i, served, rating) in [
        (
            3,
            b.electrolysis
                .as_ref()
                .map(|e| e.delivered_w)
                .or_else(|| c.electrolysis.is_none().then_some(0.0)),
            c.electrolysis
                .as_ref()
                .map(|e| e.power_limit_w)
                .unwrap_or(0.0),
        ),
        (
            4,
            b.sabatier
                .as_ref()
                .map(|e| e.delivered_w)
                .or_else(|| c.sabatier.is_none().then_some(0.0)),
            c.sabatier.as_ref().map(|e| e.power_limit_w).unwrap_or(0.0),
        ),
    ] {
        result.equipment[i].normalization_w = Some(rating);
        // Missing budgets cannot establish zero operation for configured equipment.
        result.equipment[i].served_w = served;
        result.equipment[i].designed_level = served.map(|w| fraction(w, rating)).unwrap_or(0.0);
    }
    result
}
pub fn source(u: &Universe, id: u64) -> Result<Snapshot, String> {
    let h = u
        .habitats
        .iter()
        .find(|h| h.id == id)
        .ok_or_else(|| format!("Habitat #{id} is absent from this live world"))?;
    Ok(snapshot(h, u.time))
}

struct Levels([AtomicU32; VOICES]);
impl Levels {
    fn new(levels: [f32; VOICES]) -> Self {
        Self(levels.map(|v| AtomicU32::new(v.to_bits())))
    }
    fn set(&self, levels: [f32; VOICES]) {
        for (value, target) in self.0.iter().zip(levels) {
            value.store(target.to_bits(), Ordering::Relaxed);
        }
    }
    fn get(&self) -> [f32; VOICES] {
        self.0.each_ref().map(|v| {
            let v = f32::from_bits(v.load(Ordering::Relaxed));
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
    }
}
#[derive(Asset, TypePath, Clone)]
struct InteriorSound {
    levels: Arc<Levels>,
}
struct InteriorDecoder {
    levels: Arc<Levels>,
    target: [f32; VOICES],
    smooth: [f32; VOICES],
    phase: [f64; VOICES],
    counter: u32,
    rng: u32,
    noise: f32,
}
impl Decodable for InteriorSound {
    type DecoderItem = f32;
    type Decoder = InteriorDecoder;
    fn decoder(&self) -> InteriorDecoder {
        InteriorDecoder {
            levels: self.levels.clone(),
            target: [0.0; VOICES],
            smooth: [0.0; VOICES],
            phase: [0.0; VOICES],
            counter: 0,
            rng: 0x61e7_358b,
            noise: 0.0,
        }
    }
}
impl Iterator for InteriorDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        // Atomic targets are sampled once per short audio block; no locks or allocations
        // on the audio thread. Eighty-millisecond smoothing avoids clicks on budget changes.
        if self.counter == 0 {
            self.target = self.levels.get();
        }
        self.counter = (self.counter + 1) % 64;
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        let white = (self.rng as f32 / u32::MAX as f32) * 2.0 - 1.0;
        self.noise += 0.035 * (white - self.noise);
        let mut value = 0.0;
        for (i, frequency) in [60.0, 115.0, 85.0, 170.0, 210.0].into_iter().enumerate() {
            self.smooth[i] += (self.target[i] - self.smooth[i]) / (SR as f32 * 0.08);
            if self.target[i] == 0.0 && self.smooth[i] < 1e-6 {
                self.smooth[i] = 0.0;
            }
            // Bounded f64 phases keep an indefinite stream precise. Pitch is designed,
            // not inferred motor RPM or a calibrated fan law.
            self.phase[i] = (self.phase[i]
                + std::f64::consts::TAU * frequency * (0.65 + 0.35 * self.smooth[i] as f64)
                    / SR as f64)
                % std::f64::consts::TAU;
            let tone = (self.phase[i].sin() + 0.18 * (self.phase[i] * 2.0).sin()) as f32;
            let texture = if i <= 1 {
                0.09 * tone + 0.32 * self.noise
            } else {
                0.065 * tone + 0.12 * self.noise
            };
            value += texture * self.smooth[i];
        }
        Some(value.clamp(-0.8, 0.8))
    }
}
impl Source for InteriorDecoder {
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
        None
    }
}

#[derive(Resource, Default)]
pub struct Listener {
    pub equipment_view: bool,
    export_open: bool,
    selected: Option<u64>,
    generation: Option<u64>,
    choice: Option<u64>,
    controls: Option<Arc<Levels>>,
    status: Option<String>,
    output: String,
    task: Option<Task<Result<(), String>>>,
}
impl Listener {
    pub(crate) fn choose(&mut self, id: u64, generation: u64) {
        self.equipment_view = true;
        self.selected = Some(id);
        self.choice = Some(id);
        self.generation = Some(generation);
        self.status = None;
    }
    fn stop(&mut self) {
        self.selected = None;
        self.generation = None;
        if let Some(c) = &self.controls {
            c.set([0.0; VOICES]);
        }
    }
    fn synchronize(&mut self, sim: &Sim) {
        if self.selected.is_some() {
            if self.generation.is_some_and(|g| g != sim.generation) {
                self.stop();
                self.status = Some(
                    "Listening stopped after load, rewind or undo. Choose a habitat again.".into(),
                );
            } else if !sim
                .universe
                .habitats
                .iter()
                .any(|h| Some(h.id) == self.selected)
            {
                self.stop();
                self.status = Some(
                    "Listening stopped: the selected habitat is no longer in this world.".into(),
                );
            } else {
                self.generation = Some(sim.generation);
            }
        }
        if self
            .choice
            .is_none_or(|id| !sim.universe.habitats.iter().any(|h| h.id == id))
        {
            self.choice = sim.universe.habitats.first().map(|h| h.id);
        }
    }
    pub fn metadata(&self, u: &Universe) -> serde_json::Value {
        serde_json::json!({"listener_context":if self.selected.is_some() {"habitat_interior_preview"} else {"exterior_space"},"selected_habitat_id":self.selected,"source":self.selected.and_then(|id|source(u,id).ok()),"presentation_only":true})
    }
}
#[derive(Component)]
struct InteriorPlayer;
pub struct HabitatAudioPlugin;
impl Plugin for HabitatAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<InteriorSound>()
            .init_resource::<Listener>()
            .add_systems(Startup, init)
            .add_systems(
                Update,
                update.run_if(in_state(AppState::Observing).and(resource_exists::<Sim>)),
            )
            .add_systems(OnExit(AppState::Observing), clear);
    }
}
fn init(mut listener: ResMut<Listener>, args: Res<crate::args::Args>) {
    listener.equipment_view = args.habitat_audio.is_some()
        || matches!(
            args.panel.as_deref(),
            Some("habitat-sound" | "habitat-sound-export")
        );
    listener.export_open = args.panel.as_deref() == Some("habitat-sound-export");
    listener.selected = args.habitat_audio;
    listener.choice = args.habitat_audio;
}
fn clear(mut listener: ResMut<Listener>) {
    listener.stop();
    listener.choice = None;
    listener.controls = None;
}
#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    mut listener: ResMut<Listener>,
    sim: Res<Sim>,
    args: Res<crate::args::Args>,
    ui: Res<crate::ui::UiState>,
    settings: Res<UserSettings>,
    mut sounds: ResMut<Assets<InteriorSound>>,
    players: Query<Entity, With<InteriorPlayer>>,
) {
    if let Some(task) = listener.task.as_mut() {
        if let Some(result) = block_on(future::poll_once(task)) {
            listener.status = Some(match result {
                Ok(()) => "Saved eight-second WAV and source report.".into(),
                Err(e) => e,
            });
            listener.task = None;
        }
    }
    listener.synchronize(&sim);
    if args.capture.is_some() {
        return;
    }
    if listener.selected.is_none() {
        for e in &players {
            commands.entity(e).despawn();
        }
        listener.controls = None;
        return;
    }
    let levels = if ui.photo_mode || settings.sound_volume <= 0.0 {
        [0.0; VOICES]
    } else {
        listener
            .selected
            .and_then(|id| source(&sim.universe, id).ok())
            .map(|s| s.levels())
            .unwrap_or([0.0; VOICES])
    };
    if let Some(c) = &listener.controls {
        c.set(levels);
    } else {
        let controls = Arc::new(Levels::new(levels));
        let handle = sounds.add(InteriorSound {
            levels: controls.clone(),
        });
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.audio.gain(
                settings.sound_volume,
                SoundChannel::Environment,
                false,
            ))),
            SoundChannel::Environment,
            InteriorPlayer,
        ));
        listener.controls = Some(controls);
    }
}

pub fn panel(ui: &mut egui::Ui, listener: &mut Listener, sim: &Sim, args: &crate::args::Args) {
    listener.synchronize(sim);
    ui.strong("Habitat listening preview");
    ui.label(egui::RichText::new("Choose an interior preview explicitly. The camera stays where it is; exterior space has no equipment sound.").small().color(crate::ui::MUTED));
    egui::ComboBox::from_id_salt("audio-habitat")
        .selected_text(
            listener
                .choice
                .and_then(|id| sim.universe.habitats.iter().find(|h| h.id == id))
                .map(|h| format!("{} · #{}", h.definition.name, h.id))
                .unwrap_or_else(|| "No live habitats".into()),
        )
        .show_ui(ui, |ui| {
            for h in &sim.universe.habitats {
                ui.selectable_value(
                    &mut listener.choice,
                    Some(h.id),
                    format!("{} · #{}", h.definition.name, h.id),
                );
            }
        });
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                listener.choice.is_some(),
                egui::Button::new("Listen inside selected habitat"),
            )
            .clicked()
        {
            if let Some(id) = listener.choice {
                listener.choose(id, sim.generation);
            }
        }
        if ui
            .add_enabled(
                listener.selected.is_some(),
                egui::Button::new("Return to exterior / stop"),
            )
            .clicked()
        {
            listener.stop();
        }
    });
    if args.capture.is_some() {
        ui.label(
            egui::RichText::new(
                "Developer capture: playback disabled. Source values remain inspectable.",
            )
            .small()
            .color(crate::ui::ACCENT),
        );
    }
    if let Some(id) = listener.selected {
        if let Ok(s) = source(&sim.universe, id) {
            ui.label(format!(
                "Interior preview #{} · {} · {:.0} s interval ending {:.3} s",
                id,
                s.support.replace('_', " "),
                s.interval_s,
                s.completed_habitat_time_s
            ));
            if sim.paused {
                ui.label(
                    egui::RichText::new(
                        "Clock paused: holding the last completed equipment interval.",
                    )
                    .small()
                    .color(crate::ui::MUTED),
                );
            }
            egui::CollapsingHeader::new("Export eight-second sound preview").default_open(listener.export_open).show(ui,|ui| {
                ui.label("Fixed interval WAV and exact source state. Export ignores mixer settings; playback uses them. Choose a new folder.");
                if listener.output.is_empty() {listener.output=crate::persistence::data_dir().join("habitat-sound").display().to_string();}
                ui.text_edit_singleline(&mut listener.output);
                if ui.add_enabled(listener.task.is_none(),egui::Button::new("Save WAV and source report")).clicked() {
                    let world_time=sim.universe.time;
                    let h=sim.universe.habitats.iter().find(|h|h.id==id).expect("validated selected habitat").clone();
                    let path=std::path::PathBuf::from(&listener.output);
                    listener.task=Some(AsyncComputeTaskPool::get().spawn(async move {export_snapshot(&h,world_time,&path)}));listener.status=Some("Saving sound preview…".into());
                }
            });
            egui::Grid::new("audio-equipment")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Designed source");
                    ui.label("Served power");
                    ui.label("Source level");
                    ui.end_row();
                    for e in &s.equipment {
                        ui.label(e.label);
                        ui.label(
                            e.served_w
                                .map(|w| format!("{w:.2} W"))
                                .unwrap_or_else(|| "Unavailable".into()),
                        );
                        ui.label(format!("{:.1}%", 100.0 * e.designed_level));
                        ui.end_row();
                    }
                });
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Designed power fractions; no measured decibel inference.")
                        .small()
                        .color(crate::ui::MUTED),
                );
                if ui.small_button("Copy source report").clicked() {
                    ui.ctx()
                        .copy_text(serde_json::to_string_pretty(&s).expect("finite source report"));
                }
            });
        }
    }
    if let Some(message) = &listener.status {
        ui.label(
            egui::RichText::new(message)
                .small()
                .color(crate::ui::ACCENT),
        );
    }
}

pub fn export(u: &Universe, id: u64, path: &Path) -> Result<(), String> {
    let h = u
        .habitats
        .iter()
        .find(|h| h.id == id)
        .ok_or_else(|| format!("Habitat #{id} is absent from this world"))?;
    export_snapshot(h, u.time, path)
}
fn export_snapshot(h: &Habitat, world_time: f64, path: &Path) -> Result<(), String> {
    h.config.validate()?;
    if let Some(c) = &h.budget.configuration {
        c.validate()?;
    }
    if !world_time.is_finite()
        || !h.last_time.is_finite()
        || h.last_time > world_time
        || !h.budget.seconds.is_finite()
        || h.budget.seconds < 0.0
        || !h.budget.delivered_w.is_finite()
        || h.budget.delivered_w < 0.0
    {
        return Err("Invalid source clock or electrical interval".into());
    }
    let s = snapshot(h, world_time);
    std::fs::create_dir(path).map_err(|e| {
        format!("Choose a new sound preview folder; existing results are protected: {e}")
    })?;
    let result = (|| -> std::io::Result<()> {
        let count = SR * 8;
        let mut file = std::io::BufWriter::new(std::fs::File::create(path.join("preview.wav"))?);
        file.write_all(b"RIFF")?;
        file.write_all(&(36 + count * 2).to_le_bytes())?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&SR.to_le_bytes())?;
        file.write_all(&(SR * 2).to_le_bytes())?;
        file.write_all(&2u16.to_le_bytes())?;
        file.write_all(&16u16.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&(count * 2).to_le_bytes())?;
        let sound = InteriorSound {
            levels: Arc::new(Levels::new(s.levels())),
        };
        for value in sound.decoder().take(count as usize) {
            file.write_all(&((value * i16::MAX as f32).round() as i16).to_le_bytes())?;
        }
        file.flush()?;
        // The source report marks a completed export; partial I/O failures cannot be
        // mistaken for a complete report. No user mixer or simulation state is modified.
        let report = serde_json::json!({"format":"cosmogon-habitat-sound-preview","schema_version":1,"complete":true,"source":s,"habitat":h,
            "audio":{"file":"preview.wav","sample_rate_hz":SR,"channels":1,"pcm_bits":16,"duration_s":8,"mixer_applied":false,"fade_in_time_constant_s":0.08},
            "references":["https://www.nasa.gov/directorates/esdmd/hhp/acoustics-and-noise-control/","https://www.nasa.gov/general/nasa-quiet-space-fan-research-to-benefit-commercial-space-stations/"]});
        std::fs::write(
            path.join("source.json"),
            serde_json::to_vec_pretty(&report)?,
        )
    })();
    result.map_err(|e| format!("Sound preview incomplete: {e}; choose another new folder to retry"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn habitat() -> Habitat {
        let mut h = Habitat::new(
            7,
            cosmogon_sim::habitat::Definition::at(
                cosmogon_sim::BodyRef { system: 0, body: 0 },
                0.0,
                0.0,
            ),
            0.0,
        )
        .unwrap();
        h.budget.configuration = Some(h.config.clone());
        h.budget.seconds = 60.0;
        h.budget.environment_available = true;
        h.budget.temperature_domain_supported = true;
        h
    }
    #[test]
    fn actual_engine_intervals_supply_the_sound_and_constrained_reactions_stop() {
        for power in [0.0, 650.0, 5000.0] {
            let mut u = Universe::new(cosmogon_sim::UniverseSettings {
                scenario: cosmogon_sim::Scenario::SolarSystemLab,
                ..Default::default()
            });
            let body = cosmogon_sim::BodyRef {
                system: 0,
                body: u.systems[0].find_body("Mars").unwrap() as u32,
            };
            let mut d = cosmogon_sim::habitat::Definition::at(body, 0.0, 0.0);
            d.config.firm_power_w = power;
            d.config.solar_area_m2 = 0.0;
            d.initial.battery_j = 0.0;
            d.config.electrolysis = Some(Default::default());
            d.config.sabatier = Some(Default::default());
            u.apply_edit(cosmogon_sim::sandbox::Edit::CreateHabitat {
                definition: Box::new(d),
            })
            .unwrap();
            let id = u.habitats[0].id;
            u.advance_by(120.0);
            let before = u.clone();
            let s = source(&u, id).unwrap();
            assert_eq!(u, before);
            assert_eq!(s.support, "supported");
            assert_eq!(s.equipment[0].served_w, Some(power.min(500.0)));
            assert_eq!(
                s.equipment[1].served_w,
                Some((power - 500.0).clamp(0.0, 300.0))
            );
            assert_eq!(
                s.equipment[3].served_w,
                Some(
                    u.habitats[0]
                        .budget
                        .electrolysis
                        .as_ref()
                        .unwrap()
                        .delivered_w
                )
            );
            assert_eq!(
                s.equipment[4].served_w,
                Some(u.habitats[0].budget.sabatier.as_ref().unwrap().delivered_w)
            );
            if power == 0.0 {
                assert_eq!(s.levels(), [0.0; VOICES]);
            }
            if power == 5000.0 {
                assert!(s.equipment[3].designed_level > 0.0);
                assert!(s.equipment[4].designed_level > 0.0);
                // Full product stores stop production even when the bus remains powered.
                let h = &u.habitats[0];
                let mut c = h.config.clone();
                c.crew = 0;
                c.electrolysis.as_mut().unwrap().oxygen_capacity_kg = h.state.oxygen_reserve_kg;
                c.sabatier.as_mut().unwrap().methane_capacity_kg = h.state.methane_kg;
                u.apply_edit(cosmogon_sim::sandbox::Edit::ConfigureHabitat {
                    system: 0,
                    id,
                    config: Box::new(c),
                })
                .unwrap();
                u.advance_by(60.0);
                let constrained = source(&u, id).unwrap();
                assert_eq!(constrained.equipment[3].served_w, Some(0.0));
                assert_eq!(constrained.equipment[4].served_w, Some(0.0));
            }
        }
    }
    #[test]
    fn sources_follow_priority_power_and_completed_configuration_without_inventory_shortcuts() {
        let mut h = habitat();
        h.budget.delivered_w = 650.0;
        h.config.base_load_w = 0.0; // A newer configuration must not change this interval.
        let s = snapshot(&h, 70.0);
        assert_eq!(s.equipment[0].served_w, Some(500.0));
        assert_eq!(s.equipment[1].served_w, Some(150.0));
        assert_eq!(s.equipment[2].served_w, Some(0.0));
        assert_eq!(s.levels(), [1.0, 0.5, 0.0, 0.0, 0.0]);
        h.state.hydrogen_kg = 100.0;
        h.state.methane_kg = 100.0;
        h.budget.delivered_w = 0.0;
        assert_eq!(snapshot(&h, 70.0).levels(), [0.0; VOICES]);
        h.budget.delivered_w = 1100.0;
        assert_eq!(snapshot(&h, 70.0).levels(), [1.0, 1.0, 1.0, 0.0, 0.0]);
        h.budget.configuration.as_mut().unwrap().electrolysis = Some(Default::default());
        let missing = snapshot(&h, 70.0);
        assert_eq!(missing.equipment[3].served_w, None);
        assert_eq!(missing.equipment[3].designed_level, 0.0);
        h.budget.configuration = None;
        assert_eq!(
            snapshot(&h, 70.0).support,
            "interval_configuration_unavailable"
        );
        assert_eq!(snapshot(&h, 70.0).levels(), [0.0; VOICES]);
        h.budget.configuration = Some(h.config.clone());
        h.budget.environment_available = false;
        assert_eq!(snapshot(&h, 70.0).levels(), [0.0; VOICES]);
        h.budget.environment_available = true;
        h.budget.temperature_domain_supported = false;
        assert_eq!(snapshot(&h, 70.0).levels(), [0.0; VOICES]);
    }
    #[test]
    fn decoder_is_finite_bounded_repeatable_and_smoothly_stops() {
        let levels = Arc::new(Levels::new([1.0; VOICES]));
        let sound = InteriorSound {
            levels: levels.clone(),
        };
        let a: Vec<_> = sound.decoder().take(SR as usize).collect();
        let b: Vec<_> = sound.decoder().take(SR as usize).collect();
        assert_eq!(a, b);
        assert!(a.iter().all(|v| v.is_finite() && v.abs() <= 0.8));
        assert!(a.iter().any(|v| v.abs() > 0.1));
        assert!(a[0].abs() < 0.001);
        let mut d = sound.decoder();
        for _ in 0..SR {
            d.next();
        }
        let previous = d.next().unwrap();
        levels.set([0.0; VOICES]);
        let next = d.next().unwrap();
        assert!((next - previous).abs() < 0.05);
        for _ in 0..SR * 2 {
            d.next();
        }
        assert_eq!(d.next(), Some(0.0));
        assert!(d
            .phase
            .iter()
            .all(|p| *p >= 0.0 && *p < std::f64::consts::TAU));
        levels.set([f32::NAN, f32::INFINITY, -1.0, 0.0, 0.0]);
        assert_eq!(levels.get(), [0.0; VOICES]);
    }
    #[test]
    fn listener_clears_on_undo_removal_and_leaving_but_never_edits_world() {
        let mut u = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        u.habitats.push(habitat());
        let mut sim = Sim::new(
            u,
            crate::sim::Session::Reference {
                dataset: "test".into(),
                title: "test".into(),
            },
        );
        let before = sim.universe.clone();
        let mut l = Listener::default();
        l.choose(7, sim.generation);
        l.synchronize(&sim);
        assert_eq!(l.selected, Some(7));
        assert_eq!(sim.universe, before);
        sim.generation += 1;
        l.synchronize(&sim);
        assert_eq!(l.selected, None);
        l.choose(7, sim.generation);
        sim.universe.habitats.clear();
        l.synchronize(&sim);
        assert_eq!(l.selected, None);
        l.choose(7, sim.generation);
        l.stop();
        assert_eq!(l.selected, None);
    }
    #[test]
    fn protected_wav_export_is_repeatable_and_unpowered_preview_is_exactly_silent() {
        let root = std::env::temp_dir().join(format!("cosmogon-audio-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let mut h = habitat(); // Zero delivered power is a real, supported silent interval.
        export_snapshot(&h, 60.0, &root.join("silent")).unwrap();
        let bytes = std::fs::read(root.join("silent/preview.wav")).unwrap();
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(bytes.len(), 44 + SR as usize * 8 * 2);
        assert!(bytes[44..].iter().all(|b| *b == 0));
        assert!(export_snapshot(&h, 60.0, &root.join("silent")).is_err());
        assert_eq!(
            bytes,
            std::fs::read(root.join("silent/preview.wav")).unwrap()
        );
        h.budget.delivered_w = 1100.0;
        h.budget.requested_w = 1100.0;
        h.budget.unserved_w = 0.0;
        for name in ["a", "b"] {
            export_snapshot(&h, 60.0, &root.join(name)).unwrap();
        }
        assert_eq!(
            std::fs::read(root.join("a/preview.wav")).unwrap(),
            std::fs::read(root.join("b/preview.wav")).unwrap()
        );
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("a/source.json")).unwrap()).unwrap();
        assert_eq!(report["habitat"], serde_json::to_value(&h).unwrap());
        assert_eq!(report["source"]["equipment"][0]["served_w"], 500.0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
