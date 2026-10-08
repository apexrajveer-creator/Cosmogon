//! Native controls observe the shared finite phase model and stage ordinary edits.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{habitat::Habitat, moisture};

pub fn controls(ui: &mut egui::Ui, equipment: &mut Option<moisture::Config>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable finite cabin moisture")
        .changed()
    {
        *equipment = enabled.then(moisture::Config::default);
    }
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("Moisture and equipment assumptions").default_open(expand).show(ui,|ui| {
            super::habitat::number(ui,"Airborne fraction of reference moisture",&mut c.airborne_fraction,0.0,1.0);
            super::habitat::number(ui,"Relative humidity setpoint (fraction)",&mut c.relative_humidity_setpoint,0.0,1.0);
            super::habitat::number(ui,"Maximum electric power (W)",&mut c.power_limit_w,0.0,1e6);
            let original=c.rate_kg_s*86_400.0;let mut rate=original;
            super::habitat::number(ui,"Maximum condensate collection (kg/day)",&mut rate,0.0,8_640_000.0);
            if rate!=original {c.rate_kg_s=rate/86_400.0;}
            let original=c.electricity_j_kg/1e6;let mut energy=original;
            super::habitat::number(ui,"Process electricity (MJ/kg)",&mut energy,0.000001,1000.0);
            if energy!=original {c.electricity_j_kg=energy*1e6;}
            super::habitat::number(ui,"Isolated condensate capacity (kg)",&mut c.condensate_capacity_kg,0.0,1e7);
            ui.label("Default vapor fraction is inferred from nominal latent heat. Ratings/setpoint are assumptions. Disable retains all water; held liquid can still enter the powered recycler.");
        });
    }
}
pub fn readings(ui: &mut egui::Ui, h: &Habitat) {
    ui.strong("Finite cabin vapor → isolated condensate");
    ui.label(
        egui::RichText::new("NIST water data · assumed moisture partition and equipment")
            .color(ACCENT),
    );
    if !h.uses_moisture() {
        ui.label("Moisture tracking is not enabled; legacy immediate collection remains. No humidity observation is available.");
        return;
    }
    let s = &h.state;
    let rh = s.relative_humidity(&h.config);
    ui.label(format!(
        "Vapor {:.5} kg · raw condensate {:.5} kg · {:.2} K",
        s.water_vapor_kg, s.condensate_kg, s.temperature_k
    ));
    ui.label(format!(
        "Vapor pressure {:.2} Pa · relative humidity {}",
        s.vapor_pressure_pa(&h.config),
        rh.map(|v| format!("{:.2}%", 100.0 * v))
            .unwrap_or("unavailable outside 273.15–303 K".into())
    ));
    if rh.is_some_and(|v| v > 1.0) {
        ui.colored_label(
            ACCENT,
            "Supersaturated state retained; wall/ice condensation is unmodeled.",
        );
    }
    if h.config.moisture.is_none() {
        ui.label("Equipment disabled; vapor, condensate and phase energy retained.");
    }
    if let Some(b) = &h.budget.moisture {
        egui::Grid::new("moisture-readings")
            .striped(true)
            .show(ui, |ui| {
                for (label, value) in [
                    (
                        "Interval electric request / served",
                        format!("{:.2} / {:.2} W", b.requested_w, b.delivered_w),
                    ),
                    (
                        "Crew vapor / collected liquid",
                        format!("{:.5} / {:.5} kg", b.crew_vapor_kg, b.condensed_kg),
                    ),
                    (
                        "Crew phase debit / condensation heat",
                        format!(
                            "{:.3} / {:.3} kJ",
                            b.crew_phase_energy_j / 1000.0,
                            b.condensation_heat_j / 1000.0
                        ),
                    ),
                    (
                        "Held liquid processed / recovered",
                        format!(
                            "{:.5} / {:.5} kg",
                            h.budget.condensate_processed_kg, h.budget.condensate_recovered_kg
                        ),
                    ),
                ] {
                    ui.label(label);
                    ui.strong(value);
                    ui.end_row();
                }
            });
        if !b.constraints.is_empty() {
            ui.colored_label(ACCENT, format!("Constraints: {}", b.constraints.join("; ")));
        }
    } else {
        ui.label("No completed active moisture interval; flow observations unavailable.");
    }
    egui::CollapsingHeader::new("Water phase, sources and limits").show(ui,|ui| {
        ui.label(format!("Retained reference vapor phase energy {:.3} MJ · emitted {:.5} kg · condensed {:.5} kg",s.phase_energy_j()/1e6,h.totals.crew_vapor_kg,h.totals.condensed_kg));
        ui.label("Reference phase energy 44.004 kJ/mol at 298.15 K. Crew phase storage is debited from immediate heat; actual condensation returns it. Electricity is already in bus heat.");
        ui.hyperlink_to("NIST saturation fit",moisture::SATURATION_SOURCE);
        ui.hyperlink_to("NIST gas thermochemistry",moisture::GAS_SOURCE);
        ui.label(egui::RichText::new("Start vapor/headroom only; raw condensate waits for later powered recycling. No potable-water quality, temperature-dependent COP, wall/ice condensation, physical crop exchange or field calibration. Model & export retains actual numeric budgets and source constants.").small().color(MUTED));
    });
}

pub fn model(ui: &mut egui::Ui) {
    ui.strong("Finite moisture · source equations and explicit assumptions");
    ui.label("pᵥ = Mᵥ R T / (molar mass × cabin volume); RH = pᵥ / pₛₐₜ.");
    ui.label("NIST: log₁₀(P bar) = 5.40221 − 1838.675 / (T K − 31.737).");
    ui.label("Liquid-water fit used only 273.15–303 K. Outside this domain, RH and condensation control are unavailable; stored vapor remains.");
    ui.label(format!(
        "Reference phase energy {:.6} MJ/kg at 298.15 K",
        moisture::REFERENCE_PHASE_J_KG / 1e6
    ));
    ui.label(format!(
        "Default airborne fraction {:.6} = 6.118 MJ / (2.946 kg × reference phase energy)",
        moisture::REFERENCE_AIRBORNE_FRACTION
    ));
    ui.label("This fraction is an inference from nominal crew heat, not a measured vapor partition. Changing it changes modeled sensible heat and vapor together.");
    ui.label("Condensation uses start vapor above setpoint, start liquid headroom, rate and allocated electricity. Crew emissions and freed liquid storage become usable next interval. Condensate is isolated before later powered recycling.");
    ui.hyperlink_to(
        "NASA nominal crew interfaces",
        cosmogon_sim::habitat::SOURCE,
    );
    ui.hyperlink_to("NIST saturation coefficients", moisture::SATURATION_SOURCE);
    ui.hyperlink_to("NIST gas formation enthalpy", moisture::GAS_SOURCE);
    ui.hyperlink_to(
        "NIST liquid formation enthalpy",
        cosmogon_sim::electrolysis::ENTHALPY_SOURCE,
    );
    ui.label(egui::RichText::new("Numerical conservation does not establish predictive accuracy, hardware COP, water quality or safe cabin conditions. No wall/ice condensation, humidity sensing, crop exchange, temperature-dependent phase energy or uncertainty propagation.").small().color(MUTED));
}
