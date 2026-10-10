//! Native controls for actual world spacecraft and separate pinned flight studies.
use super::{ACCENT, MUTED};
use crate::sim::{Sim, Target};
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_egui::egui;
use cosmogon_sim::{
    research,
    sandbox::Edit,
    save,
    supply_vehicle::Plan,
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
}
impl FlightUi {
    pub fn initialize(&mut self, args: &crate::args::Args) {
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
        self.task.is_some() || self.queued_output.is_some()
    }
    pub fn metadata(&self) -> Value {
        json!({"live_observation":self.live_observation,"staged_flight":self.draft,"request":self.request,"pending":self.pending(),"source":self.source,"selected_vehicle":self.selected_vehicle,"selected_observation":self.selected_row,"selected_row":self.result.as_ref().and_then(|r|r.rows.get(self.selected_row)),"complete_manifest":self.result.as_ref().map(|r|&r.manifest),"status":self.status,"scope":"Actual live-world craft edits, plus independent pinned-world flight studies; no physical cargo delivery"})
    }
    pub fn poll(&mut self) {
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
        if self.task.is_none() {
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
        ui.label(egui::RichText::new("First spherical contact is not a safe landing. Cargo remains aboard; no habitat supply grant.").small().color(ACCENT));
        match pane {
            4 => self.live(ui, sim, plan, now),
            5 => self.recorded(ui, &sim.universe),
            _ => self.files(ui, &sim.universe),
        }
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
            let valid = d.validate(&sim.universe);
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
        ui.label("No atmosphere/entry heating, attitude, staging, terrain landing, docking or cargo transfer. Created cargo is an explicit initial condition. Gravity/frame changes stop unresolved flight with a retained reason.");
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
                egui::Label::new(serde_json::to_string_pretty(&recording::context()).unwrap())
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
        "Mass {:.9} kg · held fuel {:.9} kg · cargo {:.9} kg",
        p.mass_kg(d),
        p.propellant_kg,
        d.cargo.mass_kg()
    ));
    ui.label(format!(
        "Removed fuel {:.9} kg · applied ideal capability {:.9} m/s",
        p.exhausted_propellant_kg, p.cumulative_ideal_delta_v_m_s
    ));
    ui.label(format!(
        "Powered {:.9} s · fuel-limited command {:.9} s",
        p.powered_duration_s, p.fuel_limited_duration_s
    ));
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
            _ => r.flight.position_m.x - b.rows[0].flight.position_m.x,
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
        if metric == 0 {
            "kg"
        } else if metric == 3 {
            "m"
        } else {
            "m/s"
        }
    ));
}
pub fn preflight(args: &crate::args::Args) -> Result<(), String> {
    if let Some(path) = &args.flight_recording {
        let bundle = Bundle::read(Path::new(path))?;
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
