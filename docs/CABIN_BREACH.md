# Finite cabin breach orifice flow (outward and opt-in inward)

Open Habitat engineering → **Breach**. Overview reads actual cabin pressure/temperature, cumulative gas outside the cabin, cumulative inward dry air (while engaged) and the completed finite flow interval. Settings stages an ordinary undoable configuration edit; **Apply configuration** commits it, while Discard restores current settings. Enable an explicit breach; **zero area or coefficient closes flow**. Disabling retains escaped gas and inward history and their source pins. Units shown for throat area are mm²; definitions and simulation use m².

The model applies [NASA ideal compressible open-orifice theory](https://github.com/nasa/gunns/wiki/Standard_Flow_Equation) to finite well-mixed oxygen, nitrogen, carbon dioxide and tracked vapor. [NASA Glenn](https://www.grc.nasa.gov/www/k-12/BGP/mflchk.html) explains sonic choking. [NASA leak-path guidance](https://ntrs.nasa.gov/api/citations/20190033458/downloads/20190033458.pdf) distinguishes open holes from tight seals and molecular/tortuous leakage. This is an **open breach**, with continuum assumed; it does not convert a measured seal leak into a hole size.

Throat area, discharge coefficient, constant gamma, external receiver pressure and supported cabin pressure floor are **design assumptions**. The default enabled example has area 1 mm², coefficient 0.65, gamma 1.4, receiver 600 Pa and floor 1000 Pa. The receiver is never presented as measured Mars weather. Gas from isolated cylinders/products or liquid entrainment is never invented.

## Outward flow (default)

Only actual cabin gas is removed. Cumulative `state.escaped_gas` stays **outside the cabin**, unavailable for processing or crew use. `State::mass_kg()` still means held habitat mass; `accounted_mass_kg()` adds escaped gas for finite bookkeeping. Moisture history subtracts escaped vapor as well as condensation. A closed breach leaves stocks intact. Receiver pressure above cabin suspends the outward exchange explicitly (`ReverseUnsupported`) unless inward flow is opted in.

## Opt-in inward flow (build 57B)

A new configuration checkbox, **Allow inward flow when receiver exceeds cabin pressure**, enables the reverse direction: when the receiver pressure exceeds cabin pressure, the same orifice law runs with the receiver as upstream. Rates are reported **negative = inward**. Inflow composition is an **authored assumption**: dry air at 21% O₂ / 79% N₂ by mole with no vapor or CO₂, at the authored **inward receiver temperature** (default 250 K, bounded 100–500 K), with no reservoir depletion and no composition mixing with the actual receiver. Imported mass accumulates in `totals.inflow_gas` and joins the cabin stocks, so `accounted_mass_kg()` grows; the finite bookkeeping identity is cabin + escaped = original + cumulative inward. Inward thermal coupling uses the imported-enthalpy relation T(n) = (C·T₀ + Δn·cp·T_in)/(Cs + n·cv) with the same constant gamma; imported enthalpy is reported as **negative outflow enthalpy** so the structure/gas/phase/outflow residual closes exactly as outward. The lumped cooling debit C(T₀−T₁) may be negative (net heating). Equilibrium stops at receiver pressure; the 100–500 K domain bounds the path (`InflowTemperatureBoundary`). The two new configuration fields serialize only when engaged, so legacy configurations, saves, recordings and the golden stay byte-identical; the model pin stays `cosmogon-cabin-breach/1`.

Thermal coupling remains a documented approximation for both directions: the existing crew/process/thermal minute runs first, then a finite breach substep. Configured lumped C is the substep's initial effective heat capacity; gas cv=R/(gamma−1) implies structure Cs=C−n₀cv. Negative Cs is unsupported. Shared gas/structure temperature follows the direction-specific relation above. Separate structure, gas internal energy, constant reference vapor phase and outflow (or imported) enthalpy satisfy the reported substep identity. Vapor's phase energy leaves with vapor; it is not debited twice as sensible heat. The main fixed-C interval report uses a **temperature-equivalent cooling debit** C(T₀−T₁), which differs from full gas enthalpy. Both are exported and distinguished in WHY.

Deterministic step-doubled RK4 uses at most 1% current moles per attempted step, local mole disagreement at most 10⁻¹¹ of initial moles and a 1024-attempt workload limit. Fixed bisection resolves the chosen pressure/equilibrium boundary, the 100 K outward boundary or the supported inward temperature bound. Start/end pressure/temperature/rates, attempts, actual resolved seconds, partial transfers and status remain recorded. **Partial/unsupported intervals keep their raw transfers but have unavailable full-interval flow/cooling graph values.** The local criterion is not a universal global error or field-accuracy bound.

Optional configuration example, inside an ordinary habitat definition (engaged inward flow):

```json
"breach": {
  "model": "cosmogon-cabin-breach/1",
  "area_m2": 0.0001,
  "discharge_coefficient": 0.65,
  "gamma": 1.4,
  "outside_pressure_pa": 110000.0,
  "minimum_pressure_pa": 1000.0,
  "allow_inward_flow": true
}
```

`allow_inward_flow` defaults to false and `inward_temperature_k` to 250 K; both are omitted from serialization while at their defaults, so legacy configurations round-trip byte-identically.

Create/edit a definition with the existing `habitat-definition`/native Build workflow. `cosmogon-cli habitat --definition FILE --output NEW_DIR` records actual states; `habitat-inspect` reopens them. New studies have **110 CSV columns**, including individual cumulative/interval escapes and separate thermal terms (inward evidence lives in the retained JSON budgets/manifest, not new columns); previous 38/49/66/85-column studies remain readable. Exact initial/final worlds, source constants, configuration, original definition and model pin are retained. Existing exports are protected.

New graph metrics are `total_pressure`, `escaped_gas`, `breach_loss` and `breach_cooling` (the last is equivalent cooling, signed; it is negative for net inward heating). Ordinary/comparison/paired graph navigation and WHY use retained observations independently of live state. Four new controlled settings—`breach_area_m2`, `breach_discharge`, `breach_outside_pressure_pa`, `breach_gamma`—reuse the ordinary single/paired study engine. Enable breach before sweeping its settings. Common initial stocks remain fixed; exact paired replay verifies engine reproducibility. Older designs retain their original metric sets/analyses; breach designs add the four metrics.

Native capture example:

```bash
./target/release/cosmogon --load /tmp/b57b/study/final.cosmo \
    --focus Mars --speed 0 --panel habitat-breach \
    --capture /tmp/breach.png --capture-metadata --capture-after 240 --quit
```

`--panel habitat-breach-settings|habitat-breach-model` selects controls/source context. Capture and performance metadata retain current selection, staged configuration and actual shared report. Ordinary save/resume and disabled retained history require `cosmogon-cabin-breach/1`; absent legacy worlds remain unchanged, including golden `0xf2c4140a45ae509b`.

The formula, analytic/step-resolution checks and numerical identities do not establish calibrated geometry, rapid coupled-transient fidelity, hardware reliability, safe air or health outcomes. No seal/rarefied flow, measured inward composition, jet/structural forces, varying receiver, variable heat capacities, wall/ice condensation, EVA or predictive uncertainty. Physical greenhouse, logistics and the master directive remain incomplete. [Preimplementation design](CABIN_BREACH_DESIGN.md).
