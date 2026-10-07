# Habitat engineering budgets

This is the first authoritative settlement-engineering slice of the master directive, following terrain study checkpoint `1e7ae1e`. Phase 9 remains partial. See [research evidence](HABITAT_RESEARCH.md). Equipment values and design bands are explicit user assumptions; the model is not validated mission hardware, medical response or settlement safety.

## Use

View → Habitat engineering, or the command palette, opens the window. Select a solid planet/moon, then Build & import → Build habitat experiment. A green/red square marks the experiment coordinate; click it to open that habitat. Real Universe reference sessions require the existing Clone to sandbox action before any build/configuration. Everything uses ordinary edit, undo and redo; adding a habitat does not activate or alter orbital dynamics.

Budgets displays the last completed interval: pressure, O2/CO2 partial pressures, temperature, battery, generation/served/shortfall power, finite inventories and unmet reference crew demand. “Run at 1 min/s” advances the ordinary world clock; it does not run a separate hidden simulation. The fixed minute scheduler can make very high cosmic time speeds CPU limited. “Pause clock” pauses the whole world. Interval flows and residuals expand for detailed inspection. These residuals establish bookkeeping, not predictive accuracy.

Controls stages every engineering setting until Apply. Reducing battery/capture capacity below stored quantity is rejected without destroying energy/mass. Initial stocks can be set through a bounded advanced JSON definition. Configuring completes any edit-time remainder under the previous configuration before applying new settings. The original definition/initial configuration stays pinned separately from the current configuration and the configuration used in the last completed interval.

Model & export provides equations, source and limits. “Save snapshot to new folder” exports one actual current observation and exact starting/ending worlds. It is explicitly a snapshot, not a time series. Existing folders/files are protected; incomplete I/O is reported, never success.

## Headless trajectories and reproduction

CLI `habitat-definition` produces a complete editable preset pinned to an existing body ID in the chosen world. Create a JSON definition with `model: "cosmogon-habitat-budget/1"`, an existing `body: {system, body}` ID, geographic coordinates, name, complete `config` and `initial` objects. Exported definition.json provides a template. Unknown fields, unsupported model versions, invalid/oversized input and unsupported placement are rejected. Body IDs identify the supplied world; a name is not substituted for a different ID.

```sh
./target/release/cosmogon-cli habitat-definition --target Mars --latitude -4.59 \
    --longitude 137.44 --output /tmp/habitat.json
./target/release/cosmogon-cli habitat --definition /tmp/habitat.json \
    --samples 25 --interval-days 0.041666666666666664 --output /tmp/habitat-day-1
./target/release/cosmogon --load /tmp/habitat-day-1/final.cosmo \
    --focus Mars --panel habitat --quiet --capture /tmp/habitat.png \
    --capture-metadata --capture-after 240 --quit
./target/release/cosmogon-cli habitat --initial /tmp/habitat-day-1/initial.cosmo \
    --samples 25 --interval-days 0.041666666666666664 --output /tmp/habitat-replay-1
```

CLI defaults to Lab and 25 hourly samples (one day); 1–1024 samples, interval 1 second–30 days and maximum total horizon 30 days. --initial supplies an exact saved world; a new --definition builds an additional habitat in that world. Without --definition, --initial resumes the existing habitat (use --habitat-id for multiple experiments). Research samples record both requested world time and last completed habitat time; intervals shorter than a minute can repeat the same habitat state. Native --habitat-definition FILE explicitly builds from the file after loading the chosen world. --panel habitat|habitat-controls|habitat-build|habitat-model supports captures. Invalid automated definitions fail before renderer startup.

Exports: definition.json (original initial setup), report.json (current configuration/state and source/limits), samples.json (actual full observer states and interval configurations), samples.csv (unit-labelled numeric columns; unavailable environment cells are empty), initial.cosmo/final.cosmo (exact worlds), manifest.json (model/source, units, sample count and FNV state identities). FNV is not a cryptographic checksum. Replaying initial.cosmo without --definition avoids constructing a duplicate. Native photo metadata retains actual habitat reports.

## Mechanisms and assumptions

The engine owns `Universe.habitats` and a fixed 60-second task registered only when habitats exist. Empty habitats are omitted in old/default serialized worlds, preserving the golden `0xf2c4140a45ae509b`. Worlds with habitat state/history pin the model version and validate stocks, configuration, IDs, source/body references and scheduling. An unavailable or removed solid host suspends budgets explicitly. Existing abstract civilization colonies do not become invented physical installations. A separate monotonic ID counter prevents reuse after deletion and remains absent from untouched worlds. Leaving the model’s 100–500 K temperature computation domain suspends subsequent budgets with an explicit alert; this interval guard is not a physically validated safety band. Rebuild an extreme experiment with viable inputs.

Mass: O2 regulator transfers from a finite cylinder; cabin ideal-gas O2/N2/CO2 are stored in kg, partial pressures follow nRT/V. NASA BVAD Rev2 Table 3-31 nominal crew cycles consume oxygen, potable water and dry/hydrated food, and produce CO2, collected wastewater and dry waste. Complete inputs/outputs conserve total stored mass. Available inputs bound reference-cycle throughput; unmet person-seconds quantify engineering demand, not starvation/asphyxiation physiology. Powered scrubber transfers CO2 into a finite capture store. Powered wastewater recovery returns the configured fraction and retains the remainder as brine; no impurity/potability performance is established.

Electricity: firm power is an explicit external boundary supply, with no implied generator fuel. Horizontal PV uses instantaneous stellar inverse-square flux and local cosine in the same approximate body frame as the renderer, multiplied by assumed efficiency/transmission. No terrain shadow, eclipse, secondary-star contribution or variable dust/weather. Battery charge/discharge power, capacity, efficiency and losses are explicit. Power priority: base, scrubber, recycler, heater, radiator pump. Zero processor power disables it. Excess generation is curtailed; unserved power accumulates. Equipment and battery are collocated with the cabin; delivered electricity and battery losses become cabin heat.

Heat: fixed lumped capacity, linear conduction to modelled global exterior temperature, bounded heater and powered Stefan–Boltzmann radiator using that temperature as its sink. Total reference metabolic heat assumes immediate condensate recovery; humidity/latent-storage dynamics are absent. Constant-interval linear conduction is integrated analytically, with a stable small-exponent series; other flows/solar and radiative controls use the fixed interval. The global planet temperature is not a local weather measurement or resolved radiator view factor. The sink/control simplification is exposed.

Alerts record entry/recovery for unavailable environment, unmet electrical demand, oxygen/CO2 design thresholds, temperature band and incomplete nominal cycles. Existing Disaster/Space notification routes control popup, inbox, sound and pause. No per-minute event spam, death prediction or safety certification. Ambient habitat-specific synthesis remains future.

## Remaining phase 9

Greenhouse, electrolysis/reaction chemistry, humidity/leaks/EVA, airlock/structural design, contaminant processing, radiation, resources, construction phases, maintenance, logistics, resolved habitat meshes, site engineering, equipment reliability/calibration and cross-habitat transfers remain unimplemented. Add them as researched mechanisms with domain tests and honest uncertainty. No claim that phase 9 or the master directive is complete.

Dynamic-save correction: loading now restores the skipped, derived acceleration cache before between-step position/illumination readers run. A dedicated N-body save/continue fixture and habitat replay cover this; no serialized trajectory or initial golden change.
