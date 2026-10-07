//! Native control-panel state and bounded observer recording; never writes Universe.
use crate::{
    args::Args,
    sim::{Sim, Target},
    state::AppState,
};
use bevy::prelude::*;
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use cosmogon_sim::research::{read_parameter, Experiment, ParameterId, ResearchTarget, PARAMETERS};
use std::collections::BTreeMap;

pub fn target(t: Target) -> ResearchTarget {
    match t {
        Target::Star(s) => ResearchTarget::Star(s),
        Target::Body(r) => ResearchTarget::Body(r),
    }
}
#[derive(Resource)]
pub struct Research {
    pub open: bool,
    pub tab: usize,
    pub query: String,
    pub model_index: usize,
    pub info: Option<ParameterId>,
    pub baseline: BTreeMap<ParameterId, f64>,
    pub baseline_target: Option<(ResearchTarget, u64)>,
    pub experiment: Option<Experiment>,
    pub recording: bool,
    pub status: Option<String>,
    pub automatic_done: bool,
    pub sample_interval_s: f64,
    pub plot: ParameterId,
    pub saved: Option<cosmogon_sim::research::SavedRecording>,
    pub recording_path: String,
    pub recording_task: Option<Task<Result<cosmogon_sim::research::SavedRecording, String>>>,
    pub import_automatic_done: bool,
    pub validation_requested: bool,
    pub validation_report: Option<cosmogon_sim::research::ValidationReport>,
    pub validation_task: Option<Task<cosmogon_sim::research::ValidationReport>>,
    generation: u64,
    frames: usize,
    automatic_started: bool,
    last_sample_real: f64,
}
impl Default for Research {
    fn default() -> Self {
        Self {
            open: false,
            tab: 0,
            query: String::new(),
            model_index: 0,
            info: None,
            baseline: BTreeMap::new(),
            baseline_target: None,
            experiment: None,
            recording: false,
            status: None,
            automatic_done: false,
            sample_interval_s: 1.0,
            plot: ParameterId::Temperature,
            saved: None,
            recording_path: String::new(),
            recording_task: None,
            import_automatic_done: true,
            validation_requested: false,
            validation_report: None,
            validation_task: None,
            generation: 0,
            frames: 0,
            automatic_started: false,
            last_sample_real: 0.0,
        }
    }
}
impl Research {
    pub fn selection(&mut self, sim: &Sim, t: ResearchTarget) {
        if self.baseline_target != Some((t, sim.generation)) {
            self.baseline = PARAMETERS
                .iter()
                .filter_map(|p| read_parameter(&sim.universe, t, p.id).map(|r| (p.id, r.value)))
                .collect();
            self.baseline_target = Some((t, sim.generation));
            self.info = None;
            self.model_index = usize::from(sim.universe.system(t.system()).is_dynamic());
        }
    }
    pub fn start(
        &mut self,
        sim: &Sim,
        t: ResearchTarget,
        limit: usize,
        now: f64,
    ) -> Result<(), String> {
        self.experiment = Some(Experiment::new(&sim.universe, t, limit)?);
        self.saved = None;
        self.experiment.as_mut().unwrap().sampling_policy = format!("Native observer: requested real-time interval {} s; actual simulation times are in samples.csv",self.sample_interval_s);
        self.recording = true;
        self.generation = sim.generation;
        self.frames = 0;
        self.last_sample_real = now;
        self.status = Some(
            "Recording observed states. Changing selection does not change the recorded target."
                .into(),
        );
        let e = self.experiment.as_ref().unwrap();
        if !e.parameters.iter().any(|p| p.descriptor.id == self.plot) {
            self.plot = e.parameters[0].descriptor.id;
        }
        Ok(())
    }
    pub fn load_recording(&mut self) {
        if self.recording_task.is_some() {
            return;
        }
        let path = std::path::PathBuf::from(self.recording_path.trim());
        self.recording_task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { cosmogon_sim::research::SavedRecording::read(&path) }),
        );
        self.status = Some("Opening recorded data in the background…".into());
    }
    pub fn stop(&mut self, reason: &str) {
        self.recording = false;
        if let Some(e) = &mut self.experiment {
            e.stop_reason = Some(reason.into());
        }
        self.status = Some(reason.into());
    }
    pub fn export(&mut self, path: &std::path::Path) -> Result<(), String> {
        self.experiment
            .as_ref()
            .ok_or("Start a recording first")?
            .export(path)?;
        self.status = Some(format!("Experiment saved to {}", path.display()));
        Ok(())
    }
}

