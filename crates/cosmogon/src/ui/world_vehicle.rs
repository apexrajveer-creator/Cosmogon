//! Native controls for actual world spacecraft and separate pinned flight studies.
use super::{ACCENT, MUTED};
use crate::sim::{Sim, Target};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::{
    research,
    sandbox::Edit,
    save,
    supply_vehicle::{Cargo, Plan},
    vehicle_arrival::{Approach, GeographicStart},
    vehicle_arrival_recording::Report as ArrivalReport,
    vehicle_support::{Definition as SupportDefinition,Reading as SupportReading},
    vehicle_support_recording::Report as SupportReport,
    world_vehicle::{Definition, Flight, Phase},
    world_vehicle_recording::{self as recording, Bundle, Study},
    Universe,
};
use serde_json::{json, Value};
use std::path::Path;

type WorkerResult = Result<(Bundle, String, bool), String>;
#[derive(Default)]
pub struct FlightUi {
    draft: Option<Definition>,
    selected_vehicle: Option<u64>,
    samples: usize,
    interval_s: f64,
    result: Option<Bundle>,
    source: String,
    opened_directory: Option<String>,
    input: String,
    origin_input: String,
    definition_input: String,
    definition_output: String,
    output: String,
    selected_row: usize,
    metric: usize,
    status: Option<String>,
    task: Option<Task<WorkerResult>>,
    queued_output: Option<String>,
    requested_row: Option<usize>,
    request: Option<Value>,
    live_observation: Option<recording::Observation>,
    geographic_start: GeographicStart,
    arrival_result: Option<ArrivalReport>,
    arrival_task: Option<Task<Result<ArrivalReport, String>>>,
    arrival_requested: bool,
    arrival_status: Option<String>,
    arrival_output: String,
    live_approach: Option<Value>,
    support_draft:SupportDefinition,
    support_result:Option<SupportReport>,
    support_task:Option<Task<Result<(SupportReport,bool),String>>>,
    support_requested:bool,
    support_output:String,
    support_input:String,
    support_status:Option<String>,
    live_support:Option<Value>,
    cargo_removal_kg: f64,
    cargo_removal_relative: [f64; 3],
    transfer_habitat: Option<u64>,
    transfer_cargo: [f64; 4],
    transfer_status: Option<String>,

}
impl FlightUi {
    pub fn initialize(&mut self, args: &crate::args::Args) {
        self.support_requested=args.panel.as_deref()==Some("vehicle-support") || args.support_output.is_some();
        self.support_output=args.support_output.clone().unwrap_or_default();
        if let Some(path)=&args.support_definition {match SupportDefinition::read(Path::new(path)) {Ok(d)=>self.support_draft=d,Err(e)=>{self.support_status=Some(e);self.support_requested=false;}}}
        if let Some(path)=&args.support_recording {self.support_input=path.clone();self.requested_row=args.flight_observation;self.open_support(path.clone());return;}
        self.arrival_requested =
            args.panel.as_deref() == Some("vehicle-arrival") || args.arrival_output.is_some();
        self.arrival_output = args.arrival_output.clone().unwrap_or_default();
        self.samples = 121;
        self.interval_s = 1.;
        self.output = args.flight_output.clone().unwrap_or_default();
        self.requested_row = args.flight_observation;
        if let Some(path) = &args.flight_recording {
            self.input = path.clone();
            self.queued_output = args.flight_output.clone();
            self.open(path.clone());
        } else if let Some(definition) = &args.flight_definition {
            self.definition_input = definition.clone();
            self.origin_input = args.flight_origin.clone().unwrap_or_default();
            let origin = self.origin_input.clone();
            let definition = definition.clone();
            let output = args.flight_output.clone();
            self.request = Some(
                json!({"operation":"Run explicit pinned world-flight definition","definition":definition,"origin":origin,"output":output}),
            );
            self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
                let study = Study::read(Path::new(&definition))?;
                let world = recording::read_world(Path::new(&origin))?;
                let bundle = Bundle::run(&world, study)?;
                if let Some(path) = output {
                    bundle.export(Path::new(&path))?;
                    return Ok((Bundle::read(Path::new(&path))?, path, true));
                }
                Ok((
                    bundle,
                    format!("Pinned definition {definition}; original world {origin}"),
                    true,
                ))
            }));
            self.status = Some("Running actual flight in the exact pinned world branch…".into());
        }
    }
    pub fn pending(&self) -> bool {
        self.task.is_some() || self.queued_output.is_some() || self.arrival_task.is_some() || self.support_task.is_some()
    }
    pub fn metadata(&self) -> Value {
        json!({"support_draft":self.support_draft,"support_context":cosmogon_sim::vehicle_support::context(),"support_manifest":self.support_result.as_ref().map(|r|&r.manifest),"support_selected_row":self.support_result.as_ref().and_then(|r|r.points.get(self.selected_row)),"support_status":self.support_status,"support_pending":self.support_task.is_some(),"live_support":self.live_support,"live_approach":self.live_approach,"arrival_context":cosmogon_sim::vehicle_arrival::context(),"geographic_start":self.geographic_start,"arrival_pending":self.arrival_task.is_some(),"arrival_selected_row":self.arrival_result.as_ref().and_then(|r|r.points.get(self.selected_row)),"arrival_manifest":self.arrival_result.as_ref().map(|r|&r.manifest),"arrival_status":self.arrival_status,"live_observation":self.live_observation,"staged_flight":self.draft,"request":self.request,"pending":self.pending(),"source":self.source,"selected_vehicle":self.selected_vehicle,"selected_observation":self.selected_row,"selected_row":self.result.as_ref().and_then(|r|r.rows.get(self.selected_row)),"complete_manifest":self.result.as_ref().map(|r|&r.manifest),"status":self.status,"scope":"Actual live-world craft edits, plus independent pinned-world flight studies; no physical cargo delivery"})
    }
    pub fn poll(&mut self) {
        if let Some(result)=self.support_task.as_mut().and_then(|t|block_on(future::poll_once(t))) {
            self.support_task=None;
            match result {
                Ok((report,restore))=>{
                    if self.requested_row.is_some_and(|i|i>=report.points.len()) {self.requested_row=None;self.support_status=Some("Requested support row is outside the actual report; previous results are preserved.".into());}
                    else {
                        self.selected_row=self.requested_row.take().unwrap_or(self.selected_row).min(report.points.len()-1);
                        if restore {self.support_draft=report.definition.clone();self.draft=Some(report.flight.study.flight.clone());self.result=Some(report.flight.clone());self.source=self.support_input.clone();self.opened_directory=Some(Path::new(&self.support_input).join("flight").to_string_lossy().into());}
                        self.support_result=Some(report);self.support_status=Some("Original-world footprint, forces and conditional hold evidence verified. Cargo remains aboard.".into());
                    }
                },Err(e)=>self.support_status=Some(format!("Support operation failed: {e}. Previous evidence is preserved.")),
            }
        }

        if let Some(result) = self
            .arrival_task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.arrival_task = None;
            match result {
                Ok(report) => {
                    self.arrival_result = Some(report);
                    self.arrival_status = Some(
                        "Replayed rotating approach evidence retained; contact is not a landing."
                            .into(),
                    );
                }
                Err(e) => {
                    self.arrival_status = Some(format!("Approach calculation failed: {e}"));
                }
            }
        }
        if let Some(result) = self
            .task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.task = None;
            match result {
                Ok((bundle, source, restore)) => {
                    if self.requested_row.is_some_and(|i| i >= bundle.rows.len()) {
                        self.requested_row = None;
                        self.queued_output = None;
                        self.status=Some("Requested flight observation is outside the actual recording; previous results are preserved.".into());
                        return;
                    }
                    self.selected_row = self
                        .requested_row
                        .take()
                        .unwrap_or(self.selected_row)
                        .min(bundle.rows.len().saturating_sub(1));
                    if restore {
                        self.draft = Some(bundle.study.flight.clone());
                        self.samples = bundle.study.samples;
                        self.interval_s = bundle.study.interval_s;
                    }
                    self.opened_directory = Path::new(&source).is_dir().then_some(source.clone());
                    self.source = source;
                    self.result = Some(bundle);
                    self.arrival_result = None;
                    if self.support_result.as_ref().is_some_and(|r|self.result.as_ref().is_none_or(|b|r.flight.manifest!=b.manifest)) {self.support_result = None;}
                    if self.support_requested && self.queued_output.is_none() {self.support_requested=false;self.start_support();}
                    if self.arrival_requested {
                        self.start_arrival();
                    }
                    self.status=Some("Exact flight observations and the complete world branch retained; live world unchanged.".into());
                }
                Err(e) => {
                    self.queued_output = None;
                    self.requested_row = None;
                    self.status = Some(format!(
                        "Flight operation failed: {e}. Previous results are preserved."
                    ));
                }
            }
        }
        if self.task.is_none() && self.arrival_task.is_none() {
            if let Some(path) = self.queued_output.take() {
                self.export(path);
            }
        }
    }
    fn open(&mut self, path: String) {
        if self.pending() && self.task.is_some() {
            return;
        }
        self.request = Some(
            json!({"operation":"Verify and reopen exact original flight recording","recording":path}),
        );
        self.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { Ok((Bundle::read(Path::new(&path))?, path, true)) }),
        );
        self.status = Some(
            "Checking source pins, every observation, CSV and complete final-world replay…".into(),
        );
    }
    fn start(&mut self, world: &Universe, original: bool) {
        if self.pending() {
            return;
        }
        let Some(d) = self.draft.clone() else {
            self.status = Some("Choose explicit initial flight conditions first.".into());
            return;
        };
        let origin = if original {
            match &self.result {
                Some(b) => b.origin.clone(),
                None => return,
            }
        } else {
            world.clone()
        };
        let study = match Study::new(&origin, d, self.samples, self.interval_s) {
            Ok(s) => s,
            Err(e) => {
                self.status = Some(e);
                return;
            }
        };
        let directory = original.then(|| self.opened_directory.clone()).flatten();
        let expected = research::state_fingerprint(&origin).expect("validated study origin");
        self.request = Some(
            json!({"operation":"Run exact request-time flight branch","origin_kind":if original {"original_recorded_world"} else {"current_live_world"},"initial_state_fingerprint":expected,"initial_time_s":origin.time,"data_versions":save::data_versions(&origin),"study":study}),
        );
        self.task=Some(AsyncComputeTaskPool::get().spawn(async move {
            let origin=if let Some(directory)=directory {
                let loaded=Bundle::read(Path::new(&directory))?;
                if research::state_fingerprint(&loaded.origin)? != expected {return Err("Original flight starting world changed on disk; reopen it before rerunning".into());}
                loaded.origin
            } else {origin};
            Ok((Bundle::run(&origin,study)?,"Independent actual flight branch from request-time origin".into(),false))
        }));
        self.status = Some(
            "Resolving finite burns, gravity and supported contacts in the pinned branch…".into(),
        );
    }
    fn export(&mut self, path: String) {
        if self.task.is_some() {
            return;
        }
        let Some(bundle) = self.result.clone() else {
            self.status = Some("Run or reopen a complete flight first.".into());
            return;
        };
        self.request = Some(
            json!({"operation":"Export completed flight observations and worlds","source":self.source,"origin_fingerprint":bundle.manifest["origin_fingerprint"],"output":path}),
        );
        self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
            bundle.export(Path::new(&path))?;
            Ok((Bundle::read(Path::new(&path))?, path, false))
        }));
        self.status =
            Some("Independently replaying and writing protected full-world evidence…".into());
    }
    pub fn draw(&mut self, ui: &mut egui::Ui, sim: &mut Sim, plan: &Plan, pane: usize, now: f64) {
        if self.samples == 0 {
            self.samples = 121;
            self.interval_s = 1.;
        }
        if let Some(status) = &self.status {
            ui.add(egui::Label::new(status).wrap());
        }
        ui.label(
            egui::RichText::new(
                "Finite spacecraft · real world gravity · explicit authored initial conditions",
            )
            .strong(),
        );
        ui.label(egui::RichText::new("Contact does not establish safe landing. Cargo remains aboard; no habitat supply grant.").small().color(ACCENT));
        match pane {
            4 => self.live(ui, sim, plan, now),
            5 => self.recorded(ui, &sim.universe),
            7 => self.arrival(ui, sim, plan, now),
            8 => self.support(ui,sim),
            _ => self.files(ui, &sim.universe),
        }
    }
    fn open_support(&mut self,path:String) {
        if self.pending(){return;}
        self.support_input=path.clone();self.support_status=Some("Checking every support reading and its exact original-world flight branch…".into());
        self.support_task=Some(AsyncComputeTaskPool::get().spawn(async move {Ok((SupportReport::read(Path::new(&path))?,true))}));
    }
    fn start_support(&mut self) {
        if self.task.is_some() || self.support_task.is_some() || self.arrival_task.is_some(){return;}
        let Some(bundle)=self.result.clone() else {self.support_status=Some("Run or reopen a terrain-contact flight first.".into());return;};
        let definition=self.support_draft.clone();if let Err(e)=definition.validate(){self.support_status=Some(e);return;}
        let directory=self.opened_directory.clone();let expected=bundle.manifest.clone();let output=self.support_output.trim().to_string();
        self.request=Some(json!({"operation":"Derive exact original-world footprint and conditional hold report","flight_manifest":expected,"support_definition":definition,"output":output}));
        self.support_status=Some("Replaying actual contact loads and the original moving terrain…".into());
        self.support_task=Some(AsyncComputeTaskPool::get().spawn(async move {
            let bundle=if let Some(directory)=directory {let loaded=Bundle::read(Path::new(&directory))?;if loaded.manifest!=expected {return Err("Opened flight source changed; reopen it before deriving support evidence".into());}loaded}else{bundle};
            let report=SupportReport::run(&bundle,definition)?;
            if !output.is_empty(){report.export(Path::new(&output))?;return Ok((SupportReport::read(Path::new(&output))?,false));}Ok((report,false))
        }));
    }
    fn support(&mut self,ui:&mut egui::Ui,sim:&Sim) {
        ui.strong("Footprint, loads and ground-relative motion");
        ui.label("A contact footprint shows where the modeled feet push. A conditional hold comparison checks the forces needed to follow the rotating ground. Neither attaches the craft or permits delivery.");
        if let Some(status)=&self.support_status {ui.add(egui::Label::new(status).wrap());}
        if let Some(report)=&self.support_result {
            ui.label("Verified original-world observations · completed assumptions remain frozen");
            ui.add(egui::Slider::new(&mut self.selected_row,0..=report.points.len()-1).text("Observation"));let p=&report.points[self.selected_row];
            if let Some(r)=&p.reading {support_readings(ui,r);}else{ui.colored_label(ACCENT,p.unavailable_reason.as_deref().unwrap_or("Support unavailable"));}
        }else{
            let craft=self.selected_vehicle.and_then(|id|sim.universe.vehicles.iter().find(|f|f.id==id)).or_else(||sim.universe.vehicles.iter().find(|f|f.contact.is_some()));
            self.live_support=craft.map(|f|match SupportReading::live(&sim.universe,f,&self.support_draft) {Ok(r)=>json!({"reading":r}),Err(e)=>json!({"unavailable_reason":e})});
            if let Some(value)=&self.live_support {if let Some(reading)=value.get("reading") {if let Ok(r)=serde_json::from_value::<SupportReading>(reading.clone()){support_readings(ui,&r);}}else{ui.colored_label(ACCENT,value["unavailable_reason"].as_str().unwrap_or("Support unavailable"));}}
            else {ui.label("Create a terrain-contact craft for live readings, or reopen a complete flight for original-world evidence.");}
        }
        ui.collapsing("Authored comparison limits for a new report",|ui|{
            ui.label("Editable engineering assumptions. Completed reports retain their own limits; inclusion is an instantaneous comparison, not a landing certificate.");
            let d=&mut self.support_draft;
            for(label,value)in[("Plane departure (m)",&mut d.maximum_plane_departure_m),("Normal deviation (rad)",&mut d.maximum_normal_deviation_rad),("Coordinate roundoff indicator (m)",&mut d.maximum_position_roundoff_m),("Minimum hull edge (m)",&mut d.minimum_hull_edge_m),("Required load margin (m)",&mut d.minimum_load_margin_m),("Relative speed (m/s)",&mut d.maximum_relative_speed_m_s),("Relative angular speed (rad/s)",&mut d.maximum_relative_angular_speed_rad_s),("Relative acceleration (m/s²)",&mut d.maximum_relative_acceleration_m_s2),("Force imbalance fraction",&mut d.maximum_force_imbalance_fraction),("Torque imbalance fraction",&mut d.maximum_torque_imbalance_fraction),("Derivative step (s)",&mut d.derivative_step_s),("Derivative error indicator (m/s²)",&mut d.maximum_derivative_error_m_s2)]{ui.horizontal_wrapped(|ui|{ui.label(label);ui.add(egui::DragValue::new(value).speed(0.001).max_decimals(9));});}
        });
        ui.label("New protected support folder (optional)");ui.text_edit_singleline(&mut self.support_output);
        if ui.add_enabled(!self.pending()&&self.result.is_some(),egui::Button::new("Derive support from original recorded world")).clicked(){self.start_support();}
        ui.label("Complete support report folder");ui.text_edit_singleline(&mut self.support_input);
        if ui.add_enabled(!self.pending()&&!self.support_input.trim().is_empty(),egui::Button::new("Verify and reopen support report")).clicked(){self.open_support(self.support_input.trim().into());}
        ui.collapsing("Complete support model, source and frozen assumptions",|ui|{ui.add(egui::Label::new(serde_json::to_string_pretty(&json!({"context":cosmogon_sim::vehicle_support::context(),"retained_definition":self.support_result.as_ref().map(|r|&r.definition)})).unwrap()).wrap());});
    }
    fn start_arrival(&mut self) {
        if self.task.is_some() || self.arrival_task.is_some() {
            return;
        }
        let Some(bundle) = self.result.clone() else {
            return;
        };
        let directory = self.opened_directory.clone();
        let expected = bundle.manifest.clone();
        let output = self.arrival_output.trim().to_string();
        self.arrival_status =
            Some("Replaying the complete original branch and rotating source frames…".into());
        self.arrival_task=Some(AsyncComputeTaskPool::get().spawn(async move {
            let bundle=if let Some(directory)=directory {let loaded=Bundle::read(Path::new(&directory))?;if loaded.manifest!=expected {return Err("Opened flight source changed on disk; reopen it before deriving approach evidence".into());}loaded} else {bundle};
            let report=ArrivalReport::run(&bundle,bundle.study.flight.parent)?;
            if !output.is_empty() {report.export(Path::new(&output))?;return ArrivalReport::read(Path::new(&output));}
            Ok(report)
        }));
    }
    fn arrival(&mut self, ui: &mut egui::Ui, sim: &mut Sim, plan: &Plan, now: f64) {
        ui.strong("Rotating surface-relative approach");
        ui.label("Ground motion includes body translation and spin. Geographic altitude is above the spherical flight boundary; overview terrain is a separate reading.");
        if let Some(status) = &self.arrival_status {
            ui.add(egui::Label::new(status).wrap());
        }
        if let Some(report) = &self.arrival_result {
            ui.label(format!(
                "Recorded target {} · {} actual samples",
                report.flight.origin.body(report.definition.target).name,
                report.points.len()
            ));
            ui.add(
                egui::Slider::new(&mut self.selected_row, 0..=report.points.len() - 1)
                    .text("Observation"),
            );
            let point = &report.points[self.selected_row];
            ui.label(format!(
                "World sample {:.6} s · {}",
                point.world_time_s,
                if point.historical {
                    "original contact frame retained"
                } else {
                    "source frame observed in actual replay"
                }
            ));
            if let Some(reading) = &point.reading {
                arrival_readings(ui, reading);
            } else if let Some(reason) = &point.unavailable_reason {
                ui.colored_label(ACCENT, reason);
            }
        } else if let Some(flight) = self
            .selected_vehicle
            .and_then(|id| sim.universe.vehicles.iter().find(|c| c.id == id))
            .or_else(|| sim.universe.vehicles.first())
        {
            let value = Approach::live(&sim.universe, flight, flight.definition.parent);
            self.live_approach = Some(match &value {
                Ok(reading) => {
                    json!({"reading":reading,"scope":"Actual active live craft at its resolved clock"})
                }
                Err(reason) => json!({"unavailable_reason":reason}),
            });
            match value {
                Ok(reading) => arrival_readings(ui, &reading),
                Err(reason) => {
                    ui.colored_label(ACCENT, reason);
                }
            }
        } else {
            self.live_approach = None;
            ui.label("Create a finite live craft or reopen a complete flight recording to observe approach.");
        }
        ui.collapsing("Geographic initial conditions",|ui| {
            let mut contact=self.geographic_start.model==cosmogon_sim::vehicle_arrival::CONTACT_START_MODEL;
            if ui.checkbox(&mut contact,"Stage geographic conditions with terrain contact hardware").changed() {self.geographic_start.model=if contact {cosmogon_sim::vehicle_arrival::CONTACT_START_MODEL} else {cosmogon_sim::vehicle_arrival::MODEL}.into();}
            ui.label("Explicit authored ground-relative start. Contact mode permits signed height relative to the spherical base; actual terrain and feet must clear. No surface launch, safe landing or stock loading is implied.");
            for (label,value) in [("North latitude (°)",&mut self.geographic_start.latitude_deg),("East longitude (°)",&mut self.geographic_start.longitude_east_deg),("Above spherical radius (m)",&mut self.geographic_start.spherical_altitude_m),("East velocity (m/s)",&mut self.geographic_start.east_velocity_m_s),("North velocity (m/s)",&mut self.geographic_start.north_velocity_m_s),("Up velocity (m/s)",&mut self.geographic_start.up_velocity_m_s)] {
                ui.horizontal(|ui|{ui.label(label);ui.add(egui::DragValue::new(value).speed(0.1).max_decimals(9));});
            }
            let target=match sim.selected {Some(Target::Body(r))=>Some(r),_=>None};
            let candidate=target.ok_or_else(||"Select a solid starting body".to_string()).and_then(|r| {
                if contact {
                    let pose=self.geographic_start.absolute(&sim.universe,r)?;
                    let hardware=if let Some(h)=self.draft.as_ref().filter(|d|d.parent==r).and_then(|d|d.contact.clone()) {h} else {cosmogon_sim::vehicle_contact_dynamics::Hardware::example(&sim.universe,r,pose.position_m)?};
                    self.geographic_start.contact_flight(&sim.universe,r,plan.clone(),hardware)
                } else {self.geographic_start.flight(&sim.universe,r,plan.clone())}
            });
            if let Err(reason)=&candidate {ui.colored_label(ACCENT,reason);}
            if ui.add_enabled(candidate.is_ok(),egui::Button::new("Stage these geographic conditions for a recorded study")).clicked() {
                self.draft=candidate.as_ref().ok().cloned();self.status=Some(format!("Geographic vectors staged at world {:.6} s; reuse this paused world for the same longitude, or explicitly recompute before creation.",sim.universe.time));
            }
            if ui.add_enabled(candidate.is_ok() && !sim.is_reference(),egui::Button::new("Create geographic craft at the current world clock")).clicked() {
                match sim.edit(Edit::CreateVehicle {definition:Box::new(candidate.unwrap())},now) {Ok(_)=>{self.selected_vehicle=sim.universe.vehicles.last().map(|c|c.id);self.arrival_result=None;self.arrival_status=Some("Geographic craft created through an undoable edit using the actual current frame.".into());},Err(e)=>self.arrival_status=Some(e)}
            }
        });
        ui.collapsing("Recorded approach evidence and files",|ui| {
            ui.label("Uses the complete original flight and starting body; independently resolves every frame. Historical contact keeps its first observed frame.");
            ui.label("Fresh approach output folder (optional)");ui.text_edit_singleline(&mut self.arrival_output);
            if ui.add_enabled(!self.pending() && self.result.is_some(),egui::Button::new("Replay approach from retained flight")).clicked() {self.arrival_requested=true;self.start_arrival();}
            ui.hyperlink_to("JPL NAIF rotation and state transforms","https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html");
            ui.label("Relative kinetic energy is not landing-gear force, stability or safety. Source velocity is the derivative of the modeled ephemeris; spherical-contact centre interpolation and overview terrain registration remain approximate.");
        });
    }
    fn live(&mut self, ui: &mut egui::Ui, sim: &mut Sim, plan: &Plan, now: f64) {
        ui.label(format!(
            "Actual world clock {:.6} s · simulation {}",
            sim.universe.time,
            if sim.paused {
                "paused"
            } else if sim.last.lagging {
                "resolving flight; requested speed exceeds available processing"
            } else {
                "running"
            }
        ));
        if sim.is_reference() {
            ui.colored_label(
                ACCENT,
                "Clone this reference to a sandbox before creating or removing live craft.",
            );
        }
        let selected = match sim.selected {
            Some(Target::Body(r)) => Some(r),
            _ => None,
        };
        let label = selected
            .map(|r| {
                format!(
                    "Use selected {} · assumed circular start at 200 km",
                    sim.universe.body(r).name
                )
            })
            .unwrap_or_else(|| "Select a starting planet or moon".into());
        if ui
            .add_enabled(selected.is_some(), egui::Button::new(label))
            .clicked()
        {
            match Definition::circular_example(&sim.universe, selected.unwrap(), plan.clone()) {
                Ok(d) => self.draft = Some(d),
                Err(e) => self.status = Some(e),
            }
        }
        if let Some(d) = &mut self.draft {
            ui.label(format!(
                "Staged parent {} · {}",
                sim.universe
                    .systems
                    .get(d.parent.system as usize)
                    .and_then(|s| s.bodies.get(d.parent.body as usize))
                    .map(|b| b.name.as_str())
                    .unwrap_or("Unavailable"),
                d.plan.definition.name
            ));
            if ui
                .button("Copy current design and burn plan into staged flight")
                .clicked()
            {
                d.plan = plan.clone();
            }
            ui.label("Relative position (m) and velocity (m/s) in non-rotating system axes; parent translation and velocity are added at creation.");
            for (label, vec) in [
                ("Relative position", &mut d.initial_relative_position_m),
                ("Relative velocity", &mut d.initial_relative_velocity_m_s),
            ] {
                ui.horizontal(|ui| {
                    ui.label(label);
                    for (axis, v) in [("X", &mut vec.x), ("Y", &mut vec.y), ("Z", &mut vec.z)] {
                        ui.label(axis);
                        ui.add(egui::DragValue::new(v).speed(1.).max_decimals(9));
                    }
                });
            }
            let mut terrain_contact=d.contact.is_some();
            if ui.checkbox(&mut terrain_contact,"Use terrain and rigid-body contact model").changed() {
                if terrain_contact {
                    let centre=sim.universe.systems.get(d.parent.system as usize).and_then(|s|s.bodies.get(d.parent.body as usize).map(|_|s.body_state(d.parent.body as usize,sim.universe.time).pos));
                    match centre.ok_or_else(||"Starting body is unavailable".to_string()).and_then(|p|cosmogon_sim::vehicle_contact_dynamics::Hardware::example(&sim.universe,d.parent,p+d.initial_relative_position_m)) {
                        Ok(h)=>{d.contact=Some(h);d.model=cosmogon_sim::world_vehicle::CONTACT_MODEL.into();},
                        Err(e)=>self.status=Some(e),
                    }
                } else {d.contact=None;d.model=cosmogon_sim::world_vehicle::MODEL.into();}
            }
            if let Some(h)=&mut d.contact {hardware_controls(ui,h);d.model=h.flight_model().into();}
            let valid = Flight::new(0,d.clone(),&sim.universe);
            if let Err(e) = &valid {
                ui.colored_label(ACCENT, e);
            }
            if ui
                .add_enabled(
                    !sim.is_reference() && valid.is_ok(),
                    egui::Button::new("Create finite craft in this world"),
                )
                .clicked()
            {
                match sim.edit(
                    Edit::CreateVehicle {
                        definition: Box::new(d.clone()),
                    },
                    now,
                ) {
                    Ok(_) => {
                        self.selected_vehicle = sim.universe.vehicles.last().map(|v| v.id);
                        self.status=Some("Finite craft created through an undoable world edit; ordinary simulation time drives its flight.".into());
                    }
                    Err(e) => self.status = Some(e),
                }
            }
        }
        ui.separator();
        if self
            .selected_vehicle
            .is_none_or(|id| !sim.universe.vehicles.iter().any(|v| v.id == id))
        {
            self.selected_vehicle = sim.universe.vehicles.first().map(|v| v.id);
        }
        egui::ComboBox::from_id_salt("live-finite-vehicle")
            .selected_text(
                self.selected_vehicle
                    .and_then(|id| sim.universe.vehicles.iter().find(|v| v.id == id))
                    .map(|v| format!("{} · #{}", v.definition.plan.definition.name, v.id))
                    .unwrap_or("No finite craft in this world".into()),
            )
            .show_ui(ui, |ui| {
                for v in &sim.universe.vehicles {
                    ui.selectable_value(
                        &mut self.selected_vehicle,
                        Some(v.id),
                        format!(
                            "{} · #{} · {}",
                            v.definition.plan.definition.name,
                            v.id,
                            recording::phase_key(&v.phase)
                        ),
                    );
                }
            });
        if let Some(flight) = self
            .selected_vehicle
            .and_then(|id| sim.universe.vehicles.iter().find(|v| v.id == id))
            .cloned()
        {
            readings(ui, &flight, sim.universe.time);
            self.live_observation = Some(recording::Observation {
                world_time_s: sim.universe.time,
                flight: flight.clone(),
            });
            if ui
                .add_enabled(
                    !sim.is_reference(),
                    egui::Button::new("Remove this live craft"),
                )
                .clicked()
            {
                match sim.edit(Edit::RemoveVehicle {system:flight.definition.parent.system,id:flight.id},now) {Ok(_)=>self.status=Some("Live craft removed through an undoable world edit; recorded studies are preserved.".into()),Err(e)=>self.status=Some(e)}
            }
            if flight.cargo.is_some() {
                ui.horizontal(|ui| {
                    ui.label("Remove cargo (potable water, kg)");
                    ui.add(egui::DragValue::new(&mut self.cargo_removal_kg).range(0.0..=1e6).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Ejection velocity relative to craft (m/s)");
                    for axis in 0..3 {
                        ui.add(
                            egui::DragValue::new(&mut self.cargo_removal_relative[axis])
                                .range(-1000.0..=1000.0)
                                .speed(0.1),
                        );
                    }
                });
                if ui
                    .add_enabled(
                        !sim.is_reference(),
                        egui::Button::new("Remove cargo from this craft"),
                    )
                    .clicked()
                {
                    let requested = Cargo {
                        oxygen_reserve_kg: 0.,
                        potable_water_kg: self.cargo_removal_kg,
                        food_dry_kg: 0.,
                        food_water_kg: 0.,
                    };
                    match sim.edit(
                        Edit::RemoveVehicleCargo {
                            system: flight.definition.parent.system,
                            id: flight.id,
                            requested,
                            velocity_relative_m_s: self.cargo_removal_relative,
                        },
                        now,
                    ) {
                        Ok(_) => self.status = Some(
                            "Cargo removed through an undoable world edit; momentum and ledger evidence retained.".into(),
                        ),
                        Err(e) => self.status = Some(e),
                    }
                }
                let same_system: Vec<_> = sim.universe
                    .habitats
                    .iter()
                    .filter(|h| h.definition.body.system == flight.definition.parent.system)
                    .collect();
                if !same_system.is_empty() {
                    ui.separator();
                    ui.strong("Co-located habitat transfer");
                    ui.label("Atomic move of held cargo into finite habitat storage. Reach, rate and remaining capacity are checked; nothing is modeled if the craft is not co-located with the habitat within the handling reach.");
                    egui::ComboBox::from_id_salt("live-transfer-habitat")
                        .selected_text(
                            self.transfer_habitat
                                .and_then(|id| same_system.iter().find(|h| h.id == id))
                                .map(|h| format!("Habitat #{} · {}", h.id, h.definition.name))
                                .unwrap_or("Select a habitat in this system".into()),
                        )
                        .show_ui(ui, |ui| {
                            for h in &same_system {
                                ui.selectable_value(
                                    &mut self.transfer_habitat,
                                    Some(h.id),
                                    format!("Habitat #{} · {}", h.id, h.definition.name),
                                );
                            }
                        });
                    for (label, index) in [
                        ("Oxygen reserve (kg)", 0),
                        ("Potable water (kg)", 1),
                        ("Food dry (kg)", 2),
                        ("Food water (kg)", 3),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(label);
                            ui.add(egui::DragValue::new(&mut self.transfer_cargo[index]).range(0.0..=1e6).speed(1.0));
                        });
                    }
                    if let Some(status) = &self.transfer_status { ui.label(status.clone()); }
                    let requested = Cargo {
                        oxygen_reserve_kg: self.transfer_cargo[0],
                        potable_water_kg: self.transfer_cargo[1],
                        food_dry_kg: self.transfer_cargo[2],
                        food_water_kg: self.transfer_cargo[3],
                    };
                    if ui
                        .add_enabled(
                            !sim.is_reference() && self.transfer_habitat.is_some(),
                            egui::Button::new("Deliver cargo into this habitat"),
                        )
                        .clicked()
                    {
                        match sim.edit(
                            Edit::TransferVehicleCargoToHabitat {
                                system: flight.definition.parent.system,
                                vehicle_id: flight.id,
                                habitat_id: self.transfer_habitat.expect("enabled habitat"),
                                requested,
                            },
                            now,
                        ) {
                            Ok(_) => {
                                self.status = Some(
                                    "Cargo delivered through an undoable world edit; craft and habitat ledgers record the atomic transfer.".into(),
                                );
                                self.transfer_status = Some(
                                    "Transfer accepted; habitat stocks and receipts updated.".into(),
                                );
                            }
                            Err(e) => {
                                self.status = Some(e.clone());
                                self.transfer_status = Some(e);
                            }
                        }
                    }
                    if !flight.transfers.is_empty() {
                        ui.label(format!(
                            "Retained transfer records on this craft: {}",
                            flight.transfers.len()
                        ));
                        if let Some(t) = flight.transfers.last() {
                            ui.label(format!(
                                "Last transfer at {:.3} s world time · requested {:.6} kg → accepted {:.6} kg · trimmed {:.6} kg",
                                t.world_time_s,
                                t.requested.mass_kg(),
                                t.accepted.mass_kg(),
                                t.rejected.mass_kg(),
                            ));
                        }
                    }
                }
                if let Some(limits)=&flight.definition.launch {
                    ui.separator();
                    ui.strong("Launch limits and trajectory evidence");
                    ui.label(format!(
                        "Structural specific-force bound {:.3} m/s² · liftoff above start +{:.3} m · surface start within {:.3} m",
                        limits.maximum_acceleration_m_s2, limits.liftoff_altitude_m, limits.maximum_start_altitude_m
                    ));
                    if let Some(ledger)=&flight.launch_ledger {
                        ui.label(format!(
                            "Peak acceleration {:.3} m/s² · propellant consumed {:.3} kg · substeps {}",
                            ledger.peak_acceleration_m_s2, ledger.propellant_consumed_kg, ledger.substeps_observed
                        ));
                        match (ledger.liftoff_elapsed_s, ledger.burnout_elapsed_s) {
                            (Some(l), Some(b)) => ui.label(format!(
                                "Liftoff at {l:.3} s · burnout at {b:.3} s · apogee {:.3} m above local terrain",
                                ledger.apogee_altitude_m
                            )),
                            (Some(l), None) => ui.label(format!(
                                "Liftoff at {l:.3} s · still powered or no burnout yet · apogee {:.3} m above local terrain",
                                ledger.apogee_altitude_m
                            )),
                            _ => ui.label("On the pad; no liftoff evidence yet."),
                        };
                    }
                    ui.label(egui::RichText::new("Evidence is accepted-substep geometry on the propulsion clock; no atmospheric ascent losses or attitude controller are modeled.").small().color(MUTED));
                }
            }
        }
        ui.label(egui::RichText::new("Live state is saved with the sandbox. Active flight resolves one-second tasks; high time warp may lag. Use a separate recorded branch for controlled studies.").small().color(MUTED));
    }
    fn study_controls(&mut self, ui: &mut egui::Ui, world: &Universe) {
        ui.horizontal(|ui| {
            ui.label("Observations");
            ui.add(egui::DragValue::new(&mut self.samples).range(2..=4096));
            ui.label("Interval (s)");
            ui.add(
                egui::DragValue::new(&mut self.interval_s)
                    .range(0.001..=86400.)
                    .speed(0.1),
            );
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.pending() && self.draft.is_some(),
                    egui::Button::new("Study from current live world"),
                )
                .clicked()
            {
                self.start(world, false);
            }
            if ui
                .add_enabled(
                    !self.pending() && self.result.is_some() && self.draft.is_some(),
                    egui::Button::new("Study from original recorded world"),
                )
                .clicked()
            {
                self.start(world, true);
            }
        });
        ui.label("Creates a new craft in an independent branch. Draft controls define the new study; the live world and original results retain their own state.");
    }
    fn recorded(&mut self, ui: &mut egui::Ui, world: &Universe) {
        if self.result.is_none() {
            self.study_controls(ui, world);
            ui.label("Choose initial conditions in World flight, then run a branch or reopen a complete flight recording.");
            return;
        }
        let bundle = self.result.as_ref().expect("checked completed result");
        ui.label(format!(
            "{} recorded samples · {}",
            bundle.rows.len(),
            bundle.study.flight.plan.definition.name
        ));
        ui.add(
            egui::Slider::new(&mut self.selected_row, 0..=bundle.rows.len() - 1)
                .text("Observation"),
        );
        let row = &bundle.rows[self.selected_row];
        readings(ui, &row.flight, row.world_time_s);
        ui.horizontal_wrapped(|ui| {
            for (i, label) in [
                "Held fuel",
                "System-frame speed",
                "Applied ideal capability",
                "Inertial X displacement",
            ]
            .iter()
            .enumerate()
            {
                ui.selectable_value(&mut self.metric, i, *label);
            }
        });
        if bundle.study.flight.contact.is_some() {ui.horizontal_wrapped(|ui| {
            for (index,label) in [(4,"Loaded feet"),(5,"Normal spring energy"),(6,"Normal and smooth-friction loss"),(7,"Body angular speed"),(8,"Cumulative contact residual")] {ui.selectable_value(&mut self.metric,index,label);}
        });} else if self.metric>3 {self.metric=0;}
        if bundle.study.flight.contact.as_ref().is_some_and(|h|h.shear.is_some()) {ui.horizontal_wrapped(|ui| {
            for (index,label) in [(9,"Tangential stored energy"),(10,"Plastic slip loss"),(11,"Unloading release"),(12,"Endpoint numerical term"),(13,"Endpoint shear residual"),(14,"Accumulated slip distance")] {ui.selectable_value(&mut self.metric,index,label);}
        });} else if self.metric>8 {self.metric=0;}
        plot(ui, bundle, self.metric, self.selected_row);
        ui.collapsing("Retained source and clock support",|ui|{
            ui.add(egui::Label::new(format!("Source: {} · origin {}",self.source,bundle.manifest["origin_fingerprint"])).wrap());
            ui.label("Graph points use actual world sample times. Craft values are from the last completed task or retained terminal event; last-substep values are not whole-span transfers.");
        });
        ui.collapsing("New flight study", |ui| self.study_controls(ui, world));
    }
    fn files(&mut self, ui: &mut egui::Ui, world: &Universe) {
        ui.strong("Protected full-world recordings");
        ui.label("Complete recording folder");
        ui.text_edit_singleline(&mut self.input);
        if ui
            .add_enabled(
                !self.pending() && !self.input.trim().is_empty(),
                egui::Button::new("Verify and reopen flight recording"),
            )
            .clicked()
        {
            self.open(self.input.trim().into());
        }
        ui.label("New output folder");
        ui.text_edit_singleline(&mut self.output);
        if ui
            .add_enabled(
                !self.pending() && self.result.is_some() && !self.output.trim().is_empty(),
                egui::Button::new("Export completed flight evidence"),
            )
            .clicked()
        {
            self.export(self.output.trim().into());
        }
        ui.label("Pinned study definition and original world (.cosmo)");
        ui.text_edit_singleline(&mut self.definition_input);
        ui.text_edit_singleline(&mut self.origin_input);
        if ui
            .add_enabled(
                !self.pending()
                    && !self.definition_input.trim().is_empty()
                    && !self.origin_input.trim().is_empty(),
                egui::Button::new("Run definition from its original world"),
            )
            .clicked()
        {
            let definition = self.definition_input.trim().to_string();
            let origin = self.origin_input.trim().to_string();
            self.request = Some(
                json!({"operation":"Run explicit pinned definition","definition":definition,"origin":origin}),
            );
            self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
                let u = recording::read_world(Path::new(&origin))?;
                let study = Study::read(Path::new(&definition))?;
                Ok((
                    Bundle::run(&u, study)?,
                    format!("Pinned {definition} and {origin}"),
                    true,
                ))
            }));
        }
        ui.label("New setup folder · definition and exact original world");
        ui.text_edit_singleline(&mut self.definition_output);
        if ui
            .add_enabled(
                !self.pending()
                    && self.draft.is_some()
                    && !self.definition_output.trim().is_empty(),
                egui::Button::new("Save staged flight and its exact starting world"),
            )
            .clicked()
        {
            self.status=Some(match Study::new(world,self.draft.clone().unwrap(),self.samples,self.interval_s).and_then(|s|s.export_setup(world,Path::new(self.definition_output.trim()))) {Ok(())=>"Protected setup folder written with definition.json and origin.cosmo; the original starting world is retained exactly.".into(),Err(e)=>e});
        }
        ui.separator();
        ui.strong("Model and source limits");
        ui.label("Newtonian test particle; primary star, companion and present-body gravity. Fixed task and source-sampling approximations are retained in every manifest.");
        ui.label("Original flight uses spherical contact. Explicit terrain mode adds authored rigid-body/gear dynamics; it does not establish safe landing or attachment. No atmosphere/entry heating, staging, docking or cargo transfer. Created cargo is an explicit initial condition; source edits stop unresolved flight.");
        for (title, url) in [
            (
                "NASA Glenn gravity",
                "https://www1.grc.nasa.gov/beginners-guide-to-aeronautics/rocket-weight/",
            ),
            (
                "JPL NAIF frames",
                "https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/frames.html",
            ),
            (
                "Horizons state vectors",
                "https://ssd.jpl.nasa.gov/horizons/manual.html",
            ),
        ] {
            ui.hyperlink_to(title, url);
        }
        ui.collapsing("Complete model context", |ui| {
            ui.add(
                egui::Label::new(serde_json::to_string_pretty(&self.result.as_ref().map(|b|recording::context_for(&b.study.flight)).or_else(||self.draft.as_ref().map(recording::context_for)).unwrap_or_else(recording::context)).unwrap())
                    .wrap(),
            );
        });
        if let Some(bundle) = &self.result {
            ui.collapsing("Retained source and full-state identities", |ui| {
                ui.add(
                    egui::Label::new(serde_json::to_string_pretty(&bundle.manifest).unwrap())
                        .wrap(),
                );
            });
        }
    }
}
fn support_readings(ui:&mut egui::Ui,r:&SupportReading) {
    ui.label(format!("Craft {:.6} s · observed world {:.6} s · {} loaded feet · mass {:.6} kg",r.craft_time_s,r.world_time_s,r.feet.iter().filter(|f|f.loaded).count(),r.mass_kg));
    ui.colored_label(ACCENT,if r.within_declared_instantaneous_envelope {"Within authored instantaneous envelope; future stability, attachment and cargo access remain unproven."}else{"Outside authored instantaneous envelope; contact continues evolving."});
    if let Some(p)=&r.planar {
        support_footprint(ui,p,r.hold.as_ref().and_then(|a|a.load_line_uv_m));
    }
    ui.label(format!("Ground-relative speed {:.6} m/s · angular speed {:.9} rad/s",r.relative_velocity_m_s.length(),r.relative_angular_velocity_rad_s.length()));
    ui.label(format!("Actual reaction {:.6} N · torque about centre {:.6} N·m",r.contact_force_n.length(),r.contact_torque_about_centre_n_m.length()));
    if let Some(p)=&r.planar {
        ui.label(format!("Loaded footprint {:.6} m² · geometric centre margin {:+.6} m",p.hull_area_m2,p.geometric_centre_margin_m));
        ui.label(format!("Force-weighted site margin {:+.6} m · actual wrench margin {:+.6} m",p.force_weighted_site_margin_m,p.wrench_centre_margin_m));
        ui.label(format!("Ground-plane departure {:.6} m · normal deviation {:.6} rad · coordinate indicator {:.6} m",p.maximum_plane_departure_m,p.maximum_normal_deviation_rad,r.position_roundoff_indicator_m));
    }else{ui.colored_label(ACCENT,r.planar_unavailable_reason.as_deref().unwrap_or("Planar footprint unavailable"));}
    if let Some(a)=&r.hold {
        ui.label(format!("Required hold-line margin {} · force imbalance {:.6} N · torque imbalance {:.6} N·m",a.load_line_margin_m.map(|m|format!("{m:+.6} m")).unwrap_or_else(||"unavailable".into()),a.force_imbalance_n.length(),a.torque_imbalance_n_m.length()));
        ui.label(format!("Ground-relative acceleration {:.6} m/s² · derivative error indicator {:.9} m/s²",a.relative_acceleration_m_s2.length(),a.derivative_error_indicator_m_s2));
    }else{ui.colored_label(ACCENT,r.hold_unavailable_reason.as_deref().unwrap_or("Hold comparison unavailable"));}
    for reason in &r.envelope_exclusions {ui.add(egui::Label::new(reason).wrap());}
    ui.collapsing("Individual loads and complete numeric support reading",|ui|{ui.add(egui::Label::new(serde_json::to_string_pretty(r).unwrap()).wrap());});
}
fn support_footprint(ui:&mut egui::Ui,p:&cosmogon_sim::vehicle_support::Planar,line:Option<[f64;2]>) {
    let centre=[-p.origin_from_centre_m.dot(p.axis_u),-p.origin_from_centre_m.dot(p.axis_v)];
    let markers=[(Some(centre),"Geometric centre",egui::Color32::WHITE),(Some(p.force_weighted_site_uv_m),"Weighted sites",egui::Color32::from_rgb(100,190,220)),(Some(p.wrench_centre_uv_m),"Applied wrench",ACCENT),(line,"Required hold line",egui::Color32::from_rgb(220,120,170))];
    let values=p.hull_uv_m.iter().copied().chain(markers.iter().filter_map(|(v,_,_)|*v)).collect::<Vec<_>>();
    let minx=values.iter().map(|v|v[0]).fold(f64::INFINITY,f64::min);let maxx=values.iter().map(|v|v[0]).fold(f64::NEG_INFINITY,f64::max);let miny=values.iter().map(|v|v[1]).fold(f64::INFINITY,f64::min);let maxy=values.iter().map(|v|v[1]).fold(f64::NEG_INFINITY,f64::max);let span=(maxx-minx).max(maxy-miny).max(0.01);
    let (rect,_)=ui.allocate_exact_size(egui::vec2(ui.available_width().min(500.),180.),egui::Sense::hover());let scale=(rect.width().min(rect.height())-20.) as f64/span;let mid=[(minx+maxx)*0.5,(miny+maxy)*0.5];
    let map=|v:[f64;2]|egui::pos2(rect.center().x+((v[0]-mid[0])*scale)as f32,rect.center().y-((v[1]-mid[1])*scale)as f32);
    let points=p.hull_uv_m.iter().copied().map(map).collect::<Vec<_>>();ui.painter().add(egui::Shape::convex_polygon(points.clone(),egui::Color32::from_rgba_unmultiplied(200,155,60,20),egui::Stroke::new(1.5_f32,ACCENT)));
    for point in points {ui.painter().circle_filled(point,3.,ACCENT);}
    for (value,_,color) in &markers {if let Some(v)=value {ui.painter().circle_stroke(map(*v),4.,egui::Stroke::new(2.0_f32,*color));}}
    ui.horizontal_wrapped(|ui|{for(_,label,color)in markers {ui.colored_label(color,label);}});
    ui.label("Declared ground-plane coordinates, equal scale; positive margins mean inside this footprint only.");
}
fn hardware_controls(ui:&mut egui::Ui,h:&mut cosmogon_sim::vehicle_contact_dynamics::Hardware) {
    let mut persistent=h.shear.is_some();
    if ui.checkbox(&mut persistent,"Persistent foot shear (replaces smooth sliding friction)").changed() {
        if persistent {h.model=cosmogon_sim::vehicle_contact_dynamics::SHEAR_MODEL.into();h.shear=Some(cosmogon_sim::vehicle_shear::Definition::example());for f in &mut h.feet {f.response.sliding_friction=0.;}}
        else {h.model=cosmogon_sim::vehicle_contact::MODEL.into();h.shear=None;}
    }
    ui.collapsing("Contact hardware assumptions",|ui| {
        ui.label("Editable authored mechanics. Inertia scales with retained mass; symmetric fuel removal carries co-rotating angular momentum. Central thrust uses inertial command axes.");
        if let Some(s)=&mut h.shear {
            ui.label("A foot retains elastic shear at rest and slips beyond its finite friction capacity. This does not establish stable landing or attachment.");
            for (label,value) in [("Tangential stiffness (N/m)",&mut s.stiffness_n_m),("Coulomb coefficient",&mut s.friction_coefficient),("Elastic shear limit (m)",&mut s.maximum_elastic_shear_m),("Tangential structural limit (N)",&mut s.maximum_tangential_force_n)] {
                ui.horizontal_wrapped(|ui| {ui.label(label);ui.add(egui::DragValue::new(value).speed(0.01).max_decimals(9));});
            }
        }
        for (label,v) in [("Gyration radii (m)",&mut h.gyration_radii_m),("Initial body angular velocity (rad/s)",&mut h.initial_angular_velocity_body_rad_s)] {
            ui.label(label);ui.horizontal_wrapped(|ui| {for (axis,value) in [("X",&mut v.x),("Y",&mut v.y),("Z",&mut v.z)] {ui.label(axis);ui.add(egui::DragValue::new(value).speed(0.01).max_decimals(9));}});
        }
        ui.label("Unit Hamilton orientation (body axes to inertial axes; X Y Z W)");
        ui.horizontal_wrapped(|ui| {for value in &mut h.initial_orientation_xyzw {ui.add(egui::DragValue::new(value).speed(0.001).max_decimals(9));}});
        if ui.button("Normalize authored orientation").clicked() {
            let norm=h.initial_orientation_xyzw.iter().map(|v|v*v).sum::<f64>().sqrt();
            if norm.is_finite() && norm>0. {for v in &mut h.initial_orientation_xyzw {*v/=norm;}}
        }
        for (label,value) in [("Spherical hull radius (m)",&mut h.hull_radius_m),("Terrain normal probe (m)",&mut h.normal_probe_m),("Maximum step (s)",&mut h.maximum_step_s)] {
            ui.horizontal_wrapped(|ui| {ui.label(label);ui.add(egui::DragValue::new(value).speed(0.001).max_decimals(9));});
        }
        for (index,foot) in h.feet.iter_mut().enumerate() {ui.collapsing(format!("Foot {}",index+1),|ui| {
            ui.label("Body offset (m)");ui.horizontal_wrapped(|ui| {for (axis,value) in [("X",&mut foot.offset_body_m.x),("Y",&mut foot.offset_body_m.y),("Z",&mut foot.offset_body_m.z)] {ui.label(axis);ui.add(egui::DragValue::new(value).speed(0.01));}});
            for (label,value) in [("Stiffness (N/m)",&mut foot.response.stiffness_n_m),("Damping (N·s/m)",&mut foot.response.damping_n_s_m),("Maximum stroke (m)",&mut foot.response.maximum_stroke_m),("Resultant load limit (N)",&mut foot.response.maximum_load_n),("Sliding friction coefficient",&mut foot.response.sliding_friction),("Slip smoothing speed (m/s)",&mut foot.response.slip_speed_scale_m_s)] {
                ui.horizontal_wrapped(|ui| {ui.label(label);ui.add(egui::DragValue::new(value).speed(0.01).max_decimals(9));});
            }
        });}
        ui.label("Attachment, stability certification, entry/heating and physical cargo delivery remain unavailable. Persistent shear requires zero legacy sliding coefficient; disabling it leaves that coefficient editable. Study work is bounded; use a short recording first.");
    });
}
fn arrival_readings(ui: &mut egui::Ui, r: &Approach) {
    ui.label(format!(
        "Craft {:.6} s · frame observed at world {:.6} s",
        r.craft_time_s, r.world_time_s
    ));
    ui.label(format!(
        "Latitude {:.7}° · east longitude {}",
        r.latitude_deg,
        r.longitude_east_deg
            .map(|x| format!("{x:.7}°"))
            .unwrap_or_else(|| "undefined at pole".into())
    ));
    ui.label(format!(
        "Above sphere {:.6} m · overview terrain clearance {}",
        r.spherical_altitude_m,
        r.terrain_clearance_m
            .map(|x| format!("{x:.6} m"))
            .unwrap_or_else(|| "unavailable at pole".into())
    ));
    ui.label(format!(
        "Up velocity {:+.6} m/s · horizontal {:.6} m/s · ground-relative speed {:.6} m/s",
        r.radial_velocity_m_s, r.horizontal_speed_m_s, r.surface_relative_speed_m_s
    ));
    ui.label(format!(
        "Actual mass {:.6} kg · radial kinetic energy {:.6} J · horizontal {:.6} J",
        r.current_mass_kg, r.radial_kinetic_energy_j, r.horizontal_kinetic_energy_j
    ));
    ui.colored_label(
        ACCENT,
        "Spherical contact does not establish a terrain landing, attachment or cargo delivery.",
    );
    ui.collapsing("Complete rotating frame and terrain support", |ui| {
        ui.add(egui::Label::new(serde_json::to_string_pretty(r).unwrap()).wrap());
    });
}
fn readings(ui: &mut egui::Ui, f: &Flight, world_time: f64) {
    let p = &f.propulsion;
    let d = &f.definition.plan.definition;
    ui.strong(format!(
        "{} · world {:.6} s · craft {:.6} s · elapsed {:.9} s",
        recording::phase_key(&f.phase),
        world_time,
        f.last_time_s,
        p.elapsed_s
    ));
    match &f.phase {
        Phase::Impact {
            source,
            position_residual_m,
            tolerance_m,
            ..
        } => {
            ui.colored_label(ACCENT,format!("First spherical contact with {source:?} · raw residual {position_residual_m} m · tolerance {tolerance_m} m. No safe landing or delivery."));
        }
        Phase::Unsupported { reason } => {
            ui.colored_label(ACCENT, format!("Stopped at last supported state: {reason}"));
        }
        Phase::Flying => {}
    }
    ui.label(format!(
        "Mass {:.9} kg · held fuel {:.9} kg · cargo {:.9} kg{}",
        f.mass_kg(),
        p.propellant_kg,
        match &f.cargo {
            Some(ledger) => ledger.held.mass_kg(),
            None => d.cargo.mass_kg(),
        },
        if f.cargo.is_some() {
            format!(" · ledger events {}", f.cargo_events.len())
        } else {
            String::new()
        }
    ));
    ui.label(format!(
        "Removed fuel {:.9} kg · applied ideal capability {:.9} m/s",
        p.exhausted_propellant_kg, p.cumulative_ideal_delta_v_m_s
    ));
    ui.label(format!(
        "Powered {:.9} s · fuel-limited command {:.9} s",
        p.powered_duration_s, p.fuel_limited_duration_s
    ));
    if let Some(r)=&f.contact {
        let count=r.feet.iter().filter(|foot|foot.response.normal_load_n>0.).count();
        ui.strong(format!("Terrain contact · {count}/{} loaded feet · orientation and motion are integrated",r.feet.len()));
        ui.label(format!("Peak compression {:.6} m · peak resultant load {:.3} N",r.last_work.maximum_compression_m,r.last_work.maximum_resultant_load_n));
        ui.label(format!("Body angular speed {:.6} rad/s · {} resolved substeps in {:.6} s",r.angular_velocity_body_rad_s.length(),r.last_work.substeps,r.last_work.duration_s));
        ui.label(format!("Contact balance residual {:.6} J · accumulated {:.6} J",r.last_work.contact_balance_residual_j,r.cumulative_work.contact_balance_residual_j));
        if let Some(s)=&r.cumulative_work.shear {
            let storage=f.definition.contact.as_ref().and_then(|h|h.shear.as_ref()).and_then(|d|r.feet.iter().map(|foot|d.energy_j(foot.shear.as_ref()? ).ok()).collect::<Option<Vec<_>>>()).map(|v|v.iter().sum::<f64>());
            if let Some(storage)=storage {ui.label(format!("Persistent shear · stored {:.6} J · slip loss {:.6} J · unloading release {:.6} J",storage,s.plastic_dissipation_j,s.unloading_release_j));}
            else {ui.colored_label(ACCENT,"Retained tangential storage is unavailable");}
            ui.label(format!("Endpoint numerical term {:.6} J · constitutive residual {:.6} J",s.numerical_dissipation_j,s.balance_residual_j));
            ui.label("Authored persistent shear. Endpoint work and actual midpoint motion have separate balances; numerical terms are not measured friction. Stability, attachment and cargo delivery remain unavailable.");
        } else {ui.label("Authored finite gear response with sliding friction. No static attachment, safe landing certification or cargo delivery.");}
        ui.collapsing("Per-foot forces, rotating terrain and full work evidence",|ui| {
            for (i,foot) in r.feet.iter().enumerate() {
                ui.label(format!("Foot {} · gap {:.6} m · compression {:.6} m · normal {:.3} N · slope {:.3}°",i+1,foot.surface.tangent_gap_m,foot.response.compression_m,foot.response.normal_load_n,foot.surface.slope_rad.to_degrees()));
            }
            ui.add(egui::Label::new(serde_json::to_string_pretty(r).unwrap()).wrap());
        });
    }
    if let Some(h)=&f.definition.handling {
        ui.strong("Finite surface handling equipment");
        ui.label(format!("Species limit {:.0} kg · total {:.0} kg · shared O₂ interface {:.0} kg · required window {:.3} s",h.per_species_limit_kg,h.total_limit_kg,h.oxygen_interface_limit_kg,h.required_operation_window_s));
        ui.label(format!("Declared draw {:.0} W · thermal rejection {:.0} W (assumptions; no craft power bus or thermal integration) · current supported window {}",h.power_w,h.thermal_rejection_w,f.operation_ledger.as_ref().map(|l|format!("{:.3} s",l.current_window_s)).unwrap_or("none".into())));
        ui.label("Handling is authorize-only: this build never transfers, receives or delivers cargo.");
    }
    ui.collapsing("Position, velocity and complete numeric readings",|ui|{
        ui.label(format!("System-local position [{:.6}, {:.6}, {:.6}] m",f.position_m.x,f.position_m.y,f.position_m.z));
        ui.label(format!("System-frame velocity [{:.9}, {:.9}, {:.9}] m/s",f.velocity_m_s.x,f.velocity_m_s.y,f.velocity_m_s.z));
        ui.label(format!("Impulse {:.9} N·s · applied thrust vector [{:.9}, {:.9}, {:.9}] m/s",p.cumulative_impulse_n_s,p.velocity_change_m_s[0],p.velocity_change_m_s[1],p.velocity_change_m_s[2]));
        ui.label(format!("Raw fuel closure {} kg",d.initial_propellant_kg-p.propellant_kg-p.exhausted_propellant_kg));
        if let Phase::Impact {radius_m,source_sample_duration_s,..}=&f.phase {ui.label(format!("Contact spherical radius {radius_m} m · source sample {source_sample_duration_s} s"));}
        ui.label(format!("{} actual gravity source samples at the retained craft clock",f.last_sources.len()));
    });
}
fn plot(ui: &mut egui::Ui, b: &Bundle, metric: usize, selected: usize) {
    let values = b
        .rows
        .iter()
        .map(|r| match metric {
            0 => r.flight.propulsion.propellant_kg,
            1 => r.flight.velocity_m_s.length(),
            2 => r.flight.propulsion.cumulative_ideal_delta_v_m_s,
            3 => r.flight.position_m.x - b.rows[0].flight.position_m.x,
            4 => r.flight.contact.as_ref().map(|s|s.feet.iter().filter(|f|f.response.normal_load_n>0.).count() as f64).unwrap_or(0.),
            5 => r.flight.contact.as_ref().map(|s|s.feet.iter().map(|f|f.response.stored_elastic_energy_j).sum()).unwrap_or(0.),
            6 => r.flight.contact.as_ref().map(|s|s.cumulative_work.normal_dissipation_j+s.cumulative_work.sliding_dissipation_j).unwrap_or(0.),
            7 => r.flight.contact.as_ref().map(|s|s.angular_velocity_body_rad_s.length()).unwrap_or(0.),
            9..=14 => r.flight.contact.as_ref().and_then(|s|s.cumulative_work.shear.as_ref()).map(|w|match metric {
                9=>w.elastic_storage_change_j,10=>w.plastic_dissipation_j,11=>w.unloading_release_j,12=>w.numerical_dissipation_j,13=>w.balance_residual_j,_=>w.plastic_slip_distance_m,
            }).unwrap_or(0.),
            _ => r.flight.contact.as_ref().map(|s|s.cumulative_work.contact_balance_residual_j).unwrap_or(0.),
        })
        .collect::<Vec<_>>();
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(max.abs().max(1.) * 1e-9);
    let duration = (b.rows.last().unwrap().world_time_s - b.rows[0].world_time_s).max(1e-9);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(120.), 130.),
        egui::Sense::hover(),
    );
    let bounds = rect.shrink2(egui::vec2(8., 10.));
    let points = b
        .rows
        .iter()
        .zip(&values)
        .map(|(r, v)| {
            egui::pos2(
                bounds.left()
                    + ((r.world_time_s - b.rows[0].world_time_s) / duration) as f32
                        * bounds.width(),
                bounds.bottom() - ((*v - min) / span) as f32 * bounds.height(),
            )
        })
        .collect::<Vec<_>>();
    ui.painter().line_segment(
        [bounds.left_bottom(), bounds.right_bottom()],
        egui::Stroke::new(1.0_f32, MUTED),
    );
    ui.painter().add(egui::Shape::line(
        points.clone(),
        egui::Stroke::new(1.5_f32, ACCENT),
    ));
    ui.painter()
        .circle_filled(points[selected], 3., egui::Color32::WHITE);
    ui.label(format!(
        "World elapsed 0–{duration:.6} s · raw range {min:.9}–{max:.9} {}",
        match metric {0=>"kg",3|14=>"m",4=>"feet",5|6|8|9..=13=>"J",7=>"rad/s",_=>"m/s"}
    ));
}
pub fn preflight(args: &crate::args::Args) -> Result<(), String> {
    if let Some(path)=&args.support_definition {SupportDefinition::read(Path::new(path))?;}
    if let Some(path)=&args.support_recording {let report=SupportReport::read(Path::new(path))?;if args.flight_observation.is_some_and(|i|i>=report.points.len()){return Err("Support observation is outside the actual report".into());}}
    if args.support_output.as_ref().is_some_and(|p|Path::new(p.trim()).exists()){return Err("Existing support output is protected; choose a fresh folder".into());}
    for output in [&args.flight_output,&args.arrival_output] {if args.support_output.as_ref().zip(output.as_ref()).is_some_and(|(a,b)|{let a=Path::new(a.trim());let b=Path::new(b.trim());a.starts_with(b)||b.starts_with(a)}){return Err("Support, flight and approach exports require separate non-nested folders".into());}}
    if let Some(path) = &args.flight_recording {
        let bundle = Bundle::read(Path::new(path))?;
        if (args.panel.as_deref()==Some("vehicle-support") || args.support_output.is_some()) && bundle.study.flight.contact.is_none(){return Err("Support evidence requires terrain-contact hardware".into());}
        if args.panel.as_deref() == Some("vehicle-arrival") || args.arrival_output.is_some() {
            cosmogon_sim::vehicle_arrival::SurfaceFrame::at(
                &bundle.origin,
                bundle.study.flight.parent,
                bundle.origin.time,
            )?;
        }
        if args
            .flight_observation
            .is_some_and(|i| i >= bundle.rows.len())
        {
            return Err("Flight observation is outside the actual recording".into());
        }
    } else if let Some(path) = &args.flight_definition {
        let study = Study::read(Path::new(path))?;
        let world = recording::read_world(Path::new(
            args.flight_origin
                .as_deref()
                .ok_or("Flight definition requires its original world")?,
        ))?;
        study.validate(&world)?;
        if (args.panel.as_deref()==Some("vehicle-support") || args.support_output.is_some()) && study.flight.contact.is_none(){return Err("Support evidence requires terrain-contact hardware".into());}
        if args.panel.as_deref() == Some("vehicle-arrival") || args.arrival_output.is_some() {
            cosmogon_sim::vehicle_arrival::SurfaceFrame::at(
                &world,
                study.flight.parent,
                world.time,
            )?;
        }
        if args.flight_observation.is_some_and(|i| i >= study.samples) {
            return Err("Flight observation is outside the actual definition".into());
        }
    }
    if args
        .flight_output
        .as_ref()
        .is_some_and(|p| Path::new(p.trim()).exists())
    {
        return Err("Existing flight output is protected; choose a new folder".into());
    }
    if args
        .arrival_output
        .as_ref()
        .is_some_and(|p| Path::new(p.trim()).exists())
    {
        return Err("Existing approach output is protected; choose a new folder".into());
    }
    if args
        .arrival_output
        .as_ref()
        .zip(args.flight_output.as_ref())
        .is_some_and(|(a, b)| Path::new(a.trim()) == Path::new(b.trim()))
    {
        return Err("Flight and approach exports need separate fresh folders".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Universe, Definition) {
        let u = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let d = Definition::circular_example(
            &u,
            cosmogon_sim::BodyRef { system: 0, body: 3 },
            Plan::example(),
        )
        .unwrap();
        (u, d)
    }
    fn finish(form: &mut FlightUi) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while form.pending() {
            form.poll();
            assert!(
                std::time::Instant::now() < deadline,
                "Flight worker did not finish"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[test]
    fn support_worker_keeps_request_limits_and_reopens_exact_protected_report() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let (world,d)=setup();let centre=world.systems[0].body_state(d.parent.body as usize,world.time).pos;
        let h=cosmogon_sim::vehicle_contact_dynamics::Hardware::example(&world,d.parent,centre+d.initial_relative_position_m).unwrap().with_shear(cosmogon_sim::vehicle_shear::Definition::example()).unwrap();let d=d.with_contact(&world,h).unwrap();
        let b=Bundle::run(&world,Study::new(&world,d,3,1.).unwrap()).unwrap();let path=std::env::temp_dir().join(format!("cosmogon-native-support-{}",std::process::id()));let _=std::fs::remove_dir_all(&path);
        let mut form=FlightUi {result:Some(b.clone()),support_output:path.to_string_lossy().into(),..Default::default()};let definition=form.support_draft.clone();form.start_support();form.support_draft.minimum_load_margin_m=2.;finish(&mut form);let report=form.support_result.as_ref().unwrap();assert_eq!(report.definition,definition);assert_eq!(form.support_draft.minimum_load_margin_m,2.);assert_eq!(report.flight.manifest,b.manifest);assert_eq!(SupportReport::read(&path).unwrap().manifest,report.manifest);
        form.support_output.clear();form.open_support(path.to_string_lossy().into());finish(&mut form);assert_eq!(form.support_draft,definition);assert_eq!(form.result.as_ref().unwrap().manifest,b.manifest);
        let args=crate::args::Args {support_output:Some(path.to_string_lossy().into()),..Default::default()};assert!(preflight(&args).is_err());std::fs::write(path.join("support.csv"),"changed").unwrap();assert!(SupportReport::read(&path).is_err());std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn contact_worker_pins_hardware_and_exports_exact_new_numeric_evidence() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let (world,d)=setup();
        let centre=world.systems[0].body_state(d.parent.body as usize,world.time).pos;
        let h=cosmogon_sim::vehicle_contact_dynamics::Hardware::example(&world,d.parent,centre+d.initial_relative_position_m).unwrap();
        let d=d.with_contact(&world,h).unwrap();
        let mut form=FlightUi {draft:Some(d.clone()),samples:3,interval_s:1.,..Default::default()};
        form.start(&world,false);form.draft.as_mut().unwrap().contact.as_mut().unwrap().hull_radius_m=1.;
        finish(&mut form);
        let original=form.result.as_ref().unwrap().clone();assert_eq!(original.study.flight,d);
        assert_eq!(form.draft.as_ref().unwrap().contact.as_ref().unwrap().hull_radius_m,1.);
        assert_eq!(original.rows.last().unwrap().flight.last_time_s,world.time+2.);
        assert!(original.rows.last().unwrap().flight.contact.as_ref().unwrap().cumulative_work.substeps>0);
        let path=std::env::temp_dir().join(format!("cosmogon-native-contact-{}",std::process::id()));let _=std::fs::remove_dir_all(&path);
        original.export(&path).unwrap();assert!(path.join("contact.csv").exists());
        assert_eq!(Bundle::read(&path).unwrap().manifest,original.manifest);
        std::fs::write(path.join("contact.csv"),"changed").unwrap();assert!(Bundle::read(&path).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn persistent_shear_worker_retains_frozen_law_and_exact_protected_export() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let (world,d)=setup();let centre=world.systems[0].body_state(d.parent.body as usize,world.time).pos;
        let h=cosmogon_sim::vehicle_contact_dynamics::Hardware::example(&world,d.parent,centre+d.initial_relative_position_m).unwrap().with_shear(cosmogon_sim::vehicle_shear::Definition::example()).unwrap();
        let d=d.with_contact(&world,h).unwrap();let mut form=FlightUi{draft:Some(d.clone()),samples:3,interval_s:1.,..Default::default()};
        form.start(&world,false);form.draft.as_mut().unwrap().contact.as_mut().unwrap().shear.as_mut().unwrap().stiffness_n_m=1.;finish(&mut form);
        let original=form.result.as_ref().unwrap();assert_eq!(original.study.flight,d);assert_eq!(original.manifest["model"],cosmogon_sim::world_vehicle::SHEAR_MODEL);
        let path=std::env::temp_dir().join(format!("cosmogon-native-shear-{}",std::process::id()));let _=std::fs::remove_dir_all(&path);
        original.export(&path).unwrap();assert!(path.join("shear.csv").exists());assert!(original.export(&path).is_err());
        assert_eq!(Bundle::read(&path).unwrap().manifest,original.manifest);std::fs::write(path.join("shear.csv"),"changed").unwrap();assert!(Bundle::read(&path).is_err());std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn arrival_worker_freezes_exact_source_keeps_drafts_and_preflights_protected_output() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let (world, d) = setup();
        let bundle = Bundle::run(&world, Study::new(&world, d, 5, 1.).unwrap()).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-native-arrival-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        bundle.export(&root.join("flight")).unwrap();
        let mut form = FlightUi {
            result: Some(bundle.clone()),
            opened_directory: Some(root.join("flight").to_string_lossy().into()),
            arrival_output: root.join("arrival").to_string_lossy().into(),
            ..Default::default()
        };
        form.start_arrival();
        form.geographic_start.latitude_deg = 23.;
        finish(&mut form);
        assert_eq!(form.geographic_start.latitude_deg, 23.);
        assert_eq!(form.arrival_result.as_ref().unwrap().points.len(), 5);
        assert_eq!(
            ArrivalReport::read(&root.join("arrival")).unwrap().manifest,
            form.arrival_result.as_ref().unwrap().manifest
        );
        let mut args = crate::args::Args {
            flight_recording: Some(root.join("flight").to_string_lossy().into()),
            arrival_output: Some(root.join("arrival").to_string_lossy().into()),
            panel: Some("vehicle-arrival".into()),
            ..Default::default()
        };
        assert!(preflight(&args).is_err());
        args.arrival_output = Some(root.join("new").to_string_lossy().into());
        assert!(preflight(&args).is_ok());
        std::fs::write(root.join("flight/observations.json"), "[]").unwrap();
        form.arrival_output = root.join("refused").to_string_lossy().into();
        form.start_arrival();
        finish(&mut form);
        assert!(!root.join("refused").exists());
        assert!(form.arrival_status.as_ref().unwrap().contains("failed"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn flight_workers_pin_world_and_controls_preserve_later_drafts_and_cancel_failed_exports() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let (mut world, d) = setup();
        let root = std::env::temp_dir().join(format!(
            "cosmogon-native-world-flight-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let mut form = FlightUi {
            draft: Some(d.clone()),
            samples: 9,
            interval_s: 0.5,
            ..Default::default()
        };
        let initial = research::state_fingerprint(&world).unwrap();
        form.start(&world, false);
        world.advance_by(2.);
        form.draft
            .as_mut()
            .unwrap()
            .plan
            .definition
            .cargo
            .potable_water_kg = 0.;
        finish(&mut form);
        let original = form.result.as_ref().unwrap().clone();
        assert_eq!(original.study.flight, d);
        assert_eq!(original.study.initial_state_fingerprint, initial);
        assert_eq!(
            form.draft
                .as_ref()
                .unwrap()
                .plan
                .definition
                .cargo
                .potable_water_kg,
            0.
        );
        let source = form.source.clone();
        let draft = form.draft.clone();
        form.queued_output = Some(root.join("must-not-exist").to_string_lossy().into());
        form.open(root.join("missing").to_string_lossy().into());
        finish(&mut form);
        assert_eq!(form.result.as_ref().unwrap().manifest, original.manifest);
        assert_eq!(form.draft, draft);
        assert_eq!(form.source, source);
        assert!(!root.exists());
        original.export(&root.join("original")).unwrap();
        form.requested_row = Some(3);
        form.queued_output = Some(root.join("export").to_string_lossy().into());
        form.open(root.join("original").to_string_lossy().into());
        finish(&mut form);
        assert_eq!(form.selected_row, 3);
        assert_eq!(form.draft.as_ref().unwrap(), &d);
        let exported = Bundle::read(&root.join("export")).unwrap();
        assert_eq!(exported.manifest, original.manifest);
        for name in [
            "definition.json",
            "origin.cosmo",
            "observations.json",
            "trajectories.csv",
            "final.cosmo",
            "manifest.json",
        ] {
            assert_eq!(
                std::fs::read(root.join("original").join(name)).unwrap(),
                std::fs::read(root.join("export").join(name)).unwrap()
            );
        }
        form.requested_row = Some(4095);
        form.queued_output = Some(root.join("invalid-index").to_string_lossy().into());
        form.open(root.join("original").to_string_lossy().into());
        finish(&mut form);
        assert_eq!(form.selected_row, 3);
        assert!(!root.join("invalid-index").exists());
        // Restored controls can branch the exact original world; they do not rewind live time.
        let live = research::state_fingerprint(&world).unwrap();
        form.start(&world, true);
        finish(&mut form);
        assert_eq!(form.result.as_ref().unwrap().manifest, original.manifest);
        assert_eq!(research::state_fingerprint(&world).unwrap(), live);
        form.open(root.join("original").to_string_lossy().into());
        finish(&mut form);
        std::fs::write(
            root.join("original/trajectories.csv"),
            "changed after reopen",
        )
        .unwrap();
        form.start(&world, true);
        finish(&mut form);
        assert!(form.status.as_ref().unwrap().contains("failed"));
        assert_eq!(form.result.as_ref().unwrap().manifest, original.manifest);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn pinned_setup_and_recording_preflight_check_real_source_and_sample_bounds() {
        let (u, d) = setup();
        let root = std::env::temp_dir().join(format!(
            "cosmogon-native-flight-preflight-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let study = Study::new(&u, d, 9, 0.5).unwrap();
        study.export_setup(&u, &root.join("setup")).unwrap();
        let mut args = crate::args::Args {
            flight_definition: Some(root.join("setup/definition.json").to_string_lossy().into()),
            flight_origin: Some(root.join("setup/origin.cosmo").to_string_lossy().into()),
            flight_observation: Some(8),
            ..Default::default()
        };
        assert!(preflight(&args).is_ok());
        args.flight_observation = Some(9);
        assert!(preflight(&args).is_err());
        args.flight_observation = None;
        args.flight_output = Some(root.to_string_lossy().into());
        assert!(preflight(&args).is_err());
        let mut changed = u.clone();
        changed.advance_by(1.);
        save::write_file(&root.join("changed.cosmo"), &changed, "changed").unwrap();
        args.flight_output = None;
        args.flight_origin = Some(root.join("changed.cosmo").to_string_lossy().into());
        assert!(preflight(&args).is_err());
        Bundle::run(&u, study)
            .unwrap()
            .export(&root.join("recording"))
            .unwrap();
        args.flight_definition = None;
        args.flight_origin = None;
        args.flight_recording = Some(root.join("recording").to_string_lossy().into());
        assert!(preflight(&args).is_ok());
        std::fs::write(root.join("recording/observations.json"), "[]").unwrap();
        assert!(preflight(&args).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
