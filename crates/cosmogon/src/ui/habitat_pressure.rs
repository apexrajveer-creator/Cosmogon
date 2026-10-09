//! Read-only native pressure evidence. Numeric derivation/export is shared with CLI.
use super::{ACCENT, DANGER, MUTED};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::{
    habitat_pressure::{self as pressure, Change, Report, State},
    habitat_recording::Recording,
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reference {
    #[default]
    Previous,
    First,
    Chosen,
}
#[derive(Default)]
pub struct View {
    pub pane: usize,
    pub reference: Reference,
    pub index: usize,
    path: String,
    status: Option<String>,
    task: Option<Task<Result<String, String>>>,
}
impl View {
    pub fn pending(&self) -> bool {
        self.task.is_some()
    }
    pub fn poll(&mut self) {
        if let Some(result) = self
            .task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.task = None;
            self.status = Some(result.unwrap_or_else(|e| format!("Pressure export failed: {e}")));
        }
    }
    fn report(&self, r: &Recording, index: usize) -> Result<Report, String> {
        pressure::explain(
            r,
            index,
            match self.reference {
                Reference::Previous => None,
                Reference::First => Some(0),
                Reference::Chosen => Some(self.index),
            },
        )
    }
    pub fn metadata(&self, r: &Recording, index: usize, source: &str) -> serde_json::Value {
        match self.report(r, index) {
            Ok(report) => {
                serde_json::json!({"pane":self.pane,"loaded_source":source,"report":report,"export_pending":self.pending()})
            }
            Err(e) => {
                serde_json::json!({"pane":self.pane,"loaded_source":source,"error":e,"export_pending":self.pending()})
            }
        }
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        r: &Recording,
        selected: &mut usize,
        source: &str,
        paired: bool,
    ) {
        ui.label(
            egui::RichText::new(if paired {
                "Recorded model pressure · A, run-000 (one branch, not a paired effect)"
            } else {
                "Recorded model pressure · A, independent of the live world"
            })
            .strong()
            .color(ACCENT),
        );
        *selected = (*selected).min(r.rows.len() - 1);
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Slider::new(selected, 0..=r.rows.len() - 1).text("Sample"));
            ui.label("Reference");
            egui::ComboBox::from_id_salt("pressure-reference")
                .selected_text(match self.reference {
                    Reference::Previous => "Previous stored",
                    Reference::First => "First stored",
                    Reference::Chosen => "Chosen index",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.reference,
                        Reference::Previous,
                        "Previous stored",
                    );
                    ui.selectable_value(&mut self.reference, Reference::First, "First stored");
                    ui.selectable_value(&mut self.reference, Reference::Chosen, "Chosen index");
                });
            if self.reference == Reference::Chosen {
                ui.add(egui::DragValue::new(&mut self.index).range(0..=r.rows.len() - 1));
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (i, label) in [
                "Components",
                "Change accounting",
                "Breach substep",
                "Evidence & export",
            ]
            .iter()
            .enumerate()
            {
                ui.selectable_value(&mut self.pane, i, *label);
            }
        });
        ui.separator();
        let report = match self.report(r, *selected) {
            Ok(r) => r,
            Err(e) => {
                ui.colored_label(DANGER, e);
                ui.label("Loaded observations are preserved; choose an available reference index.");
                return;
            }
        };
        let row = &report.observation;
        ui.label(format!("Sample #{} · elapsed {:.3} h · {:.3} K · {:.3} m³",row.index,(row.sample.simulation_time_s-r.rows[0].simulation_time_s)/3600.0,row.state.temperature_k,row.state.volume_m3)).on_hover_text(format!("World {:.9} s since J2000; completed habitat {:.9} s. Exact input times are retained in the report/export.",row.sample.simulation_time_s,row.sample.habitat.last_time));
        ui.strong(format!(
            "Total pressure {:.6} kPa · {:.9} Pa",
            row.state.total_pressure_pa / 1000.0,
            row.state.total_pressure_pa
        ));
        match self.pane {
            0 => {
                ui.label("Ideal gas: pᵢ = nᵢRT/V. Vapor is gas; liquid stores and reserve tanks are excluded.");
                components(ui, &row.state, "current-pressure-components");
                ui.label(egui::RichText::new("Mole fraction counts gas molecules, not mass. Uniform ideal gas at one lumped temperature; modelled inputs, no measured atmosphere or quantified uncertainty.").small().color(MUTED));
            }
            1 => {
                if let (Some(a), Some(c)) = (&report.reference, &report.change) {
                    ui.label(format!(
                        "Stored #{} → #{} · Δt world {:.3} h / completed {:.3} h",
                        a.index,
                        row.index,
                        (row.sample.simulation_time_s - a.sample.simulation_time_s) / 3600.0,
                        (row.sample.habitat.last_time - a.sample.habitat.last_time) / 3600.0
                    ));
                    ui.label(format!(
                        "Reference {:.6} Pa · selected {:.6} Pa",
                        a.state.total_pressure_pa, row.state.total_pressure_pa
                    ));
                    accounting(ui, c, "stored-pressure-change");
                } else {
                    ui.colored_label(
                        ACCENT,
                        "No previous stored observation. Change accounting is unavailable.",
                    );
                    ui.label("Choose an explicit reference to compare actual states. The selected gas components remain available.");
                }
            }
            2 => {
                if let Some(b) = &report.breach_substep {
                    ui.label("Only the resolved breach substep, after the minute’s process/thermal update.").on_hover_text(b.scope);
                    ui.colored_label(
                        if b.full_interval_supported {
                            ACCENT
                        } else {
                            DANGER
                        },
                        b.budget.status.label(),
                    );
                    ui.label(format!(
                        "Resolved {:.9} / requested {:.3} s",
                        b.resolved_seconds, b.requested_seconds
                    ))
                    .on_hover_text(format!(
                        "Completed minute end {:.9} s; not a whole-observation interval",
                        b.completed_interval_end_s
                    ));
                    egui::CollapsingHeader::new("Retained substep inputs").show(ui, |ui| {
                        ui.label(format!(
                            "Retained {:.6} → {:.6} K · {:.3} m³",
                            b.initial.temperature_k,
                            b.final_state.temperature_k,
                            b.initial.volume_m3
                        ));
                        ui.label(format!(
                            "Substep pressure {:.6} → {:.6} Pa",
                            b.initial.total_pressure_pa, b.final_state.total_pressure_pa
                        ));
                    });
                    accounting(ui, &b.change, "breach-pressure-change");
                    ui.label(egui::RichText::new("Boundary or unsupported status does not establish full-interval flow. Only retained resolved transfers are accounted here.").small().color(MUTED));
                } else {
                    ui.colored_label(ACCENT, "No retained breach substep in this observation.");
                    ui.label("Unavailable evidence is not proof of zero historical leakage.");
                }
            }
            _ => {
                ui.label("Save exact selected/reference samples, gas components, signed terms, source identity and retained breach inputs. Completion manifest is written last in a new folder.");
                ui.label(format!("Loaded source: {source}"));
                ui.hyperlink_to("NASA Glenn · ideal-gas equation", pressure::SOURCE);
                ui.label(format!(
                    "Derivation {} · R = {:.14} J/(mol K)",
                    pressure::MODEL,
                    report.gas_constant_j_mol_k
                ));
                ui.label(format!(
                    "Component sum residual {:.3e} Pa · habitat formatter residual {:.3e} Pa",
                    row.state.component_sum_residual_pa, row.habitat_pressure_residual_pa
                ));
                if self.path.is_empty() {
                    self.path = crate::persistence::data_dir()
                        .join("pressure-evidence")
                        .display()
                        .to_string();
                }
                ui.add(egui::TextEdit::singleline(&mut self.path).desired_width(f32::INFINITY));
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(
                            !self.pending(),
                            egui::Button::new("Save evidence to new folder"),
                        )
                        .clicked()
                    {
                        let captured = report.clone();
                        let path = std::path::PathBuf::from(&self.path);
                        self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
                            pressure::export(&captured, &path)?;
                            Ok(format!(
                                "Saved captured observation #{} to {}",
                                captured.observation.index,
                                path.display()
                            ))
                        }));
                        self.status = Some("Saving the captured pressure evidence…".into());
                    }
                    if ui.button("Copy full pressure report").clicked() {
                        match serde_json::to_string_pretty(&report) {
                            Ok(s) => ui.ctx().copy_text(s),
                            Err(e) => self.status = Some(e.to_string()),
                        }
                    }
                });
                if let Some(status) = &self.status {
                    ui.colored_label(ACCENT, status);
                }
                ui.label(egui::RichText::new(pressure::LIMITS).small().color(MUTED));
            }
        }
    }
}

