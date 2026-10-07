//! A cancellable background experiment branches from frozen state, never the live Sim.
use bevy::prelude::*;
use cosmogon_sim::research::{
    BatchProgress, BatchRunSummary, ExperimentDefinition, ParameterId, ResearchTarget,
};
use cosmogon_sim::Universe;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
};

enum Update {
    Progress(BatchProgress),
    Finished(Result<Vec<BatchRunSummary>, String>),
}
struct Job {
    receiver: Mutex<mpsc::Receiver<Update>>,
    cancel: Arc<AtomicBool>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

#[derive(Resource)]
pub struct ResearchBatch {
    pub frozen: Option<Universe>,
    pub definition: Option<ExperimentDefinition>,
    pub target_name: String,
    pub status: Option<String>,
    pub progress: Option<BatchProgress>,
    pub results: Vec<BatchRunSummary>,
    pub output: Option<PathBuf>,
    pub definition_path: String,
    pub sweep_enabled: bool,
    pub sweep_parameter: ParameterId,
    pub sweep_target: Option<ResearchTarget>,
    pub sweep_from: f64,
    pub sweep_to: f64,
    pub sweep_count: usize,
    pub compare_parameter: ParameterId,
    pub results_mode: bool,
    pub finished_success: Option<bool>,
    pub automatic_done: bool,
    automatic_started: bool,
    job: Option<Job>,
}
impl Default for ResearchBatch {
    fn default() -> Self {
        Self {
            frozen: None,
            definition: None,
            target_name: String::new(),
            status: None,
            progress: None,
            results: Vec::new(),
            output: None,
            definition_path: String::new(),
            sweep_enabled: false,
            sweep_parameter: ParameterId::Albedo,
            sweep_target: None,
            sweep_from: 0.2,
            sweep_to: 0.4,
            sweep_count: 5,
            compare_parameter: ParameterId::Temperature,
            results_mode: false,
            finished_success: None,
            automatic_done: false,
            automatic_started: false,
            job: None,
        }
    }
}
impl ResearchBatch {
    pub fn running(&self) -> bool {
        self.job.is_some()
    }
    pub fn freeze(&mut self, u: &Universe, target: ResearchTarget) -> Result<(), String> {
        if self.running() {
            return Err("Wait for or cancel the current experiment".into());
        }
        let name = target
            .name(u)
            .ok_or("Select an existing target")?
            .to_string();
        // Use the recording boundary to enforce the snapshot size limit before retaining state.
        cosmogon_sim::research::Experiment::new(u, target, 2)?;
        self.frozen = Some(u.clone());
        self.definition = Some(ExperimentDefinition::new(target));
        self.definition.as_mut().unwrap().initial_fingerprint =
            Some(cosmogon_sim::research::state_fingerprint(u)?);
        self.target_name = name;
        self.results.clear();
        self.finished_success = None;
        self.results_mode = false;
        self.output = None;
        self.progress = None;
        self.sweep_enabled = false;
        self.sweep_target = Some(target);
        if let Some(p) = cosmogon_sim::research::PARAMETERS.iter().find(|p| {
            p.editable && cosmogon_sim::research::read_parameter(u, target, p.id).is_some()
        }) {
            self.sweep_parameter = p.id;
            let value = cosmogon_sim::research::read_parameter(u, target, p.id)
                .unwrap()
                .value;
            self.sweep_from = value;
            self.sweep_to = value;
        }
        self.status=Some("Starting state frozen. Runs use independent copies; the live universe can keep running.".into());
        Ok(())
    }
    pub fn start(&mut self, directory: PathBuf) -> Result<(), String> {
        if self.running() {
            return Err("An experiment is already running".into());
        }
        let initial = self
            .frozen
            .as_ref()
            .ok_or("Freeze a starting state first")?
            .clone();
        let definition = self
            .definition
            .as_ref()
            .ok_or("No experiment definition")?
            .clone();
        let (sender, receiver) = mpsc::sync_channel(8);
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let out = directory.clone();
        // Validation and all integration happen on the worker. A full queue drops only progress.
        std::thread::Builder::new()
            .name("cosmogon-research".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    cosmogon_sim::research::run_batch(
                        &initial,
                        &definition,
                        &out,
                        || flag.load(Ordering::Relaxed),
                        |p| {
                            let _ = sender.try_send(Update::Progress(p));
                        },
                    )
                }))
                .unwrap_or_else(|_| {
                    Err("Experiment worker failed; inspect any partial output".into())
                });
                let _ = sender.send(Update::Finished(result));
            })
            .map_err(|e| e.to_string())?;
        self.job = Some(Job {
            receiver: Mutex::new(receiver),
            cancel,
        });
        self.output = Some(directory);
        self.results.clear();
        self.finished_success = None;
        self.progress = None;
        self.status = Some("Running an independent frozen-state experiment…".into());
        Ok(())
    }
    pub fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
            self.status =
                Some("Cancellation requested. Completed exported runs are retained.".into());
        }
    }
    pub fn poll(&mut self) {
        let messages = if let Some(job) = &self.job {
            match job.receiver.lock() {
                Ok(rx) => rx.try_iter().collect::<Vec<_>>(),
                Err(_) => Vec::new(),
            }
        } else {
            return;
        };
        for m in messages {
            match m {
                Update::Progress(p) => self.progress = Some(p),
                Update::Finished(result) => {
                    self.job = None;
                    self.finished_success = Some(result.is_ok());
                    self.results_mode = true;
                    match result {
                        Ok(r) => {
                            self.status=Some(format!("Completed {} independent runs. CSV and replay definition are saved.",r.len()));
                            self.results = r;
                        }
                        Err(e) => {
                            self.status = Some(e);
                            // A failed/cancelled batch still has genuine completed results on disk.
                            if let Some(path) = self.output.as_ref().map(|p| p.join("batch.json")) {
                                if std::fs::metadata(&path).is_ok_and(|m| m.len() <= 1024 * 1024) {
                                    if let Ok(bytes) = std::fs::read(path) {
                                        if let Ok(value) =
                                            serde_json::from_slice::<serde_json::Value>(&bytes)
                                        {
                                            if value["format"] == "cosmogon-experiment-batch"
                                                && value["schema_version"] == 1
                                            {
                                                if let Ok(runs) =
                                                    serde_json::from_value(value["runs"].clone())
                                                {
                                                    self.results = runs;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
pub struct ResearchBatchPlugin;
impl Plugin for ResearchBatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResearchBatch>()
            .add_systems(Update, automatic.before(crate::state::Frame::Simulate));
    }
}
#[allow(clippy::too_many_arguments)]
fn automatic(
    mut batch: ResMut<ResearchBatch>,
    args: Res<crate::args::Args>,
    sim: Option<Res<crate::sim::Sim>>,
    research: Res<crate::research::Research>,
    performance: Res<crate::performance::Performance>,
    mut exit: MessageWriter<AppExit>,
) {
    batch.poll();
    if args.research_batch.is_none() {
        return;
    }
    if !batch.automatic_started {
        let Some(sim) = sim else {
            return;
        };
        batch.automatic_started = true;
        let result = (|| -> Result<(), String> {
            let plan = args
                .research_plan
                .as_deref()
                .ok_or("Batch automation requires --research-plan DEFINITION_JSON")?;
            let definition = ExperimentDefinition::read(std::path::Path::new(plan))?;
            let target = definition.measurement_target;
            batch.freeze(&sim.universe, target)?;
            if let Some(s) = &definition.sweep {
                batch.sweep_target = Some(s.target);
                batch.sweep_enabled = true;
                batch.sweep_parameter = s.parameter;
                batch.sweep_from = s.values.first().copied().unwrap_or(0.0);
                batch.sweep_to = s.values.last().copied().unwrap_or(0.0);
                batch.sweep_count = s.values.len();
            }
            batch.definition = Some(definition);
            if let Some(p) = batch
                .definition
                .as_ref()
                .and_then(|d| d.primary_measurement)
            {
                batch.compare_parameter = p;
            }
            batch.start(PathBuf::from(args.research_batch.as_ref().unwrap()))
        })();
        if let Err(e) = result {
            error!("Research batch failed: {e}");
            batch.status = Some(e);
            batch.automatic_done = true;
            if args.quit_after_capture {
                exit.write(AppExit::error());
            }
            return;
        }
    }
    if !batch.running() && !batch.automatic_done {
        batch.automatic_done = true;
        let failed = batch.finished_success != Some(true);
        if failed {
            error!("Research batch failed: {:?}", batch.status);
        }
        if args.quit_after_capture
            && (failed
                || (args.capture.is_none()
                    && (args.profile.is_none() || performance.automatic_done)
                    && (args.research_export.is_none() || research.automatic_done)))
        {
            exit.write(if failed {
                AppExit::error()
            } else {
                AppExit::Success
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_results_are_visible_but_never_marked_successful() {
        use cosmogon_sim::research::ParameterSweep;
        let u = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = ResearchTarget::Body(cosmogon_sim::BodyRef { system: 0, body: 2 });
        let mut d = ExperimentDefinition::new(target);
        d.samples = 2;
        d.interval_s = 60.0;
        d.sweep = Some(ParameterSweep {
            target,
            parameter: ParameterId::Albedo,
            values: vec![0.2, 0.4],
        });
        let path = std::env::temp_dir().join(format!(
            "cosmogon-partial-native-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let stop = std::cell::Cell::new(false);
        let error = cosmogon_sim::research::run_batch(
            &u,
            &d,
            &path,
            || stop.get(),
            |p| {
                if p.completed_runs == 1 {
                    stop.set(true);
                }
            },
        )
        .unwrap_err();
        let (sender, receiver) = mpsc::sync_channel(8);
        sender.send(Update::Finished(Err(error))).unwrap();
        let mut batch = ResearchBatch {
            output: Some(path.clone()),
            job: Some(Job {
                receiver: Mutex::new(receiver),
                cancel: Arc::new(AtomicBool::new(false)),
            }),
            ..Default::default()
        };
        batch.poll();
        assert!(!batch.running());
        assert_eq!(batch.results.len(), 1);
        assert_eq!(batch.finished_success, Some(false));
        assert!(batch.results_mode);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn frozen_branch_does_not_follow_live_edits_or_time() {
        let mut u = Universe::new(cosmogon_sim::UniverseSettings {
            scenario: cosmogon_sim::Scenario::SolarSystemLab,
            ..Default::default()
        });
        let target = ResearchTarget::Body(cosmogon_sim::BodyRef { system: 0, body: 2 });
        let mut b = ResearchBatch::default();
        b.freeze(&u, target).unwrap();
        let before = serde_json::to_string(b.frozen.as_ref().unwrap()).unwrap();
        u.apply_edit(
            cosmogon_sim::research::parameter_edit(&u, target, ParameterId::Albedo, 0.6).unwrap(),
        )
        .unwrap();
        u.advance_by(60.0);
        assert_eq!(
            serde_json::to_string(b.frozen.as_ref().unwrap()).unwrap(),
            before
        );
        assert_ne!(serde_json::to_string(&u).unwrap(), before);
    }
}
