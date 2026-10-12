//! Finite cargo-removal mass-epoch research workflow; no world delivery.
use super::MUTED;
use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use bevy_egui::egui;
use cosmogon_sim::{
    supply_vehicle::Command,
    vehicle_cargo_mass as epochs,
};
use serde_json::{json, Value};

type WorkerResult = Result<(epochs::Bundle, String, bool), String>;

#[derive(Resource)]
pub struct CargoEpochsUi {
    draft: epochs::Plan,
    result: Option<epochs::Bundle>,
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
    metric: usize,
}
impl Default for CargoEpochsUi {
    fn default() -> Self {
        Self {
            draft: epochs::Plan::example(),
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
            metric: 0,
        }
    }
}
impl CargoEpochsUi {
    pub fn pending(&self) -> bool {
        self.task.is_some() || self.queued_output.is_some()
    }
    pub fn metadata(&self) -> Value {
        let mut v = self.pane_metadata();
        v["scope"] = json!("Independent finite burn/removal mass-epoch bench; no live world edits, flight integration or habitat delivery");
        v
    }
    fn pane_metadata(&self) -> Value {
        json!({"staged_plan":self.draft,"request":self.request,"source":self.source,"pending":self.pending(),"selected_observation":self.selected,"selected_row":self.result.as_ref().and_then(|r|r.rows.get(self.selected)),"complete_manifest":self.result.as_ref().map(|r|&r.manifest),"status":self.status})
    }
    pub(crate) fn poll(&mut self) {
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
                        "Cargo-epochs operation failed: {e}. Previous observations are preserved."
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
    fn commit(&mut self, bundle: epochs::Bundle, source: String, restore: bool) {
        if self
            .requested_observation
            .is_some_and(|n| n >= bundle.rows.len())
        {
            self.requested_observation = None;
            self.queued_output = None;
            self.status = Some(
                "Requested observation is outside the reopened sequence; previous results are preserved.".into(),
            );
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
        self.status = Some(
            "Actual burn/removal mass epochs retained; removal grants no impulse; live world unchanged."
                .into(),
        );
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
            json!({"plan":plan,"output":output,"source_context":epochs::context(),"scope":"Exact request-time independent sequence; later draft edits do not replace completed observations"}),
        );
        self.task = Some(AsyncComputeTaskPool::get().spawn(async move {
            if let Some(path) = output {
                plan.export(std::path::Path::new(&path))?;
                Ok((epochs::Bundle::read(std::path::Path::new(&path))?, path, false))
            } else {
                Ok((
                    epochs::Bundle::run(plan)?,
                    "Independent current sequence; no file written".into(),
                    false,
                ))
            }
        }));
        self.status =
            Some("Calculating piecewise mass epochs and exact removal events…".into());
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
                Ok((epochs::Bundle::read(std::path::Path::new(&path))?, path, true))
            }));
        self.status =
            Some("Checking complete sequence, sources, JSON/CSV and independent replay…".into());
    }
}
/// Automated inputs are checked before GPU startup; workers recheck disk at request.
pub fn preflight(args: &crate::args::Args) -> Result<(), String> {
    let bundle = if let Some(path) = &args.cargo_plan {
        Some(epochs::Bundle::run(epochs::Plan::read(std::path::Path::new(path))?)?)
    } else if let Some(path) = &args.cargo_recording {
        Some(epochs::Bundle::read(std::path::Path::new(path))?)
    } else {
        None
    };
    if let Some(index) = args.cargo_observation {
        if bundle.as_ref().is_none_or(|b| index >= b.rows.len()) {
            return Err("Requested cargo-epochs observation is outside its actual plan".into());
        }
    }
    if args
        .cargo_output
        .as_ref()
        .is_some_and(|p| std::path::Path::new(p.trim()).exists())
    {
        return Err("Existing cargo-epochs outputs are protected; choose a new folder".into());
    }
    Ok(())
}
pub fn init(form: &mut CargoEpochsUi, args: &crate::args::Args) {
    form.requested_observation = args.cargo_observation;
    form.output = args.cargo_output.clone().unwrap_or_default();
    if let Some(path) = &args.cargo_recording {
        form.path = path.clone();
        form.queued_output = args.cargo_output.clone();
        form.open(path.clone());
    } else if let Some(path) = &args.cargo_plan {
        form.definition_input = path.clone();
        match epochs::Plan::read(std::path::Path::new(path)) {
            Ok(plan) => {
                form.draft = plan;
                form.start(args.cargo_output.clone());
            }
            Err(e) => {
                form.requested_observation = None;
                form.status = Some(e);
            }
        }
    } else if args.panel.as_deref() == Some("cargo-epochs") {
        form.start(None);
    }
}
pub fn draw(ui: &mut egui::Ui, form: &mut CargoEpochsUi, busy: bool) {
    ui.label("Finite burn → species removal → later burn accounting · removal grants no impulse");
    ui.collapsing("Vehicle definition (immutable original design)",|ui|{definition(ui,&mut form.draft.definition);});
    ui.separator();
    steps(ui,form);
    ui.separator();
    match form.result.is_some() {
        true=>results(ui,form),
        false=>{ui.label("Calculate or reopen a complete sequence to inspect actual mass epochs.");}
    }
    ui.separator();
    files(ui,form,busy);
    ui.separator();
    if let Err(e)=form.draft.validate(){ui.colored_label(super::ACCENT,e);}
    if ui.add_enabled(!busy && form.draft.validate().is_ok(),egui::Button::new("Calculate this sequence")).clicked(){form.start(None);}
    if let Some(status)=&form.status {ui.add(egui::Label::new(status).wrap());}
    ui.label(egui::RichText::new("Mass-epoch bench only. Not connected to authoritative flight, contact, storage or habitat exchange; no cargo delivery is implied.").small().color(MUTED));
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
fn cargo_species(ui: &mut egui::Ui, label: &str, c: &mut cosmogon_sim::supply_vehicle::Cargo) {
    ui.label(label);
    number(ui,"Reserve oxygen (kg)",&mut c.oxygen_reserve_kg,0.0..=1e9);
    number(ui,"Potable water (kg)",&mut c.potable_water_kg,0.0..=1e9);
    number(ui,"Dry food (kg)",&mut c.food_dry_kg,0.0..=1e9);
    number(ui,"Food-bound water (kg)",&mut c.food_water_kg,0.0..=1e9);
}
fn definition(ui: &mut egui::Ui, d: &mut cosmogon_sim::supply_vehicle::Definition) {
    ui.label("Every design value is an explicit engineering assumption. Editing here does not rewrite past epoch denominators of completed calculations.");
    ui.text_edit_singleline(&mut d.name);
    number(ui,"Dry mass including hardware/containers (kg)",&mut d.dry_mass_kg,1.0..=1e12);
    number(ui,"Propellant tank capacity (kg)",&mut d.propellant_capacity_kg,0.0..=1e12);
    number(ui,"Originally held propellant (kg)",&mut d.initial_propellant_kg,0.0..=1e12);
    number(ui,"Payload mass capacity (kg)",&mut d.payload_capacity_kg,0.0..=4e9);
    number(ui,"Rated effective thrust (N)",&mut d.rated_thrust_n,0.0..=1e9);
    number(ui,"Assumed effective specific impulse (s)",&mut d.effective_isp_s,0.1..=1e5);
    cargo_species(ui,"Initially held consumable cargo · distinct from propellant",&mut d.cargo);
    if d.validate().is_ok(){
        ui.label(format!("Initial vehicle {:.6} kg · ideal initial scalar capability {:.6} m/s",d.inert_mass_kg()+d.initial_propellant_kg,d.initial_ideal_delta_v_m_s()));
    }
}
fn steps(ui: &mut egui::Ui, form: &mut CargoEpochsUi) {
    ui.label("Ordered steps act on one held cargo and propellant inventory. Removal is central and comoving at the live clock; it never exhausts propellant or grants thrust.");
    number(ui,"Observation interval (s)",&mut form.draft.observation_interval_s,0.001..=1e7);
    let mut remove = None;
    for (i, step) in form.draft.steps.iter_mut().enumerate() {
        ui.push_id(i,|ui|{
            ui.separator();
            match step {
                epochs::Step::Burn(c)=>{
                    ui.horizontal(|ui|{ui.strong(format!("Step {} · Burn",i+1));if ui.small_button("Remove").clicked(){remove=Some(i);}});
                    number(ui,"Duration (s)",&mut c.duration_s,0.0..=1e7);
                    number(ui,"Throttle (0 coast / 1 rated)",&mut c.throttle,0.0..=1.);
                    ui.horizontal_wrapped(|ui|{ui.label("Direction");for (label,dir) in [("+X",[1.,0.,0.]),("−X",[-1.,0.,0.]),("+Y",[0.,1.,0.]),("−Y",[0.,-1.,0.]),("+Z",[0.,0.,1.]),("−Z",[0.,0.,-1.])]{ui.selectable_value(&mut c.direction,dir,label);}});
                    ui.label(format!("Exact unit vector [{:.9}, {:.9}, {:.9}]",c.direction[0],c.direction[1],c.direction[2]));
                },
                epochs::Step::RemoveCargo(r)=>{
                    ui.horizontal(|ui|{ui.strong(format!("Step {} · Remove cargo",i+1));if ui.small_button("Remove").clicked(){remove=Some(i);}});
                    cargo_species(ui,"Requested species (must not overdraw held stock)",&mut r.requested);
                    ui.horizontal(|ui|{ui.label("Velocity (m/s)");for (label,k) in [("+X",0),("−X",1),("zero",2)]{if ui.small_button(label).clicked(){r.velocity_m_s=match k {0=>[1.,0.,0.],1=>[-1.,0.,0.],_=>[0.,0.,0.]};}}});
                    for (axis,value) in ["X","Y","Z"].into_iter().zip(r.velocity_m_s.iter_mut()){number(ui,axis,value,-1e6..=1e6);}
                    ui.label(egui::RichText::new("Central comoving removal carries material momentum/energy only; no plumbing, rotation or recipient model.").small().color(MUTED));
                },
            }
        });
    }
    if let Some(i) = remove {
        form.draft.steps.remove(i);
    }
    ui.horizontal(|ui|{
        if ui.add_enabled(form.draft.steps.len()<epochs::MAX_STEPS,egui::Button::new("Add burn")).clicked(){
            form.draft.steps.push(epochs::Step::Burn(Command{duration_s:60.,throttle:0.,direction:[1.,0.,0.]}));
        }
        if ui.add_enabled(form.draft.steps.len()<epochs::MAX_STEPS,egui::Button::new("Add cargo removal")).clicked(){
            form.draft.steps.push(epochs::Step::RemoveCargo(epochs::RemovalRequest{requested:cosmogon_sim::supply_vehicle::Cargo{oxygen_reserve_kg:0.,potable_water_kg:10.,food_dry_kg:0.,food_water_kg:0.},velocity_m_s:[0.,0.,0.]}));
        }
    });
    ui.label("At most 256 steps and 8192 observations. Exact fuel depletion receives a separate retained observation.");
}
fn results(ui: &mut egui::Ui, form: &mut CargoEpochsUi) {
    let Some(bundle) = &form.result else { return; };
    ui.add(egui::Label::new(format!("Retained source: {}", form.source)).wrap());
    let summary = &bundle.manifest["summary"];
    ui.label(format!(
        "{} observations · {} steps · {} removals · {}",
        bundle.rows.len(),
        bundle.plan.steps.len(),
        summary["removal_count"],
        bundle.plan.definition.name
    ));
    ui.label(format!(
        "Initial mass {:.9} kg · final mass {:.9} kg · cumulative ideal capability {:.9} m/s",
        summary["initial_mass_kg"].as_f64().unwrap_or_default(),
        summary["final_mass_kg"].as_f64().unwrap_or_default(),
        summary["cumulative_ideal_delta_v_m_s"].as_f64().unwrap_or_default()
    ));
    ui.collapsing("Epoch denominators and removal ledger",|ui|{
        for epoch in summary["epochs"].as_array().expect("validated summary epochs") {
            match epoch["kind"].as_str() {
                Some("burn")=>{ui.label(format!(
                    "Burn step {} · {:.9} s · m0 {:.6} → m1 {:.6} kg · ideal Δv {:.6} m/s · fuel {:.6} kg",
                    epoch["step_index"].as_u64().unwrap_or_default(),
                    epoch["elapsed_s"].as_f64().unwrap_or_default(),
                    epoch["initial_mass_kg"].as_f64().unwrap_or_default(),
                    epoch["final_mass_kg"].as_f64().unwrap_or_default(),
                    epoch["ideal_delta_v_m_s"].as_f64().unwrap_or_default(),
                    epoch["actual_propellant_kg"].as_f64().unwrap_or_default()
                ));},
                Some("removal")=>{ui.label(format!(
                    "Removal step {} · requested O₂ {:.4}/{:.4} H₂O {:.4}/{:.4} food {:.4}/{:.4} fw {:.4}/{:.4} kg · residual {} kg · |p| {:.6} kg·m/s · KE {:.6} J",
                    epoch["step_index"].as_u64().unwrap_or_default(),
                    epoch["requested"]["oxygen_reserve_kg"].as_f64().unwrap_or_default(),
                    epoch["actual_removed"]["oxygen_reserve_kg"].as_f64().unwrap_or_default(),
                    epoch["requested"]["potable_water_kg"].as_f64().unwrap_or_default(),
                    epoch["actual_removed"]["potable_water_kg"].as_f64().unwrap_or_default(),
                    epoch["requested"]["food_dry_kg"].as_f64().unwrap_or_default(),
                    epoch["actual_removed"]["food_dry_kg"].as_f64().unwrap_or_default(),
                    epoch["requested"]["food_water_kg"].as_f64().unwrap_or_default(),
                    epoch["actual_removed"]["food_water_kg"].as_f64().unwrap_or_default(),
                    epoch["mass_residual_kg"],
                    {
                        let p=&epoch["carried_linear_momentum_kg_m_s"];
                        (p["x"].as_f64().unwrap_or_default().powi(2)+p["y"].as_f64().unwrap_or_default().powi(2)+p["z"].as_f64().unwrap_or_default().powi(2)).sqrt()
                    },
                    epoch["carried_kinetic_energy_j"].as_f64().unwrap_or_default()
                ));},
                _=>{}
            }
        }
    });
    let labels = [
        "Vehicle mass (kg)",
        "Held propellant (kg)",
        "Cumulative ideal capability spent (m/s)",
        "Held cargo mass (kg)",
        "Velocity change along X (m/s)",
    ];
    egui::ComboBox::from_id_salt("cargo-epochs-metric")
        .selected_text(labels[form.metric])
        .show_ui(ui, |ui| {
            for (i, label) in labels.iter().enumerate() {
                ui.selectable_value(&mut form.metric, i, *label);
            }
        });
    let points = bundle.rows.iter().map(|r| {
        (r.state.elapsed_s, match form.metric {
            0 => r.vehicle_mass_kg,
            1 => r.state.propellant_kg,
            2 => r.state.cumulative_ideal_delta_v_m_s,
            3 => r.held.mass_kg(),
            _ => r.state.velocity_change_m_s[0],
        })
    }).collect::<Vec<_>>();
    super::charts::line_chart(ui, labels[form.metric], &points, false, |v| format!("{v:.6}"));
    ui.label("Elapsed bench seconds · retained points; connecting lines are display guides.");
    ui.add(egui::Slider::new(&mut form.selected, 0..=bundle.rows.len() - 1).text("Observation"));
    if let Some(row) = bundle.rows.get(form.selected) {
        ui.label(format!(
            "{} · step {} · {:.9} s · held fuel {:.9} kg · mass {:.9} kg",
            row.phase.key(),
            row.step_index.map(|v| v.to_string()).unwrap_or_else(||"initial".into()),
            row.state.elapsed_s,
            row.state.propellant_kg,
            row.vehicle_mass_kg
        ));
        ui.label(format!(
            "Held cargo O₂ {:.6} · water {:.6} · food {:.6} · food water {:.6} kg",
            row.held.oxygen_reserve_kg,row.held.potable_water_kg,row.held.food_dry_kg,row.held.food_water_kg
        ));
        ui.label(format!(
            "Cumulative impulse {:.9} N·s · ideal capability {:.9} m/s · velocity [{:.9}, {:.9}, {:.9}] m/s",
            row.state.cumulative_impulse_n_s,row.state.cumulative_ideal_delta_v_m_s,
            row.state.velocity_change_m_s[0],row.state.velocity_change_m_s[1],row.state.velocity_change_m_s[2]
        ));
        if let Some(i)=&row.burn{
            ui.label(format!(
                "Burn interval {:.9} s · mass {:.9} → {:.9} kg · ideal Δv {:.9} m/s · fuel {:.9} kg · raw residual {} kg",
                i.duration_s,i.initial_mass_kg,i.final_mass_kg,i.ideal_delta_v_m_s,i.actual_propellant_kg,i.mass_residual_kg
            ));
        }
        if let Some(e)=&row.removal{
            ui.label(format!(
                "Removal requested [{:.6}, {:.6}, {:.6}, {:.6}] kg · actual [{:.6}, {:.6}, {:.6}, {:.6}] kg",
                e.requested.oxygen_reserve_kg,e.requested.potable_water_kg,e.requested.food_dry_kg,e.requested.food_water_kg,
                e.actual_removed.oxygen_reserve_kg,e.actual_removed.potable_water_kg,e.actual_removed.food_dry_kg,e.actual_removed.food_water_kg
            ));
            ui.label(format!(
                "Mass residual {} kg · carried momentum [{:.6}, {:.6}, {:.6}] kg·m/s · KE {:.6} J",
                e.mass_residual_kg,e.carried_linear_momentum_kg_m_s.x,e.carried_linear_momentum_kg_m_s.y,e.carried_linear_momentum_kg_m_s.z,e.carried_kinetic_energy_j
            ));
        }
        if ui.button("Copy exact selected observation JSON").clicked() {
            ui.ctx().copy_text(serde_json::to_string_pretty(row).unwrap_or_default());
        }
    }
    ui.label("Completed observations belong to their retained sequence; staged edits wait for a new calculation.");
}
fn files(ui: &mut egui::Ui, form: &mut CargoEpochsUi, busy: bool) {
    ui.label(format!("Model {} · removal grants no impulse and exhausts no propellant", epochs::MODEL));
    ui.add(egui::Label::new(epochs::SEQUENCE_LIMITS).wrap());
    if let Some(sources)=epochs::context()["sources"].as_array(){
        for source in sources {
            if let Some(url)=source.as_str(){ui.hyperlink_to("Propulsion source (NASA GRC)",url);}
        }
    }
    ui.separator();
    ui.label("Complete recorded folder");
    ui.text_edit_singleline(&mut form.path);
    if ui.add_enabled(!busy && !form.path.trim().is_empty(),egui::Button::new("Verify and reopen recording")).clicked(){
        form.open(form.path.trim().into());
    }
    ui.label("New output folder for this staged sequence");
    ui.text_edit_singleline(&mut form.output);
    if ui.add_enabled(!busy && !form.output.trim().is_empty() && form.draft.validate().is_ok(),egui::Button::new("Calculate and export protected JSON + CSV")).clicked(){
        form.start(Some(form.output.trim().into()));
    }
    ui.label("New definition file");
    ui.text_edit_singleline(&mut form.definition_output);
    if ui.add_enabled(!busy && !form.definition_output.trim().is_empty(),egui::Button::new("Save staged sequence definition")).clicked(){
        form.status = Some(match form.draft.write(std::path::Path::new(form.definition_output.trim())){
            Ok(())=>"Sequence definition saved; no live world change.".into(),
            Err(e)=>e,
        });
    }
    ui.label("Typed sequence file to stage");
    ui.text_edit_singleline(&mut form.definition_input);
    if ui.add_enabled(!busy && !form.definition_input.trim().is_empty(),egui::Button::new("Load sequence into controls")).clicked(){
        form.status = Some(match epochs::Plan::read(std::path::Path::new(form.definition_input.trim())){
            Ok(plan)=>{form.draft=plan;"Sequence staged; previous completed observations retain their own inputs.".into()}
            Err(e)=>e,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn finish(form: &mut CargoEpochsUi) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while form.pending() {
            form.poll();
            assert!(
                std::time::Instant::now() < deadline,
                "Cargo-epochs worker did not complete"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[test]
    fn workers_pin_requested_sequence_preserve_later_edits_and_refuse_stale_exports() {
        AsyncComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
        let root = std::env::temp_dir().join(format!(
            "cosmogon-native-cargo-epochs-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let mut ui = CargoEpochsUi::default();
        let plan = ui.draft.clone();
        ui.start(None);
        ui.draft.steps.clear();
        finish(&mut ui);
        assert_eq!(ui.result.as_ref().unwrap().plan, plan);
        assert!(ui.draft.steps.is_empty());
        assert_eq!(
            ui.request.as_ref().unwrap()["plan"],
            serde_json::to_value(&plan).unwrap()
        );
        let original = ui.result.as_ref().unwrap().manifest.clone();
        let draft = ui.draft.clone();
        ui.queued_output = Some(root.join("must-not-exist").to_string_lossy().into());
        ui.open(root.join("missing").to_string_lossy().into());
        finish(&mut ui);
        assert!(!root.exists());
        assert_eq!(ui.result.as_ref().unwrap().manifest, original);
        assert_eq!(ui.draft, draft);
        assert!(ui.status.as_ref().unwrap().contains("failed"));
        plan.export(&root.join("source")).unwrap();
        ui.requested_observation = Some(2);
        ui.queued_output = Some(root.join("rerun").to_string_lossy().into());
        ui.open(root.join("source").to_string_lossy().into());
        finish(&mut ui);
        assert_eq!(ui.draft, plan);
        assert_eq!(ui.selected, 2);
        assert_eq!(
            epochs::Bundle::read(&root.join("rerun")).unwrap().manifest,
            original
        );
        assert_eq!(ui.metadata()["complete_manifest"], original);
        let before = std::fs::read(root.join("rerun/manifest.json")).unwrap();
        ui.start(Some(root.join("rerun").to_string_lossy().into()));
        finish(&mut ui);
        assert_eq!(
            std::fs::read(root.join("rerun/manifest.json")).unwrap(),
            before
        );
        ui.requested_observation = Some(8191);
        ui.queued_output = Some(root.join("invalid-selection").to_string_lossy().into());
        ui.open(root.join("source").to_string_lossy().into());
        finish(&mut ui);
        assert!(!root.join("invalid-selection").exists());
        assert_eq!(ui.selected, 2);
        assert!(ui.status.as_ref().unwrap().contains("outside"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn automated_preflight_checks_actual_rows_and_protected_outputs() {
        let root = std::env::temp_dir().join(format!(
            "cosmogon-cargo-epochs-preflight-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let p = epochs::Plan::example();
        p.write(&root.join("plan.json")).unwrap();
        let mut args = crate::args::Args {
            cargo_plan: Some(root.join("plan.json").to_string_lossy().into()),
            cargo_observation: Some(p.run().unwrap().len() - 1),
            ..Default::default()
        };
        assert!(preflight(&args).is_ok());
        args.cargo_observation = Some(p.run().unwrap().len());
        assert!(preflight(&args).is_err());
        args.cargo_observation = None;
        args.cargo_output = Some(root.to_string_lossy().into());
        assert!(preflight(&args).is_err());
        p.export(&root.join("recording")).unwrap();
        args.cargo_output = None;
        args.cargo_plan = None;
        args.cargo_recording = Some(root.join("recording").to_string_lossy().into());
        assert!(preflight(&args).is_ok());
        std::fs::write(root.join("recording/epochs.csv"), "altered").unwrap();
        assert!(preflight(&args).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
