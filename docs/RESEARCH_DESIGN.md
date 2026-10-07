# Research design and acceptance direction

The owner's direction is to exceed minimal feature checklists through primary-source research,
brainstorming and actual product use, while completing every master-directive area progressively.
A student, an interested explorer and a researcher should be able to use the same authoritative
state with increasing detail. A panel title does not confer research-grade accuracy.

## Design decisions implemented in this build

| Need | Mechanism | Evidence expected |
|---|---|---|
| Understand what an input does | Search, sections, units, dependency/formula info and guided questions | A radius study must produce the model's quarter-gravity consequence at doubled radius |
| Inspect orbital results | Parent-relative osculating quantities, state vectors and explicit frames | Earth/Moon checks, hyperbolic/undefined quantity handling, exports with canonical units |
| Compare alternatives fairly | Independent branches from a frozen initial state | No cross-run edits or time carryover; repeated runs/replays yield identical CSV |
| Reuse a study | Typed versioned definitions, saved base, state identity and ordered input grid | Wrong base/version/parameter rejected; non-uniform loaded grids preserved |
| Assess numerical behavior | System diagnostics, solver sweeps and measured validation reports | Actual values, case inputs, reference identity and acceptance conditions; no universal error claim |
| Inspect records | Nearest-sample cursor, zoom/pan, units, fractional/log views and event markers | Native captures and direct interaction checks; fractional change is not an error bar |
| Know what is observed | Separate estimated/modelled/procedural/user-modified states | Exact-field star provenance; missing uncertainty stays missing |
| Recover from unfinished work | Cancellation and complete/incomplete batch status | Completed exports retained; invalid plans never produce a completed batch |

REBOUND documents [energy/angular-momentum/centre-of-mass diagnostics](https://rebound.hanno-rein.de/simulation/diagnostics/)
and [reproducible simulation archives](https://rebound.hanno-rein.de/simulationarchive/).
Astropy provides [explicit physical units](https://docs.astropy.org/en/stable/units/) and
[uncertainty distributions](https://docs.astropy.org/en/stable/uncertainty/).
JPL distinguishes [coordinate frames, instantaneous osculating elements and ephemeris uncertainty](https://ssd.jpl.nasa.gov/horizons/manual.html).
The mechanisms above are our product design inference from those needs, not a claim that their
implementations or validation transfer to Cosmogon. No external library was added for branding.

## Next scientific acceptance work

A serious research workflow needs regime-specific model evidence, dataset/source versions,
uncertainty/covariance ingestion where actually supplied, compatible replay algorithms, convergence
studies, stable event/state identities and export formats usable outside the native app. A cached
endpoint regression is useful but does not prove trajectory accuracy everywhere between endpoints.
Physically meaningful significant figures are distinct from double-precision serialization digits.

Analytic/cached endpoint reports and cross-object primary-measurement studies are now implemented.
Completed-trajectory comparison, recorded-data reopening and saved guidance/favorites/graph choices are now built.
Priority next: expand sampling diagnostics, observed-data ingestion and model-specific acceptance evidence; improve graph/definition controls
and manual keyboard/assistive-technology checks; then connect observed terrain/catalogue data and
engineering quantities as their actual authoritative models are built. Never expose a decorative
control for an unimplemented solver/settlement/spacecraft quantity.

Broader controls, units/frames/time scales, uncertainty propagation, covariance-aware datasets,
observatory/ephemeris services, independent external-integrator comparisons and specialized climate,
stellar/impact models remain acceptance gaps. Guided explanations must describe those limitations
without hiding the computed result. Research, Education and Exploration should eventually become
persisted workspace modes rather than separate simulations or misleading fidelity labels.
