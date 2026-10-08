//! Presentation-only recorded graph layout/navigation; original values stay unchanged.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{
    habitat_comparison::Alignment, habitat_factorial::Effect, habitat_recording::Metric,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scale {
    #[default]
    Linear,
    Log10,
    SignedLog,
}
impl Scale {
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Log10 => "Log10 (positive)",
            Self::SignedLog => "Signed log",
        }
    }
    pub fn forward(self, v: f64, t: f64) -> Option<f64> {
        if !v.is_finite() {
            return None;
        }
        Some(match self {
            Self::Linear => v,
            Self::Log10 => {
                if v <= 0. {
                    return None;
                }
                v.log10()
            }
            Self::SignedLog => {
                let a = v.abs() / t;
                if a <= 1. {
                    v / t
                } else {
                    v.signum() * (1. + a.log10())
                }
            }
        })
    }
    pub fn inverse(self, v: f64, t: f64) -> f64 {
        match self {
            Self::Linear => v,
            Self::Log10 => 10_f64.powf(v),
            Self::SignedLog => {
                if v.abs() <= 1. {
                    v * t
                } else {
                    v.signum() * t * 10_f64.powf(v.abs() - 1.)
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Seconds,
    #[default]
    Hours,
    Days,
}
impl TimeUnit {
    pub fn factor(self) -> f64 {
        match self {
            Self::Seconds => 1.,
            Self::Hours => 3600.,
            Self::Days => 86400.,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Seconds => "s",
            Self::Hours => "terrestrial hours",
            Self::Days => "terrestrial days",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub format: String,
    pub schema_version: u32,
    pub metric: String,
    pub effect: String,
    pub alignment: String,
    pub window: [f64; 2],
    pub scale: Scale,
    pub signed_threshold: f64,
    pub include_zero: bool,
    pub time_unit: TimeUnit,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            format: "cosmogon-habitat-graph-layout".into(),
            schema_version: 1,
            metric: Metric::default().id().into(),
            effect: Effect::default().id().into(),
            alignment: "elapsed".into(),
            window: [0., 1.],
            scale: Scale::Linear,
            signed_threshold: 1.,
            include_zero: false,
            time_unit: TimeUnit::Hours,
        }
    }
}
impl Layout {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "cosmogon-habitat-graph-layout" || self.schema_version != 1 {
            return Err("Unsupported habitat graph layout format/version".into());
        }
        Metric::parse(&self.metric)?;
        Effect::parse(&self.effect)?;
        Alignment::parse(&self.alignment)?;
        let [a, b] = self.window;
        if !a.is_finite() || !b.is_finite() || a < 0. || b > 1. || b - a + 1e-15 < 1e-6 {
            return Err("Graph window must span at least one millionth of the complete time extent, within 0–1".into());
        }
        if !self.signed_threshold.is_finite() || !(1e-12..=1e12).contains(&self.signed_threshold) {
            return Err("Signed log threshold must be finite and between 1e-12 and 1e12 in current metric units".into());
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let m = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if !m.is_file() || m.len() > 65536 {
            return Err("Graph layout must be a regular file of at most 64 KiB".into());
        }
        let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if !f.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Graph layout must be a regular file".into());
        }
        let mut bytes = Vec::new();
        f.take(65537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Graph layout exceeds 64 KiB".into());
        }
        let d: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        d.validate()?;
        Ok(d)
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
            .map_err(|e| format!("Choose a new layout file; existing files are protected: {e}"))?;
        f.write_all(&bytes).map_err(|e| e.to_string())
    }
    pub fn pan(&mut self, fraction: f64) {
        if !fraction.is_finite() {
            return;
        }
        let width = self.window[1] - self.window[0];
        let start = (self.window[0] + width * fraction).clamp(0., 1. - width);
        self.window = [start, start + width];
    }
    pub fn zoom(&mut self, factor: f64, anchor: f64) {
        if !factor.is_finite() || factor <= 0. || !anchor.is_finite() {
            return;
        }
        let old = self.window[1] - self.window[0];
        let anchor = anchor.clamp(0., 1.);
        let width = (old * factor).clamp(1e-6, 1.);
        let fixed = self.window[0] + old * anchor;
        let start = (fixed - width * anchor).clamp(0., 1. - width);
        self.window = [start, start + width];
    }
    pub fn center(&mut self, coordinate: f64, extent: [f64; 2]) {
        if extent[1] <= extent[0] {
            return;
        }
        let width = self.window[1] - self.window[0];
        let middle = ((coordinate - extent[0]) / (extent[1] - extent[0])).clamp(0., 1.);
        let start = (middle - width / 2.).clamp(0., 1. - width);
        self.window = [start, start + width];
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Datum {
    pub index: usize,
    pub coordinate_s: f64,
    pub value: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Plot {
    pub extent_s: [f64; 2],
    pub window_s: [f64; 2],
    pub displayed: Vec<Vec<Datum>>,
    pub supported_count: usize,
    pub unavailable_count: usize,
    pub scale_omitted_count: usize,
    pub y_bounds: Option<[f64; 2]>,
}
impl Plot {
    pub fn new(series: &[Vec<Datum>], layout: &Layout) -> Self {
        let mut extent = [f64::INFINITY, f64::NEG_INFINITY];
        for p in series.iter().flatten() {
            extent[0] = extent[0].min(p.coordinate_s);
            extent[1] = extent[1].max(p.coordinate_s);
        }
        if !extent[0].is_finite() || !extent[1].is_finite() {
            extent = [0., 0.];
        }
        let duration = extent[1] - extent[0];
        let window = [
            extent[0] + duration * layout.window[0],
            extent[0] + duration * layout.window[1],
        ];
        let displayed = series
            .iter()
            .map(|s| {
                s.iter()
                    .filter(|p| p.coordinate_s >= window[0] && p.coordinate_s <= window[1])
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut supported = 0;
        let mut unavailable = 0;
        let mut omitted = 0;
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for p in displayed.iter().flatten() {
            match p.value {
                None => unavailable += 1,
                Some(v) => match layout.scale.forward(v, layout.signed_threshold) {
                    Some(y) => {
                        supported += 1;
                        low = low.min(y);
                        high = high.max(y);
                    }
                    None => omitted += 1,
                },
            }
        }
        let bounds = if supported == 0 {
            None
        } else {
            if layout.include_zero && layout.scale != Scale::Log10 {
                low = low.min(0.);
                high = high.max(0.);
            }
            let pad = if layout.scale == Scale::Linear {
                (high - low).max(high.abs().max(low.abs()) * 0.01).max(1e-9) * 0.05
            } else {
                (high - low).max(0.1) * 0.05
            };
            Some([low - pad, high + pad])
        };
        Self {
            extent_s: extent,
            window_s: window,
            displayed,
            supported_count: supported,
            unavailable_count: unavailable,
            scale_omitted_count: omitted,
            y_bounds: bounds,
        }
    }
    pub fn nearest(&self, target: f64) -> Option<usize> {
        self.displayed
            .first()?
            .iter()
            .min_by(|a, b| {
                (a.coordinate_s - target)
                    .abs()
                    .total_cmp(&(b.coordinate_s - target).abs())
            })
            .map(|p| p.index)
    }
    pub fn selected_visible(&self, index: usize) -> bool {
        self.displayed
            .first()
            .is_some_and(|s| s.iter().any(|p| p.index == index))
    }
}
pub struct View {
    pub layout: Layout,
    path: String,
    output: String,
    pub status: Option<String>,
    preference_path: PathBuf,
    writable: bool,
}
impl Default for View {
    fn default() -> Self {
        Self {
            layout: Layout::default(),
            path: String::new(),
            output: String::new(),
            status: None,
            preference_path: crate::persistence::data_dir().join("habitat-graph-layout.json"),
            writable: true,
        }
    }
}
impl View {
    pub fn load_preferences(&mut self) {
        match Layout::read(&self.preference_path) {
            Ok(d) => self.layout = d,
            Err(e) => {
                if self.preference_path.exists() {
                    self.writable = false;
                    self.status = Some(format!(
                        "Saved graph layout preserved; saving disabled: {e}"
                    ));
                }
            }
        }
    }
    pub fn save_preferences(&mut self) -> Result<(), String> {
        if !self.writable {
            return Err("Invalid/newer saved graph layout is preserved; saving disabled".into());
        }
        // Recheck at write time in case another app/import replaced the preference file.
        if self.preference_path.exists() {
            Layout::read(&self.preference_path)?;
        }
        self.layout.validate()?;
        let temp = self
            .preference_path
            .with_extension(format!("{}.tmp", std::process::id()));
        self.layout.export(&temp)?;
        std::fs::rename(&temp, &self.preference_path).map_err(|e| e.to_string())
    }
    pub fn metadata(&self, series: &[Vec<Datum>], selected: usize, layout: &Layout) -> Value {
        let p = Plot::new(series, layout);
        let visible = p.selected_visible(selected);
        json!({"layout":layout,"status":self.status,"preference_saving_enabled":self.writable,"plot":p,"selected_observation_visible":visible,"scope":"Display-only layout on current complete aligned recording extent; raw observations/analyses/exports stay unchanged. Visible values are originals; y_bounds are transformed display coordinates."})
    }
    pub fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.small_button("Zoom in").clicked() {
                self.layout.zoom(0.5, 0.5);
            }
            if ui.small_button("Zoom out").clicked() {
                self.layout.zoom(2., 0.5);
            }
            if ui.small_button("Earlier").clicked() {
                self.layout.pan(-0.5);
            }
            if ui.small_button("Later").clicked() {
                self.layout.pan(0.5);
            }
            if ui.small_button("Full range").clicked() {
                self.layout.window = [0., 1.];
            }
            egui::ComboBox::from_id_salt("habitat-graph-scale")
                .selected_text(self.layout.scale.label())
                .width(120.)
                .show_ui(ui, |ui| {
                    for s in [Scale::Linear, Scale::Log10, Scale::SignedLog] {
                        ui.selectable_value(&mut self.layout.scale, s, s.label());
                    }
                });
        });
    }
    pub fn settings(&mut self, ui: &mut egui::Ui) -> bool {
        let before = self.layout.clone();
        ui.strong("Recorded graph layout");
        ui.label("Display choices are reusable on other recordings. Window fractions use the current complete aligned time extent; raw values and full exports are unchanged.");
        self.toolbar(ui);
        ui.horizontal(|ui| {
            ui.label("Start / end fraction");
            let end = self.layout.window[1];
            ui.add(
                egui::DragValue::new(&mut self.layout.window[0])
                    .range(0.0..=end - 1e-6)
                    .speed(0.01)
                    .max_decimals(6),
            );
            let start = self.layout.window[0];
            ui.add(
                egui::DragValue::new(&mut self.layout.window[1])
                    .range(start + 1e-6..=1.)
                    .speed(0.01)
                    .max_decimals(6),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Y axis");
            ui.checkbox(
                &mut self.layout.include_zero,
                "Include zero (linear/signed)",
            );
        });
        let threshold_speed = (self.layout.signed_threshold.abs() * 0.02).max(1e-12);
        ui.horizontal(|ui| {
            ui.label("Signed log threshold, current metric units");
            ui.add(
                egui::DragValue::new(&mut self.layout.signed_threshold)
                    .range(1e-12..=1e12)
                    .speed(threshold_speed),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Time labels");
            for t in [TimeUnit::Seconds, TimeUnit::Hours, TimeUnit::Days] {
                ui.selectable_value(&mut self.layout.time_unit, t, t.label());
            }
        });
        ui.label("Positive log10 omits nonpositive points explicitly. Signed log: y/t inside |y| ≤ t; sign(y) × [1 + log10(|y|/t)] outside. Axis ticks and cursor values keep original units.");
        if ui.button("Save as my graph layout").clicked() {
            self.status = Some(match self.save_preferences() {
                Ok(()) => "Graph layout saved; simulation unchanged.".into(),
                Err(e) => e,
            });
        }
        ui.label("Existing layout file to import");
        ui.text_edit_singleline(&mut self.path);
        if ui
            .add_enabled(
                !self.path.trim().is_empty(),
                egui::Button::new("Import graph layout"),
            )
            .clicked()
        {
            match Layout::read(Path::new(self.path.trim())) {
                Ok(d) => {
                    self.layout = d;
                    self.status = Some("Graph layout imported; recording data unchanged.".into());
                }
                Err(e) => {
                    self.status = Some(format!("Import failed; current layout preserved: {e}"))
                }
            }
        }
        ui.label("New layout file to export");
        ui.text_edit_singleline(&mut self.output);
        if ui
            .add_enabled(
                !self.output.trim().is_empty(),
                egui::Button::new("Export protected graph layout"),
            )
            .clicked()
        {
            self.status = Some(match self.layout.export(Path::new(self.output.trim())) {
                Ok(()) => "Graph layout exported; existing files protected.".into(),
                Err(e) => e,
            });
        }
        if let Some(s) = &self.status {
            ui.colored_label(ACCENT, s);
        }
        self.layout != before
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        series: &[Vec<Datum>],
        selected: &mut usize,
        unit: &str,
        height: f32,
        connect: bool,
    ) {
        self.toolbar(ui);
        let mut p = Plot::new(series, &self.layout);
        ui.label(
            egui::RichText::new(format!(
                "{} · {} · click selects · right-drag pans · double-click resets",
                self.layout.scale.label(),
                self.layout.time_unit.label()
            ))
            .small()
            .color(MUTED),
        );
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::click_and_drag(),
        );
        let plot = egui::Rect::from_min_max(
            rect.min + egui::vec2(68., 8.),
            rect.max - egui::vec2(12., 24.),
        );
        if plot.width() < 1. || plot.height() < 1. {
            return;
        }
        if response.double_clicked() {
            self.layout.window = [0., 1.];
            p = Plot::new(series, &self.layout);
        }
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = ui.input(|i| i.pointer.delta().x);
            self.layout.pan(-delta as f64 / plot.width() as f64);
            p = Plot::new(series, &self.layout);
        }
        let span = p.window_s[1] - p.window_s[0];
        if response.clicked_by(egui::PointerButton::Primary)
            || response.dragged_by(egui::PointerButton::Primary)
        {
            if let Some(pos) = response.interact_pointer_pos() {
                let t = p.window_s[0]
                    + span * ((pos.x - plot.left()) / plot.width()).clamp(0., 1.) as f64;
                if let Some(index) = p.nearest(t) {
                    *selected = index;
                }
            }
        }
        let painter = ui.painter_at(rect);
        let marks = ui.painter().with_clip_rect(plot.intersect(ui.clip_rect()));
        painter.rect_filled(plot, 4., egui::Color32::from_rgb(11, 15, 24));
        let x = |t: f64| {
            plot.left()
                + plot.width()
                    * if span > 0. {
                        ((t - p.window_s[0]) / span) as f32
                    } else {
                        0.5
                    }
        };
        let at = |d: &Datum, y: f64, bounds: [f64; 2]| {
            egui::pos2(
                x(d.coordinate_s),
                plot.bottom() - plot.height() * ((y - bounds[0]) / (bounds[1] - bounds[0])) as f32,
            )
        };
        for n in 0..=3 {
            let fraction = n as f64 / 3.;
            let y = plot.bottom() - plot.height() * fraction as f32;
            painter.line_segment(
                [egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(40)),
            );
            if let Some(bounds) = p.y_bounds {
                let v = self.layout.scale.inverse(
                    bounds[0] + fraction * (bounds[1] - bounds[0]),
                    self.layout.signed_threshold,
                );
                let text = if v.abs() >= 1e5 || (v != 0. && v.abs() < 0.01) {
                    format!("{v:.3e}")
                } else {
                    format!("{v:.4}")
                };
                painter.text(
                    egui::pos2(plot.left() - 4., y),
                    egui::Align2::RIGHT_CENTER,
                    text,
                    egui::FontId::proportional(10.),
                    MUTED,
                );
            }
            if span > 0. || n == 0 {
                painter.text(
                    egui::pos2(
                        if span > 0. {
                            plot.left() + plot.width() * fraction as f32
                        } else {
                            plot.center().x
                        },
                        plot.bottom() + 5.,
                    ),
                    if n == 3 {
                        egui::Align2::RIGHT_TOP
                    } else {
                        egui::Align2::CENTER_TOP
                    },
                    format!(
                        "{:.3}",
                        (p.window_s[0] + span * fraction) / self.layout.time_unit.factor()
                    ),
                    egui::FontId::proportional(10.),
                    MUTED,
                );
            }
        }
        if let Some(bounds) = p.y_bounds {
            for (i, s) in p.displayed.iter().enumerate() {
                let color = if i == 0 {
                    ACCENT
                } else {
                    egui::Color32::from_rgb(91, 196, 230)
                };
                let mut previous = None;
                for d in s {
                    if let Some(y) = d
                        .value
                        .and_then(|v| self.layout.scale.forward(v, self.layout.signed_threshold))
                    {
                        let pos = at(d, y, bounds);
                        if connect {
                            if let Some(old) = previous {
                                marks.line_segment([old, pos], egui::Stroke::new(1.5_f32, color));
                            }
                        }
                        if i == 0 {
                            marks.circle_filled(pos, 2.5, color);
                        } else {
                            marks.rect_filled(
                                egui::Rect::from_center_size(pos, egui::vec2(5., 5.)),
                                0.,
                                color,
                            );
                        }
                        previous = Some(pos);
                    } else {
                        previous = None;
                    }
                }
            }
        } else {
            painter.text(
                plot.center(),
                egui::Align2::CENTER_CENTER,
                if p.scale_omitted_count > 0 {
                    "No values supported by this scale"
                } else if p.displayed.iter().all(Vec::is_empty) {
                    "No stored observations in this time window"
                } else {
                    "No supported observations for this metric"
                },
                egui::FontId::proportional(12.),
                MUTED,
            );
        }
        if let Some(d) = p
            .displayed
            .first()
            .and_then(|s| s.iter().find(|d| d.index == *selected))
        {
            marks.line_segment(
                [
                    egui::pos2(x(d.coordinate_s), plot.top()),
                    egui::pos2(x(d.coordinate_s), plot.bottom()),
                ],
                egui::Stroke::new(1.0_f32, MUTED),
            );
            if let Some((bounds, y)) = p.y_bounds.zip(
                d.value
                    .and_then(|v| self.layout.scale.forward(v, self.layout.signed_threshold)),
            ) {
                marks.circle_filled(at(d, y, bounds), 4., egui::Color32::WHITE);
            }
        } else if let Some(d) = series
            .first()
            .and_then(|s| s.iter().find(|d| d.index == *selected))
        {
            ui.horizontal(|ui| {
                ui.colored_label(ACCENT, "Selected observation is outside this window");
                if ui.small_button("Center selected").clicked() {
                    self.layout.center(d.coordinate_s, p.extent_s);
                }
            });
        }
        if p.scale_omitted_count > 0 {
            ui.colored_label(
                ACCENT,
                format!(
                    "{} original values omitted by positive log10; {} stays unchanged",
                    p.scale_omitted_count, unit
                ),
            );
        }
        if self.layout.scale == Scale::SignedLog {
            ui.label(
                egui::RichText::new(format!(
                    "Signed log threshold: {} {unit}",
                    self.layout.signed_threshold
                ))
                .small()
                .color(MUTED),
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_pan_preserve_span_at_edges_and_center_actual_coordinate() {
        let mut d = Layout::default();
        d.zoom(0.5, 0.5);
        assert_eq!(d.window, [0.25, 0.75]);
        d.pan(9.);
        assert_eq!(d.window, [0.5, 1.]);
        d.pan(-9.);
        assert_eq!(d.window, [0., 0.5]);
        d.center(105., [100., 120.]);
        assert_eq!(d.window, [0., 0.5]);
        d.zoom(1e-12, 0.5);
        assert!((d.window[1] - d.window[0] - 1e-6).abs() < 1e-15);
        assert!(d.validate().is_ok());
        d.zoom(f64::NAN, 0.5);
        assert!(d.validate().is_ok());
        d.zoom(1e30, 0.5);
        assert_eq!(d.window, [0., 1.]);
    }
    #[test]
    fn scales_keep_signed_zero_positive_domain_and_original_inverse() {
        for s in [Scale::Linear, Scale::Log10, Scale::SignedLog] {
            for v in [-100., -0.01, 0., 0.01, 1., 100.] {
                let y = s.forward(v, 0.01);
                if s == Scale::Log10 && v <= 0. {
                    assert!(y.is_none());
                } else {
                    let x = s.inverse(y.unwrap(), 0.01);
                    assert!((x - v).abs() < 1e-10);
                }
            }
        }
        assert_eq!(Scale::SignedLog.forward(-0.01, 0.01), Some(-1.));
        assert_eq!(Scale::SignedLog.forward(0.01, 0.01), Some(1.));
        assert!(Scale::Linear.forward(f64::INFINITY, 1.).is_none());
    }
    #[test]
    fn actual_irregular_coordinates_determine_visibility_and_selection() {
        let data = vec![
            vec![
                Datum {
                    index: 0,
                    coordinate_s: 0.,
                    value: Some(-4.),
                },
                Datum {
                    index: 1,
                    coordinate_s: 1.,
                    value: Some(2.),
                },
                Datum {
                    index: 2,
                    coordinate_s: 10.,
                    value: None,
                },
            ],
            vec![Datum {
                index: 0,
                coordinate_s: 20.,
                value: Some(6.),
            }],
        ];
        let d = Layout {
            window: [0., 0.5],
            ..Default::default()
        };
        let p = Plot::new(&data, &d);
        assert_eq!(p.extent_s, [0., 20.]);
        assert_eq!(p.window_s, [0., 10.]);
        assert_eq!(p.nearest(8.), Some(2));
        assert_eq!(p.supported_count, 2);
        assert_eq!(p.unavailable_count, 1);
        assert!(p.y_bounds.unwrap()[0] < -4.);
        assert!(p.displayed[1].is_empty());
        let p = Plot::new(
            &data,
            &Layout {
                window: [0.6, 0.8],
                ..d
            },
        );
        assert_eq!(p.nearest(12.), None);
        assert!(p.y_bounds.is_none());
    }
    #[test]
    fn scale_omissions_and_snapshot_are_explicit_without_fake_support() {
        let data = vec![vec![Datum {
            index: 0,
            coordinate_s: 7.,
            value: Some(0.),
        }]];
        let p = Plot::new(
            &data,
            &Layout {
                scale: Scale::Log10,
                window: [0.2, 0.8],
                ..Default::default()
            },
        );
        assert_eq!(p.window_s, [7., 7.]);
        assert_eq!(p.scale_omitted_count, 1);
        assert_eq!(p.supported_count, 0);
        assert!(p.y_bounds.is_none());
        assert!(p.selected_visible(0));
    }
    #[test]
    fn strict_bounded_layout_roundtrip_and_protected_output() {
        let path = std::env::temp_dir().join(format!("cosmogon-graph-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let d = Layout {
            window: [0.1, 0.9],
            scale: Scale::SignedLog,
            ..Default::default()
        };
        d.export(&path).unwrap();
        assert_eq!(Layout::read(&path).unwrap(), d);
        let bytes = std::fs::read(&path).unwrap();
        assert!(d.export(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let mut v = serde_json::to_value(&d).unwrap();
        v["schema_version"] = json!(99);
        std::fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(Layout::read(&path).is_err());
        v["schema_version"] = json!(1);
        v["window"] = json!([0.8, 0.2]);
        assert!(serde_json::from_value::<Layout>(v)
            .unwrap()
            .validate()
            .is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn explicit_preference_save_reopens_layout_and_rechecks_future_replacements() {
        let root = std::env::temp_dir().join(format!("cosmogon-graph-save-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let path = root.join("preferences.json");
        let mut view = View {
            preference_path: path.clone(),
            ..Default::default()
        };
        view.layout.window = [0.25, 0.5];
        view.layout.scale = Scale::SignedLog;
        view.save_preferences().unwrap();
        let mut loaded = View {
            preference_path: path.clone(),
            ..Default::default()
        };
        loaded.load_preferences();
        assert_eq!(loaded.layout, view.layout);
        let future = b"{\"format\":\"cosmogon-habitat-graph-layout\",\"schema_version\":99}";
        std::fs::write(&path, future).unwrap();
        assert!(view.save_preferences().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), future);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_existing_preference_cannot_be_overwritten() {
        let path =
            std::env::temp_dir().join(format!("cosmogon-graph-prefs-{}.json", std::process::id()));
        let bytes = b"future invalid data";
        std::fs::write(&path, bytes).unwrap();
        let mut view = View {
            preference_path: path.clone(),
            ..Default::default()
        };
        view.load_preferences();
        assert!(view.save_preferences().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
    }
}
