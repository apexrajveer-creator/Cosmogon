//! Native controls observe the shared scheduled wear model and stage ordinary edits.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{
    habitat::Habitat,
    habitat_maintenance::{ComponentMaintenance, MaintenanceConfig, MAINTENANCE_MODEL},
};

const NAMES: [&str; 7] = [
    "Scrubber",
    "Recycler",
    "Heater",
    "Radiator",
    "Moisture",
    "Electrolysis",
    "Sabatier",
];

fn slot_mut(c: &mut MaintenanceConfig, i: usize) -> &mut Option<ComponentMaintenance> {
    match i {
        0 => &mut c.scrubber,
        1 => &mut c.recycler,
        2 => &mut c.heater,
        3 => &mut c.radiator,
        4 => &mut c.moisture,
        5 => &mut c.electrolysis,
        6 => &mut c.sabatier,
        _ => unreachable!(),
    }
}

pub fn controls(ui: &mut egui::Ui, equipment: &mut Option<MaintenanceConfig>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable scheduled equipment maintenance")
        .changed()
    {
        *equipment = enabled.then(|| MaintenanceConfig {
            scrubber: Some(ComponentMaintenance::default()),
            ..Default::default()
        });
    }
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("Component schedules and rates")
            .default_open(expand)
            .show(ui, |ui| {
                for i in 0..NAMES.len() {
                    let name = NAMES[i];
                    let mut on = slot_mut(c, i).is_some();
                    if ui.checkbox(&mut on, name).changed() {
                        *slot_mut(c, i) = on.then(ComponentMaintenance::default);
                    }
                    if let Some(comp) = slot_mut(c, i) {
                        ui.indent(format!("{name}-maintenance"), |ui| {
                            super::habitat::number(
                                ui,
                                "Wear rate (condition fraction per hour)",
                                &mut comp.wear_per_hour,
                                0.0,
                                1000.0,
                            );
                            super::habitat::number(
                                ui,
                                "Repair rate (condition fraction per hour)",
                                &mut comp.repair_per_hour,
                                1.0e-6,
                                1.0e6,
                            );
                            super::habitat::number(
                                ui,
                                "Operating interval between windows (h)",
                                &mut comp.interval_hours,
                                60.0 / 3600.0,
                                1.0e6,
                            );
                            super::habitat::number(
                                ui,
                                "Maintenance window duration (h)",
                                &mut comp.duration_hours,
                                60.0 / 3600.0,
                                1.0e6,
                            );
                            super::habitat::number(
                                ui,
                                "Maintenance bus power (W)",
                                &mut comp.maintenance_power_w,
                                0.0,
                                1.0e6,
                            );
                            super::habitat::number(
                                ui,
                                "Capacity calibration (fraction of rated)",
                                &mut comp.capacity_calibration,
                                1.0e-6,
                                1.0,
                            );
                        });
                    }
                }
                ui.label("At least one component must stay configured. Calibration above rated capacity is refused because process ratings are the modeled ceilings. Disabling freezes held condition and cycle position; re-enabling resumes from the retained values.");
            });
    }
}

