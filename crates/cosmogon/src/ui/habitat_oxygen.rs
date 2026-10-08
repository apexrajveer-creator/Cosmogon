//! Presentation over authoritative reaction stocks and interval flows.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{electrolysis, habitat::Habitat};

pub fn controls(ui: &mut egui::Ui, equipment: &mut Option<electrolysis::Config>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable water electrolysis")
        .changed()
    {
        *equipment = enabled.then(electrolysis::Config::default);
    }
    ui.label("Staged equipment settings. Disabling keeps held products; capacity changes cannot discard them.");
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("Electrolysis equipment assumptions").default_open(expand).show(ui, |ui| {
            super::habitat::number(ui, "Maximum electric power (W)", &mut c.power_limit_w, 0.0, 1.0e6);
            let original_rate = c.oxygen_rate_kg_s * cosmogon_sim::time::SECONDS_PER_DAY;
            let mut rate = original_rate;
            super::habitat::number(ui, "Maximum oxygen production (kg/day)", &mut rate, 0.0, 8_640_000.0);
            if rate != original_rate { c.oxygen_rate_kg_s = rate / cosmogon_sim::time::SECONDS_PER_DAY; }
            let original_energy = c.electricity_j_kg_oxygen / 1.0e6;
            let mut energy = original_energy;
            super::habitat::number(ui, "Electricity per oxygen (MJ/kg)", &mut energy, electrolysis::REFERENCE_ENTHALPY_J_KG_OXYGEN / 1.0e6, 1000.0);
            if energy != original_energy { c.electricity_j_kg_oxygen = energy * 1.0e6; }
            super::habitat::number(ui, "Shared oxygen reserve capacity (kg)", &mut c.oxygen_capacity_kg, 0.0, 1.0e7);
            super::habitat::number(ui, "Isolated hydrogen product capacity (kg)", &mut c.hydrogen_capacity_kg, 0.0, 1.0e7);
            ui.label("Illustrative ratings, not flight hardware. No automatic venting, compression, tank structure or Sabatier processing.");
        });
    }
}
pub fn readings(ui: &mut egui::Ui, h: &Habitat) {
    ui.strong("Water + electricity → stored oxygen + hydrogen");
    ui.label(
        egui::RichText::new("Finite resources · NIST reference chemistry · user-assumed equipment")
            .color(ACCENT),
    );
    ui.label(format!(
        "Held oxygen reserve {:.4} kg · isolated hydrogen {:.6} kg",
        h.state.oxygen_reserve_kg, h.state.hydrogen_kg
    ));
    if h.config.electrolysis.is_none() {
        ui.label("Equipment is currently disabled; held products and cumulative reaction history remain.");
    }
    if let Some(b) = &h.budget.electrolysis {
        ui.label(format!(
            "Last completed {:.2} s interval; its configuration is retained in the report.",
            h.budget.seconds
        ));
        egui::Grid::new("oxygen-reaction-readings")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (label, value) in [
                    (
                        "Electric power requested / served",
                        format!("{:.2} / {:.2} W", b.requested_w, b.delivered_w),
                    ),
                    ("Water consumed", format!("{:.6} kg", b.water_consumed_kg)),
                    (
                        "Oxygen produced to reserve",
                        format!("{:.6} kg", b.oxygen_generated_kg),
                    ),
                    (
                        "Hydrogen produced to isolated store",
                        format!("{:.6} kg", b.hydrogen_generated_kg),
                    ),
                    (
                        "Electric energy",
                        format!("{:.3} kJ", b.electricity_j / 1000.0),
                    ),
                    (
                        "Reference reaction enthalpy transfer",
                        format!("{:.3} kJ", b.reaction_enthalpy_j / 1000.0),
                    ),
                    (
                        "Reactor waste heat",
                        format!("{:.3} kJ", b.waste_heat_j / 1000.0),
                    ),
                ] {
                    ui.label(label);
                    ui.strong(value);
                    ui.end_row();
                }
            });
        if !b.constraints.is_empty() {
            ui.colored_label(
                ACCENT,
                format!(
                    "Production constraints: {}",
                    b.constraints
                        .iter()
                        .map(|c| c.label())
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            );
        }
    } else {
        ui.colored_label(MUTED, "No completed active reaction interval. Disabled, waiting or suspended budgets are not zero production measurements.");
    }
    ui.label(format!(
        "Cumulative reference enthalpy transferred {:.4} MJ",
        h.totals.reaction_enthalpy_j / 1.0e6
    ));
    egui::CollapsingHeader::new("How production is limited and accounted").show(ui,|ui| {
        ui.label("Base services, capture, recycling and thermal control get power first. Nominal crew water is reserved before planning production. Rate, water, electric power and both product stores bound the reaction. Recovered water and freed storage are usable next interval; new O₂ reaches the ordinary regulator next interval.");
        ui.label("Reactor electric energy = reference reaction enthalpy + waste heat. Reaction enthalpy is subtracted from the habitat thermal budget, not counted twice. Cumulative transfer is not current recoverable chemical energy after metabolic exchanges.");
        ui.hyperlink_to("NIST liquid water thermochemistry",electrolysis::ENTHALPY_SOURCE);
        if let Some(m)=cosmogon_sim::research::PhysicsModelRegistry::get("electrolysis") {ui.label(m.equations);ui.label(m.limitations);}
        ui.label("Model & export records the exact world, interval, optional chemistry model, source constants and numeric CSV/JSON rows. Equipment ratings and predictive uncertainty are unvalidated.");
    });
}
