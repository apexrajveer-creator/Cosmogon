//! Screenshots.
//!
//! * Photo mode (camera button / P): hides the interface for a moment and saves a clean,
//!   full-resolution PNG to `~/Pictures/Cosmogon`.
//! * Developer capture: `--capture out.png [--capture-after N] [--quit]` saves a screenshot
//!   from the GPU (no OS screen-recording permission needed). Used for visual regression
//!   checks and documentation images.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

use crate::args::Args;
use crate::state::AppState;

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Photo>()
            .init_resource::<CaptureOutcome>()
            .add_systems(Update, capture.run_if(in_state(AppState::Observing).or(in_state(AppState::MainMenu))))
            .add_systems(Update, take_photo.run_if(in_state(AppState::Observing)));
    }
}

/// A pending photo: the interface is hidden for a couple of frames, then the frame is saved.
#[derive(Resource, Default)]
pub struct Photo {
    pub requested: bool,
    frames: u32,
    restore_hidden: Option<bool>,
    restore_gizmos: bool,
}

/// Where photos go: `~/Pictures/Cosmogon/cosmogon-<unix time>.png`.
pub fn photo_path() -> std::path::PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(std::path::PathBuf::from).unwrap_or_else(|| ".".into());
    let dir = home.join("Pictures").join("Cosmogon");
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    dir.join(format!("cosmogon-{secs}.png"))
}

#[allow(clippy::too_many_arguments)]
fn take_photo(mut commands: Commands, mut photo: ResMut<Photo>, mut ui: ResMut<crate::ui::UiState>, mut sim: ResMut<crate::sim::Sim>, time: Res<Time>, mut gizmos: ResMut<GizmoConfigStore>, rig: Res<crate::camera::CameraRig>, settings: Res<crate::persistence::UserSettings>) {
    if !photo.requested {
        if !ui.photo_requested {
            return;
        }
        ui.photo_requested = false;
        photo.requested = true;
    }
    if photo.restore_hidden.is_none() {
        photo.restore_hidden = Some(ui.hidden);
        ui.hidden = true;
        ui.photo_mode = true;
        // Orbits, trails and predictions are gizmos: switch them off for the shot.
        photo.restore_gizmos = gizmos.config_mut::<DefaultGizmoConfigGroup>().0.enabled;
        gizmos.config_mut::<DefaultGizmoConfigGroup>().0.enabled = false;
        photo.frames = 0;
        return;
    }
    photo.frames += 1;
    // Two frames for the interface to disappear from the swap chain.
    if photo.frames == 2 {
        let path = photo_path();
        let ok = path.parent().is_some_and(|d| std::fs::create_dir_all(d).is_ok());
        if ok {
            let metadata = settings.photo_metadata.then(|| photo_metadata(&sim, &rig, &settings));
            commands.spawn(Screenshot::primary_window()).observe(save_photo(path, metadata, false));
            sim.status = Some(("Saving photo…".into(), time.elapsed_secs_f64()));
        } else {
            sim.status = Some(("Couldn't create the Pictures/Cosmogon folder".into(), time.elapsed_secs_f64()));
        }
    }
    if photo.frames >= 4 {
        ui.hidden = photo.restore_hidden.take().unwrap_or(false);
        ui.photo_mode = false;
        gizmos.config_mut::<DefaultGizmoConfigGroup>().0.enabled = photo.restore_gizmos;
        photo.requested = false;
    }
}

#[allow(clippy::too_many_arguments)]
fn capture(mut commands: Commands, args: Res<Args>, mut frames: Local<u32>, mut done: Local<bool>, mut exit: MessageWriter<AppExit>, state: Res<State<AppState>>, perf: Res<crate::performance::Performance>, outcome: Res<CaptureOutcome>, sim: Option<Res<crate::sim::Sim>>, rig: Res<crate::camera::CameraRig>, settings: Res<crate::persistence::UserSettings>) {
    let Some(path) = &args.capture else { return };
    // Wait for the requested universe rather than capturing the menu.
    if (args.new.is_some() || args.load.is_some() || args.continue_latest) && *state.get() != AppState::Observing {
        return;
    }
    *frames += 1;
    if !*done && *frames == args.capture_after {
        *done = true;
        info!("capturing screenshot to {path}");
        let metadata = args.capture_metadata.then(|| sim.as_ref().map(|sim| photo_metadata(sim, &rig, &settings))).flatten();
        commands.spawn(Screenshot::primary_window()).observe(save_photo(path.into(), metadata, true));
    }
    if *done && args.quit_after_capture && outcome.0.is_none() && *frames > args.capture_after.saturating_add(2000) { error!("Screenshot capture timed out"); exit.write(AppExit::error()); return; }
    if *done && args.quit_after_capture && outcome.0 == Some(false) { exit.write(AppExit::error()); return; }
    if *done && args.quit_after_capture && (args.profile.is_none() || perf.automatic_done) && *frames > args.capture_after + 20 && outcome.0 == Some(true) {
        exit.write(AppExit::Success);
    }
}


#[derive(Resource, Default)]
struct CaptureOutcome(Option<bool>);

fn photo_metadata(sim: &crate::sim::Sim, rig: &crate::camera::CameraRig, settings: &crate::persistence::UserSettings) -> serde_json::Value {
    serde_json::json!({
        "format": "cosmogon-photo", "schema_version": 1,
        "app_version": env!("CARGO_PKG_VERSION"),
        "capture_request_simulation_time_s": sim.universe.time,
        "seed": sim.universe.settings.seed,
        "dataset_versions": cosmogon_sim::save::data_versions(&sim.universe),
        "camera_mode": if rig.free.is_some() { "free" } else { "orbit" },
        "camera_position_render_m": rig.eye().to_array(), "orientation_xyzw": rig.rotation.to_array(),
        "presentation": settings.presentation,
        "graphics": settings.graphics.label(),
        "note": "View state at capture request; does not change simulation physics."
    })
}

// Bevy supplies these observer parameters; keep the callback's ownership explicit.
#[allow(clippy::type_complexity)]
fn save_photo(path: std::path::PathBuf, metadata: Option<serde_json::Value>, developer: bool) -> impl FnMut(On<ScreenshotCaptured>, Option<ResMut<crate::sim::Sim>>, Res<Time>, ResMut<CaptureOutcome>) {
    move |event, sim, time, mut outcome| {
        let result = (|| -> Result<(), String> {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
            let img = event.image.clone().try_into_dynamic().map_err(|e| e.to_string())?.to_rgb8();
            img.save_with_format(&path, image::ImageFormat::Png).map_err(|e| e.to_string())?;
            if let Some(mut data) = metadata.clone() {
                data["resolution_pixels"] = serde_json::json!([img.width(), img.height()]);
                let bytes = serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?;
                std::fs::write(path.with_extension("png.json"), bytes).map_err(|e| format!("PNG saved, but metadata failed: {e}"))?;
            }
            Ok(())
        })();
        if developer { outcome.0 = Some(result.is_ok()); }
        let message = match result {
            Ok(()) => { info!("Photo saved to {}", path.display()); format!("Photo saved to {}", path.display()) }
            Err(e) => { error!("Photo failed: {e}"); format!("Photo failed: {e}") }
        };
        if let Some(mut sim) = sim { sim.status = Some((message, time.elapsed_secs_f64())); }
    }
}
