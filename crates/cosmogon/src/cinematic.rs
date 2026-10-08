//! Authored camera paths are presentation tools, never spacecraft trajectories.
use crate::{
    camera::{target_info, CameraRig},
    navigation::{Flight, Presentation},
    sim::{Sim, Target},
};
use bevy::math::{DQuat, DVec3};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Ease {
    #[default]
    Linear,
    Smoothstep,
}
impl Ease {
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear speed",
            Self::Smoothstep => "Ease in/out",
        }
    }
    fn fraction(self, v: f64) -> f64 {
        match self {
            Self::Linear => v,
            Self::Smoothstep => v * v * (3.0 - 2.0 * v),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keyframe {
    pub time_s: f64,
    pub position_m: [f64; 3],
    pub orientation_xyzw: [f64; 4],
    #[serde(deserialize_with = "strict_presentation")]
    pub presentation: Presentation,
    pub ease: Ease,
}
fn strict_presentation<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Presentation, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Lens {
        fov_deg: f32,
        exposure_ev: f32,
        warmth: f32,
        saturation: f32,
    }
    let l = Lens::deserialize(d)?;
    Ok(Presentation {
        fov_deg: l.fov_deg,
        exposure_ev: l.exposure_ev,
        warmth: l.warmth,
        saturation: l.saturation,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub format: String,
    pub schema_version: u32,
    pub name: String,
    pub world: String,
    /// One translating render-frame anchor. Body spin is not interpolated here.
    pub anchor: Option<Target>,
    pub look_at: Option<Target>,
    pub keys: Vec<Keyframe>,
}
#[derive(Clone, Debug)]
pub struct Pose {
    pub position: DVec3,
    pub rotation: DQuat,
    pub presentation: Presentation,
    pub time_s: f64,
    pub segment: usize,
}
pub fn state_fingerprint(u: &cosmogon_sim::Universe) -> u64 {
    serde_json::to_vec(u)
        .expect("Validated world serializes")
        .into_iter()
        .fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ b as u64).wrapping_mul(0x100000001b3)
        })
}
pub fn world_key(sim: &Sim) -> String {
    match &sim.session {
        crate::sim::Session::Sandbox(m) if m.template == "imported" => {
            format!("saved-state:{:016x}", sim.camera_source_fingerprint)
        }
        _ => crate::navigation::world_key(sim),
    }
}
fn object(sim: &Sim, target: Target) -> Result<DVec3, String> {
    let system = sim
        .universe
        .systems
        .get(target.system() as usize)
        .ok_or("Camera reference system is absent")?;
    if let Target::Body(r) = target {
        if !system
            .bodies
            .get(r.body as usize)
            .is_some_and(|b| b.exists())
        {
            return Err("Camera reference object is absent or destroyed".into());
        }
    }
    Ok(target_info(sim, target).0)
}
fn lens_valid(p: &Presentation) -> bool {
    [p.fov_deg, p.exposure_ev, p.warmth, p.saturation]
        .iter()
        .all(|v| v.is_finite())
        && (10.0..=110.0).contains(&p.fov_deg)
        && (-6.0..=6.0).contains(&p.exposure_ev)
        && (-0.03..=0.03).contains(&p.warmth)
        && (0.0..=2.0).contains(&p.saturation)
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "cosmogon-camera-path" || self.schema_version != 1 {
            return Err("Unsupported camera-path format/version; the original is preserved".into());
        }
        if self.name.trim().is_empty()
            || self.name.chars().count() > 80
            || self.world.is_empty()
            || self.world.len() > 256
            || !(2..=64).contains(&self.keys.len())
        {
            return Err("A path needs a name, world identity and 2–64 keys".into());
        }
        let mut previous = None;
        for key in &self.keys {
            let q = DQuat::from_array(key.orientation_xyzw);
            if !key.time_s.is_finite()
                || !(0.0..=3600.0).contains(&key.time_s)
                || previous.is_some_and(|t| key.time_s <= t)
                || !key
                    .position_m
                    .iter()
                    .all(|v| v.is_finite() && v.abs() < 1e28)
                || !q.is_finite()
                || (q.length_squared() - 1.0).abs() > 1e-4
                || !lens_valid(&key.presentation)
            {
                return Err("Keys require finite bounded poses/lenses and increasing presentation seconds (0–3600)".into());
            }
            previous = Some(key.time_s);
        }
        if self.keys[0].time_s != 0.0 {
            return Err("The first key starts at presentation time zero".into());
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 192 * 1024 {
            return Err("Path definitions must be regular files no larger than 192 KiB".into());
        }
        let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if !f.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Path definitions must be regular files".into());
        }
        let mut bytes = Vec::new();
        f.take(192 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 192 * 1024 {
            return Err("Camera path exceeds 192 KiB".into());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        value.validate()?;
        Ok(value)
    }
    pub fn export(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("Existing camera paths are protected; choose a new file: {e}"))?;
        f.write_all(&bytes)
            .map_err(|e| format!("Incomplete path export: {e}"))
    }
    pub fn duration(&self) -> f64 {
        self.keys.last().map(|k| k.time_s).unwrap_or(0.0)
    }
    pub fn context(&self, sim: &Sim) -> Result<(), String> {
        if self.world != world_key(sim) {
            return Err("This path belongs to another world".into());
        }
        if let Some(target) = self.anchor {
            object(sim, target)?;
        }
        if let Some(target) = self.look_at {
            object(sim, target)?;
        }
        Ok(())
    }
    pub fn pose(&self, sim: &Sim, time_s: f64) -> Result<Pose, String> {
        self.validate()?;
        self.context(sim)?;
        if !time_s.is_finite() {
            return Err("Presentation time must be finite".into());
        }
        let t = time_s.clamp(0.0, self.duration());
        let i = self
            .keys
            .partition_point(|k| k.time_s <= t)
            .saturating_sub(1)
            .min(self.keys.len() - 2);
        let a = &self.keys[i];
        let b = &self.keys[i + 1];
        let fraction = a
            .ease
            .fraction(((t - a.time_s) / (b.time_s - a.time_s)).clamp(0.0, 1.0));
        let p = if fraction == 0.0 {
            DVec3::from_array(a.position_m)
        } else if fraction == 1.0 {
            DVec3::from_array(b.position_m)
        } else {
            DVec3::from_array(a.position_m).lerp(DVec3::from_array(b.position_m), fraction)
        };
        let qa = DQuat::from_array(a.orientation_xyzw).normalize();
        let mut qb = DQuat::from_array(b.orientation_xyzw).normalize();
        // q and -q encode the same orientation. Always follow the shorter rotation.
        if qa.dot(qb) < 0.0 {
            qb = -qb;
        }
        let mut rotation = qa.slerp(qb, fraction).normalize();
        let position = p + self
            .anchor
            .map(|target| object(sim, target))
            .transpose()?
            .unwrap_or(DVec3::ZERO);
        if let Some(target) = self.look_at {
            let direction = object(sim, target)? - position;
            if !direction.is_finite() || direction.length_squared() < 1e-12 {
                return Err(
                    "Look-at target coincides with the camera; change that key or turn look-at off"
                        .into(),
                );
            }
            let forward = direction.normalize();
            let up = if forward.dot(DVec3::Y).abs() > 0.9999 {
                DVec3::X
            } else {
                DVec3::Y
            };
            // look_at_rh returns a view rotation; the camera needs its inverse.
            rotation = DQuat::look_to_rh(forward, up).inverse().normalize();
        }
        let blend = |a: f32, b: f32| ((a as f64) * (1.0 - fraction) + (b as f64) * fraction) as f32;
        let presentation = Presentation {
            fov_deg: blend(a.presentation.fov_deg, b.presentation.fov_deg),
            exposure_ev: blend(a.presentation.exposure_ev, b.presentation.exposure_ev),
            warmth: blend(a.presentation.warmth, b.presentation.warmth),
            saturation: blend(a.presentation.saturation, b.presentation.saturation),
        };
        Ok(Pose {
            position,
            rotation,
            presentation,
            time_s: t,
            segment: i,
        })
    }
}
impl Pose {
    pub fn apply(
        &self,
        rig: &mut CameraRig,
        presentation: &mut Presentation,
        anchor: Option<Target>,
    ) {
        rig.surface_anchor = None;
        rig.focus = anchor.or(rig.focus);
        rig.rotation = self.rotation;
        rig.free = Some(Flight::new(self.position, self.rotation));
        rig.navigation_notice = None;
        *presentation = self.presentation.clone();
    }
}

