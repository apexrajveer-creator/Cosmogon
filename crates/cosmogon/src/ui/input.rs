//! Discoverable physical-key customization backed by the shared native mapping.
use super::{ACCENT, MUTED};
use crate::input::{key_label, Action, Bindings, CODES};
use bevy_egui::egui;
use std::path::Path;

pub fn show(ui: &mut egui::Ui, b: &mut Bindings) {
    ui.label(
        egui::RichText::new("YOUR KEYBOARD CONTROLS")
            .strong()
            .color(ACCENT),
    );
    ui.label("Choose an action, then press a new key or select an unused key below.");
    ui.label(egui::RichText::new("Physical key positions use US labels; printed letters can differ on other layouts. Escape, modifiers and Cmd/Ctrl shortcuts stay fixed.").small().color(MUTED));
    ui.horizontal(|ui| {
        ui.selectable_value(&mut b.pane, 0, "Camera");
        ui.selectable_value(&mut b.pane, 1, "Everyday");
        ui.selectable_value(&mut b.pane, 2, "Files");
    });
    if let Some(status) = &b.status {
        ui.label(egui::RichText::new(status).color(ACCENT));
    }
    ui.label(if b.dirty {
        "Applied for this run · changes are not saved"
    } else {
        "Current controls · no unsaved changes"
    });
    if b.pane < 2 {
        ui.add(
            egui::TextEdit::singleline(&mut b.filter)
                .hint_text("Find an action…")
                .desired_width(270.0),
        );
        let entries: Vec<_> = Action::ALL
            .into_iter()
            .filter(|a| {
                a.camera() == (b.pane == 0)
                    && a.label().to_lowercase().contains(&b.filter.to_lowercase())
            })
            .collect();
        if !entries.contains(&b.selected) {
            if let Some(a) = entries.first() {
                b.selected = *a;
            }
        }
        if entries.is_empty() {
            ui.label("No matching actions. Clear the search to see controls.");
        } else {
            egui::ComboBox::from_id_salt("binding_action")
                .selected_text(b.selected.label())
                .width(290.0)
                .show_ui(ui, |ui| {
                    for a in entries {
                        ui.selectable_value(&mut b.selected, a, a.label());
                    }
                });
            ui.label(format!("{} → {}", b.selected.label(), b.label(b.selected)));
            if let Some(action) = b.capturing {
                ui.label(
                    egui::RichText::new(format!("Waiting for a key: {}", action.label()))
                        .strong()
                        .color(ACCENT),
                );
                if ui.button("Cancel assignment (Escape)").clicked() {
                    b.cancel("Assignment cancelled. Release keys to resume controls.");
                }
            } else {
                if ui.button("Press a new key…").clicked() {
                    b.begin_capture(b.selected);
                }
                egui::ComboBox::from_id_salt("binding_key")
                    .selected_text("Choose a physical key…")
                    .width(290.0)
                    .show_ui(ui, |ui| {
                        for (id, _) in CODES {
                            let used = b
                                .definition
                                .keys
                                .iter()
                                .find(|(a, v)| **a != b.selected && v.as_str() == *id)
                                .map(|(a, _)| a.label());
                            let label = match used {
                                Some(a) => format!("{} · used by {a}", key_label(id)),
                                None => key_label(id),
                            };
                            if ui
                                .add_enabled(used.is_none(), egui::Button::new(label))
                                .clicked()
                            {
                                let result = b.assign(b.selected, id);
                                if let Err(e) = result {
                                    b.status = Some(e);
                                }
                                b.blocked = true;
                            }
                        }
                    });
            }
            ui.label(egui::RichText::new("Assignments must be unique. Camera and keyboard shortcuts wait until all keys are released after assignment or cancellation.").small().color(MUTED));
        }
        ui.collapsing("All current mappings", |ui| {
            egui::Grid::new("bindings_all")
                .striped(true)
                .show(ui, |ui| {
                    for a in Action::ALL {
                        ui.label(b.label(a));
                        ui.label(a.label());
                        ui.end_row();
                    }
                });
        });
    } else {
        ui.label(
            "Open a complete versioned mapping for this run. Invalid or newer files are refused.",
        );
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut b.import_path)
                    .hint_text("Existing controls.json")
                    .desired_width(260.0),
            );
            if ui.button("Open").clicked() {
                let path = b.import_path.clone();
                if let Err(e) = b.import(Path::new(&path)) {
                    b.status = Some(e);
                }
            }
        });
        ui.label("Export to a new file; existing files are protected.");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut b.export_path)
                    .hint_text("New controls.json")
                    .desired_width(260.0),
            );
            if ui.button("Export").clicked() {
                b.status = Some(match b.definition.export(Path::new(&b.export_path)) {
                    Ok(()) => "Controls exported to a new file.".into(),
                    Err(e) => e,
                });
            }
        });
        if ui.button("Copy current JSON").clicked() {
            match serde_json::to_string_pretty(&b.definition) {
                Ok(s) => ui.ctx().copy_text(s),
                Err(e) => b.status = Some(e.to_string()),
            }
        }
    }
    ui.separator();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                b.writable && b.dirty,
                egui::Button::new("Save for later launches"),
            )
            .clicked()
        {
            if let Err(e) = b.persist() {
                b.status = Some(e);
            }
        }
        if ui.button("Restore runtime defaults").clicked() {
            b.reset();
        }
    });
    if ui.button("Reload saved controls (discard edits)").clicked() {
        b.reload();
    }
    if !b.writable {
        ui.label(egui::RichText::new("The saved file remains protected. You can use runtime changes and export a new file.").small().color(ACCENT));
    }
    ui.label(egui::RichText::new("Mouse, controller and chord customization are not implemented. Shift boost, Alt fine movement and Escape recovery retain their existing behavior.").small().color(MUTED));
}