fn number(v: f64) -> String {
    if v != 0.0 && (v.abs() < 1e-5 || v.abs() > 1e8) {
        format!("{v:.9e}")
    } else {
        format!("{v:.9}")
    }
}
fn components(ui: &mut egui::Ui, state: &State, id: &str) {
    egui::Grid::new(id)
        .striped(true)
        .spacing([12.0, 5.0])
        .show(ui, |ui| {
            for label in ["Gas", "Mass kg", "Moles", "Mole fraction", "Pressure Pa"] {
                ui.strong(label);
            }
            ui.end_row();
            for c in &state.components {
                ui.label(c.label);
                ui.monospace(number(c.mass_kg));
                ui.monospace(number(c.moles));
                ui.monospace(
                    c.mole_fraction
                        .map(number)
                        .unwrap_or_else(|| "unavailable".into()),
                );
                ui.monospace(number(c.partial_pressure_pa));
                ui.end_row();
            }
        });
}
fn accounting(ui: &mut egui::Ui, change: &Change, id: &str) {
    ui.label("Order: inventory → temperature → volume. Positive raises pressure.");
    egui::Grid::new(id)
        .striped(true)
        .spacing([14.0, 3.0])
        .show(ui, |ui| {
            ui.strong("Term");
            ui.strong("Signed change Pa");
            ui.end_row();
            for t in &change.terms {
                ui.label(t.label).on_hover_text(t.formula);
                ui.monospace(number(t.pressure_change_pa));
                ui.end_row();
            }
        });
    ui.strong(format!(
        "Δp {} Pa · residual {:.3e} Pa",
        number(change.pressure_change_pa),
        change.residual_pa
    ))
    .on_hover_text(format!("Signed term sum {} Pa", number(change.term_sum_pa)));
    ui.label(egui::RichText::new("Mathematical accounting, not unique physical causation. Another order allocates interactions differently; missing minutes are not reconstructed.").small().color(MUTED));
    egui::CollapsingHeader::new("Read this result").show(ui, |ui| {
        ui.colored_label(ACCENT, &change.summary);
    });
}
