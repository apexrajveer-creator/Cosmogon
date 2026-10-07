//! Cosmogon desktop application.
//!
//! The app is a *view* onto `cosmogon_sim::Universe`: the simulation is authoritative and
//! engine-independent; everything in this crate reads from it and turns it into pixels.
//! See ARCHITECTURE.md and RENDERING.md.

// No console window for release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod args;
mod audio;
mod camera;
mod capture;
mod interact;
mod navigation;
mod notifications;
mod persistence;
mod performance;
mod research;
mod research_batch;
mod render;
mod sim;
mod state;
mod timeline;
mod ui;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::EguiPlugin;

fn main() -> bevy::app::AppExit {
    let started = std::time::Instant::now();
    let args = args::Args::parse();
    let mut settings = persistence::UserSettings::load();
    if let Some(fov) = args.fov { settings.presentation.fov_deg = fov; }
    if let Some(ev) = args.exposure { settings.presentation.exposure_ev = ev; }
    if let Some(mode) = args.sound_mode { settings.audio.mode = mode; }
    if args.quiet { settings.sound_volume = 0.0; settings.notifications.do_not_disturb = true; }
    settings.presentation.sanitize();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Cosmogon".into(),
                        resolution: WindowResolution::new(1600, 960),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin { filter: "wgpu=error,naga=warn,bevy_render=warn,cosmogon=info".into(), ..default() }),
        )
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin::default())
        .insert_resource(args)
        .insert_resource(settings)
        .insert_resource(performance::Performance::new(started))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            state::StatePlugin,
            sim::SimPlugin,
            render::RenderPlugin,
            camera::CameraPlugin,
            ui::UiPlugin,
            capture::CapturePlugin,
            timeline::TimelinePlugin,
            audio::SoundPlugin,
            notifications::NotificationPlugin,
            interact::InteractPlugin,
            performance::PerformancePlugin,
            research::ResearchPlugin,
            research_batch::ResearchBatchPlugin,
        ))
        .run()
}
