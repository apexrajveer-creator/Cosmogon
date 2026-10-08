//! Bounded, read-only performance recordings. These are app telemetry, never universe state.
//! Frame intervals include presentation waits; simulation timings are elapsed wall time.

use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::diagnostic::{DiagnosticsStore, SystemInformationDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::{diagnostic::RenderDiagnosticsPlugin, renderer::RenderAdapterInfo};
use serde::{Deserialize, Serialize};

use crate::args::Args;
use crate::camera::CameraRig;
use crate::persistence::UserSettings;
use crate::render::{terrain_lod::TerrainLod, PendingBake, ViewInfo};
use crate::sim::Sim;
use crate::state::AppState;

const WINDOW_FRAMES: usize = 600;
const MAX_FRAMES: usize = 3600;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameSample {
    pub frame_ms: f64,
    pub simulation_ms: f64,
    pub simulation_steps: u64,
    pub simulation_lagging: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Distribution {
    pub count: usize,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

impl Distribution {
    /// Nearest-rank percentiles preserve slow-frame outliers, unlike an FPS average.
    fn of(values: impl Iterator<Item = f64>) -> Option<Self> {
        let mut v: Vec<_> = values.filter(|v| v.is_finite() && *v >= 0.0).collect();
        if v.is_empty() {
            return None;
        }
        v.sort_unstable_by(f64::total_cmp);
        let n = v.len();
        let percentile = |p: f64| v[((n as f64 * p).ceil() as usize).saturating_sub(1).min(n - 1)];
        Some(Self { count: n, mean: v.iter().sum::<f64>() / n as f64, p50: percentile(0.5), p95: percentile(0.95), p99: percentile(0.99), max: v[n - 1] })
    }
}

#[derive(Default, Debug)]
struct Samples(VecDeque<FrameSample>);

impl Samples {
    fn push(&mut self, sample: FrameSample, limit: usize) {
        if !sample.frame_ms.is_finite() || sample.frame_ms <= 0.0 || !sample.simulation_ms.is_finite() || sample.simulation_ms < 0.0 {
            return;
        }
        let limit = limit.clamp(1, MAX_FRAMES);
        while self.0.len() >= limit {
            self.0.pop_front();
        }
        self.0.push_back(sample);
    }
}

#[derive(Resource)]
pub struct Performance {
    started: Instant,
    pub startup_to_observing_ms: Option<f64>,
    pub last_load_ms: Option<f64>,
    pub detailed: bool,
    pub frame_summary: Option<Distribution>,
    pub simulation_summary: Option<Distribution>,
    pub export_requested: bool,
    pub export_status: Option<String>,
    pub automatic_done: bool,
    samples: Samples,
    last_summary: Instant,
    observing_frames: usize,
    latest_report: serde_json::Value,
}

impl Performance {
    pub fn new(started: Instant) -> Self {
        Self { started, startup_to_observing_ms: None, last_load_ms: None, detailed: false, frame_summary: None, simulation_summary: None, export_requested: false, export_status: None, automatic_done: false, samples: Samples::default(), last_summary: started, observing_frames: 0, latest_report: serde_json::Value::Null }
    }

    pub fn report(&self) -> &serde_json::Value {
        &self.latest_report
    }

    fn reset(&mut self) {
        self.samples = Samples::default();
        self.frame_summary = None;
        self.simulation_summary = None;
        self.latest_report = serde_json::Value::Null;
        self.observing_frames = 0;
        // Each universe gets its own recording; never pool unrelated scenes.
    }
}

pub struct PerformancePlugin;

impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        let detailed = app.world().resource::<Args>().profile.is_some() || app.world().resource::<UserSettings>().detailed_profiling;
        app.world_mut().resource_mut::<Performance>().detailed = detailed;
        if detailed {
            app.add_plugins((RenderDiagnosticsPlugin, SystemInformationDiagnosticsPlugin));
        }
        app.add_systems(OnEnter(AppState::Observing), begin_observing)
            // A report can request exit; the existing save-on-exit system must see it this frame.
            .add_systems(Last, record.before(crate::sim::autosave_on_exit).run_if(in_state(AppState::Observing).and(resource_exists::<Sim>)));
    }
}

fn begin_observing(mut perf: ResMut<Performance>) {
    let ms = perf.started.elapsed().as_secs_f64() * 1000.0;
    perf.startup_to_observing_ms.get_or_insert(ms);
    perf.reset();
}

fn diagnostic_snapshot(diagnostics: &DiagnosticsStore, gpu_supported: bool) -> serde_json::Value {
    let mut values = std::collections::BTreeMap::new();
    // sysinfo reports zero for both process counters when process lookup is denied.
    // A running renderer cannot have zero resident memory; don't present these as measurements.
    let process_available = diagnostics.get(&SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE)
        .and_then(|d| d.measurement()).is_some_and(|m| m.value.is_finite() && m.value > 0.0);
    for d in diagnostics.iter() {
        let path = d.path().as_str();
        if !(path.starts_with("render/") || path.starts_with("process/")) {
            continue;
        }
        if (path.ends_with("/elapsed_gpu") && !gpu_supported) || (path.starts_with("process/") && !process_available) { continue; }
        if let Some(m) = d.measurement().filter(|m| m.value.is_finite() && m.value >= 0.0) {
            // Asynchronous counters are snapshots, not one independent sample per app frame.
            values.insert(path.to_string(), serde_json::json!({"value": m.value, "unit": d.suffix, "age_ms": m.time.elapsed().as_secs_f64() * 1000.0}));
        }
    }
    serde_json::json!(values)
}

// Keep the optional presentation resources together without exceeding Bevy's
// system-parameter arity; minimal measurement apps can omit all three.
#[derive(bevy::ecs::system::SystemParam)]
struct PresentationContext<'w> {
    paths: Option<Res<'w,crate::cinematic::Player>>,
    bindings: Option<Res<'w,crate::input::Bindings>>,
    habitat: Option<Res<'w,crate::ui::HabitatUi>>,
}
#[allow(clippy::too_many_arguments)]
fn record(
    mut perf: ResMut<Performance>,
    args: Res<Args>,
    time: Res<Time<Real>>,
    sim: Res<Sim>,
    settings: Res<UserSettings>,
    diagnostics: Res<DiagnosticsStore>,
    adapter: Option<Res<RenderAdapterInfo>>,
    rig: Res<CameraRig>,
    view: Res<ViewInfo>,
    lod: Res<TerrainLod>,
    pending: Query<(), With<PendingBake>>,
    windows: Query<&Window>,
    mut exit: MessageWriter<AppExit>,
    research: Option<Res<crate::research::Research>>,
    batch: Option<Res<crate::research_batch::ResearchBatch>>,
    presentation_context:PresentationContext,
) {
    let PresentationContext {paths,bindings,habitat}=presentation_context;
    perf.observing_frames += 1;
    let automatic = args.profile.is_some() && !perf.automatic_done;
    let warmup = if automatic { args.profile_warmup.max(1) } else { 1 };
    if perf.observing_frames <= warmup {
        return;
    }
    let limit = if automatic { args.profile_frames.clamp(1, MAX_FRAMES) } else { WINDOW_FRAMES };
    perf.samples.push(FrameSample {
        frame_ms: time.delta_secs_f64() * 1000.0,
        simulation_ms: if sim.paused { 0.0 } else { sim.sim_ms },
        simulation_steps: if sim.paused { 0 } else { sim.last.steps },
        simulation_lagging: !sim.paused && sim.last.lagging,
    }, limit);
    let ready = automatic && perf.samples.0.len() == limit;
    if !ready && !perf.export_requested && perf.last_summary.elapsed().as_secs_f64() < 0.25 {
        return;
    }
    perf.last_summary = Instant::now();
    perf.frame_summary = Distribution::of(perf.samples.0.iter().map(|s| s.frame_ms));
    perf.simulation_summary = Distribution::of(perf.samples.0.iter().map(|s| s.simulation_ms));
    // Bevy 0.18 render diagnostics support GPU timings on Vulkan/DX12. Metal can
    // still publish a zero-valued bloom timestamp: its presence is not support.
    let gpu_supported = adapter.as_ref().is_some_and(|a| matches!(a.backend.to_str(), "vulkan" | "dx12"));
    let metrics = diagnostic_snapshot(&diagnostics, gpu_supported);
    let gpu_available = metrics.as_object().is_some_and(|m| m.keys().any(|k| k.ends_with("/elapsed_gpu")));
    let u = &sim.universe;
    let focus_name = rig.focus.map(|t| match t {
        crate::sim::Target::Star(s) => u.system(s).star.name.as_str(),
        crate::sim::Target::Body(r) => u.body(r).name.as_str(),
    });
    let resolution = windows.single().ok().map(|w| [w.physical_width(), w.physical_height()]);
    perf.latest_report = serde_json::json!({
        "format": "cosmogon-performance", "schema_version": 1,
        "app_version": env!("CARGO_PKG_VERSION"), "recorded_unix": crate::persistence::now_unix(),
        "platform": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH,
            "adapter": adapter.as_ref().map(|a| &a.name), "backend": adapter.as_ref().map(|a| format!("{:?}", a.backend))},
        "measurement": {"detailed": perf.detailed, "automatic": automatic, "warmup_frames": warmup,
            "pipelined_rendering": !((args.habitat_audio.is_some() || args.habitat_sweep.is_some() || args.habitat_recording.is_some() || args.greenhouse_definition.is_some() || args.habitat_definition.is_some() || args.research_recording.is_some() || args.surface_site.is_some() || args.mars_terrain.is_some() || args.terrain_section.is_some()) && args.quit_after_capture),
            "window_capacity": limit, "percentile_method": "nearest-rank",
            "startup_to_observing_ms": perf.startup_to_observing_ms, "last_load_ms": perf.last_load_ms},
        "scene": {"scenario": u.settings.scenario.label(), "seed": u.settings.seed,
            "simulation_time_s": u.time, "paused": sim.paused, "speed_index": sim.speed,
            "systems": u.systems.len(), "bodies": u.systems.iter().map(|s| s.bodies.len()).sum::<usize>(),
            "civilizations": u.civs.len(), "data_versions": cosmogon_sim::save::data_versions(u),
            "physics_engine": cosmogon_sim::save::PHYSICS_ENGINE,
            "physics": u.systems.iter().map(|s| s.dynamics.as_ref().map(|d| d.settings.preset.label()).unwrap_or("Kepler")).collect::<Vec<_>>()},
        "view": {"graphics": settings.graphics.label(), "resolution_pixels": resolution,
            "present_mode": windows.single().ok().map(|w| format!("{:?}", w.present_mode)),
            "focus": focus_name, "position_m": view.origin.to_array(),
            "distance_m": rig.focus.map(|t| (rig.eye() - crate::camera::target_info(&sim, t).0).length()),
            "clearance_m": rig.clearance,
            "camera_path":paths.as_ref().and_then(|p|p.metadata(&sim,&rig)),
            "input_bindings":bindings.as_ref().map(|b|b.metadata()),
            "greenhouse_planner":habitat.as_ref().map(|h|h.crop.metadata()),
            "habitat_moisture":habitat.as_ref().and_then(|h|h.moisture_metadata(&sim.universe)),
            "habitat_recording":habitat.as_ref().and_then(|h|h.recording.metadata()),
            "camera_mode": if rig.free.is_some() { "free" } else { "orbit" },
            "orientation_xyzw": rig.rotation.to_array(), "presentation": settings.presentation,
            "yaw_rad": rig.yaw, "pitch_rad": rig.pitch, "fov_y_rad": view.fov_y,
            "terrain_visible": lod.visible_patches, "terrain_building": lod.building, "texture_bakes_pending": pending.iter().count()},
        "frame_ms": perf.frame_summary, "simulation_ms": perf.simulation_summary,
        "lagging_frames": perf.samples.0.iter().filter(|s| s.simulation_lagging).count(),
        "diagnostics": metrics, "gpu_timing_available": gpu_available, "vram_bytes": null,
        "limitations": [
            "Frame intervals include VSync, scheduling and presentation waits; they are not GPU time.",
            "Simulation time is advance_to elapsed wall time, not process CPU utilization.",
            "Process CPU is percent of all logical CPUs; process memory is GiB, not percent.",
            "Process counters are unavailable when resident memory cannot be read; zero fallback counters are omitted.",
            "Render-pass snapshots may overlap and must not be summed. Their age is recorded.",
            "GPU timing is unavailable when the backend supplies no timestamp measurements. VRAM is not measured.",
            "Observing means the universe is ready, not that every texture or terrain job has completed."
        ],
        "samples": perf.samples.0,
    });
    if ready || perf.export_requested {
        perf.export_requested = false;
        let path = if ready { PathBuf::from(args.profile.as_ref().unwrap()) } else { report_path() };
        let result = write_report(&path, &perf.latest_report);
        let failed = result.is_err();
        perf.export_status = Some(match result {
            Ok(()) => { info!("Performance report saved to {}", path.display()); format!("Saved to {}", path.display()) }
            Err(e) => { error!("Performance report failed: {e}"); format!("Export failed: {e}") }
        });
        if ready {
            perf.automatic_done = true;
            if args.quit_after_capture && (failed || (args.capture.is_none() && (args.research_export.is_none() || research.as_ref().is_some_and(|r| r.automatic_done)) && (args.research_batch.is_none() || batch.as_ref().is_some_and(|b|b.automatic_done)) && (args.research_recording.is_none() || research.as_ref().is_some_and(|r|r.import_automatic_done)))) {
                exit.write(if failed { AppExit::error() } else { AppExit::Success });
            }
        }
    }
}

