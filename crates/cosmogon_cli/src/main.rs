//! Headless runner. Useful for tuning models, CI determinism checks and batch experiments.
//!
//! ```text
//! cosmogon-cli run   [--seed N] [--scenario neighbourhood|garden|sol] [--years Y]
//!                    [--life realistic|hopeful|teeming] [--systems N] [--min-importance I]
//!                    [--save PATH]
//! cosmogon-cli check [--seed N] [--scenario ...] [--years Y]   # determinism self-check
//! cosmogon-cli survey [--seeds N] [--scenario ...] [--years Y] [--life ...]  # outcome statistics
//! ```

use std::time::Instant;

use cosmogon_sim::civ::tech::TechGraph;
use cosmogon_sim::time::{format_duration, group_digits, SECONDS_PER_YEAR};
use cosmogon_sim::universe::LIFE_PRESETS;
use cosmogon_sim::{save, Scenario, Universe, UniverseSettings};

struct Args {
    cmd: String,
    samples: usize,
    interval_days: f64,
    target: String,
    output: Option<String>,
    initial: Option<String>,
    recording: Option<String>,
    reference: Option<String>,
    latitude: f64,
    longitude: f64,
    definition: Option<String>,
    parameter: String,
    sweep_start: Option<f64>,
    sweep_end: Option<f64>,
    sweep_runs: usize,
    measure_target: Option<String>,
    measure: Option<String>,
    seed: u64,
    seeds: u64,
    scenario: Scenario,
    years: f64,
    life: usize,
    systems: u32,
    min_importance: u8,
    save: Option<String>,
}

