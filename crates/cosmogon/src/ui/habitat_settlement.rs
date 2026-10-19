//! Native settlement pane: staged construction configuration plus conserved
//! same-body stock logistics. All state stays in the engine; controls only
//! stage ordinary edits.
use super::{ACCENT, MUTED};
use bevy_egui::egui;
use cosmogon_sim::{
    habitat::{Habitat, STOCK_TRANSFER_MODEL},
    habitat_construction::{Config, CONSTRUCTION_MODEL},
    supply_vehicle::Cargo,
};

/// Staged authoring inputs for construction supply and stock transfer.
#[derive(Clone, Debug, PartialEq)]
pub struct Logistics {
    pub supply_kg: f64,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub amounts: [f64; 4],
}
impl Default for Logistics {
    fn default() -> Self {
        Self {
            supply_kg: 100.0,
            from: None,
            to: None,
            amounts: [0.0; 4],
        }
    }
}

pub enum Action {
    Supply { requested_kg: f64 },
    Transfer { from: u64, to: u64, requested: Cargo },
}

const SPECIES: [&str; 4] = [
    "O₂ reserve (kg)",
    "Potable water (kg)",
    "Dry food (kg)",
    "Food water (kg)",
];

pub fn controls(ui: &mut egui::Ui, construction: &mut Option<Config>, expand: bool) {
    let mut enabled = construction.is_some();
    if ui
        .checkbox(&mut enabled, "Enable continuous cabin construction")
        .changed()
    {
        *construction = enabled.then(Config::default);
    }
    if let Some(c) = construction {
        egui::CollapsingHeader::new("Construction rates and stock")
            .default_open(expand)
            .show(ui, |ui| {
                super::habitat::number(
                    ui,
                    "Material capacity (kg)",
                    &mut c.material_capacity_kg,
                    1.0e-6,
                    1.0e7,
                );
                super::habitat::number(
                    ui,
                    "Build rate at full power (kg/s)",
                    &mut c.build_rate_kg_s,
                    1.0e-6,
                    1.0e4,
                );
                super::habitat::number(
                    ui,
                    "Construction bus power (W)",
                    &mut c.power_w,
                    0.0,
                    1.0e6,
                );
                super::habitat::number(
                    ui,
                    "Volume built per kilogram (m³/kg)",
                    &mut c.volume_per_kg_m3,
                    1.0e-9,
                    1.0,
                );
                ui.label("Construction runs at the lowest power priority, consumes only held material, and never grows the cabin past its authored 1e6 m³ volume bound. Disabling freezes held material without discarding it.");
            });
    }
}

