//! Session supply studies retain their own frozen inputs/context and never edit stocks.
use super::{ACCENT, DANGER, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{
    Universe, greenhouse,
    greenhouse_supplies::{self as model, Bundle, Definition, Support},
};

pub struct View {
    pub definition: Definition,
    pub result: Option<Bundle>,
    pub pane: usize,
    pub selected_day: usize,
    pub resource: usize,
    pub output: String,
    pub launch: bool,
    pub status: Option<String>,
    import_path: String,
    loaded_path: Option<String>,
    loaded: Option<Bundle>,
    context: Option<(Universe, Option<u64>)>,
}
impl Default for View {
    fn default() -> Self {
        Self {
            definition: Default::default(),
            result: None,
            pane: 0,
            selected_day: 30,
            resource: 0,
            output: String::new(),
            launch: false,
            status: None,
            import_path: String::new(),
            loaded_path: None,
            loaded: None,
            context: None,
        }
    }
}
impl View {
    pub fn import(&mut self, path: &std::path::Path) -> Result<(), String> {
        let b = Bundle::read(path)?;
        self.definition = b.report.definition.clone();
        self.context = b.context.clone().map(|u| (u, b.manifest.habitat_id));
        self.result = Some(b.clone());
        self.loaded = Some(b);
        self.loaded_path = Some(path.display().to_string());
        Ok(())
    }
    fn refresh(&mut self) {
        if self
            .result
            .as_ref()
            .is_none_or(|b| b.report.definition != self.definition)
        {
            match Bundle::calculate(
                &self.definition,
                self.context.as_ref().map(|(u, id)| (u, *id)),
            ) {
                Ok(b) => {
                    self.result = Some(b);
                    self.status = None;
                }
                Err(e) => {
                    self.result = None;
                    self.status = Some(e);
                }
            }
        }
    }
    fn save(&mut self) -> Result<(), String> {
        self.result
            .as_ref()
            .ok_or("Prepare valid supply inputs first")?
            .export(std::path::Path::new(self.output.trim()))
    }
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"prepared_definition":self.definition,"report":self.result.as_ref().map(|b|&b.report),"manifest":self.result.as_ref().map(|b|&b.manifest),"loaded_source":self.loaded_path,"loaded_definition":self.loaded.as_ref().map(|b|&b.report.definition),"prepared_inputs_match_loaded":self.loaded.as_ref().map(|b|b.report.definition==self.definition),"unopened_path":self.import_path,"selected_day":self.selected_day,"resource":model::RESOURCES[self.resource.min(3)],"pane":self.pane,"export":{"path":self.output,"pending":self.launch},"status":self.status,"scope":model::LIMITS})
    }
}
fn exact(ui: &mut egui::Ui, v: Option<f64>, unit: &str) {
    if let Some(v) = v {
        ui.label(format!("{v:.4} {unit}"))
            .on_hover_text(format!("{v:.17e} {unit}"));
    } else {
        ui.colored_label(DANGER, "Unavailable");
    }
}
fn chart(ui: &mut egui::Ui, b: &Bundle, day: &mut usize, k: usize) {
    let rows = &b.report.rows;
    let budget = b.report.definition.supplies.values()[k];
    if rows[30].demand[k].is_none() {
        ui.colored_label(
            DANGER,
            "Total liquid curve unavailable: transpiration is not enabled.",
        );
        return;
    }
    let max = budget.max(rows[30].demand[k].unwrap_or(0.0)).max(1e-12);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 90.0), egui::Sense::click());
    let plot = egui::Rect::from_min_max(
        rect.min + egui::vec2(65.0, 10.0),
        rect.max - egui::vec2(12.0, 23.0),
    );
    if plot.width() < 1.0 {
        return;
    }
    let p = ui.painter_at(rect);
    p.rect_filled(plot, 4.0, egui::Color32::from_rgb(11, 15, 24));
    let y = |v: f64| plot.bottom() - plot.height() * (v / max) as f32;
    for n in 0..=3 {
        let v = max * n as f64 / 3.0;
        p.text(
            egui::pos2(plot.left() - 4.0, y(v)),
            egui::Align2::RIGHT_CENTER,
            format!("{v:.2e}"),
            egui::FontId::proportional(10.0),
            MUTED,
        );
    }
    p.line_segment(
        [
            egui::pos2(plot.left(), y(budget)),
            egui::pos2(plot.right(), y(budget)),
        ],
        egui::Stroke::new(1.0_f32, MUTED),
    );
    let mut previous = None;
    for (i, r) in rows.iter().enumerate() {
        if let Some(v) = r.demand[k] {
            let pos = egui::pos2(plot.left() + plot.width() * i as f32 / 30.0, y(v));
            let color = if r.support[k] == Support::Supported {
                ACCENT
            } else {
                DANGER
            };
            if let Some(prev) = previous {
                p.line_segment([prev, pos], egui::Stroke::new(1.5_f32, color));
            }
            p.circle_filled(pos, 2.0, color);
            previous = Some(pos);
        } else {
            previous = None;
        }
    }
    for n in [0, 10, 20, 30] {
        p.text(
            egui::pos2(
                plot.left() + plot.width() * n as f32 / 30.0,
                plot.bottom() + 5.0,
            ),
            egui::Align2::CENTER_TOP,
            n.to_string(),
            egui::FontId::proportional(10.0),
            MUTED,
        );
    }
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            *day = (((pos.x - plot.left()) / plot.width()).clamp(0.0, 1.0) * 30.0).round() as usize;
        }
    }
    let x = plot.left() + plot.width() * (*day).min(30) as f32 / 30.0;
    p.line_segment(
        [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
        egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
    );
}
pub fn panel(
    ui: &mut egui::Ui,
    v: &mut View,
    crop: &greenhouse::Definition,
    world: &Universe,
    habitat: Option<u64>,
) {
    ui.strong("Finite crop supplies · maintained-input feasibility");
    ui.label(
        egui::RichText::new("Daily forecast support · no installed crop or habitat stock transfer")
            .small()
            .color(MUTED),
    );
    ui.horizontal_wrapped(|ui| {
        for (i, label) in [
            "Needs & coverage",
            "Supplies",
            "Recipe & limits",
            "Study files",
        ]
        .iter()
        .enumerate()
        {
            ui.selectable_value(&mut v.pane, i, *label);
        }
    });
    ui.separator();
    if v.pane == 1 {
        ui.label("Explicit user budgets for one crop cycle; no future supply or recovery credit.");
        super::habitat::number(
            ui,
            "Liquid water (kg)",
            &mut v.definition.supplies.liquid_water_kg,
            0.0,
            1e9,
        );
        super::habitat::number(
            ui,
            "Carbon dioxide (kg)",
            &mut v.definition.supplies.carbon_dioxide_kg,
            0.0,
            1e9,
        );
        super::habitat::number(
            ui,
            "Nutrients, HNO₃-equivalent (kg)",
            &mut v.definition.supplies.nutrient_hno3_equivalent_kg,
            0.0,
            1e9,
        );
        super::habitat::number(
            ui,
            "Lighting energy (J)",
            &mut v.definition.supplies.lighting_energy_j,
            0.0,
            1e16,
        );
        ui.label(format!(
            "Study crop: {:.1} m² · {}",
            v.definition.crop.area_m2,
            v.definition.crop.regime.label()
        ));
        if ui.button("Use current crop planner inputs").clicked() {
            v.definition.crop = crop.clone();
        }
        ui.label("Crop Inputs prepares area, regime, fixture and optional humidity. Copying is explicit; opened study assumptions stay separate.");
    }
    v.refresh();
    if v.launch {
        v.launch = false;
        v.status = Some(match v.save() {
            Ok(()) => "Supply study saved with exact numbers and frozen context".into(),
            Err(e) => e,
        });
    }
    if let Some(b) = &v.result {
        let r = &b.report;
        let day = v.selected_day.min(30);
        let row = &r.rows[day];
        match v.pane {
            0 => {
                ui.label(format!(
                    "{:.1} m² · {}",
                    r.definition.crop.area_m2,
                    r.definition.crop.regime.label()
                ));
                let support = match r.supported_prefix_last_day {
                    Some(n) => format!("All four recorded demands within budget through day {n}"),
                    None => "Complete support unavailable: missing total liquid demand".into(),
                };
                ui.colored_label(
                    if r.supported_prefix_last_day == Some(30) {
                        ACCENT
                    } else {
                        DANGER
                    },
                    support,
                );
                if let Some(n) = r.first_known_unsupported_day {
                    ui.label(format!("First known exceeded daily observation: day {n}; crossing bracket [{}, {n}] d, no exact deadline",n.saturating_sub(1)));
                }
                egui::ComboBox::from_id_salt("crop-supply-resource")
                    .selected_text(format!(
                        "{} [{}]",
                        model::LABELS[v.resource],
                        model::UNITS[v.resource]
                    ))
                    .show_ui(ui, |ui| {
                        for k in 0..4 {
                            ui.selectable_value(&mut v.resource, k, model::LABELS[k]);
                        }
                    });
                chart(ui, b, &mut v.selected_day, v.resource);
                ui.label(
                    egui::RichText::new(
                        "Daily demand points · gray budget · red beyond budget; lines are guides",
                    )
                    .small()
                    .color(MUTED),
                );
                ui.add(egui::Slider::new(&mut v.selected_day, 0..=30).text("Days after emergence"));
                // Slider changes take effect in this frame; never label a different row.
                let row = &r.rows[v.selected_day.min(30)];
                egui::Grid::new("crop-supply-values")
                    .striped(true)
                    .show(ui, |ui| {
                        for s in ["Resource", "Required", "Budget margin", "Support"] {
                            ui.strong(s);
                        }
                        ui.end_row();
                        for k in 0..4 {
                            ui.label(model::LABELS[k]);
                            exact(ui, row.demand[k], model::UNITS[k]);
                            exact(ui, row.margin[k], model::UNITS[k]);
                            ui.label(row.support[k].key()).on_hover_text(match r.constraints[k].exhaustion_bracket_days {Some([a,b])=>format!("First exceeded daily observation: {b} d; bracket [{a}, {b}] d, no exact deadline"),None=>if row.demand[k].is_some(){"Known demand within budget through day 30".into()}else{"Total demand unavailable; no complete support claim".into()}});
                            ui.end_row();
                        }
                    });
                ui.label(format!(
                    "Potential edible dry mass: {:.4} kg · recipe O₂: {:.4} kg",
                    row.edible_dry_biomass_kg, row.recipe_oxygen_kg
                ));
                ui.label("Potential output under maintained inputs only; budget excess does not predict stressed yield or credit inventories.");
            }
            2 => {
                ui.hyperlink_to(
                    "Volk & Cullingford 1992 · NASA Table 2, p528",
                    model::SOURCE,
                );
                ui.label("kg per kg dry production · edible / inedible");
                egui::Grid::new("crop-recipe-coefficients")
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, e, i) in [
                            ("CO₂", 1.82, 1.68),
                            ("Metabolic water", 0.57, 0.55),
                            ("HNO₃-equivalent", 0.20, 0.08),
                            ("O₂", 1.59, 1.32),
                        ] {
                            ui.label(label);
                            ui.label(format!("{e:.2} / {i:.2}"));
                            ui.end_row();
                        }
                    });
                ui.label(format!(
                    "Day {day}: {:.5} kg metabolic incorporation + {} kg light-period vapor",
                    row.metabolic_water_kg,
                    row.transpired_vapor_kg
                        .map(|x| format!("{x:.5}"))
                        .unwrap_or("unavailable".into())
                ));
                ui.label(format!(
                    "Rounded recipe mass residual: {:.17e} kg",
                    row.recipe_mass_residual_kg
                ));
                ui.label("Inputs minus dry production and recipe O₂. Inedible rounding residual is −0.01 kg/kg; no physical coupling or hidden correction.");
                ui.label("Original MEC carbon/O₂ quantities are retained separately in the full forecast. This combined scenario is not empirically validated.");
                egui::CollapsingHeader::new("Full assumptions and unavailable quantities").show(
                    ui,
                    |ui| {
                        ui.label(model::LIMITS);
                        ui.hyperlink_to("Original NASA MEC source", greenhouse::SOURCE);
                    },
                );
            }
            3 => {
                ui.label("Protected definition, complete report/forecast, exact CSV and optional frozen world context.");
                if let Some(path) = &v.loaded_path {
                    ui.label(
                        egui::RichText::new(format!("Opened source: {path}"))
                            .small()
                            .color(MUTED),
                    );
                }
                ui.text_edit_singleline(&mut v.output);
                if ui.button("Save supply study to new folder").clicked() {
                    v.launch = true;
                }
                ui.separator();
                ui.label("Open definition file or verified study folder");
                ui.text_edit_singleline(&mut v.import_path);
                if ui.button("Open supply study").clicked() {
                    let p = std::path::PathBuf::from(v.import_path.trim());
                    v.status = Some(match v.import(&p) {
                        Ok(()) => "Opened exact study inputs and frozen context".into(),
                        Err(e) => e,
                    });
                }
                if v.loaded.is_some() && ui.button("Restore opened study").clicked() {
                    if let Some(b) = v.loaded.clone() {
                        v.definition = b.report.definition.clone();
                        v.context = b.context.clone().map(|u| (u, b.manifest.habitat_id));
                        v.result = Some(b);
                    }
                }
                ui.separator();
                ui.label(v.context.as_ref().map(|(u,id)|format!("Frozen context at {:.3} s · habitat {:?}; independent of live world",u.time,id)).unwrap_or("No world context; budgets are explicit assumptions".into()));
                if ui
                    .button("Freeze current world as observation context")
                    .clicked()
                {
                    v.context = Some((world.clone(), habitat));
                    v.result = None;
                }
                if v.context.is_some() && ui.button("Remove frozen context").clicked() {
                    v.context = None;
                    v.result = None;
                }
                ui.label("Context does not supply crops or allocate stores. Unopened path text never changes the loaded study.");
            }
            _ => {}
        }
    }
    if let Some(status) = &v.status {
        ui.colored_label(ACCENT, status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supply_study_retains_opened_inputs_frozen_context_and_independent_path() {
        let path = std::env::temp_dir().join(format!(
            "cosmogon-native-crop-supplies-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        let mut u = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let d = Definition::default();
        Bundle::calculate(&d, Some((&u, None)))
            .unwrap()
            .export(&path)
            .unwrap();
        let mut v = View::default();
        v.import(&path).unwrap();
        let before = v.metadata();
        u.advance_by(60.0);
        v.import_path = "unopened-other-study".into();
        v.refresh();
        assert_eq!(v.metadata()["report"], before["report"]);
        assert_eq!(v.metadata()["manifest"], before["manifest"]);
        assert_eq!(v.metadata()["loaded_source"], before["loaded_source"]);
        assert_eq!(v.metadata()["unopened_path"], "unopened-other-study");
        assert!(v.import(&path.join("missing")).is_err());
        assert_eq!(v.metadata()["report"], before["report"]);
        v.definition.supplies.lighting_energy_j = 0.0;
        v.refresh();
        assert_eq!(v.metadata()["prepared_inputs_match_loaded"], false);
        assert_eq!(
            v.metadata()["loaded_definition"],
            serde_json::to_value(&d).unwrap()
        );
        assert_eq!(
            v.result
                .as_ref()
                .unwrap()
                .report
                .first_known_unsupported_day,
            Some(1)
        );
        assert_eq!(
            v.result.as_ref().unwrap().manifest,
            serde_json::from_value::<model::Manifest>(before["manifest"].clone()).unwrap()
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}