fn number<T: std::str::FromStr>(text: String, flag: &str) -> Result<T, String> {
    text.parse()
        .map_err(|_| format!("{flag}: invalid number {text}"))
}
fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut a = Args {
        cmd: "run".into(),
        samples: 60,
        interval_days: 1.0,
        target: "Earth".into(),
        output: None,
        initial: None,
        recording: None,
        reference: None,
        latitude: 0.0,
        longitude: 0.0,
        definition: None,
        parameter: "albedo".into(),
        sweep_start: None,
        sweep_end: None,
        sweep_runs: 5,
        measure_target: None,
        measure: None,
        seed: 1,
        seeds: 10,
        scenario: Scenario::Sol,
        years: 200_000.0,
        life: 0,
        systems: 24,
        min_importance: 3,
        save: None,
    };
    let mut it = args;
    let mut explicit_scenario = false;
    if let Some(c) = it.next() {
        a.cmd = c;
    }
    if matches!(a.cmd.as_str(), "--help" | "-h" | "help") {
        a.cmd = "help".into();
        return Ok(a);
    }
    while let Some(flag) = it.next() {
        if matches!(flag.as_str(), "--help" | "-h") {
            a.cmd = "help".into();
            return Ok(a);
        }
        let mut val = || {
            it.next()
                .filter(|v| !v.starts_with("--"))
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--samples" => a.samples = number::<usize>(val()?, &flag)?.clamp(2, 4096),
            "--interval-days" => a.interval_days = number(val()?, &flag)?,
            "--target" => a.target = val()?,
            "--output" => a.output = Some(val()?),
            "--latitude" => a.latitude = number(val()?, &flag)?,
            "--longitude" => a.longitude = number(val()?, &flag)?,
            "--reference" => a.reference = Some(val()?),
            "--recording" => a.recording = Some(val()?),
            "--initial" => a.initial = Some(val()?),
            "--definition" => a.definition = Some(val()?),
            "--parameter" => a.parameter = val()?,
            "--from" => a.sweep_start = Some(number(val()?, &flag)?),
            "--to" => a.sweep_end = Some(number(val()?, &flag)?),
            "--runs" => a.sweep_runs = number(val()?, &flag)?,
            "--measure-target" => a.measure_target = Some(val()?),
            "--measure" => a.measure = Some(val()?),
            "--seed" => a.seed = number(val()?, &flag)?,
            "--seeds" => a.seeds = number(val()?, &flag)?,
            "--years" => a.years = number(val()?, &flag)?,
            "--systems" => a.systems = number(val()?, &flag)?,
            "--min-importance" => a.min_importance = number(val()?, &flag)?,
            "--save" => a.save = Some(val()?),
            "--scenario" => {
                explicit_scenario = true;
                a.scenario = match val()?.as_str() {
                    "neighbourhood" | "neighborhood" => Scenario::Neighbourhood,
                    "garden" => Scenario::GardenWorld,
                    "sol" => Scenario::Sol,
                    "lab" => Scenario::SolarSystemLab,
                    "system" => Scenario::StarSystem,
                    "empty" => Scenario::EmptySystem,
                    name => return Err(format!("Unknown scenario {name}")),
                };
            }
            "--life" => {
                let v = val()?;
                a.life = LIFE_PRESETS
                    .iter()
                    .position(|p| p.0.eq_ignore_ascii_case(&v))
                    .ok_or("Choose life: realistic, hopeful or teeming")?;
            }
            unknown => return Err(format!("Unknown flag {unknown}; use --help")),
        }
    }
    if !explicit_scenario && matches!(a.cmd.as_str(), "experiment" | "sweep") {
        a.scenario = Scenario::SolarSystemLab;
    }
    if matches!(a.cmd.as_str(), "run" | "check" | "survey" | "profile")
        && (!a.years.is_finite() || a.years < 0.0)
    {
        return Err("--years must be finite and nonnegative".into());
    }
    if !(1..=4096).contains(&a.systems) {
        return Err("--systems must be between 1 and 4096".into());
    }
    if a.cmd == "survey" && !(1..=1_000_000).contains(&a.seeds) {
        return Err("--seeds must be between 1 and 1000000".into());
    }
    Ok(a)
}
fn help() {
    println!(
        r#"Cosmogon headless tools
  surface --target Mars          Sample authoritative MOLA height and source-cell coverage JSON
  datasets                       Print embedded MOLA source/version/checksum metadata
  models                         Print model/source/parameter registry JSON
  inspect --recording DIR         Validate and print saved numeric recording data
  experiment --output NEW_DIR    Record a saved or new universe at uniform simulation times
  sweep --output NEW_DIR         Run independent branches from the same starting state
  replay --output NEW_DIR        Run an exported definition against its frozen base
  validate [--output NEW_FILE]   Calculate analytic and cached JPL regression measurements
  run | check | survey | profile Run the released simulation/determinism tools

Surface coordinates: --latitude -90..90 --longitude -360..360 (degrees, east positive)

Research inputs:
  --initial BASE.cosmo           Use a complete saved state; overrides scenario/seed
  --scenario lab|sol|garden|neighbourhood|system|empty --seed N
  --target NAME                  Observed object, or changed object for a sweep
  --samples 2..4096 --interval-days DAYS (86400 seconds per day)
  --parameter KEY --from VALUE --to VALUE --runs 2..64
  --measure-target NAME --measure KEY
  --recording DIR [--reference DIR --measure KEY] (inspect actual rows/differences)
  --definition definition.json   Replay requires this and --initial
Values use canonical units listed by models. Missing uncertainty is not inferred.
Research creation defaults to Solar System Lab. Results are exploratory implemented models;
use measured validation conditions to assess suitability. Existing exports are protected.

Simulation inputs: --years Y --life realistic|hopeful|teeming --systems N
                   --seeds N --min-importance N --save FILE
"#
    );
}

fn settings(a: &Args, seed: u64) -> UniverseSettings {
    let (_, life, intel, _) = LIFE_PRESETS[a.life];
    UniverseSettings {
        seed,
        scenario: a.scenario,
        system_count: a.systems,
        life_rate: life,
        intelligence_rate: intel,
        ..Default::default()
    }
}

