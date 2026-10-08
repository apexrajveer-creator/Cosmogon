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
    pub habitat_audio: Option<u64>,
    pub habitat_audio_export: Option<String>,
    pub habitat_recording: Option<String>,
    pub habitat_compare: Option<String>,
    pub habitat_alignment: Option<String>,
    pub habitat_sweep: Option<String>,
    pub habitat_study_output: Option<String>,
    pub habitat_study_id: Option<u64>,
    pub habitat_metric: Option<String>,
    pub habitat_sample: Option<usize>,
    pub greenhouse_definition: Option<String>,
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
                "--free-camera" => a.free_camera = true,
                "--habitat-compare" => a.habitat_compare=Some(val()),
                "--habitat-audio" => {let v=val();a.habitat_audio=Some(v.parse().unwrap_or(u64::MAX));},
                "--habitat-audio-export" => a.habitat_audio_export=Some(val()),
                "--habitat-alignment" => a.habitat_alignment=Some(val()),
                "--habitat-sweep" => a.habitat_sweep=Some(val()),
                "--habitat-study-output" => a.habitat_study_output=Some(val()),
                "--habitat-study-id" => {let v=val();a.habitat_study_id=Some(v.parse().unwrap_or(u64::MAX));},
                "--habitat-recording" => a.habitat_recording=Some(val()),
                "--habitat-metric" => a.habitat_metric=Some(val()),
                "--habitat-sample" => {let v=val();a.habitat_sample=Some(v.parse().unwrap_or(usize::MAX));},
                "--greenhouse-definition" => a.greenhouse_definition=Some(val()),
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
        if (a.habitat_recording.is_some() || a.greenhouse_definition.is_some() || a.habitat_definition.is_some() || a.terrain_section.is_some() || a.mars_terrain.is_some() || a.surface_site.is_some() || a.profile.is_some() || a.research_export.is_some() || a.research_batch.is_some() || a.research_recording.is_some()) && a.new.is_none() && a.load.is_none() && !a.continue_latest {
            a.new = Some(Scenario::SolarSystemLab);
        }
        a
    }

    fn validate_surface(&self) -> Result<(), String> {
        if self.habitat_audio==Some(u64::MAX) {return Err("Invalid --habitat-audio ID".into());}
        if self.habitat_audio.is_some() && self.load.is_none() {return Err("--habitat-audio requires --load SAVE".into());}
        if self.habitat_audio_export.is_some() {
            if self.habitat_audio.is_none() {return Err("--habitat-audio-export requires --habitat-audio ID and --load SAVE".into());}
            if self.capture.is_some() || self.profile.is_some() || self.habitat_sweep.is_some() || self.habitat_definition.is_some() || self.research_export.is_some() || self.new.is_some() || self.advance_years!=0.0 {return Err("Sound export reads one saved interval; do not combine with a renderer or simulation launch".into());}
            if self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.research_recording.is_some() || self.research_workspace.is_some() || self.research_plan.is_some() || self.research_batch.is_some() || self.terrain_section.is_some() || self.mars_terrain.is_some() || self.surface_site.is_some() || self.panel.is_some() || self.menu.is_some() || self.continue_latest || self.what_if.is_some() || self.tour.is_some() || self.nbody.is_some() || self.focus.is_some() || self.latlon.is_some() {return Err("Sound export cannot ignore other workspace, camera or study requests".into());}
        }
        if (self.habitat_metric.is_some() || self.habitat_sample.is_some()) && self.habitat_recording.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-metric/--habitat-sample require --habitat-recording or --habitat-sweep".into());}
        if self.habitat_compare.is_some() && self.habitat_recording.is_none() {return Err("--habitat-compare requires --habitat-recording A".into());}
        if self.habitat_alignment.is_some() && self.habitat_compare.is_none() && self.habitat_sweep.is_none() {return Err("--habitat-alignment requires a comparison or sweep".into());}
        if let Some(a)=&self.habitat_alignment {cosmogon_sim::habitat_comparison::Alignment::parse(a)?;}
        if self.habitat_study_id==Some(u64::MAX) {return Err("Invalid --habitat-study-id".into());}
        if self.habitat_sweep.is_some() {
            if self.load.is_none() || self.habitat_study_output.is_none() {return Err("--habitat-sweep requires --load SAVE and --habitat-study-output NEW_DIR".into());}
            if self.habitat_recording.is_some() || self.greenhouse_definition.is_some() || self.habitat_definition.is_some() {return Err("Choose a sweep launch or imported recording/crop/habitat definition".into());}
        } else if self.habitat_study_output.is_some() || self.habitat_study_id.is_some() {return Err("--habitat-study-output/--habitat-study-id require --habitat-sweep".into());}
        if self.habitat_recording.is_some() && self.greenhouse_definition.is_some() {return Err("Choose a habitat recording or a crop planner definition for the launch pane".into());}
        if let Some(metric)=&self.habitat_metric {cosmogon_sim::habitat_recording::Metric::parse(metric)?;}
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

}