pub fn readings(ui: &mut egui::Ui, h: &Habitat) {
    ui.strong("Scheduled equipment wear and maintenance");
    ui.label(
        egui::RichText::new("Authored wear and repair rates · capacity calibration · ordinary bus power")
            .color(ACCENT),
    );
    if !h.uses_maintenance() {
        ui.label("Scheduled maintenance is not enabled; installed equipment stays at rated capacity indefinitely. No maintenance observation is available.");
        return;
    }
    let worst = h
        .state
        .maintenance
        .as_ref()
        .map(|s| {
            s.slots()
                .iter()
                .filter_map(|slot| **slot)
                .map(|c| c.condition)
                .fold(1.0_f64, f64::min)
        })
        .unwrap_or(1.0);
    ui.label(format!(
        "Cumulative maintenance {:.4} h · lowest condition {:.1}%",
        h.totals.maintenance_hours,
        100.0 * worst
    ));
    if let Some(s) = &h.state.maintenance {
        ui.strong("Held condition and cycle position");
        egui::Grid::new("maintenance-state")
            .striped(true)
            .show(ui, |ui| {
                for label in ["Component", "Condition", "Cycle position"] {
                    ui.label(egui::RichText::new(label).strong());
                }
                ui.end_row();
                for (i, name) in NAMES.iter().enumerate() {
                    let Some(cond) = *s.slots()[i] else {
                        continue;
                    };
                    ui.label(*name);
                    ui.strong(format!("{:.1}%", 100.0 * cond.condition));
                    ui.strong(format!("{:.0} s", cond.cycle_s));
                    ui.end_row();
                }
            });
    }
    if let Some(b) = &h.budget.maintenance {
        ui.label(format!(
            "Last completed {:.3} s interval · end {:.3} s",
            h.budget.seconds, h.last_time
        ));
        egui::Grid::new("maintenance-interval")
            .striped(true)
            .show(ui, |ui| {
                for label in [
                    "Component",
                    "Condition before → after",
                    "Operating / maintenance",
                    "Served",
                    "Scale",
                    "Wear",
                    "Restored",
                ] {
                    ui.label(egui::RichText::new(label).strong());
                }
                ui.end_row();
                for (i, name) in NAMES.iter().enumerate() {
                    let Some(cb) = *b.slots()[i] else {
                        continue;
                    };
                    ui.label(*name);
                    ui.strong(format!(
                        "{:.1} → {:.1}%",
                        100.0 * cb.condition_before,
                        100.0 * cb.condition_after
                    ));
                    ui.label(format!(
                        "{:.1} / {:.1} s",
                        cb.operating_s, cb.maintenance_s
                    ));
                    ui.label(format!("{:.1}%", 100.0 * cb.served_fraction));
                    ui.label(format!("{:.3}", cb.effective_scale));
                    ui.label(format!("{:.5}", cb.wear));
                    ui.label(format!("{:.5}", cb.restored));
                    ui.end_row();
                }
            });
    } else {
        ui.label("No completed interval under the maintenance configuration yet; wear observations unavailable.");
    }
    egui::CollapsingHeader::new("Maintenance model and limits").show(ui, |ui| {
        ui.label(format!(
            "Model {MAINTENANCE_MODEL} · condition lost per served operating hour, restored per maintenance hour; process capacity scales by calibration × condition."
        ));
        ui.label("Maintenance power is ordinary bus electricity and becomes cabin heat through the existing ledger. No crew-labor, spares or consumable mass is tracked.");
        ui.label(egui::RichText::new("Wear/repair rates and calibration are authored assumptions, not measured hardware reliability. No stochastic failures, spares inventory, age curves beyond linear scaling, or maintenance of solar, battery, regulator, crop or storage hardware. Field calibration remains unquantified.").small().color(MUTED));
    });
}

pub fn model(ui: &mut egui::Ui) {
    ui.strong("Scheduled equipment wear and maintenance · source equations and explicit assumptions");
    ui.label("condition(t+dt) = clamp(condition(t) − wear · served · op_h + repair · maint_h, 0, 1); effective scale = capacity_calibration · condition.");
    ui.label("Cycle = interval + duration; each simulation interval intersects [cycle, cycle+dt) with the operating and maintenance phases exactly, including wrap-around.");
    ui.label("Process capture, rated production, heater limit and radiator rejection multiply by the component scale. A maintenance window requests only maintenance power and produces nothing.");
    ui.label(format!("Model {MAINTENANCE_MODEL}"));
    ui.hyperlink_to(
        "NASA BVAD Rev2 · nominal crew interfaces",
        cosmogon_sim::habitat::SOURCE,
    );
    ui.label("Ratings, wear and repair rates are user assumptions. Numerical conservation does not establish predictive accuracy, hardware reliability or field calibration.");
    ui.label(egui::RichText::new("No stochastic failures, spares, crew labor, consumable mass, age curves beyond linear scaling, or maintenance of solar, battery, regulator, crop or storage hardware. Uncertainty propagation and field calibration remain unquantified.").small().color(MUTED));
}
