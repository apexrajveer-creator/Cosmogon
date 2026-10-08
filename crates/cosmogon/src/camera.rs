//! Astronomical camera: orbit / zoom / track / cinematic focus transitions, all in `f64`.
//!
//! The camera's absolute position becomes the floating origin each frame (see `render`).
//! Zoom is exponential and the near plane follows the distance to the nearest surface, so
//! the same controls work from a few hundred metres above a planet to tens of light-years.

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::Skybox;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::math::{DQuat, DVec3};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, Hdr};
use bevy_egui::EguiContexts;
use cosmogon_sim::astro::{AU, LIGHT_YEAR};
use cosmogon_sim::BodyRef;

use crate::args::Args;
use crate::persistence::UserSettings;
use crate::render::{to_render, SkyHandle, ViewInfo};
use crate::sim::{Sim, Target};
use crate::state::{AppState, Frame};

#[derive(Component)]
pub struct MainCamera;

struct Transition {
    from_pos: DVec3,
    from_dist: f64,
    elapsed: f32,
    duration: f32,
    from_rotation: DQuat,
}

#[derive(Resource)]
pub struct CameraRig {
    pub focus: Option<Target>,
    pub free: Option<crate::navigation::Flight>,
    pub surface_anchor: Option<crate::surface::SurfaceAnchor>,
    pub navigation_notice: Option<String>,
    pub rotation: DQuat,
    pub nearest_surface: Option<BodyRef>,
    pub clearance: f64,
    start_free: bool,
    pub target_pos: DVec3,
    pub distance: f64,
    pub desired_distance: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub focus_radius: f64,
    /// Radius of the ground under the camera (terrain-aware when close-up terrain is active).
    pub ground_radius: f64,
    transition: Option<Transition>,
    pub applied_args: bool,
    /// Offset of the orbit pivot from the target: zoomed far out, the camera swings round the
    /// Galactic Centre instead of the focused star.
    pub pivot_shift: DVec3,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self { focus: None, free: None, surface_anchor: None, navigation_notice: None, rotation: DQuat::IDENTITY, nearest_surface: None, clearance: 1.0e7, start_free: false, target_pos: DVec3::ZERO, distance: 3.0e7, desired_distance: 3.0e7, yaw: 0.6, pitch: 0.35, focus_radius: 6.4e6, ground_radius: 6.4e6, transition: None, applied_args: false, pivot_shift: DVec3::ZERO }
    }
}

impl CameraRig {
    /// Fly to a new target. `distance_radii`: how far to end up, in target radii (None = keep a sensible default).
    pub fn focus_on(&mut self, target: Target, sim: &Sim, distance_radii: Option<f64>) {
        self.surface_anchor = None;
        self.navigation_notice = None;
        let (pos, radius) = target_info(sim, target);
        if let Some(free) = self.free.take() {
            let back = free.rotation * DVec3::Z;
            self.yaw = back.x.atan2(back.z);
            self.pitch = back.y.clamp(-1.0, 1.0).asin();
            self.distance = (free.position - pos).length().max(radius * 1.5);
            self.target_pos = free.position - back * self.distance;
            self.pivot_shift = DVec3::ZERO;
            self.rotation = free.rotation;
        }
        let default_radii = match target {
            Target::Star(_) => system_view_distance(sim, target.system()) / radius,
            Target::Body(_) => 4.0,
        };
        self.transition = Some(Transition { from_pos: self.target_pos, from_dist: self.distance, elapsed: 0.0, duration: 1.8, from_rotation: self.rotation });
        self.focus = Some(target);
        self.desired_distance = radius * distance_radii.unwrap_or(default_radii);
    }

    pub fn detach(&mut self) {
        if self.free.is_none() {
            self.free = Some(crate::navigation::Flight::new(self.eye(), self.rotation));
            self.transition = None;
        }
    }

