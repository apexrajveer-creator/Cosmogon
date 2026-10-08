//! Independent recorded observations, never edits or fabricated live history.
use super::{ACCENT, MUTED};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_egui::egui;
use cosmogon_sim::{
    habitat_comparison::{self, Alignment},
    habitat_recording::{Metric, Recording},
    habitat_sweep::{self, Parameter, Plan},
    Universe,
};

fn parse_levels(s: &str) -> Result<Vec<f64>,String> {
    s.split(',').map(|s|s.trim().parse::<f64>().map_err(|_|"Enter finite comma-separated values".into())).collect()
}
pub fn check_input(path: &std::path::Path,sample: Option<usize>) -> Result<(),String> {
    let count=if path.join("plan.json").exists() {cosmogon_sim::habitat_factorial::read(path)?.recordings[0].rows.len()} else {Recording::read(path)?.rows.len()};
    if sample.is_some_and(|i|i>=count) {return Err("Requested habitat observation is outside the recording".into());}
    Ok(())
}
fn open_design(location:std::path::PathBuf,path:String,sample:Option<usize>) -> Result<SweepResult,String> {
    let mut design=cosmogon_sim::habitat_factorial::read(&location)?;
    let b=design.recordings.pop().ok_or("Empty paired design")?;
    let a=design.recordings.remove(0);
    Ok(SweepResult {a,b,manifest:design.manifest,path,sample})
}
pub struct Launch {
    pub plan: Plan,
    pub id: Option<u64>,
    pub output: String,
    pub sample: Option<usize>,
    pub initial_path: Option<String>,
}
fn run_sweep(
    base: Universe,
    id: u64,
    plan: Plan,
    path: String,
    sample: Option<usize>,
) -> Result<SweepResult, String> {
    let manifest = habitat_sweep::execute(&base, id, &plan, std::path::Path::new(&path))?;
    let a = Recording::read(&std::path::Path::new(&path).join("run-000"))?;
    let b = Recording::read(
        &std::path::Path::new(&path).join(format!("run-{:03}", plan.branch_count() - 1)),
    )?;
    Ok(SweepResult {
        a,
        b,
        manifest,
        path,
        sample,
    })
}
struct SweepResult {
    a: Recording,
    b: Recording,
    manifest: serde_json::Value,
    path: String,
    sample: Option<usize>,
}
struct SweepForm {
    parameter: Parameter,
    values: String,
    samples: usize,
    interval_hours: f64,
    output: String,
    paired: bool,
    secondary_parameter: Parameter,
    secondary_values: String,
}
impl Default for SweepForm {
    fn default() -> Self {
        Self {
            parameter: Parameter::FirmPowerW,
            values: "0, 2000, 4000".into(),
            samples: 25,
            interval_hours: 1.0,
            output: String::new(),
            paired: false,
            secondary_parameter: Parameter::WaterRecoveryFraction,
            secondary_values: "0.8, 0.95".into(),
        }
    }
}

