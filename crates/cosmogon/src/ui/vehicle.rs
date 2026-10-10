//! Authored vehicle design remains independent of the live universe.
use super::{ACCENT, MUTED};
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContexts};
use cosmogon_sim::supply_vehicle::{self as vehicle, Bundle, Command, Plan};
use serde_json::{json, Value};

type WorkerResult = Result<(Bundle, String, bool), String>;

#[derive(Resource)]
pub struct VehicleUi {
    draft: Plan,
    result: Option<Bundle>,
    source: String,
    path: String,
    output: String,
    definition_output: String,
    definition_input: String,
    status: Option<String>,
    task: Option<Task<WorkerResult>>,
    queued_output: Option<String>,
    requested_observation: Option<usize>,
    request: Option<Value>,
    selected: usize,
    pane: usize,
    metric: usize,
}
impl Default for VehicleUi {
    fn default() -> Self {
        Self {
            draft: Plan::example(),
            result: None,
            source: String::new(),
            path: String::new(),
            output: String::new(),
            definition_output: String::new(),
            definition_input: String::new(),
            status: None,
            task: None,
            queued_output: None,
            requested_observation: None,
            request: None,
            selected: 0,
            pane: 0,
            metric: 0,
        }
    }
}
impl VehicleUi {
    pub fn pending(&self) -> bool {
        self.task.is_some() || self.queued_output.is_some()
    }
    pub fn metadata(&self) -> Value {
        json!({"pane":self.pane,"staged_plan":self.draft,"request":self.request,"source":self.source,"pending":self.pending(),"selected_observation":self.selected,"selected_row":self.result.as_ref().and_then(|r|r.rows.get(self.selected)),"complete_manifest":self.result.as_ref().map(|r|&r.manifest),"status":self.status,"scope":"Independent finite propulsion design bench; no live world edits, flight trajectory or cargo delivery"})
    }
    fn poll(&mut self) {
        if let Some(result) = self
            .task
            .as_mut()
            .and_then(|t| block_on(future::poll_once(t)))
        {
            self.task = None;
            match result {
                Ok((bundle, source, restore)) => self.commit(bundle, source, restore),
                Err(e) => {
                    self.queued_output = None;
                    self.requested_observation = None;
                    self.status = Some(format!(
                        "Vehicle operation failed: {e}. Previous observations are preserved."
                    ));
                }
            }
        }
        if self.task.is_none() {
            if let Some(output) = self.queued_output.take() {
                self.start(Some(output));
            }
        }
    }
    fn commit(&mut self, bundle: Bundle, source: String, restore: bool) {
        if self
            .requested_observation
            .is_some_and(|n| n >= bundle.rows.len())
        {
            self.requested_observation = None;
            self.queued_output = None;
            self.status=Some("Requested observation is outside the reopened design; previous results are preserved.".into());
            return;
        }
        self.selected = self
            .requested_observation
            .take()
            .unwrap_or(self.selected)
            .min(bundle.rows.len().saturating_sub(1));
        if restore {
            self.draft = bundle.plan.clone();
        }
        self.result = Some(bundle);
        self.source = source;
        self.status =
            Some("Actual finite propulsion observations retained; live world unchanged.".into());
    }
    fn start(&mut self, output: Option<String>) {
        let plan = self.draft.clone();
        if self.task.is_some() {
            return;
        }
        if let Err(e) = plan.validate() {
            self.queued_output = None;
            self.requested_observation = None;
            self.status = Some(e);
            return;
        }
        self.request = Some(
            json!({"plan":plan,"output":output,"source_context":vehicle::context(),"scope":"Exact request-time independent plan; later draft edits do not replace completed observations"}),
        );
        self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
            if let Some(path) = output {
                plan.export(std::path::Path::new(&path))?;
                Ok((Bundle::read(std::path::Path::new(&path))?, path, false))
            } else {
                Ok((
                    Bundle::run(plan)?,
                    "Independent current design; no file written".into(),
                    false,
                ))
            }
        }));
        self.status = Some("Calculating finite burns and exact depletion boundaries…".into());
    }
    fn open(&mut self, path: String) {
        if self.task.is_some() {
            return;
        }
        self.request = Some(
            json!({"recording":path,"operation":"Verify and reopen original retained observations"}),
        );
        self.task =
            Some(AsyncComputeTaskPool::get().spawn(async move {
                Ok((Bundle::read(std::path::Path::new(&path))?, path, true))
            }));
        self.status =
            Some("Checking complete design, sources, JSON/CSV and independent replay…".into());
    }
}
/// Automated inputs are checked before GPU startup; workers recheck disk at request.
pub fn preflight(args: &crate::args::Args) -> Result<(), String> {
    let bundle = if let Some(path) = &args.vehicle_plan {
        Some(Bundle::run(Plan::read(std::path::Path::new(path))?)?)
    } else if let Some(path) = &args.vehicle_recording {
        Some(Bundle::read(std::path::Path::new(path))?)
    } else {
        None
    };
    if let Some(index) = args.vehicle_observation {
        if bundle.as_ref().is_none_or(|b| index >= b.rows.len()) {
            return Err("Requested vehicle observation is outside its actual plan".into());
        }
    }
    if args
        .vehicle_output
        .as_ref()
        .is_some_and(|p| std::path::Path::new(p.trim()).exists())
    {
        return Err("Existing vehicle outputs are protected; choose a new folder".into());
    }
    Ok(())
}
pub fn init(mut form: ResMut<VehicleUi>, args: Res<crate::args::Args>) {
    form.pane = match args.panel.as_deref() {
        Some("vehicle-burns") => 1,
        Some("vehicle-results") => 2,
        Some("vehicle-model" | "vehicle-files") => 3,
        _ => 0,
    };
    form.requested_observation = args.vehicle_observation;
    form.output = args.vehicle_output.clone().unwrap_or_default();
    if let Some(path) = &args.vehicle_recording {
        form.path = path.clone();
        form.queued_output = args.vehicle_output.clone();
        form.open(path.clone());
    } else if let Some(path) = &args.vehicle_plan {
        form.definition_input = path.clone();
        match Plan::read(std::path::Path::new(path)) {
            Ok(plan) => {
                form.draft = plan;
                form.start(args.vehicle_output.clone());
            }
            Err(e) => {
                form.requested_observation = None;
                form.status = Some(e);
            }
        }
    } else if form.pane == 2 {
        form.start(None);
    }
}
pub fn panel(
    mut contexts: EguiContexts,
    mut state: ResMut<super::UiState>,
    mut form: ResMut<VehicleUi>,
) {
    form.poll();
    if !state.vehicle_open || state.hidden || state.photo_mode {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = state.vehicle_open;
    let max_height = (ctx.content_rect().height() - 180.).max(240.);
    egui::Window::new("Supply vehicle design").open(&mut open).default_pos([345.,82.]).default_width(600.).default_height(570.).max_height(max_height).show(ctx,|ui| {
        ui.label("Finite propellant and cargo · authored free-space propulsion bench");
        ui.horizontal_wrapped(|ui|{for (i,label) in ["Design","Burn plan","Results","Models & files"].iter().enumerate(){ui.selectable_value(&mut form.pane,i,*label);}});
        let busy=form.pending();
        egui::ScrollArea::vertical().id_salt("vehicle-design-scroll").show(ui,|ui| {
            match form.pane {
                0=>design(ui,&mut form),1=>burns(ui,&mut form),2=>results(ui,&mut form),_=>files(ui,&mut form,busy),
            }
            ui.separator();
            if let Err(e)=form.draft.validate(){ui.colored_label(ACCENT,e);}
            if ui.add_enabled(!busy && form.draft.validate().is_ok(),egui::Button::new("Calculate this design")).clicked(){form.start(None);}
            if let Some(status)=&form.status {ui.add(egui::Label::new(status).wrap());}
            ui.label(egui::RichText::new("Design-only cargo stays aboard. This does not launch a vehicle, establish a landing or supply a habitat.").small().color(MUTED));
        });
    });
    state.vehicle_open = open;
}
fn number(ui: &mut egui::Ui, label: &str, value: &mut f64, range: std::ops::RangeInclusive<f64>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::DragValue::new(value)
                .range(range)
                .speed(0.1)
                .max_decimals(9),
        );
    });
}
fn design(ui: &mut egui::Ui, form: &mut VehicleUi) {
    ui.label("Every design value is an explicit engineering assumption.");
    let d = &mut form.draft.definition;
    ui.text_edit_singleline(&mut d.name);
    number(
        ui,
        "Dry mass including hardware/containers (kg)",
        &mut d.dry_mass_kg,
        1.0..=1e12,
    );
    number(
        ui,
        "Propellant tank capacity (kg)",
        &mut d.propellant_capacity_kg,
        0.0..=1e12,
    );
    number(
        ui,
        "Originally held propellant (kg)",
        &mut d.initial_propellant_kg,
        0.0..=1e12,
    );
    number(
        ui,
        "Payload mass capacity (kg)",
        &mut d.payload_capacity_kg,
        0.0..=4e9,
    );
    number(
        ui,
        "Rated effective thrust (N)",
        &mut d.rated_thrust_n,
        0.0..=1e9,
    );
    number(
        ui,
        "Assumed effective specific impulse (s)",
        &mut d.effective_isp_s,
        0.1..=1e5,
    );
    ui.strong("Retained consumable cargo · distinct from propellant");
    number(
        ui,
        "Reserve oxygen (kg)",
        &mut d.cargo.oxygen_reserve_kg,
        0.0..=1e9,
    );
    number(
        ui,
        "Potable water (kg)",
        &mut d.cargo.potable_water_kg,
        0.0..=1e9,
    );
    number(ui, "Dry food (kg)", &mut d.cargo.food_dry_kg, 0.0..=1e9);
    number(
        ui,
        "Food-bound water (kg)",
        &mut d.cargo.food_water_kg,
        0.0..=1e9,
    );
    ui.label(format!(
        "Cargo {:.6} kg · initial vehicle {:.6} kg",
        d.cargo.mass_kg(),
        d.inert_mass_kg() + d.initial_propellant_kg
    ));
    if d.validate().is_ok() {
        ui.label(format!(
            "Ideal initial scalar capability {:.6} m/s · effective exhaust {:.6} m/s",
            d.initial_ideal_delta_v_m_s(),
            d.exhaust_velocity_m_s()
        ));
    }
    ui.label("Capacity changes do not silently truncate stocks. Correct invalid held masses before running.");
}
fn burns(ui: &mut egui::Ui, form: &mut VehicleUi) {
    ui.label("Commands act on one finite held propellant inventory. Direction is an authored bench axis; no gravity or attitude dynamics.");
    number(
        ui,
        "Observation interval (s)",
        &mut form.draft.observation_interval_s,
        0.001..=1e7,
    );
    let mut remove = None;
    for (i, c) in form.draft.commands.iter_mut().enumerate() {
        ui.push_id(i,|ui|{
            ui.separator();ui.horizontal(|ui|{ui.strong(format!("Command {}",i+1));if ui.small_button("Remove").clicked(){remove=Some(i);}});
            number(ui,"Duration (s)",&mut c.duration_s,0.0..=1e7);number(ui,"Throttle (0 coast / 1 rated)",&mut c.throttle,0.0..=1.);
            ui.horizontal_wrapped(|ui|{ui.label("Direction");for (label,dir) in [("+X",[1.,0.,0.]),("−X",[-1.,0.,0.]),("+Y",[0.,1.,0.]),("−Y",[0.,-1.,0.]),("+Z",[0.,0.,1.]),("−Z",[0.,0.,-1.])]{ui.selectable_value(&mut c.direction,dir,label);}});
            ui.label(format!("Exact unit vector [{:.9}, {:.9}, {:.9}]",c.direction[0],c.direction[1],c.direction[2]));
            ui.collapsing("Custom inertial unit direction",|ui|{for (axis,value) in ["X","Y","Z"].into_iter().zip(c.direction.iter_mut()){number(ui,axis,value,-1.0..=1.0);}ui.label("Supply a unit vector; invalid directions are refused, never silently normalized.");});
        });
    }
    if let Some(i) = remove {
        form.draft.commands.remove(i);
    }
    if ui
        .add_enabled(
            form.draft.commands.len() < 1024,
            egui::Button::new("Add command"),
        )
        .clicked()
    {
        form.draft.commands.push(Command {
            duration_s: 60.,
            throttle: 0.,
            direction: [1., 0., 0.],
        });
    }
    ui.label("At most 8192 observations. Exact fuel depletion receives a separate retained observation; regular points do not hide the cutoff.");
}
fn results(ui: &mut egui::Ui, form: &mut VehicleUi) {
    let Some(bundle) = &form.result else {
        ui.label("Calculate or reopen a complete design to inspect actual finite observations.");
        return;
    };
    ui.add(egui::Label::new(format!("Retained source: {}", form.source)).wrap());
    ui.label(format!(
        "{} actual observations · {} commands · {}",
        bundle.rows.len(),
        bundle.plan.commands.len(),
        bundle.plan.definition.name
    ));
    let summary = &bundle.manifest["summary"];
    ui.label(format!(
        "Initial mass {:.9} kg · final mass {:.9} kg · retained cargo {:.9} kg",
        summary["initial_mass_kg"].as_f64().unwrap_or_default(),
        summary["final_mass_kg"].as_f64().unwrap_or_default(),
        bundle.plan.definition.cargo.mass_kg()
    ));
    ui.label(format!(
        "Raw propellant closure residual: {} kg",
        summary["mass_residual_kg"]
    ));
    let labels = [
        "Held propellant (kg)",
        "Vehicle mass (kg)",
        "Cumulative ideal capability spent (m/s)",
        "Velocity change along X (m/s)",
        "Velocity change along Y (m/s)",
        "Velocity change along Z (m/s)",
        "Fuel-limited firing duration (s)",
    ];
    egui::ComboBox::from_id_salt("vehicle-metric")
        .selected_text(labels[form.metric])
        .show_ui(ui, |ui| {
            for (i, label) in labels.iter().enumerate() {
                ui.selectable_value(&mut form.metric, i, *label);
            }
        });
    let points = bundle
        .rows
        .iter()
        .map(|r| {
            (
                r.state.elapsed_s,
                match form.metric {
                    0 => r.state.propellant_kg,
                    1 => r.state.mass_kg(&bundle.plan.definition),
                    2 => r.state.cumulative_ideal_delta_v_m_s,
                    3..=5 => r.state.velocity_change_m_s[form.metric - 3],
                    _ => r.state.fuel_limited_duration_s,
                },
            )
        })
        .collect::<Vec<_>>();
    super::charts::line_chart(ui, labels[form.metric], &points, false, |v| {
        format!("{v:.6}")
    });
    ui.label("Elapsed bench seconds · retained points; connecting lines are display guides.");
    ui.add(egui::Slider::new(&mut form.selected, 0..=bundle.rows.len() - 1).text("Observation"));
    if let Some(row) = bundle.rows.get(form.selected) {
        ui.label(format!(
            "{} · {:.9} s · held fuel {:.9} kg · exhaust {:.9} kg",
            row.phase.key(),
            row.state.elapsed_s,
            row.state.propellant_kg,
            row.state.exhausted_propellant_kg
        ));
        ui.label(format!(
            "Velocity change [{:.9}, {:.9}, {:.9}] m/s",
            row.state.velocity_change_m_s[0],
            row.state.velocity_change_m_s[1],
            row.state.velocity_change_m_s[2]
        ));
        ui.label(format!(
            "Powered {:.9} s · fuel-limited {:.9} s · impulse {:.9} N·s",
            row.state.powered_duration_s,
            row.state.fuel_limited_duration_s,
            row.state.cumulative_impulse_n_s
        ));
        if let Some(i) = &row.interval {
            ui.label(format!(
                "Preceding interval {:.9} s · removed fuel {:.9} kg · raw mass residual {} kg",
                i.duration_s, i.actual_propellant_kg, i.mass_residual_kg
            ));
            ui.label(format!(
                "Commanded thrust {:.9} N · actual interval impulse {:.9} N·s · ideal Δv {:.9} m/s",
                i.commanded_thrust_n, i.impulse_n_s, i.ideal_delta_v_m_s
            ));
            if let Some(a) = i.initial_powered_acceleration_m_s2 {
                ui.label(format!(
                    "Powered acceleration {:.9} → {:.9} m/s²",
                    a,
                    i.final_powered_acceleration_m_s2.unwrap_or(a)
                ));
            } else {
                ui.label("Powered acceleration unavailable: this retained interval has no powered duration.");
            }
        }
        if ui.button("Copy exact selected observation JSON").clicked() {
            ui.ctx()
                .copy_text(serde_json::to_string_pretty(row).unwrap_or_default());
        }
    }
    ui.label("Completed observations belong to their retained plan; edits in Design/Burn plan are staged for a new calculation.");
}
fn files(ui: &mut egui::Ui, form: &mut VehicleUi, busy: bool) {
    ui.label("mdot = thrust / (Isp × g0) · ideal Δv = Isp × g0 × ln(initial mass / final mass)");
    ui.label(format!(
        "Model {} · standard g0 = {} m/s²",
        vehicle::MODEL,
        vehicle::G0
    ));
    ui.add(egui::Label::new(vehicle::LIMITS).wrap());
    if let Some(sources) = vehicle::context()["sources"].as_array() {
        for source in sources {
            ui.hyperlink_to(
                source["role"].as_str().unwrap_or("Source"),
                source["url"].as_str().unwrap_or(""),
            );
        }
    }
    ui.separator();
    ui.label("Complete recorded folder");
    ui.text_edit_singleline(&mut form.path);
    if ui
        .add_enabled(
            !busy && !form.path.trim().is_empty(),
            egui::Button::new("Verify and reopen recording"),
        )
        .clicked()
    {
        form.open(form.path.trim().into());
    }
    ui.label("New output folder for this staged design");
    ui.text_edit_singleline(&mut form.output);
    if ui
        .add_enabled(
            !busy && !form.output.trim().is_empty() && form.draft.validate().is_ok(),
            egui::Button::new("Calculate and export protected JSON + CSV"),
        )
        .clicked()
    {
        form.start(Some(form.output.trim().into()));
    }
    ui.label("New definition file");
    ui.text_edit_singleline(&mut form.definition_output);
    if ui
        .add_enabled(
            !busy && !form.definition_output.trim().is_empty(),
            egui::Button::new("Save staged definition"),
        )
        .clicked()
    {
        form.status = Some(
            match form
                .draft
                .write(std::path::Path::new(form.definition_output.trim()))
            {
                Ok(()) => "Definition saved; no live world change.".into(),
                Err(e) => e,
            },
        );
    }
    ui.label("Typed plan file to stage");
    ui.text_edit_singleline(&mut form.definition_input);
    if ui
        .add_enabled(
            !busy && !form.definition_input.trim().is_empty(),
            egui::Button::new("Load plan into controls"),
        )
        .clicked()
    {
        form.status = Some(
            match Plan::read(std::path::Path::new(form.definition_input.trim())) {
                Ok(plan) => {
                    form.draft = plan;
                    "Plan staged; previous completed observations retain their own inputs.".into()
                }
                Err(e) => e,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn finish(form: &mut VehicleUi) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while form.pending() {
            form.poll();
            assert!(
                std::time::Instant::now() < deadline,
                "Vehicle worker did not complete"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[test]
    fn workers_pin_requested_plan_preserve_later_edits_and_refuse_stale_exports() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let root =
            std::env::temp_dir().join(format!("cosmogon-native-vehicle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut ui = VehicleUi::default();
        let plan = ui.draft.clone();
        ui.start(None);
        ui.draft.definition.cargo.potable_water_kg = 0.;
        ui.path = "Unopened folder".into();
        finish(&mut ui);
        assert_eq!(ui.result.as_ref().unwrap().plan, plan);
        assert_eq!(ui.draft.definition.cargo.potable_water_kg, 0.);
        assert_eq!(
            ui.request.as_ref().unwrap()["plan"],
            serde_json::to_value(&plan).unwrap()
        );
        let original = ui.result.as_ref().unwrap().manifest.clone();
        let source = ui.source.clone();
        let draft = ui.draft.clone();
        ui.queued_output = Some(root.join("must-not-exist").to_string_lossy().into());
        ui.open(root.join("missing").to_string_lossy().into());
        finish(&mut ui);
        assert!(!root.exists());
        assert_eq!(ui.result.as_ref().unwrap().manifest, original);
        assert_eq!(ui.source, source);
        assert_eq!(ui.draft, draft);
        assert!(ui.status.as_ref().unwrap().contains("failed"));
        plan.export(&root.join("source")).unwrap();
        ui.requested_observation = Some(3);
        ui.queued_output = Some(root.join("rerun").to_string_lossy().into());
        ui.open(root.join("source").to_string_lossy().into());
        finish(&mut ui);
        assert_eq!(ui.draft, plan);
        assert_eq!(ui.selected, 3);
        assert_eq!(
            Bundle::read(&root.join("rerun")).unwrap().manifest,
            original
        );
        assert_eq!(ui.metadata()["complete_manifest"], original);
        assert!(ui.request.as_ref().unwrap()["output"]
            .as_str()
            .unwrap()
            .ends_with("rerun"));
        let before = std::fs::read(root.join("rerun/manifest.json")).unwrap();
        ui.start(Some(root.join("rerun").to_string_lossy().into()));
        finish(&mut ui);
        assert_eq!(
            std::fs::read(root.join("rerun/manifest.json")).unwrap(),
            before
        );
        assert_eq!(ui.result.as_ref().unwrap().manifest, original);
        ui.requested_observation = Some(8191);
        ui.queued_output = Some(root.join("invalid-selection").to_string_lossy().into());
        ui.open(root.join("source").to_string_lossy().into());
        finish(&mut ui);
        assert!(!root.join("invalid-selection").exists());
        assert_eq!(ui.selected, 3);
        assert!(ui.status.as_ref().unwrap().contains("outside"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn automated_preflight_checks_actual_rows_and_protected_outputs() {
        let root =
            std::env::temp_dir().join(format!("cosmogon-vehicle-preflight-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let p = Plan::example();
        p.write(&root.join("plan.json")).unwrap();
        let mut args = crate::args::Args {
            vehicle_plan: Some(root.join("plan.json").to_string_lossy().into()),
            vehicle_observation: Some(p.run().unwrap().len() - 1),
            ..Default::default()
        };
        assert!(preflight(&args).is_ok());
        args.vehicle_observation = Some(p.run().unwrap().len());
        assert!(preflight(&args).is_err());
        args.vehicle_observation = None;
        args.vehicle_output = Some(root.to_string_lossy().into());
        assert!(preflight(&args).is_err());
        p.export(&root.join("recording")).unwrap();
        args.vehicle_output = None;
        args.vehicle_plan = None;
        args.vehicle_recording = Some(root.join("recording").to_string_lossy().into());
        assert!(preflight(&args).is_ok());
        std::fs::write(root.join("recording/trajectories.csv"), "altered").unwrap();
        assert!(preflight(&args).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
