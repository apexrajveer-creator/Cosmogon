//! Body-fixed exploration state, separate from physical state. Travel is a camera operation.
use bevy::math::{DMat3, DQuat, DVec3};
use cosmogon_sim::{BodyRef, planet::{landing, mola, terrain::{dir_from_lat_lon, ElevationData}}};
use serde::{Deserialize, Serialize};
use crate::{navigation::Flight, render::{body_rotation_f64, to_render}, sim::Sim};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceAnchor {
    pub body: BodyRef,
    pub latitude_deg: f64,
    pub longitude_east_deg: f64,
    /// Radial distance above the shared MOLA height on the app's spherical base.
    pub altitude_m: f64,
    pub site_id: Option<String>,
    pub terrain_dataset: String,
    pub tracked: bool,
    /// Camera orientation in the body's local frame, including subsequent look adjustments.
    pub local_orientation_xyzw: [f64; 4],
}

impl SurfaceAnchor {
    pub fn new(body: BodyRef, latitude: f64, longitude: f64, altitude: f64, site: Option<String>, tracked: bool) -> Result<Self, String> {
        mola::read(latitude, longitude)?;
        if !altitude.is_finite() || !(60.0..=1.0e8).contains(&altitude) {
            return Err("Choose an altitude between 60 m and 100,000 km above the shared terrain.".into());
        }
        let local = DVec3::from_array(dir_from_lat_lon(latitude.to_radians(), longitude.to_radians()));
        // North tangent is explicit even at the poles; avoid a parallel look/up vector.
        let (sl, cl) = latitude.to_radians().sin_cos();
        let (so, co) = longitude.to_radians().sin_cos();
        let north = DVec3::new(-sl * co, -sl * so, cl);
        // A shallow overview looks toward the ground and north; the probe remains in view.
        let forward = (-local * 0.985 + north * 0.173).normalize();
        let right = forward.cross(local).normalize();
        let up = right.cross(forward).normalize();
        let orientation = DQuat::from_mat3(&DMat3::from_cols(right, up, -forward)).normalize();
        let anchor = Self { body, latitude_deg: latitude, longitude_east_deg: longitude.rem_euclid(360.0), altitude_m: altitude, site_id: site, terrain_dataset: mola::DATASET.into(), tracked, local_orientation_xyzw: orientation.to_array() };
        if !anchor.valid() { return Err("Invalid surface anchor or mismatched landing reference.".into()); }
        Ok(anchor)
    }

    pub fn horizon(mut self) -> Self {
        let local=DVec3::from_array(dir_from_lat_lon(self.latitude_deg.to_radians(),self.longitude_east_deg.to_radians()));
        let (sl,cl)=self.latitude_deg.to_radians().sin_cos();
        let (so,co)=self.longitude_east_deg.to_radians().sin_cos();
        let north=DVec3::new(-sl*co,-sl*so,cl);
        let forward=(-local*0.342+north*0.940).normalize();
        let right=forward.cross(local).normalize();
        self.local_orientation_xyzw=DQuat::from_mat3(&DMat3::from_cols(right,right.cross(forward).normalize(),-forward)).normalize().to_array();
        self
    }

    pub fn valid(&self) -> bool {
        let q = DQuat::from_array(self.local_orientation_xyzw);
        self.terrain_dataset == mola::DATASET && self.latitude_deg.is_finite() && (-90.0..=90.0).contains(&self.latitude_deg)
            && self.longitude_east_deg.is_finite() && (0.0..360.0).contains(&self.longitude_east_deg)
            && self.altitude_m.is_finite() && (60.0..=1.0e8).contains(&self.altitude_m)
            && q.is_finite() && (q.length_squared() - 1.0).abs() < 0.01
            && self.site_id.as_deref().is_none_or(|id| landing::find(id).is_some_and(|s| s.latitude_north_deg == self.latitude_deg && s.longitude_east_deg == self.longitude_east_deg))
    }