fn main() {
    let a = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    match a.cmd.as_str() {
        "help" => help(),
        "datasets" => println!("{}", cosmogon_sim::planet::mola::METADATA),
        "surface" => { if let Err(e) = surface(&a) { eprintln!("Surface probe failed: {e}"); std::process::exit(1); } }
        "models" => println!("{}", serde_json::to_string_pretty(&serde_json::json!({"format":"cosmogon-model-registry", "schema_version":1, "models":cosmogon_sim::research::PhysicsModelRegistry::models(), "sources":cosmogon_sim::research::SCIENTIFIC_SOURCES, "parameters":cosmogon_sim::research::PARAMETERS})).unwrap()),
        "inspect" => {
            let result=(||->Result<serde_json::Value,String>{
                let path=a.recording.as_ref().ok_or("inspect requires --recording DIR")?;
                let table=cosmogon_sim::research::SavedRecording::read(std::path::Path::new(path))?;
                if let Some(reference)=&a.reference {
                    let reference=cosmogon_sim::research::SavedRecording::read(std::path::Path::new(reference))?;
                    let key=a.measure.as_ref().ok_or("Difference inspection requires --measure KEY")?;
                    let parameter=cosmogon_sim::research::PARAMETERS.iter().find(|p|p.key.eq_ignore_ascii_case(key)||format!("{:?}",p.id).eq_ignore_ascii_case(key)).ok_or("Unknown measurement; use models")?;
                    let difference=table.difference_from(&reference,parameter.id)?;
                    Ok(serde_json::json!({"format":"cosmogon-recording-difference","schema_version":1,"parameter":parameter.key,"unit":parameter.unit,"target":table.target_name,"reference":reference.target_name,"points":difference,"alignment":"Identical elapsed sample times since each recording's own start; no interpolation or absolute-epoch alignment","limitations":"Difference is not a physical error estimate or uncertainty. Shared initial conditions are not inferred."}))
                } else {serde_json::to_value(&table).map_err(|e|e.to_string())}
            })();
            match result {Ok(data)=>println!("{}",serde_json::to_string_pretty(&data).unwrap()),Err(e)=>{eprintln!("Recording inspect failed: {e}");std::process::exit(1);}}
        }
        "experiment" => { if let Err(e) = experiment(&a) { eprintln!("Experiment failed: {e}"); std::process::exit(1); } }
        "sweep" | "replay" => { if let Err(e) = batch(&a) { eprintln!("Batch failed: {e}"); std::process::exit(1); } }
        "validate" => {
            let report=cosmogon_sim::research::validation_report();
            if let Some(path)=&a.output {if let Err(e)=report.export(std::path::Path::new(path)){eprintln!("Validation export failed: {e}");std::process::exit(1);}}
            else {println!("{}",serde_json::to_string_pretty(&report).unwrap());}
            if !report.passes_regression {std::process::exit(1);}
        }
        "run" => run(&a),
        "check" => check(&a),
        "survey" => survey(&a),
        "profile" => profile(&a),
        c => { eprintln!("unknown command {c}; use run | check | survey | profile | models | experiment | sweep | replay | validate"); std::process::exit(1); }
    }
}

fn run(a: &Args) {
    let t0 = Instant::now();
    let mut u = Universe::new(settings(a, a.seed));
    println!(
        "Created {} in {:.2?}: {} systems, {} biospheres",
        a.scenario.label(),
        t0.elapsed(),
        u.systems.len(),
        u.biospheres.len()
    );
    let t1 = Instant::now();
    u.advance_by(a.years * SECONDS_PER_YEAR);
    println!(
        "Simulated {} in {:.2?}\n",
        format_duration(a.years * SECONDS_PER_YEAR),
        t1.elapsed()
    );
    for e in u
        .history
        .events
        .iter()
        .filter(|e| e.importance >= a.min_importance)
    {
        println!(
            "[{:>26}] {:<12} {}{}",
            cosmogon_sim::time::format_date(e.time, u.start_time, u.gregorian()),
            e.category.label(),
            e.title,
            if e.importance >= 5 { "  ★" } else { "" }
        );
    }
    let graph = TechGraph::embedded();
    for c in &u.civs {
        println!(
            "\n{} ({}) — {:?}\n  population {}  capacity {}  stability {:.2}  era {}  techs {}/{}  settlements {}  energy {:.2e} W",
            c.name,
            c.species.name,
            c.status,
            group_digits(c.population),
            group_digits(c.capacity),
            c.stability,
            c.era(graph),
            c.discoveries.len(),
            graph.len(),
            c.sites.iter().filter(|s| s.active()).count(),
            c.total_power_w()
        );
        let k: Vec<String> = cosmogon_sim::civ::knowledge::Domain::ALL
            .iter()
            .map(|d| format!("{}={:.2e}", &d.name()[..4], c.knowledge[d.index()]))
            .collect();
        println!("  knowledge: {}", k.join(" "));
    }
    if let Some(path) = &a.save {
        save::write_file(std::path::Path::new(path), &u, "cli").expect("save");
        println!("\nSaved to {path}");
    }
}