    pub fn travel_surface(&mut self, anchor: crate::surface::SurfaceAnchor, sim: &Sim) -> Result<(), String> {
        let flight = anchor.pose(sim)?;
        self.rotation = flight.rotation;
        self.free = Some(flight);
        self.focus = Some(Target::Body(anchor.body));
        self.nearest_surface = Some(anchor.body);
        self.clearance = anchor.altitude_m;
        self.transition = None;
        self.start_free = false;
        self.navigation_notice = Some(if anchor.tracked { "Following the surface. Movement releases tracking; looking around keeps it." } else { "Snapshot view placed. The camera does not follow the planet's motion or spin." }.into());
        self.surface_anchor = Some(anchor);
        Ok(())
    }

    pub fn release_surface(&mut self) {
        self.surface_anchor = None;
        if let Some(f) = &mut self.free { f.velocity = DVec3::ZERO; }
        self.navigation_notice = Some("Surface tracking released. Free flight continues from this view.".into());
    }

    pub fn follow_surface(&mut self, sim: &Sim) {
        if let Some(a) = &self.surface_anchor {
            if a.tracked {
                match a.pose(sim) {
                    Ok(f) => { self.rotation = f.rotation; self.free = Some(f); }
                    Err(e) => { self.release_surface(); self.navigation_notice = Some(e); }
                }
            }
        }
    }

    pub fn eye(&self) -> DVec3 {
        if let Some(free) = &self.free { return free.position; }
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target_pos + self.pivot_shift + DVec3::new(cp * sy, sp, cp * cy) * self.distance
    }
}

/// A distance from which a whole star system fits on screen.
pub fn system_view_distance(sim: &Sim, system: u32) -> f64 {
    let sys = sim.universe.system(system);
    let extent = sys.bodies.iter().filter(|b| b.parent.is_none() && b.exists()).map(|b| b.orbit.apoapsis().min(200.0 * AU)).fold(0.5 * AU, f64::max);
    extent * 2.6
}

/// Position (render frame) and radius of a target.
pub fn target_info(sim: &Sim, target: Target) -> (DVec3, f64) {
    let u = &sim.universe;
    match target {
        Target::Star(s) => {
            let sys = u.system(s);
            (to_render(sys.star_position(u.time)), sys.star.current_radius(u.time))
        }
        Target::Body(r) => (to_render(u.body_position(r, u.time)), u.body(r).radius),
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRig>()
            .init_resource::<crate::navigation::Viewpoints>()
            .init_resource::<crate::cinematic::Player>()
            .add_systems(OnExit(AppState::Observing), crate::cinematic::leave_world)
            .add_systems(PostStartup, spawn_camera)
            .add_systems(OnEnter(AppState::Observing), initial_focus)
            .add_systems(Update, (camera_input, crate::cinematic::update, update_camera, crate::cinematic::record_actual).chain().in_set(Frame::Camera).run_if(in_state(AppState::Observing).and(resource_exists::<Sim>)))
            .add_systems(Update, menu_camera.in_set(Frame::Camera).run_if(not(in_state(AppState::Observing))))
            .add_systems(Update, apply_graphics_settings.run_if(resource_changed::<UserSettings>));
    }
}

fn spawn_camera(mut commands: Commands, sky: Res<SkyHandle>, settings: Res<UserSettings>) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        ColorGrading::default(),
        Tonemapping::AgX,
        Bloom { intensity: if settings.graphics.bloom() { 0.22 } else { 0.0 }, ..Bloom::NATURAL },
        msaa_for(&settings),
        Projection::Perspective(PerspectiveProjection { fov: settings.presentation.fov_deg.to_radians(), near: 1.0, far: 1.0e27, ..default() }),
        Skybox { image: sky.0.clone(), brightness: 300.0, ..default() },
        Transform::default(),
        MainCamera,
    ));
}

fn msaa_for(s: &UserSettings) -> Msaa {
    match s.graphics.msaa() {
        1 => Msaa::Off,
        2 => Msaa::Sample2,
        _ => Msaa::Sample4,
    }
}

fn apply_graphics_settings(settings: Res<UserSettings>, mut q: Query<(&mut Bloom, &mut Msaa), With<MainCamera>>) {
    for (mut bloom, mut msaa) in &mut q {
        bloom.intensity = if settings.graphics.bloom() { 0.22 } else { 0.0 };
        *msaa = msaa_for(&settings);
    }
}

