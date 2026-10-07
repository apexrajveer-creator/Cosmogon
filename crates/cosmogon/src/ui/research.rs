//! A searchable control panel and research workspace over the authoritative model.
use super::{
    units::{self, Quantity},
    UiState, ACCENT, MUTED,
};
use crate::{camera::CameraRig, research::Research, sim::Sim};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use cosmogon_sim::research::{
    parameter_edit, read_parameter, DataState, ParameterId, PhysicsModelRegistry, ResearchTarget,
    PARAMETERS, SCIENTIFIC_SOURCES,
};

fn badge(ui: &mut egui::Ui, state: DataState) {
    let color = match state {
        DataState::Measured => egui::Color32::from_rgb(110, 180, 240),
        DataState::UserModified => egui::Color32::from_rgb(235, 140, 90),
        DataState::Modelled | DataState::Estimated => ACCENT,
        DataState::Procedural => egui::Color32::from_rgb(170, 130, 220),
        _ => MUTED,
    };
    ui.label(
        egui::RichText::new(state.label())
            .size(9.0)
            .color(color)
            .strong(),
    );
}
fn quantity(id: ParameterId) -> (Quantity, usize) {
    use ParameterId::*;
    match id {
        Mass => (Quantity::Mass, 1),
        StarMass => (Quantity::Mass, 3),
        Radius => (Quantity::Length, 1),
        SemiMajor => (Quantity::Length, 5),
        Inclination => (Quantity::Angle, 0),
        OrbitalPeriod => (Quantity::Duration, 2),
        StarRadius => (Quantity::Length, 4),
        X | Y | Z => (Quantity::Length, 5),
        Vx | Vy | Vz | Escape => (Quantity::Speed, 1),
        Rotation => (Quantity::Duration, 1),
        StarAge => (Quantity::Duration, 3),
        Tilt => (Quantity::Angle, 0),
        Pressure => (Quantity::Pressure, 0),
        Co2 | Albedo => (Quantity::Fraction, 0),
        Temperature | Equilibrium | StarTemperature => (Quantity::Temperature, 0),
        _ => (Quantity::Plain, 0),
    }
}
fn control(
    ui: &mut egui::Ui,
    research: &mut Research,
    sim: &Sim,
    target: ResearchTarget,
) -> Option<cosmogon_sim::sandbox::Edit> {
    ui.label(
        egui::RichText::new(target.name(&sim.universe).unwrap_or("Unavailable object"))
            .strong()
            .size(19.0),
    );
    ui.add(
        egui::TextEdit::singleline(&mut research.query)
            .hint_text("Search parameters, sections or models…")
            .desired_width(f32::INFINITY),
    );
    if sim.is_reference() {
        ui.colored_label(
            ACCENT,
            "Real Universe is read-only. Clone to a sandbox to experiment.",
        );
    }
    ui.label(egui::RichText::new("Edits are undoable. Derived values stay locked and update from their inputs. ‘Reset’ restores this selection's snapshot.").small().color(MUTED));
    let query = research.query.trim().to_lowercase();
    let mut pending = None;
    let mut found = false;
    for section in [
        "Physical",
        "Rotation",
        "Environment",
        "Star",
        "Solver",
        "Orbit",
        "Diagnostics",
        "State vector",
    ] {
        let rows: Vec<_> = PARAMETERS
            .iter()
            .filter(|p| {
                p.section == section
                    && (query.is_empty()
                        || format!("{} {} {} {}", p.name, p.key, p.section, p.model)
                            .to_lowercase()
                            .contains(&query))
            })
            .filter_map(|p| read_parameter(&sim.universe, target, p.id).map(|r| (p, r)))
            .collect();
        if rows.is_empty() {
            continue;
        }
        found = true;
        let mut header = egui::CollapsingHeader::new(section)
            .id_salt(("research_section", section))
            .default_open(section == "Physical" || section == "Star");
        if !query.is_empty() {
            header = header.open(Some(true));
        }
        header.show(ui,|ui| {
            egui::Grid::new(("research_controls",section)).striped(true).num_columns(3).spacing([10.0,8.0]).show(ui,|ui| {
                for (p,r) in rows {
                    ui.label(p.name);
                    let (q,unit)=quantity(p.id);
                    ui.vertical(|ui| {
                        if let Some(v)=units::edit(ui,(target,p.id),r.value,q,unit,p.editable && !sim.is_reference(),None) {
                            match parameter_edit(&sim.universe,target,p.id,v) { Ok(e)=>pending=Some(e),Err(e)=>research.status=Some(e) }
                        }
                        badge(ui,r.state);
                    });
                    ui.horizontal(|ui| {
                        if ui.small_button("Info").clicked() { research.info=Some(p.id); }
                        if p.editable && !sim.is_reference() { if let Some(v)=research.baseline.get(&p.id).copied() { if ui.small_button("Reset").clicked() { match parameter_edit(&sim.universe,target,p.id,v) { Ok(e)=>pending=Some(e),Err(e)=>research.status=Some(e) } } } }
                        else { ui.label(egui::RichText::new("Locked").small().color(MUTED)); }
                    });
                    ui.end_row();
                }
            });
            if section=="State vector" { ui.label(egui::RichText::new("System frame: barycentric with N-body, fixed primary with Kepler. Changing a component preserves the other five; it activates dynamic gravity.").small().color(MUTED)); }
        });
    }
    if !found {
        ui.label("No matching supported parameters for this object.");
    }
    if let Some(id) = research.info {
        let p = id.descriptor();
        ui.separator();
        ui.label(
            egui::RichText::new(format!("{} · {}", p.name, p.unit))
                .strong()
                .color(ACCENT),
        );
        ui.label(format!("Formula / algorithm: {}", p.formula));
        ui.label(format!("Valid input / scope: {}", p.limits));
        ui.label(format!("Affects: {}", p.effects));
        let names = p
            .dependencies
            .iter()
            .map(|id| id.descriptor().name)
            .collect::<Vec<_>>()
            .join(", ");
        ui.label(format!(
            "Depends on: {}",
            if names.is_empty() {
                "Stored input"
            } else {
                &names
            }
        ));
        let reverse = PARAMETERS
            .iter()
            .filter(|d| d.dependencies.contains(&id))
            .map(|d| d.name)
            .collect::<Vec<_>>()
            .join(", ");
        if !reverse.is_empty() {
            ui.label(format!("Used by: {reverse}"));
        }
        if let Some(r) = read_parameter(&sim.universe, target, id) {
            ui.label(format!("Source: {}", r.source));
            ui.label(
                egui::RichText::new("Uncertainty: unavailable; no invented error bars.")
                    .small()
                    .color(MUTED),
            );
        }
        if ui.button("Inspect this model").clicked() {
            if let Some(i) = PhysicsModelRegistry::models()
                .iter()
                .position(|m| m.id == p.model)
            {
                research.model_index = i;
                research.tab = 1;
            }
        }
    }
    pending
}
fn models(
    ui: &mut egui::Ui,
    research: &mut Research,
    sim: &Sim,
    target: ResearchTarget,
    ui_state: &mut UiState,
) {
    validation(ui, research);
    let sys = sim.universe.system(target.system());
    ui.label(
        egui::RichText::new(format!(
            "Current gravity: {}",
            if sys.is_dynamic() {
                "Newtonian N-body / rails hybrid"
            } else {
                "Analytic Kepler"
            }
        ))
        .strong(),
    );
    if ui.button("Open gravity and solver settings").clicked() {
        ui_state.physics_open = true;
    }
    let entries = PhysicsModelRegistry::models();
    research.model_index = research.model_index.min(entries.len() - 1);
    egui::ComboBox::from_id_salt("research_model")
        .selected_text(entries[research.model_index].name)
        .width(450.0)
        .show_ui(ui, |ui| {
            for (i, m) in entries.iter().enumerate() {
                ui.selectable_value(&mut research.model_index, i, m.name);
            }
        });
    let m = &entries[research.model_index];
    ui.label(
        egui::RichText::new(format!("{} · {}", m.id, m.domain))
            .small()
            .color(MUTED),
    );
    for (title, text) in [
        ("Equations / algorithm", m.equations),
        ("Numerical method", m.numerical_method),
        ("Units", m.units),
        ("Assumptions", m.assumptions),
        ("Validity", m.validity),
        ("Known limitations", m.limitations),
        ("Expected error", m.expected_error),
        ("Fidelity", m.fidelity),
        ("Runtime cost", m.runtime_cost),
    ] {
        ui.add_space(7.0);
        ui.label(egui::RichText::new(title).strong().color(ACCENT));
        ui.label(text);
    }
    egui::CollapsingHeader::new("Validation cases and implementation").show(ui, |ui| {
        for c in m.validation_cases {
            ui.monospace(*c);
        }
        ui.monospace(m.implementation);
        ui.label(
            egui::RichText::new("These are regression references, not a live test-run status.")
                .small()
                .color(MUTED),
        );
    });
    ui.separator();
    for id in m.sources {
        if let Some(s) = PhysicsModelRegistry::source(id) {
            ui.hyperlink_to(s.title, s.url);
            ui.label(egui::RichText::new(s.limitations).small().color(MUTED));
        }
    }
}
fn validation(ui: &mut egui::Ui, research: &mut Research) {
    use egui::RichText;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                research.validation_task.is_none(),
                egui::Button::new("Check orbit models against reference cases"),
            )
            .clicked()
        {
            research.validation_requested = true;
        }
        if research.validation_task.is_some() {
            ui.spinner();
        }
    });
    ui.label(RichText::new("Measured analytic/reference cases help evaluate a model for a particular use. Regression limits are acceptance guards, not observational uncertainty. Fixed fixture settings apply; these do not test your current edited universe.").small().color(MUTED));
    if let Some(report) = &research.validation_report {
        ui.label(
            RichText::new(format!(
                "{} computed cases · {}",
                report.cases.len(),
                if report.passes_regression {
                    "regression guards passed"
                } else {
                    "a regression guard failed"
                }
            ))
            .strong()
            .color(ACCENT),
        );
        egui::CollapsingHeader::new("Measured results and conditions")
            .default_open(true)
            .show(ui, |ui| {
                for case in &report.cases {
                    egui::CollapsingHeader::new(&case.name)
                        .id_salt(&case.id)
                        .default_open(case.id.contains("horizons"))
                        .show(ui, |ui| {
                            ui.label(&case.model);
                            egui::Grid::new(("validation_metrics", &case.id))
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("Measured quantity");
                                    ui.strong("Result");
                                    ui.strong("Regression limit");
                                    ui.end_row();
                                    for m in &case.metrics {
                                        ui.label(&m.quantity);
                                        ui.vertical(|ui| {
                                            ui.add(
                                                egui::Label::new(format!("{:.6e}", m.value))
                                                    .wrap_mode(egui::TextWrapMode::Extend),
                                            );
                                            ui.label(RichText::new(&m.unit).small().color(MUTED));
                                        });
                                        ui.label(format!(
                                            "{} · {}",
                                            units::fmt_num(m.regression_limit),
                                            if m.passes_regression { "PASS" } else { "FAIL" }
                                        ));
                                        ui.end_row();
                                    }
                                });
                            ui.label(RichText::new(&case.limitations).small().color(MUTED));
                            egui::CollapsingHeader::new("Inputs and reference")
                                .id_salt((&case.id, "inputs"))
                                .show(ui, |ui| {
                                    ui.label(&case.reference);
                                    ui.monospace(
                                        serde_json::to_string_pretty(&case.inputs)
                                            .unwrap_or_default(),
                                    );
                                });
                        });
                }
                for note in &report.limitations {
                    ui.label(RichText::new(note).small().color(MUTED));
                }
            });
        if ui.button("Export measured validation report").clicked() {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = crate::persistence::data_dir()
                .join("reports")
                .join(format!("validation-{nanos}.json"));
            research.status = Some(match report.export(&path) {
                Ok(()) => format!("Validation report saved to {}", path.display()),
                Err(e) => format!("Validation export failed: {e}"),
            });
        }
    }
    ui.separator();
}

