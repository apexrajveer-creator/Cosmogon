//! Optional sound mixer, recent notification history and category routing.
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::audio::{SoundMix, SoundMode};
use crate::camera::CameraRig;
use crate::notifications::{Notifications, Severity, CATEGORIES};
use crate::persistence::UserSettings;
use crate::sim::Sim;
use super::{UiState, ACCENT, MUTED};

pub fn sound_settings(ui: &mut egui::Ui, settings: &mut UserSettings) {
    ui.horizontal(|ui| { for mode in SoundMode::ALL { ui.selectable_value(&mut settings.audio.mode, mode, mode.label()); } });
    ui.label(egui::RichText::new(match settings.audio.mode {
        SoundMode::Realistic => "External space effects are silent. Music and informational alerts are optional.",
        SoundMode::Cinematic => "Designed space effects and soundtrack. These are presentation sounds.",
        SoundMode::Hybrid => "Music with softer cinematic effects. Alerts use their own channel.",
    }).small().color(MUTED));
    ui.add_space(7.0);
    ui.add(egui::Slider::new(&mut settings.audio.music, 0.0..=1.0).text("Music / soundtrack"));
    ui.add_enabled(settings.audio.mode != SoundMode::Realistic, egui::Slider::new(&mut settings.audio.effects, 0.0..=1.0).text("Cinematic effects"));
    ui.add(egui::Slider::new(&mut settings.audio.alerts, 0.0..=1.0).text("Alert volume"));
    ui.horizontal(|ui| {
        if ui.button("Realistic preset").clicked() { settings.audio = SoundMix { mode: SoundMode::Realistic, music: 0.0, ..Default::default() }; }
        if ui.button("Cinematic preset").clicked() { settings.audio = SoundMix::default(); }
        if ui.button("Quiet preset").clicked() { settings.audio = SoundMix { mode: SoundMode::Hybrid, music: 0.15, effects: 0.1, alerts: 0.0 }; settings.notifications.mute_sounds = true; }
    });
    ui.checkbox(&mut settings.notifications.mute_sounds, "Mute all alert sounds");
    ui.checkbox(&mut settings.notifications.critical_sound_only, "Sound for critical alerts only");
    ui.checkbox(&mut settings.notifications.do_not_disturb, "Do Not Disturb");
    ui.label(egui::RichText::new("Do Not Disturb keeps events in the inbox and Chronicle, suppressing popups, alert sounds and configured pauses. Milestone slowdown is a separate preference.").small().color(MUTED));
    ui.label(egui::RichText::new("This mix controls the existing score and effects. Wind, interiors, weather and physical acoustic propagation are not modeled yet.").small().color(MUTED));
}

fn routing(ui: &mut egui::Ui, settings: &mut UserSettings) {
    ui.label("Choose how each event category reaches you.");
    ui.label(egui::RichText::new("Priority 1–5 comes from the simulation; 5 is a milestone. The threshold applies to popup, sound and pause. Inbox can retain lower-priority events.").small().color(MUTED));
    ui.add_space(8.0);
    egui::Grid::new("alert_routes").striped(true).spacing([10.0, 10.0]).show(ui, |ui| {
        for label in ["Category", "Inbox", "Popup", "Sound", "Pause", "Min. priority"] { ui.label(egui::RichText::new(label).color(ACCENT)); } ui.end_row();
        for rule in &mut settings.notifications.rules {
            ui.label(rule.category.label());
            ui.checkbox(&mut rule.center, ""); ui.checkbox(&mut rule.popup, "");
            ui.checkbox(&mut rule.sound, ""); ui.checkbox(&mut rule.pause, "");
            ui.add(egui::DragValue::new(&mut rule.min_importance).range(1..=5));
            ui.end_row();
        }
    });
    ui.add_space(8.0);
    if ui.button("Chronicle only for every category").clicked() {
        for r in &mut settings.notifications.rules { r.center = false; r.popup = false; r.sound = false; r.pause = false; }
    }
    if ui.button("Restore default routing").clicked() { settings.notifications = Default::default(); }
    ui.label(egui::RichText::new("Turning every route off does not erase Chronicle events. Routing takes effect immediately; Save preferences keeps it for future launches.").small().color(MUTED));
}

