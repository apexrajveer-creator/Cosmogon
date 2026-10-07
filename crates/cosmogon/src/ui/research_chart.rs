//! Plot actual states. Shared elapsed-time views do not align absolute epochs or interpolate.
use super::{units, ACCENT, MUTED};
use crate::research_workspace::{Scale, TimeUnit};
use bevy_egui::egui;

pub struct Series {
    pub label: String,
    pub points: Vec<(f64, f64)>,
}
const COLORS: [egui::Color32; 8] = [
    ACCENT,
    egui::Color32::from_rgb(105, 183, 245),
    egui::Color32::from_rgb(130, 214, 161),
    egui::Color32::from_rgb(229, 137, 161),
    egui::Color32::from_rgb(192, 162, 241),
    egui::Color32::from_rgb(113, 210, 212),
    egui::Color32::from_rgb(222, 193, 127),
    egui::Color32::from_rgb(184, 198, 218),
];
#[allow(clippy::too_many_arguments)]
pub fn chart(
    ui: &mut egui::Ui,
    title: &str,
    unit: &str,
    series: &[Series],
    events: &[(f64, String)],
    range: &mut Option<(f64, f64)>,
    scale: &mut Scale,
    time_unit: TimeUnit,
    cursor: &mut Option<f64>,
) {
    ui.label(egui::RichText::new(format!("{title} ({unit})")).strong());
    if series.is_empty() || series.iter().all(|s| s.points.len() < 2) {
        ui.label("Waiting for at least two observed states…");
        return;
    }
    let duration = series
        .iter()
        .filter_map(|s| s.points.last().map(|p| p.0))
        .fold(0.0, f64::max);
    let positive = series.iter().flat_map(|s| &s.points).all(|p| p.1 > 0.0);
    let nonzero = series
        .iter()
        .all(|s| s.points.first().is_some_and(|p| p.1 != 0.0));
    if (*scale == Scale::Log && !positive) || (*scale == Scale::Fractional && !nonzero) {
        *scale = Scale::Linear;
        ui.label("Scale returned to linear: these states do not support the saved transform.");
    }
    let mut fraction = range.unwrap_or((0.0, 1.0));
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(duration > 0.0, egui::Button::new("Zoom in").small())
            .clicked()
        {
            let center = (fraction.0 + fraction.1) * 0.5;
            let half = ((fraction.1 - fraction.0) * 0.25).max(5e-6);
            fraction = (center - half, center + half);
            *range = Some(fraction);
        }
        if ui
            .add_enabled(duration > 0.0, egui::Button::new("Zoom out").small())
            .clicked()
        {
            let center = (fraction.0 + fraction.1) * 0.5;
            let half = fraction.1 - fraction.0;
            fraction = ((center - half).max(0.0), (center + half).min(1.0));
            *range = Some(fraction);
        }
        if ui.small_button("Show all").clicked() {
            *range = None;
            fraction = (0.0, 1.0);
        }
        ui.selectable_value(scale, Scale::Linear, "Linear");
        if ui
            .add_enabled(
                positive,
                egui::Button::new("Log")
                    .small()
                    .selected(*scale == Scale::Log),
            )
            .on_hover_text("Positive recorded values only")
            .clicked()
        {
            *scale = Scale::Log;
        }
        if ui
            .add_enabled(
                nonzero,
                egui::Button::new("Change from start")
                    .small()
                    .selected(*scale == Scale::Fractional),
            )
            .on_hover_text("Each series uses its own first value; this is not an accuracy estimate")
            .clicked()
        {
            *scale = Scale::Fractional;
        }
    });
    let window = (fraction.0 * duration, fraction.1 * duration);
    let transform = |v: f64, first: f64| match scale {
        Scale::Linear => v,
        Scale::Log => v.log10(),
        Scale::Fractional => (v - first) / first.abs(),
    };
    if series.iter().any(|s| {
        s.points.first().is_some_and(|first| {
            s.points
                .iter()
                .any(|p| !transform(p.1, first.1).is_finite())
        })
    }) {
        ui.label("This transform exceeds finite numeric range. Use linear scale or inspect the exported values.");
        return;
    }
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for s in series {
        if let Some(first) = s.points.first() {
            for p in s
                .points
                .iter()
                .filter(|p| p.0 >= window.0 && p.0 <= window.1)
            {
                let v = transform(p.1, first.1);
                low = low.min(v);
                high = high.max(v);
            }
        }
    }
    if !low.is_finite() || !high.is_finite() {
        low = 0.0;
        high = 1.0;
    }
    if low == high {
        let pad = low.abs().max(1.0) * 1e-6;
        low -= pad;
        high += pad;
    }
    if !(high - low).is_finite() {
        ui.label(
            "The vertical range exceeds this graph's numeric limits. Inspect the exported values.",
        );
        return;
    }
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(180.0), 160.0),
        egui::Sense::click_and_drag(),
    );
    let plot = rect.shrink2(egui::vec2(8.0, 20.0));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(19, 25, 36));
    let screen = |p: (f64, f64), first: f64| {
        egui::pos2(
            plot.left()
                + ((p.0 - window.0) / (window.1 - window.0).max(f64::EPSILON)) as f32
                    * plot.width(),
            plot.bottom() - ((transform(p.1, first) - low) / (high - low)) as f32 * plot.height(),
        )
    };
    for (i, s) in series.iter().enumerate() {
        if let Some(first) = s.points.first() {
            let points = s
                .points
                .iter()
                .filter(|p| p.0 >= window.0 && p.0 <= window.1)
                .map(|p| screen(*p, first.1))
                .collect::<Vec<_>>();
            if points.len() >= 2 {
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(1.8_f32, COLORS[i % COLORS.len()]),
                ));
            }
        }
    }
    for (t, _) in events
        .iter()
        .filter(|(t, _)| *t >= window.0 && *t <= window.1)
        .take(80)
    {
        let x = plot.left()
            + ((*t - window.0) / (window.1 - window.0).max(f64::EPSILON)) as f32 * plot.width();
        painter.line_segment(
            [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
            egui::Stroke::new(0.8_f32, MUTED.gamma_multiply(0.6)),
        );
    }
    let label = |v: f64| {
        if *scale == Scale::Log {
            units::fmt_num(10.0f64.powf(v))
        } else {
            units::fmt_num(v)
        }
    };
    painter.text(
        rect.left_top() + egui::vec2(8.0, 2.0),
        egui::Align2::LEFT_TOP,
        label(high),
        egui::FontId::proportional(11.0),
        MUTED,
    );
    painter.text(
        rect.left_bottom() + egui::vec2(8.0, -2.0),
        egui::Align2::LEFT_BOTTOM,
        label(low),
        egui::FontId::proportional(11.0),
        MUTED,
    );
    painter.text(
        rect.right_bottom() + egui::vec2(-8.0, -2.0),
        egui::Align2::RIGHT_BOTTOM,
        format!(
            "{}–{} {}",
            units::fmt_num(window.0 / time_unit.factor()),
            units::fmt_num(window.1 / time_unit.factor()),
            time_unit.label()
        ),
        egui::FontId::proportional(11.0),
        MUTED,
    );
    if response.clicked() {
        response.request_focus();
    }
    if let Some(pointer) = response.hover_pos() {
        *cursor = Some(
            window.0
                + ((pointer.x - plot.left()) / plot.width()).clamp(0.0, 1.0) as f64
                    * (window.1 - window.0),
        );
    }
    if response.has_focus() {
        let points = &series[0].points;
        if !points.is_empty() {
            let current = cursor.unwrap_or(0.0);
            let i = points
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| (a.0 - current).abs().total_cmp(&(b.0 - current).abs()))
                .map(|(i, _)| i)
                .unwrap_or(0);
            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                *cursor = Some(points[i.saturating_sub(1)].0);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                *cursor = Some(points[(i + 1).min(points.len() - 1)].0);
            }
        }
    }
    if response.dragged() && range.is_some() && duration > 0.0 {
        let shift = -(response.drag_delta().x / plot.width()) as f64 * (fraction.1 - fraction.0);
        let width = fraction.1 - fraction.0;
        let start = (fraction.0 + shift).clamp(0.0, (1.0 - width).max(0.0));
        *range = Some((start, start + width));
    }
    if let Some(time) = *cursor {
        if time >= window.0 && time <= window.1 {
            let x = plot.left()
                + ((time - window.0) / (window.1 - window.0).max(f64::EPSILON)) as f32
                    * plot.width();
            painter.line_segment(
                [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
                egui::Stroke::new(0.8_f32, MUTED),
            );
        }
    }
    for (i, s) in series.iter().enumerate() {
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(COLORS[i % COLORS.len()], &s.label);
            if let Some(time) = *cursor {
                if let Some(p) = s
                    .points
                    .iter()
                    .min_by(|a, b| (a.0 - time).abs().total_cmp(&(b.0 - time).abs()))
                {
                    ui.monospace(format!(
                        "{:.10e} {} at {} {}",
                        p.1,
                        unit,
                        units::fmt_num(p.0 / time_unit.factor()),
                        time_unit.label()
                    ));
                    if let Some(first) = s.points.first() {
                        if *scale == Scale::Fractional {
                            ui.label(format!("Δ / |initial| = {:.6e}", transform(p.1, first.1)));
                        }
                        painter.circle_filled(screen(*p, first.1), 3.0, COLORS[i % COLORS.len()]);
                    }
                }
            }
        });
    }
    if let Some(time) = *cursor {
        if let Some((_, name)) = events
            .iter()
            .min_by(|a, b| (a.0 - time).abs().total_cmp(&(b.0 - time).abs()))
            .filter(|e| (e.0 - time).abs() <= duration / 100.0)
        {
            ui.label(format!("Nearby event: {name}"));
        }
    }
    ui.label(egui::RichText::new("Shared elapsed-time range and cursor; nearest actual state in each series. Click a graph, then use ←/→ to inspect states. No interpolation or uncertainty band.").small().color(MUTED));
}
