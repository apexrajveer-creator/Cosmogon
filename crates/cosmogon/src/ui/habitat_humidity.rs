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
            ui.label("Default vapor fraction is inferred from nominal latent heat. Ratings/setpoint are assumptions. Disable restores future crew output to legacy liquid collection and retains held water. Zero power/rate keeps vapor emission active.");
        });
    }
}
pub fn wall_controls(ui: &mut egui::Ui, equipment: &mut Option<moisture::WallConfig>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable passive wall condensation")
        .changed()
    {
        *equipment = enabled.then(moisture::WallConfig::default);
    }
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("Wall exchange assumptions").default_open(expand).show(ui,|ui| {
            super::habitat::number(ui,"Condensing wall area (m²)",&mut c.area_m2,0.0,1e5);
            let original=c.mass_transfer_kg_s_m2_pa*1e9;let mut k=original;
            super::habitat::number(ui,"Mass-transfer coefficient (×10⁻⁹ kg/(s·m²·Pa))",&mut k,0.0,1000.0);
            if k!=original {c.mass_transfer_kg_s_m2_pa=k*1e-9;}
            super::habitat::number(ui,"Wall thermal coupling to cabin (0 = exterior)",&mut c.wall_coupling,0.0,1.0);
            super::habitat::number(ui,"Wall liquid film capacity (kg)",&mut c.liquid_capacity_kg,0.0,1e7);
            let original=c.rate_kg_s*86_400.0;let mut rate=original;
            super::habitat::number(ui,"Maximum exchange (kg/day)",&mut rate,0.0,8_640_000.0);
            if rate!=original {c.rate_kg_s=rate/86_400.0;}
            ui.label("Signed lumped mass transfer toward saturation at the coupled wall temperature. Held film has no recycling path; no ice or drips. Outside 273.15–303 K wall temperature no exchange occurs.");
        });
    }
}
pub fn leak_controls(ui: &mut egui::Ui, equipment: &mut Option<cosmogon_sim::habitat::LeakConfig>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable nominal structural leak")
        .changed()
    {
        *equipment = enabled.then(cosmogon_sim::habitat::LeakConfig::default);
    }
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("Leak assumptions").default_open(expand).show(ui,|ui| {
            let original=c.rate_kg_s*86_400.0;let mut rate=original;
            super::habitat::number(ui,"Leak rate (kg/day)",&mut rate,0.0,8_640_000.0);
            if rate!=original {c.rate_kg_s=rate/86_400.0;}
            super::habitat::number(ui,"Assumed cabin-gas specific heat (J/kg)",&mut c.gas_specific_heat_j_kg,1.0,1e5);
            ui.label("Proportional species escape with enthalpy removal; independent of the breach orifice model. No seal degradation, inward or rarefied flow.");
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
            if h.uses_wall() {
                "Supersaturated state retained until passive wall exchange acts on it."
            } else {
                "Supersaturated state retained; wall/ice condensation is unmodeled."
            },
        );
    }
    if h.config.moisture.is_none() {
        ui.label("Moisture mechanism disabled: future crew output uses legacy liquid collection. Held vapor, condensate and phase energy remain.");
    }
    if let Some(b) = &h.budget.moisture {
        ui.label(format!("Last completed {:.3} s interval · end {:.3} s", h.budget.seconds, h.last_time));
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
    if h.uses_wall() {
        ui.strong("Passive wall condensation / evaporation");
        ui.label(format!(
            "Wall film {:.5} kg · cumulative net {:.5} kg · film capacity {:.2} kg",
            s.wall_liquid_kg,
            h.totals.wall_condensed_kg,
            h.config
                .wall_condensation
                .as_ref()
                .map_or(0.0, |w| w.liquid_capacity_kg)
        ));
        if let Some(b) = &h.budget.wall {
            ui.label(format!(
                "Last interval net {:.5} kg · phase heat {:.3} kJ · wall temperature {}",
                b.net_condensed_kg,
                b.condensation_heat_j / 1000.0,
                b.wall_temperature_k
                    .map(|v| format!("{v:.2} K"))
                    .unwrap_or("outside saturation domain".into())
            ));
            if !b.constraints.is_empty() {
                ui.colored_label(ACCENT, format!("Wall constraints: {}", b.constraints.join("; ")));
            }
        }
        ui.label("Lumped surface mass transfer toward saturation at the coupled wall temperature; held film has no recycling path. No ice, fogging or drips.");
    }
    if h.uses_leak() {
        ui.strong("Nominal structural leak");
        let leaked = h.totals.leaked_gas.unwrap_or_default();
        ui.label(format!(
            "Cumulative leaked {:.5} kg (O₂ {:.5} · N₂ {:.5} · CO₂ {:.5} · vapor {:.5})",
            leaked.mass_kg(),
            leaked.oxygen_kg,
            leaked.nitrogen_kg,
            leaked.co2_kg,
            leaked.vapor_kg
        ));
        if let Some(b) = &h.budget.leak {
            ui.label(format!(
                "Last interval {:.5} kg over {:.1} s · enthalpy removal {:.3} kJ",
                b.leaked.mass_kg(),
                h.budget.seconds,
                b.cooling_j / 1000.0
            ));
        }
        ui.label("Proportional species escape at an assumed cabin-gas specific heat; independent of the pressure-driven breach orifice model.");
    }
    egui::CollapsingHeader::new("Water phase, sources and limits").show(ui,|ui| {
        ui.label(format!("Retained reference vapor phase energy {:.3} MJ · emitted {:.5} kg · condensed {:.5} kg",s.phase_energy_j()/1e6,h.totals.crew_vapor_kg,h.totals.condensed_kg));
        ui.label("Reference phase energy 44.004 kJ/mol at 298.15 K. Crew phase storage is debited from immediate heat; actual condensation returns it. Electricity is already in bus heat.");
        if h.uses_wall() {
            ui.label(format!("Wall exchange model {} · signed net condensation moves the same reference phase heat into or out of cabin sensible heat.", moisture::WALL_MODEL));
        }
        ui.hyperlink_to("NIST saturation fit",moisture::SATURATION_SOURCE);
        ui.hyperlink_to("NIST gas thermochemistry",moisture::GAS_SOURCE);
        ui.label(egui::RichText::new("Start vapor/headroom only; raw condensate waits for later powered recycling. No potable-water quality, temperature-dependent COP, ice condensation, physical crop exchange or field calibration. Model & export retains actual numeric budgets and source constants.").small().color(MUTED));
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
    ui.label("Wall exchange: net = k · area · (pᵥ − pₛₐₜ(T_wall)) · dt with T_wall = T_ext + coupling · (T_cab − T_ext), bounded by vapor, film capacity and a rate cap; film liquid has no recycling path.");
    ui.label("Structural leak: leaked = rate · dt of cabin gas by species mass fraction with assumed constant cabin-gas specific heat enthalpy removal; no ice condensation or seal/inward/rarefied flow.");
    ui.label(egui::RichText::new("Numerical conservation does not establish predictive accuracy, hardware COP, water quality or safe cabin conditions. No ice condensation, humidity sensing, crop exchange, temperature-dependent phase energy or uncertainty propagation.").small().color(MUTED));
}
