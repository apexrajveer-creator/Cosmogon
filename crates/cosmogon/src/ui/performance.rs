//! Optional instrumentation panel. Presentation controls never issue simulation edits.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::performance::Performance;
use crate::persistence::UserSettings;
use super::{kv, UiState, ACCENT, MUTED};

pub fn panel(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>, mut perf: ResMut<Performance>, mut settings: ResMut<UserSettings>) -> Result {
    if !ui_state.performance_open || ui_state.hidden || ui_state.photo_mode {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Performance").open(&mut ui_state.performance_open).default_pos([330.0, 76.0]).default_width(390.0).resizable(true).show(ctx, |ui| {
        ui.label(egui::RichText::new("FRAME PACING").size(11.0).color(ACCENT));
        ui.label(egui::RichText::new("Recent frames · measurements update four times a second").small().color(MUTED));
        if let Some(d) = &perf.frame_summary {
            egui::Grid::new("performance_summary").num_columns(2).spacing([20.0, 7.0]).show(ui, |ui| {
                kv(ui, "Average", format!("{:.1} ms  ·  {:.1} FPS", d.mean, 1000.0 / d.mean));
                kv(ui, "95th / 99th percentile", format!("{:.1} / {:.1} ms", d.p95, d.p99));
                kv(ui, "Slowest frame", format!("{:.1} ms", d.max));
                kv(ui, "Recorded frames", d.count.to_string());
                if let Some(s) = &perf.simulation_summary {
                    kv(ui, "Simulation average / p95", format!("{:.2} / {:.2} ms", s.mean, s.p95));
                }
                if let Some(ms) = perf.last_load_ms {
                    kv(ui, "Last universe load", format!("{:.2} s", ms / 1000.0));
                }
            });
            ui.add_space(8.0);
            if let Some(samples) = perf.report()["samples"].as_array() {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 55.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(17, 22, 33));
                let scale = d.max.max(33.3) as f32;
                let budget_y = rect.bottom() - rect.height() * 16.667 / scale;
                painter.line_segment([egui::pos2(rect.left(), budget_y), egui::pos2(rect.right(), budget_y)], egui::Stroke::new(1.0_f32, MUTED.gamma_multiply(0.35)));
                let points: Vec<_> = samples.iter().enumerate().map(|(i, s)| {
                    egui::pos2(rect.left() + rect.width() * i as f32 / samples.len().saturating_sub(1).max(1) as f32, rect.bottom() - rect.height() * s["frame_ms"].as_f64().unwrap_or(0.0) as f32 / scale)
                }).collect();
                if points.len() > 1 {
                    painter.add(egui::Shape::line(points, egui::Stroke::new(1.0_f32, ACCENT)));
                }
                ui.label(egui::RichText::new("Frame time in ms · reference line: 60 FPS (16.7 ms)").small().color(MUTED));
            }
        } else {
            ui.label("Collecting frames…");
        }
        ui.add_space(8.0);
        ui.collapsing("Process & renderer", |ui| {
            if !perf.detailed {
                ui.label("Detailed measurements are off for this session.");
            }
            let metrics = &perf.report()["diagnostics"];
            egui::Grid::new("performance_process").num_columns(2).show(ui, |ui| {
                for (label, key, unit) in [("Process memory", "process/mem_usage", "GiB"), ("Process CPU (all cores)", "process/cpu_usage", "%")] {
                    let m = &metrics[key];
                    let text = match (m["value"].as_f64(), m["age_ms"].as_f64()) {
                        (Some(v), Some(age)) if age < 2000.0 => format!("{v:.2} {unit}"),
                        _ => "Unavailable".into(),
                    };
                    kv(ui, label, text);
                }
                kv(ui, "GPU timestamps", if perf.report()["gpu_timing_available"].as_bool() == Some(true) { "Available in export" } else { "Unavailable" });
                kv(ui, "VRAM", "Not measured");
            });
            ui.label(egui::RichText::new("Exports include supported render-pass timings. Frame time includes display waits and is not GPU time.").small().color(MUTED));
            if ui.checkbox(&mut settings.detailed_profiling, "Detailed measurements at next launch").changed() {
                settings.save();
            }
        });
        ui.add_space(10.0);
        if ui.add_enabled(perf.frame_summary.is_some(), egui::Button::new("Export performance report")).clicked() {
            perf.export_requested = true;
        }
        if let Some(status) = &perf.export_status {
            ui.add(egui::Label::new(egui::RichText::new(status).small()).wrap().selectable(true));
        }
        ui.label(egui::RichText::new("Local JSON report · does not change your universe").small().color(MUTED));
    });
    Ok(())
}