fn sources(ui: &mut egui::Ui, sim: &Sim, target: ResearchTarget) {
    ui.label(egui::RichText::new("Scientific data and source context").strong());
    ui.label(egui::RichText::new("Measured, derived, estimated, modelled, ML inference, procedural, user modified and unknown are distinct states. No ML inference is produced by the current models.").small().color(MUTED));
    ui.add_space(8.0);
    egui::CollapsingHeader::new("Selected object's value states")
        .default_open(true)
        .show(ui, |ui| {
            for p in PARAMETERS {
                if let Some(r) = read_parameter(&sim.universe, target, p.id) {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(p.name);
                        badge(ui, r.state);
                    });
                    ui.label(egui::RichText::new(r.source).small().color(MUTED));
                }
            }
        });
    egui::CollapsingHeader::new("Embedded dataset versions").show(ui, |ui| {
        let versions = cosmogon_sim::save::data_versions(&sim.universe);
        if versions.is_empty() {
            ui.label("This universe has no recorded observational dataset dependency.");
        }
        for (id, version) in versions {
            ui.label(format!("{id}: {version}"));
        }
    });
    for s in SCIENTIFIC_SOURCES {
        egui::CollapsingHeader::new(s.title)
            .id_salt(s.id)
            .show(ui, |ui| {
                ui.hyperlink_to("Open reference", s.url);
                ui.label(s.version);
                ui.label(s.limitations);
                ui.monospace(s.implementation);
            });
    }
    ui.label(egui::RichText::new("Source descriptions do not grant redistribution rights. Detailed inventory: data/SOURCE_REGISTRY. No runtime download is required.").small().color(MUTED));
}
fn plots(ui: &mut egui::Ui, research: &mut Research, sim: &Sim, target: ResearchTarget, now: f64) {
    ui.horizontal(|ui| {
        if research.recording {
            if ui.button("Stop recording").clicked() {
                if let Some(e) = &mut research.experiment {
                    e.observe_context(&sim.universe);
                }
                research.stop("Recording stopped by user.");
            }
        } else if ui.button("Start new recording").clicked() {
            if let Err(e) = research.start(sim, target, 4096, now) {
                research.status = Some(e);
            }
        }
        ui.add(
            egui::Slider::new(&mut research.sample_interval_s, 0.1..=10.0)
                .text("real seconds / sample"),
        );
    });
    ui.label(egui::RichText::new("Recording stays attached to its starting object. Unchanged paused states are not duplicated; paused edits receive separate rows. Rewind or loading stops it. Limit: 4096 states.").small().color(MUTED));
    if let Some(e) = &research.experiment {
        ui.label(format!(
            "{} · {} / {} samples",
            e.target_name,
            e.samples.len(),
            e.limit
        ));
        egui::ComboBox::from_id_salt("research_plot")
            .selected_text(research.plot.descriptor().name)
            .show_ui(ui, |ui| {
                for p in &e.parameters {
                    ui.selectable_value(&mut research.plot, p.descriptor.id, p.descriptor.name);
                }
            });
        let p = research.plot.descriptor();
        let events = e
            .events
            .iter()
            .map(|event| (event.time - e.initial_header.sim_time, event.title.clone()))
            .collect::<Vec<_>>();
        super::research_chart::chart(
            ui,
            ("research_main", p.id),
            p.name,
            p.unit,
            &e.points(p.id),
            &events,
        );
        let companions: Vec<_> = e
            .parameters
            .iter()
            .filter(|p| {
                matches!(
                    p.descriptor.id,
                    ParameterId::Temperature
                        | ParameterId::Insolation
                        | ParameterId::Habitability
                        | ParameterId::Gravity
                        | ParameterId::Luminosity
                        | ParameterId::StarTemperature
                ) && p.descriptor.id != research.plot
            })
            .take(3)
            .map(|p| p.descriptor.id)
            .collect();
        for id in companions {
            let p = id.descriptor();
            super::research_chart::chart(
                ui,
                ("research_companion", p.id),
                p.name,
                p.unit,
                &e.points(id),
                &events,
            );
        }
        egui::CollapsingHeader::new(format!("Observed Chronicle events ({})", e.events.len()))
            .show(ui, |ui| {
                for event in e.events.iter().rev().take(20) {
                    ui.label(format!(
                        "{} s · {}",
                        event.time - e.initial_header.sim_time,
                        event.title
                    ));
                }
            });
        if ui.button("Export CSV + reproducibility bundle").clicked() {
            let path = crate::research::export_path();
            if let Err(e) = research.export(&path) {
                research.status = Some(format!("Export failed: {e}"));
            }
        }
        ui.label(egui::RichText::new("Includes unit-labelled columns, model/source metadata, initial save, observed edits and events. Error bars remain unavailable.").small().color(MUTED));
    } else {
        ui.label("Start a recording to compare orbital, physical and environment quantities on the same timeline.");
    }
}
fn research_targets(u: &cosmogon_sim::Universe) -> Vec<(ResearchTarget, String)> {
    let mut targets = Vec::new();
    for (system, sys) in u.systems.iter().enumerate() {
        targets.push((
            ResearchTarget::Star(system as u32),
            format!("{} · star", sys.star.name),
        ));
        for (body, b) in sys.bodies.iter().enumerate().filter(|(_, b)| b.exists()) {
            targets.push((
                ResearchTarget::Body(cosmogon_sim::BodyRef {
                    system: system as u32,
                    body: body as u32,
                }),
                format!("{} · {}", b.name, sys.star.name),
            ));
        }
    }
    targets
}

