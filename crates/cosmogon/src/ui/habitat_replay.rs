//! Explicit worker-driven saved-design verification; the live world remains independent.
use super::{ACCENT, MUTED};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::habitat_replay::{self, Progress};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
#[derive(Default)]
pub struct View {
    pub launch: bool,
    task: Option<Task<Result<Value, String>>>,
    export_task: Option<Task<Result<(), String>>>,
    cancel: Arc<AtomicBool>,
    progress: Arc<Mutex<Option<Progress>>>,
    pub report: Option<Value>,
    source: String,
    report_source: String,
    output: String,
    status: Option<String>,
}
impl View {
    pub fn pending(&self) -> bool {
        self.task.is_some() || self.export_task.is_some()
    }
    pub fn reset(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.task = None;
        self.report = None;
        self.progress = Arc::new(Mutex::new(None));
        self.status = None;
        self.source.clear();
        self.report_source.clear();
    }
    pub fn start(&mut self, source: String) {
        if self.pending() {
            return;
        }
        self.launch = false;
        self.source = source.clone();
        self.cancel = Arc::new(AtomicBool::new(false));
        self.progress = Arc::new(Mutex::new(None));
        let cancel = self.cancel.clone();
        let progress = self.progress.clone();
        self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
            habitat_replay::verify_with(std::path::Path::new(&source), |p| {
                if cancel.load(Ordering::Relaxed) {
                    return Err("Replay canceled; recorded observations are preserved".into());
                }
                *progress
                    .lock()
                    .map_err(|_| "Replay progress is unavailable")? = Some(p);
                Ok(())
            })
        }));
        self.status = Some(
            "Reopening pinned sources and independently recomputing all retained branches…".into(),
        );
    }
    pub fn poll(&mut self) {
        if let Some(result) = self
            .task
            .as_mut()
            .and_then(|task| block_on(future::poll_once(task)))
        {
            self.task = None;
            match result {Ok(report)=>{self.status=Some(if report["verified"]==true {"Replay matches every recorded observation and endpoint.".into()}else{"Replay differs from retained observations. Inspect exact mismatch details below.".into()});self.report_source=self.source.clone();self.report=Some(report);},Err(e)=>self.status=Some(format!("Replay did not complete: {e}. Previous report and recorded views are preserved."))}
        }
        if let Some(result) = self
            .export_task
            .as_mut()
            .and_then(|task| block_on(future::poll_once(task)))
        {
            self.export_task = None;
            self.status = Some(match result {
                Ok(()) => "Protected verification report exported.".into(),
                Err(e) => format!("Verification export failed: {e}. The report is preserved."),
            });
        }
    }
    pub fn metadata(&self) -> Value {
        json!({"pending":self.task.is_some(),"export_pending":self.export_task.is_some(),"cancel_requested":self.cancel.load(Ordering::Relaxed),"request_source":self.source,"report_source":self.report_source,"progress":self.progress.lock().ok().and_then(|p|*p),"report":self.report,"status":self.status,"scope":"Request-time saved-design verification; no live-world edits or hardware-validation claim"})
    }
    pub fn show(&mut self, ui: &mut egui::Ui, source: &str, other_pending: bool) {
        ui.strong("Recompute saved trajectories");
        ui.label("Check every complete observation and initial/final world identity against a fresh run from the retained common state. Recorded results stay unchanged.");
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !other_pending && !self.pending(),
                    egui::Button::new("Verify all retained branches"),
                )
                .clicked()
            {
                self.start(source.into());
            }
            if self.task.is_some() && ui.button("Cancel verification").clicked() {
                self.cancel.store(true, Ordering::Relaxed);
                self.status = Some("Cancel requested; waiting for a scheduler boundary…".into());
            }
        });
        if let Some(p) = self.progress.lock().ok().and_then(|p| *p) {
            ui.label(format!(
                "{} / {} observations checked · branch {} · observation {}",
                p.checked_rows, p.total_rows, p.branch_index, p.observation_index
            ));
            if self.task.is_some() {
                ui.add(egui::ProgressBar::new(
                    p.checked_rows as f32 / p.total_rows.max(1) as f32,
                ));
            }
        }
        if let Some(status) = &self.status {
            ui.colored_label(ACCENT, status);
        }
        if let Some(report) = &self.report {
            ui.strong(format!(
                "{} / {} observations match · {}",
                report["matched_rows"],
                report["total_rows"],
                if report["verified"] == true {
                    "all endpoints/source versions match"
                } else {
                    "differences retained"
                }
            ));
            ui.label(
                egui::RichText::new(format!("Completed report: {}", self.report_source))
                    .small()
                    .color(MUTED),
            );
            egui::Grid::new("replay-branches")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Branch");
                    ui.strong("Rows match");
                    ui.strong("Initial / final world");
                    ui.strong("Sources");
                    ui.end_row();
                    if let Some(branches) = report["branches"].as_array() {
                        for b in branches {
                            ui.label(b["index"].to_string());
                            ui.label(format!(
                                "{} / {}",
                                b["matched_observations"], b["checked_observations"]
                            ));
                            ui.label(format!(
                                "{} / {}",
                                if b["initial_fingerprint_match"] == true {
                                    "match"
                                } else {
                                    "different"
                                },
                                if b["final_fingerprint_match"] == true {
                                    "match"
                                } else {
                                    "different"
                                }
                            ));
                            ui.label(if b["source_versions_match"] == true {
                                "match"
                            } else {
                                "different"
                            });
                            ui.end_row();
                        }
                    }
                });
            if let Some(first) = report["mismatches"].as_array().and_then(|m| m.first()) {
                ui.colored_label(
                    ACCENT,
                    format!(
                        "First differing observation: branch {} · index {} · world time {} s",
                        first["branch_index"],
                        first["observation_index"],
                        first["simulation_time_s"]
                    ),
                );
            }
            if ui.button("Copy complete verification JSON").clicked() {
                ui.ctx()
                    .copy_text(serde_json::to_string_pretty(report).unwrap_or_default());
            }
            ui.label("New verification report folder");
            ui.text_edit_singleline(&mut self.output);
            if ui
                .add_enabled(
                    !self.pending() && !self.output.trim().is_empty(),
                    egui::Button::new("Export protected verification report"),
                )
                .clicked()
            {
                let report = report.clone();
                let output = std::path::PathBuf::from(self.output.trim());
                self.export_task = Some(
                    AsyncComputeTaskPool::get()
                        .spawn(async move { habitat_replay::export(&report, &output) }),
                );
                self.status = Some("Exporting the completed report…".into());
            }
            egui::CollapsingHeader::new("Exact mismatch details and method").show(ui,|ui| {ui.code(serde_json::to_string_pretty(&json!({"mismatches":report["mismatches"],"method":report["method"],"limits":report["limits"]})).unwrap_or_default());});
        }
        ui.label(egui::RichText::new("Reproducibility with this engine/source version. This does not test against real equipment, quantify predictive uncertainty or establish cryptographic authenticity.").small().color(MUTED));
        ui.hyperlink_to(
            "NASA model verification / validation distinction",
            habitat_replay::METHODOLOGY_SOURCE,
        );
    }
}

impl Drop for View {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
