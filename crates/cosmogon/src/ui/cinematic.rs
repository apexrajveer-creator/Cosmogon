//! Native authored-path controls; all poses use the shared evaluator.
use super::{ACCENT, MUTED};
use crate::{
    camera::CameraRig,
    cinematic::{Ease, Player},
    navigation::{Presentation, Viewpoints},
    sim::Sim,
};
use bevy_egui::egui;
use std::path::Path;

pub fn show(
    ui: &mut egui::Ui,
    player: &mut Player,
    sim: &Sim,
    rig: &mut CameraRig,
    p: &mut Presentation,
    views: &Viewpoints,
) {
    ui.label(
        egui::RichText::new("AUTHORED CAMERA PATH")
            .strong()
            .color(ACCENT),
    );
    ui.label("Capture views, choose their timing, then play or inspect a shot.");
    ui.label(egui::RichText::new("Presentation seconds are separate from simulation time. No physical trajectory or automatic time rewind.").small().color(MUTED));
    ui.horizontal(|ui| {
        if ui.button("New path").clicked() {
            player.new_draft(sim, rig);
        }
        ui.selectable_value(&mut player.editor_tab, 0, "Compose");
        ui.selectable_value(&mut player.editor_tab, 1, "Keys");
        ui.selectable_value(&mut player.editor_tab, 2, "Files");
    });
    if player.editor_tab == 2 {
        open_file(ui, player, sim);
    }
    if let Some(message) = &player.status {
        ui.label(egui::RichText::new(message).color(ACCENT));
    }
    let Some(mut d) = player.draft.clone() else {
        ui.label("Start a path here or open an exported definition. Opening this panel does not move the camera.");
        return;
    };
    ui.separator();
    let mut name = d.name.clone();
    ui.horizontal(|ui| {
        ui.label("Name");
        if ui
            .add(
                egui::TextEdit::singleline(&mut name)
                    .char_limit(80)
                    .desired_width(240.0),
            )
            .changed()
            && !name.trim().is_empty()
        {
            d.name = name;
            player.draft = Some(d.clone());
        }
    });
    ui.label(
        egui::RichText::new(format!(
            "{} · {} keys · {:.3} presentation s",
            if player.playing {
                "PLAYING"
            } else if player.active {
                "PREVIEW HELD"
            } else {
                "CAMERA RELEASED"
            },
            d.keys.len(),
            d.duration()
        ))
        .color(ACCENT),
    );
    ui.label(
        egui::RichText::new(format!(
            "Reference: {} · one translating frame; body spin is not followed",
            d.anchor
                .map(|t| target_name(sim, t))
                .unwrap_or_else(|| "Absolute frame".into())
        ))
        .small()
        .color(MUTED),
    );
    if d.context(sim).is_err() {
        ui.label(
            egui::RichText::new(
                "World or reference differs. Open a compatible world or start a new path.",
            )
            .color(egui::Color32::LIGHT_RED),
        );
    }
    let ready = d.validate().is_ok() && d.context(sim).is_ok();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                ready,
                egui::Button::new(if player.playing { "Pause" } else { "Play" }),
            )
            .clicked()
        {
            if player.playing {
                player.playing = false;
                player.status =
                    Some("Paused at this presentation time. Reference motion can continue.".into());
            } else if let Err(e) = player.play(sim, rig, p) {
                player.status = Some(e);
            }
        }
        if ui
            .add_enabled(player.active, egui::Button::new("Stop & release"))
            .clicked()
        {
            player.stop("Stopped. Camera and lens remain at the last view.");
        }
        if ui.add_enabled(ready, egui::Button::new("Start")).clicked() {
            if let Err(e) = player.scrub(sim, rig, p, 0.0) {
                player.status = Some(e);
            }
        }
    });
    if ready {
        let mut phase = player.time_s;
        if ui
            .add(
                egui::Slider::new(&mut phase, 0.0..=d.duration())
                    // Clamp edits, not the live clock: default clamping rounds every frame
                    // and reports a changed value, which would pause untouched playback.
                    .clamping(egui::SliderClamping::Edits)
                    .text("Presentation seconds")
                    .fixed_decimals(3),
            )
            .changed()
        {
            if let Err(e) = player.scrub(sim, rig, p, phase) {
                player.status = Some(e);
            }
        }
        if let Some(pose) = &player.evaluated {
            let correction = (rig.eye() - pose.position).length();
            if correction > 1e-6 {
                ui.label(egui::RichText::new(format!("Actual camera differs by {correction:.3} m; ground clearance can correct authored positions.")).small().color(ACCENT));
            }
        }
    } else {
        ui.label("Two or more valid keys are needed for playback/export.");
    }
    if player.editor_tab == 0 {
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut player.spacing_s)
                    .range(0.1..=300.0)
                    .speed(0.1)
                    .suffix(" s between new keys"),
            );
            if ui
                .add_enabled(d.keys.len() < 64, egui::Button::new("Capture current view"))
                .clicked()
            {
                if let Err(e) = player.append(sim, rig, p) {
                    player.status = Some(e);
                }
                d = player.draft.clone().unwrap();
            }
        });
        egui::ComboBox::from_id_salt("path_viewpoint")
            .selected_text("Append saved viewpoint…")
            .show_ui(ui, |ui| {
                for v in views.items.iter().filter(|v| v.belongs_to(sim)) {
                    if ui.selectable_label(false, &v.name).clicked() {
                        let mut temp = CameraRig::default();
                        let mut lens = Presentation::default();
                        let result = v
                            .restore(sim, &mut temp, &mut lens)
                            .and_then(|_| player.append(sim, &temp, &lens));
                        if let Err(e) = result {
                            player.status = Some(e);
                        }
                        d = player.draft.clone().unwrap();
                    }
                }
            });
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    sim.selected.is_some() && d.keys.len() >= 2,
                    egui::Button::new("Look at selection"),
                )
                .clicked()
            {
                let mut next = d.clone();
                next.look_at = sim.selected;
                match next.pose(sim, player.time_s) {
                    Ok(_) => {
                        player.stop("Look-at set. Press Play or scrub to preview.");
                        player.draft = Some(next.clone());
                        d = next;
                    }
                    Err(e) => player.status = Some(e),
                }
            }
            if ui
                .add_enabled(
                    d.look_at.is_some(),
                    egui::Button::new("Use authored orientation"),
                )
                .clicked()
            {
                d.look_at = None;
                player.stop("Authored orientation restored.");
                player.draft = Some(d.clone());
            }
        });
        if let Some(t) = d.look_at {
            ui.label(format!("Look-at target: {}", target_name(sim, t)));
        }
    }
    if player.editor_tab == 1 {
        ui.separator();
        ui.label("Keyframes · position, orientation and lens captured from actual views");
        player.selected = player.selected.min(d.keys.len().saturating_sub(1));
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("path_selected_key")
                .selected_text(format!("Key {}", player.selected + 1))
                .show_ui(ui, |ui| {
                    for (i, k) in d.keys.iter().enumerate() {
                        ui.selectable_value(
                            &mut player.selected,
                            i,
                            format!(
                                "Key {} · {:.3} s · {:.1}°",
                                i + 1,
                                k.time_s,
                                k.presentation.fov_deg
                            ),
                        );
                    }
                });
            if let Some(k) = d.keys.get(player.selected) {
                if ui
                    .add_enabled(ready, egui::Button::new("Preview selected key"))
                    .clicked()
                {
                    if let Err(e) = player.scrub(sim, rig, p, k.time_s) {
                        player.status = Some(e);
                    }
                }
            }
        });
        player.selected = player.selected.min(d.keys.len().saturating_sub(1));
        if let Some(key) = d.keys.get(player.selected).cloned() {
            let mut next = key.clone();
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label(format!("Edit key {}", player.selected + 1));
                changed |= ui
                    .add_enabled(
                        player.selected > 0,
                        egui::DragValue::new(&mut next.time_s)
                            .range(0.0..=3600.0)
                            .speed(0.1)
                            .suffix(" s"),
                    )
                    .changed();
                egui::ComboBox::from_id_salt("key_ease")
                    .selected_text(next.ease.label())
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(&mut next.ease, Ease::Linear, "Linear speed")
                            .changed();
                        changed |= ui
                            .selectable_value(&mut next.ease, Ease::Smoothstep, "Ease in/out")
                            .changed();
                    });
            });
            ui.label(
            egui::RichText::new(
                "Timing applies to the segment leaving this key; lens values interpolate with it.",
            )
            .small()
            .color(MUTED),
        );
            changed |= ui
                .add(
                    egui::Slider::new(&mut next.presentation.fov_deg, 10.0..=110.0)
                        .text("Key vertical field of view (°)"),
                )
                .changed();
            if changed {
                let mut candidate = d.clone();
                candidate.keys[player.selected] = next;
                match candidate.validate_draft() {
                    Ok(()) => {
                        player.stop("Key updated. Scrub or Play to inspect.");
                        player.draft = Some(candidate.clone());
                        d = candidate;
                    }
                    Err(e) => player.status = Some(format!("Key unchanged: {e}")),
                }
            }
            ui.horizontal(|ui| {
                if ui.button("Replace with current view").clicked() {
                    let mut temp = Player::default();
                    temp.draft = Some(crate::cinematic::Definition {
                        keys: Vec::new(),
                        ..d.clone()
                    });
                    match temp.append(sim, rig, p) {
                        Ok(()) => {
                            let mut replacement = temp.draft.unwrap().keys.remove(0);
                            replacement.time_s = key.time_s;
                            replacement.ease = key.ease;
                            let mut candidate = d.clone();
                            candidate.keys[player.selected] = replacement;
                            match candidate.validate_draft() {
                                Ok(()) => {
                                    player.stop("Key view replaced.");
                                    player.draft = Some(candidate.clone());
                                    d = candidate;
                                }
                                Err(e) => player.status = Some(e),
                            }
                        }
                        Err(e) => player.status = Some(e),
                    }
                }
                if ui.button("Remove key").clicked() {
                    d.keys.remove(player.selected);
                    if let Some(first) = d.keys.first_mut() {
                        let offset = first.time_s;
                        for k in &mut d.keys {
                            k.time_s -= offset;
                        }
                    }
                    player.stop("Key removed. Remaining keys start at zero.");
                    player.time_s = 0.0;
                    player.draft = Some(d.clone());
                }
            });
        }
    }
    if player.editor_tab == 2 {
        ui.separator();
        ui.label("Save to a new file · existing files are protected");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut player.export_path)
                    .hint_text("New camera path JSON")
                    .desired_width(250.0),
            );
            if ui
                .add_enabled(
                    d.validate().is_ok() && !player.export_path.trim().is_empty(),
                    egui::Button::new("Save new file"),
                )
                .clicked()
            {
                player.status = Some(match d.export(Path::new(player.export_path.trim())) {
                    Ok(()) => "Camera path saved.".into(),
                    Err(e) => e,
                });
            }
        });
        if ui.button("Copy definition JSON").clicked() {
            match serde_json::to_string_pretty(&d) {
                Ok(s) => ui.ctx().copy_text(s),
                Err(e) => player.status = Some(e.to_string()),
            }
        }
    }
    ui.label(egui::RichText::new("Drafts stay in memory until exported. Stop releases control without restoring an earlier lens. Video, depth-of-field, spline paths and simulation-rate synchronization remain unavailable.").small().color(MUTED));
}

fn target_name(sim: &Sim, t: crate::sim::Target) -> String {
    let Some(s) = sim.universe.systems.get(t.system() as usize) else {
        return "Missing reference".into();
    };
    match t {
        crate::sim::Target::Star(_) => s.star.name.clone(),
        crate::sim::Target::Body(r) => s
            .bodies
            .get(r.body as usize)
            .filter(|b| b.exists())
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "Missing reference".into()),
    }
}

fn open_file(ui: &mut egui::Ui, player: &mut Player, sim: &Sim) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut player.import_path)
                .hint_text("Camera path JSON")
                .desired_width(240.0),
        );
        if ui
            .add_enabled(
                !player.import_path.trim().is_empty(),
                egui::Button::new("Open"),
            )
            .clicked()
        {
            let path = player.import_path.trim().to_owned();
            if let Err(e) = player.import(Path::new(&path), sim) {
                player.status = Some(format!("Could not open; previous draft preserved: {e}"));
            }
        }
    });
}
