# Finite cabin moisture and condensate

Habitat engineering → Humidity adds an optional finite water-phase budget to the same live habitat, ordinary configuration edits, exact saves, CLI trajectories and reopened recordings. It prepares phase 9 for physical crop exchange without installing an unconstrained greenhouse. [Research and source/domain decisions](HUMIDITY_RESEARCH.md).

## Activate and understand

Existing definitions and worlds remain on their legacy immediate moisture collection law. Enable finite cabin moisture in Humidity → Settings or an explicit `config.moisture` definition. Activation starts with zero airborne inventory; it does not grant water or assume a starting relative humidity. Crew emissions then partition the existing nominal water output. Disabling retains vapor, isolated condensate, phase energy and version pins; stored condensate can still enter the powered recycler. Future crew moisture then uses legacy liquid collection. To continue tracking crew vapor with an idle dehumidifier, keep the model enabled and set its power or rate to zero.

The default airborne fraction is **inferred**, using NASA's nominal latent heat divided by nominal respiration/perspiration water and NIST reference phase energy. It is editable and does not establish that all sweat becomes vapor. The remaining nominal output remains liquid wastewater. Nominal crew cycles are still bounded by actual oxygen, potable water and food; shortfall is unserved demand, without physiological outcomes.

Overview shows actual vapor/liquid stocks, vapor pressure, relative humidity, current temperature, completed interval water/power/phase transfers and constraints. Settings stages the vapor fraction, RH setpoint, rate, power, electricity per kilogram and isolated liquid capacity. Apply uses ordinary undoable configuration edits; capacity below held liquid is refused. Model & sources exposes equations, source links, inference and limits.

## Shared deterministic mechanism

Vapor partial pressure follows the well-mixed ideal-gas relation and contributes to total cabin pressure. Oxygen and CO₂ retain their own partial-pressure equations. Relative humidity uses the selected NIST Antoine fit only at **273.15–303 K**, with liquid water. Outside that domain RH and RH-driven condensation are unavailable; existing water is retained. Supersaturation is displayed explicitly; there is no fabricated passive wall/ice condensation.

Crew phase energy is held with emitted vapor and debited from immediate sensible cabin heating. The dehumidifier returns constant reference phase heat when actual vapor becomes liquid. Its allocated electricity is already included in bus heat. This preserves thermal plus reference phase energy without counting nominal latent heat twice. Constant 298.15 K phase enthalpy is a reduced bookkeeping reference, not temperature-dependent latent heat or hardware COP.

Requests use start-of-interval vapor above setpoint, start liquid headroom and assumed processing rate. Power comes after base services, scrubber, recycler, heater, radiator, electrolysis and Sabatier. The process consumes only its allocated electricity and available planned vapor. New crew vapor and newly freed storage wait until the next interval. Collected liquid waits for later powered recycling, after wastewater and held Sabatier liquid; recovery splits it into assumed recovered water and retained brine. No instantaneous potable-water credit or disposal.

## Exact studies and observation

`cosmogon-cli habitat-definition` continues emitting a legacy setup; add an explicit optional configuration:

```json
"moisture": {
  "model": "cosmogon-cabin-moisture/1",
  "airborne_fraction": 0.85,
  "relative_humidity_setpoint": 0.5,
  "power_limit_w": 500.0,
  "rate_kg_s": 0.00017361111111111112,
  "electricity_j_kg": 1500000.0,
  "condensate_capacity_kg": 20.0
}
```

Here 0.85 is an explicit rounded user assumption; the native default uses the full source-derived ratio. No equipment rating is measured hardware. Unknown fields/future model and invalid/nonfinite settings are refused. Definitions are bounded regular files (64 KiB); FIFO/directory inputs are refused before opening. Native invalid definitions fail before GPU initialization.

```sh
./target/release/cosmogon-cli habitat --definition /tmp/moisture-habitat.json \
    --output /tmp/moisture-study
./target/release/cosmogon-cli habitat-inspect --recording /tmp/moisture-study
./target/release/cosmogon --load /tmp/moisture-study/final.cosmo --focus Mars \
    --panel habitat-humidity --quiet --capture /tmp/moisture.png \
    --capture-metadata --capture-after 240 --quit
```

`habitat-humidity-settings` and `habitat-humidity-model` select the other native views. Reopened recordings add vapor, condensate, RH and completed condensation-heat graphs; unavailable domains remain unavailable. Shared WHY adds actual dehumidifier power and signed phase/water terms. Protected studies retain complete optional source constants/model descriptor, exact endpoints and 85-column CSV, with unavailable interval cells blank. Existing 38-/49-/66-column recordings remain readable and every retained column is checked against JSON. Model pins survive disabling and missing/future pins are refused.

## Limits

This is a reduced numerical engineering model. No hardware calibration, biological humidity response, local humidity sensing, air circulation, adsorption, wall/ice condensation, leaks, temperature-dependent phase/COP model, contaminant removal or safe-water guarantee. Predictive uncertainty remains unquantified. Physical greenhouse resource exchange, nutritional adequacy and the wider master directive remain incomplete.
