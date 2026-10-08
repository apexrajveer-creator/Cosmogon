//! Native planning instrument over the shared source model. No hidden resource credits.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::greenhouse::{Definition, Forecast, Regime};

pub struct CropPlanner {
    pub definition: Definition,
    pub result: Option<Forecast>,
    selected_day: usize,
    graph: usize,
    import_path: String,
    export_path: String,
    pub status: Option<String>,
}
impl Default for CropPlanner {
    fn default() -> Self {
        Self {
            definition: Definition::default(),
            result: None,
            selected_day: 30,
            graph: 0,
            import_path: String::new(),
            export_path: String::new(),
            status: None,
        }
    }
}
impl CropPlanner {
    pub fn import(&mut self, path: &std::path::Path) -> Result<(), String> {
        // Calculate first so failed imports preserve both prepared inputs and old results.
        let d = Definition::read(path)?;
        let result = d.calculate()?;
        self.definition = d;
        self.result = Some(result);
        self.selected_day = self.selected_day.min(30);
        Ok(())
    }
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"prepared_definition":self.definition,"forecast":self.result,"selected_age_days_after_emergence":self.selected_day,"prepared_inputs_match_forecast":self.result.as_ref().is_some_and(|f|f.definition==self.definition),"semantics":"Session planning view; may be hidden. Forecast does not install a greenhouse or allocate habitat resources."})
    }
}
fn chart(ui: &mut egui::Ui, f: &Forecast, selected: &mut usize, metric: usize) {
    let value = |i: usize| {
        let r = &f.rows[i];
        match metric {
            1 => r.net_oxygen_kg,
            2 => r.lighting_electric_j / 3.6e6,
            _ => r.edible_dry_biomass_kg,
        }
    };
    let ymax = value(30).max(1e-12);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 100.0),
        egui::Sense::click(),
    );
    let plot = egui::Rect::from_min_max(
        rect.min + egui::vec2(43.0, 10.0),
        rect.max - egui::vec2(12.0, 23.0),
    );
    if plot.width() < 1.0 {
        return;
    }
    let painter = ui.painter_at(rect);
    painter.rect_filled(plot, 4.0, egui::Color32::from_rgb(11, 15, 24));
    for n in 0..=3 {
        let y = plot.bottom() - plot.height() * n as f32 / 3.0;
        painter.line_segment(
            [egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(40)),
        );
        painter.text(
            egui::pos2(plot.left() - 4.0, y),
            egui::Align2::RIGHT_CENTER,
            format!("{:.2}", ymax * n as f64 / 3.0),
            egui::FontId::proportional(10.0),
            MUTED,
        );
    }
    for day in [0, 10, 20, 30] {
        let x = plot.left() + plot.width() * day as f32 / 30.0;
        painter.text(
            egui::pos2(x, plot.bottom() + 6.0),
            egui::Align2::CENTER_TOP,
            day.to_string(),
            egui::FontId::proportional(11.0),
            MUTED,
        );
    }
    let points = (0..=30)
        .map(|i| {
            egui::pos2(
                plot.left() + plot.width() * i as f32 / 30.0,
                plot.bottom() - plot.height() * (value(i) / ymax) as f32,
            )
        })
        .collect::<Vec<_>>();
    painter.add(egui::Shape::line(
        points.clone(),
        egui::Stroke::new(2.0_f32, ACCENT),
    ));
    if response.clicked() {
        if let Some(p) = response.interact_pointer_pos() {
            *selected =
                (((p.x - plot.left()) / plot.width()).clamp(0.0, 1.0) * 30.0).round() as usize;
        }
    }
    let p = points[(*selected).min(30)];
    painter.line_segment(
        [egui::pos2(p.x, plot.top()), egui::pos2(p.x, plot.bottom())],
        egui::Stroke::new(1.0_f32, MUTED),
    );
    painter.circle_filled(p, 3.5, ACCENT);
}
pub fn panel(
    ui: &mut egui::Ui,
    planner: &mut CropPlanner,
    world: &cosmogon_sim::Universe,
    habitat: Option<u64>,
) {
    ui.strong("Controlled lettuce crop planner");
    ui.label(
        egui::RichText::new("NASA MEC · five corrected low-light regimes · 30 terrestrial days")
            .color(ACCENT),
    );
    ui.label("Controlled forecast · no equipment installed or habitat inventory credits.");
    ui.label(
        egui::RichText::new("23°C · 16 h light/day · 19.2 plants/m² · unlimited water & nutrients")
            .small()
            .color(MUTED),
    );
    egui::CollapsingHeader::new("Growing regime and fixture assumptions").show(ui,|ui| {
        egui::ComboBox::from_id_salt("crop-regime").selected_text(planner.definition.regime.label()).show_ui(ui,|ui| {for regime in Regime::ALL {ui.selectable_value(&mut planner.definition.regime,regime,regime.label());}});
        ui.horizontal(|ui| {ui.label("Planted area (m²)");ui.add(egui::DragValue::new(&mut planner.definition.area_m2).range(0.1..=100_000.0).speed(0.1));});
        ui.horizontal(|ui| {ui.label("Assumed incident photon efficacy (µmol/J)");ui.add(egui::DragValue::new(&mut planner.definition.fixture_efficacy_umol_j).range(0.1..=10.0).speed(0.05));});
        ui.label("Waldmann’s Green lettuce · 23°C light/dark · 16 h light per 24 h cycle · 19.2 plants/m² · unlimited regulated CO₂, water and nutrients.");
    });
    if planner
        .result
        .as_ref()
        .is_none_or(|f| f.definition != planner.definition)
    {
        match planner.definition.calculate() {
            Ok(result) => planner.result = Some(result),
            Err(e) => {
                planner.status = Some(e);
                planner.result = None;
            }
        }
    }
    if let Some(f) = &planner.result {
        let last = &f.rows[30];
        ui.label(format!(
            "{:.1} m² · {}",
            f.definition.area_m2,
            f.definition.regime.label()
        ));
        ui.label(format!(
            "At day 30: {:.3} kg edible dry biomass · {:.3} kg net O₂ · {:.1} kWh lighting",
            last.edible_dry_biomass_kg,
            last.net_oxygen_kg,
            last.lighting_electric_j / 3.6e6
        ));
        ui.label(format!(
            "Lamps-on demand {:.1} W · DLI {:.2} mol photons/(m² day)",
            f.lamps_on_power_w, f.daily_light_integral_mol_m2_day
        ));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut planner.graph, 0, "Edible dry kg");
            ui.selectable_value(&mut planner.graph, 1, "Net oxygen kg");
            ui.selectable_value(&mut planner.graph, 2, "Lighting kWh");
        });
        chart(ui, f, &mut planner.selected_day, planner.graph);
        ui.horizontal(|ui| {
            ui.label("Inspect age after emergence (days)");
            ui.add(egui::Slider::new(&mut planner.selected_day, 0..=30)).on_hover_text("Select an age here or click the graph. The line connects computed daily observations; cumulative values use analytic integration.");
        });
        let r = &f.rows[planner.selected_day];
        egui::Grid::new("crop-selected-row")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (label, v) in [
                    (
                        "Edible / inedible dry biomass",
                        format!(
                            "{:.4} / {:.4} kg",
                            r.edible_dry_biomass_kg, r.inedible_dry_biomass_kg
                        ),
                    ),
                    (
                        "Net CO₂ uptake / O₂ production",
                        format!("{:.4} / {:.4} kg", r.net_co2_kg, r.net_oxygen_kg),
                    ),
                    (
                        "Current daily dry growth",
                        format!("{:.5} kg/day", r.dry_biomass_kg_day),
                    ),
                    (
                        "Canopy absorbed light",
                        format!("{:.2}%", r.canopy_absorbed_fraction * 100.0),
                    ),
                ] {
                    ui.label(label);
                    ui.strong(v);
                    ui.end_row();
                }
            });
        egui::CollapsingHeader::new("Compare the five published regimes").show(ui,|ui| {
            ui.label("Same area and fixture efficacy. Both light and CO₂ may differ; this table is not a single-variable causal experiment.");
            if let Ok(cases)=cosmogon_sim::greenhouse::compare(&planner.definition) {
                egui::Grid::new("crop-comparison").num_columns(4).striped(true).show(ui,|ui| {
                    for title in ["PPF / CO₂","Edible dry kg","Net O₂ kg","Lamp kWh"] {ui.strong(title);}ui.end_row();
                    for case in cases {let (p,c,_)=case.definition.regime.values();ui.label(format!("{p:.0} / {c:.0}"));ui.label(format!("{:.3}",case.harvest.edible_dry_biomass_kg));ui.label(format!("{:.3}",case.harvest.net_oxygen_kg));ui.label(format!("{:.1}",case.harvest.lighting_electric_j/3.6e6));ui.end_row();}
                });
            }
        });
        egui::CollapsingHeader::new("Habitat context and missing connections").show(ui,|ui| {
            if let Some(h)=habitat.and_then(|id|world.habitats.iter().find(|h|h.id==id)) {let p=h.state.pressure_pa(&h.config);let ppm=if p>0.0 {Some(h.state.co2_pa(&h.config)/p*1e6)} else {None};ui.label(format!("Current habitat #{}: {:.2} K · CO₂ {} µmol/mol air",h.id,h.state.temperature_k,ppm.map(|v|format!("{v:.1}")).unwrap_or("unavailable".into())));ui.label(format!("Current generated/served power {:.1}/{:.1} W; crop lamps would require {:.1} W while on. This observation does not establish future headroom or reserve that demand.",h.budget.generated_w,h.budget.delivered_w,f.lamps_on_power_w));}
            else {ui.label("No selected habitat. This standalone crop forecast still requires a supplied controlled environment.");}
            ui.label("Closed-loop greenhouse exchange, nutrient stocks, humidity, transpiration, crop stress, cultivation equipment and actual harvest remain unimplemented. Dry edible lettuce is not a complete crew diet; fresh mass and calories are unavailable.");
        });
        egui::CollapsingHeader::new("Source equations and reproducible export").show(ui,|ui| {
            ui.hyperlink_to("NASA BVAD Rev2 · crop equations and corrected low-light cases",cosmogon_sim::greenhouse::SOURCE);
            if let Some(m)=cosmogon_sim::research::PhysicsModelRegistry::get("greenhouse") {ui.label(m.equations);ui.label(m.expected_error);}
            ui.label("Exports source coefficients, assumptions, definition, all 31 numeric rows and the exact current world/habitat context. Existing folders are protected. Prediction uncertainty is unavailable.");
            if planner.export_path.is_empty() {planner.export_path=crate::persistence::data_dir().join("greenhouse-study").display().to_string();}
            ui.text_edit_singleline(&mut planner.export_path);
            if ui.button("Save forecast to new folder").clicked() {planner.status=Some(match cosmogon_sim::greenhouse::export(&planner.definition,Some((world,habitat)),std::path::Path::new(&planner.export_path)) {Ok(())=>format!("Saved {}",planner.export_path),Err(e)=>e});}
            ui.separator();ui.label("Reopen a source-pinned greenhouse definition");ui.text_edit_singleline(&mut planner.import_path);if ui.button("Open definition").clicked() {let path=std::path::PathBuf::from(&planner.import_path);planner.status=Some(match planner.import(&path) {Ok(())=>"Opened crop definition".into(),Err(e)=>e});}
        });
    }
    if let Some(status) = &planner.status {
        ui.colored_label(ACCENT, status);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_import_preserves_prepared_inputs_results_and_metadata() {
        let root = std::env::temp_dir().join(format!("cosmogon-crop-ui-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let p = root.join("definition.json");
        let mut planner = CropPlanner::default();
        let mut d = Definition {
            area_m2: 37.0,
            ..Default::default()
        };
        std::fs::write(&p, serde_json::to_vec(&d).unwrap()).unwrap();
        planner.import(&p).unwrap();
        let previous = planner.metadata();
        d.model = "future/2".into();
        std::fs::write(&p, serde_json::to_vec(&d).unwrap()).unwrap();
        assert!(planner.import(&p).is_err());
        assert_eq!(planner.metadata(), previous);
        std::fs::write(&p, vec![b' '; 65_537]).unwrap();
        assert!(planner.import(&p).is_err());
        assert_eq!(planner.metadata(), previous);
        std::fs::remove_dir_all(root).unwrap();
    }
}
