# First greenhouse mechanism: controlled crop planning

Research date 2026-10-08. Connects directive §§77–78 / phase 9 to reproducible crop evidence. This component is a planning experiment; physical greenhouse installation and closed-loop habitat inventory exchange follow in a separate build.

## Evidence

[NASA BVAD Rev2, February 2022, §§4.14.1.7–4.14.1.8, printed pp180–192 / PDF pp193–205](https://ntrs.nasa.gov/api/citations/20210024855/downloads/BVAD_2.15.22-final.pdf) publishes the Modified Energy Cascade (MEC) crop equations and coefficients. For Waldmann’s Green lettuce: canopy exponent 2.5 (Table 4-97), carbon use efficiency 0.625 (4-99), nominal 16 h light / 24 h terrestrial cycle, 23°C light/dark, density 19.2 plants/m² (4-111), edible allocation 0.95 after day 1, harvest day 30, no modeled senescence before harvest (4-112), biomass carbon fraction 0.40 and oxygen production fraction 1.08 mol O2/mol carbon (4-113). Quantum yield uses the six nonzero terms in Table 4-102. Equations 4-14, 4-17–4-21 relate light absorption, carbon gain, dry biomass and O2.

Table 4-123 explicitly corrects low-light canopy closure: PPF 200 µmol/(m² s) with CO2 330/660/990/1320 µmol/mol has tA=32 d and Amax=0.18/0.35/0.46/0.49. PPF 300 and CO2 330 has tA=32 d, Amax=0.75. Using 0.93 at these settings would overstate light capture. These five exact regimes provide the initial supported cases; no interpolation or unsupported regression domain is advertised.

[NASA crop gas-flux modeling report](https://ntrs.nasa.gov/citations/20000004403) identifies crop age alone as inadequate: photosynthesis, dark respiration and transpiration depend on controlled conditions. [NASA biomass chamber tests](https://ntrs.nasa.gov/citations/20040089951) establish crop-cycle variability and distinguish edible from total dry biomass. Neither source validates this implementation or Mars greenhouse hardware.

The official PDF snapshot retrieved on this date has SHA-256 `b9eab584c2bc88b52f6eb6744e321eb72ca58003e8ac77dc1ebeb034aba11d1e`. The rendered coefficient and low-light table pages were visually checked against the transcription. Exports retain this snapshot identity; it identifies the source used, not a guarantee that future source-server responses are identical.

## Implementable inference

Use deterministic analytic integration of the fixed-regime lettuce absorption curve through day 30. Report daily/cumulative carbon, net CO2 uptake, net oxygen, total/edible/inedible **dry** biomass, controlled conditions and source coefficients. Gross photosynthesis differs from daily net carbon after respiration; CUE is applied once. Edible allocation starts on day 1, not emergence. PPF is already photons incident per planted area; apply canopy absorption once. Areas scale quantities linearly.

Add user-assumed fixture photon efficacy (µmol incident photons / joule electric) to estimate lamps-on power and 30-day electric energy. This is an engineering assumption, not a NASA measured fixture rating. Do not infer cooling, pumping, dehumidification or full facility energy from it. Native Habitat engineering exposes the planner next to actual budgets; optional frozen habitat/world context retains an exact current observation without claiming future constant CO2/temperature, electricity availability, installed equipment or any automatic resource exchange. Source constants/regimes, definition, rows and metadata export through the shared headless/native mechanism.

## Limits

Unlimited regulated gas, water and nutrients are model boundary assumptions. No simulated commissioning, resource withdrawal, nutrient availability, humidity/transpiration, gravity/pressure correction, diseases, structural design, crop stress or edible nutritional composition. No fresh mass, calories, crew-days or complete food supply inferred from lettuce dry mass. Thirty terrestrial days are not thirty Mars sols. Coefficient reproduction and analytic bookkeeping are numerical verification, not calibration or independent crop-growth validation. Prediction uncertainty remains unavailable. Closed-loop resource/state coupling must not simply add these unconstrained yields to habitat stocks.
