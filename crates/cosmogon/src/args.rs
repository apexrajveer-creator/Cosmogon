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
}

impl Args {
    pub fn parse() -> Self {
        Self::parse_from(std::env::args().skip(1))
    }

    fn parse_from(args: impl Iterator<Item = String>) -> Self {
        let mut a = Args { capture_after: 90, profile_frames: 600, profile_warmup: 120, ..Default::default() };
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
                "--fov" => a.fov = val().parse().ok(),
                "--exposure" => a.exposure = val().parse().ok(),
                "--quiet" => a.quiet = true,
                "--sound-mode" => a.sound_mode = match val().as_str() {
                    "realistic" => Some(crate::audio::SoundMode::Realistic),
                    "cinematic" => Some(crate::audio::SoundMode::Cinematic),
                    "hybrid" => Some(crate::audio::SoundMode::Hybrid),
                    _ => None,
                },
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
        if a.profile.is_some() && a.new.is_none() && a.load.is_none() && !a.continue_latest {
            a.new = Some(Scenario::SolarSystemLab);
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
