# Habitat engineering: evidence and first mechanism

Research date: 2026-10-08. Implements directive §§77–78 and the start of phase 9.

## Evidence

[NASA BVAD Rev2 (February 2022), Table 3-31, printed p58 / PDF p72](https://ntrs.nasa.gov/api/citations/20210024855/downloads/BVAD_2.15.22-final.pdf): nominal 82 kg crewmember, daily exercise included. Per person-day: O2 0.895 kg, CO2 1.085 kg, dry food 0.800 kg, food water 0.760 kg, potable water 3.217 kg; wastewater 4.467 kg and dry waste 0.120 kg. Inputs and outputs total 5.672 kg. Total metabolic heat 12.426 MJ/day includes latent heat; all condensed moisture is collected immediately in this reduced model. Values vary by activity/diet/person. Section 4.1 describes historical 0.40 kPa ISS CO2 operation; this is an example editable design threshold, not current medical certification.

[NASA ECLSS overview, updated April 2025](https://www.nasa.gov/reference/environmental-control-and-life-support-systems-eclss/): wastewater processing, atmospheric revitalization and oxygen supply form connected subsystems. Recovery entails treatment/quality requirements, not an automatic claim that all recovered mass is potable.

[NASA MOXIE completion (September 2023)](https://www.nasa.gov/solar-system/nasas-oxygen-generating-experiment-moxie-completes-mars-mission/): intermittent demonstration, 16 runs, 122 g total oxygen, peak 12 g/h. This does not establish continuous rated production, life-support capacity or availability. No MOXIE capacity is used in this first model.

Ideal-gas conversion uses the [NIST 2022 CODATA molar gas constant](https://physics.nist.gov/cgi-bin/cuu/Value?r%7Csearch_for=R), 8.31446261815324 J/(mol K), and NIST Chemistry WebBook molecular weights: [O2](https://webbook.nist.gov/cgi/cbook.cgi?Name=O2) 31.9988, [N2](https://webbook.nist.gov/cgi/cbook.cgi?Name=N2) 28.0134, [CO2](https://webbook.nist.gov/cgi/cbook.cgi?ID=C124389) 44.0095 g/mol. Molar masses are converted to kg/mol; tabulated composition values are not universal exact isotope masses. The numerical constant provenance does not validate ideal-gas cabin dynamics.

## Engineering inference and build

Authoritative user-created habitats attach to an existing solid body's geographic coordinate. Existing abstract civilization colonies remain separate. No real habitat or resource deposit is invented. Optional state is absent from legacy/default worlds. A fixed 60-second scheduler owns advancement, independent of frame rate; state, configuration, source/version and edit journal are saved. Commands use ordinary edit/undo/reference protections and preserve gravity mode.

First model: finite oxygen cylinder makeup; ideal-gas cabin O2/N2/CO2; powered finite-capacity CO2 capture; potable/wastewater/brine stores; finite dry and hydrated food; nominal balanced crew exchanges; external firm power plus horizontal solar collection from the shared body-fixed frame and stellar flux; battery charge/discharge limits and losses; lumped heat capacity, conductive exterior exchange, powered heater and radiator. Equipment capacities, powers, efficiencies, external thermal sink and thresholds are user assumptions. Global body temperature is model context, not measured local weather. No actual purifier contaminant/health performance is claimed.

Expose per-step mass flows, electric energy balance, cumulative shortages and thermal terms. Record only alert entry/recovery and construction/configuration in Chronicle, feeding existing configurable notification routing. Model metadata and protected numeric exports retain inputs, state, source assumptions and units.

Nominal crew-cycle throughput is limited by available O2, food and water; unserved reference person-seconds are recorded. This is an engineering demand model, not starvation/asphyxiation physiology. No deaths or population outcomes are inferred. Crew mass is constant in the complete nominal cycle. Shortfalls mark reduced-model validity limits plainly.

## Excluded mechanisms and next work

No greenhouse, hydroponics, electrolyzer, Sabatier, humidity dynamics, leaks, airlock/EVA, microbes, fire, radiation, terrain slope, structural design, maintenance, logistics or mission hardware reliability in this first slice. No biological/safety certification. Capacity assumptions are not flight hardware measurements. Site illumination omits terrain horizon, eclipses, secondary stars and variable atmospheric transmission. Built-in residual checks establish numerical bookkeeping, not real-world predictive accuracy. Add these mechanisms through researched connected builds; phase 9 stays partial.