/// Pick something interesting to look at when a universe opens (or obey `--focus`).
fn initial_focus(mut rig: ResMut<CameraRig>, mut sim: ResMut<Sim>, args: Res<Args>, mut exit: MessageWriter<bevy::app::AppExit>) {
    rig.free = None;
    rig.surface_anchor = None;
    rig.navigation_notice = None;
    rig.pivot_shift = DVec3::ZERO;
    rig.start_free = args.free_camera;
    let u = &sim.universe;
    let mut target = Target::Star(0);
    let mut radii = None;
    if let Some(c) = u.civs.iter().find(|c| c.is_alive()) {
        target = Target::Body(BodyRef { system: c.system, body: c.body });
    } else if let Some(b) = u.biospheres.iter().filter(|b| b.system == 0).max_by(|a, b| a.habitability.score.total_cmp(&b.habitability.score).then(a.stage.cmp(&b.stage))) {
        if b.habitability.score > 0.0 || b.stage > cosmogon_sim::life::Stage::Sterile {
            target = Target::Body(BodyRef { system: b.system, body: b.body });
        }
    }
    if !rig.applied_args {
        rig.applied_args = true;
        if let Some(name) = &args.focus {
            if let Some(t) = find_target(&sim, name) {
                target = t;
            }
        }
        radii = args.distance;
        if let Some(y) = args.yaw {
            rig.yaw = y;
        }
        if let Some(p) = args.pitch {
            rig.pitch = p;
        }
        if args.select_civ {
            sim.selected = Some(target);
        }
    }
    let (pos, radius) = target_info(&sim, target);
    if let (Some((lat, lon)), Target::Body(r)) = (args.latlon, target) {
        // Camera above a geographic point: body-local direction rotated into the world.
        let rot = crate::render::body_rotation_f64(&sim, r, sim.universe.time);
        // "noon" = the longitude currently facing the star.
        let lon = if lon.is_nan() {
            let star = rot.inverse() * (to_render(sim.universe.system(r.system).star_position(sim.universe.time)) - pos);
            star.y.atan2(star.x).to_degrees() - 55.0
        } else {
            lon
        };
        let local = cosmogon_sim::planet::terrain::dir_from_lat_lon(lat.to_radians(), lon.to_radians());
        let d = (rot * DVec3::from_array(local)).normalize();
        rig.yaw = d.x.atan2(d.z);
        rig.pitch = d.y.asin();
    } else if args.yaw.is_none() {
        // Start on the day side, the star a little off to one side.
        let star = to_render(sim.universe.system(target.system()).star_position(sim.universe.time));
        let d = (star - pos).normalize_or(DVec3::Z);
        rig.yaw = d.x.atan2(d.z) + 0.55;
        rig.pitch = d.y.asin() + 0.25;
    }
    rig.target_pos = pos;
    rig.focus = Some(target);
    rig.focus_radius = radius;
    rig.distance = radius * radii.unwrap_or(match target {
        Target::Star(_) => system_view_distance(&sim, 0) / radius,
        Target::Body(_) => 3.2,
    });
    rig.desired_distance = rig.distance;
    rig.transition = None;
    if sim.selected.is_none() {
        sim.selected = Some(target);
    }
    if let Some(id) = &args.surface_site {
        let site = cosmogon_sim::planet::landing::find(id).expect("validated site argument");
        let result = (|| {
            let Target::Body(r) = find_target(&sim, "Mars").ok_or("Mars is absent from this world")? else { return Err("Mars target is not a body".to_string()) };
            let anchor = crate::surface::SurfaceAnchor::new_dataset(r, sim.universe.body(r).elevation_data.and_then(cosmogon_sim::planet::terrain::ElevationData::mola).ok_or("Surface travel requires observed MOLA terrain")?, site.latitude_north_deg, site.longitude_east_deg, args.surface_altitude, Some(id.clone()), !args.surface_snapshot)?;
            let anchor = if args.surface_horizon {anchor.horizon()} else {anchor};
            rig.travel_surface(anchor, &sim)?;
            sim.selected = Some(Target::Body(r));
            Ok(())
        })();
        if let Err(e) = result {
            eprintln!("Surface travel failed: {e}"); rig.navigation_notice = Some(e);
            if args.quit_after_capture { exit.write(bevy::app::AppExit::error()); }
        }
    }
}

