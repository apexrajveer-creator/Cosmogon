# Footprint support and finite supply operations — next connected mechanism

Persistent shear is integrated and accepted at `c9ab899`; [actual footprint/ground-hold evidence](VEHICLE_FOOTPRINT_SUPPORT.md) is accepted with 514 passing release tests and full packaged evidence. Neither mechanism alone makes the ship landed, attached or authorized to transfer cargo. Preserve the original directive's launch/orbit/transfer/approach/entry/landing/delivery and camera/photo scope; the current reduced contact builds are intermediate capabilities.

## Primary evidence

[MIT Underactuated Robotics, legged-robot notes](https://underactuated.csail.mit.edu/humanoids.html) treats unilateral contact forces, friction limits and their aggregate force/torque. For planar contacts it defines a force-weighted centre of pressure and relates the support hull to restricted momentum/acceleration assumptions. This supports exposing the actual contact wrench and supported planar geometry, rather than equating slow speed or a point inside a polygon with a safe landing. The notes do not validate spacecraft gear, slopes, soil, structural hazards or our solver.

NASA touchdown and contact-history references, plus their scope limits, are already recorded in [the shear memo](STATIC_FOOT_SUPPORT_DESIGN.md). Additional empirical calibration/uncertainty remain unavailable.

## Required next implementation

1. Derive a typed support reading from actual retained feet, body pose/mass/inertia and source-supported force/frame state. Retain loaded-foot count, actual reaction force/torque, individual friction/structural utilization, ground-relative translation/rotation and finite stored/slip energy.
2. Establish the planar approximation before using a planar support hull. Report normal dispersion, plane departures and coordinate resolution. Refuse unavailable/degenerate/nonplanar interpretations rather than inventing a hull or manufacturing a margin. Deterministic ordering and a bounded hull of at most eight actual contacts are sufficient.
3. Distinguish actual force-weighted centre of pressure, geometric centre-of-mass projection, required apparent-load line and momentum/force residuals. The centre of pressure being inside a hull follows positive reactions and is not a stability certificate. A margin is conditional on its declared force/frame/plane assumptions.
4. Surface operations need actual supporting motion/load evidence, source continuity and a declared envelope; keep transient contact, motion, unsupported source geometry, powered thrust and structural failures visible. Do not freeze the existing `Flying` phase or manufacture attachment from a low-speed threshold.
5. Reopen independent original-world reports with every support reading, original flight evidence and exact derived numeric/source exports. Retain original v1/v2/v3 recordings unchanged. Native and CLI actions must use the same engine-independent mechanism; no static screenshot or reference fixture as a physical state.

A historical observation cannot obtain missing source acceleration from a later live world. If a moving-ground apparent acceleration is needed, compute it from the exact replayed branch at the retained craft clock with explicit numerical/source support, and retain that evidence. Source velocity derivatives avoid subtracting large world positions twice, but still need domain/resolution tests. Kinematic/source accuracy is distinct from terrain/hardware validation.

## Delivery invariants from inspected code

Reserve oxygen, potable water, dry food and food-bound water map to distinct existing habitat stocks. Reserve oxygen must not become cabin pressure directly. Food-bound water must not become potable liquid; dry food must not automatically credit caloric or electrical energy. Crop harvest/diet remain their existing separate models.

Add explicit finite general storage/handling capacities, including shared equipment oxygen limits. Keep requests, accepted amounts, rejected held cargo and transfer mass/energy/momentum evidence separate. No unlimited tank, discarded excess or automatic timer arrival. Actual inventories remain independent of immutable original vehicle designs; changing-mass/propulsion/inertia epochs must validate the finite motion ledger rather than changing historical burn denominators.

Co-location uses the same actual body/frame/site and current supported craft clock. Any recipient stepping, thermal/handling assumptions and external material/energy ledgers must preserve existing habitat crop/breach/heat closures and ordinary undo/save/replay. Instantaneous isothermal handling, structural reactions, packaging distribution and purity/pressure are assumptions unless actually modeled. A reference transfer cannot be described as complete flight/landing/hardware feasibility.

Detailed body/terrain/civilization graphics remain queued under the owner's supplement. No graphics build is authorized in this sequence.