fn check(a: &Args) {
    let mut x = Universe::new(settings(a, a.seed));
    let mut y = Universe::new(settings(a, a.seed));
    let total = a.years * SECONDS_PER_YEAR;
    x.advance_by(total);
    let mut done = 0.0;
    let mut i = 0u64;
    while done < total {
        let step = (1.0 + (i % 7) as f64 * 13.7) * SECONDS_PER_YEAR;
        done = (done + step).min(total);
        y.advance_to(y.start_time + done, None, None);
        i += 1;
    }
    let reloaded = save::from_json(&save::to_json(&x, "check").unwrap()).unwrap();
    let same = serde_json::to_string(&x).unwrap() == serde_json::to_string(&y).unwrap();
    let rt = reloaded == x;
    println!("chunking-independent: {same}\nsave round-trip exact: {rt}");
    if !(same && rt) {
        std::process::exit(1);
    }
}

fn survey(a: &Args) {
    let mut life = 0;
    let mut complex = 0;
    let mut intelligent = 0;
    for seed in 0..a.seeds {
        let mut u = Universe::new(settings(a, seed));
        // Geological time in 1 Myr chunks until intelligence appears, then 20 kyr of history.
        let end = u.start_time + a.years * SECONDS_PER_YEAR;
        let mut emerged_at = None;
        while u.time < end && u.civs.is_empty() {
            u.advance_to((u.time + 1.0e6 * SECONDS_PER_YEAR).min(end), None, None);
        }
        if !u.civs.is_empty() {
            emerged_at = Some(u.time - u.start_time);
            u.advance_by(20_000.0 * SECONDS_PER_YEAR);
        }
        if let Some(t) = emerged_at {
            println!("seed {seed:>3}: intelligence after {}", format_duration(t));
        }
        let any = |s| u.biospheres.iter().any(|b| b.stage >= s);
        life += any(cosmogon_sim::life::Stage::Microbial) as u32;
        complex += any(cosmogon_sim::life::Stage::ComplexEcosystems) as u32;
        intelligent += !u.civs.is_empty() as u32;
        let best = u
            .civs
            .iter()
            .map(|c| c.discoveries.len())
            .max()
            .unwrap_or(0);
        println!(
            "seed {seed:>3}: civs {} (most techs {best}), events {}",
            u.civs.len(),
            u.history.events.len()
        );
    }
    println!("\nuniverses with microbial life {life}/{0}, complex life {complex}/{0}, intelligence {intelligent}/{0}", a.seeds);
}

/// Advance in chunks and report wall time per chunk with each civilization's step size.
fn profile(a: &Args) {
    let mut u = Universe::new(settings(a, a.seed));
    let chunk = (a.years / 40.0).max(1.0) * SECONDS_PER_YEAR;
    let end = u.start_time + a.years * SECONDS_PER_YEAR;
    while u.time < end {
        let t0 = Instant::now();
        let r = u.advance_to((u.time + chunk).min(end), None, None);
        let civs: Vec<String> = u
            .civs
            .iter()
            .map(|c| format!("{}:{:?} stride {} pop {} techs {} press f{:.2} d{:.2} w{:.2} short {:.2} cap% {:.0}", c.species.name, c.status, c.stride_years, cosmogon_sim::time::group_digits(c.population), c.discoveries.len(), c.pressures.food, c.pressures.disease, c.pressures.war, c.energy_shortfall, 100.0 * c.population / c.capacity.max(1.0)))
            .collect();
        println!(
            "{:>14} {:>8.1?} steps {:>9}  {}",
            format_duration(u.time - u.start_time),
            t0.elapsed(),
            r.steps,
            civs.join(" | ")
        );
    }
}