pub fn find_target(sim: &Sim, name: &str) -> Option<Target> {
    let u = &sim.universe;
    for sys in &u.systems {
        if sys.name.eq_ignore_ascii_case(name) || sys.star.name.eq_ignore_ascii_case(name) {
            return Some(Target::Star(sys.id));
        }
        if let Some(i) = sys.find_body(name) {
            return Some(Target::Body(BodyRef { system: sys.id, body: i as u32 }));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn camera_input(
    mut rig: ResMut<CameraRig>,
    mut contexts: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    keys: Res<ButtonInput<KeyCode>>,
    sim: Res<Sim>,
    args: Res<Args>,
    time: Res<Time<Real>>,
    mut settings: ResMut<UserSettings>,
    ui: Res<crate::ui::UiState>,
    mut path: ResMut<crate::cinematic::Player>,
) {
    let rig = &mut *rig;
    rig.follow_surface(&sim);
    // Automated captures must be reproducible: ignore whatever the mouse is doing.
    if args.capture.is_some() || ui.pause_menu {
        return;
    }
    let (wants_pointer, wants_keys) = match contexts.ctx_mut() {
        Ok(ctx) => (ctx.wants_pointer_input() || ctx.is_pointer_over_area(), ctx.wants_keyboard_input()),
        Err(_) => (false, false),
    };
    let cmd = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) || keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight);
    if path.active {
        let movement = !wants_keys && !cmd && [KeyCode::KeyW,KeyCode::KeyA,KeyCode::KeyS,KeyCode::KeyD,KeyCode::KeyR,KeyCode::KeyC,KeyCode::KeyQ,KeyCode::KeyE,KeyCode::KeyG,KeyCode::KeyF,KeyCode::Home,KeyCode::ArrowLeft,KeyCode::ArrowRight,KeyCode::ArrowUp,KeyCode::ArrowDown].iter().any(|k|keys.pressed(*k));
        let pointer = !wants_pointer && (scroll.delta != Vec2::ZERO || ((mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right)) && motion.delta != Vec2::ZERO));
        if movement || pointer {path.stop("Manual navigation stopped the camera path.");} else {return;}
    }
    if !wants_keys && !cmd {
        if keys.just_pressed(KeyCode::KeyG) {
            if rig.free.is_some() {
                if let Some(t) = sim.selected.or(rig.focus) { rig.focus_on(t, &sim, None); }
            } else { rig.detach(); }
        }
        if keys.just_pressed(KeyCode::KeyF) {
            if let Some(t) = sim.selected { rig.focus_on(t, &sim, None); }
        }
        if keys.just_pressed(KeyCode::Home) {
            if let Some(t) = rig.focus { rig.focus_on(Target::Star(t.system()), &sim, None); }
        }
    }
    let clearance = rig.clearance;
    if let Some(flight) = &mut rig.free {
        let dt = time.delta_secs_f64().min(0.1);
        if !wants_pointer {
            if mouse.pressed(MouseButton::Right) {
                let sign = if settings.camera.invert_y { -1.0 } else { 1.0 };
                flight.turn(-motion.delta.x as f64 * 0.003 * settings.camera.sensitivity, -motion.delta.y as f64 * 0.003 * settings.camera.sensitivity * sign, 0.0);
            }
            let lines = match scroll.unit { MouseScrollUnit::Line => scroll.delta.y as f64, MouseScrollUnit::Pixel => scroll.delta.y as f64 / 40.0 };
            if lines != 0.0 { settings.camera.speed_multiplier = (settings.camera.speed_multiplier * (lines * 0.18).exp()).clamp(0.001, 1000.0); }
        }
        if wants_keys || cmd { flight.velocity = DVec3::ZERO; return; }
        let axis = |positive, negative| (keys.pressed(positive) as i32 - keys.pressed(negative) as i32) as f64;
        flight.turn(axis(KeyCode::ArrowLeft, KeyCode::ArrowRight) * dt, axis(KeyCode::ArrowUp, KeyCode::ArrowDown) * dt, axis(KeyCode::KeyQ, KeyCode::KeyE) * dt);
        let local = DVec3::new(axis(KeyCode::KeyD, KeyCode::KeyA), axis(KeyCode::KeyR, KeyCode::KeyC), axis(KeyCode::KeyS, KeyCode::KeyW)).normalize_or_zero();
        let boost = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) { 10.0 } else if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) { 0.1 } else { 1.0 };
        flight.speed_m_s = settings.camera.speed(clearance) * boost;
        let desired = flight.rotation * local * flight.speed_m_s;
        flight.advance(desired, dt, settings.camera.inertia, settings.camera.damping);
        if local != DVec3::ZERO {
            rig.surface_anchor = None;
            rig.navigation_notice = Some("Movement released surface tracking.".into());
        } else if let Some(a) = &mut rig.surface_anchor {
            if a.tracked {
                a.local_orientation_xyzw = (crate::render::body_rotation_f64(&sim, a.body, sim.universe.time).inverse() * flight.rotation).normalize().to_array();
            }
        }
        return;
    }
    // Close to a surface, controls work on altitude and slow down so the ground is navigable.
    let ground = rig.ground_radius;
    let altitude = (rig.desired_distance - ground).max(1.0);
    let orbit_rate = 0.005 * settings.camera.sensitivity * (altitude / (rig.focus_radius * 0.5)).clamp(0.0005, 1.0);
    let zoom_by = |rig: &mut CameraRig, factor: f64| {
        rig.desired_distance = ground + (rig.desired_distance - ground).max(1.0) * factor;
    };
    // With the throw and grab tools the left button belongs to the tool; orbit with the right.
    let tool_owns_left = matches!(sim.tool, crate::sim::Tool::Throw(_) | crate::sim::Tool::Grab);
    if !wants_pointer {
        if (mouse.pressed(MouseButton::Left) && !tool_owns_left) || mouse.pressed(MouseButton::Right) {
            rig.yaw -= motion.delta.x as f64 * orbit_rate;
            rig.pitch = (rig.pitch + motion.delta.y as f64 * orbit_rate * if settings.camera.invert_y { -1.0 } else { 1.0 }).clamp(-1.54, 1.54);
        }
        let lines = match scroll.unit {
            MouseScrollUnit::Line => scroll.delta.y as f64,
            MouseScrollUnit::Pixel => scroll.delta.y as f64 / 40.0,
        };
        if lines != 0.0 {
            zoom_by(rig, (-lines * 0.15).exp());
        }
    }
    if !wants_keys && !cmd {
        let zoom = (keys.pressed(KeyCode::KeyS) as i32 - keys.pressed(KeyCode::KeyW) as i32) as f64;
        if zoom != 0.0 {
            zoom_by(rig, (zoom * 2.4 * time.delta_secs_f64().min(0.1)).exp());
        }
        rig.yaw += (keys.pressed(KeyCode::KeyA) as i32 - keys.pressed(KeyCode::KeyD) as i32) as f64 * 1.2 * time.delta_secs_f64().min(0.1);

    }
}

