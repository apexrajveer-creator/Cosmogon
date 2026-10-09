//! Request-time vector figure export; full numerical rendering lives in the shared library.
use super::{ACCENT, MUTED};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::habitat_figure::Figure;
use serde_json::{json, Value};
#[derive(Default)]
pub struct View {
    pub output: String,
    pub launch: bool,
    status: Option<String>,
    task: Option<Task<Result<Value, String>>>,
    request: Option<Value>,
    complete: Option<Value>,
}
impl View {
    pub fn pending(&self) -> bool {
        self.launch || self.task.is_some()
    }
    pub fn poll(&mut self) {
        if let Some(result) = self
            .task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.task = None;
            match result {
                Ok(v) => {
                    self.complete = Some(v);
                    self.status =
                        Some("Figure saved with exact numbers, inputs and sources.".into());
                }
                Err(e) => {
                    self.status = Some(format!(
                        "Figure export failed: {e}. Loaded evidence is preserved."
                    ));
                }
            }
        }
    }
    pub fn start(&mut self, figure: Result<Figure, String>) {
        self.launch = false;
        let figure = match figure {
            Ok(f) => f,
            Err(e) => {
                self.status = Some(format!("Could not prepare figure: {e}"));
                return;
            }
        };
        let output = self.output.trim().to_string();
        self.request = Some(
            json!({"output":output,"layout":figure.layout,"mode":figure.mode,"source_summaries":figure.source_summaries,"scope":"Actual immutable inputs captured when export was requested; current display choices may subsequently differ"}),
        );
        self.complete = None;
        self.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { figure.export(std::path::Path::new(&output)) }),
        );
        self.status = Some("Saving vector chart and original numeric/source evidence…".into());
    }
    pub fn metadata(&self) -> Value {
        json!({"pending":self.pending(),"request":self.request,"complete_manifest":self.complete,"status":self.status})
    }
    pub fn controls(&mut self, ui: &mut egui::Ui, busy: bool) -> bool {
        ui.strong("Research figure · exact stored values");
        ui.label(egui::RichText::new("SVG chart, original CSV values, exact source inputs and graph layout. Existing folders protected.").small().color(MUTED));
        if self.output.is_empty() {
            self.output = crate::persistence::data_dir()
                .join("recorded-figure")
                .display()
                .to_string();
        }
        ui.add(egui::TextEdit::singleline(&mut self.output).desired_width(ui.available_width()));
        let requested = ui
            .add_enabled(
                !busy && !self.pending() && !self.output.trim().is_empty(),
                egui::Button::new("Save figure to new folder"),
            )
            .clicked();
        if let Some(status) = &self.status {
            ui.colored_label(ACCENT, status);
        }
        ui.label(egui::RichText::new("Markers are stored samples; lines are visual guides. Missing/scale-omitted points break lines. No smoothing or invented uncertainty bands.").small().color(MUTED));
        requested
    }
}