fn report_path() -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    crate::persistence::data_dir().join("reports").join(format!("performance-{nanos}.json"))
}

/// Refuse to overwrite an earlier experiment; failures are shown in the panel/exit status.
fn write_report(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if let Err(e) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(e.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_keep_slow_frames_and_reject_invalid_measurements() {
        let d = Distribution::of((1..=100).map(|x| x as f64).chain([f64::NAN, f64::INFINITY, -1.0])).unwrap();
        assert_eq!(d, Distribution { count: 100, mean: 50.5, p50: 50.0, p95: 95.0, p99: 99.0, max: 100.0 });
        assert_eq!(Distribution::of(std::iter::empty()), None);
        assert_eq!(Distribution::of([0.0].into_iter()).unwrap().p99, 0.0);
    }

    #[test]
    fn recording_is_bounded_and_resets_between_universes() {
        let mut perf = Performance::new(Instant::now());
        for i in 1..=10 {
            perf.samples.push(FrameSample { frame_ms: i as f64, simulation_ms: 0.0, simulation_steps: 0, simulation_lagging: false }, 3);
        }
        assert_eq!(perf.samples.0.len(), 3);
        assert_eq!(perf.samples.0.front().unwrap().frame_ms, 8.0);
        perf.reset();
        assert!(perf.samples.0.is_empty());
        assert!(perf.frame_summary.is_none());
    }

    #[test]
    fn unavailable_metrics_stay_absent_and_reports_never_overwrite() {
        let value = diagnostic_snapshot(&DiagnosticsStore::default(), false);
        assert_eq!(value, serde_json::json!({}));
        let path = std::env::temp_dir().join(format!("cosmogon-report-test-{}-{}.json", std::process::id(), report_path().file_name().unwrap().to_string_lossy()));
        write_report(&path, &value).unwrap();
        assert!(write_report(&path, &serde_json::json!({"replacement": true})).is_err());
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(&path).unwrap()).unwrap(), value);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unsupported_gpu_and_denied_process_counters_are_not_zero_measurements() {
        use bevy::diagnostic::{Diagnostic, DiagnosticMeasurement, DiagnosticPath};
        let mut store = DiagnosticsStore::default();
        for (path, value) in [("process/mem_usage", 0.0), ("process/cpu_usage", 0.0), ("render/bloom/elapsed_gpu", 0.0), ("render/bloom/elapsed_cpu", 0.2)] {
            let mut d = Diagnostic::new(DiagnosticPath::new(path));
            d.add_measurement(DiagnosticMeasurement { time: bevy::platform::time::Instant::now(), value });
            store.add(d);
        }
        let snapshot = diagnostic_snapshot(&store, false);
        assert!(snapshot["process/mem_usage"].is_null());
        assert!(snapshot["process/cpu_usage"].is_null());
        assert!(snapshot["render/bloom/elapsed_gpu"].is_null());
        assert_eq!(snapshot["render/bloom/elapsed_cpu"]["value"], 0.2);
        let mut memory = Diagnostic::new(SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE);
        memory.add_measurement(DiagnosticMeasurement { time: bevy::platform::time::Instant::now(), value: 0.7 });
        store.add(memory);
        let supported = diagnostic_snapshot(&store, true);
        assert_eq!(supported["process/cpu_usage"]["value"], 0.0);
        assert_eq!(supported["process/mem_usage"]["value"], 0.7);
        assert!(supported["render/bloom/elapsed_gpu"].is_object());
    }

    #[test]
    fn recording_exports_after_warmup_without_mutating_the_universe() {
        let path = std::env::temp_dir().join(format!("cosmogon-e2e-{}-{}.json", std::process::id(), report_path().file_name().unwrap().to_string_lossy()));
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings { scenario: cosmogon_sim::Scenario::EmptySystem, ..Default::default() });
        let before = serde_json::to_string(&u).unwrap();
        let mut sim = Sim::new(u, crate::sim::Session::Reference { dataset: "test".into(), title: "Test".into() });
        sim.paused = true;
        sim.sim_ms = 123.0; // A paused frame must not reuse an earlier expensive step.
        let mut app = App::new();
        app.insert_resource(Args { profile: Some(path.to_string_lossy().into()), profile_frames: 3, profile_warmup: 2, quit_after_capture: true, ..Default::default() })
            .insert_resource(Performance::new(Instant::now()))
            .insert_resource(sim)
            .insert_resource(Time::<Real>::default())
            .init_resource::<UserSettings>()
            .init_resource::<DiagnosticsStore>()
            .init_resource::<CameraRig>()
            .init_resource::<ViewInfo>()
            .init_resource::<TerrainLod>()
            .add_message::<AppExit>()
            .add_systems(Last, record);
        for _ in 0..4 {
            app.world_mut().resource_mut::<Time<Real>>().advance_by(std::time::Duration::from_millis(16));
            app.update();
            assert!(!path.exists());
        }
        app.world_mut().resource_mut::<Time<Real>>().advance_by(std::time::Duration::from_millis(40));
        app.update();
        let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(report["frame_ms"]["count"], 3);
        assert_eq!(report["frame_ms"]["p95"], 40.0);
        assert_eq!(report["simulation_ms"]["max"], 0.0);
        assert_eq!(report["gpu_timing_available"], false);
        assert!(report["vram_bytes"].is_null());
        assert!(app.world().resource::<Performance>().automatic_done);
        assert_eq!(serde_json::to_string(&app.world().resource::<Sim>().universe).unwrap(), before);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn old_settings_load_and_profiling_preference_round_trips() {
        let mut old: UserSettings = toml::from_str("graphics = 'High'\n").unwrap();
        assert!(!old.detailed_profiling);
        old.detailed_profiling = true;
        let loaded: UserSettings = toml::from_str(&toml::to_string(&old).unwrap()).unwrap();
        assert!(loaded.detailed_profiling);
    }
}
