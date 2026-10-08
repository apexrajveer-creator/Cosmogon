//! Native completed-interval explanation. All quantities/formulas come from the engine.
use super::{ACCENT, DANGER, MUTED};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::{
    habitat::Habitat,
    habitat_diagnostics::{self as why, Diagnostics, Term},
};

#[derive(Default)]
pub struct View {
    pub section: usize,
    path: String,
    status: Option<String>,
    task: Option<Task<Result<String, String>>>,
}
impl View {
    pub fn poll(&mut self) {
        if let Some(result) = self
            .task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.task = None;
            self.status = Some(result.unwrap_or_else(|e| e));
        }
    }
    pub fn metadata(&self, h: &Habitat, world_time: f64, context: &str) -> serde_json::Value {
        serde_json::json!({"context":context,"section":(["power","water","heat","air_and_crew","evidence"][self.section.min(4)]),"report":why::report(h,world_time).ok()})
    }
    pub fn show(&mut self, ui: &mut egui::Ui, h: &Habitat, world_time: f64, paused: bool) {
        let d = why::explain(h);
        ui.label(
            egui::RichText::new("WHY · trace this habitat result")
                .strong()
                .color(ACCENT),
        );
        ui.label(format!(
            "Completed {:.3} s interval · end {:.3} s · world {:.3} s",
            d.interval_s, d.interval_end_s, world_time
        ));
        if paused {
            ui.label(
                egui::RichText::new("Paused: showing the held completed interval.")
                    .small()
                    .color(MUTED),
            );
        }
        ui.horizontal_wrapped(|ui| {
            for (i, label) in ["Power", "Water", "Heat", "Air & crew", "Evidence & export"]
                .iter()
                .enumerate()
            {
                ui.selectable_value(&mut self.section, i, *label);
            }
        });
        ui.separator();
        if d.support != "supported" {
            ui.colored_label(DANGER, d.explanations[0].text.as_str());
            ui.label("There are no supported flow terms to explain. The source snapshot and evidence can still be exported.");
        } else {
            if !d.configuration_matches_current {
                ui.colored_label(ACCENT,"Controls changed after this interval; the completed configuration is retained.");
            }
            match self.section {
                0 => {
                    explanation(ui, &d, "power_shortfall");
                    explanation(ui, &d, "power_served");
                    egui::Grid::new("why-allocations")
                        .num_columns(3)
                        .spacing([16.0, 5.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("Power priority");
                            ui.strong("Requested W");
                            ui.strong("Served W");
                            ui.end_row();
                            for a in &d.allocations {
                                ui.label(a.label).on_hover_text(a.evidence);
                                ui.monospace(number(a.requested_w));
                                ui.monospace(number(a.served_w));
                                ui.end_row();
                            }
                        });
                    ui.label(egui::RichText::new("Thermal requests are reconstructed; separate requests were not retained. Tiny values can be subtraction roundoff.").small().color(MUTED));
                    egui::CollapsingHeader::new("Supply, battery and numerical balance")
                        .default_open(false)
                        .show(ui, |ui| terms(ui, &d.power, "why-power"));
                    explanation(ui, &d, "zero_flux");
                }
                1 => {
                    explanation(ui, &d, "potable");
                    terms(ui, &d.water, "why-water");
                    ui.label(egui::RichText::new("Held raw product is processed after wastewater. New reactor products wait until a later interval; recovery is an assumed mass split, not water-quality certification.").small().color(MUTED));
                }
                2 => {
                    ui.label("Signed heat terms explain stored heat. Positive enters the cabin; negative leaves it. Heater watts are already in bus electricity.");
                    terms(ui, &d.heat, "why-heat");
                    ui.label(egui::RichText::new("Reference reaction enthalpy and lumped conduction are model terms. A small residual checks accounting, not measured temperature accuracy.").small().color(MUTED));
                }
                3 => {
                    ui.label("Actual interval mass transfers. Capture and the oxygen regulator move stored material; they create no new atoms. Newly produced oxygen feeds the regulator in a later interval.");
                    terms(ui, &d.air_and_crew, "why-air");
                    ui.label(egui::RichText::new("Crew exchanges are a nominal reference cycle. Pressure also depends on temperature and volume; these flows do not establish human health outcomes.").small().color(MUTED));
                }
                _ => {}
            }
            if self.section < 4 {
                for id in [
                    "crew_shortfall",
                    "electrolysis_constraints",
                    "sabatier_constraints",
                    "missing_reactor",
                ] {
                    explanation(ui, &d, id);
                }
            }
        }
        if self.section == 4 {
            ui.label("Export exact completed configuration, stores, terms, formulas and source references. Blank CSV values / JSON null mean unavailable, not zero.");
            if self.path.is_empty() {
                self.path = crate::persistence::data_dir()
                    .join("habitat-why")
                    .display()
                    .to_string();
            }
            ui.text_edit_singleline(&mut self.path);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        self.task.is_none(),
                        egui::Button::new("Save explanation to new folder"),
                    )
                    .clicked()
                {
                    let source = h.clone();
                    let path = std::path::PathBuf::from(&self.path);
                    self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
                        why::export(&source, world_time, &path)?;
                        Ok(format!(
                            "Saved habitat #{} interval ending {:.3} s to {}",
                            source.id,
                            source.last_time,
                            path.display()
                        ))
                    }));
                    self.status = Some("Saving the captured interval…".into());
                }
                if ui.button("Copy full source report").clicked() {
                    match why::report(h, world_time)
                        .and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
                    {
                        Ok(s) => ui.ctx().copy_text(s),
                        Err(e) => self.status = Some(e),
                    }
                }
            });
            if let Some(status) = &self.status {
                ui.label(status);
            }
            ui.hyperlink_to(
                "NASA BVAD Rev2 · crew reference",
                cosmogon_sim::habitat::SOURCE,
            );
            ui.label(egui::RichText::new(d.limits).small().color(MUTED));
            egui::CollapsingHeader::new("Recorded explanation evidence")
                .default_open(true)
                .show(ui, |ui| {
                    for e in &d.explanations {
                        ui.label(&e.text);
                        ui.label(
                            egui::RichText::new(e.evidence.join(" · "))
                                .small()
                                .color(MUTED),
                        );
                        ui.separator();
                    }
                });
            egui::CollapsingHeader::new("Completed configuration and residuals").show(ui,|ui|{
                ui.label(format!("Model {} · support {} · interval start {:?}",d.model,d.support,d.interval_start_s));
                ui.label(format!("Whole mass {:.3e} kg · electric {:.3e} J · thermal {:.3e} J",h.budget.mass_residual_kg,h.budget.electric_residual_j,h.budget.thermal_residual_j));
                ui.label("Thresholds are design assumptions, not medical limits. Input and predictive uncertainty remain unquantified. Full replay requires the saved world / ordinary habitat study.");
            });
        }
    }
}
fn number(v: Option<f64>) -> String {
    v.map(|v| {
        if v != 0.0 && (v.abs() < 1e-5 || v.abs() > 1e8) {
            format!("{v:.6e}")
        } else {
            format!("{v:.6}")
        }
    })
    .unwrap_or_else(|| "Unavailable".into())
}
fn terms(ui: &mut egui::Ui, terms: &[Term], id: &str) {
    egui::Grid::new(id)
        .num_columns(3)
        .spacing([12.0, 5.0])
        .striped(true)
        .show(ui, |ui| {
            for t in terms {
                ui.label(t.label).on_hover_text(t.evidence);
                ui.monospace(number(t.value)).on_hover_text(t.evidence);
                ui.label(t.unit);
                ui.end_row();
            }
        });
}
fn explanation(ui: &mut egui::Ui, d: &Diagnostics, id: &str) {
    if let Some(e) = d.explanations.iter().find(|e| e.id == id) {
        ui.label(&e.text).on_hover_text(e.evidence.join(" · "));
    }
}