#[allow(clippy::too_many_arguments)]
fn update_camera(
    mut rig: ResMut<CameraRig>,
    sim: Res<Sim>,
    lod: Res<crate::render::terrain_lod::TerrainLod>,
    time: Res<Time>,
    mut view: ResMut<ViewInfo>,
    windows: Query<&Window>,
    mut cam: Query<(&mut Transform, &mut Projection, &mut Bloom, &mut ColorGrading), With<MainCamera>>,
    settings: Res<UserSettings>,
) {
    rig.follow_surface(&sim);
    if rig.free.is_some() {
        update_free_camera(&mut rig, &sim, &lod, &mut view, &windows, &mut cam, &settings);
        return;
    }
    let dt = time.delta_secs().min(0.1);
    let mut orientation_blend = None;
    let Some(mut focus) = rig.focus else { return };
    // The focused body may have been destroyed or deleted: fall back to its star.
    if let Target::Body(r) = focus {
        let gone = sim.universe.systems.get(r.system as usize).and_then(|s| s.bodies.get(r.body as usize)).is_none_or(|b| !b.exists());
        if gone {
            focus = Target::Star(r.system);
            rig.focus_on(focus, &sim, None);
        }
    }
    let (live_pos, radius) = target_info(&sim, focus);
    rig.focus_radius = radius;
    // Ground under the camera: real terrain height when close-up terrain is active.
    rig.ground_radius = match focus {
        Target::Body(r) => {
            let rot = crate::render::body_rotation_f64(&sim, r, sim.universe.time);
            let dir_local = rot.inverse() * (rig.eye() - live_pos);
            lod.ground_radius(r, dir_local).unwrap_or(radius).max(radius * 0.99)
        }
        Target::Star(_) => radius,
    };
    let min_d = match focus {
        Target::Star(_) => radius * 1.5,
        Target::Body(_) => rig.ground_radius + 60.0,
    };
    rig.desired_distance = rig.desired_distance.clamp(min_d, 1.5e10 * LIGHT_YEAR);

    if let Some(tr) = rig.transition.as_mut() {
        tr.elapsed += dt;
        let s = (tr.elapsed / tr.duration).clamp(0.0, 1.0) as f64;
        let e = s * s * (3.0 - 2.0 * s);
        orientation_blend = Some((tr.from_rotation, e));
        let (from_pos, from_dist, done) = (tr.from_pos, tr.from_dist, s >= 1.0);
        rig.target_pos = from_pos.lerp(live_pos, e);
        // Logarithmic interpolation: travel feels uniform across scales; pull back mid-flight.
        let span = (from_pos - live_pos).length();
        let peak = (span * 0.6).max(from_dist).max(rig.desired_distance);
        let l0 = from_dist.ln();
        let l1 = rig.desired_distance.ln();
        let lp = peak.ln();
        let lerp = l0 + (l1 - l0) * e + (lp - l0.max(l1)).max(0.0) * (4.0 * e * (1.0 - e));
        rig.distance = lerp.exp();
        if done {
            rig.transition = None;
        }
    } else {
        rig.target_pos = live_pos;
        let k = 1.0 - (-10.0 * dt as f64).exp();
        rig.distance = (rig.distance.ln() + (rig.desired_distance.ln() - rig.distance.ln()) * k).exp();
    }

    // Past a few thousand light-years, ease the pivot over to the Galactic Centre so pulling
    // back from any star frames the whole Milky Way.
    let sun = to_render(sim.universe.systems[0].star_position(sim.universe.time));
    let gc = sun + to_render(crate::render::galaxy::galactic_centre_offset() * LIGHT_YEAR);
    let w = ((rig.distance / LIGHT_YEAR / 4_000.0).ln() / (60_000f64 / 4_000.0).ln()).clamp(0.0, 1.0);
    let w = w * w * (3.0 - 2.0 * w);
    rig.pivot_shift = (gc - rig.target_pos) * w;

    let eye = rig.eye();
    view.origin = eye;
    if let Ok(w) = windows.single() {
        view.viewport_h = w.height();
    }
    let Ok((mut tf, mut proj, mut bloom, mut grading)) = cam.single_mut() else { return };
    // Bloom makes stars glow in space; when a sunlit world fills the view it would only veil
    // the surface, so fade it with proximity.
    if settings.graphics.bloom() {
        let fill = ((rig.distance / radius - 1.0) / 4.0).clamp(0.0, 1.0) as f32;
        bloom.intensity = 0.03 + 0.19 * fill;
    }
    // Normalise in f64: squaring galaxy-scale offsets (~1e21 m) overflows f32.
    let look = (rig.target_pos + rig.pivot_shift - eye).normalize_or(DVec3::NEG_Z).as_vec3();
    *tf = Transform::IDENTITY.looking_to(look, Vec3::Y);
    let altitude = (rig.distance - rig.ground_radius).max(1.0);
    if matches!(focus, Target::Body(_)) {
        // Near the ground, tilt the view from "straight down" towards the horizon.
        let a = altitude / radius;
        let tilt = (1.0 - a / 0.06).clamp(0.0, 1.0).powf(1.5) as f32 * 1.25;
        if tilt > 0.0 {
            let radial = (eye - rig.target_pos).normalize().as_vec3();
            let up_cam = tf.up().as_vec3();
            let tangent = (up_cam - radial * up_cam.dot(radial)).normalize_or(Vec3::X);
            let forward = -radial * tilt.cos() + tangent * tilt.sin();
            *tf = Transform::IDENTITY.looking_to(forward, radial);
        }
    }
    if let Some((from, mix)) = orientation_blend {
        tf.rotation = from.slerp(tf.rotation.as_dquat(), mix).as_quat();
    }
    rig.rotation = tf.rotation.as_dquat();
    rig.clearance = altitude;
    apply_presentation(&settings, &mut proj, &mut grading);
    if rig.start_free { rig.detach(); rig.start_free = false; }
    if let Projection::Perspective(p) = proj.as_mut() {
        p.near = (altitude * 0.05).clamp(0.5, 1.0e9) as f32;
        view.fov_y = p.fov;
    }
}

