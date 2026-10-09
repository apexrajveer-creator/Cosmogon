//! Native controls expose actual installed material separately from planning forecasts.
use super::{habitat::number, ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{crop_process, habitat::Habitat};
pub fn readings(ui: &mut egui::Ui, h: &Habitat, can_edit: bool) -> bool {
    ui.strong("Finite installed crop cycle");
    ui.label(
        egui::RichText::new(
            "Actual modelled transfers · assumed maintained chamber · no biological calibration",
        )
        .small()
        .color(ACCENT),
    );
    let Some(s) = &h.state.crop else {
        ui.label("No crop was installed in this habitat's original setup. Build or import a new explicit crop experiment.");
        return false;
    };
    if h.config.crop.is_none() {
        ui.label("Disabled. All material, source and history remain stored.");
    }
    let harvest = ui
        .horizontal_wrapped(|ui| {
            let clicked = ui
                .add_enabled(
                    can_edit && s.source_progress_days == 30. && !s.harvested,
                    egui::Button::new("Harvest completed cycle"),
                )
                .clicked();
            ui.label(
                egui::RichText::new("Separate dry stores; no diet credit or new cycle.")
                    .small()
                    .color(MUTED),
            );
            clicked
        })
        .inner;
    egui::Grid::new("installed-crop-readings")
        .num_columns(2)
        .striped(true)
        .spacing([20., 6.])
        .show(ui, |ui| {
            for (label, value) in [
                (
                    "Supported source progress",
                    format!("{:.6} / 30 days", s.source_progress_days),
                ),
                (
                    "Elapsed world time",
                    format!(
                        "{:.3} terrestrial days",
                        (h.last_time - s.origin_time_s) / 86400.
                    ),
                ),
                (
                    "Nutrient equivalent",
                    format!("{:.6} kg", s.nutrient_equivalent_kg),
                ),
                (
                    "Standing edible / inedible dry",
                    format!(
                        "{:.6} / {:.6} kg",
                        s.standing_dry_kg[0], s.standing_dry_kg[1]
                    ),
                ),
                (
                    "Harvested edible / inedible dry",
                    format!(
                        "{:.6} / {:.6} kg",
                        s.harvested_dry_kg[0], s.harvested_dry_kg[1]
                    ),
                ),
                (
                    "Captured CO₂ / potable water",
                    format!(
                        "{:.6} / {:.6} kg",
                        h.state.captured_co2_kg, h.state.potable_water_kg
                    ),
                ),
                (
                    "Actual crop CO₂ / metabolic water used",
                    format!(
                        "{:.6} / {:.6} kg",
                        s.co2_consumed_kg, s.metabolic_water_consumed_kg
                    ),
                ),
                (
                    "Actual crop vapor / O₂ generated",
                    format!(
                        "{:.6} / {:.6} kg",
                        s.vapor_generated_kg, s.oxygen_generated_kg
                    ),
                ),
                (
                    "Light / auxiliary electricity",
                    format!(
                        "{:.6} / {:.6} MJ",
                        s.light_electric_j / 1e6,
                        s.auxiliary_electric_j / 1e6
                    ),
                ),
                (
                    "Assumed dry chemical storage",
                    format!("{:.6} MJ", s.chemical_energy_j() / 1e6),
                ),
            ] {
                ui.label(label);
                ui.strong(value);
                ui.end_row();
            }
        });
    if let Some(b) = &h.budget.crop {
        ui.separator();
        ui.label(format!(
            "Completed {:.0} s interval · {}",
            h.budget.seconds, b.reason
        ));
        ui.label(format!(
            "Requested / served {:.2} / {:.2} W · water {:.9} kg · O₂ {:.9} kg",
            b.requested_w,
            b.delivered_w,
            b.water_consumed_kg(),
            b.oxygen_generated_kg
        ));
        ui.collapsing("Material and heat evidence",|ui| {ui.label(format!("Chemical / vapor storage {:.6} / {:.6} J",b.chemical_storage_j,b.vapor_phase_j));ui.label(format!("Mass residual {:.3e} kg · C/H/O/N residuals {:?} kg",b.mass_residual_kg,b.element_residual_kg));ui.label("Actual electricity enters the cabin bus; retained chemical and vapor energy are subtracted from sensible heat. WHY and recorded Cabin balances retain this accounting.");});
    }
    ui.label(egui::RichText::new("Progress can pause without a survival prediction. Harvest transfers dry material into separate stores; it does not feed the nominal crew diet or restart a cycle.").small().color(MUTED));
    harvest
}
pub fn controls(
    ui: &mut egui::Ui,
    c: &mut Option<crop_process::Config>,
    original: Option<&crop_process::Config>,
) {
    let Some(original) = original else {
        ui.label("This habitat has no originally installed crop. Use New crop setup; toggling cannot grant stock or a source cycle.");
        return;
    };
    let mut enabled = c.is_some();
    if ui
        .checkbox(&mut enabled, "Enable installed crop process")
        .changed()
    {
        if enabled {
            *c = Some(original.clone());
        } else {
            *c = None;
        }
    }
    if let Some(c) = c {
        ui.checkbox(
            &mut c.assume_maintained_conditions,
            "Assume source growing conditions are maintained",
        );
        ui.label("Ideal chamber boundary: source 23°C, humidity, pressure and CO₂ regime. This is your assumption; cabin temperature/RH do not prove chamber control.");
        number(ui, "Auxiliary power (W)", &mut c.auxiliary_power_w, 0., 1e7);
        number(
            ui,
            "Dry storage capacity (kg)",
            &mut c.biomass_capacity_kg,
            0.,
            1e7,
        );
        number(
            ui,
            "Shared O₂ reserve capacity (kg)",
            &mut c.oxygen_capacity_kg,
            0.,
            1e7,
        );
        ui.label(format!(
            "Pinned source: {:.3} m² · light {:.2} W · dry chemical basis {:.3} MJ/kg",
            c.crop.area_m2,
            c.light_power_w(),
            c.dry_chemical_energy_j_kg / 1e6
        ));
        ui.label("Changing the original crop source or chemical basis requires a new setup. Disabling retains stores/history.");
    }
}
pub fn model(ui: &mut egui::Ui) {
    ui.strong("Maintained reference process, with finite physical transfers");
    ui.label("One original 30-day low-light lettuce reference. Supported production progress advances during fully powered 16-hour light periods of each 24-hour terrestrial day. Missing inputs, storage or sufficient phase/chemical energy bound progress.");
    ui.label(crop_process::LIMITS);
    ui.hyperlink_to(
        "NASA BVAD · original MEC growth / transpiration",
        cosmogon_sim::greenhouse::SOURCE,
    );
    ui.hyperlink_to(
        "NASA lunar crop paper · rounded feed recipe",
        cosmogon_sim::greenhouse_supplies::SOURCE,
    );
    ui.hyperlink_to(
        "CIAAW · nominal CHON material basis",
        cosmogon_sim::crop_chemistry::SOURCE,
    );
    ui.label("Recordings contain crop.csv at full floating-point precision, complete original setup, saved worlds and strict source pins. The original planner stays separate and grants no actual material.");
}
