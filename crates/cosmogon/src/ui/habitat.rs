//! Engineering controls issue ordinary commands; all budget state belongs to the engine.
use super::{UiState, ACCENT, DANGER, MUTED};
use crate::sim::{Sim, Target};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use cosmogon_sim::{
    habitat::{self, Config, Definition},
    sandbox::Edit,
};

#[derive(Resource)]
pub struct HabitatUi {
    selected: Option<u64>,
    key: Option<(u64, u64)>,
    baseline: Option<Config>,
    draft: Config,
    name: String,
    latitude: f64,
    longitude: f64,
    import_path: String,
    export_path: String,
    status: Option<String>,
    tab: usize,
    pub crop: super::greenhouse::CropPlanner,
}
impl Default for HabitatUi {
    fn default() -> Self {
        Self {
            selected: None,
            key: None,
            baseline: None,
            draft: Config::default(),
            name: "Surface habitat experiment".into(),
            latitude: 0.0,
            longitude: 0.0,
            import_path: String::new(),
            export_path: String::new(),
            status: None,
            tab: 0,
            crop: Default::default(),
        }
    }
}
pub(super) fn number(ui: &mut egui::Ui, label: &str, value: &mut f64, min: f64, max: f64) {
    let speed = (value.abs() * 0.01).max(0.001);
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(value)
                .range(min..=max)
                .speed(speed)
                .max_decimals(6),
        );
    });
}
fn config(ui: &mut egui::Ui, c: &mut Config) {
    egui::CollapsingHeader::new("Water electrolysis").default_open(c.electrolysis.is_some()).show(ui, |ui|super::habitat_oxygen::controls(ui,&mut c.electrolysis,true));
    egui::CollapsingHeader::new("Crew and cabin")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Nominal crew demand");
                ui.add(egui::DragValue::new(&mut c.crew).range(0..=1000));
            });
            number(ui, "Cabin volume (m³)", &mut c.volume_m3, 1.0, 1.0e6);
            number(
                ui,
                "Oxygen regulator setpoint (Pa)",
                &mut c.oxygen_setpoint_pa,
                0.0,
                200_000.0,
            );
            number(
                ui,
                "Stored oxygen makeup limit (kg/s)",
                &mut c.oxygen_makeup_rate_kg_s,
                0.0,
                100.0,
            );
        });
    egui::CollapsingHeader::new("Power and battery").show(ui, |ui| {
        number(
            ui,
            "External firm supply (W)",
            &mut c.firm_power_w,
            0.0,
            1.0e6,
        );
        number(
            ui,
            "Horizontal panel area (m²)",
            &mut c.solar_area_m2,
            0.0,
            100_000.0,
        );
        number(
            ui,
            "PV efficiency (fraction)",
            &mut c.solar_efficiency,
            0.0,
            1.0,
        );
        number(
            ui,
            "Atmospheric transmission (assumed)",
            &mut c.atmospheric_transmission,
            0.0,
            1.0,
        );
        number(
            ui,
            "Battery capacity (J)",
            &mut c.battery_capacity_j,
            0.0,
            1.0e12,
        );
        number(ui, "Charge limit (W)", &mut c.charge_limit_w, 0.0, 1.0e6);
        number(
            ui,
            "Discharge limit (W)",
            &mut c.discharge_limit_w,
            0.0,
            1.0e6,
        );
        number(
            ui,
            "Charge efficiency (fraction)",
            &mut c.charge_efficiency,
            0.01,
            1.0,
        );
        number(
            ui,
            "Discharge efficiency (fraction)",
            &mut c.discharge_efficiency,
            0.01,
            1.0,
        );
        number(ui, "Base services load (W)", &mut c.base_load_w, 0.0, 1.0e6);
    });
    egui::CollapsingHeader::new("Air and water processors").show(ui,|ui| {
        number(ui,"Scrubber load (W)",&mut c.scrubber_power_w,0.0,1.0e6);number(ui,"CO₂ capture rate (kg/s)",&mut c.scrubber_rate_kg_s,0.0,100.0);number(ui,"Finite CO₂ capture store (kg)",&mut c.capture_capacity_kg,0.0,1.0e7);
        number(ui,"Recycler load (W)",&mut c.recycler_power_w,0.0,1.0e6);number(ui,"Wastewater processing (kg/s)",&mut c.recycler_rate_kg_s,0.0,100.0);number(ui,"Recovered water fraction (assumed)",&mut c.water_recovery_fraction,0.0,1.0);
        ui.label(egui::RichText::new("Zero equipment power disables that processor. Recovery fraction does not establish contaminant removal or potable-water quality.").small().color(MUTED));
    });
    egui::CollapsingHeader::new("Thermal controls").show(ui, |ui| {
        number(
            ui,
            "Target temperature (K)",
            &mut c.temperature_setpoint_k,
            100.0,
            500.0,
        );
        number(
            ui,
            "Heat capacity (J/K)",
            &mut c.heat_capacity_j_k,
            1.0e6,
            1.0e12,
        );
        number(
            ui,
            "Envelope conductance (W/K)",
            &mut c.conductance_w_k,
            0.0,
            1000.0,
        );
        number(ui, "Heater limit (W)", &mut c.heater_limit_w, 0.0, 1.0e6);
        number(ui, "Radiator pump (W)", &mut c.radiator_pump_w, 0.0, 1.0e6);
        number(
            ui,
            "Radiator area (m²)",
            &mut c.radiator_area_m2,
            0.0,
            100_000.0,
        );
        number(
            ui,
            "Radiator emissivity (fraction)",
            &mut c.radiator_emissivity,
            0.0,
            1.0,
        );
    });
    egui::CollapsingHeader::new("Design alert thresholds").show(ui,|ui| {
        number(ui,"Minimum O₂ (Pa)",&mut c.oxygen_alert_pa,0.0,200_000.0);number(ui,"Maximum CO₂ (Pa)",&mut c.co2_alert_pa,0.0,200_000.0);number(ui,"Cold threshold (K)",&mut c.cold_alert_k,100.0,500.0);number(ui,"Hot threshold (K)",&mut c.hot_alert_k,100.0,500.0);
        ui.label("Experimental design bands; not medical limits. Sound & Alerts configures existing Disaster/Space notification routing.");
    });
}
#[allow(clippy::too_many_arguments)]
pub fn panel(
    mut contexts: EguiContexts,
    mut state: ResMut<UiState>,
    mut form: ResMut<HabitatUi>,
    mut sim: ResMut<Sim>,
    time: Res<Time>,
) -> Result {
    if let Some(id) = state.habitat_requested.take() {
        form.selected = Some(id);
        state.habitat_open = true;
    }
    if !state.habitat_open || state.hidden || state.photo_mode {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let mut open = state.habitat_open;
    let form = &mut *form;
    if form
        .selected
        .is_none_or(|id| !sim.universe.habitats.iter().any(|h| h.id == id))
    {
        form.selected = sim.universe.habitats.first().map(|h| h.id);
        form.key = None;
    }
    let mut edit = None;
    let mut export = None;
    let mut clock = None;
    let max_height = (ctx.available_rect().height() - 160.0).max(240.0);
    // The planner needs room for its axes and selected-day values on first opening.
    // Keep the window resizable; the scroll area handles smaller screens.
    egui::Window::new("Habitat engineering").open(&mut open).default_pos([345.0,82.0]).default_width(540.0).default_height(if form.tab==4 {540.0} else {480.0}).max_height(max_height).show(ctx,|ui| {
      egui::ScrollArea::vertical().max_height(max_height).show(ui,|ui| {
        if form.tab!=4 {ui.label(egui::RichText::new("Experimental mass, power and heat budgets").strong());
        ui.label(egui::RichText::new("NASA reference crew rates · user-assumed equipment · unvalidated field accuracy").small().color(ACCENT));}
        if sim.is_reference() {ui.colored_label(ACCENT,"Clone this reference world to a sandbox before building or configuring habitats.");}
        if form.tab!=4 || !sim.universe.habitats.is_empty() {egui::ComboBox::from_id_salt("habitat-selection").selected_text(form.selected.and_then(|id|sim.universe.habitats.iter().find(|h|h.id==id)).map(|h|h.definition.name.as_str()).unwrap_or("No habitats in this world")).show_ui(ui,|ui| {
            for h in &sim.universe.habitats {ui.selectable_value(&mut form.selected,Some(h.id),format!("{} · #{}",h.definition.name,h.id));}
        });}
        ui.horizontal_wrapped(|ui| {for (i,label) in ["Budgets","Controls","Build & import","Model & export","Crop planner","Oxygen"].iter().enumerate() {ui.selectable_value(&mut form.tab,i,*label);}});
        if let Some(status)=&form.status {ui.colored_label(ACCENT,status);}
        if form.tab==4 {
            super::greenhouse::panel(ui,&mut form.crop,&sim.universe,form.selected);
        } else if form.tab==2 || sim.universe.habitats.is_empty() {
            ui.label("Create a new experiment on the selected solid world. Initial stocks are finite; this does not construct a measured mission habitat.");
            let body=match sim.selected {Some(Target::Body(r))=>Some(r),_=>None};
            if let Some(r)=body {let b=sim.universe.body(r);ui.strong(format!("World: {}",b.name));
                ui.text_edit_singleline(&mut form.name);number(ui,"Latitude north (°)",&mut form.latitude,-90.0,90.0);number(ui,"Longitude east (°)",&mut form.longitude,0.0,360.0);
                ui.label("Preset: 4 reference people, 500 m³, 100 kg stored O₂, 200 kg water, 100 kg dry / 95 kg food water, 20 kWh battery half full. Equipment ratings are assumptions; edit them in Controls or import an advanced definition.");
                if ui.add_enabled(!sim.is_reference() && b.exists() && b.kind.has_surface(),egui::Button::new("Build habitat experiment")).clicked() {let mut d=Definition::at(r,form.latitude,form.longitude);d.name=form.name.clone();edit=Some(Edit::CreateHabitat {definition:Box::new(d)});}
            } else {ui.label("Select a planet or moon with a solid surface to build.");}
            ui.separator();ui.label("Import a habitat definition (model, body ID, coordinates, configuration and initial stocks). Failed imports preserve the world.");ui.text_edit_singleline(&mut form.import_path);
            if ui.add_enabled(!sim.is_reference(),egui::Button::new("Build from definition")).clicked() {match Definition::read(std::path::Path::new(&form.import_path)) {Ok(d)=>edit=Some(Edit::CreateHabitat {definition:Box::new(d)}),Err(e)=>form.status=Some(e)}}
        } else if let Some(h)=form.selected.and_then(|id|sim.universe.habitats.iter().find(|h|h.id==id)) {
            let c=&h.config;let s=&h.state;let b=&h.budget;
            ui.label(format!("#{} · world [{},{}] · {:.5}° N, {:.5}° E",h.id,h.definition.body.system,h.definition.body.body,h.definition.latitude_deg,h.definition.longitude_east_deg));
            match form.tab {
              0=>{
                ui.label(format!("Fixed minute model · last interval {:.0} s · state {:.1} s behind world clock",b.seconds,sim.universe.time-h.last_time));
                ui.horizontal(|ui| {if ui.button("Run at 1 min/s").clicked() {clock=Some(true);}
                    if ui.button("Pause clock").clicked() {clock=Some(false);}});
                if b.seconds==0.0 {ui.label("Waiting for the first completed minute. Change clock speed to see longer engineering timescales.");}
                if !b.temperature_domain_supported && b.environment_available && b.seconds>0.0 {ui.colored_label(DANGER,"Temperature outside the model’s 100–500 K computation domain; rebuild with viable inputs. Suspended budgets are not predictions.");}
                if !b.environment_available {ui.colored_label(DANGER,"Exterior environment unavailable. Budgets are suspended; no local weather or zero flux is fabricated.");}
                for (i,message) in habitat::ALERTS.iter().enumerate() {if h.alert_bits&(1<<i)!=0 {ui.colored_label(DANGER,*message);}}
                egui::Grid::new("habitat-budgets").num_columns(2).spacing([24.0,7.0]).striped(true).show(ui,|ui| {
                    for (label,value) in [
                        ("Cabin temperature",format!("{:.2} K",s.temperature_k)),("Total pressure",format!("{:.2} kPa",s.pressure_pa(c)/1000.0)),("Oxygen partial pressure",format!("{:.2} kPa",s.oxygen_pa(c)/1000.0)),("CO₂ partial pressure",format!("{:.2} Pa",s.co2_pa(c))),
                        ("Battery",format!("{:.3} / {:.3} kWh",s.battery_j/3.6e6,c.battery_capacity_j/3.6e6)),("Power generated / served",format!("{:.1} / {:.1} W",b.generated_w,b.delivered_w)),("Unserved electrical demand",format!("{:.1} W",b.unserved_w)),("Stored oxygen",format!("{:.3} kg",s.oxygen_reserve_kg)),("Potable / wastewater",format!("{:.3} / {:.3} kg",s.potable_water_kg,s.wastewater_kg)),("Dry food / food water",format!("{:.3} / {:.3} kg",s.food_dry_kg,s.food_water_kg)),("Captured CO₂",format!("{:.3} / {:.3} kg",s.captured_co2_kg,c.capture_capacity_kg)),("Unserved nominal crew demand",format!("{:.3} person-hours",h.totals.unserved_person_seconds/3600.0))] {ui.label(label);ui.strong(value);ui.end_row();}
                });
                egui::CollapsingHeader::new("Interval flows and numerical residuals").show(ui,|ui| {
                    ui.label(format!("O₂ makeup {:.6} kg · CO₂ captured {:.6} kg · water processed/recovered {:.6}/{:.6} kg",b.oxygen_makeup_kg,b.co2_captured_kg,b.water_processed_kg,b.water_recovered_kg));
                    ui.label(format!("Heat: crew {:.1} W · heater {:.1} W (included in bus load) · radiator −{:.1} W · envelope {:+.1} W · storage {:+.1} W",b.metabolic_heat_w,b.heater_w,b.radiator_rejection_w,b.conduction_w,b.thermal_storage_w));
                    ui.label(format!("Mass residual {:.3e} kg · electric residual {:.3e} J · thermal residual {:.3e} J",b.mass_residual_kg,b.electric_residual_j,b.thermal_residual_j));
                    ui.label(format!("Brine {:.3} kg · dry waste {:.3} kg · supplied reference cycle {:.2}%",s.brine_kg,s.dry_waste_kg,100.0*b.served_cycle_fraction));
                    ui.label("Residuals check bookkeeping; they do not establish real-world predictive accuracy.");
                });
              }
              1|5=>{
                if form.key!=Some((sim.generation,h.id)) || form.baseline.as_ref()!=Some(c) {form.key=Some((sim.generation,h.id));form.draft=c.clone();form.baseline=Some(c.clone());}
                ui.label("Changes are staged until Apply. Stored mass/energy stay intact; reducing capacity below stored quantity is rejected.");
                if form.tab==5 {super::habitat_oxygen::readings(ui,h);}
                ui.add_enabled_ui(!sim.is_reference(),|ui| {if form.tab==5 {super::habitat_oxygen::controls(ui,&mut form.draft.electrolysis,false);} else {config(ui,&mut form.draft);}});
                ui.horizontal(|ui| {
                    if ui.add_enabled(!sim.is_reference() && form.draft!=*c,egui::Button::new("Apply configuration")).clicked() {edit=Some(Edit::ConfigureHabitat {system:h.definition.body.system,id:h.id,config:Box::new(form.draft.clone())});}
                    if ui.button("Discard staged changes").clicked() {form.draft=c.clone();}
                    if ui.add_enabled(!sim.is_reference(),egui::Button::new("Remove habitat")).clicked() {edit=Some(Edit::RemoveHabitat {system:h.definition.body.system,id:h.id});}
                });
              }
              _=>{
                ui.hyperlink_to("NASA BVAD Rev2 · nominal crew interfaces",habitat::SOURCE);
                ui.label("O₂ 0.895 kg, CO₂ 1.085 kg, dry food 0.800 kg, food water 0.760 kg, potable water 3.217 kg, wastewater 4.467 kg, dry waste 0.120 kg per reference person-day. Metabolic heat 12.426 MJ/day. Reference cycle is limited by finite inputs; unserved demand is not physiology or mortality.");
                if let Some(model)=cosmogon_sim::research::PhysicsModelRegistry::get("habitat") {ui.label(model.equations);ui.label(model.numerical_method);ui.label(model.limitations);}
                ui.label("Export one current snapshot: CSV/JSON interval, exact initial/final world, original definition, current configuration and model/source manifest. A snapshot is not a trajectory; CLI habitat creates a bounded time series.");
                if form.export_path.is_empty() {form.export_path=crate::persistence::data_dir().join("habitat-study").display().to_string();}
                ui.text_edit_singleline(&mut form.export_path);if ui.button("Save snapshot to new folder").clicked() {export=Some((h.id,std::path::PathBuf::from(&form.export_path)));}
              }
            }
        }
      });
    });
    state.habitat_open = open;
    if let Some(run) = clock {
        if run {
            sim.speed = 1;
        }
        sim.paused = !run;
    }
    if let Some(edit) = edit {
        let created = matches!(edit, Edit::CreateHabitat { .. });
        form.status = Some(match sim.edit(edit, time.elapsed_secs_f64()) {
            Ok(out) => {
                if created {
                    form.selected = sim.universe.habitats.last().map(|h| h.id);
                    form.tab = 0;
                }
                form.key = None;
                out.summary
            }
            Err(e) => e,
        });
    }
    if let Some((id, path)) = export {
        form.status = Some(
            match habitat::sample(&sim.universe, id)
                .and_then(|row| habitat::export(&sim.universe, &sim.universe, id, &[row], &path))
            {
                Ok(()) => format!("Saved {}", path.display()),
                Err(e) => e,
            },
        );
    }
    Ok(())
}

pub fn init(mut form: ResMut<HabitatUi>, args: Res<crate::args::Args>) {
    form.tab = match args.panel.as_deref() {
        Some("habitat-controls") => 1,
        Some("habitat-build") => 2,
        Some("habitat-model") => 3,
        Some("greenhouse") => 4,
        Some("habitat-oxygen") => 5,
        _ => 0,
    };
    if let Some(path)=&args.greenhouse_definition {form.tab=4;form.crop.status=form.crop.import(std::path::Path::new(path)).err();}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn habitat_edits_use_reference_protection_and_exact_undo() {
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = cosmogon_sim::BodyRef {
            system: 0,
            body: u.systems[0].find_body("Mars").unwrap() as u32,
        };
        let mut sim = Sim::new(
            u,
            crate::sim::Session::Reference {
                dataset: "test".into(),
                title: "test".into(),
            },
        );
        let edit = Edit::CreateHabitat {
            definition: Box::new(Definition::at(r, 0.0, 0.0)),
        };
        let before = sim.universe.clone();
        assert!(sim.edit(edit.clone(), 0.0).is_err());
        assert_eq!(sim.universe, before);
        // A developer sandbox uses the same edit and snapshot path as ordinary controls.
        sim.session = crate::sim::Session::Sandbox(crate::persistence::Manifest::new(
            "Habitat test",
            "",
            "dev",
            crate::persistence::Origin::default(),
        ));
        sim.edit(edit, 0.0).unwrap();
        assert_eq!(sim.universe.habitats.len(), 1);
        sim.undo(0.0);
        assert_eq!(sim.universe, before);
    }
}