/// Slow drift around the showcase planet on the menu / loading screens.
fn menu_camera(time: Res<Time>, mut view: ResMut<ViewInfo>, mut cam: Query<(&mut Transform, &mut Projection, &mut ColorGrading), With<MainCamera>>) {
    let t = time.elapsed_secs_f64() * 0.03;
    let r = 6.371e6 * 2.6;
    let eye = DVec3::new(t.sin() * r, 0.25 * r, t.cos() * r);
    view.origin = eye;
    let Ok((mut tf, mut proj, mut grading)) = cam.single_mut() else { return };
    *grading = ColorGrading::default();
    // Look slightly past the planet so it sits off-centre, leaving room for the menu.
    let target = DVec3::new(-(t.cos()) * 0.45 * 6.371e6, 0.0, t.sin() * 0.45 * 6.371e6);
    *tf = Transform::IDENTITY.looking_to((target - eye).as_vec3().normalize(), Vec3::Y);
    if let Projection::Perspective(p) = proj.as_mut() {
        p.near = 1000.0;
        p.fov = 50f32.to_radians();
        view.fov_y = p.fov;
    }
}

fn apply_presentation(settings: &UserSettings, projection: &mut Projection, grading: &mut ColorGrading) {
    if let Projection::Perspective(p) = projection { p.fov = settings.presentation.fov_deg.to_radians(); }
    grading.global.exposure = settings.presentation.exposure_ev;
    grading.global.temperature = settings.presentation.warmth;
    grading.global.post_saturation = settings.presentation.saturation;
}