#[derive(Default)]
pub struct Study {
    pub recording: Option<Recording>,
    pub comparison: Option<Recording>,
    pub experiment_mode: bool,
    pub factorial_mode: bool,
    pub why_mode: bool,
    pub why: super::habitat_why::View,
    pub alignment: Alignment,
    comparison_path: String,
    comparison_output: String,
    comparison_task: Option<Task<Result<Recording, String>>>,
    pending_comparison_path: String,
    sweep_task: Option<Task<Result<SweepResult, String>>>,
    sweep: SweepForm,
    pub sweep_manifest: Option<serde_json::Value>,
    pub launch: Option<Launch>,
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
        if location.join("plan.json").exists() {
            self.factorial_mode = true;
            self.sweep_task=Some(AsyncComputeTaskPool::get().spawn(async move {open_design(location,path,sample)}));
            self.status=Some("Checking all four common-state branches and response analysis…".into());
            return;
        }
        self.task =
            Some(AsyncComputeTaskPool::get().spawn(async move { Recording::read(&location) }));
        self.pending_path = path;
        self.requested_sample = sample;
        self.status = Some("Checking recorded sources, endpoints and observations…".into());
    }
    pub fn pending(&self) -> bool {
        self.task.is_some()
            || self.comparison_task.is_some()
            || self.sweep_task.is_some()
            || self.launch.is_some()
    }
    pub fn start_comparison(&mut self, path: String) {
        let location = std::path::PathBuf::from(&path);
        self.comparison_task =
            Some(AsyncComputeTaskPool::get().spawn(async move { Recording::read(&location) }));
        self.pending_comparison_path = path;
        self.status = Some("Checking comparison recording…".into());
    }
    pub fn start_sweep(
        &mut self,
        base: Universe,
        id: u64,
        plan: Plan,
        path: String,
        sample: Option<usize>,
    ) {
        self.factorial_mode = plan.secondary.is_some();
        self.sweep_task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { run_sweep(base, id, plan, path, sample) }),
        );
        self.status = Some("Running independent habitat branches; live world is unchanged…".into());
    }
    pub fn launch_if_ready(&mut self, base: &Universe, selected: Option<u64>, reference: bool) {
        if let Some(launch) = self.launch.take() {
            if reference {
                self.status =
                    Some("Clone the reference world before running habitat experiments.".into());
                return;
            }
            if let Some(file) = launch.initial_path {
                self.sweep_task = Some(AsyncComputeTaskPool::get().spawn(async move {
                    let base = cosmogon_sim::save::read_file(std::path::Path::new(&file))
                        .map_err(|e| e.to_string())?;
                    let id = launch
                        .id
                        .or_else(|| (base.habitats.len() == 1).then(|| base.habitats[0].id))
                        .ok_or("Choose a habitat ID in the saved common world")?;
                    run_sweep(base, id, launch.plan, launch.output, launch.sample)
                }));
                self.status = Some(
                    "Running from the exact saved common world; live state remains independent…"
                        .into(),
                );
                return;
            }
            let id = launch
                .id
                .or(selected)
                .or_else(|| (base.habitats.len() == 1).then(|| base.habitats[0].id));
            if let Some(id) = id {
                self.start_sweep(base.clone(), id, launch.plan, launch.output, launch.sample);
            } else {
                self.status = Some("Choose a habitat in the common starting world.".into());
            }
        }
    }
    fn commit_sweep(&mut self, result: SweepResult) -> Result<(), String> {
        let index = result.sample.unwrap_or(0);
        if index >= result.a.rows.len() {
            return Err("Requested observation index is outside the completed sweep".into());
        }
        let count = result.manifest["runs"]
            .as_array()
            .filter(|runs| runs.len() >= 2)
            .ok_or("Missing completed runs")?
            .len();
        let path = format!("{}/run-000", result.path);
        let comparison_path = format!("{}/run-{:03}", result.path, count - 1);
        self.path = path;
        self.comparison_path = comparison_path;
        self.experiment_mode = false;
        self.recording = Some(result.a);
        self.comparison = Some(result.b);
        self.selected = index;
        self.factorial_mode &= result.manifest["factorial_analysis"].is_object();
        self.sweep_manifest = Some(result.manifest);
        self.status = Some(format!(
            "Completed sweep in {}. First and last runs opened; every branch is retained.",
            result.path
        ));
        Ok(())
    }
    fn commit(&mut self, r: Recording, path: String, sample: Option<usize>) -> Result<(), String> {
        let selected = sample.unwrap_or(0);
        if selected >= r.rows.len() {
            return Err("Requested observation index is outside the recording".into());
        }
        self.sweep_manifest = None;
        self.factorial_mode = false;
        self.recording = Some(r);
        self.path = path;
        self.selected = selected;
        self.status = Some("Recorded observations reopened. The live world is unchanged.".into());
        Ok(())
    }
    pub fn poll(&mut self) {
        self.why.poll();
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
        if let Some(result) = self
            .comparison_task
            .as_mut()
            .and_then(|task| block_on(future::poll_once(task)))
        {
            self.comparison_task = None;
            match result {
                Ok(r) => {
                    self.sweep_manifest = None;
                    self.factorial_mode = false;
                    self.comparison = Some(r);
                    self.comparison_path = self.pending_comparison_path.clone();
                    self.status=Some("Comparison observations reopened. Exact stored-time alignment; no live-world change.".into());
                }
                Err(e) => {
                    self.status = Some(format!(
                        "Could not open comparison: {e}. Previous comparison is preserved."
                    ))
                }
            }
        }
        if let Some(result) = self
            .sweep_task
            .as_mut()
            .and_then(|task| block_on(future::poll_once(task)))
        {
            self.sweep_task = None;
            if let Err(e) = result.and_then(|r| self.commit_sweep(r)) {
                self.status=Some(format!("Sweep failed: {e}. Previous recorded views are preserved. A folder without completion manifest is incomplete."));
            }
        }
    }
    pub fn metadata(&self) -> Option<serde_json::Value> {
        let r = self.recording.as_ref()?;
        let row = &r.rows[self.selected.min(r.rows.len() - 1)];
        Some(
            serde_json::json!({"summary":r.summary,"metric":self.metric.id(),"unit":self.metric.unit(),"selected_observation":self.selected,"selected_row":row,"why":self.why_mode.then(||self.why.metadata(&row.habitat,row.simulation_time_s,"recorded_completed_interval")),"selected_value":self.metric.value(row),"comparison":self.comparison.as_ref().map(|b| {let pairs=habitat_comparison::pairs(r,b,self.metric,self.alignment);let pair=&pairs[self.selected.min(pairs.len()-1)];serde_json::json!({"summary":b.summary,"alignment":self.alignment,"selected_pair":pair,"selected_row":pair.b_index.map(|j|&b.rows[j]),"report":habitat_comparison::report(r,b,self.metric,self.alignment)})}),"sweep":self.sweep_manifest,"paired_results_visible":self.factorial_mode,"scope":"Independent recorded view at capture request, possibly hidden; no live-world or camera change"}),
        )
    }
}
fn value_text(value: Option<f64>) -> String {
    value
        .map(|v| {
            if v != 0. && (v.abs() < 1e-5 || v.abs() > 1e8) {
                format!("{v:.12e}")
            } else {
                format!("{v:.9}")
            }
        })
        .unwrap_or_else(|| "unavailable".into())
}
fn graph(
    ui: &mut egui::Ui,
    r: &Recording,
    b: Option<&Recording>,
    alignment: Alignment,
    metric: Metric,
    selected: &mut usize,
) {
    ui.label(format!(
        "{} ({}) vs {}",
        metric.label(),
        metric.unit(),
        if alignment == Alignment::Elapsed {
            "elapsed terrestrial hours"
        } else {
            "world seconds (J2000)"
        }
    ));
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 120.0),
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
        .chain(b.into_iter().flat_map(|b| b.rows.iter()))
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
    let first = r.rows[0].simulation_time_s;
    let b_first = b.map(|b| b.rows[0].simulation_time_s).unwrap_or(first);
    let start = alignment.coordinate(first, first).min(
        b.map(|_| alignment.coordinate(b_first, b_first))
            .unwrap_or(alignment.coordinate(first, first)),
    );
    let end = alignment
        .coordinate(r.rows.last().unwrap().simulation_time_s, first)
        .max(
            b.map(|b| alignment.coordinate(b.rows.last().unwrap().simulation_time_s, b_first))
                .unwrap_or(start),
        );
    let duration = end - start;
    let x = |row: &cosmogon_sim::habitat::Sample, origin: f64| {
        plot.left()
            + plot.width()
                * ((alignment.coordinate(row.simulation_time_s, origin) - start)
                    / duration.max(1.0)) as f32
    };
    let at = |row: &cosmogon_sim::habitat::Sample, v: f64, origin: f64| {
        egui::pos2(
            x(row, origin),
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
            if alignment == Alignment::Elapsed {
                format!("{:.2}", (start + duration * n as f64 / 3.0) / 3600.0)
            } else {
                format!("{:.9e}", start + duration * n as f64 / 3.0)
            },
            egui::FontId::proportional(10.0),
            MUTED,
        );
    }
    for (series, origin, color, partner) in std::iter::once((r, first, ACCENT, false)).chain(
        b.into_iter()
            .map(|b| (b, b_first, egui::Color32::from_rgb(91, 196, 230), true)),
    ) {
        let mut previous = None;
        for row in &series.rows {
            if let Some(v) = metric.value(row) {
                let p = at(row, v, origin);
                if let Some(old) = previous {
                    painter.line_segment([old, p], egui::Stroke::new(1.7_f32, color));
                }
                if partner {
                    painter.rect_filled(
                        egui::Rect::from_center_size(p, egui::vec2(4., 4.)),
                        0.,
                        color,
                    );
                } else {
                    painter.circle_filled(p, 2., color);
                }
                previous = Some(p);
            } else {
                previous = None;
            }
        }
    }
    if response.clicked() || response.dragged() {
        if let Some(p) = response.interact_pointer_pos() {
            let target =
                ((p.x - plot.left()) / plot.width()).clamp(0.0, 1.0) as f64 * duration + start;
            *selected = r
                .rows
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    (alignment.coordinate(a.simulation_time_s, first) - target)
                        .abs()
                        .total_cmp(
                            &(alignment.coordinate(b.simulation_time_s, first) - target).abs(),
                        )
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }
    let row = &r.rows[(*selected).min(r.rows.len() - 1)];
    let cursor = x(row, first);
    painter.line_segment(
        [
            egui::pos2(cursor, plot.top()),
            egui::pos2(cursor, plot.bottom()),
        ],
        egui::Stroke::new(1.0_f32, MUTED),
    );
    if let Some(v) = metric.value(row) {
        painter.circle_filled(at(row, v, first), 4.0, egui::Color32::WHITE);
    }
}
pub fn panel(
    ui: &mut egui::Ui,
    study: &mut Study,
    world: &Universe,
    habitat_id: &mut Option<u64>,
    reference: bool,
) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut study.experiment_mode,
            false,
            "Recordings & comparisons",
        );
        ui.selectable_value(&mut study.experiment_mode, true, "Run experiment");
    });
    ui.horizontal_wrapped(|ui| {
        if ui.selectable_label(!study.why_mode && !study.factorial_mode,"Curves & comparison").clicked() {study.why_mode=false;study.factorial_mode=false;}
        if ui.selectable_label(study.why_mode && !study.factorial_mode,"WHY selected interval").clicked() {study.why_mode=true;study.factorial_mode=false;}
        if study.sweep_manifest.as_ref().is_some_and(|m|m["factorial_analysis"].is_object()) && ui.selectable_label(study.factorial_mode,"Paired results").clicked() {study.factorial_mode=true;study.why_mode=false;}
    });
    if study.factorial_mode && !study.experiment_mode {paired_results(ui,study);return;}

    if study.why_mode && !study.experiment_mode {
        ui.label(egui::RichText::new("Recorded evidence · independent of the live world").color(ACCENT));
        if let Some(r)=&study.recording {
            study.selected=study.selected.min(r.rows.len()-1);
            ui.label(format!("A: {} · {} recorded observations",r.summary["name"].as_str().unwrap_or("Habitat"),r.rows.len()));
            ui.add(egui::Slider::new(&mut study.selected,0..=r.rows.len()-1).text("Observation index"));
            let row=&r.rows[study.selected];
            study.why.show(ui,&row.habitat,row.simulation_time_s,false);
            ui.label(egui::RichText::new("This explains A's exact saved interval. Return to Curves & comparison for supported B−A differences; no unique cause is attributed across changed initial conditions.").small().color(MUTED));
        } else {ui.label("Open a recording in Curves & comparison first.");}
        return;
    }
    if study.experiment_mode {
        experiment(ui, study, world, habitat_id, reference);
        if let Some(status) = &study.status {
            ui.colored_label(ACCENT, status);
        }
        return;
    }
    if study.sweep_manifest.as_ref().is_some_and(|m|m["factorial_analysis"].is_object()) {ui.colored_label(ACCENT,"These two corners change both settings. Paired results shows all four combinations and conditional effects.");}
    ui.label(
        egui::RichText::new("Actual exported observations · independent of the live world")
            .color(ACCENT),
    );
    if let Some(r) = &study.recording {
        ui.label(format!(
            "A: {} · world [{},{}] · {} stored observations",
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
        if let Some(b) = &study.comparison {
            ui.label(format!(
                "B: {} · {} observations",
                b.summary["name"].as_str().unwrap_or("Habitat"),
                b.rows.len()
            ));
            ui.horizontal(|ui| {
                ui.colored_label(ACCENT, "● A");
                ui.colored_label(egui::Color32::from_rgb(91, 196, 230), "■ B");
                ui.selectable_value(&mut study.alignment, Alignment::Elapsed, "Elapsed time");
                ui.selectable_value(&mut study.alignment, Alignment::World, "World time");
            });
        }
        graph(
            ui,
            r,
            study.comparison.as_ref(),
            study.alignment,
            study.metric,
            &mut study.selected,
        );
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
        if let Some(b) = &study.comparison {
            let pairs = habitat_comparison::pairs(r, b, study.metric, study.alignment);
            let pair = &pairs[study.selected];
            ui.strong(format!(
                "B: {} {} · B−A: {} {}",
                value_text(pair.b_value),
                study.metric.unit(),
                value_text(pair.b_minus_a),
                study.metric.unit()
            ));
            ui.label(format!(
                "Comparison support: {} · B world time {} s",
                match pair.support {
                    "matched" => "Matched stored observation",
                    "no_exact_timestamp" => "No observation at this exact time",
                    "different_interval_support" =>
                        "Different completed intervals; subtraction unavailable",
                    _ => "One recorded value is unavailable",
                },
                pair.b_world_time_s
                    .map(|t| format!("{t:.3}"))
                    .unwrap_or_else(|| "no matching observation".into())
            ));
            let ah = &r.rows[0].habitat;
            let bh = &b.rows[0].habitat;
            if ah.definition != bh.definition || ah.config != bh.config || ah.state != bh.state {
                ui.label(
                    egui::RichText::new(
                        "Initial conditions differ; inspect settings before attributing a cause.",
                    )
                    .small()
                    .color(ACCENT),
                );
            }
            egui::CollapsingHeader::new("Compare original conditions and export differences").show(
                ui,
                |ui| {
                    let report = habitat_comparison::report(r, b, study.metric, study.alignment);
                    ui.label(format!(
                        "Initial conditions differ: {} · source versions differ: {}",
                        report["initial_conditions_differ"], report["source_versions_differ"]
                    ));
                    ui.label(report["limits"].as_str().unwrap_or_default());
                    ui.label(format!("A: {} · B: {}", study.path, study.comparison_path));
                    if ui.button("Copy complete comparison JSON").clicked() {
                        ui.ctx()
                            .copy_text(serde_json::to_string_pretty(&report).unwrap_or_default());
                    }
                    ui.label("New comparison output folder");
                    ui.text_edit_singleline(&mut study.comparison_output);
                    if ui.button("Export actual comparison JSON and CSV").clicked() {
                        study.status = Some(
                            match habitat_comparison::export(
                                r,
                                b,
                                study.metric,
                                study.alignment,
                                std::path::Path::new(&study.comparison_output),
                            ) {
                                Ok(()) => {
                                    "Comparison exported; prior folders are protected.".into()
                                }
                                Err(e) => e,
                            },
                        );
                    }
                    egui::ScrollArea::vertical()
                        .max_height(160.)
                        .show(ui, |ui| {
                            ui.code(serde_json::to_string_pretty(&report).unwrap_or_default());
                        });
                },
            );
        }
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
                let reports=if let Some(b)=&study.comparison {let pairs=habitat_comparison::pairs(r,b,study.metric,study.alignment);serde_json::json!({"a":row.habitat.report(),"b":pairs[study.selected].b_index.map(|j|b.rows[j].habitat.report())})} else {row.habitat.report()};
                let data = serde_json::to_string_pretty(&reports).unwrap_or_default();
                if ui.button("Copy selected reports JSON").clicked() {
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
    ui.label("Comparison study folder (B)");
    ui.text_edit_singleline(&mut study.comparison_path);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                !study.pending() && study.recording.is_some(),
                egui::Button::new("Open comparison"),
            )
            .clicked()
        {
            study.start_comparison(study.comparison_path.clone());
        }
        if ui
            .add_enabled(
                !study.pending() && study.comparison.is_some(),
                egui::Button::new("Remove comparison"),
            )
            .clicked()
        {
            study.comparison = None;
            study.sweep_manifest = None;
        }
    });
    if let Some(status) = &study.status {
        ui.colored_label(ACCENT, status);
    }
}
fn paired_results(ui:&mut egui::Ui,study:&mut Study) {
    let Some(m)=&study.sweep_manifest else {ui.label("Open a completed paired experiment first.");return;};
    let report=&m["factorial_analysis"];
    ui.strong("Paired settings · four actual final observations");
    ui.label(format!("A: {} · {} {}", report["a"]["parameter"].as_str().unwrap_or_default(), report["a"]["levels"],report["a"]["unit"].as_str().unwrap_or_default()));
    ui.label(format!("B: {} · {} {}", report["b"]["parameter"].as_str().unwrap_or_default(), report["b"]["levels"],report["b"]["unit"].as_str().unwrap_or_default()));
    egui::ComboBox::from_id_salt("paired-response-metric").selected_text(study.metric.label()).show_ui(ui,|ui| {for metric in Metric::ALL {ui.selectable_value(&mut study.metric,metric,metric.label());}});
    if report["completed_interval_configurations_match_current"].as_array().is_some_and(|values|values.iter().any(|v|v!=true)) {ui.colored_label(ACCENT,"Flows retain pre-edit settings. Advance before interpreting changed equipment.");}
    let metric=&report["metrics"][study.metric.id()];let c=&metric["contrasts"];let responses=&c["responses"];
    ui.label(format!("Final stored world time {} s · observation {} · {}",report["simulation_time_s"],report["observation_index"],study.metric.unit()));
    egui::Grid::new("paired-responses").striped(true).show(ui,|ui| {
        ui.label("");ui.strong("A0 (first level)");ui.strong("A1 (second level)");ui.end_row();
        for (label,left,right) in [("B0 (first)",0,1),("B1 (second)",2,3)] {
            ui.strong(label);ui.label(value_text(responses[left].as_f64()));ui.label(value_text(responses[right].as_f64()));ui.end_row();
        }
    });
    egui::Grid::new("paired-contrasts").striped(true).show(ui,|ui| {
        for (label,key) in [("A change at B0","a_at_b0"),("A change at B1","a_at_b1"),("B change at A0","b_at_a0"),("B change at A1","b_at_a1"),("Average A change","average_a"),("Average B change","average_b"),("Difference of conditional A changes","difference_in_differences"),("Standard AB effect (half that difference)","standard_ab_effect")] {
            ui.label(label);ui.strong(value_text(c[key].as_f64()));ui.end_row();
        }
    });
    if metric["support"]!="matched" {ui.colored_label(ACCENT,"Contrasts unavailable: missing value or mismatched interval support.");}
    ui.label(egui::RichText::new("Conditional model results at these levels. No statistical significance, predictive confidence, global optimum or hardware calibration. Graph endpoints change both settings.").small().color(MUTED));
    if ui.button("Copy complete four-branch analysis JSON").clicked() {ui.ctx().copy_text(serde_json::to_string_pretty(report).unwrap_or_default());}
    ui.hyperlink_to("NIST factorial design method",cosmogon_sim::habitat_factorial::METHOD_SOURCE);
    ui.label(egui::RichText::new(format!("All four protected recordings and factorial.json: {}",study.path.rsplit_once("/run-").map(|p|p.0).unwrap_or(&study.path))).small().color(MUTED));
}
fn experiment(
    ui: &mut egui::Ui,
    study: &mut Study,
    world: &Universe,
    habitat_id: &mut Option<u64>,
    reference: bool,
) {
    ui.strong("Controlled habitat experiment");
    ui.label("Vary settings in independent copies of the current world. Initial stocks stay fixed; the live world continues unchanged. Equipment/environment are assumptions; this measures modeled sensitivity.");
    egui::ComboBox::from_id_salt("sweep-habitat")
        .selected_text(
            habitat_id
                .and_then(|id| world.habitats.iter().find(|h| h.id == id))
                .map(|h| h.definition.name.as_str())
                .unwrap_or("Select a live habitat"),
        )
        .show_ui(ui, |ui| {
            for h in &world.habitats {
                ui.selectable_value(
                    habitat_id,
                    Some(h.id),
                    format!("{} · #{}", h.definition.name, h.id),
                );
            }
        });
    egui::ComboBox::from_id_salt("habitat-sweep-parameter")
        .selected_text(study.sweep.parameter.label())
        .show_ui(ui, |ui| {
            for p in Parameter::ALL {
                ui.selectable_value(&mut study.sweep.parameter, p, p.label());
            }
        });
    ui.label(format!(
        "Distinct values, separated by commas ({})",
        study.sweep.parameter.unit()
    ));
    ui.text_edit_singleline(&mut study.sweep.values);
    ui.checkbox(&mut study.sweep.paired,"Pair with a second setting (four combinations)");
    if study.sweep.paired {
        egui::ComboBox::from_id_salt("habitat-second-setting").selected_text(study.sweep.secondary_parameter.label()).show_ui(ui,|ui| {
            for p in Parameter::ALL {ui.selectable_value(&mut study.sweep.secondary_parameter,p,p.label());}
        });
        ui.label(format!("Exactly two levels for each setting · second levels ({})",study.sweep.secondary_parameter.unit()));
        ui.text_edit_singleline(&mut study.sweep.secondary_values);
    }

    ui.horizontal(|ui| {
        ui.label("Observations per run");
        ui.add(egui::DragValue::new(&mut study.sweep.samples).range(1..=1024));
    });
    ui.horizontal(|ui| {
        ui.label("Interval (terrestrial hours)");
        ui.add(
            egui::DragValue::new(&mut study.sweep.interval_hours)
                .range(1. / 3600. ..=720.)
                .speed(0.01)
                .max_decimals(9),
        );
    });
    ui.label("New output folder (all runs and common state)");
    ui.text_edit_singleline(&mut study.sweep.output);
    let values = study
        .sweep
        .values
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f64>()
                .map_err(|_| "Enter finite comma-separated values".to_string())
        })
        .collect::<Result<Vec<_>, _>>();
    let plan = values.and_then(|values| {
        let p = Plan {
            format: "cosmogon-habitat-sweep-plan".into(),
            schema_version: if study.sweep.paired {2} else {1},
            parameter: study.sweep.parameter,
            values,
            samples: study.sweep.samples,
            interval_s: study.sweep.interval_hours * 3600.,
            secondary: if study.sweep.paired {
                let levels=parse_levels(&study.sweep.secondary_values)?;
                Some(cosmogon_sim::habitat_factorial::Secondary {parameter:study.sweep.secondary_parameter,values:levels.try_into().map_err(|_|"Second setting needs exactly two levels")?})
            } else {None},
        };
        p.validate()?;
        Ok(p)
    });
    let validation = plan.as_ref().map_err(Clone::clone).and_then(|_| {
        habitat_id
            .filter(|id| world.habitats.iter().any(|h| h.id == *id))
            .ok_or_else(|| "Select a live habitat".to_string())
            .and_then(|id| plan.as_ref().map_err(Clone::clone)?.validate_world(world, id))
    });
    if let Err(e) = &validation {
        ui.colored_label(ACCENT, e);
    }
    if reference {
        ui.colored_label(ACCENT, "Clone this reference world to a sandbox first.");
    }
    if ui
        .add_enabled(
            !study.pending()
                && !reference
                && validation.is_ok()
                && !study.sweep.output.trim().is_empty(),
            egui::Button::new("Run and open first/last trajectories"),
        )
        .clicked()
    {
        study.start_sweep(
            world.clone(),
            habitat_id.expect("validated habitat"),
            plan.expect("validated plan"),
            study.sweep.output.clone(),
            None,
        );
    }
    if let Some(manifest) = &study.sweep_manifest {
        ui.label(format!(
            "{} completed branches · {} ({})",
            manifest["runs"].as_array().map(Vec::len).unwrap_or(0),
            manifest["parameter"],
            manifest["unit"]
        ));
        if ui.button("Copy completed sweep manifest").clicked() {
            ui.ctx()
                .copy_text(serde_json::to_string_pretty(manifest).unwrap_or_default());
        }
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
    #[test]
    fn failed_sweep_commit_and_reference_launch_preserve_prior_views() {
        let u = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let h = cosmogon_sim::habitat::Habitat::new(
            0,
            cosmogon_sim::habitat::Definition::at(
                cosmogon_sim::BodyRef { system: 0, body: 3 },
                0.,
                0.,
            ),
            u.time,
        )
        .unwrap();
        let r = Recording {
            rows: vec![cosmogon_sim::habitat::Sample {
                simulation_time_s: u.time,
                habitat: h,
            }],
            summary: serde_json::json!({"name":"Prior"}),
        };
        let mut study = Study::default();
        study.commit(r.clone(), "prior".into(), None).unwrap();
        study.comparison = Some(r.clone());
        let before = study.metadata();
        let make = |sample, manifest| SweepResult {
            a: r.clone(),
            b: r.clone(),
            manifest,
            path: "new".into(),
            sample,
        };
        assert!(study
            .commit_sweep(make(Some(1), serde_json::json!({"runs":[{},{}]})))
            .is_err());
        assert_eq!(study.metadata(), before);
        assert_eq!(study.path, "prior");
        assert!(study
            .commit_sweep(make(None, serde_json::json!({"runs":[]})))
            .is_err());
        assert_eq!(study.metadata(), before);
        assert_eq!(study.path, "prior");
        study.launch = Some(Launch {
            plan: Plan::linear(Parameter::FirmPowerW, 0., 4000., 2, 1, 60.).unwrap(),
            id: Some(0),
            output: "unused".into(),
            sample: None,
            initial_path: None,
        });
        study.launch_if_ready(&u, None, true);
        assert!(!study.pending());
        assert_eq!(study.metadata(), before);
        assert!(u.habitats.is_empty());
        study
            .commit_sweep(make(None, serde_json::json!({"runs":[{},{}]})))
            .unwrap();
        assert_eq!(study.path, "new/run-000");
        assert_eq!(study.comparison_path, "new/run-001");
        assert!(study.metadata().unwrap()["comparison"]["selected_pair"]["b_minus_a"] == 0.);
    }
    #[test]
    fn moisture_worker_opens_actual_comparison_and_preserves_view_on_invalid_branch() {
        let mut world = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let mut definition = cosmogon_sim::habitat::Definition::at(cosmogon_sim::BodyRef {system:0,body:3},0.0,0.0);
        definition.config.moisture = Some(cosmogon_sim::moisture::Config::default());
        world.apply_edit(cosmogon_sim::sandbox::Edit::CreateHabitat {definition:Box::new(definition)}).unwrap();
        world.advance_to(world.time+86400.0,None,None);
        let before = cosmogon_sim::research::state_fingerprint(&world).unwrap();
        let root = std::env::temp_dir().join(format!("cosmogon-native-moisture-worker-{}",std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let plan = Plan::linear(Parameter::MoisturePowerW,0.0,500.0,2,3,3600.0).unwrap();
        let result = run_sweep(world.clone(),0,plan,root.join("good").to_string_lossy().into_owned(),Some(2)).unwrap();
        let mut study = Study {metric:Metric::Vapor,..Default::default()};
        study.commit_sweep(result).unwrap();
        let metadata = study.metadata().unwrap();
        assert_eq!(metadata["sweep"]["parameter"],"moisture_power_w");
        let pair = &metadata["comparison"]["selected_pair"];
        assert!(pair["b_minus_a"].as_f64().unwrap() < 0.0);
        assert_eq!(pair["a_value"].as_f64(),Some(study.recording.as_ref().unwrap().rows[2].habitat.state.water_vapor_kg));
        let plan = Plan::linear(Parameter::MoistureCondensateCapacityKg,20.0,0.0,2,3,60.0).unwrap();
        assert!(run_sweep(world.clone(),0,plan,root.join("invalid").to_string_lossy().into_owned(),None).is_err());
        assert!(!root.join("invalid").exists());
        assert_eq!(study.metadata(),Some(metadata));
        assert_eq!(cosmogon_sim::research::state_fingerprint(&world).unwrap(),before);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn paired_worker_reopens_verified_results_and_preserves_views_on_bad_input() {
        let mut world=Universe::new(cosmogon_sim::UniverseSettings {scenario:cosmogon_sim::Scenario::SolarSystemLab,..Default::default()});
        let mut definition=cosmogon_sim::habitat::Definition::at(cosmogon_sim::BodyRef {system:0,body:3},0.0,0.0);
        definition.config.moisture=Some(cosmogon_sim::moisture::Config::default());
        world.apply_edit(cosmogon_sim::sandbox::Edit::CreateHabitat {definition:Box::new(definition)}).unwrap();world.advance_to(world.time+86400.0,None,None);
        let root=std::env::temp_dir().join(format!("cosmogon-native-paired-worker-{}",std::process::id()));let _=std::fs::remove_dir_all(&root);
        let mut plan=Plan::linear(Parameter::MoisturePowerW,0.0,500.0,2,3,3600.0).unwrap();plan.schema_version=2;plan.secondary=Some(cosmogon_sim::habitat_factorial::Secondary {parameter:Parameter::MoistureAirborneFraction,values:[0.2,1.0]});
        let result=run_sweep(world,0,plan,root.to_string_lossy().into_owned(),Some(2)).unwrap();
        let mut study=Study {factorial_mode:true,metric:Metric::Vapor,..Default::default()};study.commit_sweep(result).unwrap();
        let before=study.metadata();assert!(before.as_ref().unwrap()["paired_results_visible"].as_bool().unwrap());assert_eq!(before.as_ref().unwrap()["sweep"]["runs"].as_array().unwrap().len(),4);
        let reopened=open_design(root.clone(),root.to_string_lossy().into_owned(),Some(2)).unwrap();study.commit_sweep(reopened).unwrap();assert_eq!(study.metadata(),before);
        assert!(check_input(&root,Some(3)).is_err());
        std::fs::write(root.join("factorial.json"),b"{}").unwrap();assert!(open_design(root.clone(),root.to_string_lossy().into_owned(),None).is_err());assert_eq!(study.metadata(),before);
        std::fs::remove_dir_all(root).unwrap();
    }

}
