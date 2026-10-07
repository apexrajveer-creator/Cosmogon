//! Numeric terrain sections of the same field used by the probe and renderer.
use bevy::prelude::*;
use bevy_egui::{egui,EguiContexts};
use cosmogon_sim::{BodyRef,planet::{landing,mola,section::{Definition,Point,Section},terrain::ElevationData}};
use super::{UiState,ACCENT,MUTED};
use crate::{camera::CameraRig,sim::{Sim,Target},surface::SurfaceAnchor};

#[derive(Resource,Default)]
pub struct Study {
    definition: Definition,
    section: Option<Section>,
    cursor: usize,
    status: Option<String>,
    export_path: String,
}
fn endpoint(ui:&mut egui::Ui,title:&str,point:&mut Point) {
    ui.strong(title);
    let old=point.landing_id.clone();
    egui::ComboBox::from_id_salt(title).selected_text(point.landing_id.as_deref().and_then(landing::find).map(|s|s.mission).unwrap_or("Custom coordinate")).show_ui(ui,|ui| {
        ui.selectable_value(&mut point.landing_id,None,"Custom coordinate");
        for site in &landing::SITES {ui.selectable_value(&mut point.landing_id,Some(site.id.into()),site.mission);}
    });
    if old!=point.landing_id {if let Some(id)=&point.landing_id {*point=Point::site(id).expect("registry selection");}}
    let mut changed=false;
    ui.horizontal(|ui| {
        ui.label("N°");changed |= ui.add(egui::DragValue::new(&mut point.latitude_deg).range(-90.0..=90.0).speed(0.05).max_decimals(5)).changed();
        ui.label("E°");changed |= ui.add(egui::DragValue::new(&mut point.longitude_east_deg).range(-360.0..=360.0).speed(0.05).max_decimals(5)).changed();
    });
    if changed {point.landing_id=None;}
}
fn graph(ui:&mut egui::Ui,section:&Section,cursor:&mut usize) {
    ui.label(egui::RichText::new("Height above areoid (m) vs distance on the reference sphere (km)").small().color(MUTED));
    let (rect,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),185.0),egui::Sense::click_and_drag());
    let plot=rect.shrink2(egui::vec2(8.0,18.0));let painter=ui.painter_at(rect);
    painter.rect_filled(rect,4.0,egui::Color32::from_rgb(9,13,23));
    let minimum=section.sampled_minimum_height_m;let maximum=section.sampled_maximum_height_m;
    let span=(maximum-minimum).max(2.0);let low=minimum-span*0.05;let range=span*1.1;
    let at=|i:usize| {let row=&section.rows[i];egui::pos2(plot.left()+(row.distance_m/section.total_distance_m.max(1.0)) as f32*plot.width(),plot.bottom()-((row.reading.height_above_areoid_m-low)/range) as f32*plot.height())};
    for k in 0..=4 {let y=plot.top()+plot.height()*k as f32/4.0;painter.line_segment([egui::pos2(plot.left(),y),egui::pos2(plot.right(),y)],egui::Stroke::new(1.0_f32,egui::Color32::from_gray(35)));}
    painter.add(egui::Shape::line((0..section.rows.len()).map(at).collect(),egui::Stroke::new(1.8_f32,ACCENT)));
    for row in &section.rows {if row.depends_on_provider_interpolation {painter.circle_filled(at(row.index),2.0,egui::Color32::from_rgb(215,120,110));}}
    if response.hovered() || response.dragged() {if let Some(p)=response.interact_pointer_pos().or_else(||ui.input(|i|i.pointer.hover_pos())) {*cursor=(((p.x-plot.left())/plot.width()).clamp(0.0,1.0)*(section.rows.len()-1) as f32).round() as usize;}}
    *cursor=(*cursor).min(section.rows.len()-1);let selected=at(*cursor);
    painter.line_segment([egui::pos2(selected.x,plot.top()),egui::pos2(selected.x,plot.bottom())],egui::Stroke::new(1.0_f32,egui::Color32::from_gray(150)));
    painter.circle_filled(selected,4.0,egui::Color32::WHITE);
    let font=egui::FontId::proportional(10.0);
    painter.text(rect.right_top()-egui::vec2(8.0,0.0),egui::Align2::RIGHT_TOP,format!("Sampled min {:.2} m",minimum),font.clone(),MUTED);
    painter.text(rect.left_top()+egui::vec2(8.0,0.0),egui::Align2::LEFT_TOP,format!("Sampled max {:.2} m",maximum),font.clone(),MUTED);
    painter.text(rect.left_bottom()-egui::vec2(-8.0,2.0),egui::Align2::LEFT_BOTTOM,"0 km",font.clone(),MUTED);
    painter.text(rect.right_bottom()-egui::vec2(8.0,2.0),egui::Align2::RIGHT_BOTTOM,format!("{:.2} km",section.total_distance_m/1000.0),font,MUTED);
}