/// Navigation context only: no forces or simulation edits. All distances stay in f64.
fn surroundings(sim: &Sim, eye: DVec3) -> (Option<BodyRef>, f64) {
    let mut clearance = f64::MAX;
    let mut surface = None;
    let mut surface_clearance = f64::MAX;
    let u = &sim.universe;
    for sys in &u.systems {
        clearance = clearance.min(((eye - to_render(sys.star_position(u.time))).length() - sys.star.current_radius(u.time)).max(1.0));
        for (i, body) in sys.bodies.iter().enumerate().filter(|(_, b)| b.exists()) {
            let r = BodyRef { system: sys.id, body: i as u32 };
            let d = ((eye - to_render(u.body_position(r, u.time))).length() - body.radius).max(0.0);
            clearance = clearance.min(d.max(1.0));
            if body.kind.has_surface() && d < surface_clearance { surface_clearance = d; surface = Some(r); }
        }
    }
    (surface, clearance)
}

fn update_free_camera(
    rig: &mut CameraRig, sim: &Sim, lod: &crate::render::terrain_lod::TerrainLod,
    view: &mut ViewInfo, windows: &Query<&Window>,
    cam: &mut Query<(&mut Transform, &mut Projection, &mut Bloom, &mut ColorGrading), With<MainCamera>>,
    settings: &UserSettings,
) {
    let Some(flight) = &mut rig.free else { return };
    let (surface, mut clearance) = surroundings(sim, flight.position);
    rig.nearest_surface = surface;
    if let Some(r) = surface {
        let center = to_render(sim.universe.body_position(r, sim.universe.time));
        let delta = flight.position - center;
        let rot = crate::render::body_rotation_f64(sim, r, sim.universe.time);
        let radius = lod.ground_radius(r, rot.inverse() * delta).unwrap_or(sim.universe.body(r).radius);
        let altitude = delta.length() - radius;
        // Match orbit mode's clearance: metre-level camera placement exposes the
        // existing terrain mesh's precision/near-plane limits when a body moves underneath.
        if altitude < 60.0 {
            flight.position = center + delta.normalize_or(DVec3::Y) * (radius + 60.0);
            flight.velocity = DVec3::ZERO;
            clearance = 60.0;
        }
        clearance = clearance.min(altitude.max(60.0));
    }
    rig.clearance = clearance;
    rig.rotation = flight.rotation;
    view.origin = flight.position;
    if let Ok(w) = windows.single() { view.viewport_h = w.height(); }
    let Ok((mut tf, mut proj, mut bloom, mut grading)) = cam.single_mut() else { return };
    *tf = Transform::from_rotation(flight.rotation.as_quat());
    let nearby_radius = surface.map(|r| sim.universe.body(r).radius).unwrap_or(1.0);
    bloom.intensity = if settings.graphics.bloom() { 0.03 + 0.19 * (clearance / (nearby_radius * 4.0)).clamp(0.0, 1.0) as f32 } else { 0.0 };
    apply_presentation(settings, &mut proj, &mut grading);
    if let Projection::Perspective(p) = proj.as_mut() {
        p.near = (clearance * 0.05).clamp(0.5, 1e9) as f32;
        view.fov_y = p.fov;
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    use crate::navigation::{Flight, Presentation, Viewpoint};
    #[test]
    fn detach_and_return_to_orbit_preserve_the_starting_eye() {
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings { scenario: cosmogon_sim::Scenario::EmptySystem, ..Default::default() });
        let sim = Sim::new(u, crate::sim::Session::Reference { dataset: "test".into(), title: "Test".into() });
        let mut rig = CameraRig { focus: Some(Target::Star(0)), rotation: DQuat::from_euler(bevy::math::EulerRot::YXZ, 0.5, 0.3, 0.8), ..Default::default() };
        let eye = rig.eye();
        rig.detach();
        assert_eq!(rig.eye(), eye);
        rig.focus_on(Target::Star(0), &sim, Some(4.0));
        assert!((rig.eye() - eye).length() < 1e-5);
        assert!(rig.free.is_none());
    }
    #[test]
    fn viewpoint_and_presentation_restore_leave_universe_unchanged() {
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings { scenario: cosmogon_sim::Scenario::SolarSystemLab, ..Default::default() });
        let sim = Sim::new(u, crate::sim::Session::Reference { dataset: "test".into(), title: "Test".into() });
        let before = serde_json::to_string(&sim.universe).unwrap();
        let target = find_target(&sim, "Mars").unwrap();
        let eye = target_info(&sim, target).0 + DVec3::Z * 1e8;
        let mut rig = CameraRig { focus: Some(target), free: Some(Flight::new(eye, DQuat::IDENTITY)), ..Default::default() };
        let mut presentation = Presentation { exposure_ev: 2.0, fov_deg: 32.0, ..Default::default() };
        let saved = Viewpoint::capture("Mars", &sim, &rig, &presentation);
        rig.free.as_mut().unwrap().position += DVec3::X * 1e6;
        presentation.exposure_ev = -5.0;
        saved.restore(&sim, &mut rig, &mut presentation).unwrap();
        assert!((rig.eye() - eye).length() < 1e-3);
        assert_eq!(presentation.exposure_ev, 2.0);
        assert_eq!(serde_json::to_string(&sim.universe).unwrap(), before);
    }
}
