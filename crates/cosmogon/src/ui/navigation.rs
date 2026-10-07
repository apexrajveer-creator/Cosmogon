//! Discoverable camera, lens and viewpoint controls for the native app.
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::camera::CameraRig;
use crate::navigation::{Presentation, Viewpoint, Viewpoints};
use crate::persistence::UserSettings;
use crate::sim::Sim;
use super::{UiState, ACCENT, MUTED};

pub fn panel(mut contexts: EguiContexts, mut state: ResMut<UiState>, mut rig: ResMut<CameraRig>, mut settings: ResMut<UserSettings>, mut views: ResMut<Viewpoints>, mut sim: ResMut<Sim>, time: Res<Time>) -> Result {
    if !state.camera_open || state.hidden || state.photo_mode { return Ok(()); }
    let ctx = contexts.ctx_mut()?;
    let mut open = state.camera_open;
    egui::Window::new("Camera & Photo").open(&mut open).default_pos([325.0, 75.0]).default_width(400.0).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.camera_tab, 0, "Navigate");
            ui.selectable_value(&mut state.camera_tab, 1, "Photo");
            ui.selectable_value(&mut state.camera_tab, 2, "Viewpoints");
        });
        ui.separator();
        match state.camera_tab {
            0 => {
                ui.label(egui::RichText::new(if rig.surface_anchor.as_ref().is_some_and(|a|a.tracked) { "SURFACE TRACKING" } else if rig.free.is_some() { "FREE FLIGHT" } else { "ORBIT & TRACK" }).color(ACCENT).strong());
                ui.horizontal(|ui| {
                    if ui.add_enabled(rig.free.is_none(), egui::Button::new("Detach camera (G)")).clicked() { rig.detach(); }
                    if ui.button("Orbit selection (F)").clicked() {
                        if let Some(t) = sim.selected.or(rig.focus) { rig.focus_on(t, &sim, None); }
                    }
                    if ui.button("Stop").clicked() { if let Some(f) = &mut rig.free { f.velocity = bevy::math::DVec3::ZERO; } }
                });
                if rig.surface_anchor.is_some() && ui.button("Release surface view").clicked() {rig.release_surface();}
                if let Some(message)=&rig.navigation_notice {ui.label(egui::RichText::new(message).small().color(ACCENT));}
                ui.add_space(6.0);
                ui.label("W / S forward & back · A / D sideways · R / C up & down");
                ui.label("Right-drag to look · Q / E roll · arrows to look");
                ui.label("Wheel adjusts speed · Shift ×10 · Alt/Option ×0.1");
                ui.label(egui::RichText::new("The camera is a navigation tool, not a simulated spacecraft.").small().color(MUTED));
                ui.separator();
                ui.checkbox(&mut settings.camera.auto_speed, "Adapt speed to nearby objects");
                if !settings.camera.auto_speed {
                    ui.horizontal(|ui| {
                        ui.label("Base speed (m/s)");
                        ui.add(egui::DragValue::new(&mut settings.camera.manual_speed_m_s).speed(10.0).range(0.01..=1e22));
                    });
                }
                ui.add(egui::Slider::new(&mut settings.camera.speed_multiplier, 0.001..=1000.0).logarithmic(true).text("Speed multiplier"));
                ui.label(format!("Travel speed: {} / s", super::distance(settings.camera.speed(rig.clearance))));
                ui.checkbox(&mut settings.camera.inertia, "Inertial movement");
                if settings.camera.inertia { ui.add(egui::Slider::new(&mut settings.camera.damping, 0.2..=30.0).text("Response")); }
                ui.add(egui::Slider::new(&mut settings.camera.sensitivity, 0.1..=5.0).text("Mouse sensitivity"));
                ui.checkbox(&mut settings.camera.invert_y, "Invert vertical mouse look");
                if ui.button("Save camera preferences").clicked() { settings.camera.sanitize(); sim.status = Some((match settings.try_save() { Ok(()) => "Camera preferences saved".into(), Err(e) => format!("Could not save preferences: {e}") }, time.elapsed_secs_f64())); }
            }
            1 => {
                ui.label(egui::RichText::new("PRESENTATION ONLY").color(ACCENT).strong());
                ui.label(egui::RichText::new("These controls change the image. Stellar output, climate and solar power are unchanged.").small().color(MUTED));
                ui.add_space(7.0);
                ui.add(egui::Slider::new(&mut settings.presentation.fov_deg, 10.0..=110.0).text("Vertical field of view (°)"));
                ui.add(egui::Slider::new(&mut settings.presentation.exposure_ev, -6.0..=6.0).text("Exposure (EV)"));
                ui.add(egui::Slider::new(&mut settings.presentation.warmth, -0.03..=0.03).text("Cool ↔ warm"));
                ui.add(egui::Slider::new(&mut settings.presentation.saturation, 0.0..=2.0).text("Saturation"));
                ui.horizontal(|ui| {
                    if ui.button("Reset lens & colour").clicked() { settings.presentation = Presentation::default(); }
                    if ui.button("Save as defaults").clicked() { sim.status = Some((match settings.try_save() { Ok(()) => "Photo defaults saved".into(), Err(e) => format!("Could not save preferences: {e}") }, time.elapsed_secs_f64())); }
                });
                ui.separator();
                ui.checkbox(&mut sim.paused, "Pause simulation");
                ui.checkbox(&mut settings.photo_metadata, "Write camera & dataset metadata (JSON)");
                if ui.button("Take clean PNG (P)").clicked() { state.photo_requested = true; }
                ui.label(egui::RichText::new("Saves at the window's native pixel resolution to Pictures/Cosmogon. Navigation remains available while composing.").small().color(MUTED));
                if ui.button("Hide interface (Tab to restore)").clicked() { state.hidden = true; }
            }
            _ => {
                ui.label(egui::RichText::new("SAVED VIEWPOINTS").color(ACCENT).strong());
                ui.label(egui::RichText::new("Position, orientation and lens for this universe. Surface views can follow body spin and motion; other views restore relative to their reference object. Time is never rewound.").small().color(MUTED));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut state.viewpoint_name).hint_text("Name this view").desired_width(235.0));
                    if ui.add_enabled(views.writable && !state.viewpoint_name.trim().is_empty(), egui::Button::new("Save view")).clicked() {
                        let value = Viewpoint::capture(&state.viewpoint_name, &sim, &rig, &settings.presentation);
                        views.status = Some(match views.add(value) { Ok(()) => "Viewpoint saved.".into(), Err(e) => e });
                    }
                });
                let entries: Vec<_> = views.items.iter().enumerate().filter(|(_, v)| v.belongs_to(&sim)).map(|(i, v)| (i, v.clone())).collect();
                if entries.is_empty() { ui.label("No saved viewpoints in this universe yet."); }
                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    for (i, view) in entries {
                        ui.push_id(i, |ui| { ui.horizontal(|ui| {
                            if ui.button(&view.name).on_hover_text("Restore this viewpoint in free-camera mode").clicked() {
                                views.status = Some(match view.restore(&sim, &mut rig, &mut settings.presentation) {
                                    Ok(()) => {
                                        if let Some(a)=&rig.surface_anchor {
                                            state.show_surface_anchor(a);
                                            sim.selected=Some(crate::sim::Target::Body(a.body));
                                        }
                                        format!("Restored {}",view.name)
                                    }
                                    Err(e)=>e
                                });
                            }
                            if ui.small_button("Remove").clicked() { state.remove_viewpoint = Some(i); }
                        });
                        if let Some(a)=&view.surface_anchor {ui.label(egui::RichText::new(format!("{} · {:.5}° N, {:.5}° E · {:.0} m · {}",a.site_id.as_deref().unwrap_or("Surface"),a.latitude_deg,a.longitude_east_deg,a.altitude_m,if a.tracked {"follow surface"} else {"snapshot"})).small().color(MUTED));}
                        });
                    }
                });
                if let Some(i) = state.remove_viewpoint {
                    ui.horizontal(|ui| {
                        ui.label("Remove this saved view?");
                        if ui.button("Remove view").clicked() {
                            views.status = Some(match views.remove(i) { Ok(()) => "Viewpoint removed.".into(), Err(e) => e });
                            state.remove_viewpoint = None;
                        }
                        if ui.button("Keep").clicked() { state.remove_viewpoint = None; }
                    });
                }
                if let Some(message) = &views.status { ui.add(egui::Label::new(message).wrap()); }
            }
        }
    });
    state.camera_open = open;
    Ok(())
}
