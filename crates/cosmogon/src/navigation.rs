//! Native navigation and presentation state. None of this is serialized into Universe.
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::camera::{find_target, target_info, CameraRig};
use crate::sim::{Session, Sim};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraControls {
    pub sensitivity: f64,
    pub invert_y: bool,
    pub inertia: bool,
    pub damping: f64,
    pub auto_speed: bool,
    pub speed_multiplier: f64,
    pub manual_speed_m_s: f64,
}
impl Default for CameraControls {
    fn default() -> Self {
        Self { sensitivity: 1.0, invert_y: false, inertia: false, damping: 8.0, auto_speed: true, speed_multiplier: 1.0, manual_speed_m_s: 1000.0 }
    }
}
impl CameraControls {
    pub fn sanitize(&mut self) {
        self.sensitivity = finite(self.sensitivity, 1.0).clamp(0.1, 5.0);
        self.damping = finite(self.damping, 8.0).clamp(0.2, 30.0);
        self.speed_multiplier = finite(self.speed_multiplier, 1.0).clamp(0.001, 1000.0);
        self.manual_speed_m_s = finite(self.manual_speed_m_s, 1000.0).clamp(0.01, 1e22);
    }
    pub fn speed(&self, clearance: f64) -> f64 {
        let base = if self.auto_speed { clearance.max(0.25) * 0.2 } else { self.manual_speed_m_s };
        (base * self.speed_multiplier).clamp(0.01, 1e22)
    }
}
fn finite(v: f64, fallback: f64) -> f64 { if v.is_finite() { v } else { fallback } }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Presentation {
    pub fov_deg: f32,
    pub exposure_ev: f32,
    /// CIE chromaticity adjustment, deliberately not a claimed Kelvin temperature.
    pub warmth: f32,
    pub saturation: f32,
}
impl Default for Presentation {
    fn default() -> Self { Self { fov_deg: 50.0, exposure_ev: 0.0, warmth: 0.0, saturation: 1.0 } }
}
impl Presentation {
    pub fn sanitize(&mut self) {
        self.fov_deg = finite(self.fov_deg as f64, 50.0).clamp(10.0, 110.0) as f32;
        self.exposure_ev = finite(self.exposure_ev as f64, 0.0).clamp(-6.0, 6.0) as f32;
        self.warmth = finite(self.warmth as f64, 0.0).clamp(-0.03, 0.03) as f32;
        self.saturation = finite(self.saturation as f64, 1.0).clamp(0.0, 2.0) as f32;
    }
}

