//! Native planning instrument over the shared source model. No hidden resource credits.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::greenhouse::{Definition, Forecast, MODEL, Regime, WaterAssumptions};

pub struct CropPlanner {
    pub definition: Definition,
    pub result: Option<Forecast>,
    selected_day: usize,
    pub graph: usize,
    pub pane: usize,
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
            pane: 0,
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
        serde_json::json!({"prepared_definition":self.definition,"forecast":self.result,"selected_age_days_after_emergence":self.selected_day,"pane":self.pane,"graph":self.graph,"prepared_inputs_match_forecast":self.result.as_ref().is_some_and(|f|f.definition==self.definition),"semantics":"Session planning view; may be hidden. Forecast does not install a greenhouse or allocate habitat resources."})
    }
}
fn chart(ui: &mut egui::Ui, f: &Forecast, selected: &mut usize, metric: usize) {
    let value = |i: usize| {
        let r = &f.rows[i];
        match metric {
            1 => r.net_oxygen_kg,
            2 => r.lighting_electric_j / 3.6e6,
            3 => r
                .transpiration
                .as_ref()
                .map(|w| w.transpired_water_l)
                .unwrap_or(0.0),
            4 => r
                .transpiration
                .as_ref()
                .map(|w| w.transpired_water_l_day)
                .unwrap_or(0.0),
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
        egui::RichText::new("NASA MEC · five regimes · 23°C · 16 h light per terrestrial day")
            .color(ACCENT),
    );
    ui.label(
        egui::RichText::new("Planning only · no installed equipment or habitat inventory credits")
            .small()
            .color(MUTED),
    );
    ui.horizontal_wrapped(|ui| {
        for (i, label) in ["Forecast", "Inputs", "Compare", "Water model", "Files"]
            .iter()
            .enumerate()
        {
            ui.selectable_value(&mut planner.pane, i, *label);
        }
    });
    ui.separator();
    if planner.pane == 1 {
        ui.strong("Growing regime and user assumptions");
        egui::ComboBox::from_id_salt("crop-regime")
            .selected_text(planner.definition.regime.label())
            .show_ui(ui, |ui| {
                for regime in Regime::ALL {
                    ui.selectable_value(&mut planner.definition.regime, regime, regime.label());
                }
            });
        ui.horizontal(|ui| {
            ui.label("Planted area (m²)");
            ui.add(
                egui::DragValue::new(&mut planner.definition.area_m2)
                    .range(0.1..=100_000.0)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Assumed incident photon efficacy (µmol/J)");
            ui.add(
                egui::DragValue::new(&mut planner.definition.fixture_efficacy_umol_j)
                    .range(0.1..=10.0)
                    .speed(0.05),
            );
        });
        let mut enabled = planner.definition.transpiration.is_some();
        if ui
            .checkbox(&mut enabled, "Estimate light-period transpiration")
            .changed()
        {
            planner.definition.model = MODEL.into();
            planner.definition.transpiration = enabled.then(WaterAssumptions::default);
        }
        if let Some(w) = &mut planner.definition.transpiration {
            ui.horizontal(|ui| {
                ui.label("Assumed relative humidity (fraction)");
                ui.add(
                    egui::DragValue::new(&mut w.relative_humidity)
                        .range(0.0..=1.0)
                        .speed(0.005),
                );
            });
            ui.label(
                "Maintained humidity · fixed assumed 101.325 kPa · source leaf temperature 23°C.",
            );
            ui.label("Dry air producing negative conductance is refused. Saturated-air zero flux is a mathematical boundary, not a growing recommendation. Humidity does not model crop stress or alter biomass.");
        } else {
            ui.label("Transpiration is unavailable. Older definitions retain biomass-only semantics until explicitly enabled; imported files are never rewritten.");
        }
        ui.label("Waldmann’s Green lettuce · 19.2 plants/m² · pinned PPF/CO₂ pairs; no arbitrary regime interpolation.");
        ui.label("Fixture efficacy includes optical/fixture losses. No pumps, cooling or dehumidification electricity is inferred.");
    }
    if planner
        .result
        .as_ref()
        .is_none_or(|f| f.definition != planner.definition)
    {
        match planner.definition.calculate() {
            Ok(result) => {
                planner.result = Some(result);
                planner.status = None;
            }
            Err(e) => {
                planner.status = Some(e);
                planner.result = None;
            }
        }
    }
    if let Some(f) = &planner.result {
        let last = &f.rows[30];
        match planner.pane {
            0 => {
                ui.label(format!(
                    "{:.1} m² · {}",
                    f.definition.area_m2,
                    f.definition.regime.label()
                ));
                ui.label(format!(
                    "Day 30: {:.3} kg edible dry · {:.3} kg O₂ · {:.1} kWh lamps",
                    last.edible_dry_biomass_kg,
                    last.net_oxygen_kg,
                    last.lighting_electric_j / 3.6e6
                ));
                if let (Some(w), Some(meta)) = (&last.transpiration, &f.transpiration) {
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.2} L transpired · assumed RH {:.1}% · {:.3} kPa deficit",
                            w.transpired_water_l,
                            meta.assumed_relative_humidity * 100.0,
                            meta.vapor_pressure_deficit_kpa
                        ))
                        .color(ACCENT),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Liquid-equivalent vapor transfer, not recovered or potable water.",
                        )
                        .small()
                        .color(MUTED),
                    );
                } else {
                    ui.colored_label(
                        ACCENT,
                        "Water unavailable · this definition has no humidity assumption.",
                    );
                    if planner.graph >= 3 {
                        planner.graph = 0;
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    for (i, label) in [
                        "Edible dry kg",
                        "Net O₂ kg",
                        "Lighting kWh",
                        "Total water L",
                        "Daily water L/d",
                    ]
                    .iter()
                    .enumerate()
                    {
                        ui.add_enabled_ui(i < 3 || f.transpiration.is_some(), |ui| {
                            ui.selectable_value(&mut planner.graph, i, *label);
                        });
                    }
                });
                chart(ui, f, &mut planner.selected_day, planner.graph);
                ui.horizontal(|ui| {
                    ui.label("Age after emergence (terrestrial days)");
                    ui.add(egui::Slider::new(&mut planner.selected_day,0..=30)).on_hover_text("Click the graph or select a computed age. Biomass is integrated analytically; water uses deterministic numerical quadrature. Lines connect observations, not measured crop data.");
                });
                let r = &f.rows[planner.selected_day];
                egui::Grid::new("crop-selected-row")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        for (label, value) in [
                            (
                                "Edible / inedible dry biomass",
                                format!(
                                    "{:.4} / {:.4} kg",
                                    r.edible_dry_biomass_kg, r.inedible_dry_biomass_kg
                                ),
                            ),
                            (
                                "Net CO₂ uptake / O₂",
                                format!("{:.4} / {:.4} kg", r.net_co2_kg, r.net_oxygen_kg),
                            ),
                            (
                                "Daily dry growth / light absorbed",
                                format!(
                                    "{:.5} kg/d / {:.2}%",
                                    r.dry_biomass_kg_day,
                                    r.canopy_absorbed_fraction * 100.0
                                ),
                            ),
                        ] {
                            ui.label(label);
                            ui.strong(value);
                            ui.end_row();
                        }
                        if let Some(w) = &r.transpiration {
                            for (label, value) in [
                                (
                                    "Cumulative / daily transpiration",
                                    format!(
                                        "{:.4} L / {:.4} L/d",
                                        w.transpired_water_l, w.transpired_water_l_day
                                    ),
                                ),
                                (
                                    "Cumulative / daily vapor mass",
                                    format!(
                                        "{:.4} kg / {:.4} kg/d",
                                        w.transpired_vapor_kg, w.transpired_vapor_kg_day
                                    ),
                                ),
                            ] {
                                ui.label(label);
                                ui.strong(value);
                                ui.end_row();
                            }
                        }
                    });
            }
            1 => {
                ui.separator();
                ui.label(format!(
                    "Current lamps-on demand {:.1} W · DLI {:.2} mol/(m² day)",
                    f.lamps_on_power_w, f.daily_light_integral_mol_m2_day
                ));
            }
            2 => {
                ui.strong("Harvest at the five published regimes");
                ui.label("Same area, efficacy and humidity assumption. Light and CO₂ may both differ; not a single-variable causal experiment.");
                if let Ok(cases) = cosmogon_sim::greenhouse::compare(&planner.definition) {
                    egui::Grid::new("crop-comparison")
                        .num_columns(5)
                        .striped(true)
                        .show(ui, |ui| {
                            for label in [
                                "PPF / CO₂",
                                "Dry edible kg",
                                "Net O₂ kg",
                                "Lamps kWh",
                                "Water L",
                            ] {
                                ui.strong(label);
                            }
                            ui.end_row();
                            for c in cases {
                                let (p, co2, _) = c.definition.regime.values();
                                ui.label(format!("{p:.0} / {co2:.0}"));
                                ui.label(format!("{:.3}", c.harvest.edible_dry_biomass_kg));
                                ui.label(format!("{:.3}", c.harvest.net_oxygen_kg));
                                ui.label(format!("{:.1}", c.harvest.lighting_electric_j / 3.6e6));
                                ui.label(
                                    c.harvest
                                        .transpiration
                                        .as_ref()
                                        .map(|w| format!("{:.2}", w.transpired_water_l))
                                        .unwrap_or("Unavailable".into()),
                                );
                                ui.end_row();
                            }
                        });
                }
                ui.label("PPF: µmol photons/(m² s) · CO₂: µmol/mol air. Water is modeled light-period vapor transfer; no condensate equipment is assumed.");
            }
            3 => {
                ui.strong("Water model and numerical evidence");
                if let Some(w) = &f.transpiration {
                    egui::Grid::new("crop-water-model")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for (label, value) in [
                                (
                                    "Assumed RH / pressure",
                                    format!(
                                        "{:.1}% / {:.3} kPa",
                                        w.assumed_relative_humidity * 100.0,
                                        w.assumed_pressure_kpa
                                    ),
                                ),
                                (
                                    "Saturated / air vapor pressure",
                                    format!(
                                        "{:.5} / {:.5} kPa",
                                        w.saturated_vapor_pressure_kpa, w.air_vapor_pressure_kpa
                                    ),
                                ),
                                (
                                    "Vapor pressure deficit",
                                    format!("{:.5} kPa", w.vapor_pressure_deficit_kpa),
                                ),
                                (
                                    "Aerodynamic conductance",
                                    format!("{:.2} mol/(m² s)", w.aerodynamic_conductance_mol_m2_s),
                                ),
                                (
                                    "Water molecular mass / density",
                                    format!(
                                        "{:.3} g/mol / {:.2} g/L",
                                        w.water_molecular_g_mol, w.water_density_g_l
                                    ),
                                ),
                            ] {
                                ui.label(label);
                                ui.strong(value);
                                ui.end_row();
                            }
                        });
                    let row = &f.rows[planner.selected_day];
                    if let Some(r) = &row.transpiration {
                        ui.label(format!(
                            "Day {}: gross / light-cycle net {:.5} / {:.5} µmol C/(m² s)",
                            planner.selected_day,
                            r.gross_carbon_umol_m2_s,
                            r.light_cycle_net_carbon_umol_m2_s
                        ));
                        ui.label(format!(
                            "Stomatal / series canopy conductance {:.6} / {:.6} mol/(m² s)",
                            r.stomatal_conductance_mol_m2_s, r.canopy_conductance_mol_m2_s
                        ));
                    }
                    ui.label(format!(
                        "Simpson {} vs {} subdivisions/day: harvest difference {:.3e} L",
                        w.subdivisions_per_day,
                        w.subdivisions_per_day * 2,
                        w.harvest_resolution_difference_l
                    ));
                    ui.label("Resolution difference is numerical evidence, not an error bound or predictive uncertainty. Independent empirical accuracy remains unavailable.");
                    ui.hyperlink_to(
                        "NASA BVAD Rev2 · equations 4-23–27 and 4-29",
                        cosmogon_sim::greenhouse::SOURCE,
                    );
                    ui.label(&w.semantics);
                } else {
                    ui.label("Transpiration unavailable. Explicitly enable it in Inputs to add a versioned humidity assumption. No water value is fabricated for legacy definitions.");
                }
            }
            _ => {
                let lamps_on_power_w = f.lamps_on_power_w;
                ui.strong("Reproducible forecast files");
                ui.label("Exports all 31 rows, assumptions, coefficients, sources, numerical evidence and exact optional context. Existing folders are protected.");
                if planner.export_path.is_empty() {
                    planner.export_path = crate::persistence::data_dir()
                        .join("greenhouse-study")
                        .display()
                        .to_string();
                }
                ui.text_edit_singleline(&mut planner.export_path);
                if ui.button("Save forecast to new folder").clicked() {
                    planner.status = Some(
                        match cosmogon_sim::greenhouse::export(
                            &planner.definition,
                            Some((world, habitat)),
                            std::path::Path::new(&planner.export_path),
                        ) {
                            Ok(()) => format!("Saved {}", planner.export_path),
                            Err(e) => e,
                        },
                    );
                }
                ui.separator();
                ui.label("Reopen a source-pinned definition");
                ui.text_edit_singleline(&mut planner.import_path);
                if ui.button("Open definition").clicked() {
                    let path = std::path::PathBuf::from(&planner.import_path);
                    planner.status = Some(match planner.import(&path) {
                        Ok(()) => "Opened crop definition without rewriting it".into(),
                        Err(e) => e,
                    });
                }
                egui::CollapsingHeader::new("Source, limits and observed habitat context").show(ui,|ui| {
                    ui.hyperlink_to("NASA BVAD Rev2",cosmogon_sim::greenhouse::SOURCE);
                    ui.label(cosmogon_sim::greenhouse::LIMITS);
                    if let Some(h)=habitat.and_then(|id|world.habitats.iter().find(|h|h.id==id)) {
                        let pressure=h.state.pressure_pa(&h.config);let ppm=if pressure>0.0 {Some(h.state.co2_pa(&h.config)/pressure*1e6)}else{None};
                        ui.label(format!("Observed habitat #{}: {:.2} K · {:.2} kPa · CO₂ {} µmol/mol",h.id,h.state.temperature_k,pressure/1000.0,ppm.map(|v|format!("{v:.1}")).unwrap_or("unavailable".into())));
                        ui.label(format!("Generated/served power {:.1}/{:.1} W; planner lamps require {:.1} W while on. No future headroom, maintained crop boundary or reserve is established.",h.budget.generated_w,h.budget.delivered_w,lamps_on_power_w));
                    } else {ui.label("No selected habitat; controlled boundary inputs still require an external supplied chamber.");}
                    ui.label("Humidity dynamics, finite irrigation/nutrients/CO₂, hardware, actual harvest, full facility energy and independent calibration remain incomplete. Dry lettuce is not a complete crew diet.");
                });
            }
        }
    }
    if let Some(status) = &planner.status {
        ui.colored_label(ACCENT, status);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_native_import_has_no_water_and_invalid_water_preserves_result() {
        let root = std::env::temp_dir().join(format!("cosmogon-water-ui-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("definition.json");
        let d = Definition {
            model: cosmogon_sim::greenhouse::LEGACY_MODEL.into(),
            transpiration: None,
            ..Default::default()
        };
        std::fs::write(&path, serde_json::to_vec(&d).unwrap()).unwrap();
        let mut planner = CropPlanner::default();
        planner.import(&path).unwrap();
        assert!(planner.result.as_ref().unwrap().transpiration.is_none());
        let previous = planner.metadata();
        let d = Definition {
            transpiration: Some(WaterAssumptions {
                relative_humidity: 0.1,
                ..Default::default()
            }),
            ..Default::default()
        };
        std::fs::write(&path, serde_json::to_vec(&d).unwrap()).unwrap();
        assert!(planner.import(&path).is_err());
        assert_eq!(planner.metadata(), previous);
        std::fs::write(&path, serde_json::to_vec(&Definition::default()).unwrap()).unwrap();
        planner.import(&path).unwrap();
        assert!(planner.result.as_ref().unwrap().transpiration.is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
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
