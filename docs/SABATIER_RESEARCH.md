# Next connected reaction: finite carbon-dioxide reduction

Research date 2026-10-08. This is a proposed next phase-9 mechanism, not a completed physical greenhouse or fully closed life-support system.

## Primary evidence and source choices

[NASA ECLSS](https://www.nasa.gov/reference/environmental-control-and-life-support-systems-eclss/) describes using hydrogen from oxygen generation with captured exhaled CO₂ in Sabatier processing to produce methane and water. Flight systems can dispose of methane; this application must explicitly represent storage or an accounted external flow rather than silently create unlimited disposal.

[NIST CO₂](https://webbook.nist.gov/cgi/cbook.cgi?ID=C124389&Mask=1) reports CODATA gas formation enthalpy −393.51 ±0.13 kJ/mol. [NIST liquid water](https://webbook.nist.gov/cgi/cbook.cgi?Name=water&cTC=on) reports −285.830 ±0.040 kJ/mol. [NIST methane](https://webbook.nist.gov/cgi/cbook.cgi?ID=C74828&Mask=1) contains distinct recommendations: Chase/JANAF −74.87 kJ/mol and Manion 2002 adopted recommendation −74.6 ±0.3 kJ/mol. Choose the latter explicitly; do not attach its uncertainty to the older value. At 298.15 K, CO₂(g) + 4 H₂(g) → CH₄(g) + 2 H₂O(l) then gives ΔH = −252.750 kJ/mol CO₂. This overall reference includes condensation to liquid; it does not establish catalyst temperature, kinetics, pressure, separation or operating hardware heat rejection.

Use existing NIST molar masses CO₂ 44.0095, H₂ 2.01588 and internally derived water 18.01528 g/mol. Derive methane mass 44.0095 + 4×2.01588 − 2×18.01528 = 16.04246 g/mol to preserve internally consistent mass; the source's displayed methane mass 16.0425 is rounded. Record all choices and reported uncertainties without inferring propagated predictive confidence.

## Proposed bounded engineering mechanism

Optional version-pinned equipment consumes finite captured CO₂ and isolated hydrogen. Store finite methane and separated raw liquid product in isolated stores. Existing powered water processing consumes wastewater first, then start-of-interval raw product water using the same user-assumed recovery fraction; rejected water goes to brine. Newly produced water waits until a later interval. No purification quality is established, and this shared recycler assumption is not a claim about flight hardware. Allocate process electrical demand after existing life support and electrolysis, bounded by stocks, user-assumed processing rate/power and product capacity. Only start-of-interval inputs are available: no immediate self-recycling inside the same fixed minute.

Add reference reaction heat release to the existing lumped habitat heat budget; process electricity remains dissipated heat. Track cumulative CO₂ consumption, reference heat release, raw-water processing and held methane. Validate held hydrogen against electrolysis generation less this consumption. Disabling/reconfiguration retains products and history; decreasing a product capacity below held stock is rejected. Old/default worlds remain absent of optional chemistry; both reaction model versions remain explicit when histories depend on them.

Do not call the apparatus a validated ISS simulator or suggest that stoichiometry yields complete oxygen/water recovery. Hydrogen availability, finite methane storage, crew exchanges, resource/power limits and separate uncertainty determine the actual reduced trajectory.

## Acceptance gates and limitations

Independent molar/mass/enthalpy fixtures; finite stock/storage/zero power tests; exact coupled scheduler chunking and save/replay; legacy electrolysis/uncoupled trajectory preservation; malformed/source-version rejection; actual protected CSV/JSON reports and inspected native controls/constraints. No equilibrium solver, reactor startup/operating-temperature state, catalyst aging, gas purity, compressor/storage-pressure design, methane combustion/venting, biological greenhouse exchange or field calibration. These remain explicit gaps in the full directive.
