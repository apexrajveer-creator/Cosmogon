//! Native controls author the optional life-support calibration factors and sensitivity
//! bands; every value is an authored design input flowing through the ordinary draft
//! configuration Apply path.
use bevy_egui::egui;
use cosmogon_sim::habitat_calibration::Config as CalibrationConfig;

pub fn controls(ui: &mut egui::Ui, calibration: &mut Option<CalibrationConfig>, expand: bool) {
    let mut enabled = calibration.is_some();
    if ui
        .checkbox(&mut enabled, "Enable life-support calibration")
        .changed()
    {
        *calibration = enabled.then(CalibrationConfig::default);
    }
    if let Some(c) = calibration {
        egui::CollapsingHeader::new("Calibration factors and bands")
            .default_open(expand)
            .show(ui, |ui| {
                super::habitat::number(ui, "Crew demand factor (×reference)", &mut c.crew_demand.central, 0.1, 10.0);
                super::habitat::number(ui, "Crew demand band (± fraction)", &mut c.crew_demand.relative_band, 0.0, 1.0);
                super::habitat::number(ui, "CO₂ capture factor (×reference)", &mut c.scrubber.central, 0.1, 10.0);
                super::habitat::number(ui, "CO₂ capture band (± fraction)", &mut c.scrubber.relative_band, 0.0, 1.0);
                super::habitat::number(ui, "Water processing factor (×reference)", &mut c.recycler.central, 0.1, 10.0);
                super::habitat::number(ui, "Water processing band (± fraction)", &mut c.recycler.relative_band, 0.0, 1.0);
                super::habitat::number(ui, "O₂ makeup factor (×reference)", &mut c.oxygen_makeup.central, 0.1, 10.0);
                super::habitat::number(ui, "O₂ makeup band (± fraction)", &mut c.oxygen_makeup.relative_band, 0.0, 1.0);
                ui.label(egui::RichText::new("Factors multiply the published reference rates in place (1.0 keeps the reference). Bands are authored ± sensitivity bounds around applied interval exchanges, not measured variance or confidence intervals; no fitted parameters are claimed.").small().color(super::MUTED));
            });
    }
}
