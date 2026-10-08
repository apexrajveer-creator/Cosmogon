# Controlled transpiration: research and implementation boundary

Prepared before implementation, 2026-10-08, following keyboard checkpoint `46f516e`. This connects the crop planner to the water/humidity requirements of master §§77–78. Installation and finite resource exchange remain later gates.

[NASA BVAD Rev2 §4.14.1.8](https://ntrs.nasa.gov/api/citations/20210024855/downloads/BVAD_2.15.22-final.pdf), printed pp186–188 / PDF pages200–202, specifies vapor-pressure deficit (4-23), gross/light-cycle net photosynthesis (4-24–25), series canopy conductance (4-26), planophile conductance (4-27), and daily transpiration (4-29). Lettuce uses the planophile case, nominal 23°C/16 h. Water constants are 18.015 g/mol and 998.23 g/L. Relative humidity is a fraction; atmospheric pressure is in kPa. Source SHA-256 remains `b9eab584c2bc88b52f6eb6744e321eb72ca58003e8ac77dc1ebeb034aba11d1e`. All three equation/table pages were rendered locally and visually checked.

[NASA hypobaric lettuce gas-exchange study](https://ntrs.nasa.gov/citations/20040089887) investigates pressure-dependent crop behavior. It does not validate this implementation or arbitrary pressure extrapolation.

## Engineering inference and intended mechanism

Retain the exact five existing low-light regimes and analytic biomass calculation. Add a separately pinned optional transpiration assumption to new version-2 definitions, with explicit user-assumed humidity and fixed terrestrial reference pressure 101.325 kPa. Older version-1 definitions remain biomass-only; do not silently invent humidity or overwrite source files. Preserve all old biomass results bit-for-bit. A newer water model/version or an attempted water extension on a version-1 definition is refused.

Compute light-cycle gross/net carbon rates per planted area, vapor pressure, stomatal/aerodynamic/series conductance, daily and cumulative transpired liquid-equivalent liters and vapor mass. The light-cycle net equation differs from daily carbon gain; apply its respiration factor exactly once. Water is a modeled vapor transfer under maintained boundary conditions, not drinking-water production, irrigation demand, condensate recovery or full facility consumption. No dark-period term is supplied by this equation. No dehumidifier rating, energy or latent-heat coefficient is invented.

At sufficiently dry air the published planophile conductance coefficient becomes negative. Reject that unsupported mathematical domain rather than clip negative flux or pretend the polynomial predicts stress. Saturated-air zero deficit is a mathematical boundary, not a recommended operating condition. This domain check is not independent experimental validation; uncertainty remains unavailable.

Use deterministic composite Simpson integration over each terrestrial day with a fixed subdivision count, and retain the difference from doubled resolution as numerical evidence only. Verify against independently implemented fine quadrature, boundary cases, area scaling, fixture independence, legacy compatibility and protected exports. Show water/units/assumptions in native curves, age inspection, comparisons, source registry and shared CLI exports. Preserve exact optional habitat context without granting any inventory credit. Native captures and an ordinary packaged run verify the connected product, not empirical crop accuracy.
