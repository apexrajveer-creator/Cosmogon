//! Ordinary staged controls and actual finite outward-gas observations.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{breach, habitat::Habitat};
pub fn controls(ui: &mut egui::Ui, equipment: &mut Option<breach::Config>, expand: bool) {
    let mut enabled = equipment.is_some();
    if ui
        .checkbox(&mut enabled, "Enable outward cabin breach")
        .changed()
    {
        *equipment = enabled.then(breach::Config::default);
    }
    if let Some(c) = equipment {
        egui::CollapsingHeader::new("Breach geometry and gas assumptions").default_open(expand).show(ui,|ui|{
  let original=c.area_m2*1e6;let mut area=original;super::habitat::number(ui,"Open throat area (mm²)",&mut area,0.0,1e6);if area!=original {c.area_m2=area/1e6;}
  super::habitat::number(ui,"Assumed discharge coefficient",&mut c.discharge_coefficient,0.0,1.0);
  super::habitat::number(ui,"Assumed constant gas gamma",&mut c.gamma,1.05,1.67);
  super::habitat::number(ui,"Assumed outside receiver pressure (Pa)",&mut c.outside_pressure_pa,0.0,2e6);
  super::habitat::number(ui,"Supported cabin pressure floor (Pa)",&mut c.minimum_pressure_pa,100.0,2e6);
  ui.label("Zero area/coefficient closes flow. Disable preserves escaped history. Receiver is a fixed design assumption; no measured site pressure, inward replenishment or seal leakage is inferred.");
 });
    }
}
pub fn readings(ui: &mut egui::Ui, h: &Habitat) {
    ui.strong("Cabin gas → outside receiver");
    ui.colored_label(
        ACCENT,
        "Open-orifice model · geometry and receiver are assumptions",
    );
    if !h.uses_breach() {
        ui.label("Breach flow is disabled; existing finite cabin stores remain sealed in this model. Enable it in Settings to study an outward open hole.");
        return;
    }
    let g = h.state.escaped_gas.unwrap_or_default();
    ui.label(format!(
        "Now: total {:.3} kPa · O₂ {:.3} kPa · {:.3} K",
        h.state.pressure_pa(&h.config) / 1000.0,
        h.state.oxygen_pa(&h.config) / 1000.0,
        h.state.temperature_k
    ));
    ui.label(format!(
        "Outside cabin: {:.6} kg total · O₂ {:.6} · N₂ {:.6} · CO₂ {:.6} · vapor {:.6} kg",
        g.mass_kg(),
        g.oxygen_kg,
        g.nitrogen_kg,
        g.co2_kg,
        g.vapor_kg
    ));
    if let Some(b) = &h.budget.breach {
        ui.colored_label(ACCENT, b.status.label());
        ui.label(format!(
            "Completed {:.3} s interval · resolved flow {:.6} s · {} steps",
            h.budget.seconds, b.integrated_seconds, b.integration_steps
        ));
        egui::Grid::new("breach-interval")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (label, value) in [
                    (
                        "Substep pressure",
                        format!(
                            "{:.3} → {:.3} kPa",
                            b.initial_pressure_pa / 1000.0,
                            b.final_pressure_pa / 1000.0
                        ),
                    ),
                    (
                        "Substep temperature",
                        format!(
                            "{:.6} → {:.6} K",
                            b.initial_temperature_k, b.final_temperature_k
                        ),
                    ),
                    (
                        "Initial / final outward rate",
                        format!(
                            "{:.6} / {:.6} g/s",
                            b.initial_rate_kg_s * 1000.0,
                            b.final_rate_kg_s * 1000.0
                        ),
                    ),
                    (
                        "Resolved interval escaped gas",
                        format!("{:.6} kg", b.escaped.mass_kg()),
                    ),
                    (
                        "Equivalent lumped cooling",
                        format!("{:.6} kJ", b.lumped_cooling_j / 1000.0),
                    ),
                    (
                        "Full outward enthalpy",
                        format!("{:.6} kJ", b.outflow_enthalpy_j / 1000.0),
                    ),
                    (
                        "Breach energy / accounted mass residual",
                        format!(
                            "{:+.3e} J / {:+.3e} kg",
                            b.energy_residual_j, h.budget.mass_residual_kg
                        ),
                    ),
                ] {
                    ui.label(label);
                    ui.strong(value);
                    ui.end_row();
                }
            });
        if !b.status.supported() {
            ui.label("Remaining flow is unsupported; partial transfers stay recorded. Full-interval flow/cooling graphs are unavailable for this interval.");
        }
        ui.label(format!(
            "Initial flow {}. Completed settings are retained independently of staged controls.",
            if b.initial_choked {
                "choked"
            } else {
                "subcritical or inactive"
            }
        ));
    } else {
        ui.label("No completed active breach interval. Disabling keeps previously escaped gas outside the cabin.");
    }
    ui.label(egui::RichText::new("Crew/process/thermal minute runs before the breach substep. Equivalent cooling is not full gas enthalpy. Numerical identities do not establish field accuracy. WHY and complete study exports retain components, source pins and model limits.").small().color(MUTED));
}
pub fn model(ui: &mut egui::Ui) {
    ui.strong("Pressure-driven finite outward breach");
    ui.label("p = n R T / V; Rmix = R n / gas mass. Flow becomes choked below (2/(γ+1))^(γ/(γ−1)); above that ratio the subcritical law follows receiver/cabin pressure.");
    ui.label("O₂, N₂, CO₂ and vapor leave in actual well-mixed proportions. Reserve O₂, isolated H₂/methane and liquids remain. Escaped mass belongs to an outside ledger, unavailable for cabin reuse.");
    ui.label("Thermal substep: cv = R/(γ−1); structure Cs = configured C − n₀cv; T(n) = T₀[(Cs+ncv)/C]^(γ−1). Negative Cs is unsupported. Structure + gas internal energy + reference vapor phase + outward enthalpy sum to the reported residual.");
    ui.label("Existing minute heat/process budget precedes breach flow. Its cooling debit C(T₀−T₁) is temperature-equivalent, not full outflow enthalpy. Constant properties and operator splitting limit rapid-transient fidelity.");
    ui.label("Bounded deterministic step-doubled RK4: at most 1% gas moles per attempted step, 1024 attempts; local mole disagreement ≤10⁻¹¹ of initial moles. Boundary bisection stops at equilibrium, chosen pressure floor or 100 K. Actual partial time is retained; unknown flow is unavailable, not replenished.");
    ui.hyperlink_to("NASA Glenn choking equations", breach::SOURCE);
    ui.hyperlink_to("NASA GUNNS orifice theory", breach::THEORY_SOURCE);
    ui.hyperlink_to("NASA leak-path regime limits", breach::LIMITS_SOURCE);
    ui.label(egui::RichText::new("Continuum is assumed. No seal/permeation/rarefied calibration, inward composition, jet/structural forces, wall condensation, variable receiver, EVA, equipment validation or physiological predictions. Input/predictive uncertainty is unquantified.").small().color(MUTED));
}
