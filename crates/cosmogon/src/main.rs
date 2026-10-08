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
mod surface;
mod notifications;
mod persistence;
mod performance;
mod research;
mod research_workspace;
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
    // Reject invalid automated inputs before starting the renderer. Early GPU teardown
    // can otherwise wait for a render task that still needs the main-thread executor.
    // Interactive opening remains asynchronous and reports errors in the workspace.
    if args.quit_after_capture {
        if let Some(path)=&args.greenhouse_definition {if let Err(e)=cosmogon_sim::greenhouse::Definition::read(std::path::Path::new(path)).and_then(|d|d.calculate()) {eprintln!("Cannot open greenhouse definition: {e}");return bevy::app::AppExit::error();}}
        if let Some(path)=&args.load {if let Err(e)=cosmogon_sim::save::read_file(std::path::Path::new(path)) {eprintln!("Cannot load automated world: {e}");return bevy::app::AppExit::error();}}
        if let Some(path)=&args.habitat_definition {if let Err(e)=cosmogon_sim::habitat::Definition::read(std::path::Path::new(path)) {eprintln!("Cannot open habitat definition: {e}");return bevy::app::AppExit::error();}}
        if let Some(path)=&args.terrain_section {if let Err(error)=cosmogon_sim::planet::section::Definition::read(std::path::Path::new(path)).and_then(|d|d.calculate()) {eprintln!("Cannot open terrain section: {error}");return bevy::app::AppExit::error();}}
        if let Some(path) = &args.research_recording {
            if let Err(error) = cosmogon_sim::research::SavedRecording::read(std::path::Path::new(path)) {
                eprintln!("Cannot open research recording: {error}");
                return bevy::app::AppExit::error();
            }
        }
    }
    let mut settings = persistence::UserSettings::load();
    if let Some(fov) = args.fov { settings.presentation.fov_deg = fov; }
    if let Some(ev) = args.exposure { settings.presentation.exposure_ev = ev; }
    if let Some(mode) = args.sound_mode { settings.audio.mode = mode; }
    if args.quiet { settings.sound_volume = 0.0; settings.notifications.do_not_disturb = true; }
    settings.presentation.sanitize();

    let plugins = DefaultPlugins.build();
    // The pipelined renderer's shutdown waits without servicing its main-thread
    // executor. Imported-table automation may finish during GPU initialization;
    // render those short jobs on the main thread so teardown cannot deadlock.
    let plugins = if (args.greenhouse_definition.is_some() || args.habitat_definition.is_some() || args.research_recording.is_some() || args.surface_site.is_some() || args.mars_terrain.is_some() || args.terrain_section.is_some()) && args.quit_after_capture {
        plugins.disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
    } else {
        plugins
    };

    App::new()
        .add_plugins(
            plugins
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
