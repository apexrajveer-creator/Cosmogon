//! Separate surrogate composition; no biological inventory or hidden source correction.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{crop_chemistry as model, greenhouse_supplies::Bundle};
#[derive(Default)]
pub struct View {
    pub report: Option<model::Report>,
    pub pane: usize,
    pub output: String,
    pub launch: bool,
    pub status: Option<String>,
    pub import_path: String,
}
impl View {
    pub fn refresh(&mut self, b: &Bundle) {
        if self.report.is_none() {
            match model::calculate(b) {
                Ok(r) => self.report = Some(r),
                Err(e) => self.status = Some(e),
            }
        }
    }
    pub fn invalidate(&mut self) {
        self.report = None;
        self.status = None;
    }
    pub fn save(&mut self, b: &Bundle) -> Result<(), String> {
        model::export(b, std::path::Path::new(self.output.trim()))
    }
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"report":self.report,"pane":self.pane,"unopened_path":self.import_path,"export":{"path":self.output,"pending":self.launch},"status":self.status})
    }
}
fn exact(ui: &mut egui::Ui, value: f64, unit: &str) {
    ui.label(format!("{value:.5} {unit}"))
        .on_hover_text(format!("{value:.17e} {unit}"));
}
// Returns an explicit request to open a verified chemistry bundle; source replacement stays atomic.
pub fn panel(
    ui: &mut egui::Ui,
    v: &mut View,
    b: &Bundle,
    day: &mut usize,
    loaded_path: Option<&str>,
) -> Option<std::path::PathBuf> {
    v.refresh(b);
    if v.launch {
        v.launch = false;
        v.status = Some(match v.save(b) {
            Ok(()) => "Conservative chemistry saved with exact original study and context".into(),
            Err(e) => e,
        });
    }
    ui.horizontal_wrapped(|ui| {
        for (i, s) in ["Element balance", "Assumptions", "Chemistry files"]
            .iter()
            .enumerate()
        {
            ui.selectable_value(&mut v.pane, i, *s);
        }
    });
    let mut open = None;
    if let Some(r) = &v.report {
        match v.pane {
            0 => {
                ui.label(
                    egui::RichText::new(
                        "CHON-only engineering surrogate · potential growth, no inventory transfer",
                    )
                    .small()
                    .color(MUTED),
                );
                ui.add(egui::Slider::new(day, 0..=30).text("Days after emergence"));
                let row = &r.rows[(*day).min(30)];
                ui.label(format!(
                    "Dry matter: {:.5} kg edible + {:.5} kg inedible · source inputs {}",
                    row.edible_dry_biomass_kg,
                    row.inedible_dry_biomass_kg,
                    row.source_supply_support.key()
                ));
                egui::Grid::new("crop-chemistry-products")
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, value) in [
                            ("Published recipe O₂", row.source_recipe_oxygen_kg),
                            ("Conservative surrogate O₂", row.conservative_oxygen_kg),
                            ("Surrogate minus published", row.oxygen_difference_kg),
                        ] {
                            ui.label(label);
                            exact(ui, value, "kg");
                            ui.end_row();
                        }
                    });
                egui::Grid::new("crop-element-balance")
                    .striped(true)
                    .show(ui, |ui| {
                        for s in ["Element", "Input kg", "Output kg", "Residual kg"] {
                            ui.strong(s);
                        }
                        ui.end_row();
                        for k in 0..4 {
                            ui.label(["C", "H", "O", "N"][k]);
                            for x in [
                                row.input_elements_kg[k],
                                row.output_elements_kg[k],
                                row.element_residual_kg[k],
                            ] {
                                ui.label(format!("{x:.5e}"))
                                    .on_hover_text(format!("{x:.17e} kg"));
                            }
                            ui.end_row();
                        }
                    });
                ui.label(format!("Mass residual: source {:.5e} kg · surrogate {:.5e} kg",row.source_recipe_mass_residual_kg,row.mass_residual_kg)).on_hover_text(format!("Source {:.17e} kg; surrogate {:.17e} kg. Inputs minus dry matter and O₂; floating roundoff is retained.",row.source_recipe_mass_residual_kg,row.mass_residual_kg));
                ui.label(format!(
                    "Separate transpired vapor: {}",
                    row.transpired_vapor_kg
                        .map(|x| format!("{x:.5} kg"))
                        .unwrap_or("unavailable".into())
                ));
                ui.label(egui::RichText::new("Inferred dry composition; no ash/trace nutrients, ionic balance or crop physiology. Budget excess does not predict stressed yield.").small().color(MUTED));
            }
            1 => {
                ui.hyperlink_to(
                    "NASA 1992 Table 2: original rounded crop inputs",
                    cosmogon_sim::greenhouse_supplies::SOURCE,
                );
                ui.hyperlink_to(
                    "CIAAW 2024: separately pinned nominal atomic weights",
                    model::SOURCE,
                );
                ui.label("Per 1 kg dry matter: CO₂ + metabolic H₂O + HNO₃-equivalent → inferred CHON + O₂. All input C/H/N enters dry matter; remaining oxygen becomes O₂.");
                egui::Grid::new("crop-chemistry-pools")
                    .striped(true)
                    .show(ui, |ui| {
                        for s in ["kg / kg dry", "Edible", "Inedible"] {
                            ui.strong(s);
                        }
                        ui.end_row();
                        for (label, a, z) in [
                            (
                                "Published O₂",
                                r.pools[0].published_oxygen_kg_per_kg,
                                r.pools[1].published_oxygen_kg_per_kg,
                            ),
                            (
                                "Derived O₂",
                                r.pools[0].conservative_oxygen_kg_per_kg,
                                r.pools[1].conservative_oxygen_kg_per_kg,
                            ),
                        ] {
                            ui.label(label);
                            exact(ui, a, "");
                            exact(ui, z, "");
                            ui.end_row();
                        }
                        for k in 0..4 {
                            ui.label(format!("Inferred dry {}", ["C", "H", "O", "N"][k]));
                            exact(ui, r.pools[0].inferred_dry_elements_kg_per_kg[k], "");
                            exact(ui, r.pools[1].inferred_dry_elements_kg_per_kg[k], "");
                            ui.end_row();
                        }
                    });
                ui.label("Inedible 1.31 versus published 1.32 is an engineered closure assumption, not a corrected measurement. Original source values remain retained.");
                egui::CollapsingHeader::new("Atomic basis, inferred formula and full limits").show(
                    ui,
                    |ui| {
                        for k in 0..4 {
                            ui.label(format!(
                                "{}: {:.4} g/mol",
                                model::ELEMENTS[k],
                                r.atomic_g_mol[k]
                            ));
                        }
                        for p in &r.pools {
                            ui.label(format!(
                                "{} formula atoms per carbon C/H/O/N: {:?}",
                                p.name, p.inferred_formula_atoms_per_carbon
                            ));
                        }
                        ui.label(model::LIMITS);
                    },
                );
            }
            2 => {
                ui.label("Protected chemistry JSON, exact elemental CSV and complete original supply bundle, including frozen context.");
                if let Some(path) = loaded_path {
                    ui.label(
                        egui::RichText::new(format!("Opened source: {path}"))
                            .small()
                            .color(MUTED),
                    );
                }
                ui.text_edit_singleline(&mut v.output);
                if ui.button("Save chemistry to new folder").clicked() {
                    v.launch = true;
                }
                ui.separator();
                ui.label("Open verified chemistry folder");
                ui.text_edit_singleline(&mut v.import_path);
                if ui.button("Open chemistry study").clicked() {
                    open = Some(std::path::PathBuf::from(v.import_path.trim()));
                }
                ui.label(b.context.as_ref().map(|u|format!("Original frozen context {:.3} s · habitat {:?}; independent of live world",u.time,b.manifest.habitat_id)).unwrap_or("No world context; explicit planning inputs".into()));
                ui.label("Prepared supply edits recalculate this separate chemistry report; the opened source is retained. Study files restores the opened inputs.");
            }
            _ => {}
        }
    }
    if let Some(s) = &v.status {
        ui.colored_label(ACCENT, s);
    }
    open
}
