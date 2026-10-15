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
    moisture_pane:usize,
    breach_pane:usize,
    crop_pane:usize,
    crop_setup:Definition,
    why: super::habitat_why::View,
    pub crop: super::greenhouse::CropPlanner,
    pub recording: super::habitat_recordings::Study,
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
            moisture_pane:0,
            breach_pane:0,
            crop_pane:0,
            crop_setup:cosmogon_sim::crop_process::definition_at(cosmogon_sim::BodyRef {system:0,body:0},0.,0.),
            why: Default::default(),
            crop: Default::default(),
            recording: Default::default(),
        }
    }
}
impl HabitatUi {
    pub fn crop_metadata(&self,u:&cosmogon_sim::Universe)->Option<serde_json::Value> {
        if self.tab!=11 {return None;}
        Some(serde_json::json!({"pane":self.crop_pane,"selected_habitat":self.selected,"staged_configuration":self.draft,"new_setup":self.crop_setup,"observation":self.selected.and_then(|id|u.habitats.iter().find(|h|h.id==id)).map(|h|h.report())}))
    }

    pub fn breach_metadata(&self,u:&cosmogon_sim::Universe)->Option<serde_json::Value> {
        if self.tab!=10{return None;}
        Some(serde_json::json!({"pane":self.breach_pane,"selected_habitat":self.selected,"staged_configuration":self.draft,"observation":self.selected.and_then(|id|u.habitats.iter().find(|h|h.id==id)).map(|h|h.report())}))
    }
    pub fn moisture_metadata(&self,u:&cosmogon_sim::Universe)->Option<serde_json::Value> {
        if self.tab!=9{return None;}
        Some(serde_json::json!({"pane":self.moisture_pane,"selected_habitat":self.selected,"staged_configuration":self.draft,"observation":self.selected.and_then(|id|u.habitats.iter().find(|h|h.id==id)).map(|h|h.report())}))
    }