use bevy::prelude::*;

/// Playback owns only camera/presentation state. Drafts are saved only on explicit export.
#[derive(Resource)]
pub struct Player {
    pub draft: Option<Definition>,
    pub time_s: f64,
    pub active: bool,
    pub playing: bool,
    pub selected: usize,
    pub editor_tab: usize,
    pub spacing_s: f64,
    pub import_path: String,
    pub export_path: String,
    pub status: Option<String>,
    pub automatic_done: bool,
    generation: Option<u64>,
    pub evaluated: Option<Pose>,
    actual: Option<(DVec3, DQuat, [f32; 4])>,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            draft: None,
            time_s: 0.0,
            active: false,
            playing: false,
            selected: 0,
            editor_tab: 0,
            spacing_s: 8.0,
            import_path: String::new(),
            export_path: String::new(),
            status: None,
            automatic_done: false,
            generation: None,
            evaluated: None,
            actual: None,
        }
    }
}
fn lens(p: &Presentation) -> [f32; 4] {
    [p.fov_deg, p.exposure_ev, p.warmth, p.saturation]
}
impl Player {
    pub fn stop(&mut self, reason: &str) {
        self.playing = false;
        self.active = false;
        self.actual = None;
        self.status = Some(reason.into());
    }
    pub fn new_draft(&mut self, sim: &Sim, rig: &CameraRig) {
        self.stop("New path. Capture two or more views before playback/export.");
        self.time_s = 0.0;
        self.selected = 0;
        self.evaluated = None;
        self.draft = Some(Definition {
            format: "cosmogon-camera-path".into(),
            schema_version: 1,
            name: "Untitled path".into(),
            world: world_key(sim),
            anchor: rig.focus,
            look_at: None,
            keys: Vec::new(),
        });
        self.generation = Some(sim.generation);
    }
    pub fn append(
        &mut self,
        sim: &Sim,
        rig: &CameraRig,
        presentation: &Presentation,
    ) -> Result<(), String> {
        let mut d = self.draft.clone().ok_or("Create a path first")?;
        d.context(sim)?;
        if d.keys.len() >= 64 {
            return Err("A path supports at most 64 keys".into());
        }
        let base = d
            .anchor
            .map(|t| object(sim, t))
            .transpose()?
            .unwrap_or(DVec3::ZERO);
        let t = if d.keys.is_empty() {
            0.0
        } else {
            d.duration() + self.spacing_s
        };
        let key = Keyframe {
            time_s: t,
            position_m: (rig.eye() - base).to_array(),
            orientation_xyzw: rig.rotation.normalize().to_array(),
            presentation: presentation.clone(),
            ease: Ease::Linear,
        };
        d.keys.push(key);
        d.validate_draft()?;
        self.stop("View captured. Keys use one translating reference; body spin is not followed.");
        self.selected = d.keys.len() - 1;
        self.draft = Some(d);
        self.generation = Some(sim.generation);
        Ok(())
    }
    pub fn import(&mut self, path: &Path, sim: &Sim) -> Result<(), String> {
        let d = Definition::read(path)?;
        d.context(sim)?;
        // Validate geometric pose at the requested start before replacing the previous draft.
        d.pose(sim, 0.0)?;
        self.stop("Path opened. Press Play or scrub to move the camera.");
        self.time_s = 0.0;
        self.selected = 0;
        self.evaluated = None;
        self.draft = Some(d);
        self.generation = Some(sim.generation);
        Ok(())
    }
    pub fn scrub(
        &mut self,
        sim: &Sim,
        rig: &mut CameraRig,
        presentation: &mut Presentation,
        t: f64,
    ) -> Result<(), String> {
        let d = self.draft.as_ref().ok_or("Create or open a path first")?;
        let pose = d.pose(sim, t)?;
        let anchor = d.anchor;
        // A rejected evaluation leaves the previous camera, lens and phase untouched.
        pose.apply(rig, presentation, anchor);
        self.time_s = pose.time_s;
        self.evaluated = Some(pose);
        self.active = true;
        self.playing = false;
        self.generation = Some(sim.generation);
        self.actual = Some((rig.eye(), rig.rotation, lens(presentation)));
        self.status =
            Some("Preview held. Presentation time is independent of simulation time.".into());
        Ok(())
    }
    pub fn play(
        &mut self,
        sim: &Sim,
        rig: &mut CameraRig,
        presentation: &mut Presentation,
    ) -> Result<(), String> {
        let d = self.draft.as_ref().ok_or("Create or open a path first")?;
        let t = if self.time_s >= d.duration() {
            0.0
        } else {
            self.time_s
        };
        self.scrub(sim, rig, presentation, t)?;
        self.playing = true;
        self.status = Some("Playing · manual navigation stops the path.".into());
        Ok(())
    }
    fn external_change(&self, rig: &CameraRig, p: &Presentation) -> bool {
        self.actual.as_ref().is_some_and(|(pos, q, l)| {
            rig.free.is_none() || rig.eye() != *pos || rig.rotation != *q || lens(p) != *l
        })
    }
    pub fn advance(
        &mut self,
        sim: &Sim,
        rig: &mut CameraRig,
        p: &mut Presentation,
        dt: f64,
        suspended: bool,
    ) {
        if !self.active {
            return;
        }
        if self.generation != Some(sim.generation) {
            self.stop("World state changed; playback stopped. Review the path before restarting.");
            return;
        }
        if self.external_change(rig, p) {
            self.stop("Camera or lens changed; path control released.");
            return;
        }
        if suspended {
            return;
        }
        let t = if self.playing && dt.is_finite() && dt > 0.0 {
            self.time_s + dt
        } else {
            self.time_s
        };
        let result = self
            .draft
            .as_ref()
            .ok_or_else(|| "No active path".into())
            .and_then(|d| d.pose(sim, t));
        match result {
            Ok(pose) => {
                pose.apply(rig, p, self.draft.as_ref().and_then(|d| d.anchor));
                self.time_s = pose.time_s;
                if self
                    .draft
                    .as_ref()
                    .is_some_and(|d| self.time_s >= d.duration())
                    && self.playing
                {
                    self.playing = false;
                    self.status =
                        Some("End reached. Preview held; Stop releases camera control.".into());
                }
                self.evaluated = Some(pose);
                self.actual = Some((rig.eye(), rig.rotation, lens(p)));
            }
            Err(e) => self.stop(&format!("Path stopped: {e}")),
        }
    }
    pub fn metadata(&self, sim: &Sim, rig: &CameraRig) -> Option<serde_json::Value> {
        let d = self.draft.as_ref()?;
        Some(
            serde_json::json!({"definition":d,"editor_tab":self.editor_tab,"presentation_time_s":self.time_s,"simulation_time_s":sim.universe.time,"active":self.active,"playing":self.playing,"world_matches":d.world==world_key(sim),"generation_matches":self.generation==Some(sim.generation),"status":self.status,"evaluated":self.evaluated.as_ref().map(|p|serde_json::json!({"time_s":p.time_s,"segment":p.segment,"position_render_m":p.position.to_array(),"orientation_xyzw":p.rotation.to_array(),"presentation":p.presentation})),"actual_position_render_m":rig.eye().to_array(),"position_correction_m":self.evaluated.as_ref().map(|p|(rig.eye()-p.position).length()),"anchor_position_render_m":d.anchor.and_then(|t|object(sim,t).ok()).map(|p|p.to_array()),"look_at_position_render_m":d.look_at.and_then(|t|object(sim,t).ok()).map(|p|p.to_array()),"source_state_fingerprint":format!("{:016x}",sim.camera_source_fingerprint),"scope":"Authored presentation path; no physical trajectory, simulation rewind or automatic time-rate change. Ground clearance may correct the camera position."}),
        )
    }
}
impl Definition {
    /// Authoring permits fewer keys; playback/export requires a complete path.
    pub(crate) fn validate_draft(&self) -> Result<(), String> {
        if self.keys.len() >= 2 {
            return self.validate();
        }
        let mut d = self.clone();
        if let Some(first) = d.keys.first().cloned() {
            let mut b = first;
            b.time_s = 1.0;
            d.keys.push(b);
            d.validate()
        } else {
            Ok(())
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut player: ResMut<Player>,
    mut rig: ResMut<CameraRig>,
    mut settings: ResMut<crate::persistence::UserSettings>,
    sim: Res<Sim>,
    time: Res<Time<Real>>,
    ui: Res<crate::ui::UiState>,
    args: Res<crate::args::Args>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    if !player.automatic_done && rig.focus.is_some() {
        player.automatic_done = true;
        player.editor_tab = match args.panel.as_deref() {
            Some("paths-keys") => 1,
            Some("paths-files") => 2,
            _ => 0,
        };
        if let Some(path) = &args.camera_path {
            let result = player.import(Path::new(path), &sim).and_then(|_| {
                player.scrub(
                    &sim,
                    &mut rig,
                    &mut settings.presentation,
                    args.camera_path_time.unwrap_or(0.0),
                )
            });
            match result {
                Ok(()) => {
                    player.playing = args.camera_path_play;
                }
                Err(e) => {
                    error!("Cannot preview camera path: {e}");
                    player.status = Some(e);
                    if args.quit_after_capture {
                        exit.write(bevy::app::AppExit::error());
                    }
                }
            }
        }
    }
    player.advance(
        &sim,
        &mut rig,
        &mut settings.presentation,
        time.delta_secs_f64(),
        ui.pause_menu || ui.photo_mode,
    );
}
/// Record the actual post-clearance view so renderer corrections are not mistaken for input.
pub fn record_actual(
    mut player: ResMut<Player>,
    rig: Res<CameraRig>,
    settings: Res<crate::persistence::UserSettings>,
) {
    if player.active {
        player.actual = Some((rig.eye(), rig.rotation, lens(&settings.presentation)));
    }
}
pub fn leave_world(mut player: ResMut<Player>) {
    player.stop("World closed; path control released.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Session;
    fn sim() -> Sim {
        Sim::new(
            cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings {
                scenario: cosmogon_sim::Scenario::EmptySystem,
                ..Default::default()
            }),
            Session::Reference {
                dataset: "path-test".into(),
                title: "Test".into(),
            },
        )
    }
    fn definition(sim: &Sim) -> Definition {
        Definition {
            format: "cosmogon-camera-path".into(),
            schema_version: 1,
            name: "Actual authored views".into(),
            world: world_key(sim),
            anchor: None,
            look_at: None,
            keys: vec![
                Keyframe {
                    time_s: 0.0,
                    position_m: [10.0, 20.0, 30.0],
                    orientation_xyzw: DQuat::IDENTITY.to_array(),
                    presentation: Presentation::default(),
                    ease: Ease::Linear,
                },
                Keyframe {
                    time_s: 10.0,
                    position_m: [110.0, 220.0, 330.0],
                    orientation_xyzw: DQuat::from_rotation_y(1.0).to_array(),
                    presentation: Presentation {
                        fov_deg: 90.0,
                        exposure_ev: 2.0,
                        warmth: 0.02,
                        saturation: 0.5,
                    },
                    ease: Ease::Linear,
                },
            ],
        }
    }
    #[test]
    fn endpoints_easing_lens_and_antipodal_orientation_are_consistent() {
        let sim = sim();
        let mut d = definition(&sim);
        assert_eq!(
            d.pose(&sim, 0.0).unwrap().position.to_array(),
            d.keys[0].position_m
        );
        assert_eq!(
            d.pose(&sim, 10.0).unwrap().position.to_array(),
            d.keys[1].position_m
        );
        let m = d.pose(&sim, 5.0).unwrap();
        assert_eq!(m.position, DVec3::new(60.0, 120.0, 180.0));
        assert_eq!(m.presentation.fov_deg, 70.0);
        assert!((m.rotation.dot(DQuat::from_rotation_y(0.5)).abs() - 1.0).abs() < 1e-12);
        d.keys[0].ease = Ease::Smoothstep;
        assert!((d.pose(&sim, 2.5).unwrap().position.x - 25.625).abs() < 1e-12);
        d.keys[1].orientation_xyzw = (-DQuat::IDENTITY).to_array();
        assert_eq!(d.pose(&sim, 5.0).unwrap().rotation, DQuat::IDENTITY);
        d.keys[0].position_m = [1e27, -1e27, 1e27];
        d.keys[1].position_m = [-1e27, 1e27, -1e27];
        assert_eq!(
            d.pose(&sim, 10.0).unwrap().position.to_array(),
            d.keys[1].position_m
        );
        assert!(d.pose(&sim, 4.0).unwrap().position.is_finite());
    }
    #[test]
    fn look_at_uses_camera_negative_z_and_handles_poles_and_degeneracy() {
        let sim = sim();
        let mut d = definition(&sim);
        d.look_at = Some(Target::Star(0));
        for position in [[0.0, 0.0, 1e9], [0.0, 1e9, 0.0], [1e9, 0.0, 0.0]] {
            d.keys[0].position_m = position;
            let p = d.pose(&sim, 0.0).unwrap();
            let direction = (object(&sim, Target::Star(0)).unwrap() - p.position).normalize();
            assert!((p.rotation * DVec3::NEG_Z - direction).length() < 1e-10);
            assert!(p.rotation.is_normalized());
        }
        d.keys[0].position_m = object(&sim, Target::Star(0)).unwrap().to_array();
        assert!(d.pose(&sim, 0.0).is_err());
    }
    #[test]
    fn finite_pose_timing_bounds_and_schema_are_enforced() {
        let sim = sim();
        let d = definition(&sim);
        for field in 0..8 {
            let mut bad = d.clone();
            match field {
                0 => bad.keys[1].time_s = 0.0,
                1 => bad.keys[0].time_s = 1.0,
                2 => bad.keys[1].position_m[0] = f64::NAN,
                3 => bad.keys[1].orientation_xyzw = [0.0; 4],
                4 => bad.keys[1].presentation.fov_deg = 111.0,
                5 => bad.keys[1].time_s = 3601.0,
                6 => bad.schema_version = 99,
                _ => bad.keys.truncate(1),
            };
            assert!(bad.validate().is_err());
        }
        assert!(d.pose(&sim, f64::NAN).is_err());
        let mut v = serde_json::to_value(&d).unwrap();
        v["invented_field"] = true.into();
        assert!(serde_json::from_value::<Definition>(v).is_err());
    }
    #[test]
    fn rendering_editor_does_not_scrub_or_pause_the_live_clock() {
        let sim = sim();
        let mut player = Player { draft: Some(definition(&sim)), ..Default::default() };
        let mut rig = CameraRig::default();
        let mut p = Presentation::default();
        let views = crate::navigation::Viewpoints::default();
        player.play(&sim, &mut rig, &mut p).unwrap();
        let context = bevy_egui::egui::Context::default();
        for tab in 0..3 {
            player.editor_tab = tab;
            player.advance(&sim, &mut rig, &mut p, 0.01667891, false);
            let before = player.time_s;
            let _ = context.run(Default::default(), |ctx| {
                bevy_egui::egui::CentralPanel::default().show(ctx, |ui| {
                    crate::ui::cinematic::show(ui, &mut player, &sim, &mut rig, &mut p, &views);
                });
            });
            assert_eq!(player.time_s, before);
            assert!(player.active && player.playing);
        }
    }
    #[test]
    fn playback_pause_endpoint_external_input_and_generation_are_explicit() {
        let mut sim = sim();
        let before = serde_json::to_vec(&sim.universe).unwrap();
        let mut player = Player {
            draft: Some(definition(&sim)),
            ..Default::default()
        };
        let mut rig = CameraRig::default();
        let mut p = Presentation::default();
        player.play(&sim, &mut rig, &mut p).unwrap();
        player.advance(&sim, &mut rig, &mut p, 2.0, false);
        assert_eq!(player.time_s, 2.0);
        player.advance(&sim, &mut rig, &mut p, 3.0, true);
        assert_eq!(player.time_s, 2.0);
        player.playing = false;
        player.advance(&sim, &mut rig, &mut p, 3.0, false);
        assert_eq!(player.time_s, 2.0);
        player.play(&sim, &mut rig, &mut p).unwrap();
        player.advance(&sim, &mut rig, &mut p, 20.0, false);
        assert_eq!(player.time_s, 10.0);
        assert!(player.active && !player.playing);
        player.play(&sim, &mut rig, &mut p).unwrap();
        assert_eq!(player.time_s, 0.0);
        rig.free.as_mut().unwrap().position.x += 1.0;
        player.advance(&sim, &mut rig, &mut p, 1.0, false);
        assert!(!player.active);
        player.play(&sim, &mut rig, &mut p).unwrap();
        sim.generation += 1;
        player.advance(&sim, &mut rig, &mut p, 1.0, false);
        assert!(!player.active);
        assert_eq!(serde_json::to_vec(&sim.universe).unwrap(), before);
    }
    #[test]
    fn anchor_motion_is_read_only_missing_targets_do_not_fall_back() {
        let sim = sim();
        let mut d = definition(&sim);
        d.anchor = Some(Target::Star(0));
        let p = d.pose(&sim, 5.0).unwrap();
        assert_eq!(
            p.position,
            object(&sim, Target::Star(0)).unwrap() + DVec3::new(60.0, 120.0, 180.0)
        );
        d.anchor = Some(Target::Star(u32::MAX));
        assert!(d.pose(&sim, 0.0).is_err());
        d.anchor = None;
        d.world = "another world".into();
        assert!(d.pose(&sim, 0.0).is_err());
    }
    #[test]
    fn moving_and_destroyed_body_references_use_current_authoritative_positions() {
        let mut sim = Sim::new(
            cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings {
                scenario: cosmogon_sim::Scenario::SolarSystemLab,
                ..Default::default()
            }),
            Session::Reference {
                dataset: "moving-test".into(),
                title: "Test".into(),
            },
        );
        let target = crate::camera::find_target(&sim, "Mars").unwrap();
        let mut d = definition(&sim);
        d.anchor = Some(target);
        let before = d.pose(&sim, 5.0).unwrap();
        sim.universe.advance_by(120.0);
        let snapshot = serde_json::to_vec(&sim.universe).unwrap();
        let after = d.pose(&sim, 5.0).unwrap();
        assert!((after.position - before.position).length() > 1.0);
        assert_eq!(
            after.position,
            object(&sim, target).unwrap() + DVec3::new(60.0, 120.0, 180.0)
        );
        assert_eq!(serde_json::to_vec(&sim.universe).unwrap(), snapshot);
        if let Target::Body(r) = target {
            sim.universe.systems[r.system as usize].bodies[r.body as usize].removed =
                Some(cosmogon_sim::astro::Removal {
                    time: sim.universe.time,
                    cause: cosmogon_sim::astro::RemovalCause::Deleted,
                });
        } else {
            panic!("Mars is a body");
        }
        assert!(d.pose(&sim, 5.0).is_err());
    }
    #[test]
    fn authoring_from_actual_views_and_rejected_scrub_preserve_source() {
        let sim = sim();
        let before = serde_json::to_vec(&sim.universe).unwrap();
        let mut rig = CameraRig::default();
        rig.free = Some(Flight::new(DVec3::new(0.0, 0.0, 1e9), DQuat::IDENTITY));
        let mut p = Presentation::default();
        let mut player = Player::default();
        player.new_draft(&sim, &rig);
        player.append(&sim, &rig, &p).unwrap();
        assert!(player.draft.as_ref().unwrap().validate().is_err());
        rig.free.as_mut().unwrap().position.x = 1e8;
        player.append(&sim, &rig, &p).unwrap();
        let d = player.draft.as_ref().unwrap();
        assert_eq!(d.keys[1].position_m[0], 1e8);
        assert_eq!(d.duration(), 8.0);
        let eye = rig.eye();
        let original = lens(&p);
        assert!(player.scrub(&sim, &mut rig, &mut p, f64::NAN).is_err());
        assert_eq!(rig.eye(), eye);
        assert_eq!(lens(&p), original);
        assert_eq!(serde_json::to_vec(&sim.universe).unwrap(), before);
    }
    #[test]
    fn protected_bounded_exports_and_transactional_imports() {
        let sim = sim();
        let d = definition(&sim);
        let dir = std::env::temp_dir().join(format!(
            "cosmogon-path-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("shot.json");
        d.export(&path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(d.export(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let loaded = Definition::read(&path).unwrap();
        assert_eq!(
            serde_json::to_vec(&loaded).unwrap(),
            serde_json::to_vec(&d).unwrap()
        );
        let mut player = Player::default();
        player.import(&path, &sim).unwrap();
        let previous = serde_json::to_vec(player.draft.as_ref().unwrap()).unwrap();
        let bad = dir.join("future.json");
        let mut future = d.clone();
        future.schema_version = 99;
        std::fs::write(&bad, serde_json::to_vec(&future).unwrap()).unwrap();
        assert!(player.import(&bad, &sim).is_err());
        assert_eq!(
            serde_json::to_vec(player.draft.as_ref().unwrap()).unwrap(),
            previous
        );
        std::fs::write(&bad, vec![b' '; 192 * 1024 + 1]).unwrap();
        assert!(Definition::read(&bad).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn imported_source_identity_survives_session_ids_but_distinguishes_states() {
        let u = sim().universe;
        let a = Sim::new(
            u.clone(),
            Session::Sandbox(crate::persistence::Manifest::new(
                "A",
                "",
                "imported",
                Default::default(),
            )),
        );
        let b = Sim::new(
            u.clone(),
            Session::Sandbox(crate::persistence::Manifest::new(
                "B",
                "",
                "imported",
                Default::default(),
            )),
        );
        assert_eq!(world_key(&a), world_key(&b));
        let mut changed = u;
        changed.time += 1.0;
        let c = Sim::new(
            changed,
            Session::Sandbox(crate::persistence::Manifest::new(
                "C",
                "",
                "imported",
                Default::default(),
            )),
        );
        assert_ne!(world_key(&a), world_key(&c));
    }
}