pub fn panel(mut contexts: EguiContexts, mut state: ResMut<UiState>, mut notices: ResMut<Notifications>, mut settings: ResMut<UserSettings>, mut sim: ResMut<Sim>, mut rig: ResMut<CameraRig>, time: Res<Time>) -> Result {
    if !state.alerts_open || state.hidden || state.photo_mode { return Ok(()); }
    let ctx = contexts.ctx_mut()?;
    let mut open = state.alerts_open;
    egui::Window::new("Sound & Alerts").open(&mut open).default_pos([325.0, 75.0]).default_width(600.0).show(ctx, |ui| {
        ui.horizontal(|ui| { ui.selectable_value(&mut state.alerts_tab, 0, "Inbox"); ui.selectable_value(&mut state.alerts_tab, 1, "Sound"); ui.selectable_value(&mut state.alerts_tab, 2, "Routing"); });
        ui.separator();
        match state.alerts_tab {
            1 => {
                ui.add(egui::Slider::new(&mut settings.sound_volume, 0.0..=1.0).text("Master volume"));
                sound_settings(ui, &mut settings);
            }
            2 => routing(ui, &mut settings),
            _ => {
                ui.horizontal(|ui| {
                    ui.label(format!("{} unread", notices.unread()));
                    if ui.button("Mark all read").clicked() { notices.mark_read(); }
                    ui.checkbox(&mut settings.notifications.do_not_disturb, "Do Not Disturb");
                });
                ui.add(egui::TextEdit::singleline(&mut state.notification_query).hint_text("Search titles and details…").desired_width(f32::INFINITY));
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("notice_severity").selected_text(format!("{} or higher", state.notice_severity.label())).show_ui(ui, |ui| { for s in Severity::ALL { ui.selectable_value(&mut state.notice_severity, s, s.label()); } });
                    egui::ComboBox::from_id_salt("notice_category").selected_text(state.notice_category.map(|c| c.label()).unwrap_or("All categories")).show_ui(ui, |ui| { ui.selectable_value(&mut state.notice_category, None, "All categories"); for c in CATEGORIES { ui.selectable_value(&mut state.notice_category, Some(c), c.label()); } });
                    ui.checkbox(&mut state.notice_unread, "Unread only");
                });
                let query = state.notification_query.trim().to_lowercase();
                let matches: Vec<_> = notices.entries.iter().enumerate().rev().filter(|(_, n)| n.severity >= state.notice_severity && state.notice_category.is_none_or(|c| c == n.event.category) && (!state.notice_unread || !n.read) && (query.is_empty() || n.event.title.to_lowercase().contains(&query) || n.event.detail.to_lowercase().contains(&query))).map(|(i, n)| (i, n.clone())).collect();
                if matches.is_empty() { ui.label("No matching events. New events arrive here as the universe evolves."); }
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    for (i, n) in matches {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{} · {} · priority {}", n.severity.label(), n.event.category.label(), n.event.importance)).small().color(MUTED));
                                if !n.read && ui.small_button("Read").clicked() { if let Some(value) = notices.entries.get_mut(i) { value.read = true; } }
                                if let Some(target) = n.target(&sim) { if ui.small_button("Go to object").clicked() { sim.selected = Some(target); rig.focus_on(target, &sim, None); } }
                            });
                            ui.label(egui::RichText::new(&n.event.title).strong().color(if n.read { super::TEXT } else { ACCENT }));
                            ui.label(&n.event.detail);
                            ui.label(egui::RichText::new(format!("Simulation time: {}", cosmogon_sim::time::format_duration(n.event.time))).small().color(MUTED));
                            ui.separator();
                        });
                    }
                });
                ui.label(egui::RichText::new("Most recent 500 routed events. Saved Chronicle history remains in the universe; read marks reset on loading or rewind. Severity labels group existing events, rather than predict new hazards.").small().color(MUTED));
                if ui.button("Open Chronicle").clicked() { state.show_left = true; state.left_tab = super::LeftTab::Chronicle; }
            }
        }
        ui.separator();
        if ui.button("Save preferences").clicked() { settings.audio.sanitize(); settings.notifications.sanitize(); sim.status = Some((match settings.try_save() { Ok(()) => "Sound & alert preferences saved".into(), Err(e) => format!("Could not save preferences: {e}") }, time.elapsed_secs_f64())); }
    });
    state.alerts_open = open;
    Ok(())
}
