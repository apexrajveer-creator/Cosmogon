//! Bounded single-parameter experiments through the ordinary habitat edit/scheduler.
use crate::{
    habitat,
    habitat_recording::{Metric, Recording},
    sandbox::Edit,
    Universe,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Parameter {
    #[default]
    FirmPowerW,
    SolarAreaM2,
    BatteryCapacityJ,
    WaterRecoveryFraction,
    TemperatureSetpointK,
    ElectrolysisPowerW,
    ElectrolysisRateKgS,
    SabatierPowerW,
    SabatierRateKgS,
    MoistureAirborneFraction,
    MoistureHumiditySetpoint,
    MoisturePowerW,
    MoistureRateKgS,
    MoistureElectricityJKg,
    MoistureCondensateCapacityKg,
    BreachAreaM2,BreachDischarge,BreachOutsidePressurePa,BreachGamma,
    VolumeM3, HeatCapacityJK, ConductanceWK, RadiatorAreaM2,
}
impl Parameter {
    pub const ALL: [Self; 23] = [
        Self::FirmPowerW,
        Self::SolarAreaM2,
        Self::BatteryCapacityJ,
        Self::WaterRecoveryFraction,
        Self::TemperatureSetpointK,
        Self::ElectrolysisPowerW,
        Self::ElectrolysisRateKgS,
        Self::SabatierPowerW,
        Self::SabatierRateKgS,
        Self::MoistureAirborneFraction,
        Self::MoistureHumiditySetpoint,
        Self::MoisturePowerW,
        Self::MoistureRateKgS,
        Self::MoistureElectricityJKg,
        Self::MoistureCondensateCapacityKg,Self::BreachAreaM2,Self::BreachDischarge,Self::BreachOutsidePressurePa,Self::BreachGamma,
        Self::VolumeM3, Self::HeatCapacityJK, Self::ConductanceWK, Self::RadiatorAreaM2,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::VolumeM3 => "volume_m3", Self::HeatCapacityJK => "heat_capacity_j_k",
            Self::ConductanceWK => "conductance_w_k", Self::RadiatorAreaM2 => "radiator_area_m2",
            Self::BreachAreaM2=>"breach_area_m2",Self::BreachDischarge=>"breach_discharge",Self::BreachOutsidePressurePa=>"breach_outside_pressure_pa",Self::BreachGamma=>"breach_gamma",
            Self::FirmPowerW => "firm_power_w",
            Self::SolarAreaM2 => "solar_area_m2",
            Self::BatteryCapacityJ => "battery_capacity_j",
            Self::WaterRecoveryFraction => "water_recovery_fraction",
            Self::TemperatureSetpointK => "temperature_setpoint_k",
            Self::ElectrolysisPowerW => "electrolysis_power_w",
            Self::ElectrolysisRateKgS => "electrolysis_rate_kg_s",
            Self::SabatierPowerW => "sabatier_power_w",
            Self::SabatierRateKgS => "sabatier_rate_kg_s",
            Self::MoistureAirborneFraction => "moisture_airborne_fraction",
            Self::MoistureHumiditySetpoint => "moisture_humidity_setpoint",
            Self::MoisturePowerW => "moisture_power_w",
            Self::MoistureRateKgS => "moisture_rate_kg_s",
            Self::MoistureElectricityJKg => "moisture_electricity_j_kg",
            Self::MoistureCondensateCapacityKg => "moisture_condensate_capacity_kg",
        }
    }
    pub fn parse(s: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|p| p.id() == s)
            .ok_or_else(|| format!("Unsupported habitat sweep parameter {s}"))
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::VolumeM3 => "Cabin volume (fixed initial gas)",
            Self::HeatCapacityJK => "Effective cabin heat capacity",
            Self::ConductanceWK => "Total envelope heat conductance",
            Self::RadiatorAreaM2 => "Radiator area",
            Self::BreachAreaM2=>"Open breach throat area",Self::BreachDischarge=>"Assumed breach discharge coefficient",Self::BreachOutsidePressurePa=>"Assumed outside receiver pressure",Self::BreachGamma=>"Assumed gas heat-capacity ratio",
            Self::FirmPowerW => "External firm supply",
            Self::SolarAreaM2 => "Horizontal solar panel area",
            Self::BatteryCapacityJ => "Battery capacity (fixed initial energy)",
            Self::WaterRecoveryFraction => "Water recovery fraction",
            Self::TemperatureSetpointK => "Cabin temperature setpoint",
            Self::ElectrolysisPowerW => "Electrolysis equipment power",
            Self::ElectrolysisRateKgS => "Electrolysis oxygen rate",
            Self::SabatierPowerW => "CO₂ reduction equipment power",
            Self::SabatierRateKgS => "CO₂ reduction rate",
            Self::MoistureAirborneFraction => "Crew airborne moisture fraction",
            Self::MoistureHumiditySetpoint => "Relative humidity setpoint",
            Self::MoisturePowerW => "Dehumidifier electric power",
            Self::MoistureRateKgS => "Condensate collection rate",
            Self::MoistureElectricityJKg => "Condensation process electricity",
            Self::MoistureCondensateCapacityKg => {
                "Isolated condensate capacity (fixed initial mass)"
            }
        }
    }
    pub fn unit(self) -> &'static str {
        match self {
            Self::VolumeM3 => "m³", Self::HeatCapacityJK => "J/K", Self::ConductanceWK => "W/K",
            Self::FirmPowerW
            | Self::ElectrolysisPowerW
            | Self::SabatierPowerW
            | Self::MoisturePowerW => "W",
            Self::SolarAreaM2 | Self::BreachAreaM2 | Self::RadiatorAreaM2 => "m²",
            Self::BreachOutsidePressurePa=>"Pa",Self::BreachDischarge | Self::BreachGamma=>"dimensionless",
            Self::BatteryCapacityJ => "J",
            Self::WaterRecoveryFraction
            | Self::MoistureAirborneFraction
            | Self::MoistureHumiditySetpoint => "fraction",
            Self::TemperatureSetpointK => "K",
            Self::ElectrolysisRateKgS | Self::SabatierRateKgS | Self::MoistureRateKgS => "kg/s",
            Self::MoistureElectricityJKg => "J/kg",
            Self::MoistureCondensateCapacityKg => "kg",
        }
    }
    pub fn geometry_or_thermal(self) -> bool {
        matches!(self, Self::VolumeM3 | Self::HeatCapacityJK | Self::ConductanceWK | Self::RadiatorAreaM2)
    }
    pub fn assumption(self) -> Option<&'static str> {
        match self {
            Self::VolumeM3 => Some("Same initial gas mass and temperature, different assumed volume: initial pressure changes. This is a counterfactual cabin, not physical expansion or piston work."),
            Self::HeatCapacityJK => Some("Same initial temperature, different effective heat capacity. Initial implied sensible energy is not held equal; capacity is an assumption, not measured material data."),
            Self::ConductanceWK => Some("Total envelope conductance in W/K, not material conductivity or insulation thickness. No automatic geometry or equipment scaling."),
            Self::RadiatorAreaM2 => Some("Radiating area changes at fixed emissivity, pump limit and initial stores. Actual removal is also limited by available power and thermostat demand."),
            _ => None,
        }
    }
    /// Illustrative two levels within existing bounds, never an optimal/calibrated design.
    pub fn illustrative_levels(self, c: &habitat::Config) -> Option<[f64; 2]> {
        let (current, minimum, maximum, zero_example) = match self {
            Self::VolumeM3 => (c.volume_m3, 1.0, 1e6, 500.0),
            Self::HeatCapacityJK => (c.heat_capacity_j_k, 1e6, 1e12, 50e6),
            Self::ConductanceWK => (c.conductance_w_k, 0.0, 1000.0, 10.0),
            Self::RadiatorAreaM2 => (c.radiator_area_m2, 0.0, 100_000.0, 25.0),
            _ => return None,
        };
        c.validate().ok()?;
        Some([(current * 0.5).clamp(minimum, maximum),
            if current == 0.0 { zero_example } else { (current * 2.0).clamp(minimum, maximum) }])
    }
    pub fn set(self, c: &mut habitat::Config, value: f64) -> Result<(), String> {
        if !value.is_finite() {
            return Err("Sweep values must be finite".into());
        }
        match self {
            Self::VolumeM3 => c.volume_m3 = value,
            Self::HeatCapacityJK => c.heat_capacity_j_k = value,
            Self::ConductanceWK => c.conductance_w_k = value,
            Self::RadiatorAreaM2 => c.radiator_area_m2 = value,
            Self::BreachAreaM2|Self::BreachDischarge|Self::BreachOutsidePressurePa|Self::BreachGamma=>{let b=c.breach.as_mut().ok_or("Enable outward cabin breach before sweeping its assumptions")?;match self {Self::BreachAreaM2=>b.area_m2=value,Self::BreachDischarge=>b.discharge_coefficient=value,Self::BreachOutsidePressurePa=>b.outside_pressure_pa=value,Self::BreachGamma=>b.gamma=value,_=>unreachable!()}},
            Self::FirmPowerW => c.firm_power_w = value,
            Self::SolarAreaM2 => c.solar_area_m2 = value,
            Self::BatteryCapacityJ => c.battery_capacity_j = value,
            Self::WaterRecoveryFraction => c.water_recovery_fraction = value,
            Self::TemperatureSetpointK => c.temperature_setpoint_k = value,
            Self::ElectrolysisPowerW => {
                c.electrolysis
                    .as_mut()
                    .ok_or("Enable electrolysis before sweeping its assumptions")?
                    .power_limit_w = value
            }
            Self::ElectrolysisRateKgS => {
                c.electrolysis
                    .as_mut()
                    .ok_or("Enable electrolysis before sweeping its assumptions")?
                    .oxygen_rate_kg_s = value
            }
            Self::SabatierPowerW => {
                c.sabatier
                    .as_mut()
                    .ok_or("Enable CO₂ reduction before sweeping its assumptions")?
                    .power_limit_w = value
            }
            Self::SabatierRateKgS => {
                c.sabatier
                    .as_mut()
                    .ok_or("Enable CO₂ reduction before sweeping its assumptions")?
                    .co2_rate_kg_s = value
            }
            Self::MoistureAirborneFraction => {
                c.moisture
                    .as_mut()
                    .ok_or("Enable finite cabin moisture before sweeping its assumptions")?
                    .airborne_fraction = value
            }
            Self::MoistureHumiditySetpoint => {
                c.moisture
                    .as_mut()
                    .ok_or("Enable finite cabin moisture before sweeping its assumptions")?
                    .relative_humidity_setpoint = value
            }
            Self::MoisturePowerW => {
                c.moisture
                    .as_mut()
                    .ok_or("Enable finite cabin moisture before sweeping its assumptions")?
                    .power_limit_w = value
            }
            Self::MoistureRateKgS => {
                c.moisture
                    .as_mut()
                    .ok_or("Enable finite cabin moisture before sweeping its assumptions")?
                    .rate_kg_s = value
            }
            Self::MoistureElectricityJKg => {
                c.moisture
                    .as_mut()
                    .ok_or("Enable finite cabin moisture before sweeping its assumptions")?
                    .electricity_j_kg = value
            }
            Self::MoistureCondensateCapacityKg => {
                c.moisture
                    .as_mut()
                    .ok_or("Enable finite cabin moisture before sweeping its assumptions")?
                    .condensate_capacity_kg = value
            }
        }
        c.validate()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub format: String,
    pub schema_version: u32,
    pub parameter: Parameter,
    pub values: Vec<f64>,
    pub samples: usize,
    pub interval_s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<crate::habitat_factorial::Secondary>,
}
impl Plan {
    pub fn linear(
        parameter: Parameter,
        from: f64,
        to: f64,
        runs: usize,
        samples: usize,
        interval_s: f64,
    ) -> Result<Self, String> {
        if !(2..=16).contains(&runs) || !from.is_finite() || !to.is_finite() {
            return Err("A habitat sweep needs 2–16 runs and finite endpoints".into());
        }
        let values = (0..runs)
            .map(|i| {
                if i == runs - 1 {
                    to
                } else {
                    from + (to - from) * i as f64 / (runs - 1) as f64
                }
            })
            .collect();
        let plan = Self {
            format: "cosmogon-habitat-sweep-plan".into(),
            schema_version: 1,
            parameter,
            values,
            samples,
            interval_s,
            secondary: None,
        };
        plan.validate()?;
        Ok(plan)
    }
    /// Preserve legacy catalogs; these plans additionally expose existing total pressure.
    pub fn metrics(&self, h: &habitat::Habitat) -> Vec<Metric> {
        let mut metrics: Vec<_> = Metric::for_habitat(h).collect();
        if self.configuration_assumptions().is_some() && !metrics.contains(&Metric::TotalPressure) {
            metrics.push(Metric::TotalPressure);
        }
        metrics
    }
    pub fn configuration_assumptions(&self) -> Option<serde_json::Value> {
        let settings: Vec<_> = std::iter::once(self.parameter)
            .chain(self.secondary.as_ref().map(|b| b.parameter))
            .filter_map(|p| p.assumption().map(|note| serde_json::json!({"parameter":p.id(),"unit":p.unit(),"assumption":note})))
            .collect();
        if settings.is_empty() {return None;}
        Some(serde_json::json!({"format":"cosmogon-habitat-configuration-assumptions","schema_version":1,
            "settings":settings,"initial_controls":"Actual initial stocks and temperature preserved through ordinary edits; original definition remains unchanged. No gas refill, automatic geometry/equipment scaling or simulated expansion.",
            "capacity_scope":"Effective lumped heat capacity is assumed. Changing it does not preserve equal implied initial thermal energy. Breach support still requires nonnegative structure heat capacity.",
            "sources":[{"url":"https://www.grc.nasa.gov/www/k-12/airplane/eqstat.html","scope":"Ideal gas pV=nRT; supports the fixed-molecule volume/pressure relationship, not measured cabin geometry."},{"url":"https://www.grc.nasa.gov/www/k-12/airplane/specheat.html","scope":"Heat capacity and constant-volume heat/temperature relationships; no calibration of effective cabin capacity or envelope/radiator properties."}],
            "limits":"Deterministic sensitivity at supplied assumptions, not field calibration, predictive uncertainty, physical expansion or equal-energy comparison."}))
    }
    pub(crate) fn with_assumptions(&self, mut report: serde_json::Value) -> serde_json::Value {
        if let Some(note) = self.configuration_assumptions() {report["configuration_assumptions"] = note;}
        report
    }
    pub fn branch_count(&self) -> usize {
        self.values.len() * if self.secondary.is_some() { 2 } else { 1 }
    }
    pub fn validate(&self) -> Result<(), String> {
        let version = if self.secondary.is_some() { 2 } else { 1 };
        if self.format != "cosmogon-habitat-sweep-plan" || self.schema_version != version {
            return Err("Unsupported habitat sweep plan format or schema".into());
        }
        if !(2..=16).contains(&self.values.len())
            || !(1..=1024).contains(&self.samples)
            || self.samples * self.branch_count() > 8192
        {
            return Err(
                "Sweep requires 2–16 runs, 1–1024 samples and at most 8192 total observations"
                    .into(),
            );
        }
        if self
            .values
            .iter()
            .enumerate()
            .any(|(i, v)| !v.is_finite() || self.values[..i].contains(v))
        {
            return Err("Sweep values must be finite and distinct".into());
        }
        if let Some(b) = &self.secondary {
            if self.values.len() != 2
                || b.parameter == self.parameter
                || b.values[0] == b.values[1]
                || b.values.iter().any(|v| !v.is_finite())
            {
                return Err("Paired studies require two distinct settings, each with exactly two finite distinct levels".into());
            }
        }
        if !self.interval_s.is_finite()
            || !(1.0..=30.0 * crate::time::SECONDS_PER_DAY).contains(&self.interval_s)
            || self.interval_s * (self.samples - 1) as f64 > 30.0 * crate::time::SECONDS_PER_DAY
        {
            return Err("Sweep intervals must be finite, at least one second, and within a 30-day total horizon".into());
        }
        Ok(())
    }
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        use std::io::Read;
        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 65536 {
            return Err("Sweep plan must be a regular file within 64 KiB".into());
        }
        let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if !f.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Sweep plan must be a regular file".into());
        }
        let mut bytes = Vec::new();
        f.take(65537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Sweep plan exceeds 64 KiB".into());
        }
        let plan: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        plan.validate()?;
        Ok(plan)
    }
    pub fn validate_world(&self, base: &Universe, id: u64) -> Result<(), String> {
        self.configurations(base, id).map(|_| ())
    }
    pub(crate) fn configurations(
        &self,
        base: &Universe,
        id: u64,
    ) -> Result<Vec<habitat::Config>, String> {
        self.validate()?;
        let h = base
            .habitats
            .iter()
            .find(|h| h.id == id)
            .ok_or("No such habitat in the common starting world")?;
        if !base.time.is_finite()
            || !((base.time + self.interval_s * (self.samples - 1) as f64).is_finite())
            || (self.samples > 1 && base.time + self.interval_s <= base.time)
        {
            return Err("Sweep observation times are not representable in this world".into());
        }
        (0..self.branch_count())
            .map(|i| {
                let value = self.values[i % self.values.len()];
                let mut c = h.config.clone();
                self.parameter.set(&mut c, value)?;
                if let Some(b) = &self.secondary {
                    b.parameter.set(&mut c, b.values[i / 2])?;
                }
                let mut preview = base.clone();
                preview.apply_edit(Edit::ConfigureHabitat {
                    system: h.definition.body.system,
                    id,
                    config: Box::new(c.clone()),
                })?;
                Ok(c)
            })
            .collect()
    }
}
pub fn execute(
    base: &Universe,
    id: u64,
    plan: &Plan,
    path: &std::path::Path,
) -> Result<serde_json::Value, String> {
    use std::io::Write;
    // Preview every ordinary edit before creating output; no silent capacity clamping.
    let configs = plan.configurations(base, id)?;
    let base_text = crate::save::to_json(base, "Habitat sweep common starting world")
        .map_err(|e| e.to_string())?;
    let plan_text = serde_json::to_vec_pretty(plan).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(path)
        .map_err(|e| format!("Existing sweep results are protected; choose a new folder: {e}"))?;
    let write = |name: &str, bytes: &[u8]| -> Result<(), String> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| format!("Incomplete habitat sweep: {e}"))?;
        f.write_all(bytes)
            .map_err(|e| format!("Incomplete habitat sweep: {e}"))
    };
    write("base.cosmo", base_text.as_bytes())?;
    write("plan.json", &plan_text)?;
    let system = base
        .habitats
        .iter()
        .find(|h| h.id == id)
        .expect("validated habitat")
        .definition
        .body
        .system;
    let mut runs = Vec::new();
    let mut paired_recordings = Vec::new();
    for (i, config) in configs.into_iter().enumerate() {
        let mut world = base.clone();
        world.apply_edit(Edit::ConfigureHabitat {
            system,
            id,
            config: Box::new(config),
        })?;
        let initial = world.clone();
        let mut rows = Vec::with_capacity(plan.samples);
        for k in 0..plan.samples {
            world.advance_to(initial.time + k as f64 * plan.interval_s, None, None);
            rows.push(habitat::sample(&world, id)?);
        }
        let folder = format!("run-{i:03}");
        habitat::export(&initial, &world, id, &rows, &path.join(&folder))?;
        let recording = Recording::read(&path.join(&folder))?;
        let final_values = plan.metrics(&rows.last().unwrap().habitat).into_iter()
            .map(|m| {
                (
                    m.id().to_string(),
                    serde_json::json!({"unit":m.unit(),"value":m.value(rows.last().unwrap())}),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();
        let mut run = serde_json::json!({"index":i,"value":plan.values[i%plan.values.len()],"folder":folder,"recording":recording.summary,"final_metrics":final_values});
        if let Some(b) = &plan.secondary {
            run["secondary_value"] = serde_json::json!(b.values[i / 2]);
            paired_recordings.push(recording);
        }
        runs.push(run);
    }
    let mut manifest = serde_json::json!({"format":"cosmogon-habitat-sweep","schema_version":1,"complete":true,"habitat_id":id,"common_state_fingerprint":crate::research::state_fingerprint(base)?,"common_data_versions":crate::save::data_versions(base),"parameter":plan.parameter.id(),"unit":plan.parameter.unit(),"plan":plan,"runs":runs,"mechanism":"Independent clones of one common state; one ordinary configuration edit at the common time; old-configuration remainders complete before the change. Initial stocks are preserved.","limits":"Model sensitivity under assumed equipment and environment, not confidence intervals, optimization proof, causal inference about field hardware or field calibration. Completion manifest is written last; folders lacking it are incomplete."});
    if plan.secondary.is_some() {
        let analysis = crate::habitat_factorial::report(plan, &paired_recordings)?;
        write(
            "factorial.json",
            &serde_json::to_vec_pretty(&analysis).map_err(|e| e.to_string())?,
        )?;
        manifest["schema_version"] = 2.into();
        manifest["factorial_analysis"] = analysis;
        manifest["mechanism"]="Four independent common-state clones; both settings changed in one atomic ordinary edit. Standard order A0B0, A1B0, A0B1, A1B1. Initial stocks and old-configuration remainders preserved.".into();
    }
    manifest = plan.with_assumptions(manifest);
    write(
        "manifest.json",
        &serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> Universe {
        let mut u = Universe::new(crate::UniverseSettings {
            scenario: crate::Scenario::SolarSystemLab,
            ..Default::default()
        });
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(habitat::Definition::at(
                crate::BodyRef { system: 0, body: 3 },
                0.0,
                0.0,
            )),
        })
        .unwrap();
        u
    }
    #[test]
    fn sweep_preserves_common_state_and_replays_exactly() {
        let u = world();
        let before = crate::research::state_fingerprint(&u).unwrap();
        let plan = Plan::linear(Parameter::FirmPowerW, 0., 4000., 3, 3, 3600.).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-habitat-sweep-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let result = execute(&u, 0, &plan, &root.join("a")).unwrap();
        execute(&u, 0, &plan, &root.join("b")).unwrap();
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), before);
        for i in 0..3 {
            let name = format!("run-{i:03}");
            for file in ["samples.json", "samples.csv", "report.json"] {
                assert_eq!(
                    std::fs::read(root.join("a").join(&name).join(file)).unwrap(),
                    std::fs::read(root.join("b").join(&name).join(file)).unwrap()
                );
            }
            for file in ["initial.cosmo", "final.cosmo"] {
                let normalize = |folder: &str| {
                    let mut v: serde_json::Value = serde_json::from_slice(
                        &std::fs::read(root.join(folder).join(&name).join(file)).unwrap(),
                    )
                    .unwrap();
                    v["header"].as_object_mut().unwrap().remove("saved_at_unix");
                    v
                };
                assert_eq!(normalize("a"), normalize("b"));
            }
            let r = Recording::read(&root.join("a").join(name)).unwrap();
            assert_eq!(r.rows[0].habitat.state, u.habitats[0].state);
            let mut expected = u.habitats[0].config.clone();
            expected.firm_power_w = plan.values[i];
            assert_eq!(r.rows[0].habitat.config, expected);
        }
        assert_eq!(result["complete"], true);
        assert!(execute(&u, 0, &plan, &root.join("a")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_all_branch_edits_and_limits_fail_before_output() {
        let u = world();
        let root = std::env::temp_dir().join(format!(
            "cosmogon-habitat-sweep-invalid-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        for (p, a, b) in [
            (Parameter::BatteryCapacityJ, 72e6, 1.),
            (Parameter::ElectrolysisPowerW, 100., 200.),
            (Parameter::WaterRecoveryFraction, 0.8, 1.1),
            (Parameter::FirmPowerW, 10., -1.),
        ] {
            let plan = Plan::linear(p, a, b, 2, 3, 60.).unwrap();
            assert!(execute(&u, 0, &plan, &root).is_err());
            assert!(!root.exists());
        }
        for (a, b, n, s, t) in [
            (0., 0., 2, 3, 60.),
            (0., 1., 17, 3, 60.),
            (0., 1., 16, 1024, 60.),
            (0., 1., 2, 1025, 60.),
            (0., 1., 2, 25, 2. * 86400.),
            (0., f64::NAN, 2, 3, 60.),
        ] {
            assert!(Plan::linear(Parameter::FirmPowerW, a, b, n, s, t).is_err());
        }
        let plan = Plan::linear(Parameter::FirmPowerW, 0., 1., 2, 1, 60.).unwrap();
        let mut future = serde_json::to_value(&plan).unwrap();
        future["schema_version"] = 2.into();
        assert!(serde_json::from_value::<Plan>(future)
            .unwrap()
            .validate()
            .is_err());
    }
    fn moisture_world() -> Universe {
        let mut u = world();
        let mut c = u.habitats[0].config.clone();
        c.moisture = Some(crate::moisture::Config::default());
        c.firm_power_w = 10_000.0;
        c.solar_area_m2 = 0.0;
        c.conductance_w_k = 0.0;
        c.heater_limit_w = 0.0;
        c.radiator_area_m2 = 0.0;
        c.radiator_pump_w = 0.0;
        c.heat_capacity_j_k = 1e9;
        u.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        u.advance_to(u.time + 86400.0, None, None);
        assert!(u.habitats[0].state.water_vapor_kg > 0.0);
        assert!(u.habitats[0].state.condensate_kg > 0.0);
        u
    }
    #[test]
    fn moisture_sweeps_retain_stocks_sources_and_exact_replay() {
        let u = moisture_world();
        let before = crate::research::state_fingerprint(&u).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-moisture-sweeps-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (parameter, a, b) in [
            (Parameter::MoistureAirborneFraction, 0.0, 1.0),
            (Parameter::MoistureHumiditySetpoint, 0.2, 0.8),
            (Parameter::MoisturePowerW, 0.0, 500.0),
            (Parameter::MoistureRateKgS, 0.0, 15.0 / 86400.0),
            (Parameter::MoistureElectricityJKg, 1e6, 3e6),
            (Parameter::MoistureCondensateCapacityKg, 1.0, 20.0),
        ] {
            let plan = Plan::linear(parameter, a, b, 2, 3, 3600.0).unwrap();
            let folder = root.join(parameter.id());
            let manifest = execute(&u, 0, &plan, &folder.join("a")).unwrap();
            execute(&u, 0, &plan, &folder.join("b")).unwrap();
            assert_eq!(
                manifest["common_data_versions"]["cosmogon-cabin-moisture"],
                crate::moisture::MODEL
            );
            for i in 0..2 {
                let name = format!("run-{i:03}");
                let recording = Recording::read(&folder.join("a").join(&name)).unwrap();
                assert_eq!(recording.rows[0].habitat.state, u.habitats[0].state);
                assert_eq!(recording.rows[0].habitat.totals, u.habitats[0].totals);
                let mut expected = u.habitats[0].config.clone();
                parameter.set(&mut expected, plan.values[i]).unwrap();
                assert_eq!(recording.rows[0].habitat.config, expected);
                for file in ["samples.json", "samples.csv", "report.json"] {
                    assert_eq!(
                        std::fs::read(folder.join("a").join(&name).join(file)).unwrap(),
                        std::fs::read(folder.join("b").join(&name).join(file)).unwrap()
                    );
                }
                let h = &recording.rows.last().unwrap().habitat;
                let mb = h.budget.moisture.as_ref().unwrap();
                assert!(
                    (mb.electricity_j
                        - mb.condensed_kg * expected.moisture.as_ref().unwrap().electricity_j_kg)
                        .abs()
                        < 1e-8
                );
                assert!(
                    (h.state.water_vapor_kg - (h.totals.crew_vapor_kg - h.totals.condensed_kg))
                        .abs()
                        < 1e-8
                );
            }
            if parameter == Parameter::MoisturePowerW || parameter == Parameter::MoistureRateKgS {
                let a = Recording::read(&folder.join("a/run-000")).unwrap();
                let b = Recording::read(&folder.join("a/run-001")).unwrap();
                assert_eq!(
                    a.rows.last().unwrap().habitat.totals.condensed_kg,
                    u.habitats[0].totals.condensed_kg
                );
                assert!(
                    b.rows.last().unwrap().habitat.totals.condensed_kg
                        > u.habitats[0].totals.condensed_kg
                );
                let pairs = crate::habitat_comparison::pairs(
                    &a,
                    &b,
                    Metric::Vapor,
                    crate::habitat_comparison::Alignment::Elapsed,
                );
                assert_eq!(pairs[0].b_minus_a, Some(0.0));
                assert!(pairs.last().unwrap().b_minus_a.unwrap() < 0.0);
            }
        }
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(), before);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn airborne_partition_changes_phase_and_sensible_heat_without_creating_water() {
        let mut u = moisture_world();
        let mut c = u.habitats[0].config.clone();
        c.moisture.as_mut().unwrap().power_limit_w = 0.0;
        u.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        let initial = u.habitats[0].state.clone();
        let root = std::env::temp_dir().join(format!(
            "cosmogon-moisture-phase-sweep-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        execute(
            &u,
            0,
            &Plan::linear(Parameter::MoistureAirborneFraction, 0.0, 1.0, 2, 2, 3600.0).unwrap(),
            &root,
        )
        .unwrap();
        let a = Recording::read(&root.join("run-000")).unwrap();
        let b = Recording::read(&root.join("run-001")).unwrap();
        let ah = &a.rows[1].habitat;
        let bh = &b.rows[1].habitat;
        let vapor_delta = bh.state.water_vapor_kg - ah.state.water_vapor_kg;
        let sensible_delta =
            (ah.state.temperature_k - bh.state.temperature_k) * ah.config.heat_capacity_j_k;
        assert!(vapor_delta > 0.0);
        assert!((sensible_delta - vapor_delta * crate::moisture::REFERENCE_PHASE_J_KG).abs() < 0.1);
        for h in [ah, bh] {
            assert!((h.state.mass_kg() - initial.mass_kg()).abs() < 1e-8);
            assert_eq!(
                h.totals.metabolic_person_seconds,
                ah.totals.metabolic_person_seconds
            );
            assert!(h.budget.mass_residual_kg.abs() < 1e-8);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn moisture_branches_reject_disabled_invalid_or_discarded_inventory_before_output() {
        let u = moisture_world();
        let legacy = world();
        let root =
            std::env::temp_dir().join(format!("cosmogon-moisture-invalid-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (p, a, b) in [
            (Parameter::MoistureAirborneFraction, 0.0, 1.01),
            (Parameter::MoistureHumiditySetpoint, 0.5, -0.1),
            (Parameter::MoisturePowerW, 100.0, -1.0),
            (Parameter::MoistureRateKgS, 0.0, -1.0),
            (Parameter::MoistureElectricityJKg, 1e6, 0.0),
            (Parameter::MoistureCondensateCapacityKg, 20.0, 0.0),
        ] {
            let plan = Plan::linear(p, a, b, 2, 3, 60.0).unwrap();
            assert!(execute(&u, 0, &plan, &root).is_err());
            assert!(execute(&legacy, 0, &plan, &root).is_err());
            assert!(!root.exists());
        }
    }
    #[test]
    fn geometry_branches_preserve_actual_gas_and_inverse_volume_pressure() {
        let u=world();
        let before=crate::research::state_fingerprint(&u).unwrap();
        let p=Plan::linear(Parameter::VolumeM3,250.0,1000.0,2,1,60.0).unwrap();
        let root=std::env::temp_dir().join(format!("cosmogon-volume-fixed-gas-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&root);
        let m=execute(&u,0,&p,&root).unwrap();
        let a=Recording::read(&root.join("run-000")).unwrap();
        let b=Recording::read(&root.join("run-001")).unwrap();
        for r in [&a,&b] {
            assert_eq!(r.rows[0].habitat.state,u.habitats[0].state);
            assert_eq!(r.rows[0].habitat.definition,u.habitats[0].definition);
            assert_eq!(p.metrics(&r.rows[0].habitat).len(),19);
        }
        let pa=a.rows[0].habitat.state.pressure_pa(&a.rows[0].habitat.config);
        let pb=b.rows[0].habitat.state.pressure_pa(&b.rows[0].habitat.config);
        assert!((pa/pb-4.0).abs()<1e-12);
        assert!((pa*250.0-pb*1000.0).abs()<1e-7);
        assert_eq!(m["runs"][0]["final_metrics"]["total_pressure"]["value"],pa);
        assert_eq!(m["configuration_assumptions"],p.configuration_assumptions().unwrap());
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(),before);
        let old=Plan::linear(Parameter::FirmPowerW,0.,1.,2,1,60.).unwrap();
        assert_eq!(old.metrics(&u.habitats[0]).len(),18);
        assert!(old.configuration_assumptions().is_none());
        let mut enabled=u.habitats[0].clone();enabled.config.breach=Some(crate::breach::Config::default());
        assert_eq!(p.metrics(&enabled).len(),22);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn thermal_assumptions_replay_records_at_fixed_stocks_and_temperature() {
        let u=moisture_world();
        let before=crate::research::state_fingerprint(&u).unwrap();
        let root=std::env::temp_dir().join(format!("cosmogon-thermal-settings-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&root);
        for (parameter,a,b) in [(Parameter::HeatCapacityJK,5e7,1e9),(Parameter::ConductanceWK,0.,100.),(Parameter::RadiatorAreaM2,0.,50.),(Parameter::VolumeM3,250.,1000.)] {
            let p=Plan::linear(parameter,a,b,2,3,3600.).unwrap();
            let folder=root.join(parameter.id());
            execute(&u,0,&p,&folder.join("a")).unwrap();
            execute(&u,0,&p,&folder.join("b")).unwrap();
            for i in 0..2 {
                let branch=format!("run-{i:03}");
                for file in ["samples.json","samples.csv","report.json"] {
                    assert_eq!(std::fs::read(folder.join("a").join(&branch).join(file)).unwrap(),std::fs::read(folder.join("b").join(&branch).join(file)).unwrap());
                }
                let r=Recording::read(&folder.join("a").join(&branch)).unwrap();
                assert_eq!(r.rows[0].habitat.state,u.habitats[0].state);
                assert_eq!(r.rows[0].habitat.totals,u.habitats[0].totals);
                let mut replay=crate::save::read_file(&folder.join("a").join(&branch).join("initial.cosmo")).unwrap();
                for row in &r.rows {
                    replay.advance_to(row.simulation_time_s,None,None);
                    assert_eq!(habitat::sample(&replay,0).unwrap(),*row);
                    assert!((row.habitat.state.accounted_mass_kg()-u.habitats[0].state.accounted_mass_kg()).abs()<1e-8);
                    assert!(row.habitat.budget.thermal_residual_j.abs()<0.1);
                }
            }
            let aa=Recording::read(&folder.join("a/run-000")).unwrap();
            let bb=Recording::read(&folder.join("a/run-001")).unwrap();
            if parameter==Parameter::HeatCapacityJK {
                let t=u.habitats[0].state.temperature_k;
                assert_ne!(aa.rows[0].habitat.config.heat_capacity_j_k*t,bb.rows[0].habitat.config.heat_capacity_j_k*t);
                assert!(aa.rows[2].habitat.state.temperature_k>bb.rows[2].habitat.state.temperature_k);
            }
            if parameter==Parameter::ConductanceWK {assert!(aa.rows[2].habitat.state.temperature_k>bb.rows[2].habitat.state.temperature_k);}
        }
        assert_eq!(crate::research::state_fingerprint(&u).unwrap(),before);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn geometry_thermal_bounds_and_illustrations_preflight_every_branch() {
        let u=world();
        let root=std::env::temp_dir().join(format!("cosmogon-thermal-invalid-{}",std::process::id()));
        let _=std::fs::remove_dir_all(&root);
        for (parameter,minimum,maximum) in [(Parameter::VolumeM3,1.,1e6),(Parameter::HeatCapacityJK,1e6,1e12),(Parameter::ConductanceWK,0.,1000.),(Parameter::RadiatorAreaM2,0.,100000.)] {
            assert_eq!(Parameter::parse(parameter.id()).unwrap(),parameter);
            assert_eq!(serde_json::to_value(parameter).unwrap(),parameter.id());
            for invalid in [minimum-1.,maximum+1.] {
                let p=Plan::linear(parameter,minimum,invalid,2,1,60.).unwrap();
                assert!(execute(&u,0,&p,&root).is_err());assert!(!root.exists());
            }
            let mut config=u.habitats[0].config.clone();
            for bound in [minimum,maximum] {
                parameter.set(&mut config,bound).unwrap();
                let levels=parameter.illustrative_levels(&config).unwrap();
                assert_ne!(levels[0],levels[1]);
                for level in levels {parameter.set(&mut config,level).unwrap();}
            }
            assert!(parameter.set(&mut config,f64::NAN).is_err());
        }
    }

}