fn experiments(
    ui: &mut egui::Ui,
    batch: &mut crate::research_batch::ResearchBatch,
    sim: &Sim,
    target: ResearchTarget,
) {
    use cosmogon_sim::research::{ExperimentDefinition, ParameterSweep};
    ui.heading("Ask a question. Change one input. Compare the outcomes.");
    ui.label("Freeze a starting state, then run independent copies in the background. Your live universe stays separate from the experiment.");
    if ui
        .add_enabled(
            !batch.running(),
            egui::Button::new("Freeze selected object's starting state"),
        )
        .clicked()
    {
        if let Err(e) = batch.freeze(&sim.universe, target) {
            batch.status = Some(e);
        }
    }
    ui.horizontal(|ui| {
        ui.selectable_value(&mut batch.results_mode, false, "Set up study");
        ui.selectable_value(&mut batch.results_mode, true, "Results");
    });
    if batch.running() && batch.results_mode && ui.button("Cancel running study").clicked() {
        batch.cancel();
    }
    let running = batch.running();
    let previous_grid = (
        batch.sweep_enabled,
        batch.sweep_parameter,
        batch.sweep_target,
        batch.sweep_from,
        batch.sweep_to,
        batch.sweep_count,
    );
    if !batch.results_mode {
        if let (Some(initial), Some(definition)) = (&batch.frozen, &mut batch.definition) {
            ui.label(
                egui::RichText::new(format!(
                    "Frozen: {} · {}",
                    batch.target_name,
                    initial.date_label()
                ))
                .strong()
                .color(ACCENT),
            );
            ui.add_enabled_ui(!running, |ui| {
            ui.horizontal(|ui| {
                ui.label("Experiment name");ui.text_edit_singleline(&mut definition.name);
            });
            egui::CollapsingHeader::new("Guided questions").default_open(true).show(ui,|ui| {
                if read_parameter(initial,definition.measurement_target,ParameterId::Radius).is_some() {
                    if ui.button("How does radius affect surface gravity?").clicked() {
                        let radius=read_parameter(initial,definition.measurement_target,ParameterId::Radius).unwrap().value;
                        definition.name="Radius and surface gravity".into();definition.samples=2;definition.interval_s=60.0;
                        batch.sweep_enabled=true;batch.sweep_parameter=ParameterId::Radius;batch.sweep_from=radius*0.5;batch.sweep_to=radius*2.0;batch.sweep_count=4;batch.sweep_target=Some(definition.measurement_target);batch.compare_parameter=ParameterId::Gravity;definition.primary_measurement=Some(ParameterId::Gravity);
                    }
                    ui.label(egui::RichText::new("For a fixed mass, g = GM/R² predicts one quarter of the surface gravity at twice the radius. The run measures the actual model output.").small().color(MUTED));
                }
                if read_parameter(initial,definition.measurement_target,ParameterId::Albedo).is_some() {
                    if ui.button("How does reflected sunlight affect temperature?").clicked() {
                        definition.name="Albedo and climate response".into();definition.samples=8;definition.interval_s=86400.0;
                        batch.sweep_enabled=true;batch.sweep_parameter=ParameterId::Albedo;batch.sweep_from=0.1;batch.sweep_to=0.6;batch.sweep_count=6;batch.sweep_target=Some(definition.measurement_target);batch.compare_parameter=ParameterId::Temperature;definition.primary_measurement=Some(ParameterId::Temperature);
                    }
                    ui.label(egui::RichText::new("Albedo is the reflected fraction. The global-mean climate approximation includes greenhouse/feedback fits; this is not a weather forecast.").small().color(MUTED));
                }
                if read_parameter(initial,definition.measurement_target,ParameterId::Steps).is_some() {
                    if ui.button("Does a finer solver change this result?").clicked() {
                        definition.name="Solver convergence exploration".into();definition.samples=16;definition.interval_s=86400.0;
                        batch.sweep_enabled=true;batch.sweep_parameter=ParameterId::Steps;batch.sweep_from=128.0;batch.sweep_to=512.0;batch.sweep_count=4;batch.sweep_target=Some(definition.measurement_target);batch.compare_parameter=ParameterId::SystemEnergy;definition.primary_measurement=Some(ParameterId::SystemEnergy);
                    }
                    ui.label(egui::RichText::new("Compare trajectories and energy across step counts. Agreement is a convergence clue; physical model error and missing forces remain.").small().color(MUTED));
                }
            });
            ui.separator();
            ui.horizontal(|ui| {ui.label("States per run");ui.add(egui::DragValue::new(&mut definition.samples).range(2..=4096));});
            let mut days=definition.interval_s/86400.0;
            ui.horizontal(|ui| {ui.label("Simulation days between states");if ui.add(egui::DragValue::new(&mut days).range(0.000001..=365_250_000.0).speed(0.1)).changed(){definition.interval_s=days*86400.0;}});
            ui.horizontal(|ui| {
                ui.label("Measure on");
                egui::ComboBox::from_id_salt("experiment_measure_target").selected_text(definition.measurement_target.name(initial).unwrap_or("Unavailable")).show_ui(ui,|ui| {
                    for (t,label) in research_targets(initial) {
                        if ui.selectable_value(&mut definition.measurement_target,t,label).changed(){definition.primary_measurement=None;}
                    }
                });
            });
            egui::ComboBox::from_id_salt("experiment_primary_measure").selected_text(definition.primary_measurement.map(|p|p.descriptor().name).unwrap_or("Choose a primary measurement")).show_ui(ui,|ui| {
                for p in PARAMETERS.iter().filter(|p|read_parameter(initial,definition.measurement_target,p.id).is_some()) {
                    if ui.selectable_value(&mut definition.primary_measurement,Some(p.id),p.name).changed(){batch.compare_parameter=p.id;}
                }
            });
            ui.checkbox(&mut batch.sweep_enabled,"Vary one input across independent runs");
            if batch.sweep_enabled {
                let mut input_target=batch.sweep_target.unwrap_or(definition.measurement_target);
                egui::ComboBox::from_id_salt("experiment_input_target").selected_text(format!("Change on {}",input_target.name(initial).unwrap_or("Unavailable"))).show_ui(ui,|ui| {
                    for (t,label) in research_targets(initial) {if ui.selectable_value(&mut input_target,t,label).changed(){
                        if let Some(p)=PARAMETERS.iter().find(|p|p.editable && read_parameter(initial,t,p.id).is_some()){batch.sweep_parameter=p.id;let v=read_parameter(initial,t,p.id).unwrap().value;batch.sweep_from=v;batch.sweep_to=v;}
                    }}
                });
                batch.sweep_target=Some(input_target);
                egui::ComboBox::from_id_salt("experiment_input").selected_text(batch.sweep_parameter.descriptor().name).show_ui(ui,|ui| {
                    for p in PARAMETERS.iter().filter(|p|p.editable && read_parameter(initial,input_target,p.id).is_some()) {
                        if ui.selectable_value(&mut batch.sweep_parameter,p.id,p.name).changed() {
                            let value=read_parameter(initial,input_target,p.id).unwrap().value;batch.sweep_from=value;batch.sweep_to=value;
                        }
                    }
                });
                let p=batch.sweep_parameter.descriptor();
                ui.label(format!("Input values use {} · {}",p.unit,p.limits));
                let (q,unit)=quantity(p.id);
                ui.horizontal(|ui|{ui.label("From");if let Some(v)=units::edit(ui,"sweep_from",batch.sweep_from,q,unit,true,None){batch.sweep_from=v;}});
                ui.horizontal(|ui|{ui.label("To");if let Some(v)=units::edit(ui,"sweep_to",batch.sweep_to,q,unit,true,None){batch.sweep_to=v;}});
                ui.horizontal(|ui|{ui.label("Independent runs");ui.add(egui::DragValue::new(&mut batch.sweep_count).range(2..=64));});
                if batch.sweep_from==batch.sweep_to {ui.label(egui::RichText::new("Both endpoints match: these will be repeated identical inputs.").small().color(MUTED));}
                if previous_grid != (batch.sweep_enabled,batch.sweep_parameter,batch.sweep_target,batch.sweep_from,batch.sweep_to,batch.sweep_count) || definition.sweep.is_none() {
                match ExperimentDefinition::linear_values(batch.sweep_from,batch.sweep_to,batch.sweep_count) {
                    Ok(values)=>definition.sweep=Some(ParameterSweep{target:input_target,parameter:batch.sweep_parameter,values}),
                    Err(e)=>batch.status=Some(e),
                }
                }
            } else {definition.sweep=None;}
            egui::CollapsingHeader::new("Reusable definitions").show(ui,|ui| {
                if ui.button("Save definition and frozen state").clicked() {
                    let path=crate::research::export_path();
                    match definition.save_bundle(initial,&path) {Ok(())=>batch.status=Some(format!("Prepared study saved to {}",path.display())),Err(e)=>batch.status=Some(e)}
                }
                ui.label("Load a definition JSON against this frozen state. Object IDs, version and any pinned state fingerprint are validated before running.");
                ui.text_edit_singleline(&mut batch.definition_path);
                if ui.button("Load definition").clicked() {
                    match ExperimentDefinition::read(std::path::Path::new(&batch.definition_path)) {
                        Ok(d)=>{if let Some(s)=&d.sweep {batch.sweep_target=Some(s.target);batch.sweep_enabled=true;batch.sweep_parameter=s.parameter;batch.sweep_from=s.values.first().copied().unwrap_or(0.0);batch.sweep_to=s.values.last().copied().unwrap_or(0.0);batch.sweep_count=s.values.len();}else{batch.sweep_enabled=false;}batch.compare_parameter=d.primary_measurement.unwrap_or(batch.compare_parameter);*definition=d;batch.status=Some("Definition loaded; run validates it against the frozen state.".into());},
                        Err(e)=>batch.status=Some(e),
                    }
                }
                ui.label(egui::RichText::new("The exported definition preserves fixed overrides and an ordered sweep. Replay it with the exported base.cosmo using the headless tool.").small().color(MUTED));
            });
        });
            ui.label(
                egui::RichText::new(format!(
                    "Requested horizon: {} · {} run(s)",
                    cosmogon_sim::time::format_duration(
                        (definition.samples.saturating_sub(1)) as f64 * definition.interval_s
                    ),
                    definition.runs()
                ))
                .color(MUTED),
            );
            if batch.running() {
                if ui.button("Cancel experiment").clicked() {
                    batch.cancel();
                }
            } else if ui.button("Run and export experiment").clicked() {
                let path = crate::research::export_path();
                if let Err(e) = batch.start(path) {
                    batch.status = Some(e);
                }
            }
        }
    }
    if let Some(p) = &batch.progress {
        ui.label(format!(
            "Completed {} / {} runs · current state {} / {}",
            p.completed_runs, p.total_runs, p.current_sample, p.total_samples
        ));
    }
    if batch.results_mode && !batch.results.is_empty() {
        ui.separator();
        ui.heading("Compare completed runs");
        egui::ComboBox::from_id_salt("experiment_output")
            .selected_text(batch.compare_parameter.descriptor().name)
            .show_ui(ui, |ui| {
                for p in PARAMETERS {
                    if batch
                        .results
                        .iter()
                        .all(|r| r.final_values.contains_key(p.key))
                    {
                        ui.selectable_value(&mut batch.compare_parameter, p.id, p.name);
                    }
                }
            });
        let p = batch.compare_parameter.descriptor();
        egui::Grid::new("experiment_comparison")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Run");
                let input = batch
                    .definition
                    .as_ref()
                    .and_then(|d| d.sweep.as_ref())
                    .map(|s| {
                        format!(
                            "{} ({})",
                            s.parameter.descriptor().name,
                            s.parameter.descriptor().unit
                        )
                    })
                    .unwrap_or_else(|| "Initial input".into());
                ui.strong(input);
                ui.strong(format!("Final ({})", p.unit));
                ui.strong("Δ from run 1");
                ui.end_row();
                for r in &batch.results {
                    ui.label(r.run.to_string());
                    ui.label(
                        r.input_value
                            .map(units::fmt_num)
                            .unwrap_or_else(|| "Baseline".into()),
                    );
                    ui.label(
                        r.final_values
                            .get(p.key)
                            .map(|v| format!("{v:.10e}"))
                            .unwrap_or_else(|| "Unavailable".into()),
                    );
                    ui.label(
                        match (
                            r.final_values.get(p.key),
                            batch.results[0].final_values.get(p.key),
                        ) {
                            (Some(v), Some(first)) => format!("{:+.6e}", v - first),
                            _ => "Unavailable".into(),
                        },
                    );
                    ui.end_row();
                }
            });
        ui.label(egui::RichText::new("Displayed digits resolve numerical differences; they do not imply observational precision. Δ uses the first completed branch, not an unmodified control.").small().color(MUTED));
    }
    if let Some(s) = &batch.status {
        ui.separator();
        ui.label(s);
    }
    if let Some(path) = &batch.output {
        ui.label(
            egui::RichText::new(path.display().to_string())
                .small()
                .color(MUTED),
        );
    }
    ui.label(egui::RichText::new("Exploratory studies: every run starts from the same frozen state; no cross-run carryover or invented measurements. Cancelled/failed batches retain completed exports and mark the batch incomplete.").small().color(MUTED));
}