pub fn readings(
    ui: &mut egui::Ui,
    h: &Habitat,
    habitats: &[Habitat],
    can_edit: bool,
    logistics: &mut Logistics,
) -> Option<Action> {
    ui.strong("Finite settlement construction and logistics");
    ui.label(
        egui::RichText::new("Held material · authored build rates · conserved same-body stock transfer")
            .color(ACCENT),
    );
    let mut action = None;
    if h.uses_construction() {
        ui.label(format!(
            "Held material {:.3} / {:.3} kg · structure built {:.3} kg · cumulative volume +{:.6} m³ · live cabin {:.6} m³",
            h.state.construction_material_kg,
            h.config
                .construction
                .as_ref()
                .or(h.definition.config.construction.as_ref())
                .map(|c| c.material_capacity_kg)
                .unwrap_or(0.0),
            h.state.construction_structure_kg,
            h.totals.constructed_volume_m3,
            h.config.volume_m3,
        ));
        if let Some(b) = &h.budget.construction {
            egui::Grid::new("construction-interval")
                .striped(true)
                .show(ui, |ui| {
                    for label in [
                        "Interval",
                        "Material before → after",
                        "Consumed",
                        "Volume added",
                        "Power requested / delivered",
                        "Served",
                    ] {
                        ui.label(egui::RichText::new(label).strong());
                    }
                    ui.end_row();
                    ui.label(format!("{:.0} s", h.budget.seconds));
                    ui.label(format!(
                        "{:.3} → {:.3} kg",
                        b.material_before_kg, b.material_after_kg
                    ));
                    ui.label(format!("{:.3} kg", b.consumed_kg));
                    ui.label(format!("{:.6} m³", b.volume_added_m3));
                    ui.label(format!("{:.1} / {:.1} W", b.requested_w, b.delivered_w));
                    ui.label(format!("{:.1}%", 100.0 * b.served_fraction));
                    ui.end_row();
                });
            if !b.constraints.is_empty() {
                ui.label(format!("Constraints: {}.", b.constraints.join("; ")));
            }
        } else {
            ui.label("No completed construction interval yet; consumption observations unavailable.");
        }
    } else {
        ui.label("Cabin construction is not enabled. Enable it in Settings, then supply finite material below; nothing is granted without an explicit charge.");
    }
    if can_edit {
        ui.separator();
        ui.strong("Author explicit material supply");
        super::habitat::number(
            ui,
            "Supply charge (kg)",
            &mut logistics.supply_kg,
            0.0,
            1.0e7,
        );
        if ui.button("Supply construction material").clicked() {
            action = Some(Action::Supply {
                requested_kg: logistics.supply_kg,
            });
        }
        ui.separator();
        ui.strong("Transfer supply stocks between habitats (same body)");
        egui::ComboBox::from_id_salt("stock-transfer-from")
            .selected_text(
                logistics
                    .from
                    .map(|id| format!("#{id}"))
                    .unwrap_or_else(|| "Source habitat".into()),
            )
            .show_ui(ui, |ui| {
                for h2 in habitats {
                    ui.selectable_value(&mut logistics.from, Some(h2.id), format!("#{} · {}", h2.id, h2.definition.name));
                }
            });
        egui::ComboBox::from_id_salt("stock-transfer-to")
            .selected_text(
                logistics
                    .to
                    .map(|id| format!("#{id}"))
                    .unwrap_or_else(|| "Recipient habitat".into()),
            )
            .show_ui(ui, |ui| {
                for h2 in habitats {
                    ui.selectable_value(&mut logistics.to, Some(h2.id), format!("#{} · {}", h2.id, h2.definition.name));
                }
            });
        for (i, label) in SPECIES.iter().enumerate() {
            super::habitat::number(ui, label, &mut logistics.amounts[i], 0.0, 1.0e7);
        }
        let total: f64 = logistics.amounts.iter().sum();
        if ui
            .add_enabled(
                logistics.from.is_some() && logistics.to.is_some() && total > 0.0,
                egui::Button::new("Transfer stocks"),
            )
            .clicked()
        {
            if let (Some(from), Some(to)) = (logistics.from, logistics.to) {
                action = Some(Action::Transfer {
                    from,
                    to,
                    requested: Cargo {
                        oxygen_reserve_kg: logistics.amounts[0],
                        potable_water_kg: logistics.amounts[1],
                        food_dry_kg: logistics.amounts[2],
                        food_water_kg: logistics.amounts[3],
                    },
                });
            }
        }
        ui.label(egui::RichText::new("Supply grants only its declared bounded charge. Transfer moves only held stock the sender has and the recipient can hold; a transfer that accepts nothing is refused.").small().color(MUTED));
    }
    if !h.stock_transfers.is_empty() {
        egui::CollapsingHeader::new(format!("Retained stock transfer records ({})", h.stock_transfers.len()))
            .show(ui, |ui| {
                for r in h.stock_transfers.iter().rev().take(12) {
                    ui.label(format!(
                        "{} at {:.3} s · #{} · O₂ {:.3} · water {:.3} · food {:.3} · food water {:.3} kg",
                        if r.outbound { "Outbound" } else { "Inbound" },
                        r.world_time_s,
                        r.counterparty,
                        r.moved.oxygen_reserve_kg,
                        r.moved.potable_water_kg,
                        r.moved.food_dry_kg,
                        r.moved.food_water_kg,
                    ));
                }
                if h.stock_transfers.len() > 12 {
                    ui.label(format!("{} earlier records retained in the save.", h.stock_transfers.len() - 12));
                }
            });
    }
    action
}

pub fn model(ui: &mut egui::Ui) {
    ui.strong("Settlement construction and logistics · source equations and explicit assumptions");
    ui.label(format!(
        "consumed = min(held material, build_rate · dt · served); volume += consumed · volume_per_kg; model {CONSTRUCTION_MODEL}."
    ));
    ui.label(format!(
        "Transfer: per species accepted = min(requested, sender held, recipient free); sender decrease equals receiver increase; retained records pair across both habitats; model {STOCK_TRANSFER_MODEL}."
    ));
    ui.label("Construction power is an ordinary lowest-priority bus slot and becomes cabin heat through the existing ledger. Consumed material becomes structure mass at unchanged total mass.");
    ui.label(egui::RichText::new("Construction rates, bulk ratio and power are authored assumptions. No structural strength, thermal-mass growth, dust, EVA, crew labour, assembly physics, roads, vehicles, warehouses or markets are modelled. Numerical conservation does not establish predictive accuracy.").small().color(MUTED));
}