/// Offline observer samples use exact requested simulation times; there is no renderer.
fn experiment(a: &Args) -> Result<(), String> {
    use cosmogon_sim::research::Experiment;
    if !a.interval_days.is_finite() || !(1e-6..=3652.5).contains(&a.interval_days) {
        return Err("Choose 0.000001–3652.5 days between samples".into());
    }
    let output = a
        .output
        .as_deref()
        .ok_or("experiment requires --output NEW_DIRECTORY")?;
    if std::path::Path::new(output).exists() {
        return Err("The output directory already exists; choose a new path".into());
    }
    let mut u = initial_universe(a)?;
    let t = find_target(&u, &a.target).ok_or_else(|| format!("No object named {}", a.target))?;
    let mut recording = Experiment::new(&u, t, a.samples)?;
    recording.sampling_policy = format!(
        "Offline observer: uniform requested interval {} days (86400 seconds per day)",
        a.interval_days
    );
    let start = u.time;
    for i in 1..a.samples {
        u.advance_to(start + i as f64 * a.interval_days * 86400.0, None, None);
        recording.capture(&u)?;
    }
    recording.stop_reason = Some("Requested offline sample count reached".into());
    recording.export(std::path::Path::new(output))?;
    println!(
        "Recorded {} states of {} to {}",
        recording.samples.len(),
        recording.target_name,
        output
    );
    Ok(())
}
fn initial_universe(a: &Args) -> Result<Universe, String> {
    if let Some(path) = &a.initial {
        save::read_file(std::path::Path::new(path)).map_err(|e| e.to_string())
    } else {
        Ok(Universe::new(settings(a, a.seed)))
    }
}
fn batch(a: &Args) -> Result<(), String> {
    use cosmogon_sim::research::{ExperimentDefinition, ParameterSweep, PARAMETERS};
    let output = a
        .output
        .as_deref()
        .ok_or("Batch requires --output NEW_DIRECTORY")?;
    let u = initial_universe(a)?;
    let definition = if a.cmd == "replay" {
        if a.initial.is_none() {
            return Err(
                "Replay requires --initial base.cosmo and --definition definition.json".into(),
            );
        }
        ExperimentDefinition::read(std::path::Path::new(
            a.definition
                .as_deref()
                .ok_or("Replay requires --definition definition.json")?,
        ))?
    } else {
        let target =
            find_target(&u, &a.target).ok_or_else(|| format!("No object named {}", a.target))?;
        let parameter = PARAMETERS
            .iter()
            .find(|p| {
                p.key == a.parameter || format!("{:?}", p.id).eq_ignore_ascii_case(&a.parameter)
            })
            .ok_or("Unknown parameter; use a canonical key from models")?
            .id;
        let measured = if let Some(name) = &a.measure_target {
            find_target(&u, name).ok_or_else(|| format!("No measurement object named {name}"))?
        } else {
            target
        };
        let mut d = ExperimentDefinition::new(measured);
        if let Some(key) = &a.measure {
            d.primary_measurement = Some(
                PARAMETERS
                    .iter()
                    .find(|p| p.key == key || format!("{:?}", p.id).eq_ignore_ascii_case(key))
                    .ok_or("Unknown measurement; use a canonical key from models")?
                    .id,
            );
        }
        d.initial_fingerprint = Some(cosmogon_sim::research::state_fingerprint(&u)?);
        d.name = format!(
            "{} {} sweep → {}",
            a.target,
            parameter.descriptor().name,
            measured.name(&u).unwrap_or("measurement")
        );
        d.samples = a.samples;
        d.interval_s = a.interval_days * 86400.0;
        d.sweep = Some(ParameterSweep {
            target,
            parameter,
            values: ExperimentDefinition::linear_values(
                a.sweep_start.ok_or("Sweep requires --from")?,
                a.sweep_end.ok_or("Sweep requires --to")?,
                a.sweep_runs,
            )?,
        });
        d
    };
    let mut last = 0;
    let results = cosmogon_sim::research::run_batch(
        &u,
        &definition,
        std::path::Path::new(output),
        || false,
        |p| {
            if p.completed_runs > last {
                last = p.completed_runs;
                eprintln!("Completed {} / {} runs", last, p.total_runs);
            }
        },
    )?;
    println!("Exported {} independent runs to {}", results.len(), output);
    Ok(())
}
fn find_target(u: &Universe, name: &str) -> Option<cosmogon_sim::research::ResearchTarget> {
    use cosmogon_sim::research::ResearchTarget;
    for (s, sys) in u.systems.iter().enumerate() {
        if sys.star.name.eq_ignore_ascii_case(name) {
            return Some(ResearchTarget::Star(s as u32));
        }
        if let Some((b, _)) = sys
            .bodies
            .iter()
            .enumerate()
            .find(|(_, b)| b.exists() && b.name.eq_ignore_ascii_case(name))
        {
            return Some(ResearchTarget::Body(cosmogon_sim::BodyRef {
                system: s as u32,
                body: b as u32,
            }));
        }
    }
    None
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    fn args(values: &[&str]) -> Result<Args, String> {
        parse_args(values.iter().map(|s| s.to_string()))
    }
    #[test]
    fn malformed_requests_return_errors_and_research_defaults_to_lab() {
        for values in [
            vec!["sweep", "--samples", "bad"],
            vec!["sweep", "--target"],
            vec!["run", "--years", "NaN"],
            vec!["run", "--systems", "999999999"],
            vec!["run", "--scenario", "missing"],
        ] {
            assert!(args(&values).is_err());
        }
        assert_eq!(args(&["sweep"]).unwrap().scenario, Scenario::SolarSystemLab);
        assert_eq!(args(&["run"]).unwrap().scenario, Scenario::Sol);
        assert_eq!(
            args(&["sweep", "--scenario", "sol"]).unwrap().scenario,
            Scenario::Sol
        );
        assert_eq!(args(&["sweep", "--help"]).unwrap().cmd, "help");
    }
}

