//! Command-line options. End users never need these; they exist for development,
//! automated screenshots and reproducible bug reports.

use bevy::prelude::Resource;
use cosmogon_sim::Scenario;

#[derive(Resource, Clone, Debug, Default)]
pub struct Args {
    /// Skip the menu and create this scenario immediately.
    pub new: Option<Scenario>,
    pub seed: Option<u64>,
    /// Life preset index (0 realistic, 1 hopeful, 2 teeming).
    pub life: Option<usize>,
    /// Simulate this many years headlessly before showing the universe.
    pub advance_years: f64,
    /// Name of a body or star system to focus at start.
    pub focus: Option<String>,
    /// Camera distance in radii of the focused object.
    pub distance: Option<f64>,
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub speed: Option<usize>,
    /// Save a screenshot to this path after `capture_after` frames, then (optionally) quit.
    pub capture: Option<String>,
    pub capture_after: u32,
    pub quit_after_capture: bool,
    pub load: Option<String>,
    pub hide_ui: bool,
    pub select_civ: bool,
    pub debug: bool,
    /// Place the camera above this latitude,longitude (degrees) of the focused body.
    pub latlon: Option<(f64, f64)>,
    pub surface_site: Option<String>,
    pub mars_terrain: Option<String>,
    pub terrain_section: Option<String>,
    pub habitat_definition: Option<String>,
    pub flight_definition:Option<String>,
    pub flight_origin:Option<String>,
    pub flight_recording:Option<String>,
    pub flight_output:Option<String>,
    pub arrival_output:Option<String>,
    pub support_output:Option<String>,
    pub support_definition:Option<String>,
    pub support_recording:Option<String>,
    pub flight_observation:Option<usize>,
    pub vehicle_plan:Option<String>,
    pub vehicle_recording:Option<String>,
    pub vehicle_output:Option<String>,
    pub vehicle_observation:Option<usize>,
    pub cargo_plan:Option<String>,
    pub cargo_recording:Option<String>,
    pub cargo_output:Option<String>,
    pub cargo_observation:Option<usize>,
    pub habitat_audio: Option<u64>,
    pub habitat_audio_export: Option<String>,
    pub habitat_recording: Option<String>,
    pub habitat_figure_output: Option<String>,
    pub habitat_balance_output: Option<String>,
    pub habitat_balance_from: Option<usize>,
    pub habitat_compare: Option<String>,
    pub habitat_alignment: Option<String>,
    pub habitat_sweep: Option<String>,
    pub habitat_study_output: Option<String>,
    pub habitat_study_id: Option<u64>,
    pub habitat_metric: Option<String>,
    pub habitat_effect: Option<String>,
    pub habitat_verify: bool,
    pub habitat_graph: Option<String>,
    pub habitat_sample: Option<usize>,
    pub habitat_branches: Option<String>,
    pub habitat_origin: Option<String>,
    pub habitat_rerun_output: Option<String>,
    pub habitat_pressure_reference:Option<usize>,
    pub greenhouse_definition: Option<String>,
    pub crop_chemistry: Option<String>,
    pub crop_chemistry_output: Option<String>,
    pub greenhouse_supplies: Option<String>,
    pub greenhouse_supply_output: Option<String>,
    pub greenhouse_supply_day: Option<usize>,
    pub greenhouse_supply_resource: Option<String>,
    pub surface_altitude: f64,
    pub surface_snapshot: bool,
    pub surface_horizon: bool,
    /// Apply a curated experiment (`--what-if chicxulub_today`) to a new Solar System Lab.
    pub what_if: Option<String>,
    /// Start this guided tour (developer: combine with --what-if for tours that need one).
    pub tour: Option<String>,
    /// Physics preset to start the home system with dynamic gravity (fast|balanced|accurate|research).
    pub nbody: Option<String>,
    /// Open a tool window at start (create | launch | physics | palette) — for captures.
    pub panel: Option<String>,
    /// Open a home screen at start (new | load | scenarios | settings | credits) — for captures.
    pub menu: Option<String>,
    /// Developer starts are saved to the library (normally they are not).
    pub persist: bool,
    /// Open the most recent sandbox (same as CONTINUE).
    pub continue_latest: bool,
    /// Export a bounded performance recording (native developer tooling).
    pub profile: Option<String>,
    pub profile_frames: usize,
    pub profile_warmup: usize,
    pub free_camera: bool,
    pub input_bindings: Option<String>,
    pub camera_path: Option<String>,
    pub camera_path_time: Option<f64>,
    pub camera_path_play: bool,
    pub fov: Option<f32>,
    pub exposure: Option<f32>,
    pub capture_metadata: bool,
    pub sound_mode: Option<crate::audio::SoundMode>,
    pub quiet: bool,
    pub research_export: Option<String>,
    pub research_samples: usize,
    pub research_every: usize,
    pub research_plan: Option<String>,
    pub research_batch: Option<String>,
    pub research_recording: Option<String>,
    pub research_workspace: Option<String>,
}

impl Args {
    pub fn parse() -> Self {
        let a = Self::parse_from(std::env::args().skip(1));
        if let Err(e) = a.validate_surface() { eprintln!("{e}"); std::process::exit(1); }
        if a.research_recording.is_some() && a.research_export.is_some() {
            eprintln!("Choose either --research-recording to open saved data or --research-export to record live data");
            std::process::exit(1);
        }
        a
    }

