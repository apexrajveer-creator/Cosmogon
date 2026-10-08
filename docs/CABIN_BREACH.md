# Finite outward cabin breach

Open Habitat engineering → **Breach**. Overview reads actual cabin pressure/temperature, cumulative gas outside the cabin and the completed finite flow interval. Settings stages an ordinary undoable configuration edit; **Apply configuration** commits it, while Discard restores current settings. Enable an explicit breach; **zero area or coefficient closes flow**. Disabling retains escaped gas and its source/history. Units shown for throat area are mm²; definitions and simulation use m².

The model applies [NASA ideal compressible open-orifice theory](https://github.com/nasa/gunns/wiki/Standard_Flow_Equation) to finite well-mixed oxygen, nitrogen, carbon dioxide and tracked vapor. [NASA Glenn](https://www.grc.nasa.gov/www/k-12/BGP/mflchk.html) explains sonic choking. [NASA leak-path guidance](https://ntrs.nasa.gov/api/citations/20190033458/downloads/20190033458.pdf) distinguishes open holes from tight seals and molecular/tortuous leakage. This is an **open breach**, with continuum assumed; it does not convert a measured seal leak into a hole size.

Throat area, discharge coefficient, constant gamma, external receiver pressure and supported cabin pressure floor are **design assumptions**. The default enabled example has area 1 mm², coefficient 0.65, gamma 1.4, receiver 600 Pa and floor 1000 Pa. The receiver is never presented as measured Mars weather. No inward gas composition, automatic replenishment, gas from isolated cylinders/products or liquid entrainment is invented.

Only actual cabin gas is removed. Cumulative `state.escaped_gas` stays **outside the cabin**, unavailable for processing or crew use. `State::mass_kg()` still means held habitat mass; `accounted_mass_kg()` adds escaped gas for finite bookkeeping. Moisture history subtracts escaped vapor as well as condensation. A closed breach leaves stocks intact. Receiver pressure above cabin, invalid thermal decomposition or pressure/temperature support suspends the breach exchange explicitly.

Thermal coupling is a documented approximation: the existing crew/process/thermal minute runs first, then a finite breach substep. Configured lumped C is the substep's initial effective heat capacity; gas cv=R/(gamma−1) implies structure Cs=C−n₀cv. Negative Cs is unsupported. Shared gas/structure temperature follows T(n)=T₀[(Cs+ncv)/C]^(gamma−1). Separate structure, gas internal energy, constant reference vapor phase and outward enthalpy satisfy the reported substep identity. Vapor's phase energy leaves with vapor; it is not debited twice as sensible heat. The main fixed-C interval report uses a **temperature-equivalent cooling debit** C(T₀−T₁), which differs from full gas enthalpy. Both are exported and distinguished in WHY.

Deterministic step-doubled RK4 uses at most 1% current moles per attempted step, local mole disagreement at most 10⁻¹¹ of initial moles and a 1024-attempt workload limit. Fixed bisection resolves the chosen pressure/equilibrium or 100 K boundary. Start/end pressure/temperature/rates, attempts, actual resolved seconds, partial escaped components and status remain recorded. **Partial/unsupported intervals keep their raw transfers but have unavailable full-interval flow/cooling graph values.** The local criterion is not a universal global error or field-accuracy bound.

Optional configuration example, inside an ordinary habitat definition:

```json
"breach": {
  "model": "cosmogon-cabin-breach/1",
  "area_m2": 0.000001,
  "discharge_coefficient": 0.65,
  "gamma": 1.4,
  "outside_pressure_pa": 600.0,
  "minimum_pressure_pa": 1000.0
}
```

Create/edit a definition with the existing `habitat-definition`/native Build workflow. `cosmogon-cli habitat --definition FILE --output NEW_DIR` records actual states; `habitat-inspect` reopens them. New studies have **110 CSV columns**, including individual cumulative/interval escapes and separate thermal terms; previous 38/49/66/85-column studies remain readable. Exact initial/final worlds, source constants, configuration, original definition and model pin are retained. Existing exports are protected.

New graph metrics are `total_pressure`, `escaped_gas`, `breach_loss` and `breach_cooling` (the last is equivalent cooling, not enthalpy). Ordinary/comparison/paired graph navigation and WHY use retained observations independently of live state. Four new controlled settings—`breach_area_m2`, `breach_discharge`, `breach_outside_pressure_pa`, `breach_gamma`—reuse the ordinary single/paired study engine. Enable breach before sweeping its settings. Common initial stocks remain fixed; exact paired replay verifies engine reproducibility. Older designs retain their original metric sets/analyses; breach designs add the four metrics.

Native capture example:

```bash
./target/release/cosmogon --load /tmp/breach-study/final.cosmo \
    --focus Mars --speed 0 --panel habitat-breach \
    --capture /tmp/breach.png --capture-metadata --capture-after 240 --quit
```

`--panel habitat-breach-settings|habitat-breach-model` selects controls/source context. Capture and performance metadata retain current selection, staged configuration and actual shared report. Ordinary save/resume and disabled retained history require `cosmogon-cabin-breach/1`; absent legacy worlds remain unchanged, including golden `0xf2c4140a45ae509b`.

The formula, analytic/step-resolution checks and numerical identities do not establish calibrated geometry, rapid coupled-transient fidelity, hardware reliability, safe air or health outcomes. No seal/rarefied/inward flow, jet/structural forces, varying receiver, variable heat capacities, wall/ice condensation, EVA or predictive uncertainty. Physical greenhouse, logistics and the master directive remain incomplete. [Preimplementation design](CABIN_BREACH_DESIGN.md).