pub struct ResearchPlugin;
impl Plugin for ResearchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Research>()
            .init_resource::<crate::research_workspace::ResearchWorkspace>()
            .add_systems(Startup, init)
            .add_systems(Update, (poll_validation, poll_recording))
            .add_systems(
                Last,
                record
                    .before(crate::sim::autosave_on_exit)
                    .run_if(in_state(AppState::Observing).and(resource_exists::<Sim>)),
            );
    }
}
fn init(
    mut research: ResMut<Research>,
    args: Res<Args>,
    mut workspace: ResMut<crate::research_workspace::ResearchWorkspace>,
) {
    if let Some(path) = &args.research_workspace {
        *workspace = crate::research_workspace::ResearchWorkspace::load(path.into());
    }
    if let Some(path) = &args.research_recording {
        research.recording_path = path.clone();
        research.open = true;
        research.tab = 3;
        research.import_automatic_done = false;
        research.load_recording();
    }
    match args.panel.as_deref() {
        Some("control") => research.open = true,
        Some("models") => {
            research.open = true;
            research.tab = 1;
        }
        Some("sources") => {
            research.open = true;
            research.tab = 2;
        }
        Some("plots") => {
            research.open = true;
            research.tab = 3;
        }
        Some("experiments") => {
            research.open = true;
            research.tab = 4;
        }
        Some("validation") => {
            research.open = true;
            research.tab = 1;
            research.validation_requested = true;
        }
        _ => {}
    }
}
fn poll_validation(mut research: ResMut<Research>) {
    if research.validation_requested && research.validation_task.is_none() {
        research.validation_requested = false;
        research.validation_report = None;
        research.validation_task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { cosmogon_sim::research::validation_report() }),
        );
        research.status =
            Some("Measuring analytic and cached ephemeris fixtures in the background…".into());
    }
    if let Some(task) = &mut research.validation_task {
        if let Some(report) = block_on(future::poll_once(task)) {
            research.status = Some(format!(
                "{} measured validation cases · {}. Applies only to the reported conditions.",
                report.cases.len(),
                if report.passes_regression {
                    "regression guards passed"
                } else {
                    "regression guard failed"
                }
            ));
            research.validation_task = None;
            research.validation_report = Some(report);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn record(
    mut research: ResMut<Research>,
    sim: Res<Sim>,
    args: Res<Args>,
    rig: Res<crate::camera::CameraRig>,
    time: Res<Time<Real>>,
    perf: Res<crate::performance::Performance>,
    mut exit: MessageWriter<AppExit>,
    batch: Res<crate::research_batch::ResearchBatch>,
) {
    let now = time.elapsed_secs_f64();
    if args.research_export.is_some() && !research.automatic_started {
        research.automatic_started = true;
        let result = sim
            .selected
            .or(rig.focus)
            .ok_or("No research target selected".into())
            .and_then(|t| research.start(&sim, target(t), args.research_samples, now));
        if result.is_ok() {
            if let Some(e) = &mut research.experiment {
                e.sampling_policy=format!("Native observer: every {} rendered app frames; actual simulation times are in samples.csv",args.research_every);
            }
        }
        if let Err(e) = result {
            error!("Cannot record research: {e}");
            research.stop(&e);
            research.automatic_done = true;
            if args.quit_after_capture {
                exit.write(AppExit::error());
            }
        }
    }
    if !research.recording {
        return;
    }
    if research.generation != sim.generation {
        research.stop(
            "Recording stopped: universe replacement or rewind. Start a separate experiment.",
        );
        if args.research_export.is_some() && args.quit_after_capture {
            exit.write(AppExit::error());
        }
        return;
    }
    research.frames += 1;
    if let Some(e) = &mut research.experiment {
        e.observe_context(&sim.universe);
    }
    let due = if args.research_export.is_some() {
        research.frames.is_multiple_of(args.research_every)
    } else {
        now - research.last_sample_real >= research.sample_interval_s
    };
    if !due {
        return;
    }
    research.last_sample_real = now;
    if let Some(e) = &mut research.experiment {
        if let Err(error) = e.capture(&sim.universe) {
            research.stop(&error);
            if args.research_export.is_some() && args.quit_after_capture {
                exit.write(AppExit::error());
            }
            return;
        }
    }
    if !research.experiment.as_ref().is_some_and(|e| e.full()) {
        return;
    }
    research.stop("Recording complete: sample limit reached.");
    if let Some(path) = &args.research_export {
        let result = research.export(std::path::Path::new(path));
        research.automatic_done = true;
        let failed = result.is_err();
        if let Err(error) = result {
            research.status = Some(format!("Export failed: {error}"));
            error!("Research export failed: {error}");
        } else {
            info!("Research experiment saved to {path}");
        }
        if args.quit_after_capture
            && (failed
                || (args.capture.is_none()
                    && (args.profile.is_none() || perf.automatic_done)
                    && (args.research_batch.is_none() || batch.automatic_done)))
        {
            exit.write(if failed {
                AppExit::error()
            } else {
                AppExit::Success
            });
        }
    }
}
pub fn export_path() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    crate::persistence::data_dir()
        .join("experiments")
        .join(format!("experiment-{nanos}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_edits_undo_exactly_and_reference_views_refuse_research_edits() {
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = cosmogon_sim::BodyRef { system: 0, body: 2 };
        let edit = cosmogon_sim::research::parameter_edit(
            &u,
            ResearchTarget::Body(r),
            ParameterId::Radius,
            7e6,
        )
        .unwrap();
        let m = crate::persistence::Manifest::new(
            "Test",
            "",
            "dev",
            crate::persistence::Origin::default(),
        );
        let mut sim = Sim::new(u, crate::sim::Session::Sandbox(m));
        let before = serde_json::to_string(&sim.universe).unwrap();
        sim.edit(edit.clone(), 0.0).unwrap();
        assert_eq!(sim.universe.body(r).radius, 7e6);
        sim.undo(1.0);
        assert_eq!(serde_json::to_string(&sim.universe).unwrap(), before);
        sim.session = crate::sim::Session::Reference {
            dataset: "test".into(),
            title: "Test".into(),
        };
        assert!(sim.edit(edit, 2.0).is_err());
        assert_eq!(serde_json::to_string(&sim.universe).unwrap(), before);
    }
    #[test]
    fn selection_snapshot_is_target_and_generation_scoped() {
        let u = cosmogon_sim::Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::EmptySystem,
            ..Default::default()
        });
        let mut sim = Sim::new(
            u,
            crate::sim::Session::Reference {
                dataset: "test".into(),
                title: "Test".into(),
            },
        );
        let mut research = Research::default();
        research.selection(&sim, ResearchTarget::Star(0));
        assert!(research.baseline.contains_key(&ParameterId::StarMass));
        assert!(!research.baseline.contains_key(&ParameterId::Mass));
        sim.generation += 1;
        research.selection(&sim, ResearchTarget::Star(0));
        assert_eq!(research.baseline_target.unwrap().1, 1);
    }
}

