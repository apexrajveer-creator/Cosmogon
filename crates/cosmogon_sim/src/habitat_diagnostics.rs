//! Read-only explanations of a completed habitat interval. No reconstructed start stocks,
//! physical forecasts, new state or unique historical causal attribution.
use crate::{
    habitat::{Habitat, SOURCE},
    time::SECONDS_PER_DAY,
    Universe,
};
use serde::Serialize;
use std::{io::Write, path::Path};

pub const MODEL: &str = "cosmogon-habitat-diagnostics/1";

/// The same evidence gate is used by research and equipment sonification.
pub fn support(h: &Habitat) -> &'static str {
    let b = &h.budget;
    if b.seconds <= 0.0 {
        "waiting_for_completed_interval"
    } else if !b.environment_available {
        "environment_unavailable"
    } else if !b.temperature_domain_supported {
        "temperature_domain_unsupported"
    } else if b.configuration.is_none() {
        "interval_configuration_unavailable"
    } else {
        "supported"
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Allocation {
    pub id: &'static str,
    pub label: &'static str,
    pub requested_w: Option<f64>,
    pub served_w: Option<f64>,
    pub rated_w: Option<f64>,
    pub evidence: &'static str,
}
fn remainder(v: f64) -> Option<f64> {
    // Priority summation followed by subtraction can leave a small negative roundoff.
    // Larger inconsistent differences are unavailable, never silently clamped away.
    (v >= -1e-6 && v.is_finite()).then_some(v.max(0.0))
}
pub fn allocations(h: &Habitat) -> Vec<Allocation> {
    let mut rows = vec![
        Allocation { id:"base", label:"Base services", requested_w:None, served_w:None, rated_w:None, evidence:"budget.configuration.base_load_w; priority 1 of delivered_w" },
        Allocation { id:"scrubber", label:"CO₂ scrubber", requested_w:None, served_w:None, rated_w:None, evidence:"budget.configuration.scrubber_power_w; priority 2 of delivered_w" },
        Allocation { id:"recycler", label:"Water recycler", requested_w:None, served_w:None, rated_w:None, evidence:"budget.configuration.recycler_power_w; priority 3 of delivered_w" },
        Allocation { id:"thermal", label:"Heater + radiator pump", requested_w:None, served_w:None, rated_w:None, evidence:"aggregate power minus services and recorded reactors; separate thermal requests not retained" },
        Allocation { id:"electrolysis", label:"Water electrolysis", requested_w:None, served_w:None, rated_w:None, evidence:"budget.electrolysis; interval configuration power limit" },
        Allocation { id:"sabatier", label:"CO₂ reduction", requested_w:None, served_w:None, rated_w:None, evidence:"budget.sabatier; interval configuration power limit" },
    ];
    if h.uses_moisture() {rows.push(Allocation {id:"moisture",label:"Dehumidifier",requested_w:None,served_w:None,rated_w:None,evidence:"budget.moisture; interval configuration power limit"});}
    if support(h) != "supported" {
        return rows;
    }
    let b = &h.budget;
    let c = b.configuration.as_ref().expect("supported configuration");
    let mut available = b.delivered_w;
    for (row, request) in
        rows[..3]
            .iter_mut()
            .zip([c.base_load_w, c.scrubber_power_w, c.recycler_power_w])
    {
        let served = available.min(request).max(0.0);
        available = (available - served).max(0.0);
        row.requested_w = Some(request);
        row.rated_w = Some(request);
        row.served_w = Some(served);
    }
    for (i, configured, rating, requested, served) in [
        (
            4,
            c.electrolysis.is_some(),
            c.electrolysis.as_ref().map(|e| e.power_limit_w),
            b.electrolysis.as_ref().map(|e| e.requested_w),
            b.electrolysis.as_ref().map(|e| e.delivered_w),
        ),
        (
            5,
            c.sabatier.is_some(),
            c.sabatier.as_ref().map(|e| e.power_limit_w),
            b.sabatier.as_ref().map(|e| e.requested_w),
            b.sabatier.as_ref().map(|e| e.delivered_w),
        ),
    ] {
        rows[i].rated_w = Some(rating.unwrap_or(0.0));
        rows[i].requested_w = requested.or_else(|| (!configured).then_some(0.0));
        rows[i].served_w = served.or_else(|| (!configured).then_some(0.0));
    }
    let moisture_request=b.moisture.as_ref().map(|m|m.requested_w).or_else(||c.moisture.is_none().then_some(0.0));
    let moisture_served=b.moisture.as_ref().map(|m|m.delivered_w).or_else(||c.moisture.is_none().then_some(0.0));
    if let Some(row)=rows.get_mut(6) {row.rated_w=Some(c.moisture.as_ref().map(|m|m.power_limit_w).unwrap_or(0.0));row.requested_w=moisture_request;row.served_w=moisture_served;}
    rows[3].rated_w = Some(c.heater_limit_w + c.radiator_pump_w);
    rows[3].requested_w = rows[4]
        .requested_w
        .zip(rows[5].requested_w)
        .and_then(|(e, s)| {
            remainder(
                b.requested_w - c.base_load_w - c.scrubber_power_w - c.recycler_power_w - e - s - moisture_request?,
            )
        });
    rows[3].served_w = rows[4]
        .served_w
        .zip(rows[5].served_w)
        .and_then(|(e, s)| remainder(available - e - s - moisture_served?));
    rows
}
#[derive(Clone, Debug, Serialize)]
pub struct Term {
    pub id: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub value: Option<f64>,
    pub evidence: &'static str,
}
fn term(
    id: &'static str,
    label: &'static str,
    unit: &'static str,
    value: Option<f64>,
    evidence: &'static str,
) -> Term {
    Term {
        id,
        label,
        unit,
        value,
        evidence,
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Explanation {
    pub id: &'static str,
    pub text: String,
    pub evidence: Vec<&'static str>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostics {
    pub model: &'static str,
    pub habitat_id: u64,
    pub interval_start_s: Option<f64>,
    pub interval_end_s: f64,
    pub interval_s: f64,
    pub support: &'static str,
    pub configuration_matches_current: bool,
    pub allocations: Vec<Allocation>,
    pub power: Vec<Term>,
    pub heat: Vec<Term>,
    pub water: Vec<Term>,
    pub air_and_crew: Vec<Term>,
    pub explanations: Vec<Explanation>,
    pub uncertainty: Option<f64>,
    pub limits: &'static str,
}
pub fn explain(h: &Habitat) -> Diagnostics {
    let b = &h.budget;
    let supported = support(h);
    let mut d = Diagnostics {
        model:MODEL, habitat_id:h.id,
        interval_start_s:(b.seconds>0.0).then_some(h.last_time-b.seconds),
        interval_end_s:h.last_time, interval_s:b.seconds, support:supported,
        configuration_matches_current:b.configuration.as_ref()==Some(&h.config),
        allocations:allocations(h), power:Vec::new(),heat:Vec::new(),water:Vec::new(),air_and_crew:Vec::new(),explanations:Vec::new(), uncertainty:None,
        limits:"Completed-interval model accounting, not a unique historical cause graph or equipment fault diagnosis. Inputs and equipment are assumptions; uncertainty is unquantified. Numerical residuals test bookkeeping, not hardware calibration or physiological safety. Start inventories, separate thermal requests and measured local weather are not retained here. Heater electricity is already in bus heat. Newly produced reactor water/oxygen cannot be credited to earlier same-interval consumers.",
    };
    if supported != "supported" {
        d.explanations.push(Explanation {id:"unavailable",text:format!("Interval decomposition unavailable: {}. Suspended or missing evidence is not zero physical demand.",supported.replace('_'," ")),evidence:vec!["budget.seconds","budget.environment_available","budget.temperature_domain_supported","budget.configuration"]});
        return d;
    }
    let c = b.configuration.as_ref().expect("supported configuration");
    let electrolysis = b.electrolysis.as_ref();
    let sabatier = b.sabatier.as_ref();
    let e_heat = electrolysis
        .map(|e| e.reaction_enthalpy_j / b.seconds)
        .or_else(|| c.electrolysis.is_none().then_some(0.0));
    let s_heat = sabatier
        .map(|e| e.reaction_heat_j / b.seconds)
        .or_else(|| c.sabatier.is_none().then_some(0.0));
    let e_water = electrolysis
        .map(|e| e.water_consumed_kg)
        .or_else(|| c.electrolysis.is_none().then_some(0.0));
    let s_water = sabatier
        .map(|e| e.water_generated_kg)
        .or_else(|| c.sabatier.is_none().then_some(0.0));
    let moisture=b.moisture.as_ref();
    let phase=moisture.map(|m|(m.crew_phase_energy_j,m.condensation_heat_j)).or_else(||c.moisture.is_none().then_some((0.0,0.0)));
    let vapor=moisture.map(|m|m.crew_vapor_kg).or_else(||c.moisture.is_none().then_some(0.0));
    let days = c.crew as f64 * b.seconds / SECONDS_PER_DAY * b.served_cycle_fraction;
    let recovered = b.water_recovered_kg;
    let potable_net = e_water.map(|w| recovered - 3.217 * days - w);
    let heat_sum = e_heat.zip(s_heat).zip(phase).map(|((e, s),(debit,credit))| {
        b.delivered_w + b.battery_losses_w + b.metabolic_heat_w - e + s - b.radiator_rejection_w
            + b.conduction_w - debit/b.seconds + credit/b.seconds
    });
    d.power = vec![
        term(
            "firm",
            "Firm supply",
            "W",
            Some(c.firm_power_w),
            "budget.configuration.firm_power_w",
        ),
        term(
            "solar",
            "PV supply",
            "W",
            remainder(b.generated_w - c.firm_power_w),
            "budget.generated_w - interval firm_power_w; sampled primary-star flux",
        ),
        term(
            "discharge",
            "Battery → bus",
            "W",
            Some((b.delivered_w - b.generated_w).max(0.0)),
            "max(delivered_w - generated_w, 0)",
        ),
        term(
            "charge",
            "Bus → battery before losses",
            "W",
            Some((b.generated_w - b.delivered_w - b.curtailed_w).max(0.0)),
            "max(generated_w - delivered_w - curtailed_w, 0)",
        ),
        term(
            "losses",
            "Battery conversion losses",
            "W",
            Some(b.battery_losses_w),
            "budget.battery_losses_w",
        ),
        term(
            "curtailed",
            "Unused supply",
            "W",
            Some(b.curtailed_w),
            "budget.curtailed_w",
        ),
        term(
            "requested",
            "Total requested",
            "W",
            Some(b.requested_w),
            "budget.requested_w",
        ),
        term(
            "delivered",
            "Total served",
            "W",
            Some(b.delivered_w),
            "budget.delivered_w",
        ),
        term(
            "unserved",
            "Unserved demand",
            "W",
            Some(b.unserved_w),
            "budget.unserved_w",
        ),
        term(
            "stored",
            "Battery stored-energy change",
            "W",
            Some(
                b.generated_w - b.delivered_w - b.battery_losses_w - b.curtailed_w
                    + b.electric_residual_j / b.seconds,
            ),
            "generated - delivered - battery losses - curtailed + electric_residual_j/seconds",
        ),
        term(
            "electric_residual",
            "Electrical bookkeeping residual",
            "J",
            Some(b.electric_residual_j),
            "budget.electric_residual_j",
        ),
    ];
    d.heat = vec![
        term("bus","Served bus electricity","W",Some(b.delivered_w),"budget.delivered_w; includes heater and pump electricity"),
        term("losses","Battery heat","W",Some(b.battery_losses_w),"budget.battery_losses_w"),
        term("crew","Reference metabolic heat","W",Some(b.metabolic_heat_w),"budget.metabolic_heat_w"),
        term("electrolysis","Electrolysis enthalpy debit","W",e_heat.map(|w|-w),"-budget.electrolysis.reaction_enthalpy_j / seconds; absent configured budget is unknown"),
        term("sabatier","CO₂ reduction reference heat","W",s_heat,"budget.sabatier.reaction_heat_j / seconds"),
        term("radiator","Radiator heat debit","W",Some(-b.radiator_rejection_w),"-budget.radiator_rejection_w"),
        term("conduction","Envelope heat exchange","W",Some(b.conduction_w),"budget.conduction_w; positive into cabin"),
        term("sum","Sum of signed heat inputs","W",heat_sum,"bus + losses + metabolic - electrolysis enthalpy + Sabatier heat - radiator + conduction"),
        term("stored","Recorded heat storage","W",Some(b.thermal_storage_w),"budget.thermal_storage_w"),
        term("thermal_residual","Thermal bookkeeping residual","J",Some(b.thermal_residual_j),"budget.thermal_residual_j; (storage - signed sum) * seconds"),
    ];
    d.water = vec![
        term(
            "person_days",
            "Served reference person-days",
            "person-day",
            Some(days),
            "interval crew * seconds/86400 * served_cycle_fraction; zero crew produces zero",
        ),
        term(
            "crew_potable",
            "Reference potable consumption",
            "kg",
            Some(3.217 * days),
            "NASA BVAD Table 3-31: 3.217 kg/person-day * actual served reference days",
        ),
        term(
            "crew_food_water",
            "Food water consumption",
            "kg",
            Some(0.760 * days),
            "NASA BVAD Table 3-31: 0.760 kg/person-day * actual served reference days",
        ),
        term(
            "crew_wastewater",
            "Reference wastewater produced",
            "kg",
            vapor.map(|v|4.467*days-v),
            if h.uses_moisture(){"4.467 kg/person-day * actual served days - recorded airborne vapor"}else{"NASA BVAD Table 3-31: 4.467 kg/person-day * actual served reference days"},
        ),
        term(
            "electrolysis",
            "Electrolysis water draw",
            "kg",
            e_water,
            "budget.electrolysis.water_consumed_kg",
        ),
        term(
            "waste_processed",
            "Wastewater processed",
            "kg",
            remainder(b.water_processed_kg - b.sabatier_water_processed_kg - b.condensate_processed_kg),
            if h.uses_moisture(){"water_processed_kg - sabatier_water_processed_kg - condensate_processed_kg"}else{"water_processed_kg - sabatier_water_processed_kg"},
        ),
        term(
            "raw_processed",
            "Held raw product processed",
            "kg",
            Some(b.sabatier_water_processed_kg),
            "budget.sabatier_water_processed_kg; held at interval start, after wastewater",
        ),
        term(
            "waste_recovered",
            "Wastewater potable recovery",
            "kg",
            remainder(recovered - b.sabatier_water_recovered_kg - b.condensate_recovered_kg),
            if h.uses_moisture(){"water_recovered_kg - sabatier_water_recovered_kg - condensate_recovered_kg"}else{"water_recovered_kg - sabatier_water_recovered_kg"},
        ),
        term(
            "raw_recovered",
            "Held raw product potable recovery",
            "kg",
            Some(b.sabatier_water_recovered_kg),
            "budget.sabatier_water_recovered_kg",
        ),
        term(
            "brine",
            "Unrecovered water → brine",
            "kg",
            remainder(b.water_processed_kg - recovered),
            "water_processed_kg - water_recovered_kg",
        ),
        term(
            "new_raw",
            "New CO₂ reduction raw product",
            "kg",
            s_water,
            "budget.sabatier.water_generated_kg; unavailable to earlier same-interval recycling",
        ),
        term(
            "potable_net",
            "Potable store net change",
            "kg",
            potable_net,
            "total recovery - nominal crew potable use - electrolysis draw",
        ),
        term(
            "mass_residual",
            "Whole habitat mass residual",
            "kg",
            Some(b.mass_residual_kg),
            "budget.mass_residual_kg; includes all closed stores, not only water",
        ),
    ];
    let oxygen_generated = electrolysis
        .map(|e| e.oxygen_generated_kg)
        .or_else(|| c.electrolysis.is_none().then_some(0.0));
    let hydrogen_generated = electrolysis
        .map(|e| e.hydrogen_generated_kg)
        .or_else(|| c.electrolysis.is_none().then_some(0.0));
    let co2_consumed = sabatier
        .map(|e| e.co2_consumed_kg)
        .or_else(|| c.sabatier.is_none().then_some(0.0));
    let hydrogen_consumed = sabatier
        .map(|e| e.hydrogen_consumed_kg)
        .or_else(|| c.sabatier.is_none().then_some(0.0));
    let methane_generated = sabatier
        .map(|e| e.methane_generated_kg)
        .or_else(|| c.sabatier.is_none().then_some(0.0));
    d.air_and_crew = vec![
        term(
            "oxygen_makeup",
            "Reserve → cabin O₂",
            "kg",
            Some(b.oxygen_makeup_kg),
            "budget.oxygen_makeup_kg; mechanical regulator transfer, not production",
        ),
        term(
            "crew_oxygen",
            "Reference crew O₂ use",
            "kg",
            Some(0.895 * days),
            "NASA BVAD Table 3-31: 0.895 kg/person-day * served reference days",
        ),
        term(
            "cabin_oxygen_net",
            "Cabin O₂ net change",
            "kg",
            Some(b.oxygen_makeup_kg - 0.895 * days),
            "regulator makeup - reference crew oxygen consumption",
        ),
        term(
            "crew_co2",
            "Reference crew CO₂ produced",
            "kg",
            Some(1.085 * days),
            "NASA BVAD Table 3-31: 1.085 kg/person-day * served reference days",
        ),
        term(
            "co2_captured",
            "Cabin → captured CO₂",
            "kg",
            Some(b.co2_captured_kg),
            "budget.co2_captured_kg; scrubber transfers mass to a finite store",
        ),
        term(
            "cabin_co2_net",
            "Cabin CO₂ net change",
            "kg",
            Some(1.085 * days - b.co2_captured_kg),
            "reference crew CO2 production - actual capture",
        ),
        term(
            "stored_oxygen_net",
            "O₂ reserve net change",
            "kg",
            oxygen_generated.map(|m| m - b.oxygen_makeup_kg),
            "electrolysis O2 generation - regulator makeup",
        ),
        term(
            "captured_co2_net",
            "Captured CO₂ net change",
            "kg",
            co2_consumed.map(|m| b.co2_captured_kg - m),
            "actual capture - recorded Sabatier CO2 consumption",
        ),
        term(
            "hydrogen_net",
            "Isolated H₂ net change",
            "kg",
            hydrogen_generated
                .zip(hydrogen_consumed)
                .map(|(g, c)| g - c),
            "electrolysis H2 generation - Sabatier H2 consumption",
        ),
        term(
            "methane_net",
            "Isolated methane net change",
            "kg",
            methane_generated,
            "recorded Sabatier methane generation; no vent or combustion",
        ),
        term(
            "dry_food_net",
            "Dry food net change",
            "kg",
            Some(-0.800 * days),
            "NASA BVAD Table 3-31: -0.800 kg/person-day * served reference days",
        ),
        term(
            "food_water_net",
            "Food water net change",
            "kg",
            Some(-0.760 * days),
            "NASA BVAD Table 3-31: -0.760 kg/person-day * served reference days",
        ),
        term(
            "dry_waste_net",
            "Dry waste net change",
            "kg",
            Some(0.120 * days),
            "NASA BVAD Table 3-31: 0.120 kg/person-day * served reference days",
        ),
    ];
    let configuration_matches_current = d.configuration_matches_current;
    if h.uses_moisture() {
        d.heat.insert(3,term("crew_vapor_phase","Crew energy retained in vapor","W",phase.map(|p|-p.0/b.seconds),"-budget.moisture.crew_phase_energy_j / seconds"));
        d.heat.insert(4,term("condensation","Condensation reference heat released","W",phase.map(|p|p.1/b.seconds),"budget.moisture.condensation_heat_j / seconds; electricity already in bus"));
        if let Some(sum)=d.heat.iter_mut().find(|t|t.id=="sum") {sum.evidence="bus + losses + metabolic - electrolysis enthalpy + Sabatier heat - crew phase + condensation - radiator + conduction";}
        d.water.push(term("crew_vapor","Crew vapor transferred to cabin","kg",vapor,"budget.moisture.crew_vapor_kg; partition of 4.467 kg/day total water output"));
        d.water.push(term("condensate","Vapor collected as raw liquid","kg",moisture.map(|m|m.condensed_kg).or_else(||c.moisture.is_none().then_some(0.0)),"budget.moisture.condensed_kg; available for later recycling"));
        d.water.push(term("condensate_processed","Held condensate processed","kg",Some(b.condensate_processed_kg),"budget.condensate_processed_kg; after wastewater and raw reactor product"));
        d.water.push(term("condensate_recovered","Assumed condensate water recovery","kg",Some(b.condensate_recovered_kg),"budget.condensate_recovered_kg; not purification quality"));
    }
    let mut note = |id, text: String, evidence: Vec<&'static str>| {
        d.explanations.push(Explanation { id, text, evidence })
    };
    if b.unserved_w > 1e-6 {
        note("power_shortfall",format!("{:.3} W of demand went unserved. Base services, air and water processing receive power before thermal controls and optional reactors.",b.unserved_w),vec!["budget.requested_w","budget.delivered_w","allocations"]);
    } else {
        note("power_served","All recorded electrical requests were served. Powered equipment can be idle when material inputs or storage constrain its processing.".into(),vec!["budget.unserved_w","budget.electrolysis.constraints","budget.sabatier.constraints"]);
    }
    note("potable",match potable_net {Some(w)=>format!("The potable store changed by {w:+.6} kg: recovered water minus reference crew use and electrolysis draw."),None=>"Potable store change cannot be decomposed: configured electrolysis has no retained interval budget.".into()},vec!["water.potable_net","budget.configuration.crew","budget.served_cycle_fraction"]);
    if !configuration_matches_current {
        note("configuration_changed","Current controls differ from this interval. This explanation retains the completed configuration; changed settings have not rewritten the result.".into(),vec!["budget.configuration","config"]);
    }
    if b.horizontal_irradiance_w_m2 == Some(0.0) {
        note("zero_flux","The sampled horizontal primary-star irradiance was zero. These inputs do not establish local weather, dust or a specific night-time cause.".into(),vec!["budget.horizontal_irradiance_w_m2"]);
    }
    if c.crew > 0 && b.served_cycle_fraction < 1.0 {
        note("crew_shortfall",format!("{:.3}% of the nominal crew cycle was supplied. The limiting start inventory is not retained, so no unique oxygen/water/food cause or health outcome is inferred.",100.0*b.served_cycle_fraction),vec!["budget.served_cycle_fraction","budget.configuration.crew"]);
    }
    for (id, labels, evidence) in [
        (
            "electrolysis_constraints",
            electrolysis.map(|e| e.constraints.iter().map(|c| c.label()).collect::<Vec<_>>()),
            "budget.electrolysis.constraints",
        ),
        (
            "sabatier_constraints",
            sabatier.map(|e| e.constraints.iter().map(|c| c.label()).collect::<Vec<_>>()),
            "budget.sabatier.constraints",
        ),
    ] {
        if let Some(labels) = labels {
            if !labels.is_empty() {
                note(id,format!("Recorded bounds: {}. These may overlap and do not identify a unique equipment fault.",labels.join("; ")),vec![evidence]);
            }
        }
    }
    if let Some(m)=moisture {if !m.constraints.is_empty() {note("moisture_constraints",format!("Recorded moisture bounds: {}. Ratings and vapor partition are assumptions.",m.constraints.join("; ")),vec!["budget.moisture.constraints"]);}}
    if c.moisture.is_some() && moisture.is_none() {note("missing_moisture","Configured moisture equipment lacks its interval budget; phase heat and dependent sums remain unavailable.".into(),vec!["budget.moisture","budget.configuration"]);}
    if c.electrolysis.is_some() && electrolysis.is_none()
        || c.sabatier.is_some() && sabatier.is_none()
    {
        note("missing_reactor","A configured reactor has no retained interval budget. Its watts, flow and heat remain unavailable, as do dependent aggregate decompositions.".into(),vec!["budget.electrolysis","budget.sabatier","budget.configuration"]);
    }
    d
}

pub fn report(h: &Habitat, world_time_s: f64) -> Result<serde_json::Value, String> {
    h.validate(world_time_s)?;
    Ok(
        serde_json::json!({"format":"cosmogon-habitat-diagnostic-report","schema_version":1,"world_time_s":world_time_s,"time_coordinate":"seconds since J2000; interval end can precede world time","diagnostics":explain(h),"habitat":h,"source":{"crew":SOURCE,"electrolysis":h.uses_electrolysis().then(crate::electrolysis::source),"sabatier":h.uses_sabatier().then(crate::sabatier::source),"moisture":h.uses_moisture().then(crate::moisture::source)}}),
    )
}
pub fn from_world(u: &Universe, id: u64) -> Result<serde_json::Value, String> {
    report(
        u.habitats
            .iter()
            .find(|h| h.id == id)
            .ok_or("No such habitat in this world")?,
        u.time,
    )
}
/// Numeric decomposition is a derived snapshot. Full replay remains the ordinary saved
/// world/study workflow; a complete manifest is written last and old folders are protected.
pub fn export(h: &Habitat, world_time_s: f64, path: &Path) -> Result<(), String> {
    let source = report(h, world_time_s)?;
    let d = explain(h);
    let mut csv = String::from("group,id,label,value,unit,evidence\n");
    fn cell(s: &str) -> String {
        format!("\"{}\"", s.replace('"', "\"\""))
    }
    for (group, terms) in [
        ("power", &d.power),
        ("heat", &d.heat),
        ("water", &d.water),
        ("air_and_crew", &d.air_and_crew),
    ] {
        for t in terms {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                group,
                t.id,
                cell(t.label),
                t.value.map(|v| v.to_string()).unwrap_or_default(),
                cell(t.unit),
                cell(t.evidence)
            ));
        }
    }
    for a in &d.allocations {
        for (kind, value) in [
            ("requested_w", a.requested_w),
            ("served_w", a.served_w),
            ("rated_w", a.rated_w),
        ] {
            csv.push_str(&format!(
                "allocation,{}_{},{},{},W,{}\n",
                a.id,
                kind,
                cell(a.label),
                value.map(|v| v.to_string()).unwrap_or_default(),
                cell(a.evidence)
            ));
        }
    }
    let data=[("report.json",serde_json::to_vec_pretty(&source).map_err(|e|e.to_string())?),("terms.csv",csv.into_bytes()),("manifest.json",serde_json::to_vec_pretty(&serde_json::json!({"format":"cosmogon-habitat-diagnostics","schema_version":1,"complete":true,"model":MODEL,"habitat_id":h.id,"world_time_s":world_time_s,"interval_end_s":h.last_time,"files":["report.json","terms.csv"],"scope":"Derived completed-interval snapshot with exact habitat inputs; not a trajectory or independent field validation"})).map_err(|e|e.to_string())?)];
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir(path)
        .map_err(|e| format!("Existing exports are protected; choose a new folder: {e}"))?;
    for (name, bytes) in data {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))
            .map_err(|e| format!("Incomplete diagnostic export: {e}"))?;
        f.write_all(&bytes)
            .map_err(|e| format!("Incomplete diagnostic export: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        habitat::{Definition, Environment},
        BodyRef,
    };
    fn habitat() -> Habitat {
        Habitat::new(
            0,
            Definition::at(BodyRef { system: 0, body: 0 }, 0.0, 0.0),
            0.0,
        )
        .unwrap()
    }
    fn env() -> Environment {
        Environment {
            irradiance_w_m2: Some(0.0),
            exterior_temperature_k: Some(210.0),
        }
    }
    fn value(terms: &[Term], id: &str) -> Option<f64> {
        terms.iter().find(|t| t.id == id).unwrap().value
    }
    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }
    #[test]
    fn actual_interval_power_heat_and_water_close_without_double_counting_heater() {
        let mut h = habitat();
        h.state.temperature_k = 280.0;
        let before = h.state.clone();
        h.step(60.0, env()).unwrap();
        let d = explain(&h);
        assert_eq!(d.support, "supported");
        near(
            d.allocations.iter().map(|a| a.requested_w.unwrap()).sum(),
            h.budget.requested_w,
        );
        near(
            d.allocations.iter().map(|a| a.served_w.unwrap()).sum(),
            h.budget.delivered_w,
        );
        near(
            value(&d.power, "stored").unwrap() * 60.0,
            h.state.battery_j - before.battery_j,
        );
        near(
            value(&d.heat, "sum").unwrap() + h.budget.thermal_residual_j / 60.0,
            h.budget.thermal_storage_w,
        );
        near(
            value(&d.water, "potable_net").unwrap(),
            h.state.potable_water_kg - before.potable_water_kg,
        );
        near(
            value(&d.water, "brine").unwrap(),
            h.state.brine_kg - before.brine_kg,
        );
        near(
            value(&d.air_and_crew, "cabin_oxygen_net").unwrap(),
            h.state.oxygen_kg - before.oxygen_kg,
        );
        near(
            value(&d.air_and_crew, "cabin_co2_net").unwrap(),
            h.state.co2_kg - before.co2_kg,
        );
        near(
            value(&d.air_and_crew, "dry_food_net").unwrap(),
            h.state.food_dry_kg - before.food_dry_kg,
        );
        assert!(h.budget.heater_w > 0.0);
        let current = h.clone();
        let _ = report(&h, 61.0).unwrap();
        assert_eq!(h, current);
        h.config.base_load_w = 9000.0;
        let after = explain(&h);
        assert!(!after.configuration_matches_current);
        assert_eq!(after.allocations[0].requested_w, Some(500.0));
    }
    #[test]
    fn actual_battery_discharge_and_priority_starvation_remain_distinct() {
        let mut h = habitat();
        h.config.firm_power_w = 0.0;
        h.step(60.0, env()).unwrap();
        let d = explain(&h);
        near(value(&d.power, "discharge").unwrap(), h.budget.delivered_w);
        assert_eq!(value(&d.power, "charge"), Some(0.0));
        let mut h = habitat();
        h.config.firm_power_w = 650.0;
        h.state.battery_j = 0.0;
        h.step(60.0, env()).unwrap();
        let d = explain(&h);
        assert_eq!(d.allocations[0].served_w, Some(500.0));
        assert_eq!(d.allocations[1].served_w, Some(150.0));
        assert_eq!(d.allocations[2].served_w, Some(0.0));
        assert!(d.explanations.iter().any(|e| e.id == "power_shortfall"));
    }
    #[test]
    fn reactor_recovery_balances_and_recorded_constraints_are_not_recomputed_from_end_stocks() {
        let mut h = habitat();
        h.config.crew = 0;
        h.config.firm_power_w = 5000.0;
        h.config.electrolysis = Some(Default::default());
        h.config.sabatier = Some(Default::default());
        h.state.captured_co2_kg = 1.0;
        h.state.hydrogen_kg = 0.5;
        h.state.sabatier_water_kg = 1.0;
        let before = h.state.clone();
        h.step(60.0, env()).unwrap();
        let d = explain(&h);
        assert_eq!(value(&d.water, "person_days"), Some(0.0));
        assert_eq!(value(&d.water, "crew_potable"), Some(0.0));
        near(
            value(&d.water, "potable_net").unwrap(),
            h.state.potable_water_kg - before.potable_water_kg,
        );
        near(
            value(&d.heat, "sum").unwrap() + h.budget.thermal_residual_j / 60.0,
            h.budget.thermal_storage_w,
        );
        near(
            value(&d.water, "new_raw").unwrap() - value(&d.water, "raw_processed").unwrap(),
            h.state.sabatier_water_kg - before.sabatier_water_kg,
        );
        near(
            value(&d.air_and_crew, "hydrogen_net").unwrap(),
            h.state.hydrogen_kg - before.hydrogen_kg,
        );
        near(
            value(&d.air_and_crew, "captured_co2_net").unwrap(),
            h.state.captured_co2_kg - before.captured_co2_kg,
        );
        near(
            value(&d.air_and_crew, "stored_oxygen_net").unwrap(),
            h.state.oxygen_reserve_kg - before.oxygen_reserve_kg,
        );
        h.budget.electrolysis.as_mut().unwrap().constraints =
            vec![crate::electrolysis::Constraint::HydrogenStorage];
        h.state.hydrogen_kg = 0.0;
        assert!(explain(&h)
            .explanations
            .iter()
            .any(|e| e.id == "electrolysis_constraints"
                && e.text.contains("hydrogen product storage")));
    }
    #[test]
    fn missing_evidence_stays_unknown_and_dependent_terms_do_not_claim_zero() {
        let mut h = habitat();
        assert_eq!(explain(&h).support, "waiting_for_completed_interval");
        h.step(60.0, Environment::default()).unwrap();
        assert_eq!(explain(&h).support, "environment_unavailable");
        assert!(explain(&h).heat.is_empty());
        h.step(120.0, env()).unwrap();
        h.budget.configuration = None;
        assert_eq!(explain(&h).support, "interval_configuration_unavailable");
        assert!(allocations(&h).iter().all(|a| a.served_w.is_none()));
        h.config.electrolysis = Some(Default::default());
        h.step(180.0, env()).unwrap();
        h.budget.electrolysis = None;
        let d = explain(&h);
        assert!(d.allocations[4].served_w.is_none());
        assert!(d.allocations[3].served_w.is_none());
        assert_eq!(value(&d.heat, "electrolysis"), None);
        assert_eq!(value(&d.heat, "sum"), None);
        assert_eq!(value(&d.water, "potable_net"), None);
        assert!(value(&d.water, "raw_recovered").is_some());
        h.budget.temperature_domain_supported = false;
        assert!(explain(&h).water.is_empty());
    }
    #[test]
    fn protected_exports_contain_exact_source_terms_and_valid_completion_marker() {
        let mut h = habitat();
        h.step(60.0, env()).unwrap();
        let p = std::env::temp_dir().join(format!("cosmogon-why-test-{}", std::process::id()));
        assert!(!p.exists());
        export(&h, 61.0, &p).unwrap();
        assert!(export(&h, 61.0, &p).is_err());
        let v: serde_json::Value =
            serde_json::from_slice(&std::fs::read(p.join("report.json")).unwrap()).unwrap();
        assert_eq!(v, report(&h, 61.0).unwrap());
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(p.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["complete"], true);
        let csv = std::fs::read_to_string(p.join("terms.csv")).unwrap();
        assert!(csv.contains("power,stored"));
        assert!(csv.contains("air_and_crew,cabin_oxygen_net"));
        assert!(csv.contains("allocation,base_served_w"));
        std::fs::remove_dir_all(p).unwrap();
    }
}
