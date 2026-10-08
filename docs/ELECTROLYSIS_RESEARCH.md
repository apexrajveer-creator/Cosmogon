# Water electrolysis: evidence and implementation decision

Research date 2026-10-08. Connects master directive phase 9 oxygen production to existing authoritative habitat stocks, power allocation, thermal accounting, saves and experiment exports.

## Established evidence

[NASA ECLSS, April 2025](https://www.nasa.gov/reference/environmental-control-and-life-support-systems-eclss/) establishes water electrolysis as the oxygen-generation mechanism and identifies the hydrogen coproduct, which flight systems vent or feed to carbon-dioxide reduction. It does not establish this application's equipment ratings or isolated tank design. [NASA's ECLSS electrolyzer exergy analysis](https://ntrs.nasa.gov/citations/20180007962) describes dependence on current density, voltage and process losses; a universal electrical efficiency is not inferred.

[NIST water thermochemistry](https://webbook.nist.gov/cgi/cbook.cgi?Name=water&cTC=on) reports liquid standard formation enthalpy −285.830 ±0.040 kJ/mol. At the 298.15 K standard reference, 2 H₂O(l) → 2 H₂(g) + O₂(g) requires +571.660 kJ/mol O₂. This is reaction enthalpy, not minimum reversible electrical work. [NIST hydrogen](https://webbook.nist.gov/cgi/cbook.cgi?Name=H2&cTG=on) and [oxygen](https://webbook.nist.gov/cgi/cbook.cgi?Name=O2) supply molar masses 2.01588 and 31.9988 g/mol. Derive water mass as (2 M_H2 + M_O2)/2 = 18.01528 g/mol to keep stoichiometry internally balanced; NIST's displayed water mass is rounded to 18.0153. No isotopic or hardware purity resolution is inferred.

## Engineering inference and chosen reduced model

Add optional, version-pinned water electrolysis to habitat configuration; keep it absent by default so existing worlds and the golden state retain their meaning. Require positive user-assumed electrical intensity at or above reference reaction enthalpy per kg O₂, allowing a heat-only operating balance. This deliberately excludes electrolysis below the thermoneutral reference with externally supplied reaction heat. Power, maximum oxygen rate and O₂/H₂ storage capacities are user assumptions, not flight ratings.

Withdraw finite potable/process water and produce reserve O₂ plus isolated stored H₂ in exact stoichiometric mass proportions. Preserve both products in the mass ledger; no implicit disposal or unlimited venting. Prioritize base services, capture, recycling and thermal control before the optional reactor. Reserve the interval's nominal crew water demand before planning reactor electricity. Bound planned production by start-of-interval water, product headroom, assumed rate and power. Recovered water and newly freed product space become usable on the next interval. Newly produced oxygen reaches the existing regulator on the next interval; do not bypass its finite flow limit.

Subtract produced-oxygen reference reaction enthalpy from delivered bus heat. Record reactor electricity, reaction-enthalpy transfer and resulting waste heat separately; otherwise the existing all-electrical-load-as-heat rule would double count chemical energy. Retain a cumulative enthalpy-transfer ledger, not a claim about recoverable chemical energy currently stored after metabolic exchanges. Disabling equipment retains products and history; reject capacity changes that would discard held products. Source/model identity accompanies saves, reports, photos and CSV/JSON studies.

## Limits and verification requirements

No electrochemical cell voltage/current model, pressure-dependent enthalpy, water purity, compression, tank structure, hydrogen leakage/ignition, Sabatier reaction, automatic venting, fuel-cell recovery or hardware availability. Reference enthalpy stays at 298.15 K; cabin thermal state does not calibrate the reaction. Equipment parameters and predictive uncertainty remain unvalidated.

Verify independent stoichiometric and energy fixtures, finite-resource/product/power limits, crew priority, disable/reconfigure preservation, exact scheduler chunking/save replay, protected exports, source/version rejection and unchanged default golden. Inspect actual native controls and observed reaction budgets before claiming the integrated feature works. Physical greenhouse installation remains a separate acceptance gate.