pub fn panel(mut contexts:EguiContexts,mut state:ResMut<UiState>,mut study:ResMut<Study>,mut sim:ResMut<Sim>,mut rig:ResMut<CameraRig>) -> Result {
    if !state.surface_study_open || state.hidden || state.photo_mode {return Ok(());}
    let ctx=contexts.ctx_mut()?;let mut open=state.surface_study_open;
    let mars=sim.universe.systems.iter().find_map(|s|s.bodies.iter().enumerate().find(|(_,b)|b.exists() && b.name.eq_ignore_ascii_case("Mars") && b.elevation_data.and_then(ElevationData::mola).is_some()).map(|(i,b)|(BodyRef {system:s.id,body:i as u32},b.radius,b.elevation_data.and_then(ElevationData::mola).expect("checked source"))));
    let study=&mut *study;
    let mut action=None;
    egui::Window::new("Mars terrain section").open(&mut open).default_pos([340.0,82.0]).default_width(660.0).show(ctx,|ui| {
        ui.label("Measure broad topography between coordinates or historical landing references.");

        let Some((_,radius,dataset))=mars else {ui.label("This world has no Mars using the pinned MOLA field. Legacy terrain is preserved.");return;};
        ui.label(egui::RichText::new(format!("{} overview · no local slopes, rover routes or landing-safety assessment. Missing uncertainty stays unavailable.",dataset.label())).small().color(ACCENT));
        study.definition.reference_radius_m=radius;
        study.definition.terrain_dataset=dataset.id().into();
        ui.columns(2,|cols| {endpoint(&mut cols[0],"Start",&mut study.definition.start);endpoint(&mut cols[1],"End",&mut study.definition.end);});
        ui.horizontal(|ui| {
            ui.label("Samples");ui.add(egui::DragValue::new(&mut study.definition.samples).range(2..=1024));
            if ui.button("Calculate section").clicked() || study.section.is_none() && study.status.is_none() {
                match study.definition.calculate() {Ok(section)=>{study.section=Some(section);study.cursor=0;study.status=None;},Err(e)=>study.status=Some(e)}
            }
        });
        if let Some(status)=&study.status {ui.label(egui::RichText::new(status).small().color(ACCENT));}
        if let Some(section)=&study.section {
            let current=section.definition==study.definition;
            if !current {ui.colored_label(ACCENT,"Inputs changed. Recalculate to update this section.");}
            ui.label(format!("{:.2} km · {:.2} km sample spacing · {:.2} m sampled relief",section.total_distance_m/1000.0,section.sample_spacing_m/1000.0,section.sampled_maximum_height_m-section.sampled_minimum_height_m));
            ui.label(egui::RichText::new("Axes scale independently; this drawing does not show a physical slope. Extra samples do not add source resolution.").small().color(MUTED));
            graph(ui,section,&mut study.cursor);
            ui.add(egui::Slider::new(&mut study.cursor,0..=section.rows.len()-1).text("Sample row"));
            let row=&section.rows[study.cursor];let reading=&row.reading;
            ui.label(format!("Row {} · {:.3} km · {:.6}° N, {:.6}° E · {:.2} m",row.index,row.distance_m/1000.0,reading.latitude_deg,reading.longitude_east_deg,reading.height_above_areoid_m));
            if row.depends_on_provider_interpolation {ui.colored_label(ACCENT,"This row includes a zero-shot provider-interpolated cell.");}
            ui.horizontal(|ui| {
                if ui.add_enabled(current,egui::Button::new("Inspect this coordinate")).clicked() {action=Some(false);}
                if ui.add_enabled(current,egui::Button::new("Visit this coordinate")).clicked() {action=Some(true);}
            });
            egui::CollapsingHeader::new("Source support and frame").show(ui,|ui| {
                ui.hyperlink_to("NASA PDS MOLA dataset",mola::SOURCE_URL);ui.label(format!("Height: {}. Distance sphere radius: {:.0} m from this simulated Mars.",mola::DATUM,section.definition.reference_radius_m));
                ui.label(landing::REGISTRATION);
                for c in reading.source_cells.iter().filter(|c|c.weight>0.0) {ui.label(format!("Cell [{},{}] · {} m · {} shots · weight {:.6}",c.row,c.column,c.median_height_m,c.observations,c.weight));}
                ui.label("Observation counts are not confidence scores. Height and endpoint positional uncertainty are unavailable.");
            });
            egui::CollapsingHeader::new("Export numeric section").show(ui,|ui| {
                if study.export_path.is_empty() {study.export_path=crate::persistence::data_dir().join("terrain-section").display().to_string();}
                ui.label("New folder only: definition, section JSON, numeric CSV and sources/checksums. Existing exports are protected.");
                ui.text_edit_singleline(&mut study.export_path);
                if ui.add_enabled(current,egui::Button::new("Save section bundle")).clicked() {study.status=Some(match section.export(std::path::Path::new(&study.export_path)) {Ok(())=>format!("Saved {}",study.export_path),Err(e)=>e});}
            });
        }
    });
    if let (Some(visit),Some((r,_,dataset)),Some(section))=(action,mars,&study.section) {
        let row=&section.rows[study.cursor];let reading=&row.reading;
        let site=if row.index==0 {section.definition.start.landing_id.clone()} else if row.index==section.rows.len()-1 {section.definition.end.landing_id.clone()} else {None};
        let result=SurfaceAnchor::new_dataset(r,dataset,reading.latitude_deg,reading.longitude_east_deg,state.surface_altitude,site,state.surface_tracked).map(|a|if state.surface_horizon {a.horizon()} else {a});
        match result {
            Ok(anchor)=>{state.show_surface_anchor(&anchor);sim.selected=Some(Target::Body(r));if visit {if let Err(e)=rig.travel_surface(anchor,&sim) {study.status=Some(e);}}}
            Err(e)=>study.status=Some(e),
        }
    }
    state.surface_study_open=open;Ok(())
}
