//! Reduced, explicit habitat budgets. Equipment ratings are user assumptions, not flight
//! hardware. NASA BVAD Rev2 Table 3-31 supplies the balanced nominal crew exchanges.
//! Shortfalls measure unserved demand, never physiological outcomes or deaths.
use crate::{
    history::{Category, Event},
    planet::terrain,
    time::SECONDS_PER_DAY,
    BodyRef, Universe,
};
use cosmogon_core::{
    constants::{SOLAR_LUMINOSITY, STEFAN_BOLTZMANN},
    dmath::DMath,
};
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "cosmogon-habitat-budget/1";
pub const SOURCE: &str =
    "https://ntrs.nasa.gov/api/citations/20210024855/downloads/BVAD_2.15.22-final.pdf";
pub const PERIOD: f64 = 60.0;
pub(crate) const TASK: usize = 4;
const R: f64 = 8.31446261815324;
const O2: f64 = 0.0319988;
const N2: f64 = 0.0280134;
const CO2: f64 = 0.0440095;

/// All equipment/environment settings are experimental design inputs in SI. NASA's crew
/// rates are fixed by MODEL; they do not certify these illustrative equipment ratings.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<crate::crop_process::Config>,
    pub crew: u32,
    pub volume_m3: f64,
    pub firm_power_w: f64,
    pub solar_area_m2: f64,
    pub solar_efficiency: f64,
    pub atmospheric_transmission: f64,
    pub battery_capacity_j: f64,
    pub charge_limit_w: f64,
    pub discharge_limit_w: f64,
    pub charge_efficiency: f64,
    pub discharge_efficiency: f64,
    pub base_load_w: f64,
    pub scrubber_power_w: f64,
    pub scrubber_rate_kg_s: f64,
    pub capture_capacity_kg: f64,
    pub recycler_power_w: f64,
    pub recycler_rate_kg_s: f64,
    pub water_recovery_fraction: f64,
    pub oxygen_makeup_rate_kg_s: f64,
    pub oxygen_setpoint_pa: f64,
    pub heater_limit_w: f64,
    pub radiator_pump_w: f64,
    pub radiator_area_m2: f64,
    pub radiator_emissivity: f64,
    pub heat_capacity_j_k: f64,
    pub conductance_w_k: f64,
    pub temperature_setpoint_k: f64,
    pub oxygen_alert_pa: f64,
    pub co2_alert_pa: f64,
    pub cold_alert_k: f64,
    pub hot_alert_k: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electrolysis: Option<crate::electrolysis::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sabatier: Option<crate::sabatier::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moisture: Option<crate::moisture::Config>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breach: Option<crate::breach::Config>,
    /// Finite general storage for the four transferable supply stocks. Absent legacy
    /// definitions keep the historical 1e7 kg general bound byte-exactly.
    #[serde(default = "default_storage_capacity", skip_serializing_if = "is_default_storage_capacity")]
    pub oxygen_reserve_capacity_kg: f64,
    #[serde(default = "default_storage_capacity", skip_serializing_if = "is_default_storage_capacity")]
    pub potable_water_capacity_kg: f64,
    #[serde(default = "default_storage_capacity", skip_serializing_if = "is_default_storage_capacity")]
    pub food_dry_capacity_kg: f64,
    #[serde(default = "default_storage_capacity", skip_serializing_if = "is_default_storage_capacity")]
    pub food_water_capacity_kg: f64,
}
fn default_storage_capacity() -> f64 {
    1.0e7
}
fn is_default_storage_capacity(v: &f64) -> bool {
    *v == 1.0e7
}
impl Default for Config {
    fn default() -> Self {
        Self {
            crew: 4,
            volume_m3: 500.0,
            firm_power_w: 4000.0,
            solar_area_m2: 40.0,
            solar_efficiency: 0.25,
            atmospheric_transmission: 0.7,
            battery_capacity_j: 72_000_000.0,
            charge_limit_w: 5000.0,
            discharge_limit_w: 5000.0,
            charge_efficiency: 0.95,
            discharge_efficiency: 0.95,
            base_load_w: 500.0,
            scrubber_power_w: 300.0,
            scrubber_rate_kg_s: 6.0 / SECONDS_PER_DAY,
            capture_capacity_kg: 100.0,
            recycler_power_w: 300.0,
            recycler_rate_kg_s: 25.0 / SECONDS_PER_DAY,
            water_recovery_fraction: 0.9,
            oxygen_makeup_rate_kg_s: 10.0 / SECONDS_PER_DAY,
            oxygen_setpoint_pa: 21_200.0,
            heater_limit_w: 2000.0,
            radiator_pump_w: 200.0,
            radiator_area_m2: 25.0,
            radiator_emissivity: 0.85,
            heat_capacity_j_k: 50_000_000.0,
            conductance_w_k: 10.0,
            temperature_setpoint_k: 293.15,
            oxygen_alert_pa: 18_000.0,
            co2_alert_pa: 400.0,
            cold_alert_k: 283.15,
            hot_alert_k: 303.15,
            crop: None,
            electrolysis: None,
            sabatier: None,
            moisture: None,
            breach: None,
            oxygen_reserve_capacity_kg: default_storage_capacity(),
            potable_water_capacity_kg: default_storage_capacity(),
            food_dry_capacity_kg: default_storage_capacity(),
            food_water_capacity_kg: default_storage_capacity(),
        }
    }
}
fn bounds(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be finite and between {min} and {max}"))
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.crew > 1000 {
            return Err("Crew demand supports 0–1000 reference people".into());
        }
        bounds(self.volume_m3, 1.0, 1.0e6, "Cabin volume (m³)")?;
        for (v, n) in [
            (self.firm_power_w, "Firm power"),
            (self.charge_limit_w, "Charge limit"),
            (self.discharge_limit_w, "Discharge limit"),
            (self.base_load_w, "Base load"),
            (self.scrubber_power_w, "Scrubber power"),
            (self.recycler_power_w, "Recycler power"),
            (self.heater_limit_w, "Heater power"),
            (self.radiator_pump_w, "Radiator pump power"),
        ] {
            bounds(v, 0.0, 1.0e6, n)?;
        }
        for (v, n) in [
            (self.solar_area_m2, "Solar area"),
            (self.radiator_area_m2, "Radiator area"),
        ] {
            bounds(v, 0.0, 100_000.0, n)?;
        }
        for (v, n) in [
            (self.solar_efficiency, "PV efficiency"),
            (self.atmospheric_transmission, "Atmospheric transmission"),
            (self.water_recovery_fraction, "Water recovery"),
            (self.radiator_emissivity, "Radiator emissivity"),
        ] {
            bounds(v, 0.0, 1.0, n)?;
        }
        for (v, n) in [
            (self.charge_efficiency, "Charge efficiency"),
            (self.discharge_efficiency, "Discharge efficiency"),
        ] {
            bounds(v, 0.01, 1.0, n)?;
        }
        bounds(self.battery_capacity_j, 0.0, 1.0e12, "Battery capacity (J)")?;
        bounds(
            self.capture_capacity_kg,
            0.0,
            1.0e7,
            "CO₂ capture capacity (kg)",
        )?;
        for (v, n) in [
            (self.oxygen_reserve_capacity_kg, "O₂ reserve storage capacity"),
            (self.potable_water_capacity_kg, "Potable water storage capacity"),
            (self.food_dry_capacity_kg, "Dry food storage capacity"),
            (self.food_water_capacity_kg, "Food water storage capacity"),
        ] {
            bounds(v, 0.0, 1.0e7, n)?;
        }
        for (v, n) in [
            (self.scrubber_rate_kg_s, "CO₂ capture rate"),
            (self.recycler_rate_kg_s, "Water processing rate"),
            (self.oxygen_makeup_rate_kg_s, "O₂ makeup rate"),
        ] {
            bounds(v, 0.0, 100.0, n)?;
        }
        bounds(self.heat_capacity_j_k, 1.0e6, 1.0e12, "Heat capacity (J/K)")?;
        bounds(self.conductance_w_k, 0.0, 1000.0, "Conductance (W/K)")?;
        for (v, n) in [
            (self.temperature_setpoint_k, "Temperature setpoint"),
            (self.cold_alert_k, "Cold alert"),
            (self.hot_alert_k, "Hot alert"),
        ] {
            bounds(v, 100.0, 500.0, n)?;
        }
        if self.cold_alert_k >= self.hot_alert_k {
            return Err("Cold alert must be below hot alert".into());
        }
        for (v, n) in [
            (self.oxygen_setpoint_pa, "O₂ setpoint"),
            (self.oxygen_alert_pa, "O₂ alert"),
            (self.co2_alert_pa, "CO₂ alert"),
        ] {
            bounds(v, 0.0, 200_000.0, n)?;
        }
        if let Some(c) = &self.crop {c.validate()?;if self.moisture.is_none(){return Err("Finite crop vapor requires enabled cabin moisture accounting".into());}}
        if let Some(c) = &self.electrolysis {c.validate()?;}
        if let Some(c) = &self.sabatier {c.validate()?;}
        if let Some(c) = &self.moisture {c.validate()?;}
        if let Some(c) = &self.breach {c.validate()?;}
        // Internal oxygen production is gated only by equipment product capacities,
        // so the reserve tank must at least cover every installed oxygen ceiling.
        if let Some(c) = &self.crop {
            if c.oxygen_capacity_kg > self.oxygen_reserve_capacity_kg {
                return Err("O₂ reserve storage capacity is below installed crop oxygen capacity".into());
            }
        }
        if let Some(e) = &self.electrolysis {
            if e.oxygen_capacity_kg > self.oxygen_reserve_capacity_kg {
                return Err("O₂ reserve storage capacity is below installed electrolysis oxygen capacity".into());
            }
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Initial {
    #[serde(default, skip_serializing_if = "zero")]
    pub captured_co2_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub crop_nutrient_equivalent_kg: f64,
    pub temperature_k: f64,
    pub oxygen_pa: f64,
    pub nitrogen_pa: f64,
    pub co2_pa: f64,
    pub battery_j: f64,
    pub oxygen_reserve_kg: f64,
    pub potable_water_kg: f64,
    pub food_dry_kg: f64,
    pub food_water_kg: f64,
}
impl Default for Initial {
    fn default() -> Self {
        Self {
            captured_co2_kg: 0.0,
            crop_nutrient_equivalent_kg: 0.0,
            temperature_k: 293.15,
            oxygen_pa: 21_200.0,
            nitrogen_pa: 79_900.0,
            co2_pa: 40.0,
            battery_j: 36_000_000.0,
            oxygen_reserve_kg: 100.0,
            potable_water_kg: 200.0,
            food_dry_kg: 100.0,
            food_water_kg: 95.0,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub model: String,
    pub name: String,
    pub body: BodyRef,
    pub latitude_deg: f64,
    pub longitude_east_deg: f64,
    pub config: Config,
    pub initial: Initial,
    /// Passive full-interval heat accounting. Absent legacy definitions stay disabled.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub record_heat: bool,
}
impl Definition {
    pub fn at(body: BodyRef, latitude_deg: f64, longitude_east_deg: f64) -> Self {
        Self {
            model: MODEL.into(),
            name: "Surface habitat experiment".into(),
            body,
            latitude_deg,
            longitude_east_deg,
            config: Config::default(),
            initial: Initial::default(),
            record_heat: true,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.model != MODEL {
            return Err(format!("Unsupported habitat model {}", self.model));
        }
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err("Habitat name must contain 1–120 bytes".into());
        }
        bounds(self.latitude_deg, -90.0, 90.0, "Latitude")?;
        bounds(self.longitude_east_deg, 0.0, 360.0, "East longitude")?;
        self.config.validate()?;
        let i = &self.initial;
        bounds(i.temperature_k, 100.0, 500.0, "Initial temperature")?;
        for (v, n) in [
            (i.oxygen_pa, "Initial O₂ pressure"),
            (i.nitrogen_pa, "Initial N₂ pressure"),
            (i.co2_pa, "Initial CO₂ pressure"),
        ] {
            bounds(v, 0.0, 200_000.0, n)?;
        }
        bounds(
            i.battery_j,
            0.0,
            self.config.battery_capacity_j,
            "Initial battery energy",
        )?;
        for (v, n) in [
            (i.oxygen_reserve_kg, "O₂ reserve"),
            (i.potable_water_kg, "Potable water"),
            (i.food_dry_kg, "Dry food"),
            (i.food_water_kg, "Food water"),
        ] {
            bounds(v, 0.0, 1.0e7, n)?;
        }
        for (stock, capacity, n) in [
            (i.oxygen_reserve_kg, self.config.oxygen_reserve_capacity_kg, "Initial O₂ reserve"),
            (
                i.potable_water_kg,
                self.config.potable_water_capacity_kg,
                "Initial potable water",
            ),
            (i.food_dry_kg, self.config.food_dry_capacity_kg, "Initial dry food"),
            (
                i.food_water_kg,
                self.config.food_water_capacity_kg,
                "Initial food water",
            ),
        ] {
            if stock > capacity {
                return Err(format!("{n} exceeds its storage capacity"));
            }
        }
        bounds(i.captured_co2_kg,0.,self.config.capture_capacity_kg,"Initial captured CO2")?;
        bounds(i.crop_nutrient_equivalent_kg,0.,1e7,"Initial crop nutrient-equivalent")?;
        if self.config.crop.is_none() && i.crop_nutrient_equivalent_kg!=0. {return Err("Initial crop nutrients require an explicit installed crop".into());}
        if let Some(c)=&self.config.crop {if i.oxygen_reserve_kg>c.oxygen_capacity_kg {return Err("Initial oxygen reserve exceeds crop product capacity".into());}}
        if let Some(c) = &self.config.electrolysis {
            if i.oxygen_reserve_kg > c.oxygen_capacity_kg {return Err("Initial O₂ reserve exceeds electrolysis product capacity".into());}
        }
        Ok(())
    }
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        use std::io::Read;
        let metadata=std::fs::metadata(path).map_err(|e|e.to_string())?;
        if !metadata.is_file() || metadata.len()>65_536 {return Err("Habitat definition must be a regular file at most 64 KiB".into());}
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65_536 {
            return Err("Habitat definition exceeds 64 KiB".into());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        value.validate()?;
        Ok(value)
    }
}
fn zero(v: &f64) -> bool {*v == 0.0}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<crate::crop_process::State>,
    pub temperature_k: f64,
    pub oxygen_kg: f64,
    pub nitrogen_kg: f64,
    pub co2_kg: f64,
    pub oxygen_reserve_kg: f64,
    pub potable_water_kg: f64,
    pub wastewater_kg: f64,
    pub brine_kg: f64,
    pub food_dry_kg: f64,
    pub food_water_kg: f64,
    pub dry_waste_kg: f64,
    pub captured_co2_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub hydrogen_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub methane_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub sabatier_water_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub water_vapor_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub condensate_kg: f64,
    /// Cumulative gas outside the cabin, never available to cabin consumers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escaped_gas: Option<crate::breach::Gas>,
    pub battery_j: f64,
}
impl State {
    pub fn mass_kg(&self) -> f64 {
        self.oxygen_kg
            + self.nitrogen_kg
            + self.co2_kg
            + self.oxygen_reserve_kg
            + self.potable_water_kg
            + self.wastewater_kg
            + self.brine_kg
            + self.food_dry_kg
            + self.food_water_kg
            + self.dry_waste_kg
            + self.captured_co2_kg
            + self.hydrogen_kg
            + self.methane_kg
            + self.sabatier_water_kg
            + self.water_vapor_kg + self.condensate_kg
            + self.crop.as_ref().map_or(0.,|c|c.mass_kg())
    }
    pub fn accounted_mass_kg(&self)->f64 {self.mass_kg()+self.escaped_gas.map(|g|g.mass_kg()).unwrap_or(0.0)}
    pub fn cabin_gas(&self)->crate::breach::Gas {crate::breach::Gas {oxygen_kg:self.oxygen_kg,nitrogen_kg:self.nitrogen_kg,co2_kg:self.co2_kg,vapor_kg:self.water_vapor_kg}}
    pub fn oxygen_pa(&self, c: &Config) -> f64 {
        self.oxygen_kg / O2 * R * self.temperature_k / c.volume_m3
    }
    pub fn co2_pa(&self, c: &Config) -> f64 {
        self.co2_kg / CO2 * R * self.temperature_k / c.volume_m3
    }
    pub fn vapor_pressure_pa(&self,c:&Config)->f64 {crate::moisture::vapor_pressure_pa(self.water_vapor_kg,self.temperature_k,c.volume_m3)}
    pub fn relative_humidity(&self,c:&Config)->Option<f64> {crate::moisture::relative_humidity(self.water_vapor_kg,self.temperature_k,c.volume_m3)}
    pub fn phase_energy_j(&self)->f64 {self.water_vapor_kg*crate::moisture::REFERENCE_PHASE_J_KG}
    pub fn pressure_pa(&self, c: &Config) -> f64 {
        (self.oxygen_kg / O2 + self.nitrogen_kg / N2 + self.co2_kg / CO2) * R * self.temperature_k
            / c.volume_m3 + self.vapor_pressure_pa(c)
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Totals {
    pub generated_j: f64,
    pub delivered_j: f64,
    pub losses_j: f64,
    pub curtailed_j: f64,
    pub unserved_j: f64,
    pub unserved_person_seconds: f64,
    pub metabolic_person_seconds: f64,
    /// Cumulative reference enthalpy transferred, not current recoverable chemical stock.
    #[serde(default, skip_serializing_if = "zero")]
    pub reaction_enthalpy_j: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub sabatier_co2_consumed_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub sabatier_heat_j: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub sabatier_water_processed_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub condensate_processed_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub crew_vapor_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub condensed_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub breach_enthalpy_j: f64,
}
/// Last completed interval; instantaneous estimates are not silently mixed with these.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Budget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<crate::crop_process::Budget>,
    pub seconds: f64,
    pub configuration: Option<Config>,
    pub environment_available: bool,
    pub temperature_domain_supported: bool,
    pub horizontal_irradiance_w_m2: Option<f64>,
    pub exterior_temperature_k: Option<f64>,
    pub generated_w: f64,
    pub requested_w: f64,
    pub delivered_w: f64,
    pub unserved_w: f64,
    pub battery_losses_w: f64,
    pub curtailed_w: f64,
    pub oxygen_makeup_kg: f64,
    pub co2_captured_kg: f64,
    pub water_processed_kg: f64,
    pub water_recovered_kg: f64,
    pub served_cycle_fraction: f64,
    pub metabolic_heat_w: f64,
    pub heater_w: f64,
    pub radiator_rejection_w: f64,
    pub conduction_w: f64,
    pub thermal_storage_w: f64,
    pub mass_residual_kg: f64,
    pub electric_residual_j: f64,
    pub thermal_residual_j: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electrolysis: Option<crate::electrolysis::Budget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sabatier: Option<crate::sabatier::Budget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moisture: Option<crate::moisture::Budget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breach: Option<crate::breach::Budget>,
    #[serde(default, skip_serializing_if = "zero")]
    pub condensate_processed_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub condensate_recovered_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub sabatier_water_processed_kg: f64,
    #[serde(default, skip_serializing_if = "zero")]
    pub sabatier_water_recovered_kg: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Environment {
    pub irradiance_w_m2: Option<f64>,
    pub exterior_temperature_k: Option<f64>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Habitat {
    pub id: u64,
    pub definition: Definition,
    pub config: Config,
    pub state: State,
    pub totals: Totals,
    pub budget: Budget,
    pub last_time: f64,
    pub alert_bits: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heat: Option<crate::habitat_heat::Ledger>,
    /// Applied atomic cargo receipts from co-located supported craft transfers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<crate::vehicle_transfer::Receipt>,
}
impl Habitat {
    pub fn new(id: u64, definition: Definition, time: f64) -> Result<Self, String> {
        bounds(time, -1.0e16, 1.0e16, "Habitat clock")?;
        if ((time + PERIOD) - time - PERIOD).abs() > 0.001 {
            return Err("Habitat model requires a clock resolving fixed minute intervals".into());
        }
        definition.validate()?;
        let c = &definition.config;
        let i = &definition.initial;
        let mass = |p: f64, m: f64| p * c.volume_m3 * m / (R * i.temperature_k);
        let state = State {
            crop: c.crop.as_ref().map(|crop|crate::crop_process::State::new(crop,i.crop_nutrient_equivalent_kg,time)).transpose()?,
            temperature_k: i.temperature_k,
            oxygen_kg: mass(i.oxygen_pa, O2),
            nitrogen_kg: mass(i.nitrogen_pa, N2),
            co2_kg: mass(i.co2_pa, CO2),
            oxygen_reserve_kg: i.oxygen_reserve_kg,
            potable_water_kg: i.potable_water_kg,
            wastewater_kg: 0.0,
            brine_kg: 0.0,
            food_dry_kg: i.food_dry_kg,
            food_water_kg: i.food_water_kg,
            dry_waste_kg: 0.0,
            captured_co2_kg: i.captured_co2_kg,
            hydrogen_kg: 0.0,
            methane_kg: 0.0,
            sabatier_water_kg: 0.0,
            water_vapor_kg:0.0,condensate_kg:0.0,escaped_gas:None,
            battery_j: i.battery_j,
        };
        let mut heat=definition.record_heat.then(||crate::habitat_heat::Ledger::new(time,c.heat_capacity_j_k));
        if c.crop.is_some(){if let Some(l)=&mut heat {l.crop_storage_debits_j=Some([0.;2]);}}
        Ok(Self {
            id,
            config: definition.config.clone(),
            definition,
            state,
            totals: Totals::default(),
            budget: Budget::default(),
            last_time: time,
            alert_bits: 0,
            heat,
            receipts: vec![],
        })
    }
    pub(crate) fn validate(&self, time: f64) -> Result<(), String> {
        self.definition.validate()?;
        crate::vehicle_transfer::validate_receipts(self, time)?;
        if self.definition.record_heat != self.heat.is_some() {return Err("Heat ledger and original definition disagree".into());}
        if let Some(heat)=&self.heat {heat.validate(self.last_time)?;}
        self.check_config(&self.config)?;
        bounds(
            self.last_time,
            time - PERIOD - 0.001,
            time,
            "Habitat state time",
        )?;
        bounds(
            self.state.temperature_k,
            1.0,
            1.0e8,
            "Cabin model temperature",
        )?;
        let s = &self.state;
        if let Some(crop)=&s.crop {
            crop.validate(self.last_time)?;
            if let Some(original)=self.definition.config.crop.as_ref() {
                if original.crop!=crop.source_crop || original.dry_chemical_energy_j_kg!=crop.dry_chemical_energy_j_kg || self.definition.initial.crop_nutrient_equivalent_kg!=crop.initial_nutrient_equivalent_kg{return Err("Installed crop source/initial stock differs from original setup".into());}
            } else {
                if let Some(live)=self.config.crop.as_ref() {
                    if live.crop!=crop.source_crop || live.dry_chemical_energy_j_kg!=crop.dry_chemical_energy_j_kg{return Err("Runtime-installed crop differs from its retained configuration".into());}
                }
                bounds(crop.initial_nutrient_equivalent_kg,0.,1e7,"Runtime crop initial nutrient-equivalent")?;
            }
        }
        if let Some(l)=&self.heat {
            match (&s.crop,l.crop_storage_debits_j) {
                (Some(c),Some(v)) => {for (actual,wanted) in v.into_iter().zip([-c.chemical_energy_j(),-c.vapor_generated_kg*crate::moisture::REFERENCE_PHASE_J_KG]) {if (actual-wanted).abs()>1e-8*wanted.abs().max(1.) {return Err("Crop heat counters disagree with actual cumulative storage".into());}}}
                (None,None)=>{},_=>return Err("Crop heat supplement and installed state disagree".into())
            }
        }
        if self.definition.config.crop.is_some() && s.crop.is_none(){return Err("Original installed crop state is missing".into());}
        if let Some(b)=&self.budget.crop {let c=self.budget.configuration.as_ref().and_then(|c|c.crop.as_ref()).ok_or("Crop interval has no retained configuration")?;s.crop.as_ref().ok_or("Crop interval lacks installed state")?.validate_budget(b,c,self.budget.seconds,self.last_time)?;}
        for value in [
            s.oxygen_kg,
            s.nitrogen_kg,
            s.co2_kg,
            s.oxygen_reserve_kg,
            s.potable_water_kg,
            s.wastewater_kg,
            s.brine_kg,
            s.food_dry_kg,
            s.food_water_kg,
            s.dry_waste_kg,
            s.captured_co2_kg,
            s.hydrogen_kg,
            s.methane_kg,
            s.sabatier_water_kg, s.water_vapor_kg, s.condensate_kg,
            s.battery_j,
        ] {
            bounds(value, 0.0, 1.0e12, "Stored habitat quantity")?;
        }
        for (stock, capacity, n) in [
            (
                s.oxygen_reserve_kg,
                self.config.oxygen_reserve_capacity_kg,
                "O₂ reserve",
            ),
            (
                s.potable_water_kg,
                self.config.potable_water_capacity_kg,
                "Potable water",
            ),
            (s.food_dry_kg, self.config.food_dry_capacity_kg, "Dry food"),
            (
                s.food_water_kg,
                self.config.food_water_capacity_kg,
                "Food water",
            ),
        ] {
            if stock > capacity {
                return Err(format!("{n} exceeds its storage capacity"));
            }
        }
        if let Some(g)=s.escaped_gas {
            g.validate()?;
            let mut original=Self::new(self.id,self.definition.clone(),self.last_time)?.state.mass_kg();
            if self.definition.config.crop.is_none() {if let Some(c)=&s.crop {original+=c.initial_nutrient_equivalent_kg;}}
            if (s.accounted_mass_kg()-original).abs()>1e-8*original.max(1.0) {return Err("Accounted cabin + escaped mass disagrees with original finite inventory".into());}
        }
        let t = &self.totals;
        for value in [
            t.generated_j,
            t.delivered_j,
            t.losses_j,
            t.curtailed_j,
            t.unserved_j,
            t.metabolic_person_seconds,
            t.unserved_person_seconds,
            t.reaction_enthalpy_j,
            t.sabatier_co2_consumed_kg,
            t.sabatier_heat_j,
            t.sabatier_water_processed_kg, t.condensate_processed_kg,t.crew_vapor_kg,t.condensed_kg,t.breach_enthalpy_j,
        ] {
            bounds(value, 0.0, 1.0e300, "Cumulative habitat budget")?;
        }
        if self.alert_bits >= 128 {
            return Err("Unsupported habitat alert bits".into());
        }
        if let Some(b) = &self.budget.electrolysis {
            b.validate(self.budget.seconds)?;
            if self.budget.configuration.as_ref().and_then(|c|c.electrolysis.as_ref()).is_none() {
                return Err("Electrolysis interval lacks its equipment configuration".into());
            }
        }
        if let Some(b) = &self.budget.sabatier {
            b.validate(self.budget.seconds)?;
            let c=self.budget.configuration.as_ref().and_then(|c|c.sabatier.as_ref()).ok_or("Sabatier interval lacks its equipment configuration")?;
            if b.requested_w>c.power_limit_w+1e-6 || b.co2_consumed_kg>c.co2_rate_kg_s*self.budget.seconds+1e-9
                || b.delivered_w>self.budget.delivered_w+1e-6
                || (b.electricity_j-b.co2_consumed_kg*c.electricity_j_kg_co2).abs()>1e-8*b.electricity_j.max(1.0) {
                return Err("Sabatier interval disagrees with retained equipment or bus budget".into());
            }
        }
        if let Some(b)=&self.budget.moisture {
            let c=self.budget.configuration.as_ref().and_then(|c|c.moisture.as_ref()).ok_or("Moisture interval lacks its configuration")?;
            b.validate(c,self.budget.seconds)?;
            let cabin=self.budget.configuration.as_ref().unwrap();
            let expected=cabin.crew as f64*self.budget.seconds/SECONDS_PER_DAY*self.budget.served_cycle_fraction*2.946*c.airborne_fraction;
            if (expected-b.crew_vapor_kg).abs()>1e-8*expected.max(1.0) {return Err("Moisture crew partition disagrees with retained interval".into());}
            if b.delivered_w>self.budget.delivered_w+1e-6 {return Err("Moisture interval exceeds bus budget".into());}
        }
        if let Some(b)=&self.budget.breach {
            let cabin=self.budget.configuration.as_ref().ok_or("Breach interval lacks cabin configuration")?;
            let c=cabin.breach.as_ref().ok_or("Breach interval lacks breach configuration")?;
            b.validate(c,self.budget.seconds,cabin.volume_m3,cabin.heat_capacity_j_k)?;
            let escaped=s.escaped_gas.ok_or("Breach interval lacks cumulative escaped ledger")?;
            for (total,interval) in [(escaped.oxygen_kg,b.escaped.oxygen_kg),(escaped.nitrogen_kg,b.escaped.nitrogen_kg),(escaped.co2_kg,b.escaped.co2_kg),(escaped.vapor_kg,b.escaped.vapor_kg),(t.breach_enthalpy_j,b.outflow_enthalpy_j)] {if total+1e-8*total.max(1.0)<interval {return Err("Breach cumulative history below completed transfer".into());}}

            for (now,start,out) in [(s.oxygen_kg,b.initial_gas.oxygen_kg,b.escaped.oxygen_kg),(s.nitrogen_kg,b.initial_gas.nitrogen_kg,b.escaped.nitrogen_kg),(s.co2_kg,b.initial_gas.co2_kg,b.escaped.co2_kg),(s.water_vapor_kg,b.initial_gas.vapor_kg,b.escaped.vapor_kg)] {
                if (now-start+out).abs()>1e-8*start.max(1.0) {return Err("Breach final cabin stocks disagree with retained transfer".into());}
            }
            if s.temperature_k!=b.final_temperature_k {return Err("Breach final temperature disagrees with cabin".into());}
        }
        for (actual,expected) in [(s.water_vapor_kg,t.crew_vapor_kg+s.crop.as_ref().map_or(0.,|c|c.vapor_generated_kg)-t.condensed_kg-s.escaped_gas.map(|g|g.vapor_kg).unwrap_or(0.0)),(s.condensate_kg,t.condensed_kg-t.condensate_processed_kg)] {
            if (actual-expected).abs()>1e-8*actual.abs().max(expected.abs()).max(t.crew_vapor_kg).max(1.0) {return Err("Moisture stores disagree with cumulative transfer history".into());}
        }
        bounds(self.budget.condensate_processed_kg,0.0,self.budget.water_processed_kg,"Condensate processed")?;
        bounds(self.budget.condensate_recovered_kg,0.0,self.budget.condensate_processed_kg,"Condensate recovered")?;
        let reacted = t.sabatier_co2_consumed_kg;
        for (actual,expected,name) in [
            (s.methane_kg,reacted*crate::sabatier::METHANE_PER_CO2,"methane"),
            (s.sabatier_water_kg,reacted*crate::sabatier::WATER_PER_CO2-t.sabatier_water_processed_kg,"raw water"),
            (t.sabatier_heat_j,reacted*crate::sabatier::REFERENCE_HEAT_J_KG_CO2,"reaction heat"),
        ] {
            if (actual-expected).abs()>1e-8*actual.abs().max(expected.abs()).max(reacted).max(1.0) {return Err(format!("Sabatier {name} disagrees with cumulative reaction history"));}
        }
        bounds(self.budget.sabatier_water_processed_kg,0.0,self.budget.water_processed_kg,"Raw product water processed")?;
        bounds(self.budget.sabatier_water_recovered_kg,0.0,self.budget.sabatier_water_processed_kg,"Raw product water recovered")?;
        let expected_hydrogen = self.totals.reaction_enthalpy_j
            / crate::electrolysis::REFERENCE_ENTHALPY_J_KG_OXYGEN
            * crate::electrolysis::HYDROGEN_PER_OXYGEN - reacted*crate::sabatier::HYDROGEN_PER_CO2;
        if (s.hydrogen_kg - expected_hydrogen).abs() > 1e-8 * expected_hydrogen.abs().max(reacted).max(1.0) {
            return Err("Held hydrogen disagrees with cumulative reaction transfer".into());
        }
        bounds(self.budget.seconds, 0.0, PERIOD + 0.001, "Budget interval")?;
        bounds(
            self.budget.served_cycle_fraction,
            0.0,
            1.0,
            "Served reference-cycle fraction",
        )?;
        if let Some(c) = &self.budget.configuration {
            c.validate()?;
        }
        for value in [
            self.budget.generated_w,
            self.budget.requested_w,
            self.budget.delivered_w,
            self.budget.unserved_w,
            self.budget.battery_losses_w,
            self.budget.curtailed_w,
            self.budget.oxygen_makeup_kg,
            self.budget.co2_captured_kg,
            self.budget.water_processed_kg,
            self.budget.water_recovered_kg,
            self.budget.metabolic_heat_w,
            self.budget.heater_w,
            self.budget.radiator_rejection_w,
        ] {
            bounds(value, 0.0, 1.0e30, "Interval habitat budget")?;
        }
        for value in [
            self.budget.conduction_w,
            self.budget.thermal_storage_w,
            self.budget.mass_residual_kg,
            self.budget.electric_residual_j,
            self.budget.thermal_residual_j,
        ] {
            bounds(value, -1.0e30, 1.0e30, "Signed interval habitat budget")?;
        }
        if let Some(v) = self.budget.horizontal_irradiance_w_m2 {
            bounds(v, 0.0, 1.0e8, "Recorded irradiation")?;
        }
        if let Some(v) = self.budget.exterior_temperature_k {
            bounds(v, 1.0, 5000.0, "Recorded exterior temperature")?;
        }
        if self.budget.environment_available
            && (self.budget.horizontal_irradiance_w_m2.is_none()
                || self.budget.exterior_temperature_k.is_none())
        {
            return Err("Available habitat environment lacks inputs".into());
        }
        Ok(())
    }
    pub fn check_config(&self, c: &Config) -> Result<(), String> {
        c.validate()?;
        if let Some(crop)=&c.crop {self.state.crop.as_ref().ok_or("Install crops through the greenhouse installation edit; configuration does not grant initial nutrients")?.check_config(crop)?;if self.state.oxygen_reserve_kg>crop.oxygen_capacity_kg{return Err("New crop product capacity is below held oxygen; no gas discarded".into());}} else if self.state.crop.is_some() && self.definition.config.crop.is_none() {return Err("A runtime-installed greenhouse cannot be disabled; its operational configuration stays with the installation".into());}
        if c.battery_capacity_j < self.state.battery_j {
            return Err(
                "New battery capacity is below stored energy; no energy was discarded".into(),
            );
        }
        if c.capture_capacity_kg < self.state.captured_co2_kg {
            return Err(
                "New capture capacity is below already captured CO₂; no mass was discarded".into(),
            );
        }
        if let Some(e) = &c.electrolysis {
            if e.oxygen_capacity_kg < self.state.oxygen_reserve_kg || e.hydrogen_capacity_kg < self.state.hydrogen_kg {
                return Err("New electrolysis product capacity is below held O₂ or H₂; no mass was discarded".into());
            }
        }
        if let Some(e) = &c.sabatier {
            if e.methane_capacity_kg < self.state.methane_kg || e.water_capacity_kg < self.state.sabatier_water_kg {
                return Err("New Sabatier capacity is below held methane or raw water; no mass was discarded".into());
            }
        }
        if let Some(m)=&c.moisture {if m.condensate_capacity_kg<self.state.condensate_kg {return Err("New condensate capacity is below held liquid; no water discarded".into());}}
        for (held, capacity, n) in [
            (
                self.state.oxygen_reserve_kg,
                c.oxygen_reserve_capacity_kg,
                "O₂ reserve storage",
            ),
            (
                self.state.potable_water_kg,
                c.potable_water_capacity_kg,
                "Potable water storage",
            ),
            (
                self.state.food_dry_kg,
                c.food_dry_capacity_kg,
                "Dry food storage",
            ),
            (
                self.state.food_water_kg,
                c.food_water_capacity_kg,
                "Food water storage",
            ),
        ] {
            if capacity < held {
                return Err(format!(
                    "New {n} capacity is below held stock; no mass was discarded"
                ));
            }
        }
        Ok(())
    }
    /// Current held supply stocks in the transferable species mapping.
    pub fn held_cargo(&self) -> crate::supply_vehicle::Cargo {
        crate::supply_vehicle::Cargo {
            oxygen_reserve_kg: self.state.oxygen_reserve_kg,
            potable_water_kg: self.state.potable_water_kg,
            food_dry_kg: self.state.food_dry_kg,
            food_water_kg: self.state.food_water_kg,
        }
    }
    /// Finite general storage capacities for the same species mapping.
    pub fn storage_capacity_cargo(&self) -> crate::supply_vehicle::Cargo {
        crate::supply_vehicle::Cargo {
            oxygen_reserve_kg: self.config.oxygen_reserve_capacity_kg,
            potable_water_kg: self.config.potable_water_capacity_kg,
            food_dry_kg: self.config.food_dry_capacity_kg,
            food_water_kg: self.config.food_water_capacity_kg,
        }
    }
    /// Free space per species: capacity minus held stock, never negative.
    pub fn available_cargo(&self) -> crate::supply_vehicle::Cargo {
        let capacity = self.storage_capacity_cargo();
        let held = self.held_cargo();
        crate::supply_vehicle::Cargo {
            oxygen_reserve_kg: (capacity.oxygen_reserve_kg - held.oxygen_reserve_kg).max(0.0),
            potable_water_kg: (capacity.potable_water_kg - held.potable_water_kg).max(0.0),
            food_dry_kg: (capacity.food_dry_kg - held.food_dry_kg).max(0.0),
            food_water_kg: (capacity.food_water_kg - held.food_water_kg).max(0.0),
        }
    }
    pub fn uses_crop(&self)->bool {self.config.crop.is_some() || self.definition.config.crop.is_some() || self.state.crop.is_some() || self.budget.crop.is_some()}
    pub fn uses_breach(&self)->bool {self.config.breach.is_some() || self.definition.config.breach.is_some() || self.budget.breach.is_some() || self.state.escaped_gas.is_some() || self.totals.breach_enthalpy_j>0.0}
    pub fn uses_moisture(&self)->bool {self.config.moisture.is_some() || self.definition.config.moisture.is_some() || self.budget.moisture.is_some() || self.totals.crew_vapor_kg>0.0 || self.state.water_vapor_kg>0.0 || self.state.condensate_kg>0.0}
    pub fn uses_sabatier(&self) -> bool {
        self.config.sabatier.is_some() || self.definition.config.sabatier.is_some() || self.budget.sabatier.is_some()
            || self.state.methane_kg>0.0 || self.state.sabatier_water_kg>0.0 || self.totals.sabatier_co2_consumed_kg>0.0
    }
    pub fn uses_electrolysis(&self) -> bool {
        self.config.electrolysis.is_some() || self.definition.config.electrolysis.is_some()
            || self.budget.electrolysis.is_some() || self.state.hydrogen_kg > 0.0 || self.totals.reaction_enthalpy_j > 0.0
    }
    /// Only fixed scheduler intervals or explicit edit-time remainders enter here.
    pub fn step(&mut self, time: f64, env: Environment) -> Result<(), String> {
        if let Some(c)=&self.config.crop {self.state.crop.as_ref().ok_or("Crop state is not installed")?.check_config(c)?;}
        let dt = time - self.last_time;
        if !dt.is_finite() || !(0.0..=PERIOD + 0.001).contains(&dt) {
            return Err("Habitat interval must be forward and at most 60 seconds".into());
        }
        if dt == 0.0 {
            return Ok(());
        }
        let (Some(flux), Some(exterior)) = (env.irradiance_w_m2, env.exterior_temperature_k) else {
            self.budget = Budget {
                seconds: dt,
                temperature_domain_supported: (100.0..=500.0).contains(&self.state.temperature_k),
                ..Default::default()
            };
            if let Some(heat)=&mut self.heat {heat.exclude(dt,false);}
            self.last_time = time;
            return Ok(());
        };
        bounds(flux, 0.0, 1.0e8, "Local irradiance")?;
        bounds(exterior, 1.0, 5000.0, "Exterior model temperature")?;
        if !(100.0..=500.0).contains(&self.state.temperature_k) {
            self.budget = Budget {
                seconds: dt,
                environment_available: true,
                horizontal_irradiance_w_m2: Some(flux),
                exterior_temperature_k: Some(exterior),
                configuration: Some(self.config.clone()),
                ..Default::default()
            };
            if let Some(heat)=&mut self.heat {heat.exclude(dt,true);}
            self.last_time = time;
            return Ok(());
        }
        let c = &self.config;
        let s = &mut self.state;
        let old_mass = s.accounted_mass_kg();
        let old_battery = s.battery_j;
        let old_temperature = s.temperature_k;
        let crop_start=(s.potable_water_kg,s.captured_co2_kg,s.oxygen_reserve_kg);
        let crop_request=c.crop.as_ref().zip(s.crop.as_ref()).map_or(0.,|(c,state)|state.request_w(c,self.last_time,dt));
        let generated = c.firm_power_w
            + flux * c.solar_area_m2 * c.solar_efficiency * c.atmospheric_transmission;
        let heater_request =
            ((c.temperature_setpoint_k - s.temperature_k).max(0.0) * c.heat_capacity_j_k / dt)
                .min(c.heater_limit_w);
        let pump_request = if s.temperature_k > c.temperature_setpoint_k {
            c.radiator_pump_w
        } else {
            0.0
        };
        // Reserve nominal crew water before planning optional production. New recycle
        // water and freed product headroom wait one interval, so requests stay causal.
        let reactor_start_stocks = (s.potable_water_kg-c.crew as f64*3.217*dt/SECONDS_PER_DAY,s.oxygen_reserve_kg,s.hydrogen_kg);
        let planned_oxygen = c.electrolysis.as_ref().map(|e|e.planned_oxygen_kg(
            s.potable_water_kg-c.crew as f64*3.217*dt/SECONDS_PER_DAY,
            s.oxygen_reserve_kg,s.hydrogen_kg,dt)).unwrap_or(0.0);
        let reactor_request = c.electrolysis.as_ref().map(|e|
            (planned_oxygen*e.electricity_j_kg_oxygen/dt).min(e.power_limit_w)).unwrap_or(0.0);
        // Order is explicit: base services, CO2 capture, water recovery, heater, radiator,
        // then optional electrolysis and CO₂ reduction. Neither displaces existing life-support power.
        let sabatier_start = crate::sabatier::Stocks {co2_kg:s.captured_co2_kg,hydrogen_kg:s.hydrogen_kg,methane_kg:s.methane_kg,water_kg:s.sabatier_water_kg};
        let planned_co2 = c.sabatier.as_ref().map(|e|e.planned_co2_kg(sabatier_start,dt)).unwrap_or(0.0);
        let sabatier_request = c.sabatier.as_ref().map(|e|(planned_co2*e.electricity_j_kg_co2/dt).min(e.power_limit_w)).unwrap_or(0.0);
        // Condensation uses start vapor/headroom and comes after existing services/reactors.
        let condensate_start=s.condensate_kg;
        let moisture_plan=c.moisture.as_ref().map(|m|m.plan(s.water_vapor_kg,s.condensate_kg,s.temperature_k,c.volume_m3,dt));
        let moisture_request=moisture_plan.as_ref().map(|p|p.requested_w).unwrap_or(0.0);
        let requests = [
            c.base_load_w,
            c.scrubber_power_w,
            c.recycler_power_w,
            heater_request,
            pump_request,
            reactor_request,
            sabatier_request,
            moisture_request,
            crop_request,
        ];
        let requested: f64 = requests.iter().sum();
        let discharge = (requested - generated)
            .max(0.0)
            .min(c.discharge_limit_w)
            .min(s.battery_j * c.discharge_efficiency / dt);
        let charge = (generated - requested)
            .max(0.0)
            .min(c.charge_limit_w)
            .min((c.battery_capacity_j - s.battery_j).max(0.0) / (c.charge_efficiency * dt));
        s.battery_j = (s.battery_j + charge * c.charge_efficiency * dt
            - discharge / c.discharge_efficiency * dt)
            .max(0.0);
        let mut available = generated + discharge;
        let mut supplied = [0.0; 9];
        for (i, request) in requests.iter().enumerate() {
            supplied[i] = available.min(*request);
            available -= supplied[i];
        }
        let delivered: f64 = supplied.iter().sum();
        let losses =
            charge * (1.0 - c.charge_efficiency) + discharge * (1.0 / c.discharge_efficiency - 1.0);
        let curtailed = (generated - delivered - charge).max(0.0);
        let unserved = (requested - delivered).max(0.0);
        let fraction = |i: usize| {
            if requests[i] > 0.0 {
                supplied[i] / requests[i]
            } else {
                0.0
            }
        };
        // A mechanical regulator transfers stored oxygen; it creates no new oxygen.
        let oxygen_target = c.oxygen_setpoint_pa * c.volume_m3 * O2 / (R * s.temperature_k);
        let makeup = (oxygen_target - s.oxygen_kg)
            .max(0.0)
            .min(c.oxygen_makeup_rate_kg_s * dt)
            .min(s.oxygen_reserve_kg);
        s.oxygen_kg += makeup;
        s.oxygen_reserve_kg -= makeup;
        let demand = c.crew as f64 * dt / SECONDS_PER_DAY;
        let cycle = if demand > 0.0 {
            demand
                .min(s.oxygen_kg / 0.895)
                .min(s.potable_water_kg / 3.217)
                .min(s.food_dry_kg / 0.800)
                .min(s.food_water_kg / 0.760)
        } else {
            0.0
        };
        s.oxygen_kg = (s.oxygen_kg - cycle * 0.895).max(0.0);
        s.potable_water_kg = (s.potable_water_kg - cycle * 3.217).max(0.0);
        s.food_dry_kg = (s.food_dry_kg - cycle * 0.800).max(0.0);
        s.food_water_kg = (s.food_water_kg - cycle * 0.760).max(0.0);
        s.co2_kg += cycle * 1.085;
        let crew_vapor=c.moisture.as_ref().map(|m|cycle*2.946*m.airborne_fraction).unwrap_or(0.0);
        s.water_vapor_kg+=crew_vapor;
        s.wastewater_kg += cycle * 4.467-crew_vapor;
        s.dry_waste_kg += cycle * 0.120;
        let captured = (c.scrubber_rate_kg_s * dt * fraction(1))
            .min(s.co2_kg)
            .min((c.capture_capacity_kg - s.captured_co2_kg).max(0.0));
        s.co2_kg -= captured;
        s.captured_co2_kg += captured;
        let processed = (c.recycler_rate_kg_s * dt * fraction(2)).min(s.wastewater_kg);
        // Recovery enters the finite potable tank; overflow stays in the processed
        // stream as brine, never discarded. Default 1e7 kg tanks never bind.
        let recovered = (processed * c.water_recovery_fraction)
            .min((c.potable_water_capacity_kg - s.potable_water_kg).max(0.0));
        s.wastewater_kg -= processed;
        s.potable_water_kg = (s.potable_water_kg + recovered).min(c.potable_water_capacity_kg);
        s.brine_kg += processed - recovered;
        // Raw product needs the existing powered processing assumption, after wastewater.
        // Products generated below are unavailable here until a later interval.
        let product_processed = (c.recycler_rate_kg_s*dt*fraction(2)-processed).max(0.0).min(sabatier_start.water_kg);
        let product_recovered = (product_processed*c.water_recovery_fraction)
            .min((c.potable_water_capacity_kg - s.potable_water_kg).max(0.0));
        s.sabatier_water_kg = (s.sabatier_water_kg-product_processed).max(0.0);
        s.potable_water_kg = (s.potable_water_kg + product_recovered).min(c.potable_water_capacity_kg);
        s.brine_kg += product_processed-product_recovered;
        let condensate_processed=(c.recycler_rate_kg_s*dt*fraction(2)-processed-product_processed).max(0.0).min(condensate_start);
        let condensate_recovered=(condensate_processed*c.water_recovery_fraction)
            .min((c.potable_water_capacity_kg - s.potable_water_kg).max(0.0));
        s.condensate_kg=(s.condensate_kg-condensate_processed).max(0.0);
        s.potable_water_kg=(s.potable_water_kg+condensate_recovered).min(c.potable_water_capacity_kg);
        s.brine_kg+=condensate_processed-condensate_recovered;
        let moisture=c.moisture.as_ref().zip(moisture_plan).map(|(m,p)|crate::moisture::Budget::from_power(m,p,supplied[7],crew_vapor,dt));
        let crew_phase=crew_vapor*crate::moisture::REFERENCE_PHASE_J_KG;
        let condensation_heat=moisture.as_ref().map(|m|m.condensation_heat_j).unwrap_or(0.0);
        if let Some(m)=&moisture {s.water_vapor_kg=(s.water_vapor_kg-m.condensed_kg).max(0.0);s.condensate_kg+=m.condensed_kg;self.totals.condensed_kg+=m.condensed_kg;}
        self.totals.crew_vapor_kg+=crew_vapor;
        self.totals.condensate_processed_kg+=condensate_processed;
        let metabolic = cycle * 12_426_000.0 / dt;
        let rejection_capacity = c.radiator_emissivity
            * c.radiator_area_m2
            * STEFAN_BOLTZMANN
            * (s.temperature_k.powi(4) - exterior.powi(4)).max(0.0)
            * fraction(4);
        let reactor = c.electrolysis.as_ref().map(|e| {
            let mut b = crate::electrolysis::Budget::from_power(e,planned_oxygen,reactor_request,supplied[5],dt);
            b.constraints = e.constraints(reactor_start_stocks.0,reactor_start_stocks.1,reactor_start_stocks.2,dt,reactor_request,supplied[5]);
            b
        });
        let reaction_enthalpy = reactor.as_ref().map(|b|b.reaction_enthalpy_j).unwrap_or(0.0);
        if let Some(b) = &reactor {
            s.potable_water_kg = (s.potable_water_kg-b.water_consumed_kg).max(0.0);
            s.oxygen_reserve_kg += b.oxygen_generated_kg;
            s.hydrogen_kg += b.hydrogen_generated_kg;
        }
        let reduction = c.sabatier.as_ref().map(|e| {
            let mut b=crate::sabatier::Budget::from_power(e,planned_co2,sabatier_request,supplied[6],dt);
            b.constraints=e.constraints(sabatier_start,dt,sabatier_request,supplied[6]); b
        });
        let reaction_heat = reduction.as_ref().map(|b|b.reaction_heat_j).unwrap_or(0.0);
        if let Some(b)=&reduction {
            s.captured_co2_kg=(s.captured_co2_kg-b.co2_consumed_kg).max(0.0);
            s.hydrogen_kg=(s.hydrogen_kg-b.hydrogen_consumed_kg).max(0.0);
            s.methane_kg+=b.methane_generated_kg;
            s.sabatier_water_kg+=b.water_generated_kg;
            self.totals.sabatier_co2_consumed_kg+=b.co2_consumed_kg;
        }
        // Crop transfer uses only feeds/headroom present at interval start after prior
        // claims. Newly captured CO2/recycled water/freed reserve space wait one minute.
        let crop=c.crop.as_ref().zip(s.crop.as_mut()).map(|(crop,state)| {
            let capacity=crop.oxygen_capacity_kg.min(c.electrolysis.as_ref().map_or(crop.oxygen_capacity_kg,|e|e.oxygen_capacity_kg));
            let stocks=crate::crop_process::Stocks {water_kg:s.potable_water_kg.min((crop_start.0-c.crew as f64*3.217*dt/SECONDS_PER_DAY-reactor.as_ref().map_or(0.,|b|b.water_consumed_kg)).max(0.)),co2_kg:s.captured_co2_kg.min((crop_start.1-reduction.as_ref().map_or(0.,|b|b.co2_consumed_kg)).max(0.)),oxygen_headroom_kg:(capacity-s.oxygen_reserve_kg).max(0.).min((capacity-crop_start.2-reactor.as_ref().map_or(0.,|b|b.oxygen_generated_kg)).max(0.))};
            state.execute(crop,self.last_time,dt,supplied[8],stocks)
        }).transpose()?;
        let crop_debit=crop.as_ref().map_or(0.,|b|b.thermal_debit_j());
        if let Some(b)=&crop {s.potable_water_kg=(s.potable_water_kg-b.water_consumed_kg()).max(0.);s.captured_co2_kg=(s.captured_co2_kg-b.co2_consumed_kg).max(0.);s.oxygen_reserve_kg+=b.oxygen_generated_kg;s.water_vapor_kg+=b.vapor_generated_kg;}
        // Reaction enthalpy leaves the thermal balance; the remaining reactor electricity
        // is waste heat. Never count all bus electricity as heat AND as reaction energy.
        let gross = delivered + losses + metabolic - reaction_enthalpy/dt + reaction_heat/dt - crew_phase/dt + condensation_heat/dt - crop_debit/dt;
        let rejection = rejection_capacity.min(
            ((s.temperature_k - c.temperature_setpoint_k).max(0.0) * c.heat_capacity_j_k / dt
                + gross)
                .max(0.0),
        );
        let net = gross - rejection;
        // Exact lumped linear conduction for constant interval heat load. Unlike Euler,
        // this remains stable for all accepted conductance/capacity combinations.
        let x = c.conductance_w_k * dt / c.heat_capacity_j_k;
        // Stable (1-exp(-x))/x near zero; the cubic series' omitted term is <1e-22.
        let phi = if x < 1.0e-5 {
            1.0 - x / 2.0 + x * x / 6.0 - x * x * x / 24.0
        } else {
            (1.0 - (-x).dexp()) / x
        };
        s.temperature_k = old_temperature
            + (exterior - old_temperature) * x * phi
            + net * dt / c.heat_capacity_j_k * phi;
        // The existing thermal/process minute precedes the bounded finite breach substep.
        let breach=c.breach.as_ref().map(|e|crate::breach::integrate(e,s.cabin_gas(),s.temperature_k,c.volume_m3,c.heat_capacity_j_k,dt));
        let cooling=breach.as_ref().map(|b|b.lumped_cooling_j).unwrap_or(0.0);
        if let Some(b)=&breach {
            s.oxygen_kg=(s.oxygen_kg-b.escaped.oxygen_kg).max(0.0);
            s.nitrogen_kg=(s.nitrogen_kg-b.escaped.nitrogen_kg).max(0.0);
            s.co2_kg=(s.co2_kg-b.escaped.co2_kg).max(0.0);
            s.water_vapor_kg=(s.water_vapor_kg-b.escaped.vapor_kg).max(0.0);
            s.escaped_gas.get_or_insert_with(Default::default).add(b.escaped);
            s.temperature_k=b.final_temperature_k;
            self.totals.breach_enthalpy_j+=b.outflow_enthalpy_j;
        }
        let storage = c.heat_capacity_j_k * (s.temperature_k - old_temperature) / dt;
        let conduction = c.conductance_w_k * (exterior - old_temperature) * phi - net * (1.0 - phi);
        let electrical_residual =
            s.battery_j - old_battery - (generated - delivered - losses - curtailed) * dt;
        self.budget = Budget {
            seconds: dt,
            configuration: Some(c.clone()),
            environment_available: true,
            temperature_domain_supported: (100.0..=500.0).contains(&s.temperature_k),
            horizontal_irradiance_w_m2: Some(flux),
            exterior_temperature_k: Some(exterior),
            generated_w: generated,
            requested_w: requested,
            delivered_w: delivered,
            unserved_w: unserved,
            battery_losses_w: losses,
            curtailed_w: curtailed,
            oxygen_makeup_kg: makeup,
            co2_captured_kg: captured,
            water_processed_kg: processed+product_processed+condensate_processed,
            water_recovered_kg: recovered+product_recovered+condensate_recovered,
            served_cycle_fraction: if demand > 0.0 { cycle / demand } else { 1.0 },
            metabolic_heat_w: metabolic,
            heater_w: supplied[3],
            radiator_rejection_w: rejection,
            conduction_w: conduction,
            thermal_storage_w: storage,
            mass_residual_kg: s.accounted_mass_kg() - old_mass,
            electric_residual_j: electrical_residual,
            thermal_residual_j: (storage - net - conduction) * dt + cooling,
            crop,
            electrolysis: reactor,
            sabatier: reduction,
            breach,
            moisture,condensate_processed_kg:condensate_processed,condensate_recovered_kg:condensate_recovered,
            sabatier_water_processed_kg: product_processed,
            sabatier_water_recovered_kg: product_recovered,
        };
        if let Some(heat)=&mut self.heat {heat.record(&self.budget);}
        self.totals.reaction_enthalpy_j += reaction_enthalpy;
        self.totals.sabatier_heat_j += reaction_heat;
        self.totals.sabatier_water_processed_kg += product_processed;
        self.totals.generated_j += generated * dt;
        self.totals.delivered_j += delivered * dt;
        self.totals.losses_j += losses * dt;
        self.totals.curtailed_j += curtailed * dt;
        self.totals.unserved_j += unserved * dt;
        self.totals.metabolic_person_seconds += cycle * SECONDS_PER_DAY;
        self.totals.unserved_person_seconds += (demand - cycle) * SECONDS_PER_DAY;
        self.last_time = time;
        Ok(())
    }
    pub fn alerts(&self) -> u16 {
        let s = &self.state;
        let c = &self.config;
        let mut bits = 0;
        for (i, condition) in [
            !self.budget.environment_available,
            self.budget.unserved_w > 1.0,
            s.oxygen_pa(c) < c.oxygen_alert_pa,
            s.co2_pa(c) > c.co2_alert_pa,
            s.temperature_k < c.cold_alert_k || s.temperature_k > c.hot_alert_k,
            self.budget.served_cycle_fraction < 0.999999,
            !(100.0..=500.0).contains(&s.temperature_k),
        ]
        .into_iter()
        .enumerate()
        {
            if condition {
                bits |= 1 << i;
            }
        }
        bits
    }
    pub fn report(&self) -> serde_json::Value {
        let mut report=serde_json::json!({"format":"cosmogon-habitat-report","schema_version":1,"habitat":self,"diagnostics":crate::habitat_diagnostics::explain(self),"oxygen_partial_pressure_pa":self.state.oxygen_pa(&self.config),"co2_partial_pressure_pa":self.state.co2_pa(&self.config),"total_pressure_pa":self.state.pressure_pa(&self.config),"data_states":{"crew_rates":"Published nominal reference model (NASA BVAD), not individual measurements","equipment_and_initial_stocks":"User-assumed design inputs","current_stores_and_budgets":"Modelled","exterior_temperature":"Global reduced climate model; local measured weather unavailable","horizontal_irradiance":"Derived from primary-star state and approximate body frame"},"electrolysis_source":self.uses_electrolysis().then(crate::electrolysis::source),"sabatier_source":self.uses_sabatier().then(crate::sabatier::source),"moisture_source":self.uses_moisture().then(crate::moisture::source),"breach_source":self.uses_breach().then(crate::breach::source),"moisture_observation":self.uses_moisture().then(||serde_json::json!({"vapor_pressure_pa":self.state.vapor_pressure_pa(&self.config),"relative_humidity":self.state.relative_humidity(&self.config),"reference_phase_energy_j":self.state.phase_energy_j(),"supersaturated":self.state.relative_humidity(&self.config).map(|v|v>1.0)})),"uncertainty":null,"uncertainty_scope":"Input and predictive model uncertainty are not quantified","source":{"url":SOURCE,"version":"NASA BVAD Rev2 February 2022, Table 3-31","equipment":"User-assumed capacities, powers, efficiencies and alert thresholds"},"limits":"Reduced nominal demand and lumped thermal model, not validated mission hardware, local weather, purification quality, medical response or safety certification. Optional reduced humidity budget; no greenhouse exchange, wall/ice condensation, seal/inward/rarefied leakage, EVA, maintenance, radiation or resupply."});
        if self.heat.is_some() {report["heat_accounting"]=crate::habitat_heat::habitat_span(self);}
        if self.uses_crop() {report["crop_source"]=crate::crop_process::source();report["data_states"]["crop_operation"] = serde_json::json!("Finite modelled transfers under user-assumed ideal maintained source conditions; not calibrated biology/chamber hardware");report["limits"]=serde_json::json!("Reduced nominal demand and lumped thermal model with optional conserved maintained-reference crop transfers. No independently controlled chamber, calibrated crop physiology, complete diet credit, wall/ice condensation, seal/inward/rarefied leakage, EVA, maintenance, radiation or resupply. Crop-specific source/progress/energy assumptions are retained separately in crop_source.");}
        report
    }
}
pub const ALERTS: [&str; 7] = [
    "environment unavailable",
    "electric demand unserved",
    "oxygen below design threshold",
    "CO₂ above design threshold",
    "temperature outside design band",
    "nominal crew cycle unserved",
    "temperature outside supported model domain; budgets suspended",
];

impl Universe {
    pub fn habitat_environment(&self, d: &Definition, time: f64) -> Environment {
        let Some(sys) = self.systems.get(d.body.system as usize) else {
            return Environment::default();
        };
        let Some(body) = sys
            .bodies
            .get(d.body.body as usize)
            .filter(|b| b.exists() && b.kind.has_surface())
        else {
            return Environment::default();
        };
        let sun =
            sys.star_local_position(time) - sys.body_local_position(d.body.body as usize, time);
        let distance = sun.length();
        if distance <= body.radius || !distance.is_finite() {
            return Environment::default();
        }
        let local = sys.inertial_to_body_fixed(d.body.body as usize, sun / distance, time);
        let radial = terrain::dir_from_lat_lon(
            d.latitude_deg.to_radians(),
            d.longitude_east_deg.to_radians(),
        );
        let cosine = local
            .iter()
            .zip(radial)
            .map(|(a, b)| a * b)
            .sum::<f64>()
            .max(0.0);
        let flux = sys.star.luminosity(time) * SOLAR_LUMINOSITY
            / (4.0 * std::f64::consts::PI * distance * distance)
            * cosine;
        if !flux.is_finite()
            || flux > 1.0e8
            || !body.temperature.is_finite()
            || !(1.0..=5000.0).contains(&body.temperature)
        {
            return Environment::default();
        }
        Environment {
            irradiance_w_m2: Some(flux),
            exterior_temperature_k: Some(body.temperature),
        }
    }
    pub(crate) fn ensure_habitat_task(&mut self) {
        if self.habitats.is_empty() {
            if self.scheduler.tasks.get(TASK).is_some_and(|t| t.name == "habitats") {
                if self.scheduler.tasks.len() == TASK + 1 {
                    self.scheduler.tasks.remove(TASK);
                } else {
                    // Later task indices must never shift when a habitat is removed.
                    self.scheduler.tasks[TASK] = crate::scheduler::Task {
                        name: "habitats".into(), origin: self.time,
                        period: crate::scheduler::NEVER, steps: 0,
                    };
                }
            }
            return;
        }
        if self.scheduler.tasks.len() == TASK {
            self.scheduler.add("habitats", self.time, PERIOD);
        } else if self.scheduler.tasks.get(TASK).is_some_and(|t| t.name == "habitats" && t.period == crate::scheduler::NEVER) {
            self.scheduler.tasks[TASK] = crate::scheduler::Task {
                name: "habitats".into(), origin: self.time, period: PERIOD, steps: 0,
            };
        }
    }
    pub(crate) fn step_habitats(&mut self, time: f64) {
        for i in 0..self.habitats.len() {
            let env = self.habitat_environment(&self.habitats[i].definition, time);
            // Invalid external domains become explicitly unavailable, never fake local data.
            if self.habitats[i].step(time, env).is_err() {
                self.habitats[i].budget = Budget::default();
                self.habitats[i].last_time = time;
            }
            self.habitat_events(i, time);
        }
    }
    pub(crate) fn habitat_events(&mut self, i: usize, time: f64) {
        let h = &mut self.habitats[i];
        let now = h.alerts();
        let changed = now ^ h.alert_bits;
        for (bit, message) in ALERTS.iter().enumerate() {
            if changed & (1 << bit) != 0 {
                let active = now & (1 << bit) != 0;
                self.history.push(Event {time,category:if active {Category::Disaster} else {Category::Space},importance:3,title:format!("{}: {}{message}",h.definition.name,if active {""} else {"recovered from "}),detail:format!("Experimental design alert. O₂ {:.1} kPa; CO₂ {:.1} Pa; {:.2} K; unserved {:.1} W. Not medical or safety certification.",h.state.oxygen_pa(&h.config)/1000.0,h.state.co2_pa(&h.config),h.state.temperature_k,h.budget.unserved_w),system:Some(h.definition.body.system),body:Some(h.definition.body.body),civ:None});
            }
        }
        h.alert_bits = now;
    }
}

/// Observer rows retain both the requested world time and the completed habitat interval.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub simulation_time_s: f64,
    pub habitat: Habitat,
}
pub fn sample(u: &Universe, id: u64) -> Result<Sample, String> {
    Ok(Sample {
        simulation_time_s: u.time,
        habitat: u
            .habitats
            .iter()
            .find(|h| h.id == id)
            .ok_or("Habitat no longer exists")?
            .clone(),
    })
}
/// Shared actual observation formatting for export and recorded consistency checks.
pub(crate) fn samples_csv(rows: &[Sample]) -> String {
    let mut csv=String::from("simulation_time_s,completed_habitat_time_s,interval_s,temperature_k,total_pressure_pa,oxygen_partial_pressure_pa,co2_partial_pressure_pa,battery_j,oxygen_kg,oxygen_reserve_kg,potable_water_kg,wastewater_kg,brine_kg,food_dry_kg,food_water_kg,dry_waste_kg,captured_co2_kg,environment_available,temperature_domain_supported,horizontal_irradiance_w_m2,exterior_temperature_k,generated_w,requested_w,delivered_w,unserved_w,battery_losses_w,curtailed_w,served_cycle_fraction,metabolic_heat_w,heater_w,radiator_rejection_w,conduction_w,thermal_storage_w,mass_residual_kg,electric_residual_j,thermal_residual_j,total_unserved_j,total_unserved_person_seconds,hydrogen_kg,electrolysis_requested_w,electrolysis_delivered_w,electrolysis_oxygen_generated_kg,electrolysis_hydrogen_generated_kg,electrolysis_water_consumed_kg,electrolysis_electricity_j,electrolysis_reaction_enthalpy_j,electrolysis_waste_heat_j,total_reaction_enthalpy_j,electrolysis_constraints,methane_kg,sabatier_water_kg,sabatier_requested_w,sabatier_delivered_w,sabatier_co2_consumed_kg,sabatier_hydrogen_consumed_kg,sabatier_methane_generated_kg,sabatier_water_generated_kg,sabatier_electricity_j,sabatier_reaction_heat_j,sabatier_total_heat_j,sabatier_water_processed_kg,sabatier_water_recovered_kg,total_sabatier_co2_consumed_kg,total_sabatier_heat_j,total_sabatier_water_processed_kg,sabatier_constraints,water_vapor_kg,condensate_kg,vapor_pressure_pa,relative_humidity,saturation_pressure_pa,reference_phase_energy_j,moisture_requested_w,moisture_delivered_w,crew_vapor_kg,crew_phase_energy_j,condensed_kg,condensation_heat_j,moisture_electricity_j,moisture_constraints,condensate_processed_kg,condensate_recovered_kg,total_crew_vapor_kg,total_condensed_kg,total_condensate_processed_kg,escaped_oxygen_kg,escaped_nitrogen_kg,escaped_co2_kg,escaped_vapor_kg,accounted_mass_kg,breach_initial_pressure_pa,breach_final_pressure_pa,breach_initial_temperature_k,breach_final_temperature_k,breach_initial_rate_kg_s,breach_final_rate_kg_s,breach_integrated_seconds,breach_steps,breach_oxygen_kg,breach_nitrogen_kg,breach_co2_kg,breach_vapor_kg,breach_lumped_cooling_j,breach_structure_energy_change_j,breach_gas_energy_change_j,breach_phase_energy_change_j,breach_outflow_enthalpy_j,breach_energy_residual_j,breach_status,total_breach_enthalpy_j\n");
    for row in rows {
        let h = &row.habitat;
        let s = &h.state;
        let b = &h.budget;
        let c = &h.config;
        let values = [
            row.simulation_time_s,
            h.last_time,
            b.seconds,
            s.temperature_k,
            s.pressure_pa(c),
            s.oxygen_pa(c),
            s.co2_pa(c),
            s.battery_j,
            s.oxygen_kg,
            s.oxygen_reserve_kg,
            s.potable_water_kg,
            s.wastewater_kg,
            s.brine_kg,
            s.food_dry_kg,
            s.food_water_kg,
            s.dry_waste_kg,
            s.captured_co2_kg,
        ];
        csv.push_str(
            &values
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push_str(&format!(
            ",{},{},{},{}",
            b.environment_available,
            b.temperature_domain_supported,
            b.horizontal_irradiance_w_m2
                .map(|v| v.to_string())
                .unwrap_or_default(),
            b.exterior_temperature_k
                .map(|v| v.to_string())
                .unwrap_or_default()
        ));
        for value in [
            b.generated_w,
            b.requested_w,
            b.delivered_w,
            b.unserved_w,
            b.battery_losses_w,
            b.curtailed_w,
            b.served_cycle_fraction,
            b.metabolic_heat_w,
            b.heater_w,
            b.radiator_rejection_w,
            b.conduction_w,
            b.thermal_storage_w,
            b.mass_residual_kg,
            b.electric_residual_j,
            b.thermal_residual_j,
            h.totals.unserved_j,
            h.totals.unserved_person_seconds,
        ] {
            csv.push(',');
            csv.push_str(&value.to_string());
        }
        csv.push(',');csv.push_str(&s.hydrogen_kg.to_string());
        let reaction_values = b.electrolysis.as_ref().map(|e|[
            e.requested_w,e.delivered_w,e.oxygen_generated_kg,e.hydrogen_generated_kg,
            e.water_consumed_kg,e.electricity_j,e.reaction_enthalpy_j,e.waste_heat_j]);
        for i in 0..8 {csv.push(',');if let Some(v)=reaction_values {csv.push_str(&v[i].to_string());}}
        csv.push(',');csv.push_str(&h.totals.reaction_enthalpy_j.to_string());
        csv.push(',');if let Some(e)=&b.electrolysis {csv.push_str(&e.constraints.iter().map(|c|c.label()).collect::<Vec<_>>().join("; "));}
        for v in [s.methane_kg,s.sabatier_water_kg] {csv.push(',');csv.push_str(&v.to_string());}
        let reduction_values=b.sabatier.as_ref().map(|e|[e.requested_w,e.delivered_w,e.co2_consumed_kg,e.hydrogen_consumed_kg,e.methane_generated_kg,e.water_generated_kg,e.electricity_j,e.reaction_heat_j,e.total_heat_j]);
        for i in 0..9 {csv.push(',');if let Some(v)=reduction_values {csv.push_str(&v[i].to_string());}}
        for v in [b.sabatier_water_processed_kg,b.sabatier_water_recovered_kg,h.totals.sabatier_co2_consumed_kg,h.totals.sabatier_heat_j,h.totals.sabatier_water_processed_kg] {csv.push(',');csv.push_str(&v.to_string());}
        csv.push(',');if let Some(e)=&b.sabatier {csv.push_str(&e.constraints.iter().map(|c|c.label()).collect::<Vec<_>>().join("; "));}
        for v in [Some(s.water_vapor_kg),Some(s.condensate_kg),h.uses_moisture().then(||s.vapor_pressure_pa(c)),h.uses_moisture().then_some(()).and_then(|_|s.relative_humidity(c)),h.uses_moisture().then_some(()).and_then(|_|crate::moisture::saturation_pa(s.temperature_k)),Some(s.phase_energy_j())] {csv.push(',');if let Some(v)=v {csv.push_str(&v.to_string());}}
        let mv=b.moisture.as_ref().map(|m|[m.requested_w,m.delivered_w,m.crew_vapor_kg,m.crew_phase_energy_j,m.condensed_kg,m.condensation_heat_j,m.electricity_j]);
        for i in 0..7 {csv.push(',');if let Some(v)=mv {csv.push_str(&v[i].to_string());}}
        csv.push(',');if let Some(m)=&b.moisture {csv.push_str(&m.constraints.join("; "));}
        for v in [b.condensate_processed_kg,b.condensate_recovered_kg,h.totals.crew_vapor_kg,h.totals.condensed_kg,h.totals.condensate_processed_kg] {csv.push(',');csv.push_str(&v.to_string());}
        let escaped=s.escaped_gas.unwrap_or_default();
        for v in [escaped.oxygen_kg,escaped.nitrogen_kg,escaped.co2_kg,escaped.vapor_kg,s.accounted_mass_kg()] {csv.push(',');csv.push_str(&v.to_string());}
        let bv=b.breach.as_ref().map(|e|[e.initial_pressure_pa,e.final_pressure_pa,e.initial_temperature_k,e.final_temperature_k,e.initial_rate_kg_s,e.final_rate_kg_s,e.integrated_seconds,e.integration_steps as f64,e.escaped.oxygen_kg,e.escaped.nitrogen_kg,e.escaped.co2_kg,e.escaped.vapor_kg,e.lumped_cooling_j,e.structure_energy_change_j,e.gas_energy_change_j,e.phase_energy_change_j,e.outflow_enthalpy_j,e.energy_residual_j]);
        for i in 0..18 {csv.push(',');if let Some(v)=bv {csv.push_str(&v[i].to_string());}}
        csv.push(',');if let Some(e)=&b.breach {csv.push_str(e.status.label());}
        csv.push(',');csv.push_str(&h.totals.breach_enthalpy_j.to_string());
        csv.push('\n');
    }
    csv
}
/// The caller supplies actual completed states; no advancement or fabricated samples.
/// Exclusive directory/file creation protects existing studies and saves.
pub fn export(
    initial: &Universe,
    final_world: &Universe,
    id: u64,
    rows: &[Sample],
    path: &std::path::Path,
) -> Result<(), String> {
    use std::io::Write;
    if rows.is_empty() || rows.len() > 1024 {
        return Err("Export requires 1–1024 observer samples".into());
    }
    let h = final_world
        .habitats
        .iter()
        .find(|h| h.id == id)
        .ok_or("No such habitat")?;
    if rows
        .iter()
        .any(|r| r.habitat.id != id || !r.simulation_time_s.is_finite())
    {
        return Err("Observer samples must identify this habitat".into());
    }
    let csv=samples_csv(rows);
    let mut manifest=serde_json::json!({"format":"cosmogon-habitat-study","schema_version":1,"model":MODEL,"habitat_id":id,"sample_count":rows.len(),"initial_state_fingerprint":crate::research::state_fingerprint(initial)?,"final_state_fingerprint":crate::research::state_fingerprint(final_world)?,"fingerprint_scope":"FNV-1a state identity, not cryptographic integrity","app_version":env!("CARGO_PKG_VERSION"),"simulation_time_coordinate":"seconds since J2000, same coordinate as world","habitat_intervals":"fixed 60 s plus explicit edit-time remainders; samples may follow the last completed interval","source":SOURCE,"model_descriptor":crate::research::PhysicsModelRegistry::get("habitat"),"electrolysis_model_descriptor":h.uses_electrolysis().then(||crate::research::PhysicsModelRegistry::get("electrolysis")),"electrolysis_source":h.report()["electrolysis_source"],"sabatier_model_descriptor":h.uses_sabatier().then(||crate::research::PhysicsModelRegistry::get("sabatier")),"sabatier_source":h.report()["sabatier_source"],"breach_source":h.uses_breach().then(crate::breach::source),"breach_model_descriptor":h.uses_breach().then(||crate::research::PhysicsModelRegistry::get("breach")),"moisture_source":h.uses_moisture().then(crate::moisture::source),"moisture_model_descriptor":h.uses_moisture().then(||crate::research::PhysicsModelRegistry::get("moisture")),"limits":h.report()["limits"]});
    if h.heat.is_some() {manifest["heat_accounting_source"]=crate::habitat_heat::source();manifest["heat_accounting_model_descriptor"]=serde_json::to_value(crate::research::PhysicsModelRegistry::get("habitat-heat")).map_err(|e|e.to_string())?;}
    if h.uses_crop() {manifest["crop_source"]=crate::crop_process::source();manifest["crop_model_descriptor"]=serde_json::to_value(crate::research::PhysicsModelRegistry::get("finite-crop")).map_err(|e|e.to_string())?;}
    let mut data=vec![("definition.json",serde_json::to_vec_pretty(&h.definition).map_err(|e|e.to_string())?),("report.json",serde_json::to_vec_pretty(&h.report()).map_err(|e|e.to_string())?),("samples.json",serde_json::to_vec_pretty(rows).map_err(|e|e.to_string())?),("samples.csv",csv.into_bytes()),("initial.cosmo",crate::save::to_json(initial,"Habitat study initial world").map_err(|e|e.to_string())?.into_bytes()),("final.cosmo",crate::save::to_json(final_world,"Habitat study final world").map_err(|e|e.to_string())?.into_bytes()),("manifest.json",serde_json::to_vec_pretty(&manifest).map_err(|e|e.to_string())?)];
    if h.uses_crop() {data.insert(data.len()-1,("crop.csv",crate::crop_process::samples_csv(rows).into_bytes()));}
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(path)
        .map_err(|e| format!("Existing exports are protected; choose a new folder: {e}"))?;
    for (name, data) in data {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| format!("Incomplete habitat export: {e}"))?;
        file.write_all(&data)
            .map_err(|e| format!("Incomplete habitat export: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn habitat() -> Habitat {
        Habitat::new(
            0,
            Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0),
            0.0,
        )
        .unwrap()
    }
    fn environment() -> Environment {
        Environment {
            irradiance_w_m2: Some(0.0),
            exterior_temperature_k: Some(210.0),
        }
    }
    #[test]
    fn balanced_crew_and_finite_processors_conserve_mass_and_electric_energy() {
        let mut h = habitat();
        let mass = h.state.mass_kg();
        let energy = h.state.battery_j;
        for k in 1..=1440 {
            h.step(k as f64 * PERIOD, environment()).unwrap();
            assert!(h.budget.mass_residual_kg.abs() < 1.0e-9);
            assert!(h.budget.electric_residual_j.abs() < 1.0e-6);
        }
        assert!((h.state.mass_kg() - mass).abs() < 1.0e-8);
        assert!((h.totals.metabolic_person_seconds - 4.0 * SECONDS_PER_DAY).abs() < 1.0e-6);
        assert!((h.state.food_dry_kg - 96.8).abs() < 1.0e-9);
        assert!((h.state.food_water_kg - 91.96).abs() < 1.0e-9);
        assert!(h.state.brine_kg > 0.0);
        assert!(
            (h.state.captured_co2_kg
                - h.definition.initial.co2_pa * h.config.volume_m3 * CO2
                    / (R * h.definition.initial.temperature_k)
                - 4.34)
                .abs()
                < 1.0e-8
        );
        assert!(
            (h.state.battery_j - energy - h.totals.generated_j
                + h.totals.delivered_j
                + h.totals.losses_j
                + h.totals.curtailed_j)
                .abs()
                < 0.01
        );
    }
    #[test]
    fn shortages_never_create_mass_power_or_negative_stores() {
        let mut h = habitat();
        h.config.firm_power_w = 0.0;
        h.config.solar_area_m2 = 0.0;
        h.state.battery_j = 0.0;
        h.state.potable_water_kg = 0.0;
        let mass = h.state.mass_kg();
        h.step(PERIOD, environment()).unwrap();
        assert_eq!(h.budget.delivered_w, 0.0);
        assert_eq!(h.budget.co2_captured_kg, 0.0);
        assert_eq!(h.budget.served_cycle_fraction, 0.0);
        assert_eq!(h.totals.unserved_person_seconds, 240.0);
        assert_eq!(h.state.mass_kg(), mass);
        assert!(h.alerts() & (1 << 1) != 0 && h.alerts() & (1 << 5) != 0);
        let prior = h.clone();
        assert!(h.step(1000.0, environment()).is_err());
        assert_eq!(h, prior);
        h.step(120.0, Environment::default()).unwrap();
        assert!(!h.budget.environment_available);
        assert!(h.budget.horizontal_irradiance_w_m2.is_none());
        assert_eq!(h.state, prior.state);
        assert!(h.budget.temperature_domain_supported);
        h.state.temperature_k = 600.0;
        let state = h.state.clone();
        h.step(180.0, environment()).unwrap();
        assert_eq!(h.state, state);
        assert!(!h.budget.temperature_domain_supported);
        assert!(h.alerts() & (1 << 6) != 0);
    }
    #[test]
    fn validation_refuses_future_model_nonfinite_and_destructive_capacity_changes() {
        let h = habitat();
        let mut d = h.definition.clone();
        d.model = "future/2".into();
        assert!(d.validate().is_err());
        d = h.definition.clone();
        d.config.solar_efficiency = f64::NAN;
        assert!(d.validate().is_err());
        let c = Config {
            battery_capacity_j: 0.0,
            ..h.config.clone()
        };
        assert!(h.check_config(&c).is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{sandbox::Edit, save, Scenario, UniverseSettings};
    fn assert_same_json(a: &Universe, b: &Universe) {
        fn differences(
            a: &serde_json::Value,
            b: &serde_json::Value,
            path: &str,
            out: &mut Vec<String>,
        ) {
            if out.len() >= 10 || a == b {
                return;
            }
            match (a, b) {
                (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                    for (k, v) in a {
                        if let Some(w) = b.get(k) {
                            differences(v, w, &format!("{path}/{k}"), out)
                        } else {
                            out.push(format!("missing {path}/{k}"));
                        }
                    }
                }
                (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                    if a.len() != b.len() {
                        out.push(format!("length {path}: {} vs {}", a.len(), b.len()));
                    }
                    for (i, (v, w)) in a.iter().zip(b).enumerate() {
                        differences(v, w, &format!("{path}/{i}"), out);
                    }
                }
                _ => out.push(format!("{path}: {a} vs {b}")),
            }
        }
        let mut out = Vec::new();
        differences(
            &serde_json::to_value(a).unwrap(),
            &serde_json::to_value(b).unwrap(),
            "",
            &mut out,
        );
        assert!(out.is_empty(), "{}", out.join("\n"));
    }
    fn world() -> (Universe, BodyRef) {
        let u = Universe::new(UniverseSettings {
            scenario: Scenario::SolarSystemLab,
            ..Default::default()
        });
        let r = BodyRef {
            system: 0,
            body: u.systems[0].find_body("Mars").unwrap() as u32,
        };
        (u, r)
    }
    #[test]
    fn scheduler_chunking_save_resume_and_edits_preserve_world_physics() {
        let (mut a, r) = world();
        let systems = a.systems.clone();
        let epoch = a.time;
        a.apply_edit(Edit::CreateHabitat {
            definition: Box::new(Definition::at(r, -4.59, 137.44)),
        })
        .unwrap();
        assert_eq!(a.systems, systems);
        let mut b = a.clone();
        a.advance_to(epoch + 7200.0, None, None);
        for i in 1..=240 {
            b.advance_to(epoch + i as f64 * 30.0, None, None);
        }
        assert_eq!(a, b);
        let mut resumed = save::from_json(&save::to_json(&a, "habitat").unwrap()).unwrap();
        assert_eq!(resumed.habitats, a.habitats);
        a.advance_by(1800.0);
        resumed.advance_by(1800.0);
        assert_same_json(&a, &resumed);
        let mut broken: serde_json::Value =
            serde_json::from_str(&save::to_json(&a, "habitat").unwrap()).unwrap();
        broken["header"]["data_versions"]["cosmogon-habitat-budgets"] =
            serde_json::json!("future/2");
        assert!(matches!(
            save::from_json(&broken.to_string()),
            Err(save::SaveError::Model(_))
        ));
        let unchanged = a.clone();
        let config = Config {
            battery_capacity_j: 0.0,
            ..a.habitats[0].config.clone()
        };
        assert!(a
            .apply_edit(Edit::ConfigureHabitat {
                system: 0,
                id: 0,
                config: Box::new(config)
            })
            .is_err());
        assert_eq!(a, unchanged);
        a.advance_by(17.0);
        let old_time = a.habitats[0].last_time;
        let old_cycle = a.habitats[0].totals.metabolic_person_seconds;
        let config = Config {
            crew: 0,
            ..a.habitats[0].config.clone()
        };
        a.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(config),
        })
        .unwrap();
        assert_eq!(a.habitats[0].last_time, a.time);
        assert!(
            (a.habitats[0].totals.metabolic_person_seconds - old_cycle - 4.0 * (a.time - old_time))
                .abs()
                < 1.0e-5
        );
        assert_eq!(a.habitats[0].budget.configuration.as_ref().unwrap().crew, 4);
        assert_eq!(a.habitats[0].config.crew, 0);
        assert!(save::from_json(&save::to_json(&a, "configured").unwrap()).is_ok());
        a.apply_edit(Edit::RemoveHabitat { system: 0, id: 0 })
            .unwrap();
        assert!(a.habitats.is_empty());
        assert_eq!(a.scheduler.tasks.len(), 4);
    }
    #[test]
    fn alert_transitions_are_meaningful_and_removed_body_suspends_budgets() {
        let (mut u, r) = world();
        let mut d = Definition::at(r, 0.0, 0.0);
        d.config.firm_power_w = 0.0;
        d.config.solar_area_m2 = 0.0;
        d.initial.battery_j = 0.0;
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(d),
        })
        .unwrap();
        let start = u.time;
        u.advance_to(start + 60.0, None, None);
        let count = u
            .history
            .events
            .iter()
            .filter(|e| e.title.contains("electric demand unserved"))
            .count();
        assert_eq!(count, 1);
        u.advance_by(600.0);
        assert_eq!(
            u.history
                .events
                .iter()
                .filter(|e| e.title.contains("electric demand unserved"))
                .count(),
            count
        );
        let c = Config {
            firm_power_w: 10_000.0,
            ..u.habitats[0].config.clone()
        };
        u.apply_edit(Edit::ConfigureHabitat {
            system: 0,
            id: 0,
            config: Box::new(c),
        })
        .unwrap();
        u.advance_by(60.0);
        assert_eq!(
            u.history
                .events
                .iter()
                .filter(|e| e.title.contains("recovered from electric demand unserved"))
                .count(),
            1
        );
        u.apply_edit(Edit::RemoveBody { body: r }).unwrap();
        let state = u.habitats[0].state.clone();
        u.advance_by(120.0);
        assert_eq!(u.habitats[0].state, state);
        assert!(!u.habitats[0].budget.environment_available);
        assert!(u.habitats[0].budget.horizontal_irradiance_w_m2.is_none());
        assert!(save::from_json(&save::to_json(&u, "destroyed host").unwrap()).is_ok());
    }
    #[test]
    fn thermal_storage_matches_analytic_constant_power_and_passive_cooling() {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        d.config = Config {
            crew: 0,
            firm_power_w: 1000.0,
            solar_area_m2: 0.0,
            base_load_w: 100.0,
            scrubber_power_w: 0.0,
            recycler_power_w: 0.0,
            heater_limit_w: 0.0,
            radiator_pump_w: 0.0,
            charge_limit_w: 0.0,
            conductance_w_k: 0.0,
            ..Default::default()
        };
        let mut h = Habitat::new(0, d, 0.0).unwrap();
        let initial = h.state.temperature_k;
        let env = Environment {
            irradiance_w_m2: Some(0.0),
            exterior_temperature_k: Some(210.0),
        };
        for k in 1..=60 {
            h.step(k as f64 * 60.0, env).unwrap();
        }
        assert!(
            (h.state.temperature_k - initial - 100.0 * 3600.0 / h.config.heat_capacity_j_k).abs()
                < 1.0e-10
        );
        assert!(h.budget.thermal_residual_j.abs() < 1.0e-4);
        h.config.base_load_w = 0.0;
        h.config.conductance_w_k = 100.0;
        let initial = h.state.temperature_k;
        for k in 61..=120 {
            h.step(k as f64 * 60.0, env).unwrap();
        }
        let exact =
            210.0 + (initial - 210.0) * (-100.0 * 3600.0 / h.config.heat_capacity_j_k).dexp();
        assert!((h.state.temperature_k - exact).abs() < 1.0e-10);
        assert!(h.budget.thermal_residual_j.abs() < 1.0e-4);
    }
    #[test]
    fn numeric_exports_and_save_boundaries_keep_actual_context_and_protect_existing_files() {
        let (mut u, r) = world();
        u.apply_edit(Edit::CreateHabitat {
            definition: Box::new(Definition::at(r, 0.0, 0.0)),
        })
        .unwrap();
        let initial = u.clone();
        let first = sample(&u, 0).unwrap();
        u.advance_by(120.0);
        let last = sample(&u, 0).unwrap();
        let root =
            std::env::temp_dir().join(format!("cosmogon-habitat-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        export(&initial, &u, 0, &[first, last], &root).unwrap();
        let csv = std::fs::read_to_string(root.join("samples.csv")).unwrap();
        assert_eq!(csv.lines().count(), 3);
        assert_eq!(
            csv.lines().next().unwrap().split(',').count(),
            csv.lines().last().unwrap().split(',').count()
        );
        let final_world = save::read_file(&root.join("final.cosmo")).unwrap();
        assert_same_json(&u, &final_world);
        assert!(export(&initial, &u, 0, &[sample(&u, 0).unwrap()], &root).is_err());
        assert_eq!(
            csv,
            std::fs::read_to_string(root.join("samples.csv")).unwrap()
        );
        let mut v: serde_json::Value =
            serde_json::from_str(&save::to_json(&u, "boundary").unwrap()).unwrap();
        v["universe"]["habitats"][0]["state"]["potable_water_kg"] = serde_json::json!(-1.0);
        assert!(matches!(
            save::from_json(&v.to_string()),
            Err(save::SaveError::Model(_))
        ));
        let mut v: serde_json::Value =
            serde_json::from_str(&save::to_json(&u, "boundary").unwrap()).unwrap();
        let duplicate = v["universe"]["habitats"][0].clone();
        v["universe"]["habitats"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(save::from_json(&v.to_string()).is_err());
        std::fs::write(root.join("bad.json"), vec![b'x'; 65_537]).unwrap();
        assert!(Definition::read(&root.join("bad.json")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storage_capacities_are_skipped_when_default_and_authored_when_present() {
        let json = serde_json::to_string(&Config::default()).unwrap();
        assert!(!json.contains("oxygen_reserve_capacity_kg"));
        assert!(!json.contains("potable_water_capacity_kg"));
        assert!(!json.contains("food_dry_capacity_kg"));
        assert!(!json.contains("food_water_capacity_kg"));
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Config::default());
        assert_eq!(parsed.oxygen_reserve_capacity_kg, 1.0e7);
        let authored = Config {
            potable_water_capacity_kg: 250.,
            ..Config::default()
        };
        let json = serde_json::to_string(&authored).unwrap();
        assert!(json.contains("potable_water_capacity_kg"));
        assert!(!json.contains("oxygen_reserve_capacity_kg"));
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.potable_water_capacity_kg, 250.);
    }

    #[test]
    fn storage_capacities_bound_initial_state_and_reconfiguration() {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        d.config.potable_water_capacity_kg = 100.;
        let error = d.validate().unwrap_err();
        assert!(error.contains("exceeds its storage capacity"), "{error}");
        d.initial.potable_water_kg = 100.;
        d.validate().unwrap();
        let mut h = Habitat::new(0, d, 0.).unwrap();
        h.validate(0.).unwrap();
        h.state.potable_water_kg = 101.;
        let error = h.validate(0.).unwrap_err();
        assert!(error.contains("below held stock"), "{error}");
        h.state.potable_water_kg = 100.;
        let mut shrink = h.config.clone();
        shrink.potable_water_capacity_kg = 99.;
        let error = h.check_config(&shrink).unwrap_err();
        assert!(error.contains("no mass was discarded"), "{error}");
    }

    #[test]
    fn available_cargo_reports_capacity_minus_held_stock() {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        d.config.oxygen_reserve_capacity_kg = 120.;
        d.config.food_dry_capacity_kg = 50.;
        d.initial.food_dry_kg = 60.;
        assert!(d.validate().is_err());
        d.initial.food_dry_kg = 45.;
        let h = Habitat::new(0, d, 0.).unwrap();
        assert_eq!(h.held_cargo().oxygen_reserve_kg, 100.);
        let free = h.available_cargo();
        assert_eq!(free.oxygen_reserve_kg, 20.);
        assert_eq!(free.food_dry_kg, 5.);
        assert_eq!(free.potable_water_kg, 1.0e7 - 200.);
        assert_eq!(free.food_water_kg, 1.0e7 - 95.);
    }

    #[test]
    fn oxygen_storage_must_cover_installed_equipment_capacity() {
        let mut d = Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0);
        d.config.electrolysis = Some(crate::electrolysis::Config::default());
        d.config.oxygen_reserve_capacity_kg = 150.;
        let error = d.validate().unwrap_err();
        assert!(error.contains("below installed electrolysis"), "{error}");
        d.config.oxygen_reserve_capacity_kg = 200.;
        d.validate().unwrap();
    }
}
