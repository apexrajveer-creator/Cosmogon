//! Independent recorded observations, never edits or fabricated live history.
use super::{ACCENT, MUTED};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_egui::egui;
use cosmogon_sim::habitat_recording::{Metric, Recording};

#[derive(Default)]
pub struct Study {
    pub recording: Option<Recording>,
    pub metric: Metric,
    pub selected: usize,
    path: String,
    pub status: Option<String>,
    task: Option<Task<Result<Recording, String>>>,
    pending_path: String,
    requested_sample: Option<usize>,
}
impl Study {
    pub fn start(&mut self, path: String, sample: Option<usize>) {
        let location = std::path::PathBuf::from(&path);
        self.task =
            Some(AsyncComputeTaskPool::get().spawn(async move { Recording::read(&location) }));
        self.pending_path = path;
        self.requested_sample = sample;
        self.status = Some("Checking recorded sources, endpoints and observations…".into());
    }
    pub fn pending(&self) -> bool {
        self.task.is_some()
    }
    fn commit(&mut self, r: Recording, path: String, sample: Option<usize>) -> Result<(), String> {
        let selected = sample.unwrap_or(0);
        if selected >= r.rows.len() {
            return Err("Requested observation index is outside the recording".into());
        }
        self.recording = Some(r);
        self.path = path;
        self.selected = selected;
        self.status = Some("Recorded observations reopened. The live world is unchanged.".into());
        Ok(())
    }
    pub fn poll(&mut self) {
        let result = self
            .task
            .as_mut()
            .and_then(|task| block_on(future::poll_once(task)));
        if let Some(result) = result {
            self.task = None;
            match result
                .and_then(|r| self.commit(r, self.pending_path.clone(), self.requested_sample))
            {
                Ok(()) => {}
                Err(e) => {
                    self.status = Some(format!(
                        "Could not open recording: {e}. Previous observations are preserved."
                    ))
                }
            }
        }
    }
    pub fn metadata(&self) -> Option<serde_json::Value> {
        let r = self.recording.as_ref()?;
        let row = &r.rows[self.selected.min(r.rows.len() - 1)];
        Some(
            serde_json::json!({"summary":r.summary,"metric":self.metric.id(),"unit":self.metric.unit(),"selected_observation":self.selected,"selected_row":row,"selected_value":self.metric.value(row),"scope":"Independent recorded view at capture request, possibly hidden; no live-world or camera change"}),
        )
    }
}
fn graph(ui: &mut egui::Ui, r: &Recording, metric: Metric, selected: &mut usize) {
    ui.label(format!(
        "{} ({}) vs elapsed terrestrial hours",
        metric.label(),
        metric.unit()
    ));
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 130.0),
        egui::Sense::click_and_drag(),
    );
    let plot = egui::Rect::from_min_max(
        rect.min + egui::vec2(64.0, 8.0),
        rect.max - egui::vec2(12.0, 26.0),
    );
    if plot.width() < 1.0 || plot.height() < 1.0 {
        return;
    }
    let painter = ui.painter_at(rect);
    painter.rect_filled(plot, 4.0, egui::Color32::from_rgb(11, 15, 24));
    let values = r
        .rows
        .iter()
        .filter_map(|row| metric.value(row))
        .collect::<Vec<_>>();
    if values.is_empty() {
        painter.text(
            plot.center(),
            egui::Align2::CENTER_CENTER,
            "No active interval observations for this metric",
            egui::FontId::proportional(12.0),
            MUTED,
        );
        return;
    }
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (high - low).max(high.abs() * 0.01).max(1e-9);
    let lo = (low - span * 0.05).max(0.0);
    let range = (high + span * 0.05 - lo).max(1e-12);
    let t0 = r.rows[0].simulation_time_s;
    let duration = r.rows.last().unwrap().simulation_time_s - t0;
    let x = |row: &cosmogon_sim::habitat::Sample| {
        plot.left() + plot.width() * ((row.simulation_time_s - t0) / duration.max(1.0)) as f32
    };
    let at = |row: &cosmogon_sim::habitat::Sample, v: f64| {
        egui::pos2(
            x(row),
            plot.bottom() - plot.height() * ((v - lo) / range) as f32,
        )
    };
    for n in 0..=3 {
        let y = plot.bottom() - plot.height() * n as f32 / 3.0;
        painter.line_segment(
            [egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(40)),
        );
        painter.text(
            egui::pos2(plot.left() - 4.0, y),
            egui::Align2::RIGHT_CENTER,
            {
                let v = lo + range * n as f64 / 3.0;
                if range < 0.01 || v.abs() >= 1e5 {
                    format!("{v:.4e}")
                } else {
                    format!("{v:.4}")
                }
            },
            egui::FontId::proportional(10.0),
            MUTED,
        );
    }
    for n in 0..=3 {
        if duration == 0.0 && n > 0 {
            continue;
        }
        let x = plot.left() + plot.width() * n as f32 / 3.0;
        painter.text(
            egui::pos2(x, plot.bottom() + 6.0),
            if n == 3 {
                egui::Align2::RIGHT_TOP
            } else {
                egui::Align2::CENTER_TOP
            },
            format!("{:.2}", duration / 3600.0 * n as f64 / 3.0),
            egui::FontId::proportional(10.0),
            MUTED,
        );
    }
    let mut previous = None;
    for row in &r.rows {
        if let Some(v) = metric.value(row) {
            let p = at(row, v);
            if let Some(old) = previous {
                painter.line_segment([old, p], egui::Stroke::new(1.7_f32, ACCENT));
            }
            painter.circle_filled(p, 2.0, ACCENT);
            previous = Some(p);
        } else {
            previous = None;
        }
    }
    if response.clicked() || response.dragged() {
        if let Some(p) = response.interact_pointer_pos() {
            let target =
                ((p.x - plot.left()) / plot.width()).clamp(0.0, 1.0) as f64 * duration + t0;
            *selected = r
                .rows
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    (a.simulation_time_s - target)
                        .abs()
                        .total_cmp(&(b.simulation_time_s - target).abs())
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }
    let row = &r.rows[(*selected).min(r.rows.len() - 1)];
    let cursor = x(row);
    painter.line_segment(
        [
            egui::pos2(cursor, plot.top()),
            egui::pos2(cursor, plot.bottom()),
        ],
        egui::Stroke::new(1.0_f32, MUTED),
    );
    if let Some(v) = metric.value(row) {
        painter.circle_filled(at(row, v), 4.0, egui::Color32::WHITE);
    }
}
pub fn panel(ui: &mut egui::Ui, study: &mut Study) {
    ui.strong("Recorded habitat experiments");
    ui.label(
        egui::RichText::new("Actual exported observations · independent of the live world")
            .color(ACCENT),
    );
    if let Some(r) = &study.recording {
        ui.label(format!(
            "{} · world [{},{}] · {} observations",
            r.summary["name"].as_str().unwrap_or("Habitat"),
            r.rows[0].habitat.definition.body.system,
            r.rows[0].habitat.definition.body.body,
            r.rows.len()
        ));
        egui::ComboBox::from_id_salt("habitat-recording-metric")
            .selected_text(study.metric.label())
            .show_ui(ui, |ui| {
                for metric in Metric::ALL {
                    ui.selectable_value(&mut study.metric, metric, metric.label());
                }
            });
        graph(ui, r, study.metric, &mut study.selected);
        ui.add(
            egui::Slider::new(&mut study.selected, 0..=r.rows.len() - 1).text("Observation index"),
        );
        let row = &r.rows[study.selected];
        ui.strong(format!(
            "{}: {} {}",
            study.metric.label(),
            study
                .metric
                .value(row)
                .map(|v| if v != 0.0 && (v.abs() < 1e-5 || v.abs() > 1e8) {
                    format!("{v:.12e}")
                } else {
                    format!("{v:.9}")
                })
                .unwrap_or_else(|| "unavailable".into()),
            study.metric.unit()
        ));
        ui.label(format!(
            "World time {:.3} s · completed habitat time {:.3} s · interval {:.3} s",
            row.simulation_time_s, row.habitat.last_time, row.habitat.budget.seconds
        ));
        ui.label(egui::RichText::new("Points are recorded samples; connecting lines are visual guides. Missing active intervals are gaps. No interpolation, prediction or world rewind.").small().color(MUTED));
        egui::CollapsingHeader::new("Selected conditions, sources and constraints").show(ui,|ui| {
            ui.label(format!("Original site {:.5}° N, {:.5}° E · crew demand {} · current cabin {:.3} K",row.habitat.definition.latitude_deg,row.habitat.definition.longitude_east_deg,row.habitat.config.crew,row.habitat.state.temperature_k));
            ui.label(format!("Current equipment: electrolysis {} · CO₂ reduction {} · completed interval environment {}",row.habitat.config.electrolysis.is_some(),row.habitat.config.sabatier.is_some(),row.habitat.budget.environment_available));
            if let Some(b)=&row.habitat.budget.electrolysis {ui.label(format!("Electrolysis constraints: {}",b.constraints.iter().map(|c|c.label()).collect::<Vec<_>>().join("; ")));}
            if let Some(b)=&row.habitat.budget.sabatier {ui.label(format!("Reduction constraints: {}",b.constraints.iter().map(|c|c.label()).collect::<Vec<_>>().join("; ")));}
            ui.hyperlink_to("NASA nominal crew model",cosmogon_sim::habitat::SOURCE);
            ui.label(r.summary["validation_scope"].as_str().unwrap_or("Structural checks only"));
            ui.label(format!("Initial identity {} · final identity {}",r.summary["initial_state_fingerprint"],r.summary["final_state_fingerprint"]));
            ui.label("Recorded source/configuration and model limits belong to the original bundle. They are not inferred from the live world, and bookkeeping checks do not establish field accuracy.");
        });
        egui::CollapsingHeader::new("All selected values, configuration and source metadata").show(
            ui,
            |ui| {
                let data = serde_json::to_string_pretty(&row.habitat.report()).unwrap_or_default();
                if ui.button("Copy selected report JSON").clicked() {
                    ui.ctx().copy_text(data.clone());
                }
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.code(data);
                    });
            },
        );
    } else {
        ui.label("Open a protected habitat-study folder from CLI habitat or the native snapshot export. A snapshot contains one observation; a trajectory contains multiple requested world times.");
    }
    ui.separator();
    ui.label("Study folder");
    ui.text_edit_singleline(&mut study.path);
    if ui
        .add_enabled(
            !study.pending(),
            egui::Button::new("Open recorded observations"),
        )
        .clicked()
    {
        study.start(study.path.clone(), None);
    }
    if let Some(status) = &study.status {
        ui.colored_label(ACCENT, status);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_selection_preserves_recording_and_recorded_values_are_independent() {
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = cosmogon_sim::BodyRef { system: 0, body: 3 };
        let h = cosmogon_sim::habitat::Habitat::new(
            0,
            cosmogon_sim::habitat::Definition::at(r, 0.0, 0.0),
            u.time,
        )
        .unwrap();
        let recording = Recording {
            rows: vec![cosmogon_sim::habitat::Sample {
                simulation_time_s: u.time,
                habitat: h,
            }],
            summary: serde_json::json!({"name":"Recorded"}),
        };
        let mut study = Study::default();
        study
            .commit(recording.clone(), "recorded".into(), None)
            .unwrap();
        let before = study.metadata();
        assert!(study.commit(recording, "bad".into(), Some(1)).is_err());
        assert_eq!(study.metadata(), before);
        assert!(u.habitats.is_empty());
        assert_eq!(study.metadata().unwrap()["selected_value"], 200.0);
    }
}
