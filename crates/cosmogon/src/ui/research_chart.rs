//! Inspect actual samples without interpolating values or inventing uncertainty bands.
use super::{units, ACCENT, MUTED};
use bevy_egui::egui;

#[derive(Clone, Default)]
struct View {
    range: Option<(f64, f64)>,
    logarithmic: bool,
    relative: bool,
}

pub fn chart(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    title: &str,
    unit: &str,
    points: &[(f64, f64)],
    events: &[(f64, String)],
) {
    let id = ui.make_persistent_id(id);
    let mut view = ui
        .ctx()
        .data_mut(|d| d.get_temp::<View>(id))
        .unwrap_or_default();
    ui.label(egui::RichText::new(format!("{title} ({unit})")).strong());
    if points.len() < 2 {
        ui.label("Waiting for at least two observed states…");
        return;
    }
    let full = (points[0].0, points[points.len() - 1].0);
    let span = (full.1 - full.0).max(f64::EPSILON);
    let mut range = view.range.unwrap_or(full);
    if range.1 < full.0 || range.0 > full.1 {
        range = full;
    }
    ui.horizontal(|ui| {
        if ui.small_button("Zoom in").clicked(){let middle=(range.0+range.1)*0.5;let half=(range.1-range.0)*0.25;range=(middle-half,middle+half);view.range=Some(range);}
        if ui.small_button("Zoom out").clicked(){let middle=(range.0+range.1)*0.5;let half=(range.1-range.0).max(span/1024.0);range=((middle-half).max(full.0),(middle+half).min(full.1));view.range=Some(range);}
        if ui.small_button("Show all").clicked(){view.range=None;range=full;}
        if ui.add_enabled(points.iter().all(|p|p.1>0.0) && !view.relative,egui::Checkbox::new(&mut view.logarithmic,"Log scale")).on_hover_text("Available for positive measurements only").changed(){view.relative=false;}
        if ui.add_enabled(points[0].1!=0.0,egui::Checkbox::new(&mut view.relative,"Change from start")).on_hover_text("(value − initial value) / |initial value|; this is a change, not a scientific error estimate").changed(){view.logarithmic=false;}
    });
    let transform = |v: f64| {
        if view.relative {
            (v - points[0].1) / points[0].1.abs()
        } else if view.logarithmic {
            v.log10()
        } else {
            v
        }
    };
    let visible: Vec<_> = points
        .iter()
        .copied()
        .filter(|p| p.0 >= range.0 && p.0 <= range.1 && p.0.is_finite() && p.1.is_finite())
        .collect();
    let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
    for p in &visible {
        let v = transform(p.1);
        low = low.min(v);
        high = high.max(v);
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
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(180.0), 145.0),
        egui::Sense::drag(),
    );
    let plot = rect.shrink2(egui::vec2(8.0, 18.0));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(19, 25, 36));
    let to_screen = |p: (f64, f64)| {
        egui::pos2(
            plot.left()
                + ((p.0 - range.0) / (range.1 - range.0).max(f64::EPSILON)) as f32 * plot.width(),
            plot.bottom() - ((transform(p.1) - low) / (high - low)) as f32 * plot.height(),
        )
    };
    if visible.len() >= 2 {
        painter.add(egui::Shape::line(
            visible.iter().copied().map(to_screen).collect(),
            egui::Stroke::new(1.8_f32, ACCENT),
        ));
    }
    for (time, _) in events
        .iter()
        .filter(|(t, _)| *t >= range.0 && *t <= range.1)
        .take(80)
    {
        let x = plot.left()
            + ((*time - range.0) / (range.1 - range.0).max(f64::EPSILON)) as f32 * plot.width();
        painter.line_segment(
            [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
            egui::Stroke::new(0.8_f32, egui::Color32::from_rgb(104, 94, 124)),
        );
    }
    let label = |v| {
        if view.logarithmic {
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
        format!("{}–{} s", units::fmt_num(range.0), units::fmt_num(range.1)),
        egui::FontId::proportional(11.0),
        MUTED,
    );
    if let Some(pointer) = response.hover_pos() {
        let x = range.0
            + ((pointer.x - plot.left()) / plot.width()).clamp(0.0, 1.0) as f64
                * (range.1 - range.0);
        if let Some(p) = visible
            .iter()
            .min_by(|a, b| (a.0 - x).abs().total_cmp(&(b.0 - x).abs()))
        {
            let screen = to_screen(*p);
            painter.circle_filled(screen, 3.5, ACCENT);
            let mut text = format!(
                "Observed state: {} s\n{} {}",
                units::fmt_num(p.0),
                units::fmt_num(p.1),
                unit
            );
            if view.relative {
                text.push_str(&format!(
                    "\nFractional change: {}",
                    units::fmt_num(transform(p.1))
                ));
            }
            if let Some((_, event)) = events
                .iter()
                .min_by(|a, b| (a.0 - p.0).abs().total_cmp(&(b.0 - p.0).abs()))
                .filter(|(t, _)| (*t - p.0).abs() <= span / 100.0)
            {
                text.push_str(&format!("\nNearby event: {event}"));
            }
            response.clone().on_hover_text(text);
        }
    }
    if response.dragged() && view.range.is_some() {
        let shift = -(response.drag_delta().x / plot.width()) as f64 * (range.1 - range.0);
        let width = range.1 - range.0;
        let start = (range.0 + shift).clamp(full.0, (full.1 - width).max(full.0));
        view.range = Some((start, start + width));
    }
    ui.label(egui::RichText::new(if view.relative {"Vertical axis: fractional change from initial reading. Zoom buttons and horizontal drag change the time range."}else{"Horizontal axis: elapsed simulation seconds. Hover inspects the nearest recorded state; vertical markers are observed events."}).small().color(MUTED));
    ui.ctx().data_mut(|d| d.insert_temp(id, view));
}