#[allow(clippy::too_many_arguments)]
pub fn panel(
    mut contexts: EguiContexts,
    mut research: ResMut<Research>,
    mut ui_state: ResMut<UiState>,
    mut sim: ResMut<Sim>,
    rig: Res<CameraRig>,
    time: Res<Time<Real>>,
    mut batch: ResMut<crate::research_batch::ResearchBatch>,
) -> Result {
    if let Some(tab) = ui_state.research_requested.take() {
        research.open = true;
        research.tab = tab;
    }
    if !research.open || ui_state.hidden || ui_state.photo_mode {
        return Ok(());
    }
    let Some(t) = sim.selected.or(rig.focus).map(crate::research::target) else {
        return Ok(());
    };
    if t.name(&sim.universe).is_none() {
        return Ok(());
    }
    research.selection(&sim, t);
    let ctx = contexts.ctx_mut()?;
    let mut open = research.open;
    let mut pending = None;
    let panel_height = (ctx.content_rect().height() - 310.0).clamp(160.0, 430.0);
    egui::Window::new("Control Panel & Research")
        .open(&mut open)
        .frame(egui::Frame::window(&ctx.style()).fill(egui::Color32::from_rgb(13, 17, 25)))
        .default_pos([310.0, 75.0])
        .default_width(650.0)
        .max_height(panel_height + 70.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (tab, name) in [
                    (0, "Controls"),
                    (1, "Models"),
                    (2, "Data"),
                    (3, "Plots & Export"),
                    (4, "Experiments"),
                ] {
                    ui.selectable_value(&mut research.tab, tab, name);
                }
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt(("research_scroll", research.tab))
                .max_height(panel_height)
                .show(ui, |ui| match research.tab {
                    1 => models(ui, &mut research, &sim, t, &mut ui_state),
                    2 => sources(ui, &sim, t),
                    3 => plots(ui, &mut research, &sim, t, time.elapsed_secs_f64()),
                    4 => experiments(ui, &mut batch, &sim, t),
                    _ => pending = control(ui, &mut research, &sim, t),
                });
            if let Some(status) = &research.status {
                ui.separator();
                ui.label(egui::RichText::new(status).small().color(ACCENT));
            }
        });
    research.open = open;
    if let Some(edit) = pending {
        match sim.edit(edit, time.elapsed_secs_f64()) {
            Ok(out) => research.status = Some(out.summary),
            Err(e) => research.status = Some(e),
        }
    }
    Ok(())
}