fn surface(a: &Args) -> Result<(), String> {
    use cosmogon_sim::planet::{mola, terrain::{ElevationData, METRES_PER_UNIT, Terrain, dir_from_lat_lon}};
    let u = if let Some(path) = &a.initial { save::read_file(std::path::Path::new(path)).map_err(|e|e.to_string())? }
        else { Universe::new(settings(a,a.seed)) };
    let body = u.systems.iter().flat_map(|s| &s.bodies).find(|b|b.exists() && b.name.eq_ignore_ascii_case(&a.target)).ok_or("Target body not found")?;
    if body.elevation_data != Some(ElevationData::MarsMola) { return Err("This probe requires a body using the embedded Mars MOLA grid; older procedural saves retain their original terrain".into()); }
    let reading=mola::read(a.latitude,a.longitude)?;
    let authoritative_height_m=Terrain::of(body).elevation(dir_from_lat_lon(a.latitude.to_radians(),a.longitude.to_radians()))*METRES_PER_UNIT;
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"format":"cosmogon-surface-probe","schema_version":1,
        "target":body.name,"reading":reading,"authoritative_height_m":authoritative_height_m,
        "frame":"IAU2000 planetocentric latitude; east longitude; GMM3 degree/order 50 areoid height",
        "sampling":"Bilinear between 0.25-degree grid pixel centres; support cells with zero observations are provider-interpolated",
        "limitations":"Global overview (14.82 km equatorial spacing), not landing safety/local relief. No per-cell uncertainty or propagated covariance. Rendering uses a spherical base radius; areoid shape is not modelled. Colours, climate and subpixel shading are generated/modelled separately."})).map_err(|e|e.to_string())?);
    Ok(())
}