    pub fn diagnostic_metadata(&self,u:&cosmogon_sim::Universe)->Option<serde_json::Value>{
        if self.tab!=8 {return None;}
        self.selected.and_then(|id|u.habitats.iter().find(|h|h.id==id)).map(|h|self.why.metadata(h,u.time,"live_completed_interval"))
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
    egui::CollapsingHeader::new("Outward cabin breach").default_open(c.breach.is_some()).show(ui,|ui|super::habitat_breach::controls(ui,&mut c.breach,true));
    egui::CollapsingHeader::new("Cabin moisture").default_open(c.moisture.is_some()).show(ui,|ui|super::habitat_humidity::controls(ui,&mut c.moisture,true));
    egui::CollapsingHeader::new("CO₂ reduction and raw water").default_open(c.sabatier.is_some()).show(ui,|ui|super::habitat_water::controls(ui,&mut c.sabatier,true));
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
    egui::CollapsingHeader::new("Finite supply storage").show(ui,|ui| {
        number(ui,"O₂ reserve tank (kg)",&mut c.oxygen_reserve_capacity_kg,0.0,1.0e7);
        number(ui,"Potable water tank (kg)",&mut c.potable_water_capacity_kg,0.0,1.0e7);
        number(ui,"Dry food store (kg)",&mut c.food_dry_capacity_kg,0.0,1.0e7);
        number(ui,"Food water store (kg)",&mut c.food_water_capacity_kg,0.0,1.0e7);
        ui.label(egui::RichText::new("Each tank is a finite general store for one transferable supply species. Oxygen reserve is never cabin pressure; food-bound water is not potable liquid; dry food implies no energy credit. Stored stock can never exceed its tank.").small().color(MUTED));
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
    mut environment: ResMut<crate::habitat_audio::Listener>,
) -> Result {
    let selected=form.selected;
    form.recording.launch_if_ready(&sim.universe,selected,sim.is_reference());
    form.recording.poll();
    form.why.poll();
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
    let mut listen = None;
    let bottom_margin = if form.tab==7 && form.recording.why_mode {200.0} else {160.0};
    let max_height = (ctx.available_rect().height() - bottom_margin).max(240.0);
    // The planner needs room for its axes and selected-day values on first opening.
    // Keep the window resizable; the scroll area handles smaller screens.
    egui::Window::new("Habitat engineering").open(&mut open).default_pos([345.0,82.0]).default_width(540.0).default_height(if form.tab==4 || form.tab==7 || form.tab==9 || form.tab==10 || form.tab==11 {540.0} else {480.0}).max_height(max_height).show(ctx,|ui| {
      egui::ScrollArea::vertical().max_height(max_height).show(ui,|ui| {
        if form.tab!=4 && form.tab!=7 && form.tab!=9 && form.tab!=10 && form.tab!=11 {ui.label(egui::RichText::new("Experimental mass, power and heat budgets").strong());
        ui.label(egui::RichText::new("NASA reference crew rates · user-assumed equipment · unvalidated field accuracy").small().color(ACCENT));}
        if sim.is_reference() {ui.colored_label(ACCENT,"Clone this reference world to a sandbox before building or configuring habitats.");}
        if form.tab!=7 && (form.tab!=4 || !sim.universe.habitats.is_empty() && (form.crop.pane!=5 || form.crop.supplies.pane==3)) {egui::ComboBox::from_id_salt("habitat-selection").selected_text(form.selected.and_then(|id|sim.universe.habitats.iter().find(|h|h.id==id)).map(|h|h.definition.name.as_str()).unwrap_or("No habitats in this world")).show_ui(ui,|ui| {
            for h in &sim.universe.habitats {ui.selectable_value(&mut form.selected,Some(h.id),format!("{} · #{}",h.definition.name,h.id));}
        });}
        ui.horizontal_wrapped(|ui| {for (i,label) in ["Budgets","Controls","Build & import","Model & export","Crop planner","Oxygen","Water recovery","Recordings","WHY","Humidity","Breach","Installed crops"].iter().enumerate() {ui.selectable_value(&mut form.tab,i,*label);}});
        if let Some(status)=&form.status {ui.colored_label(ACCENT,status);}
        if form.tab==7 {
            super::habitat_recordings::panel(ui,&mut form.recording,&sim.universe,&mut form.selected,sim.is_reference());
        } else if form.tab==4 {
            super::greenhouse::panel(ui,&mut form.crop,&sim.universe,form.selected);
        } else if form.tab==11 && (form.crop_pane==3 || sim.universe.habitats.is_empty()) {
            ui.horizontal(|ui|{ui.selectable_value(&mut form.crop_pane,3,"New crop setup");if !sim.universe.habitats.is_empty(){ui.selectable_value(&mut form.crop_pane,0,"Installed readings");}});
            ui.strong("New finite crop experiment");
            ui.label("Creates original finite stocks and a pinned reference cycle on the selected solid world. Existing habitats receive no stock grant.");
            let body=match sim.selected {Some(Target::Body(r))=>Some(r),_=>None};
            let mut create_crop=false;
            if let Some(r)=body {let b=sim.universe.body(r);ui.label(format!("Selected world: {}",b.name));create_crop=ui.add_enabled(!sim.is_reference() && b.exists() && b.kind.has_surface(),egui::Button::new("Build finite crop experiment")).clicked();} else {ui.label("Select a planet or moon with a solid surface.");}
            ui.text_edit_singleline(&mut form.crop_setup.name);
            number(ui,"Latitude north (°)",&mut form.crop_setup.latitude_deg,-90.,90.);
            number(ui,"Longitude east (°)",&mut form.crop_setup.longitude_east_deg,0.,360.);
            let c=form.crop_setup.config.crop.as_mut().expect("explicit new setup");
            ui.checkbox(&mut c.assume_maintained_conditions,"Assume source growing conditions are maintained");
            ui.label("Explicit ideal chamber boundary, not measured or simulated HVAC. Unchecked means no light request or production.");
            number(ui,"Crop area (m²)",&mut c.crop.area_m2,0.01,1000.);
            number(ui,"Fixture efficacy (µmol/J)",&mut c.crop.fixture_efficacy_umol_j,0.1,20.);
            number(ui,"Assumed dry chemical energy (J/kg)",&mut c.dry_chemical_energy_j_kg,0.,1e9);
            number(ui,"Original captured CO₂ (kg)",&mut form.crop_setup.initial.captured_co2_kg,0.,form.crop_setup.config.capture_capacity_kg);
            number(ui,"Original nutrient equivalent (kg)",&mut form.crop_setup.initial.crop_nutrient_equivalent_kg,0.,1e7);
            number(ui,"Original potable water (kg)",&mut form.crop_setup.initial.potable_water_kg,0.,1e7);
            ui.label("Other crew, battery and equipment inputs use the habitat preset. Import an advanced definition in Build & import for complete customization.");
            if create_crop {let mut d=form.crop_setup.clone();d.body=body.expect("available build target");edit=Some(Edit::CreateHabitat {definition:Box::new(d)});}
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
                    if ui.button("Pause clock").clicked() {clock=Some(false);}
                    if ui.button("WHY this result").clicked() {form.tab=8;}
                    if ui.button("Listen to equipment").clicked() {listen=Some(h.id);}});
                if b.seconds==0.0 {ui.label("Waiting for the first completed minute. Change clock speed to see longer engineering timescales.");}
                if !b.temperature_domain_supported && b.environment_available && b.seconds>0.0 {ui.colored_label(DANGER,"Temperature outside the model’s 100–500 K computation domain; rebuild with viable inputs. Suspended budgets are not predictions.");}
                if !b.environment_available {ui.colored_label(DANGER,"Exterior environment unavailable. Budgets are suspended; no local weather or zero flux is fabricated.");}
                for (i,message) in habitat::ALERTS.iter().enumerate() {if h.alert_bits&(1<<i)!=0 {ui.colored_label(DANGER,*message);}}
                egui::Grid::new("habitat-budgets").num_columns(2).spacing([24.0,7.0]).striped(true).show(ui,|ui| {
                    for (label,value) in [
                        ("Cabin temperature",format!("{:.2} K",s.temperature_k)),("Total pressure",format!("{:.2} kPa",s.pressure_pa(c)/1000.0)),("Oxygen partial pressure",format!("{:.2} kPa",s.oxygen_pa(c)/1000.0)),("CO₂ partial pressure",format!("{:.2} Pa",s.co2_pa(c))),
                        ("Battery",format!("{:.3} / {:.3} kWh",s.battery_j/3.6e6,c.battery_capacity_j/3.6e6)),("Power generated / served",format!("{:.1} / {:.1} W",b.generated_w,b.delivered_w)),("Unserved electrical demand",format!("{:.1} W",b.unserved_w)),("Stored oxygen",format!("{:.3} / {:.0} kg tank",s.oxygen_reserve_kg,c.oxygen_reserve_capacity_kg)),("Potable / wastewater",format!("{:.3} / {:.3} kg",s.potable_water_kg,s.wastewater_kg)),("Dry food / food water",format!("{:.3} / {:.3} kg",s.food_dry_kg,s.food_water_kg)),("Free tank space (O₂ · water · food · food water)",{let free=h.available_cargo();format!("{:.1} · {:.1} · {:.1} · {:.1} kg",free.oxygen_reserve_kg,free.potable_water_kg,free.food_dry_kg,free.food_water_kg)}),("Captured CO₂",format!("{:.3} / {:.3} kg",s.captured_co2_kg,c.capture_capacity_kg)),("Unserved nominal crew demand",format!("{:.3} person-hours",h.totals.unserved_person_seconds/3600.0))] {ui.label(label);ui.strong(value);ui.end_row();}
                });
                if !h.receipts.is_empty() {
                    egui::CollapsingHeader::new(format!("Received cargo transfers ({})",h.receipts.len())).show(ui,|ui| {
                        for r in h.receipts.iter().rev().take(12) {
                            ui.label(format!(
                                "At {:.3} s from craft #{} · requested {:.6} kg → accepted {:.6} kg",
                                r.world_time_s,
                                r.vehicle_id,
                                r.requested.mass_kg(),
                                r.accepted.mass_kg(),
                            ));
                        }
                        if h.receipts.len()>12 {ui.label(format!("{} earlier receipts retained in the save.",h.receipts.len()-12));}
                    });
                }
                egui::CollapsingHeader::new("Interval flows and numerical residuals").show(ui,|ui| {
                    ui.label(format!("O₂ makeup {:.6} kg · CO₂ captured {:.6} kg · water processed/recovered {:.6}/{:.6} kg",b.oxygen_makeup_kg,b.co2_captured_kg,b.water_processed_kg,b.water_recovered_kg));
                    ui.label(format!("Heat: crew {:.1} W · heater {:.1} W (included in bus load) · radiator −{:.1} W · envelope {:+.1} W · storage {:+.1} W",b.metabolic_heat_w,b.heater_w,b.radiator_rejection_w,b.conduction_w,b.thermal_storage_w));
                    ui.collapsing("Accumulated heat accounting",|ui| {
                        if let Some(l)=&h.heat {ui.label(format!("From world time {} s · executed {:.3} s",l.origin_time_s,l.executed_seconds));ui.label(format!("Excluded: environment {:.3} s · starting temperature {:.3} s",l.environment_missing_seconds,l.start_temperature_unsupported_seconds));ui.label("Actual interval heat counters are retained in saves/recordings. Open Cabin balances → Heat ledger for a recorded span.");}
                        else {ui.label("Legacy habitat: no accumulated heat history. Existing minute budgets remain available in WHY.");}
                    });
                    ui.label(format!("Mass residual {:.3e} kg · electric residual {:.3e} J · thermal residual {:.3e} J",b.mass_residual_kg,b.electric_residual_j,b.thermal_residual_j));
                    ui.label(format!("Brine {:.3} kg · dry waste {:.3} kg · supplied reference cycle {:.2}%",s.brine_kg,s.dry_waste_kg,100.0*b.served_cycle_fraction));
                    ui.label("Residuals check bookkeeping; they do not establish real-world predictive accuracy.");
                });
              }
              8=>{form.why.show(ui,h,sim.universe.time,sim.paused);}
              1|5|6|9|10|11=>{
                if form.key!=Some((sim.generation,h.id)) || form.baseline.as_ref()!=Some(c) {form.key=Some((sim.generation,h.id));form.draft=c.clone();form.baseline=Some(c.clone());}
                if form.tab!=9 && form.tab!=10 && form.tab!=11 {ui.label("Changes are staged until Apply. Stored mass/energy stay intact; reducing capacity below stored quantity is rejected.");}
                if form.tab==5 {super::habitat_oxygen::readings(ui,h);}
                if form.tab==6 {super::habitat_water::readings(ui,h);}
                if form.tab==9 {
                    ui.horizontal(|ui| {for (i,label) in ["Overview","Settings","Model & sources"].iter().enumerate() {ui.selectable_value(&mut form.moisture_pane,i,*label);}});
                    if form.moisture_pane==1 {
                        ui.horizontal(|ui| {
                            if ui.add_enabled(!sim.is_reference() && form.draft!=*c,egui::Button::new("Apply configuration")).clicked() {edit=Some(Edit::ConfigureHabitat {system:h.definition.body.system,id:h.id,config:Box::new(form.draft.clone())});}
                            if ui.button("Discard staged changes").clicked() {form.draft=c.clone();}
                        });
                        ui.label(egui::RichText::new("Staged until Apply. Capacity below held liquid is refused; stores stay intact.").small().color(MUTED));
                    }
                    if form.moisture_pane==0 {super::habitat_humidity::readings(ui,h);} else if form.moisture_pane==2 {super::habitat_humidity::model(ui);}
                }
                if form.tab==11 {
                    ui.horizontal_wrapped(|ui| {for (i,label) in ["Overview","Settings","Model & sources","New crop setup"].iter().enumerate() {ui.selectable_value(&mut form.crop_pane,i,*label);}});
                    if form.crop_pane==0 {if super::habitat_crop::readings(ui,h,!sim.is_reference()) && !sim.is_reference() {edit=Some(Edit::HarvestHabitatCrop {system:h.definition.body.system,id:h.id});}}
                    else if form.crop_pane==2 {super::habitat_crop::model(ui);}
                    else if form.crop_pane==1 {
                        ui.add_enabled_ui(!sim.is_reference(),|ui|super::habitat_crop::controls(ui,&mut form.draft.crop,h.definition.config.crop.as_ref()));
                        ui.horizontal(|ui| {if ui.add_enabled(!sim.is_reference() && form.draft!=*c,egui::Button::new("Apply configuration")).clicked(){edit=Some(Edit::ConfigureHabitat {system:h.definition.body.system,id:h.id,config:Box::new(form.draft.clone())});}
                        if ui.button("Discard staged changes").clicked(){form.draft=c.clone();}});
                    }
                }
                if form.tab==10 {
                    ui.horizontal(|ui| {for (i,label) in ["Overview","Settings","Model & sources"].iter().enumerate() {ui.selectable_value(&mut form.breach_pane,i,*label);}});
                    if form.breach_pane==1 {
                        ui.horizontal(|ui| {
                            if ui.add_enabled(!sim.is_reference() && form.draft!=*c,egui::Button::new("Apply configuration")).clicked() {edit=Some(Edit::ConfigureHabitat {system:h.definition.body.system,id:h.id,config:Box::new(form.draft.clone())});}
                            if ui.button("Discard staged changes").clicked() {form.draft=c.clone();}
                        });
                        ui.label(egui::RichText::new("Staged until Apply. Disabling keeps escaped mass/history outside the cabin.").small().color(MUTED));
                        ui.add_enabled_ui(!sim.is_reference(),|ui|super::habitat_breach::controls(ui,&mut form.draft.breach,true));
                    } else if form.breach_pane==0 {super::habitat_breach::readings(ui,h);} else {super::habitat_breach::model(ui);}
                }
                ui.add_enabled_ui(!sim.is_reference(),|ui| {if form.tab==5 {super::habitat_oxygen::controls(ui,&mut form.draft.electrolysis,false);} else if form.tab==6 {super::habitat_water::controls(ui,&mut form.draft.sabatier,false);} else if form.tab==9 {if form.moisture_pane==1 {super::habitat_humidity::controls(ui,&mut form.draft.moisture,true);}} else if form.tab!=10 && form.tab!=11 {config(ui,&mut form.draft);}});
                if form.tab!=9 && form.tab!=10 && form.tab!=11 {ui.horizontal(|ui| {
                    if ui.add_enabled(!sim.is_reference() && form.draft!=*c,egui::Button::new("Apply configuration")).clicked() {edit=Some(Edit::ConfigureHabitat {system:h.definition.body.system,id:h.id,config:Box::new(form.draft.clone())});}
                    if ui.button("Discard staged changes").clicked() {form.draft=c.clone();}
                    if ui.add_enabled(!sim.is_reference(),egui::Button::new("Remove habitat")).clicked() {edit=Some(Edit::RemoveHabitat {system:h.definition.body.system,id:h.id});}
                });}
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
    if let Some(id)=listen {environment.choose(id,sim.generation);state.habitat_open=false;state.alerts_open=true;state.alerts_tab=1;}
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
                    form.tab = if sim.universe.habitats.last().is_some_and(|h|h.uses_crop()) {form.crop_pane=0;11} else {0};
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

pub fn init(mut form: ResMut<HabitatUi>, args: Res<crate::args::Args>, mut exit: MessageWriter<bevy::app::AppExit>) {
    form.tab = match args.panel.as_deref() {
        Some("habitat-why"|"habitat-why-water"|"habitat-why-heat"|"habitat-why-air"|"habitat-why-evidence") => 8,
        Some("habitat-crop"|"habitat-crop-settings"|"habitat-crop-model"|"habitat-crop-build")=>11,
        Some("habitat-controls") => 1,
        Some("habitat-build") => 2,
        Some("habitat-model") => 3,
        Some("greenhouse-chemistry"|"greenhouse-chemistry-model"|"greenhouse-chemistry-files"|"greenhouse-supplies"|"greenhouse-supply-inputs"|"greenhouse-supply-model"|"greenhouse-supply-files"|"greenhouse"|"greenhouse-water"|"greenhouse-daily-water"|"greenhouse-inputs"|"greenhouse-compare"|"greenhouse-model"|"greenhouse-files") => 4,
        Some("habitat-oxygen") => 5,
        Some("habitat-water") => 6,
        Some("habitat-breach"|"habitat-breach-settings"|"habitat-breach-model")=>10,
        Some("habitat-humidity"|"habitat-humidity-settings"|"habitat-humidity-model")=>9,
        Some("habitat-recording-balance-heat"|"habitat-recording-balance"|"habitat-recording-balance-electricity"|"habitat-recording-balance-temperature"|"habitat-recording-balance-counters"|"habitat-recording-balance-files"|"habitat-figure"|"habitat-paired"|"habitat-paired-table"|"habitat-paired-files"|"habitat-paired-replay"|"habitat-graph-layout"|"habitat-recordings"|"habitat-recording-pressure"|"habitat-recording-pressure-change"|"habitat-recording-pressure-breach"|"habitat-recording-pressure-files"|"habitat-experiments"|"habitat-recording-why"|"habitat-recording-why-water"|"habitat-recording-why-heat"|"habitat-recording-why-air"|"habitat-recording-why-evidence") => 7,
        _ => 0,
    };
    form.crop_pane=match args.panel.as_deref(){Some("habitat-crop-settings")=>1,Some("habitat-crop-model")=>2,Some("habitat-crop-build")=>3,_=>0};
    form.breach_pane=match args.panel.as_deref(){Some("habitat-breach-settings")=>1,Some("habitat-breach-model")=>2,_=>0};
    form.moisture_pane=match args.panel.as_deref(){Some("habitat-humidity-settings")=>1,Some("habitat-humidity-model")=>2,_=>0};
    form.crop.pane=match args.panel.as_deref() {Some("greenhouse-inputs")=>1,Some("greenhouse-compare")=>2,Some("greenhouse-model")=>3,Some("greenhouse-files")=>4,_=>0};
    if args.panel.as_deref().is_some_and(|p|p.starts_with("greenhouse-suppl") || p.starts_with("greenhouse-chemistry")){form.crop.pane=5;}
    form.crop.supplies.pane=match args.panel.as_deref(){Some("greenhouse-supply-inputs")=>1,Some("greenhouse-supply-model")=>2,Some("greenhouse-supply-files")=>3,_=>0};
    if args.panel.as_deref().is_some_and(|p|p.starts_with("greenhouse-chemistry")) || args.crop_chemistry.is_some() || args.crop_chemistry_output.is_some() {form.tab=4;form.crop.pane=5;form.crop.supplies.pane=4;}
    form.crop.supplies.chemistry.pane=match args.panel.as_deref(){Some("greenhouse-chemistry-model")=>1,Some("greenhouse-chemistry-files")=>2,_=>0};
    if let Some(p)=&args.crop_chemistry_output {form.crop.supplies.chemistry.output=p.clone();form.crop.supplies.chemistry.launch=true;}
    form.crop.supplies.selected_day=args.greenhouse_supply_day.unwrap_or(30);
    form.crop.supplies.resource=args.greenhouse_supply_resource.as_deref().and_then(|r|cosmogon_sim::greenhouse_supplies::RESOURCES.iter().position(|k|*k==r)).unwrap_or(0);
    if let Some(p)=&args.greenhouse_supply_output {form.crop.supplies.output=p.clone();form.crop.supplies.launch=true;}
    form.crop.graph=match args.panel.as_deref() {Some("greenhouse-water")=>3,Some("greenhouse-daily-water")=>4,_=>0};
    form.why.section=match args.panel.as_deref(){Some("habitat-why-water")=>1,Some("habitat-why-heat")=>2,Some("habitat-why-air")=>3,Some("habitat-why-evidence")=>4,_=>0};
    form.recording.balance_mode=args.panel.as_deref().is_some_and(|p|p.starts_with("habitat-recording-balance")) || args.habitat_balance_output.is_some() || args.habitat_balance_from.is_some();
    form.recording.balance.from=args.habitat_balance_from.unwrap_or(0);form.recording.balance.to=args.habitat_sample;
    form.recording.balance.pane=match args.panel.as_deref(){Some("habitat-recording-balance-electricity")=>1,Some("habitat-recording-balance-temperature")=>2,Some("habitat-recording-balance-counters")=>3,Some("habitat-recording-balance-files")=>4,Some("habitat-recording-balance-heat")=>5,_=>0};
    if let Some(output)=&args.habitat_balance_output {form.recording.balance.output=output.clone();form.recording.balance.launch=true;}
    form.recording.figure_mode=args.panel.as_deref()==Some("habitat-figure") || args.habitat_figure_output.is_some();
    if let Some(output)=&args.habitat_figure_output {form.recording.figure.output=output.clone();form.recording.figure.launch=true;}
    form.recording.pressure_mode=args.panel.as_deref().is_some_and(|p|p.starts_with("habitat-recording-pressure"));
    form.recording.pressure.pane=match args.panel.as_deref(){Some("habitat-recording-pressure-change")=>1,Some("habitat-recording-pressure-breach")=>2,Some("habitat-recording-pressure-files")=>3,_=>0};
    if let Some(index)=args.habitat_pressure_reference {form.recording.pressure.reference=super::habitat_pressure::Reference::Chosen;form.recording.pressure.index=index;}
    form.recording.why_mode=args.panel.as_deref().is_some_and(|p|p.starts_with("habitat-recording-why"));
    form.recording.why.section=match args.panel.as_deref(){Some("habitat-recording-why-water")=>1,Some("habitat-recording-why-heat")=>2,Some("habitat-recording-why-air")=>3,Some("habitat-recording-why-evidence")=>4,_=>0};
    form.recording.experiment_mode=args.panel.as_deref()==Some("habitat-experiments");
    form.recording.keep_experiment_at_launch=form.recording.experiment_mode;
    form.recording.graph.load_preferences();
    if let Some(path)=&args.habitat_graph {match super::habitat_graph::Layout::read(std::path::Path::new(path)){Ok(d)=>form.recording.graph.layout=d,Err(e)=>form.recording.graph.status=Some(e)}}
    form.recording.apply_graph_layout();
    form.recording.graph_mode=args.panel.as_deref()==Some("habitat-graph-layout");
    form.recording.requested_origin=args.habitat_origin.as_deref().map(|v|v=="saved");
    form.recording.rerun_output=args.habitat_rerun_output.clone();
    form.recording.requested_branches=args.habitat_branches.as_deref().and_then(|s|cosmogon_sim::habitat_design::parse_branches(s).ok());
    if let Some(path)=&args.habitat_recording {form.tab=7;form.recording.metric=args.habitat_metric.as_deref().and_then(|v|cosmogon_sim::habitat_recording::Metric::parse(v).ok()).unwrap_or(form.recording.metric);form.recording.start(path.clone(),args.habitat_sample);}
    form.recording.factorial_mode=args.panel.as_deref().is_some_and(|p|p.starts_with("habitat-paired")) || form.recording.figure_mode && args.habitat_recording.as_ref().is_some_and(|p|std::path::Path::new(p).join("plan.json").exists());
    form.recording.paired_pane=match args.panel.as_deref() {Some("habitat-paired-table")=>1,Some("habitat-paired-files")=>2,Some("habitat-paired-replay")=>3,_=>0};
    form.recording.replay.launch=args.habitat_verify;
    form.recording.effect=args.habitat_effect.as_deref().and_then(|s|cosmogon_sim::habitat_factorial::Effect::parse(s).ok()).unwrap_or(form.recording.effect);
    if let Some(path)=&args.habitat_compare {form.recording.start_comparison(path.clone());}
    if let Some(a)=&args.habitat_alignment {form.recording.alignment=cosmogon_sim::habitat_comparison::Alignment::parse(a).unwrap_or_default();}
    if let Some(path)=&args.habitat_sweep {
        form.tab=7;form.recording.metric=args.habitat_metric.as_deref().and_then(|v|cosmogon_sim::habitat_recording::Metric::parse(v).ok()).unwrap_or(form.recording.metric);
        match cosmogon_sim::habitat_sweep::Plan::read(std::path::Path::new(path)) {Ok(plan)=>{if form.recording.figure_mode {form.recording.factorial_mode=plan.secondary.is_some();} form.recording.launch=Some(super::habitat_recordings::Launch {plan,id:args.habitat_study_id,output:args.habitat_study_output.clone().unwrap_or_default(),sample:args.habitat_sample,initial_path:args.load.clone()});},Err(e)=>form.recording.status=Some(e)}
    }
    form.recording.sync_graph_layout();
    if let Some(path)=&args.crop_chemistry {form.tab=4;form.crop.pane=5;form.crop.supplies.pane=4;if let Err(e)=form.crop.supplies.import_chemistry(std::path::Path::new(path)){error!("Cannot open crop chemistry: {e}");form.crop.supplies.status=Some(e);exit.write(bevy::app::AppExit::error());}}
    if let Some(path)=&args.greenhouse_supplies {form.tab=4;form.crop.pane=5;if let Err(e)=form.crop.supplies.import(std::path::Path::new(path)){error!("Cannot open crop supply study: {e}");form.crop.supplies.status=Some(e);exit.write(bevy::app::AppExit::error());}}
    if let Some(path)=&args.greenhouse_definition {form.tab=4;if let Err(e)=form.crop.import(std::path::Path::new(path)) {error!("Cannot open greenhouse definition at startup: {e}");form.crop.status=Some(e);exit.write(bevy::app::AppExit::error());}}
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