fn poll_recording(
    mut research: ResMut<Research>,
    args: Res<Args>,
    mut exit: MessageWriter<AppExit>,
    performance: Res<crate::performance::Performance>,
    batch: Res<crate::research_batch::ResearchBatch>,
) {
    let result = research
        .recording_task
        .as_mut()
        .and_then(|task| block_on(future::poll_once(task)));
    if let Some(result) = result {
        research.recording_task = None;
        research.import_automatic_done = true;
        match result {
            Ok(table) => {
                research.recording = false;
                research.status = Some(format!(
                    "Opened {} recorded states for {}. Live universe unchanged.",
                    table.samples.len(),
                    table.target_name
                ));
                research.saved = Some(table);
                if args.research_recording.is_some()
                    && args.quit_after_capture
                    && args.capture.is_none()
                    && (args.profile.is_none() || performance.automatic_done)
                    && (args.research_export.is_none() || research.automatic_done)
                    && (args.research_batch.is_none() || batch.automatic_done)
                {
                    exit.write(AppExit::Success);
                }
            }
            Err(e) => {
                research.status = Some(format!("Could not open recording: {e}"));
                if args.research_recording.is_some() && args.quit_after_capture {
                    error!("Research recording load failed: {e}");
                    exit.write(AppExit::error());
                }
            }
        }
    }
}