    pub fn pose(&self, sim: &Sim) -> Result<Flight, String> {
        if !self.valid() { return Err("Invalid or unsupported surface anchor.".into()); }
        let body = sim.universe.systems.get(self.body.system as usize).and_then(|s|s.bodies.get(self.body.body as usize))
            .filter(|b|b.exists()).ok_or("Surface reference no longer exists; tracking released.")?;
        if body.elevation_data != Some(ElevationData::MarsMola) || !body.name.eq_ignore_ascii_case("Mars") {
            return Err("Surface travel requires Mars with observed MOLA terrain; this world's stored terrain is preserved.".into());
        }
        let local = DVec3::from_array(dir_from_lat_lon(self.latitude_deg.to_radians(), self.longitude_east_deg.to_radians()));
        let radius = body.radius + mola::elevation_m(local.to_array());
        let frame = body_rotation_f64(sim, self.body, sim.universe.time);
        let center = to_render(sim.universe.body_position(self.body, sim.universe.time));
        Ok(Flight::new(center + frame * local * (radius + self.altitude_m), frame * DQuat::from_array(self.local_orientation_xyzw)))
    }
}

/// Conservative line-of-sight on the spherical overview. No claim of local terrain occlusion.
pub fn above_horizon(eye_from_center: DVec3, radial: DVec3, radius: f64) -> bool {
    eye_from_center.dot(radial) > radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{camera::{find_target, target_info, CameraRig}, navigation::{Presentation, Viewpoint}, sim::{Session, Target}};
    fn scene() -> (Sim, BodyRef) {
        let sim = Sim::new(cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings { scenario: cosmogon_sim::Scenario::SolarSystemLab, ..Default::default() }), Session::Reference { dataset: "surface-test".into(), title: "Test".into() });
        let Target::Body(r) = find_target(&sim, "Mars").unwrap() else { panic!() };
        (sim, r)
    }
    #[test]
    fn surface_pose_matches_shared_height_at_seams_poles_and_new_times() {
        let (mut sim, r) = scene();
        let before = serde_json::to_string(&sim.universe).unwrap();
        for (lat, lon) in [(18.44463,77.45088), (0.0,0.0), (0.0,360.0), (-90.0,-180.0), (90.0,270.0)] {
            let anchor = SurfaceAnchor::new(r, lat, lon, 1000.0, None, true).unwrap();
            let pose = anchor.pose(&sim).unwrap();
            let frame = body_rotation_f64(&sim,r,sim.universe.time);
            let local = frame.inverse() * (pose.position - target_info(&sim,Target::Body(r)).0);
            let expected = DVec3::from_array(dir_from_lat_lon(lat.to_radians(),lon.to_radians()));
            assert!((local.normalize() - expected).length() < 1e-10);
            assert!((local.length() - sim.universe.body(r).radius - mola::elevation_m(expected.to_array()) - 1000.0).abs() < 0.001);
            assert!(pose.rotation.is_finite());
            assert!((pose.rotation.length() - 1.0).abs() < 1e-12);
            // The probe sits within the default 50-degree FOV with room above the dock.
            assert!((DQuat::from_array(anchor.local_orientation_xyzw)*DVec3::NEG_Z).dot(-expected)>0.98);
            assert!(anchor.clone().horizon().pose(&sim).unwrap().rotation.is_finite());
        }
        assert_eq!(before,serde_json::to_string(&sim.universe).unwrap());
        let a = SurfaceAnchor::new(r, 18.44463,77.45088,100_000.0,Some("perseverance".into()),true).unwrap();
        let first = a.pose(&sim).unwrap();
        sim.universe.time += 10_000.0;
        let later = a.pose(&sim).unwrap();
        assert!((later.position - first.position).length() > 1e6);
        let local = body_rotation_f64(&sim,r,sim.universe.time).inverse() * (later.position - target_info(&sim,Target::Body(r)).0);
        assert!((local.normalize() - DVec3::from_array(dir_from_lat_lon(a.latitude_deg.to_radians(),a.longitude_east_deg.to_radians()))).length() < 1e-10);
    }
    #[test]
    fn tracked_viewpoint_restores_geography_and_lens_without_rewinding() {
        let (mut sim,r) = scene();
        let a = SurfaceAnchor::new(r,-4.59,137.44,500_000.0,Some("curiosity".into()),true).unwrap();
        let mut rig = CameraRig::default(); rig.travel_surface(a.clone(),&sim).unwrap();
        let p = Presentation { fov_deg:30.0,..Default::default() };
        let v = Viewpoint::capture("Gale",&sim,&rig,&p);
        let v:Viewpoint = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
        sim.universe.time += 5000.0;
        let before = serde_json::to_string(&sim.universe).unwrap();
        let mut restored = CameraRig::default(); let mut lens = Presentation::default();
        v.restore(&sim,&mut restored,&mut lens).unwrap();
        assert!((restored.eye() - a.pose(&sim).unwrap().position).length() < 1e-6);
        assert_eq!(restored.surface_anchor.as_ref().unwrap().site_id.as_deref(),Some("curiosity"));
        let mut ui=crate::ui::UiState::default();ui.show_surface_anchor(restored.surface_anchor.as_ref().unwrap());
        assert_eq!(ui.surface_site.as_deref(),Some("curiosity"));assert_eq!(ui.surface_latitude,-4.59);assert_eq!(ui.surface_longitude,137.44);
        assert_eq!(lens.fov_deg,30.0);
        assert_eq!(before,serde_json::to_string(&sim.universe).unwrap());
        assert!(SurfaceAnchor::new(r,0.0,0.0,1000.0,Some("curiosity".into()),true).is_err());
        assert!(SurfaceAnchor::new(r,0.0,0.0,59.0,None,true).is_err());
        sim.universe.systems[r.system as usize].bodies[r.body as usize].elevation_data=None;
        assert!(a.pose(&sim).is_err());
        assert!(v.restore(&sim,&mut restored,&mut lens).is_err());
    }
    #[test]
    fn markers_hide_beyond_the_geometric_horizon() {
        assert!(above_horizon(DVec3::Z*1100.0,DVec3::Z,1000.0));
        assert!(!above_horizon(DVec3::Z*1100.0,DVec3::NEG_Z,1000.0));
        // Facing hemisphere alone is insufficient at close range.
        assert!(!above_horizon(DVec3::Z*1100.0,DVec3::new(0.8,0.0,0.6),1000.0));
    }
    #[test]
    fn snapshot_release_and_missing_body_keep_a_finite_world_pose() {
        let (mut sim,r)=scene();
        let a=SurfaceAnchor::new(r,22.27,312.05,100_000.0,Some("viking-1".into()),false).unwrap();
        let mut rig=CameraRig::default();rig.travel_surface(a,&sim).unwrap();
        let eye=rig.eye();let orientation=rig.rotation;
        sim.universe.time+=1000.0;rig.follow_surface(&sim);
        assert_eq!(rig.eye(),eye);assert_eq!(rig.rotation,orientation);
        let a=SurfaceAnchor::new(r,0.0,0.0,100_000.0,None,true).unwrap();
        rig.travel_surface(a,&sim).unwrap();
        rig.free.as_mut().unwrap().velocity=DVec3::splat(1000.0);
        rig.release_surface();assert_eq!(rig.free.as_ref().unwrap().velocity,DVec3::ZERO);
        let released=rig.eye();sim.universe.time+=1000.0;rig.follow_surface(&sim);assert_eq!(rig.eye(),released);
        let a=SurfaceAnchor::new(r,0.0,0.0,100_000.0,None,true).unwrap();rig.travel_surface(a,&sim).unwrap();
        let placed=rig.eye();
        sim.universe.systems[r.system as usize].bodies[r.body as usize].removed=Some(cosmogon_sim::astro::body::Removal {time:sim.universe.time,cause:cosmogon_sim::astro::body::RemovalCause::Deleted});
        rig.follow_surface(&sim);assert!(rig.surface_anchor.is_none());assert_eq!(rig.eye(),placed);assert!(rig.rotation.is_finite());
        assert!(rig.navigation_notice.as_ref().unwrap().contains("no longer exists"));
    }
}