#[derive(Clone, Debug)]
pub struct Flight {
    pub position: DVec3,
    pub rotation: DQuat,
    pub velocity: DVec3,
    pub speed_m_s: f64,
}
impl Flight {
    pub fn new(position: DVec3, rotation: DQuat) -> Self {
        Self { position, rotation: rotation.normalize(), velocity: DVec3::ZERO, speed_m_s: 0.0 }
    }
    /// Exact integration of dv/dt = damping * (desired - v), independent of frame subdivision.
    pub fn advance(&mut self, desired: DVec3, dt: f64, inertia: bool, damping: f64) {
        if !dt.is_finite() || dt <= 0.0 || !desired.is_finite() { return; }
        if inertia {
            let damping = finite(damping, 8.0).clamp(0.2, 30.0);
            let decay = (-damping * dt).exp();
            self.position += desired * dt + (self.velocity - desired) * ((1.0 - decay) / damping);
            self.velocity = desired + (self.velocity - desired) * decay;
        } else {
            self.velocity = desired;
            self.position += desired * dt;
        }
    }
    pub fn turn(&mut self, yaw: f64, pitch: f64, roll: f64) {
        self.rotation = (self.rotation * DQuat::from_rotation_y(yaw) * DQuat::from_rotation_x(pitch) * DQuat::from_rotation_z(roll)).normalize();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Viewpoint {
    pub name: String,
    pub world: String,
    pub anchor: Option<String>,
    #[serde(default)]
    pub anchor_target: Option<crate::sim::Target>,
    #[serde(default)]
    pub surface_anchor: Option<crate::surface::SurfaceAnchor>,
    /// Render-frame metres relative to anchor, or absolute when anchor is absent.
    pub position_m: [f64; 3],
    pub orientation_xyzw: [f64; 4],
    pub presentation: Presentation,
    pub simulation_time_s: f64,
}
pub(crate) fn world_key(sim: &Sim) -> String {
    match &sim.session {
        Session::Reference { dataset, .. } => format!("reference:{dataset}"),
        Session::Sandbox(m) if m.template != "dev" => format!("sandbox:{}", m.id),
        _ => format!("dev:{}:{}", sim.universe.settings.scenario.label(), sim.universe.settings.seed),
    }
}
impl Viewpoint {
    pub fn capture(name: &str, sim: &Sim, rig: &CameraRig, presentation: &Presentation) -> Self {
        let anchor = rig.focus.map(|t| match t {
            crate::sim::Target::Star(s) => sim.universe.system(s).star.name.clone(),
            crate::sim::Target::Body(r) => sim.universe.body(r).name.clone(),
        });
        let base = rig.focus.map(|t| target_info(sim, t).0).unwrap_or(DVec3::ZERO);
        Self { name: name.trim().chars().take(80).collect(), world: world_key(sim), anchor, anchor_target: rig.focus, surface_anchor: rig.surface_anchor.clone(), position_m: (rig.eye() - base).to_array(), orientation_xyzw: rig.rotation.to_array(), presentation: presentation.clone(), simulation_time_s: sim.universe.time }
    }
    pub fn belongs_to(&self, sim: &Sim) -> bool { self.world == world_key(sim) }
    fn valid(&self) -> bool {
        let q = DQuat::from_array(self.orientation_xyzw);
        !self.name.trim().is_empty() && self.name.chars().count() <= 80 && self.surface_anchor.as_ref().is_none_or(|a|a.valid() && self.anchor_target == Some(crate::sim::Target::Body(a.body))) && self.position_m.iter().all(|v| v.is_finite() && v.abs() < 1e28) && q.is_finite() && (q.length_squared() - 1.0).abs() < 0.01 && self.simulation_time_s.is_finite()
    }
    pub fn restore(&self, sim: &Sim, rig: &mut CameraRig, presentation: &mut Presentation) -> Result<(), String> {
        if !self.valid() || !self.belongs_to(sim) { return Err("This viewpoint is invalid or belongs to another universe.".into()); }
        let focus = if let Some(target) = self.anchor_target {
            let valid = sim.universe.systems.get(target.system() as usize).is_some_and(|sys| match target {
                crate::sim::Target::Star(_) => true,
                crate::sim::Target::Body(r) => sys.bodies.get(r.body as usize).is_some_and(|b| b.exists()),
            });
            if !valid { return Err("The viewpoint's reference object no longer exists.".into()); }
            Some(target)
        } else { match &self.anchor {
            Some(name) => Some(find_target(sim, name).ok_or("The viewpoint's reference object no longer exists.")?),
            None => None,
        } };
        let base = focus.map(|t| target_info(sim, t).0).unwrap_or(DVec3::ZERO);
        let pos = base + DVec3::from_array(self.position_m);
        // Validate terrain/source availability before changing any camera or lens state.
        let surface_pose = self.surface_anchor.as_ref().map(|a|a.pose(sim)).transpose()?;
        rig.focus = focus.or(rig.focus);
        rig.rotation = DQuat::from_array(self.orientation_xyzw).normalize();
        rig.free = Some(Flight::new(pos, rig.rotation));
        rig.surface_anchor = self.surface_anchor.clone();
        if let Some(a) = &self.surface_anchor {
            if a.tracked {
                let flight = surface_pose.expect("validated surface pose");
                rig.rotation = flight.rotation; rig.free = Some(flight);
            }
        }
        rig.navigation_notice = None;
        *presentation = self.presentation.clone();
        presentation.sanitize();
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct ViewpointFile { version: u32, viewpoints: Vec<Viewpoint> }
#[derive(Resource)]
pub struct Viewpoints {
    pub items: Vec<Viewpoint>,
    pub status: Option<String>,
    pub writable: bool,
    path: PathBuf,
}
impl Default for Viewpoints {
    fn default() -> Self { Self::load(crate::persistence::data_dir().join("viewpoints.json")) }
}
impl Viewpoints {
    fn load(path: PathBuf) -> Self {
        let loaded = match std::fs::read(&path) {
            Ok(data) => serde_json::from_slice::<ViewpointFile>(&data).map_err(|e| e.to_string()).and_then(|file| {
                if !matches!(file.version, 1 | 2) || file.viewpoints.len() > 128 || file.viewpoints.iter().any(|v| !v.valid()) || (file.version == 1 && file.viewpoints.iter().any(|v|v.surface_anchor.is_some())) {
                    Err("Unsupported or invalid viewpoint library; the original file has been preserved.".into())
                } else { Ok(file.viewpoints) }
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.to_string()),
        };
        match loaded {
            Ok(items) => Self { items, status: None, writable: true, path },
            Err(e) => Self { items: Vec::new(), status: Some(format!("Could not load viewpoints: {e}")), writable: false, path },
        }
    }
    pub fn add(&mut self, value: Viewpoint) -> Result<(), String> {
        if !value.valid() { return Err("Give the viewpoint a name and a valid camera position.".into()); }
        if self.items.len() >= 128 { return Err("The library holds up to 128 viewpoints. Remove one first.".into()); }
        let mut next = self.items.clone(); next.push(value); self.commit(next)
    }
    pub fn remove(&mut self, index: usize) -> Result<(), String> {
        let mut next = self.items.clone();
        if index >= next.len() { return Err("Viewpoint no longer exists.".into()); }
        next.remove(index); self.commit(next)
    }
    fn commit(&mut self, next: Vec<Viewpoint>) -> Result<(), String> {
        if !self.writable { return Err("The unreadable library is protected from overwriting.".into()); }
        write_library(&self.path, &next)?;
        self.items = next;
        Ok(())
    }
}
fn write_library(path: &Path, values: &[Viewpoint]) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(&ViewpointFile { version: 2, viewpoints: values.to_vec() }).map_err(|e| e.to_string())?;
    let parent = path.parent().ok_or("Invalid viewpoint folder")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inertial_flight_is_independent_of_frame_subdivision() {
        let mut a = Flight::new(DVec3::ZERO, DQuat::IDENTITY);
        let mut b = a.clone();
        let desired = DVec3::new(23.0, -12.0, 78.0);
        a.advance(desired, 1.0, true, 8.0);
        for _ in 0..120 { b.advance(desired, 1.0 / 120.0, true, 8.0); }
        assert!((a.position - b.position).length() < 1e-10);
        assert!((a.velocity - b.velocity).length() < 1e-10);
    }
    #[test]
    fn rotation_and_motion_remain_finite_across_scales() {
        for scale in [1.0, 6.4e6, 1.5e11, 1e21] {
            let mut f = Flight::new(DVec3::splat(scale), DQuat::IDENTITY);
            for _ in 0..1000 { f.turn(0.002, -0.003, 0.005); }
            f.advance(f.rotation * DVec3::NEG_Z * scale * 0.01, 0.016, false, 8.0);
            assert!(f.position.is_finite());
            assert!((f.rotation.length() - 1.0).abs() < 1e-12);
            assert!((f.position - DVec3::splat(scale)).length() > 0.0);
        }
    }
    #[test]
    fn settings_reject_nonfinite_and_extreme_values() {
        let mut p = Presentation { fov_deg: f32::NAN, exposure_ev: f32::INFINITY, warmth: 200.0, saturation: -9.0 };
        p.sanitize(); assert_eq!(p.fov_deg, 50.0); assert_eq!(p.exposure_ev, 0.0); assert_eq!(p.saturation, 0.0);
        let mut c = CameraControls { speed_multiplier: f64::NAN, ..Default::default() };
        c.sanitize(); assert!(c.speed(1e22).is_finite()); assert!(c.speed(0.0) > 0.0);
    }
    #[test]
    fn viewpoint_library_round_trips_and_protects_unknown_versions() {
        let path = std::env::temp_dir().join(format!("cosmogon-viewpoints-{}.json", std::process::id()));
        let mut lib = Viewpoints { items: Vec::new(), status: None, writable: true, path: path.clone() };
        let v = Viewpoint { name: "Ring sunrise".into(), world: "test".into(), anchor: None, anchor_target: None, surface_anchor: None, position_m: [1.0, 2.0, 3.0], orientation_xyzw: [0.0, 0.0, 0.0, 1.0], presentation: Presentation::default(), simulation_time_s: 42.0 };
        // Version 1 omits geographic anchors and still loads.
        let mut old = serde_json::to_value(&v).unwrap(); old.as_object_mut().unwrap().remove("surface_anchor");
        std::fs::write(&path,serde_json::to_vec(&serde_json::json!({"version":1,"viewpoints":[old]})).unwrap()).unwrap();
        assert!(Viewpoints::load(path.clone()).writable);
        assert!(Viewpoints::load(path.clone()).items[0].surface_anchor.is_none());
        lib.add(v.clone()).unwrap();
        assert_eq!(Viewpoints::load(path.clone()).items[0].position_m, v.position_m);
        lib.remove(0).unwrap(); assert!(Viewpoints::load(path.clone()).items.is_empty());
        std::fs::write(&path, br#"{"version":999,"viewpoints":[]}"#).unwrap();
        let mut unknown = Viewpoints::load(path.clone());
        assert!(!unknown.writable); assert!(unknown.add(v).is_err());
        assert!(std::fs::read_to_string(&path).unwrap().contains("999"));
        std::fs::remove_file(path).unwrap();
    }
}