    fn parse_from(args: impl Iterator<Item = String>) -> Self {
        let mut a = Args { capture_after: 90, profile_frames: 600, profile_warmup: 120, research_samples: 60, research_every: 20, surface_altitude: 100_000.0, ..Default::default() };
        let mut it = args;
        while let Some(flag) = it.next() {
            let mut val = || it.next().unwrap_or_default();
            match flag.as_str() {
                "--new" => {
                    a.new = Some(match val().as_str() {
                        "garden" => Scenario::GardenWorld,
                        "sol" => Scenario::Sol,
                        "lab" => Scenario::SolarSystemLab,
                        "system" => Scenario::StarSystem,
                        "empty" => Scenario::EmptySystem,
                        _ => Scenario::Neighbourhood,
                    })
                }
                "--seed" => a.seed = val().parse().ok(),
                "--life" => a.life = val().parse().ok(),
                "--advance-years" => a.advance_years = val().parse().unwrap_or(0.0),
                "--focus" => a.focus = Some(val()),
                "--distance" => a.distance = val().parse().ok(),
                "--yaw" => a.yaw = val().parse().ok(),
                "--pitch" => a.pitch = val().parse().ok(),
                "--speed" => a.speed = val().parse().ok(),
                "--capture" => a.capture = Some(val()),
                "--capture-after" => a.capture_after = val().parse::<u32>().unwrap_or(90).clamp(1, 1_000_000),
                "--quit" => a.quit_after_capture = true,
                "--load" => a.load = Some(val()),
                "--hide-ui" => a.hide_ui = true,
                "--select-civ" => a.select_civ = true,
                "--debug" => a.debug = true,
                "--what-if" => a.what_if = Some(val()),
                "--tour" => a.tour = Some(val()),
                "--nbody" => a.nbody = Some(val()),
                "--panel" => a.panel = Some(val()),
                "--menu" => a.menu = Some(val()),
                "--persist" => a.persist = true,
                "--continue" => a.continue_latest = true,
                "--capture-metadata" => a.capture_metadata = true,
                "--input-bindings" => a.input_bindings=Some(val()),
                "--camera-path" => a.camera_path=Some(val()),
                "--camera-path-time" => a.camera_path_time=Some(val().parse().unwrap_or(f64::NAN)),
                "--camera-path-play" => a.camera_path_play=true,
                "--free-camera" => a.free_camera = true,
                "--habitat-compare" => a.habitat_compare=Some(val()),
                "--habitat-audio" => {let v=val();a.habitat_audio=Some(v.parse().unwrap_or(u64::MAX));},
                "--habitat-audio-export" => a.habitat_audio_export=Some(val()),
                "--habitat-alignment" => a.habitat_alignment=Some(val()),
                "--habitat-balance-output" => a.habitat_balance_output=Some(val()),
                "--habitat-balance-from" => a.habitat_balance_from=Some(val().parse().unwrap_or(usize::MAX)),
                "--habitat-figure-output" => a.habitat_figure_output=Some(val()),
                "--habitat-sweep" => a.habitat_sweep=Some(val()),
                "--habitat-study-output" => a.habitat_study_output=Some(val()),
                "--habitat-study-id" => {let v=val();a.habitat_study_id=Some(v.parse().unwrap_or(u64::MAX));},
                "--habitat-recording" => a.habitat_recording=Some(val()),
                "--habitat-metric" => a.habitat_metric=Some(val()),
                "--habitat-effect" => a.habitat_effect=Some(val()),
                "--habitat-graph" => a.habitat_graph=Some(val()),
                "--habitat-verify" => a.habitat_verify=true,
                "--habitat-pressure-reference" => {a.habitat_pressure_reference=Some(val().parse().unwrap_or(usize::MAX));},
                "--habitat-origin" => a.habitat_origin=Some(val()),
                "--habitat-rerun-output" => a.habitat_rerun_output=Some(val()),
                "--habitat-branches" => a.habitat_branches=Some(val()),
                "--habitat-sample" => {let v=val();a.habitat_sample=Some(v.parse().unwrap_or(usize::MAX));},
                "--crop-chemistry" => a.crop_chemistry=Some(val()),
                "--crop-chemistry-output" => a.crop_chemistry_output=Some(val()),
                "--greenhouse-supplies" => a.greenhouse_supplies=Some(val()),
                "--greenhouse-supply-output" => a.greenhouse_supply_output=Some(val()),
                "--greenhouse-supply-day" => a.greenhouse_supply_day=Some(val().parse().unwrap_or(usize::MAX)),
                "--greenhouse-supply-resource" => a.greenhouse_supply_resource=Some(val()),
                "--greenhouse-definition" => a.greenhouse_definition=Some(val()),
                "--support-output" => a.support_output=Some(val()),
                "--support-definition" => a.support_definition=Some(val()),
                "--support-recording" => a.support_recording=Some(val()),
                "--flight-definition" => a.flight_definition=Some(val()),
                "--flight-origin" => a.flight_origin=Some(val()),
                "--flight-recording" => a.flight_recording=Some(val()),
                "--flight-output" => a.flight_output=Some(val()),
                "--arrival-output" => a.arrival_output=Some(val()),
                "--flight-observation" => a.flight_observation=Some(val().parse().unwrap_or(usize::MAX)),
                "--vehicle-plan" => a.vehicle_plan=Some(val()),
                "--vehicle-recording" => a.vehicle_recording=Some(val()),
                "--vehicle-output" => a.vehicle_output=Some(val()),
                "--vehicle-observation" => a.vehicle_observation=Some(val().parse().unwrap_or(usize::MAX)),
                "--cargo-plan" => a.cargo_plan=Some(val()),
                "--cargo-recording" => a.cargo_recording=Some(val()),
                "--cargo-output" => a.cargo_output=Some(val()),
                "--cargo-observation" => a.cargo_observation=Some(val().parse().unwrap_or(usize::MAX)),
                "--habitat-definition" => a.habitat_definition=Some(val()),
                "--terrain-section" => a.terrain_section = Some(val()),
                "--mars-terrain" => a.mars_terrain = Some(val()),
                "--surface-site" => a.surface_site = Some(val()),
                "--surface-altitude" => a.surface_altitude = val().parse().unwrap_or(f64::NAN),
                "--surface-snapshot" => a.surface_snapshot = true,
                "--surface-horizon" => a.surface_horizon = true,
                "--fov" => a.fov = val().parse().ok(),
                "--exposure" => a.exposure = val().parse().ok(),
                "--quiet" => a.quiet = true,
                "--sound-mode" => a.sound_mode = match val().as_str() {
                    "realistic" => Some(crate::audio::SoundMode::Realistic),
                    "cinematic" => Some(crate::audio::SoundMode::Cinematic),
                    "hybrid" => Some(crate::audio::SoundMode::Hybrid),
                    _ => None,
                },
                "--research-export" => a.research_export = Some(val()),
                "--research-workspace" => a.research_workspace = Some(val()),
                "--research-recording" => a.research_recording = Some(val()),
                "--research-plan" => a.research_plan = Some(val()),
                "--research-batch" => a.research_batch = Some(val()),
                "--research-samples" => a.research_samples = val().parse::<usize>().unwrap_or(60).clamp(2, 4096),
                "--research-every" => a.research_every = val().parse::<usize>().unwrap_or(20).clamp(1, 600),
                "--profile" => a.profile = Some(val()),
                "--profile-frames" => a.profile_frames = val().parse::<usize>().unwrap_or(600).clamp(1, 3600),
                "--profile-warmup" => a.profile_warmup = val().parse::<usize>().unwrap_or(120).min(36_000),
                "--latlon" => {
                    let v = val();
                    let mut it = v.split(',').map(|x| if x.trim() == "noon" { Ok(f64::NAN) } else { x.trim().parse::<f64>() });
                    if let (Some(Ok(la)), Some(Ok(lo))) = (it.next(), it.next()) {
                        a.latlon = Some((la, lo));
                    }
                }
                // macOS passes a process serial number when launched from Finder.
                f if f.starts_with("-psn_") => {}
                f => eprintln!("ignoring unknown argument {f}"),
            }
        }
        if (a.support_recording.is_some() || a.flight_definition.is_some() || a.flight_recording.is_some() || a.vehicle_plan.is_some() || a.vehicle_recording.is_some() || a.cargo_plan.is_some() || a.cargo_recording.is_some() || a.panel.as_deref().is_some_and(|p|p.starts_with("vehicle")||p=="cargo-epochs") || a.input_bindings.is_some() || a.camera_path.is_some() || a.habitat_recording.is_some() || a.greenhouse_definition.is_some() || a.crop_chemistry.is_some() || a.greenhouse_supplies.is_some() || a.habitat_definition.is_some() || a.terrain_section.is_some() || a.mars_terrain.is_some() || a.surface_site.is_some() || a.profile.is_some() || a.research_export.is_some() || a.research_batch.is_some() || a.research_recording.is_some()) && a.new.is_none() && a.load.is_none() && !a.continue_latest {
            a.new = Some(Scenario::SolarSystemLab);
        }
        a
    }

