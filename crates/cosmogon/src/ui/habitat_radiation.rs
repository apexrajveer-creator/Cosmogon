//! Native controls author the optional exterior-radiation environment and cabin
//! shielding dose model; every value is an authored design input flowing through the
//! ordinary draft configuration Apply path.
use bevy_egui::egui;
use cosmogon_sim::habitat_radiation::Config as RadiationConfig;

pub fn controls(ui: &mut egui::Ui, radiation: &mut Option<RadiationConfig>, expand: bool) {
    let mut enabled = radiation.is_some();
    if ui
        .checkbox(&mut enabled, "Enable radiation shielding model")
        .changed()
    {
        *radiation = enabled.then(RadiationConfig::default);
    }
    if let Some(c) = radiation {
        egui::CollapsingHeader::new("Shielding transmissions and threshold")
            .default_open(expand)
            .show(ui, |ui| {
                super::habitat::number(ui, "Wall transmission (fraction)", &mut c.wall_transmission, 0.0, 1.0);
                super::habitat::number(ui, "Storm-shelter transmission (fraction)", &mut c.storm_shelter_transmission, 0.0, 1.0);
                super::habitat::number(ui, "Alert threshold (mSv/day)", &mut c.alert_dose_rate_msv_day, 0.0, 1.0e4);
                ui.label(egui::RichText::new(format!("Model {} is pinned. Interior dose is exterior × wall transmission, shelter is interior × shelter transmission; exterior dose comes from the sampled body/star environment. Transmissions are authored constant fractions, not measured attenuation; bands are authored design categories, not dosimetry or safety limits. No particle transport, solar-particle-event forecast or medical claim.", cosmogon_sim::habitat_radiation::MODEL)).small().color(super::MUTED));
            });
    }
}
