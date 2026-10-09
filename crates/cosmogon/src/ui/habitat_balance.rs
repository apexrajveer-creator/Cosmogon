//! Cached immutable retained-observation ledgers and protected worker exports.
use super::{ACCENT, MUTED};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::habitat_balance::Bundle;
use serde_json::{json, Value};
#[derive(Default)]
pub struct View {
    pub from: usize,
    pub to: Option<usize>,
    pub pane: usize,
    pub branch: usize,
    pub output: String,
    pub launch: bool,
    pub prepared: bool,
    bundle: Option<Bundle>,
    error: Option<String>,
    task: Option<Task<Result<Value, String>>>,
    status: Option<String>,
    request: Option<Value>,
    complete: Option<Value>,
}
impl View {
    pub fn reset(&mut self) {
        self.prepared = false;
        self.bundle = None;
        self.error = None;
    }
    pub fn set(&mut self, result: Result<Bundle, String>) {
        self.prepared = true;
        match result {
            Ok(b) => {
                self.bundle = Some(b);
                self.error = None;
            }
            Err(e) => {
                self.bundle = None;
                self.error = Some(e);
            }
        }
    }
    pub fn pending(&self) -> bool {
        self.launch || self.task.is_some()
    }
    pub fn start(&mut self, source: &str) {
        self.launch = false;
        let Some(b) = self.bundle.clone() else {
            self.status = Some(format!(
                "Could not prepare balance: {}",
                self.error
                    .as_deref()
                    .unwrap_or("Open recorded evidence first")
            ));
            return;
        };
        let output = self.output.trim().to_string();
        self.request = Some(
            json!({"loaded_source":source,"output":output,"mode":b.mode,"indices":b.reports.iter().map(|r|[r.from_index,r.to_index]).collect::<Vec<_>>(),"recordings":b.reports.iter().map(|r|&r.recording).collect::<Vec<_>>(),"scope":"Immutable actual inputs at export request; subsequent displayed choices can differ"}),
        );
        self.complete = None;
        self.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { b.export(std::path::Path::new(&output)) }),
        );
        self.status = Some("Saving exact endpoint stocks, counters and source context…".into());
    }
    pub fn poll(&mut self) {
        if let Some(r) = self
            .task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.task = None;
            match r {
                Ok(v) => {
                    self.complete = Some(v);
                    self.status = Some("Balance saved; original observations preserved.".into());
                }
                Err(e) => {
                    self.status = Some(format!(
                        "Balance export failed: {e}. Loaded evidence is preserved."
                    ));
                }
            }
        }
    }
    pub fn metadata(&self) -> Value {
        json!({"pane":self.pane,"branch":self.branch,"report":self.bundle,"error":self.error,"pending":self.pending(),"request":self.request,"complete_manifest":self.complete,"status":self.status})
    }
    // True means a new range was requested; the shared report is recomputed once.
    pub fn show(&mut self, ui: &mut egui::Ui, source: &str, count: usize, busy: bool) -> bool {
        ui.strong("Recorded cabin balances · actual retained endpoints");
        ui.add(
            egui::Label::new(
                egui::RichText::new(format!("Loaded source: {source}"))
                    .small()
                    .color(MUTED),
            )
            .truncate(),
        )
        .on_hover_text(source);
        let mut end = self.to.unwrap_or(count.saturating_sub(1));
        let old = (self.from, end);
        ui.horizontal(|ui| {
            ui.label("From observation");
            ui.add(egui::DragValue::new(&mut self.from).range(0..=count.saturating_sub(1)));
            ui.label("To");
            ui.add(egui::DragValue::new(&mut end).range(0..=count.saturating_sub(1)));
            if ui.button("First to last").clicked() {
                self.from = 0;
                end = count.saturating_sub(1);
            }
        });
        if old != (self.from, end) {
            self.to = Some(end);
            self.reset();
            return true;
        }
        if let Some(e) = &self.error {
            ui.colored_label(ACCENT, e);
            return false;
        }
        let Some(bundle) = &self.bundle else {
            ui.label("Open a recording or paired design in Curves & comparison.");
            return false;
        };
        if bundle.reports.len() == 4 {
            ui.horizontal(|ui| {
                ui.label("Independent branch");
                egui::ComboBox::from_id_salt("balance-branch")
                    .selected_text(format!("Run {:03}", self.branch))
                    .show_ui(ui, |ui| {
                        for i in 0..4 {
                            ui.selectable_value(&mut self.branch, i, format!("Run {i:03}"));
                        }
                    });
            });
        }
        let r = &bundle.reports[self.branch.min(bundle.reports.len() - 1)];
        ui.label(format!(
            "World span {:.3} s · completed state span {:.3} s",
            r.world_span_s, r.completed_span_s
        ));
        ui.horizontal(|ui| {
            for (i, label) in [
                "Stocks",
                "Electricity",
                "Temperature",
                "Counters",
                "Evidence & export",
            ]
            .iter()
            .enumerate()
            {
                ui.selectable_value(&mut self.pane, i, *label);
            }
        });
        ui.separator();
        match self.pane {
            0 => {
                ui.label(format!(
                    "Cabin/isolated mass Δ {} kg · outside Δ {} kg · accounted Δ {} kg",
                    r.mass["cabin_change_kg"],
                    r.mass["outside_change_kg"],
                    r.mass["accounted_change_kg"]
                ));
                ui.label("Escaped gas is outside the cabin and cannot be reused.");
                quantities(ui, &r.stocks);
                ui.label(format!(
                    "Component-sum residual {} kg",
                    r.mass["component_sum_residual_kg"]
                ));
            }
            1 => {
                ui.label(
                    "Generated + battery withdrawal = delivered + conversion losses + curtailed",
                );
                egui::Grid::new("balance-electric")
                    .striped(true)
                    .show(ui, |ui| {
                        for (key, label) in [
                            ("generated_j", "Generated"),
                            (
                                "battery_withdrawal_j",
                                "Battery withdrawal (negative = storage gain)",
                            ),
                            ("delivered_j", "Delivered"),
                            ("losses_j", "Conversion losses"),
                            ("curtailed_j", "Curtailed"),
                            (
                                "unserved_requested_j",
                                "Unserved requested; separate from output",
                            ),
                            ("residual_j", "Signed identity residual"),
                        ] {
                            ui.add_sized([210., 18.], egui::Label::new(label).truncate())
                                .on_hover_text(label);
                            ui.monospace(format!("{} J", r.electricity[key]));
                            ui.end_row();
                        }
                    });
                ui.label("Cumulative counter differences; sampled watts are never multiplied over this span.");
            }
            2 => {
                egui::Grid::new("balance-temperature")
                    .striped(true)
                    .show(ui, |ui| {
                        for (key, label, unit) in [
                            ("initial_temperature_k", "Initial cabin temperature", "K"),
                            ("final_temperature_k", "Final cabin temperature", "K"),
                            ("temperature_change_k", "Temperature change", "K"),
                            ("initial_capacity_j_k", "Initial effective capacity", "J/K"),
                            ("final_capacity_j_k", "Final effective capacity", "J/K"),
                            (
                                "sensible_change_j",
                                "C × ΔT for constant retained capacity",
                                "J",
                            ),
                            (
                                "phase_reference_change_j",
                                "Vapor reference phase-energy change",
                                "J",
                            ),
                        ] {
                            ui.add_sized([210., 18.], egui::Label::new(label).truncate())
                                .on_hover_text(label);
                            ui.monospace(if r.thermal[key].is_null() {
                                "Unavailable: retained capacity changed".into()
                            } else {
                                format!("{} {unit}", r.thermal[key])
                            });
                            ui.end_row();
                        }
                    });
                ui.label("Endpoint reference storage changes. No full-span thermal closure is available from sparse minute budgets.");
            }
            3 => {
                ui.label("Actual accumulated model counters. Served/unserved person-seconds describe nominal demand, not health, nutrition or survival.");
                quantities(ui, &r.cumulative_counters);
            }
            _ => {
                ui.label(format!(
                    "Retained configurations equal: {} · counter sequence monotone: {}",
                    r.retained_support["configurations_equal"],
                    r.retained_support["counter_sequence_monotone"]
                ));
                ui.label(format!("{} retained observations · {} uncompleted budgets · {} unavailable environment intervals · {} unsupported temperature intervals",r.retained_support["observations_checked"],r.retained_support["uncompleted_budget_count"],r.retained_support["sampled_environment_unavailable_count"],r.retained_support["sampled_temperature_unsupported_count"]));
                ui.label("These checks cover retained rows. Changes or unsupported behavior between them remain unobserved. Paired designs export all four independent ledgers.");
                if self.output.is_empty() {
                    self.output = crate::persistence::data_dir()
                        .join("recorded-balance")
                        .display()
                        .to_string();
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.output)
                        .desired_width(ui.available_width()),
                );
                if ui
                    .add_enabled(
                        !busy && !self.pending() && !self.output.trim().is_empty(),
                        egui::Button::new("Save all balances to new folder"),
                    )
                    .clicked()
                {
                    self.launch = true;
                }
                if let Some(s) = &self.status {
                    ui.colored_label(ACCENT, s);
                }
                ui.label(egui::RichText::new("report.json retains exact endpoint samples, original source summaries and paired method context; stocks.csv and counters.csv keep original units and numbers. Existing folders protected.").small().color(MUTED));
            }
        }
        ui.label(egui::RichText::new("Model bookkeeping; no measured-hardware calibration, reconstructed history or predictive uncertainty.").small().color(MUTED));
        false
    }
}
fn quantities(ui: &mut egui::Ui, rows: &[cosmogon_sim::habitat_balance::Quantity]) {
    egui::ScrollArea::vertical()
        .max_height(270.)
        .show(ui, |ui| {
            egui::Grid::new("balance-quantities")
                .striped(true)
                .show(ui, |ui| {
                    for t in ["Store / counter", "Before", "After", "Change"] {
                        ui.strong(t);
                    }
                    ui.end_row();
                    for q in rows {
                        ui.add_sized(
                            [170., 18.],
                            egui::Label::new(format!("{} ({})", q.label, q.unit)).truncate(),
                        )
                        .on_hover_text(format!("{} ({}) · {}", q.label, q.unit, q.scope));
                        for v in [q.initial, q.final_value, q.change] {
                            ui.monospace(format!("{v:.6e}"))
                                .on_hover_text(format!("{v:.17e}"));
                        }
                        ui.end_row();
                    }
                });
        });
}