    fn validate_surface(&self) -> Result<(), String> {
        for (value,label) in [(&self.flight_definition,"flight definition"),(&self.flight_origin,"flight original world"),(&self.flight_recording,"flight recording"),(&self.flight_output,"new flight output"),(&self.arrival_output,"new approach output"),(&self.support_output,"new support output"),(&self.support_definition,"support assumptions"),(&self.support_recording,"support recording")] {
            if value.as_deref().is_some_and(|p|p.trim().is_empty()) {return Err(format!("Choose a nonempty {label} path"));}
        }
        if self.support_recording.is_some() && (self.flight_definition.is_some() || self.flight_origin.is_some() || self.flight_recording.is_some() || self.support_definition.is_some() || self.support_output.is_some() || self.flight_output.is_some() || self.arrival_output.is_some()) {return Err("Choose an existing support report or derive a new report from explicit flight inputs".into());}
        let flight=self.flight_definition.is_some() || self.flight_recording.is_some() || self.support_recording.is_some();
        if self.support_output.is_some() && !(self.flight_definition.is_some() || self.flight_recording.is_some()) {return Err("A support output requires a complete flight source".into());}
        if self.support_definition.is_some() && !flight && self.panel.as_deref()!=Some("vehicle-support") {return Err("Support assumptions require a support panel or explicit flight source".into());}

        if self.flight_definition.is_some() && self.flight_recording.is_some() {return Err("Choose one pinned flight definition or complete flight recording".into());}
        if self.flight_definition.is_some() != self.flight_origin.is_some() {return Err("A flight definition requires its exact --flight-origin world; origin alone is not a study".into());}
        if (self.flight_output.is_some() || self.arrival_output.is_some() || self.flight_observation.is_some()) && !flight {return Err("Flight output/observation requires an explicit definition and origin or complete recording".into());}
        if self.flight_observation.is_some_and(|i|i>=4096) {return Err("Flight observation must be an integer inside the actual 0–4095 recording bounds".into());}
        if flight && (self.vehicle_plan.is_some() || self.vehicle_recording.is_some() || self.cargo_plan.is_some() || self.cargo_recording.is_some() || self.menu.is_some() || self.habitat_audio_export.is_some() || self.habitat_definition.is_some() || self.habitat_sweep.is_some() || self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.crop_chemistry.is_some() || self.greenhouse_supplies.is_some() || self.research_recording.is_some() || self.research_export.is_some() || self.research_batch.is_some() || self.terrain_section.is_some()) {return Err("Choose one automated flight, propulsion, cabin or research workspace".into());}
        for (value,label) in [(&self.vehicle_plan,"vehicle plan"),(&self.vehicle_recording,"vehicle recording"),(&self.vehicle_output,"new vehicle output")] {
            if value.as_deref().is_some_and(|s|s.trim().is_empty()){return Err(format!("Choose a nonempty {label} path"));}
        }
        let vehicle=self.vehicle_plan.is_some() || self.vehicle_recording.is_some();
        if self.vehicle_plan.is_some() && self.vehicle_recording.is_some(){return Err("Choose one vehicle plan or complete recording".into());}
        if (self.vehicle_output.is_some() || self.vehicle_observation.is_some()) && !vehicle {return Err("Vehicle output/observation requires an explicit --vehicle-plan or --vehicle-recording".into());}
        if self.vehicle_observation.is_some_and(|n|n>=8192){return Err("Vehicle observation must be an integer from 0 to 8191 inside the actual plan".into());}
        if vehicle && (self.cargo_plan.is_some() || self.cargo_recording.is_some() || self.menu.is_some() || self.habitat_audio_export.is_some() || self.habitat_definition.is_some() || self.habitat_sweep.is_some() || self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.crop_chemistry.is_some() || self.greenhouse_supplies.is_some() || self.research_recording.is_some() || self.research_export.is_some() || self.research_batch.is_some() || self.terrain_section.is_some()) {return Err("Choose one vehicle, cargo, cabin, research or offline export workspace launch".into());}
        if self.panel.as_deref().is_some_and(|p|p.starts_with("vehicle")||p=="cargo-epochs") && (self.habitat_audio_export.is_some() || self.menu.is_some()){return Err("Vehicle panels require a native observing launch".into());}
        for (value,label) in [(&self.cargo_plan,"cargo-epochs plan"),(&self.cargo_recording,"cargo-epochs recording"),(&self.cargo_output,"new cargo-epochs output")] {
            if value.as_deref().is_some_and(|s|s.trim().is_empty()){return Err(format!("Choose a nonempty {label} path"));}
        }
        let cargo_epochs=self.cargo_plan.is_some() || self.cargo_recording.is_some();
        if self.cargo_plan.is_some() && self.cargo_recording.is_some(){return Err("Choose one cargo-epochs plan or complete recording".into());}
        if (self.cargo_output.is_some() || self.cargo_observation.is_some()) && !cargo_epochs {return Err("Cargo-epochs output/observation requires an explicit --cargo-plan or --cargo-recording".into());}
        if self.cargo_observation.is_some_and(|n|n>=8192){return Err("Cargo-epochs observation must be an integer from 0 to 8191 inside the actual plan".into());}
        if cargo_epochs && (self.flight_definition.is_some() || self.flight_recording.is_some() || self.support_recording.is_some() || self.vehicle_plan.is_some() || self.vehicle_recording.is_some() || self.menu.is_some() || self.habitat_audio_export.is_some() || self.habitat_definition.is_some() || self.habitat_sweep.is_some() || self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.crop_chemistry.is_some() || self.greenhouse_supplies.is_some() || self.research_recording.is_some() || self.research_export.is_some() || self.research_batch.is_some() || self.terrain_section.is_some()) {return Err("Choose one cargo-epochs, flight, vehicle, cabin or research workspace launch".into());}

        if self.crop_chemistry.as_deref().is_some_and(|p|p.trim().is_empty()) {return Err("Choose a verified crop chemistry folder".into());}
        if self.crop_chemistry_output.as_deref().is_some_and(|p|p.trim().is_empty()) {return Err("Choose a nonempty new chemistry output folder".into());}
        if self.crop_chemistry_output.is_some() && self.crop_chemistry.is_none() && self.greenhouse_supplies.is_none() {return Err("Chemistry export requires --crop-chemistry FOLDER or --greenhouse-supplies FILE_OR_FOLDER".into());}
        if self.crop_chemistry_output.is_some() && self.greenhouse_supply_output.is_some() {return Err("Choose one supply or chemistry export".into());}
        if self.crop_chemistry.is_some() && (self.greenhouse_supplies.is_some() || self.greenhouse_definition.is_some() || self.habitat_definition.is_some() || self.habitat_recording.is_some() || self.habitat_sweep.is_some() || self.habitat_audio_export.is_some() || self.greenhouse_supply_output.is_some()) {return Err("Choose one chemistry, supply, forecast, habitat or offline export launch".into());}
        if self.greenhouse_supplies.as_deref().is_some_and(|p|p.trim().is_empty()) {return Err("Choose a crop supply definition or study folder".into());}
        if self.greenhouse_supply_day.is_some_and(|n|n>30) {return Err("Crop supply age must be an integer day from 0 to 30".into());}
        if self.greenhouse_supply_resource.as_deref().is_some_and(|r|!cosmogon_sim::greenhouse_supplies::RESOURCES.contains(&r)) {return Err("Choose liquid_water, carbon_dioxide, nutrient_hno3_equivalent or lighting_energy".into());}
        if (self.greenhouse_supply_output.is_some() || self.greenhouse_supply_day.is_some() || self.greenhouse_supply_resource.is_some()) && self.greenhouse_supplies.is_none() && self.crop_chemistry.is_none() {return Err("Crop supply export/age/resource requires --greenhouse-supplies FILE_OR_FOLDER".into());}
        if self.greenhouse_supply_output.as_deref().is_some_and(|p|p.trim().is_empty()) {return Err("Choose a nonempty new crop supply output folder".into());}
        if self.greenhouse_supplies.is_some() && (self.greenhouse_definition.is_some() || self.habitat_definition.is_some() || self.habitat_recording.is_some() || self.habitat_sweep.is_some() || self.habitat_audio_export.is_some()) {return Err("Choose one crop supply, crop forecast, habitat or offline export launch".into());}
        if self.input_bindings.as_deref()==Some("") {return Err("Choose an input-bindings file".into());}
        if self.input_bindings.is_some() && self.habitat_audio_export.is_some(){return Err("Keyboard controls require a native launch".into());}
        if self.camera_path.is_none() && (self.camera_path_time.is_some() || self.camera_path_play) {return Err("--camera-path-time/--camera-path-play require --camera-path FILE".into());}
        if self.camera_path_time.is_some_and(|t|!t.is_finite() || !(0.0..=3600.0).contains(&t)) {return Err("Camera presentation seconds must be finite and between 0 and 3600".into());}
        if self.camera_path.is_some() {
            if self.camera_path.as_deref()==Some("") {return Err("Choose a camera-path file".into());}
            if self.surface_site.is_some() || self.latlon.is_some() || self.free_camera || self.yaw.is_some() || self.pitch.is_some() || self.distance.is_some() || self.fov.is_some() || self.exposure.is_some() || self.tour.is_some() || self.menu.is_some() || self.habitat_audio_export.is_some() || self.continue_latest || self.what_if.is_some() || self.advance_years!=0.0 {return Err("Camera paths cannot ignore another camera, menu, time-advance or offline export request".into());}
            if self.capture.is_some() && self.camera_path_play {return Err("Use frozen --camera-path-time for captures; --camera-path-play is for ordinary playback".into());}
        }
        if self.habitat_audio==Some(u64::MAX) {return Err("Invalid --habitat-audio ID".into());}
        if self.habitat_audio.is_some() && self.load.is_none() {return Err("--habitat-audio requires --load SAVE".into());}
        if self.habitat_audio_export.is_some() {
            if self.habitat_audio.is_none() {return Err("--habitat-audio-export requires --habitat-audio ID and --load SAVE".into());}
            if self.capture.is_some() || self.profile.is_some() || self.habitat_sweep.is_some() || self.habitat_definition.is_some() || self.research_export.is_some() || self.new.is_some() || self.advance_years!=0.0 {return Err("Sound export reads one saved interval; do not combine with a renderer or simulation launch".into());}
            if self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.research_recording.is_some() || self.research_workspace.is_some() || self.research_plan.is_some() || self.research_batch.is_some() || self.terrain_section.is_some() || self.mars_terrain.is_some() || self.surface_site.is_some() || self.panel.is_some() || self.menu.is_some() || self.continue_latest || self.what_if.is_some() || self.tour.is_some() || self.nbody.is_some() || self.focus.is_some() || self.latlon.is_some() {return Err("Sound export cannot ignore other workspace, camera or study requests".into());}
        }
        if self.habitat_graph.as_deref()==Some("") {return Err("Choose a habitat graph layout file".into());}
        if (self.habitat_metric.is_some() || self.habitat_sample.is_some() || self.habitat_effect.is_some() || self.habitat_verify || self.habitat_graph.is_some()) && self.habitat_recording.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-metric/--habitat-sample/--habitat-effect/--habitat-verify/--habitat-graph require --habitat-recording or --habitat-sweep".into());}
        if self.habitat_compare.is_some() && self.habitat_recording.is_none() {return Err("--habitat-compare requires --habitat-recording A".into());}
        if self.habitat_alignment.is_some() && self.habitat_compare.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-alignment requires a comparison or sweep".into());}
        if let Some(a)=&self.habitat_alignment {cosmogon_sim::habitat_comparison::Alignment::parse(a)?;}
        if self.habitat_balance_from.is_some() && self.habitat_recording.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-balance-from requires recorded or sweep evidence".into());}
        if self.habitat_balance_from.is_some_and(|n|n>=1024) {return Err("--habitat-balance-from must be between 0 and 1023".into());}
        if self.habitat_balance_from.zip(self.habitat_sample).is_some_and(|(a,b)|a>b) {return Err("Choose ordered balance observations".into());}
        if let Some(output)=&self.habitat_balance_output {if output.trim().is_empty() || self.habitat_recording.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-balance-output requires recorded or sweep evidence and a nonempty new folder".into());}}
        if self.habitat_balance_output.is_some() && self.habitat_figure_output.is_some() {return Err("Choose one automatic recorded export at a time".into());}
        if let Some(output)=&self.habitat_figure_output {
            if output.trim().is_empty() || self.habitat_recording.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-figure-output requires a recording/sweep and a nonempty new folder".into());}
        }
        if self.habitat_study_id==Some(u64::MAX) {return Err("Invalid --habitat-study-id".into());}
        if self.habitat_sweep.is_some() {
            if self.load.is_none() || self.habitat_study_output.is_none() {return Err("--habitat-sweep requires --load SAVE and --habitat-study-output NEW_DIR".into());}
            if self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.habitat_definition.is_some() {return Err("Choose a sweep launch or imported recording/crop/habitat definition".into());}
        } else if self.habitat_study_output.is_some() || self.habitat_study_id.is_some() {return Err("--habitat-study-output/--habitat-study-id require --habitat-sweep".into());}
        if self.habitat_recording.is_some() && self.greenhouse_definition.is_some() {return Err("Choose a habitat recording or a crop planner definition for the launch pane".into());}
        if let Some(effect)=&self.habitat_effect {cosmogon_sim::habitat_factorial::Effect::parse(effect)?;}
        if let Some(metric)=&self.habitat_metric {cosmogon_sim::habitat_recording::Metric::parse(metric)?;}
        if self.habitat_pressure_reference.is_some() && self.habitat_recording.is_none() {return Err("--habitat-pressure-reference requires --habitat-recording".into());}
        if self.habitat_pressure_reference.is_some_and(|n|n>=1024) {return Err("--habitat-pressure-reference must be an index between 0 and 1023".into());}
        if let Some(origin)=&self.habitat_origin {if !matches!(origin.as_str(),"saved"|"live"){return Err("--habitat-origin requires saved or live".into());}}
        if self.habitat_origin.is_some() || self.habitat_rerun_output.is_some() {
            if self.habitat_recording.is_none() || self.habitat_compare.is_some(){return Err("Saved/live origin selection requires a complete recorded design without an external comparison".into());}
            if self.habitat_rerun_output.as_ref().is_some_and(|p|p.trim().is_empty()){return Err("--habitat-rerun-output requires a nonempty new folder".into());}
            if self.habitat_rerun_output.is_some() && self.habitat_origin.as_deref()==Some("live") && self.load.is_none(){return Err("An automated live-origin rerun requires an explicit --load world".into());}
        }
        if let Some(branches)=&self.habitat_branches {
            cosmogon_sim::habitat_design::parse_branches(branches)?;
            if self.habitat_compare.is_some() || self.habitat_recording.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-branches requires a single-setting design/plan without an external comparison".into());}
        }
        if self.habitat_sample.is_some_and(|n|n>=1024) {return Err("--habitat-sample must be an index between 0 and 1023".into());}

        if self.mars_terrain.as_deref().is_some_and(|v|!matches!(v,"4"|"16")) {return Err("--mars-terrain requires 4 or 16 pixels per degree".into());}
        if let Some(id) = &self.surface_site {
            if cosmogon_sim::planet::landing::find(id).is_none() { return Err(format!("Unknown --surface-site {id}; choose perseverance, curiosity, insight or viking-1")); }
            if !self.surface_altitude.is_finite() || !(60.0..=1e8).contains(&self.surface_altitude) { return Err("--surface-altitude must be finite, between 60 and 100000000 metres".into()); }
            if self.latlon.is_some() || self.focus.as_deref().is_some_and(|s|!s.eq_ignore_ascii_case("Mars")) { return Err("--surface-site selects Mars and its source coordinates; do not combine with --latlon or another --focus".into()); }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn support_launches_require_one_complete_source_and_explicit_new_outputs() {
        for flags in [vec!["--support-recording"," "],vec!["--support-output","new"],vec!["--support-recording","r","--flight-recording","f"],vec!["--support-recording","r","--support-definition","d"],vec!["--support-recording","r","--support-output","new"],vec!["--support-definition","d"],vec!["--support-recording","r","--flight-observation","4096"]] {assert!(Args::parse_from(flags.into_iter().map(String::from)).validate_surface().is_err());}
        for flags in [vec!["--support-recording","r","--flight-observation","2"],vec!["--flight-recording","f","--support-definition","d","--support-output","new","--panel","vehicle-support"],vec!["--support-definition","d","--panel","vehicle-support"]] {assert!(Args::parse_from(flags.into_iter().map(String::from)).validate_surface().is_ok());}
    }

    #[test]
    fn origin_requests_require_complete_design_context_and_explicit_live_world() {
        for v in [vec!["--habitat-origin","saved"],vec!["--habitat-recording","x","--habitat-origin","unknown"],vec!["--habitat-recording","x","--habitat-origin"],vec!["--habitat-recording","x","--habitat-rerun-output"," "],vec!["--habitat-recording","x","--habitat-origin","saved","--habitat-compare","y"],vec!["--habitat-recording","x","--habitat-origin","live","--habitat-rerun-output","new"]] {
            assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());
        }
        for v in [vec!["--habitat-recording","x","--habitat-rerun-output","new"],vec!["--habitat-recording","x","--habitat-origin","live","--habitat-rerun-output","new","--load","current.cosmo"],vec!["--habitat-recording","x","--habitat-origin","saved"]] {
            assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_ok());
        }
    }
    #[test]
    fn crop_chemistry_requires_one_explicit_valid_launch() {
        for v in [vec!["--crop-chemistry-output","x"],vec!["--crop-chemistry","x","--greenhouse-supplies","y"],vec!["--crop-chemistry","x","--habitat-recording","y"],vec!["--crop-chemistry","x","--greenhouse-supply-output","y"],vec!["--greenhouse-supplies","x","--crop-chemistry-output","a","--greenhouse-supply-output","b"],vec!["--crop-chemistry","x","--greenhouse-supply-day","31"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        assert!(Args::parse_from(["--crop-chemistry","x","--greenhouse-supply-day","12","--crop-chemistry-output","y"].into_iter().map(String::from)).validate_surface().is_ok());
        assert!(Args::parse_from(["--greenhouse-supplies","x","--crop-chemistry-output","y"].into_iter().map(String::from)).validate_surface().is_ok());
    }
    #[test]
    fn crop_supply_launch_refuses_incompatible_or_unbounded_requests() {
        for v in [vec!["--greenhouse-supply-output","x"],vec!["--greenhouse-supplies","x","--greenhouse-supply-day","31"],vec!["--greenhouse-supplies","x","--greenhouse-supply-day","nan"],vec!["--greenhouse-supplies","x","--greenhouse-supply-resource","unknown"],vec!["--greenhouse-supplies","x","--greenhouse-definition","y"],vec!["--greenhouse-supplies","x","--habitat-recording","y"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        assert!(Args::parse_from(["--greenhouse-supplies","x","--greenhouse-supply-day","12","--greenhouse-supply-resource","lighting_energy","--greenhouse-supply-output","z"].into_iter().map(String::from)).validate_surface().is_ok());
    }
    #[test]
    fn balance_export_needs_evidence_and_ordered_indices() {
        for v in [vec!["--habitat-balance-output"],vec!["--habitat-balance-output","new"],vec!["--habitat-recording","x","--habitat-balance-output"," "],vec!["--habitat-recording","x","--habitat-balance-from","nan"],vec!["--habitat-recording","x","--habitat-balance-from","1024"],vec!["--habitat-recording","x","--habitat-balance-from","3","--habitat-sample","2"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        assert!(Args::parse_from(["--habitat-recording","x","--habitat-balance-from","0","--habitat-balance-output","new"].into_iter().map(String::from)).validate_surface().is_ok());
    }
    #[test]
    fn figure_launch_requires_recorded_context_and_nonempty_new_folder() {
        for v in [vec!["--habitat-figure-output"],vec!["--habitat-figure-output","new"],vec!["--habitat-recording","x","--habitat-figure-output"," "]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--habitat-recording","x","--habitat-graph","g","--habitat-figure-output","new"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert_eq!(a.habitat_figure_output.as_deref(),Some("new"));
    }
    #[test]
    fn pressure_reference_requires_recording_and_bounded_index() {
        for v in [vec!["--habitat-pressure-reference"],vec!["--habitat-pressure-reference","0"],vec!["--habitat-recording","x","--habitat-pressure-reference","nan"],vec!["--habitat-recording","x","--habitat-pressure-reference","1024"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--habitat-recording","x","--habitat-pressure-reference","0"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert_eq!(a.habitat_pressure_reference,Some(0));
    }
    #[test]
    fn graph_layout_requires_file_and_recorded_context() {
        for values in [vec!["--habitat-graph"],vec!["--habitat-graph","layout.json"],vec!["--habitat-recording","root","--habitat-graph"]] {assert!(Args::parse_from(values.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--habitat-recording","root","--habitat-graph","layout.json"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert_eq!(a.habitat_graph.as_deref(),Some("layout.json"));
    }
    #[test]
    fn input_binding_launches_are_explicit_and_use_native_defaults(){
        let a=Args::parse_from(["--input-bindings","controls.json"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert_eq!(a.new,Some(Scenario::SolarSystemLab));
        let a=Args::parse_from(["--input-bindings",""].into_iter().map(String::from));assert!(a.validate_surface().is_err());
        let a=Args::parse_from(["--input-bindings","f","--load","s","--habitat-audio","0","--habitat-audio-export","o"].into_iter().map(String::from));assert!(a.validate_surface().is_err());
    }
    #[test]
    fn camera_path_launches_require_explicit_valid_unambiguous_controls(){
        for values in [vec!["--camera-path-time","1"],vec!["--camera-path-play"],vec!["--camera-path",""],vec!["--camera-path","p","--camera-path-time","NaN"],vec!["--camera-path","p","--camera-path-time","3601"],vec!["--camera-path","p","--latlon","0,0"],vec!["--camera-path","p","--fov","40"],vec!["--camera-path","p","--capture","x","--camera-path-play"]]{assert!(Args::parse_from(values.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--camera-path","p","--camera-path-time","3"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert_eq!(a.camera_path_time,Some(3.0));assert_eq!(a.new,Some(Scenario::SolarSystemLab));
    }
    #[test]
    fn sound_preview_requires_saved_live_context_and_export_is_unambiguous() {
        for v in [vec!["--habitat-audio","bad"],vec!["--habitat-audio","0"],vec!["--habitat-audio-export","o"],vec!["--load","s","--habitat-audio","0","--habitat-audio-export","o","--capture","x"],vec!["--load","s","--habitat-audio","0","--habitat-audio-export","o","--advance-years","1"]] {
            assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());
        }
        assert!(Args::parse_from(["--load","s","--habitat-audio","0","--habitat-audio-export","o"].into_iter().map(String::from)).validate_surface().is_ok());
    }
    #[test]
    fn surface_requests_reject_misleading_or_invalid_combinations() {
        for values in [vec!["--mars-terrain","32"],vec!["--mars-terrain",""],vec!["--surface-site","missing"],vec!["--surface-site","curiosity","--surface-altitude","NaN"],vec!["--surface-site","curiosity","--surface-altitude","59"],vec!["--surface-site","curiosity","--focus","Earth"],vec!["--surface-site","curiosity","--latlon","0,0"]] {
            assert!(Args::parse_from(values.into_iter().map(String::from)).validate_surface().is_err());
        }
        let a=Args::parse_from(["--surface-site","perseverance"].into_iter().map(String::from));
        assert!(a.validate_surface().is_ok());assert_eq!(a.new,Some(Scenario::SolarSystemLab));
    }

    #[test]
    fn research_limits_and_default_scene_are_bounded() {
        let a = Args::parse_from(["--research-export", "out", "--research-samples", "99999", "--research-every", "0"].into_iter().map(String::from));
        assert_eq!(a.research_samples, 4096); assert_eq!(a.research_every, 1);
        assert!(matches!(a.new, Some(Scenario::SolarSystemLab)));
    }

    #[test]
    fn profile_limits_are_bounded_and_a_scene_is_selected() {
        let a = Args::parse_from(["--profile", "out.json", "--profile-frames", "0", "--profile-warmup", "999999"].into_iter().map(String::from));
        assert_eq!(a.profile_frames, 1);
        assert_eq!(a.profile_warmup, 36_000);
        assert!(matches!(a.new, Some(Scenario::SolarSystemLab)));
        let b = Args::parse_from(["--profile", "out.json", "--load", "saved.cosmo", "--profile-frames", "999999"].into_iter().map(String::from));
        assert!(b.new.is_none());
        assert_eq!(b.profile_frames, 3600);
    }
    #[test]
    fn replay_launch_requires_an_explicit_recorded_design_or_sweep() {
        assert!(Args::parse_from(["--habitat-verify"].into_iter().map(String::from)).validate_surface().is_err());
        let a=Args::parse_from(["--habitat-recording","root","--habitat-verify"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert!(a.habitat_verify);
    }
    #[test]
    fn paired_effect_options_are_explicit_and_validated() {
        for key in ["average_a","average_b","difference_in_differences"] {assert!(Args::parse_from(["--habitat-recording","root","--habitat-effect",key].into_iter().map(String::from)).validate_surface().is_ok());}
        for v in [vec!["--habitat-effect","average_a"],vec!["--habitat-recording","root","--habitat-effect","significance"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
    }
    #[test]
    fn habitat_recording_options_reject_ignored_metrics_and_invalid_indices() {
        for v in [vec!["--habitat-metric","water"],vec!["--habitat-sample","1"],vec!["--habitat-recording","x","--habitat-metric","unknown"],vec!["--habitat-recording","x","--habitat-sample","nan"],vec!["--habitat-recording","x","--habitat-sample","1024"],vec!["--habitat-recording","x","--greenhouse-definition","y"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--habitat-recording","x","--habitat-metric","methane","--habitat-sample","24"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert!(matches!(a.new,Some(Scenario::SolarSystemLab)));assert_eq!(a.habitat_sample,Some(24));
    }

    #[test]
    fn habitat_comparison_and_sweep_launch_options_are_explicit() {
        for v in [vec!["--habitat-compare","b"],vec!["--habitat-alignment","world"],vec!["--habitat-recording","a","--habitat-compare","b","--habitat-alignment","nearest"],vec!["--habitat-sweep","p"],vec!["--habitat-sweep","p","--load","s","--habitat-study-output","o","--habitat-recording","a"],vec!["--habitat-study-id","0"],vec!["--habitat-sweep","p","--load","s","--habitat-study-output","o","--habitat-study-id","bad"]] {assert!(Args::parse_from(v.into_iter().map(String::from)).validate_surface().is_err());}
        assert!(Args::parse_from(["--habitat-recording","a","--habitat-compare","b","--habitat-alignment","world"].into_iter().map(String::from)).validate_surface().is_ok());
        assert!(Args::parse_from(["--habitat-sweep","p","--load","s","--habitat-study-output","o","--habitat-sample","24"].into_iter().map(String::from)).validate_surface().is_ok());
    }

    #[test]
    fn arrival_outputs_need_explicit_flight_inputs_and_nonempty_paths() {
        for flags in [vec!["--arrival-output","new"],vec!["--flight-recording","r","--arrival-output"," "]] {assert!(Args::parse_from(flags.into_iter().map(String::from)).validate_surface().is_err());}
        let args=Args::parse_from(["--flight-recording","r","--arrival-output","new","--panel","vehicle-arrival"].into_iter().map(String::from));assert!(args.validate_surface().is_ok());assert_eq!(args.arrival_output.as_deref(),Some("new"));
    }
    #[test]
    fn flight_launches_require_exact_origins_and_refuse_ignored_controls() {
        for flags in [vec!["--flight-definition",""],vec!["--flight-definition","d"],vec!["--flight-origin","s"],vec!["--flight-recording"," "],vec!["--flight-output","new"],vec!["--flight-observation","0"],vec!["--flight-recording","r","--flight-definition","d","--flight-origin","s"],vec!["--flight-recording","r","--flight-observation","NaN"],vec!["--flight-recording","r","--flight-observation","4096"],vec!["--flight-recording","r","--vehicle-plan","p"],vec!["--flight-recording","r","--habitat-recording","h"],vec!["--flight-recording","r","--menu","new"]] {
            assert!(Args::parse_from(flags.into_iter().map(String::from)).validate_surface().is_err());
        }
        let a=Args::parse_from(["--flight-definition","d","--flight-origin","s","--flight-output","new","--flight-observation","3"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert!(matches!(a.new,Some(Scenario::SolarSystemLab)));
        assert!(Args::parse_from(["--flight-recording","r"].into_iter().map(String::from)).validate_surface().is_ok());
    }
    #[test]
    fn vehicle_launches_reject_ignored_inputs_and_use_actual_index_bounds() {
        for flags in [vec!["--vehicle-plan",""],vec!["--vehicle-recording",""],vec!["--vehicle-output","new"],vec!["--vehicle-observation","3"],vec!["--vehicle-plan","p","--vehicle-recording","r"],vec!["--vehicle-plan","p","--vehicle-observation","NaN"],vec!["--vehicle-plan","p","--vehicle-observation","8192"],vec!["--vehicle-plan","p","--vehicle-output",""],vec!["--vehicle-plan","p","--habitat-recording","r"],vec!["--vehicle-plan","p","--greenhouse-definition","g"],vec!["--vehicle-plan","p","--research-export","r"],vec!["--vehicle-plan","p","--menu","new"],vec!["--panel","vehicle","--menu","new"]] {assert!(Args::parse_from(flags.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--vehicle-recording","r","--vehicle-output","new","--vehicle-observation","3"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert!(matches!(a.new,Some(Scenario::SolarSystemLab)));assert_eq!(a.vehicle_observation,Some(3));
        assert!(Args::parse_from(["--vehicle-plan","p","--load","s"].into_iter().map(String::from)).new.is_none());
    }

    #[test]
    fn cargo_epoch_launches_reject_ignored_inputs_and_use_actual_index_bounds() {
        for flags in [vec!["--cargo-plan",""],vec!["--cargo-recording",""],vec!["--cargo-output","new"],vec!["--cargo-observation","3"],vec!["--cargo-plan","p","--cargo-recording","r"],vec!["--cargo-plan","p","--cargo-observation","NaN"],vec!["--cargo-plan","p","--cargo-observation","8192"],vec!["--cargo-plan","p","--cargo-output",""],vec!["--cargo-plan","p","--vehicle-plan","v"],vec!["--cargo-plan","p","--flight-recording","f"],vec!["--cargo-plan","p","--habitat-recording","h"],vec!["--cargo-plan","p","--research-export","e"],vec!["--cargo-plan","p","--menu","new"],vec!["--panel","cargo-epochs","--menu","new"]] {assert!(Args::parse_from(flags.into_iter().map(String::from)).validate_surface().is_err());}
        let a=Args::parse_from(["--cargo-recording","r","--cargo-output","new","--cargo-observation","3"].into_iter().map(String::from));assert!(a.validate_surface().is_ok());assert!(matches!(a.new,Some(Scenario::SolarSystemLab)));assert_eq!(a.cargo_observation,Some(3));
        assert!(Args::parse_from(["--cargo-plan","p","--load","s"].into_iter().map(String::from)).new.is_none());
    }

}
