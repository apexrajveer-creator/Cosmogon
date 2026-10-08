//! Native view of the shared finite CO₂ reduction and downstream water processing.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{habitat::Habitat, sabatier};

pub fn controls(ui: &mut egui::Ui, equipment: &mut Option<sabatier::Config>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable carbon dioxide reduction (Sabatier)")
        .changed()
    {
        *equipment = enabled.then(sabatier::Config::default);
    }
    ui.label("Staged settings. Disable retains products; held raw water can still enter the powered recycler.");
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("CO₂ reduction equipment assumptions").default_open(expand).show(ui,|ui| {
            super::habitat::number(ui,"Maximum electric power (W)",&mut c.power_limit_w,0.0,1e6);
            let original=c.co2_rate_kg_s*cosmogon_sim::time::SECONDS_PER_DAY;let mut rate=original;
            super::habitat::number(ui,"Maximum CO₂ processing (kg/day)",&mut rate,0.0,8640000.0);
            if rate!=original {c.co2_rate_kg_s=rate/cosmogon_sim::time::SECONDS_PER_DAY;}
            let original=c.electricity_j_kg_co2/1e6;let mut energy=original;
            super::habitat::number(ui,"Process electricity (MJ/kg CO₂)",&mut energy,0.000001,1000.0);
            if energy!=original {c.electricity_j_kg_co2=energy*1e6;}
            super::habitat::number(ui,"Isolated methane capacity (kg)",&mut c.methane_capacity_kg,0.0,1e7);
            super::habitat::number(ui,"Raw liquid product capacity (kg)",&mut c.water_capacity_kg,0.0,1e7);
            ui.label("Illustrative ratings. No implicit methane disposal or fuel use. Raw liquid uses the existing assumed recycler recovery fraction, after wastewater.");
        });
    }
}
pub fn readings(ui: &mut egui::Ui, h: &Habitat) {
    ui.strong("Captured CO₂ + hydrogen → methane + raw water");
    ui.label(
        egui::RichText::new("Finite stores · NIST reference chemistry · assumed process hardware")
            .color(ACCENT),
    );
    ui.label(format!(
        "Held methane {:.6} kg · raw liquid {:.6} kg",
        h.state.methane_kg, h.state.sabatier_water_kg
    ));
    ui.label(format!(
        "Available captured CO₂ {:.6} kg · isolated H₂ {:.6} kg",
        h.state.captured_co2_kg, h.state.hydrogen_kg
    ));
    if h.config.sabatier.is_none() {
        ui.label("Reaction equipment disabled; products and history remain.");
    }
    if let Some(b) = &h.budget.sabatier {
        ui.label(format!(
            "Last completed {:.2} s interval; its configuration is retained.",
            h.budget.seconds
        ));
        egui::Grid::new("sabatier-readings")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (label, value) in [
                    (
                        "Electric power requested / served",
                        format!("{:.2} / {:.2} W", b.requested_w, b.delivered_w),
                    ),
                    (
                        "Captured CO₂ / hydrogen consumed",
                        format!(
                            "{:.6} / {:.6} kg",
                            b.co2_consumed_kg, b.hydrogen_consumed_kg
                        ),
                    ),
                    (
                        "Methane / raw water produced",
                        format!(
                            "{:.6} / {:.6} kg",
                            b.methane_generated_kg, b.water_generated_kg
                        ),
                    ),
                    (
                        "Process electric energy",
                        format!("{:.3} kJ", b.electricity_j / 1000.0),
                    ),
                    (
                        "Reference reaction heat released",
                        format!("{:.3} kJ", b.reaction_heat_j / 1000.0),
                    ),
                    (
                        "Total component heat",
                        format!("{:.3} kJ", b.total_heat_j / 1000.0),
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
                    "Processing constraints: {}",
                    b.constraints
                        .iter()
                        .map(|c| c.label())
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            );
        }
    } else {
        ui.label("No completed active reaction interval; flow observations are unavailable.");
    }
    ui.label(format!(
        "Raw product processed / recovered this interval {:.6} / {:.6} kg",
        h.budget.sabatier_water_processed_kg, h.budget.sabatier_water_recovered_kg
    ));
    egui::CollapsingHeader::new("Chemistry, history and limits").show(ui,|ui| {
        ui.label("CO₂(g) + 4 H₂(g) → CH₄(g) + 2 H₂O(l). Overall reference ΔH = −252.750 kJ/mol CO₂ at 298.15 K, including liquid condensation. Process electricity plus released reference heat enters the shared thermal budget.");
        ui.label(format!("Cumulative CO₂ reacted {:.6} kg · reference heat {:.3} MJ · raw product processed {:.6} kg",h.totals.sabatier_co2_consumed_kg,h.totals.sabatier_heat_j/1e6,h.totals.sabatier_water_processed_kg));
        ui.hyperlink_to("NIST CO₂ CODATA thermochemistry",sabatier::CO2_SOURCE);ui.hyperlink_to("NIST methane · Manion adopted recommendation",sabatier::METHANE_SOURCE);
        ui.label(egui::RichText::new("Start-of-interval feed only. Raw product waits for later powered processing. Reference chemistry does not establish catalyst operation, purity, storage pressure, safe water or field accuracy. No venting, combustion, reactor startup or closed-loop guarantee. Export actual CSV/JSON trajectories and source metadata in Model & export / CLI habitat.").small().color(MUTED));
    });
}
