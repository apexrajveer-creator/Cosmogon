//! Cosmogon desktop application.
//!
//! The app is a *view* onto `cosmogon_sim::Universe`: the simulation is authoritative and
//! engine-independent; everything in this crate reads from it and turns it into pixels.
//! See ARCHITECTURE.md and RENDERING.md.

// No console window for release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod args;
mod audio;
mod habitat_audio;
mod camera;
mod capture;
mod interact;
mod navigation;
mod cinematic;
mod input;
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
    if let Some(path)=&args.input_bindings {
        if let Err(e)=input::Definition::read(std::path::Path::new(path)){eprintln!("Cannot open keyboard controls: {e}");return AppExit::error();}
    }
    if let Some(path)=&args.camera_path {
        let checked=(||->Result<(),String>{
            let d=cinematic::Definition::read(std::path::Path::new(path))?;
            let requested=args.camera_path_time.unwrap_or(0.0);
            if requested>d.duration(){return Err("Requested presentation time exceeds the path duration".into());}
            let (u,template)=if let Some(save)=&args.load {(cosmogon_sim::save::read_file(std::path::Path::new(save)).map_err(|e|e.to_string())?,"imported")}else{
                let (_,life,intel,_)=cosmogon_sim::universe::LIFE_PRESETS[args.life.unwrap_or(0).min(2)];
                (cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings{seed:args.seed.unwrap_or(1),scenario:args.new.unwrap_or(cosmogon_sim::Scenario::SolarSystemLab),life_rate:life,intelligence_rate:intel,..Default::default()}),"dev")
            };
            let m=persistence::Manifest::new("Camera path preflight","",template,persistence::Origin::default());
            let sim=sim::Sim::new(u,sim::Session::Sandbox(m));
            d.pose(&sim,0.0)?;d.pose(&sim,requested)?;Ok(())
        })();
        if let Err(e)=checked {eprintln!("Cannot open camera path: {e}");return AppExit::error();}
    }
    if let Some(output)=&args.habitat_audio_export {
        let result=cosmogon_sim::save::read_file(std::path::Path::new(args.load.as_deref().expect("validated audio load")))
            .map_err(|e|e.to_string()).and_then(|u|habitat_audio::export(&u,args.habitat_audio.expect("validated audio id"),std::path::Path::new(output)));
        return match result {Ok(())=>{println!("Saved habitat sound preview to {output}");AppExit::Success},Err(e)=>{eprintln!("Cannot export habitat sound: {e}");AppExit::error()}};
    }
    // Reject invalid automated inputs before starting the renderer. Early GPU teardown
    // can otherwise wait for a render task that still needs the main-thread executor.
    // Interactive opening remains asynchronous and reports errors in the workspace.
    if args.quit_after_capture {
        if let Some(id)=args.habitat_audio {
            let result=cosmogon_sim::save::read_file(std::path::Path::new(args.load.as_deref().expect("validated audio load"))).map_err(|e|e.to_string())
                .and_then(|u|habitat_audio::source(&u,id).map(|_|()));
            if let Err(e)=result {eprintln!("Cannot preview habitat sound: {e}");return AppExit::error();}
        }
        if let Some(path)=&args.habitat_recording {
            match cosmogon_sim::habitat_recording::Recording::read(std::path::Path::new(path)) {
                Ok(r) if args.habitat_sample.is_none_or(|n|n<r.rows.len())=>{},
                Ok(_)=>{eprintln!("Requested habitat observation is outside the recording");return bevy::app::AppExit::error();},
                Err(e)=>{eprintln!("Cannot open habitat recording: {e}");return bevy::app::AppExit::error();},
            }
        }
        if let Some(path)=&args.habitat_compare {
            if let Err(e)=cosmogon_sim::habitat_recording::Recording::read(std::path::Path::new(path)) {eprintln!("Cannot open habitat comparison: {e}");return bevy::app::AppExit::error();}
        }
        if let Some(path)=&args.habitat_sweep {
            let checked=(||->Result<(),String> {
                let plan=cosmogon_sim::habitat_sweep::Plan::read(std::path::Path::new(path))?;
                let u=cosmogon_sim::save::read_file(std::path::Path::new(args.load.as_deref().expect("validated sweep load"))).map_err(|e|e.to_string())?;
                let id=args.habitat_study_id.or_else(||(u.habitats.len()==1).then(||u.habitats[0].id)).ok_or("Choose --habitat-study-id for multiple habitats")?;
                plan.validate_world(&u,id)?;
                if args.habitat_sample.is_some_and(|n|n>=plan.samples) {return Err("Requested observation index is outside the sweep plan".into());}
                if std::path::Path::new(args.habitat_study_output.as_deref().expect("validated sweep output")).exists() {return Err("Existing sweep results are protected; choose a new folder".into());}
                Ok(())
            })();
            if let Err(e)=checked {eprintln!("Cannot run habitat sweep: {e}");return bevy::app::AppExit::error();}
        }
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
    let plugins = if (args.habitat_audio.is_some() || args.habitat_sweep.is_some() || args.habitat_recording.is_some() || args.greenhouse_definition.is_some() || args.habitat_definition.is_some() || args.research_recording.is_some() || args.surface_site.is_some() || args.mars_terrain.is_some() || args.terrain_section.is_some()) && args.quit_after_capture {
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
            input::InputPlugin,
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
