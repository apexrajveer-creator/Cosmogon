## Unreleased — entry heating and terminal landing limits

- Add `Definition.entry` / `Flight.entry_ledger` (`cosmogon-vehicle-entry/1`): opt-in versioned stagnation-point heating limits (interface altitude, nose radius, Sutton-Graves flux coefficient, flux/total-heat bounds) plus retained thermal evidence (sticky interface crossing on the craft propulsion clock, running peak flux, integrated fluence, substeps) integrated on the accepted contact substeps that already resolve the gear against the parent's existing exponential bulk atmosphere. Add `Definition.landing` / `Flight.landing_ledger` (`cosmogon-vehicle-landing/1`): opt-in versioned first-touchdown speed limits (vertical/horizontal ground-relative, liftoff bound above the authored start clearance so pad settling never counts) plus retained touchdown evidence (time, speeds, start clearance, airborne flag, substeps) on the bounded gear solver. Both stop the craft atomically at the last supported macro state with `Phase::Unsupported` carrying both numbers when a bound is exceeded; entry requires gear and an atmosphere-bearing parent, landing requires gear.
- Evaluate both ledgers after every accepted contact substep; `validate_saved` requires each ledger exactly when its limits exist and revalidates on reopen; conditional `cosmogon-vehicle-entry` / `cosmogon-vehicle-landing` save pins; `vehicle-entry`/`vehicle-landing` research-registry descriptors and model/limits study context; conditional `manifest["entry_evidence"]`/`manifest["landing_evidence"]` with exact `flight-inspect` replay. CLI `entry-definition`/`landing-definition` author pad-resolved coast definitions; `flight-definition --entry FILE|--landing FILE` reads the whole definition file and requires contact gear or a launch definition. Native world-vehicle/recorded/files panes gain conditional limits/evidence readouts, disclaimers and geographic carry-over notes above the flight details. Legacy flights serialize byte-exactly and the golden remains `0xf2c4140a45ae509b`.
- Verify 613 release tests plus one existing ignored, no new clippy warnings in touched files, a fresh app/CLI release build and an eleven-case end-to-end loop (entry study with interface crossing 13.234375 s, peak 44117.097866207376 W/m² and total 302775.0217576408 J/m², pad-settling nulls as designed, touchdown at 0.22021484375 s with 0.8176573684913913/0.016955761270384735 m/s, two atomic-limit negatives, four flag negatives, build-48/49 legacy replay regressions, inspected native captures). [Guide](docs/VEHICLE_ENTRY_LANDING.md), [verification](docs/VEHICLE_ENTRY_LANDING_VALIDATION.md).

## Unreleased — osculating orbital evidence and craft rendezvous

- Add derived orbital evidence for free-flight studies: `vehicle_orbit` (`cosmogon-orbital-evidence/1`) computes instantaneous two-body elements (`osculating`: a from −μ/2E, eccentricity vector, inclination via z-clamped acos; hyperbolic apoapsis/period are `None`, never zero) from each retained observation, using the exact `GravitySample` the craft felt plus `system.body_state` at the craft clock — never origin-time reconstruction, with sample/body/radius/μ mismatch refusals. Every free-flight study records a byte-exact-replayable `orbital.csv` plus a conditional `manifest["orbital"]` summary (initial/final elements, minimum periapsis, maximum apoapsis, `closed_at_final`); contact studies omit both and refuse a stray CSV. A `vehicle-orbital-evidence` registry descriptor (sources `horizons`, `solar-facts`) is added.
- Add CLI/native craft-to-craft rendezvous: `world_vehicle_rendezvous` (`cosmogon-vehicle-rendezvous/1`) independently replays two complete flight recordings, requires one origin fingerprint, one system and one exact observation grid, and derives closest-approach evidence (index/time, distance, relative speed and position) over the shared times. `flight-rendezvous --recording A --reference B [--output NEW]` exports a protected report (`definition.json`, `rendezvous.json`, `rendezvous.csv`) or prints the manifest; readers re-verify by recomputation. The native World flight pane gains a live "Osculating orbit about the parent body" readout for craft without contact hardware.
- No universe state, simulation law or save-schema change: derived evidence carries no save pin, legacy recordings read unchanged (missing `orbital.csv` is absent evidence) and the golden remains `0xf2c4140a45ae509b`.
- Verify 596 release tests plus one existing ignored, no new clippy warnings in touched files, a fresh app/CLI release build and a thirteen-case end-to-end loop (circular study with initial `e = 3.12e-13` and honest N-body drift, equatorial geographic study whose inclination reproduces Mars's 25.1886° obliquity and speed matches circular to 5e-9 m/s, rendezvous at 342909.446772 m closest with distance varying 0.34–7.26 Mm, replay-exact inspect, six negative cases, build-48 contact-recording regression, inspected native capture). [Guide](docs/VEHICLE_ORBIT.md), [verification](docs/VEHICLE_ORBIT_VALIDATION.md).

## Unreleased — authored finite surface launch

- Add `Definition.launch` / `Flight.launch_ledger` (`cosmogon-vehicle-launch/1`): opt-in versioned structural limits (sensed specific-force bound, liftoff height above the authored start, surface-start height bound) plus retained trajectory evidence (liftoff/burnout on the propulsion clock, running post-liftoff apogee above the local terrain, peak specific force, consumed propellant, observed substeps). Exceeding the bound stops the craft atomically at the last supported macro state with `Phase::Unsupported`; launch-limited craft structurally require contact landing gear and `Flight::new` refuses non-surface starts using the contact sampler's radial clearance.
- Evaluate the ledger after every accepted contact substep (one extra local surface sample per launch substep); `validate_saved` requires the ledger exactly when limits exist and revalidates it on reopen; conditional `cosmogon-vehicle-launch` save pin; a `vehicle-launch` research-registry descriptor and launch model/limits study context.
- Add CLI `launch-definition` (pad resolved on actual local terrain, fixed inertial zenith burn on the example bench at the authored thrust, authoring-time refusal when the sensed force would exceed the bound) and `flight-definition --launch FILE` (mutually exclusive with geographic/contact/plan options); native launch limits/trajectory-evidence readout. Legacy flights serialize byte-exactly and the golden remains `0xf2c4140a45ae509b`.
- Verify 584 release tests plus one existing ignored, no new clippy warnings in touched files, a fresh CLI release build and a seven-case end-to-end loop (definition → study with liftoff 0.79296875 s, burnout 10.00390625 s, apogee 309.48951319931075 m, peak 12.820512820512821 m/s², 50.98581064899918 kg consumed → replay-exact inspect, three negative cases). [Guide](docs/VEHICLE_LAUNCH.md), [verification](docs/VEHICLE_LAUNCH_VALIDATION.md).

## Unreleased — atomic co-located habitat transfer

- Add `Edit::TransferVehicleCargoToHabitat` (`cosmogon-vehicle-transfer/1`): one atomic undoable edit that moves held cargo from a supported craft into finite habitat storage, or refuses and changes nothing. The plan layers the build-46 handling preflight with a modeled spherical great-circle reach (`reach_m`, default 1e7, bounded) and a window-scaled mass-rate envelope (`mass_rate_kg_s × current_window_s`, end-first deterministic trim; trimmed mass stays on the craft), then ulp-clamps `held + accepted ≤ tank` exactly.
- Commit both sides or neither: the recipient is stepped to the edit time first so plan and receipt see identical post-step stocks; the craft side reuses the co-moving impulse-free `remove_cargo` (velocity unchanged, zero-relative cargo event recorded); the habitat side adds stocks plus one `Receipt`. Retained `Transfer` records carry window, separation, reach, rate envelope, ground-relative velocity, ground-absorbed momentum/kinetic energy and declared power/thermal (evidence and assumptions only). `validate_transfers` re-derives ordered cargo-event matching and receipt agreement on reopen; conditional `cosmogon-vehicle-transfer` save pin.
- Add authored study transfers applied at exact sample boundaries (before the observation), conditional `manifest["transfers"]` and `transfers.csv` with exact `flight-inspect` replay, CLI `habitat-setup` (protected habitat-bearing world) and `flight-definition --handling`, a native co-located transfer section and habitat receipts readout. Legacy worlds serialize byte-exactly and the golden remains `0xf2c4140a45ae509b`.
- Verify 575 release tests plus one existing ignored, no new clippy warnings in touched files, a fresh CLI release build and an eight-case end-to-end loop (habitat-setup → handling definition → full 170 kg transfer with a real 12.38671875 s supported window, replay-exact inspect, legacy stability, six negative/clamp cases). [Guide](docs/VEHICLE_TRANSFER.md), [verification](docs/VEHICLE_TRANSFER_VALIDATION.md).

## Unreleased — finite handling equipment and recipient storage capacity

- Add four optional finite habitat general storage tanks (defaulting to the historical 1e7 kg bound, byte-exact for legacy definitions) that bound initial stock, held stock and reconfiguration per transferable species, with an O₂-tank-above-installed-equipment-capacity coupling and potable-recovery clamping that keeps overflow as brine; add `Habitat::held_cargo`/`storage_capacity_cargo`/`available_cargo` queries.
- Add opt-in authored craft handling equipment (`cosmogon-vehicle-handling/1`) with per-species, total, shared oxygen-interface and required-operation-window limits plus declared-but-unmodeled power/thermal assumptions (structurally requiring cargo ledgers and surface-operation hardware), and a pure request/accept/reject preflight with a deterministic trim order that keeps rejected stock explicit on the craft. Nothing transfers, is received or delivered.
- Add conditional `manifest["handling"]` study evidence with exact `flight-inspect` replay, a native tanks/readout exposure and a conditional `cosmogon-vehicle-handling` save pin. Legacy worlds serialize byte-exactly and the golden remains `0xf2c4140a45ae509b`.
- Verify 562 release tests plus one existing ignored, no new clippy warnings in touched files, a fresh CLI release build and a seven-case end-to-end loop (contact-study handling manifest, geographic landing with a real 9.38671875 s supported window, replay-exact inspect, legacy stability, four negative cases). [Guide](docs/VEHICLE_HANDLING_CAPACITY.md), [verification](docs/VEHICLE_HANDLING_CAPACITY_VALIDATION.md).

## Unreleased — cargo mass in authoritative world flight

- Add opt-in strictly versioned per-flight cargo mass ledgers (`cosmogon-vehicle-cargo-mass-epochs/1`) so held cargo is part of the craft's real mass in every integration step of both the spherical and rigid-contact solvers, with `Flight::mass_kg()` as the single authoritative accessor and epoch-correct thrust/mass-flow/force/inertia substeps.
- Add one shared removal path: interactive undoable `Edit::RemoveVehicleCargo` and authored timed `Definition.cargo_removals` applied at exact represented craft-clock boundaries, with exact linear momentum closure, bounded relative ejection (`MAX_CARGO_EJECTION_SPEED_M_S = 1000`, co-moving removal impulse-free) and the uniform-gyration layout rule (craft angular velocity exactly unchanged; offset layouts not representable). Every removal records a `CargoEvent`; saved worlds re-derive momentum/mass/angular closure, ledger piecewise counters and authored coverage, and tampering refuses reopen.
- Add conditional `manifest["cargo"]` study evidence with exact `flight-inspect` replay, CLI `flight-definition --cargo-ledgers`, a native held-cargo display and removal control, and a conditional `cosmogon-vehicle-cargo-flight` save pin. Legacy flights serialize byte-exactly and the golden remains `0xf2c4140a45ae509b`. Removal does not deliver or transfer cargo; storage/recipient capacity remains future work.
- Verify 548 release tests plus one existing ignored, no new clippy warnings in touched files, a fresh CLI release build and an eight-case end-to-end loop (definition → study → inspect manifest equality, authored mid-study removal 5000 → 4200 kg held with an event at exactly 0.5 s, contact+operation composition, three negative cases). [Guide](docs/VEHICLE_CARGO_FLIGHT.md), [verification](docs/VEHICLE_CARGO_FLIGHT_VALIDATION.md).

## Unreleased — solver-integrated surface-operation window

- Add `cosmogon-vehicle-surface-operation/1`: a continuous supported-operation window evaluated at every accepted contact-solver midpoint and endpoint against the authored footprint-support envelope, retained as a versioned flight ledger with counters, bounded exclusion reasons and whole-macro-step rollback. Conditional save pin for enabled operation hardware; absent legacy flights keep no ledger and the golden remains `0xf2c4140a45ae509b`.
- Add protected `operation.csv`/manifest window evidence to world-flight recordings and exact independent replay in `flight-inspect`; CLI `contact-hardware --operation-window` authors the window. Detailed foot-shear refusal diagnostics now state the failing quantities inside the unchanged refusal prefix.
- Verify 539 release tests plus one existing ignored and no new clippy warnings. End-to-end: millimeter-gap geographic start on real 4ppd MOLA height with a coast plan holds `flying` for 20 s and grows the window to 9.38671875 s ≥ 5 s required (5800 supported samples), with replay-exact inspect and negative flag cases. [Guide](docs/VEHICLE_OPERATION.md), [verification](docs/VEHICLE_OPERATION_VALIDATION.md). The window is a modeled envelope, not attachment, storage, handling or delivery; graphics remain queued.

## Unreleased — actual vehicle footprint and ground-hold evidence

- Add actual per-foot loads/utilization, shared-plane guards, deterministic loaded-site hulls, geometric/force-weighted/applied-wrench margins and conditional ground-following force/torque comparisons. Keep compliant force application separate from projected ground sites; retain source derivative/coordinate indicators and motion exclusions.
- Add a native equal-scale footprint diagram, typed authored limits, frozen original-world reports and protected exact numeric/nested-flight exports, with strict reopening and headless commands. Original flight formats, simulation laws/saves and legacy golden remain unchanged. Instantaneous inclusion does not attach a craft, prove future stability or permit cargo access. Graphics remain queued.
- Independent Mars rail comparison exposes existing source-operator approximation error separately from numerical derivative refinement. The original orbital operator and golden remain unchanged.
- Verify 514 release tests plus one existing ignored, six release/package checks, 45 CLI/native cases, eight inspected captures and a 60.053035-second ordinary run. Correct new style warnings and bring the full diagram/legend ahead of numeric detail; all 173 frozen Rust files match the checked app.

## Unreleased — integrated persistent foot shear

- Add explicit world-flight version3 / terrain-contact version2 with actual per-foot history, objective rotating-ground transport and finite tangential spring/Coulomb response in coupled motion. Keep endpoint constitutive and actual midpoint mechanical balances separate, including numerical/storage/slip/unloading terms and actual residuals.
- Connect native typed controls/graphs, explicit CLI hardware, conditional strict save pins and protected independently replayed shear CSV/nested approach evidence. Old spherical/smooth-contact recordings and absent legacy state retain their semantics; enabled state deliberately changes under separate pins. The golden remains `0xf2c4140a45ae509b`. Stable footprint/attachment, safe landing and cargo transfer remain incomplete.
- Repair checked-build overflow in the existing south-hemisphere rendering key with explicit modulo-u64 arithmetic, preserving released generated values. No new graphics, shaders, assets or dependencies; graphics remain queued. All 505 release tests plus one existing ignored pass; release/package checks, 67 CLI/native cases, six inspected screenshots and a 48.448818-second live run pass. [Verification and numerical limits](docs/VEHICLE_STATIC_SHEAR_VALIDATION.md).

## Unreleased — persistent foot shear foundation

- Add a separately versioned finite tangential spring/slider, objective terrain-frame history transport, retained plastic slip/unloading release/numerical dissipation and structural refusals. Eight component tests cover static force, independent supported-mass equilibrium, changing directions, separation, objective motion and refinement; all 500 release workspace tests plus one existing ignored pass.
- This mechanical component is not yet enabled in world flight. Existing flight/contact/save semantics, absent legacy state and golden remain unchanged. Actual multi-foot coupling, stability, attachment and delivery remain in progress; packaged terrain evidence remains the preceding 492-test checkpoint. Graphics remain queued.

## Unreleased — finite terrain and rigid-body vehicle response

- Add explicit version-2 contact hardware, actual rotating modeled-terrain feet, finite unilateral force/stroke/load/sliding friction and coupled centre/Euler-quaternion motion. Retain authored inertia/exhaust assumptions and actual contact/translation/rotation work and angular-momentum residuals; structural/source/precision/work failures stop at the prior supported state.
- Native typed hardware/geographic controls, actual live/recorded contact readings/curves, separately pinned saves and protected full-world/contact CSV replay. Explicit signed contact starts support below-spherical-base MOLA terrain; original positive geographic starts, spherical impacts and version-1 flight/approach exports retain their semantics.
- Enabled contact state/fingerprints deliberately change under separate capability/model pins. Absent legacy state and golden `0xf2c4140a45ae509b` are unchanged. Static friction, footprint stability/attachment, safe landing, physical cargo transfer, entry/hardware calibration and the full directive remain incomplete. Graphics remain queued; no shader, asset or dependency changes.

## Unreleased — rotating vehicle arrival

- Add rotating spacecraft approach frames, derivative-aware geographic initial conditions and original-branch approach recordings with native/CLI exact evidence, finite mass/energy, historical contact frames and explicit pole/terrain support. World/save/physics/old flight formats and golden are unchanged; terrain touchdown/gear response, delivery and graphics remain incomplete.

## Unreleased — physical finite vehicle flight

- Connect authored finite propellant/cargo to optional actual world spacecraft, fixed deterministic gravity/variable-mass motion, explicit spherical first contacts and retained unsupported-state reasons. Ordinary create/remove edits, undo/redo, saves and model/source pins share the authoritative world.
- Native live controls and independent pinned-world studies, protected complete origins/final worlds/36-column CSV/JSON, strict full-world reopening/replay, stable bytes and request-time worker controls. Preserve exact previous propulsion recordings; first contact is not a safe landing or delivery. Gravity/frame discontinuities currently stop unresolved flight explicitly.
- Retain habitat scheduler slots while later vehicle tasks exist; park terminal flight on the completed task time so fractional caller chunks preserve identical state. Bound study work and estimated payload before allocation. Enabled fleet state/fingerprints deliberately change; absent legacy worlds omit the new state and retain golden `0xf2c4140a45ae509b`. No graphics or dependency change; physical landing/loading/delivery, hardware fidelity and phase 10 remain incomplete.

## Unreleased — finite supply-vehicle design and propulsion

- Shared authored dry/tank/propellant/payload and finite cabin-consumable cargo design, deterministic constant-effective-Isp burns, exact fuel cutoff and actual mass/impulse/vector-capability observations. Add stable deterministic `dln1p` and expose raw numerical residuals.
- Native designer and CLI protected complete plans/24-column CSV/JSON/source exports, strict independent reopening and request-time controls without live world or supply grants. Save laws/catalogs/source constants, graphics and golden `0xf2c4140a45ae509b` remain unchanged; physical flight/delivery and phase 10 remain incomplete.

## Unreleased — frozen habitat study origins

- Explicit original-saved/current-live starting-world selection, cached frozen preflight, exact finite stores/clocks/source pins and immutable request-time context for new single/paired experiments.
- Shared `habitat-rerun` and packaged rerun/export controls, fresh destinations, changed-source/ordinary-edit refusals and successful-result-only view changes. Preserve solver, save/catalog formats, source constants, graphics and golden `0xf2c4140a45ae509b`.

## Unreleased — complete saved single-setting habitat designs

- Reopen all independent common-state branches with strict plan/source/edit/time/catalog checks, actual branch choices, restored fresh-output controls and protected all-branch trajectories/selected figures.
- Independently replay every retained branch and observation through shared native/CLI tools; preserve old paired reports and ordinary figure serialization, solver/saves and golden. Long paired setting descriptions wrap within the pane.

## Unreleased — controlled installed crop studies

- Four typed ordinary-edit controls for ideal chamber assumptions, auxiliary power and finite dry/shared oxygen capacity; exact retained original state, strict boolean/no-discard preflight and plan-dependent actual crop responses.
- Shared native/CLI single and paired studies, explicit crop/thermal source context and restored staged controls. Previous metric catalogs, simulation/save state and golden stay unchanged.

## Unreleased — recorded actual crop trajectories

- Add explicit retained crop progress/stocks/material/electric/chemical curves, signed heat debit selections, aligned-origin/completed-time comparisons and protected source-context research figures through existing native/CLI tools. Keep actual zeros distinct from absent state, disclose ideal chamber/source progress limits and preserve immutable saved catalogs, old analyses, solver/save state and golden `0xf2c4140a45ae509b`. Write optional crop CSV before habitat completion manifest. No graphics/dependency or calibrated biology claim.

## Unreleased — finite installed crop process

- Connect explicit originally installed finite crop state to real model water/CO₂/nutrient feeds, bounded oxygen/dry stores, allocated source lighting and chemical/vapor heat storage. Add native setup/controls/harvest, exact save pins, protected actual crop CSV/reopening and ordinary undoable harvest without crew diet credit or hidden replanting. Ideal maintained conditions and chemical energy remain user assumptions; source progress is not biological age. Absent legacy state and golden `0xf2c4140a45ae509b` stay unchanged; enabled habitat state/fingerprints deliberately change. No shader/graphics/dependency changes or calibrated biology claim.

## Unreleased — cumulative cabin heat accounting

- Retain passive signed heat-path/storage counters for every actual executed habitat interval, excluded coverage and capacity transitions. New definitions explicitly enable model-pinned accounting; absent legacy definitions preserve identical physics/state/replay. Native/CLI recorded Heat ledger and optional protected heat CSV expose exact span differences/residuals. Existing temperature solver and untouched-universe golden `0xf2c4140a45ae509b` stay unchanged; enabled habitat state fingerprints deliberately change. No backfilled history or hardware calibration.

## Unreleased — recorded cabin balances

- Add shared native/CLI exact endpoint stocks, cumulative model counters, mass/electrical identities and temperature references, all four independent paired branches and protected source-context JSON/CSV. Preserve original inputs, simulation laws/saves and golden `0xf2c4140a45ae509b`; never extrapolate sparse minute budgets into heat history.

## Unreleased — recorded research figures

- Add shared native/CLI protected vector figures with original full-precision numbers, complete typed observations/source identity, paired method context and reusable strict graph layouts. Actual samples, unavailable gaps and positive-log omissions remain explicit; resolve small-signal bounds/ticks and capture native export inputs at request time. No smoothing, invented uncertainty, simulation law/save/dependency change; golden `0xf2c4140a45ae509b` remains unchanged.

## Unreleased — controlled cabin geometry and thermal studies

- Add four typed volume/capacity/conductance/radiator studies, illustrative editable levels, fixed-inventory method/source notes, plan-dependent total-pressure catalogs, protected paired trajectories and exact independent replay. Reopened designs restore their actual typed controls with a fresh output field; staged inputs remain separate from completed evidence. Preserve legacy analyses, simulation laws/saves and golden `0xf2c4140a45ae509b`. No physical-expansion, equal-initial-energy or equipment-calibration claim.

## Unreleased — recorded pressure explanations

- Add actual cabin gas components, explicitly ordered inventory/temperature/volume accounting between retained observations, and separately scoped finite breach substep evidence. Native tables, CLI and protected exact-input JSON/CSV exports share one derivation; preserve original studies, simulation laws/saves and golden `0xf2c4140a45ae509b`. Mathematical accounting is distinct from unique physical causation or field calibration.

# Changelog

## Unreleased — finite cabin breach

- Connect optional source-pinned outward open-orifice flow to finite O₂/N₂/CO₂/vapor, retained outside mass history, supported bounded integration and separate gas/structure/phase/enthalpy identities. Native controls, WHY, protected 110-column recordings and four controlled settings share ordinary edits/saves/paired replay. Main heat report explicitly distinguishes temperature-equivalent cooling from outflow enthalpy. Absent legacy states and golden `0xf2c4140a45ae509b` remain unchanged. Constant-property/minute split approximation; no seal/rarefied/inward flow, calibrated geometry or EVA claim.

## Unreleased — recorded graph navigation

- Add actual-time viewport zoom/pan/selection, truthful positive/signed log display and protected reusable layouts to ordinary/comparison/paired habitat graphs. Preserve full raw observations, selected/final analyses, simulation laws, saves and golden `0xf2c4140a45ae509b`. Correct replay diagnostic comparison warnings and retain signed-zero differences.

## Unreleased — saved habitat analysis

- Add selectable stored-time paired response tables/effect curves and protected complete numeric exports, plus explicit independent whole-design replay verification. Reconstruct all four branches, compare every complete typed observation and endpoint/source identity, retain bounded differences, native progress/cancellation and protected reports. Preserve original recordings/final analyses, simulation laws, saves and golden `0xf2c4140a45ae509b`; no empirical calibration claim.

## Unreleased — finite cabin moisture
- Add paired two-setting/two-level habitat designs, four atomic common-state branches, actual conditional/interaction contrasts and protected source-pinned results with shared native/CLI reopening. Preserve single-setting plans, simulation laws, saves and the golden.

- Add six controlled moisture settings to shared native/CLI habitat experiments with exact common inventories, all-branch preflight and actual phase/water comparisons. Clarify completed moisture intervals/disable semantics and expose phase heat terms before WHY aggregates. No simulation law, save schema or golden change.


Optional source-pinned finite vapor/isolated condensate, assumed crew moisture partition, NIST-supported RH, conservative dehumidifier bus allocation, explicit crew phase debit/condensation heat and later powered liquid recycling. Native Overview/Settings/Model views, ordinary undoable edits, exact retained model pins, shared WHY and protected 85-column studies/recording graphs. Legacy configurations and golden `0xf2c4140a45ae509b` remain unchanged. No wall/ice condensation, hardware calibration, purification quality or physical greenhouse exchange claim.

## Unreleased — controlled crop transpiration

Source-pinned optional light-period water/vapor forecasts, explicit maintained humidity at fixed terrestrial reference pressure, gross/light-cycle net carbon and conductance, deterministic quadrature with resolution evidence, native water graphs/comparisons and protected numeric exports. Version-1 definitions preserve identical biomass without fabricated humidity/water; negative source conductance and unknown water versions are refused. No installed greenhouse, finite habitat exchange, field calibration, simulation/save/golden or dependency change.

## Unreleased — authored camera paths
* Native keyboard customization for 24 camera/everyday actions, shared input and cinematic interruption, capture/cancel/release gating, conflict validation, dynamic help, explicit protected versioned files and separate preferences. No Universe/save/golden change.


App-only timed keyframes, normalized orientation/look-at and lens interpolation, native authoring from actual/saved views, explicit Play/Pause/Stop/scrub, protected schema-1 files and capture/performance evidence of evaluated versus actual ground-corrected positions. Simulation laws, save schema and golden `0xf2c4140a45ae509b` unchanged. Video, DOF, spline paths and simulation-rate synchronization remain incomplete.

## Unreleased — native exploration, research and habitat engineering

- Add shared completed-interval habitat WHY diagnostics, native live/recorded power/water/heat/air explanations and protected source/CSV exports. Retain interval configuration, recorded constraints, explicit unavailable dependencies and unit/formula evidence; reuse allocations in equipment audio. Legacy studies remain readable and present diagnostic reports are checked. No physical state, source version, dependency or golden change.

- Add an optional live habitat interior listening preview with five equipment textures driven by actual completed power budgets, separate environment mixing, bounded streaming synthesis and protected eight-second WAV/source exports. Context resets on load/undo/removal; capture/photo/quiet modes suppress playback. Designed sonification, with no calibrated acoustic or physical interior claim. Simulation state and golden remain unchanged.

- Add controlled habitat single-setting experiments through ordinary deterministic configuration edits, preserving common initial inventory and protected per-run evidence. Native worker sweeps and two-recording curves share exact stored-time/interval-supported differences with CLI JSON/CSV. No interpolation, field-validation, global-optimization, save or golden claim.

- Reopen bounded habitat studies in native Recordings with actual metric curves, explicit unavailable gaps and selected rows. Shared CLI inspection checks source/endpoint/fingerprint/budget/report/CSV consistency; asynchronous transactional imports and photo metadata preserve recorded/live scope. No world or golden change.

- Connect optional finite Sabatier CO₂ reduction, isolated methane/raw water, later powered recycling and reference reaction heat to habitat controls, saves and actual source-pinned exports. Ratings are assumptions; no closed-loop or hardware-validation claim. Existing optional electrolysis and golden world are unchanged.

- Add optional finite habitat water electrolysis with separated O₂/H₂ products, crew/power priority, reaction-enthalpy and waste-heat accounting, source/version pins, native controls and actual research exports. Legacy defaults and golden `0xf2c4140a45ae509b` remain unchanged. Equipment is assumed and uncalibrated; physical greenhouse coupling remains incomplete.

- Add source-pinned controlled lettuce crop planning from NASA MEC equations and corrected low-light regimes. Shared native/headless calculations retain source snapshot, analytic cumulative growth/gas exchange, explicit assumed lighting demand, integer-age graphs, protected numeric/comparison exports and exact optional habitat context. Unlimited controlled inputs are exposed; no installation, inventory credit or closed-loop greenhouse claim. No Universe/golden change.

- Add authoritative user-created geographic habitat experiments with finite mass stores, NASA BVAD Rev2 nominal crew exchanges, powered air/water processing, primary-star solar/external supply, battery losses and lumped heat budgets. Ordinary undoable edits, exact save/resume, design-alert entry/recovery, native controls/markers, model/source context and protected snapshot/trajectory exports share the engine. Optional state and monotonic identifiers are omitted from untouched worlds; the golden remains `0xf2c4140a45ae509b`. Phase 9 remains partial, with unquantified predictive uncertainty and no safety/medical certification.
- Restore derived N-body acceleration caches on save loading before between-step position/illumination readers run. Dynamic save/continue and habitat replay fixtures cover the correction without changing serialized initial trajectories.

- Reopen native numeric terrain studies with exact pinned source/sphere/samples and transactional failure recovery. Show source-aware sampled paths and selected rows on Mars; suppress stale, hidden or mismatched traces. Preserve independent study graphs/exports and capture actual study definitions/rows in photo metadata. Add explicit current-world adaptation and bounded pre-render automation checks. No Universe or golden change.

- Add independently pinned NASA MOLA 16ppd relief and coverage, with exact source/label/provider checksums and reproducible conversion. Explicit undoable source selection shares heights across terrain, probes, landing markers, camera travel and numeric studies. Save/version and bookmark/section mismatch checks preserve reproducibility; terrain/bake caches follow source changes. Existing worlds and new defaults stay unchanged, so the golden remains `0xf2c4140a45ae509b`. Finer pixels retain interpolated support and unavailable uncertainty; no local safety/accuracy claim.

- Add shared deterministic MOLA terrain sections between historical/custom endpoints, actual native height-distance graphs and row-to-probe/travel actions. Protect source-supported CSV/definition/JSON exports and headless replay; retain source-cell weights/counts, checksums, unavailable uncertainty and overview/geodesic limits. No Universe or golden change.

- Add four cited historical Mars landing references, native surface selection/markers, shared f64 geographic camera transforms, radial-altitude travel and explicit snapshot/tracked modes. Tracked viewpoint anchors follow spin/translation and restore lens/site identity without rewinding; version-1 libraries still load and unknown versions remain protected. Native/headless probe JSON and photo sidecars retain provenance/registration limits. No Universe schema or golden change.

- New Solar System worlds attach NASA PDS MOLA 4-pixels/degree observed-derived Mars relief with observation-count coverage, source/checksum metadata and native/headless coordinate probes. Mars no longer uses generated elevation; colours/climate/fine shading remain generated/modelled. Existing saves keep their stored terrain identifier. This deliberate new-state change re-pins the deterministic golden fingerprint from `0x579903fd05a28945` to `0xf2c4140a45ae509b`.

* Adopt the owner's master development directive; add current-state audit, evidence/limitations,
  product vision, source/asset inventories and active phased gap analysis while preserving historical plans.
* Add an optional Performance panel (View menu and command palette), frame-time chart and
  bounded average/p50/p95/p99/max recordings, simulation timing and async universe-load timing.
* Export local versioned JSON with scene/camera/seed/dataset context and raw samples;
  `--profile`, `--profile-frames`, `--profile-warmup` enable reproducible native runs.
* Opt-in process/render diagnostics persist as an app preference. Unsupported GPU timings and
  unmeasured VRAM are explicitly unavailable. Export errors are visible; earlier files are protected.
* Add six-degree free flight with f64 position/orientation, adaptive/manual travel speed, roll,
  boost/fine movement, optional inertia, sensitivity and vertical inversion. Preserve orbit/focus
  transitions; nearby solid bodies drive terrain loading in free flight.
* Add Camera & Photo controls for presentation-only field of view, exposure, warmth and saturation.
  Save camera/lens preferences independently of Universe. Pause and hide-interface controls are explicit.
* Add a persistent, versioned viewpoint library with object-relative positions, stable anchor IDs,
  lens restoration and protection against overwriting invalid/newer libraries.
* PNG capture reports actual completion/failure, preserves the original overlay state and optionally
  writes camera/dataset metadata in a separate `.png.json` sidecar. Developer capture failures exit nonzero.
* Reject misleading zero fallback process counters and unsupported Metal GPU timestamp diagnostics.
* Add Realistic/Cinematic/Hybrid audio modes and independent music, cinematic-effects and alert
  levels. Live mute/mode changes apply to active sounds; leaving a universe stops its audio.
* Add a searchable recent notification inbox, category/severity/unread filters and object navigation.
  Per-category popup/inbox/sound/pause routes and priority thresholds persist in app settings.
  Do Not Disturb/photo suppress interruptions; bell bursts are coalesced, pause defaults off.
* Preference writes now replace a temporary file and new controls report failures. Older sound
  preferences retain their released mix. Developer `--sound-mode` and `--quiet` support native QA.
* Match free-flight ground clearance and near-plane/bloom behavior to orbit mode after native captures
  exposed close-range terrain precision artifacts.
* Repair macOS packaging in synced/file-provider folders by assembling/signing on the temporary
  filesystem; keep the launchable app in the local build cache, link it from `dist`, and clean staging
  on exit. The disk image remains standalone.
* Propagate Bevy AppExit from the native entry point so failed developer exports, captures and
  loads return a nonzero process status. Native verification exposed the previously discarded status.
* Add searchable native research controls over 36 shared typed parameters, dependency/formula info,
  locked derived values and undoable input changes. Read-only reference views retain protection.
* Add seven model descriptions with methods, assumptions, fidelity/cost, error limits and primary
  sources; distinguish estimated catalogue inputs from procedural stars and modelled outputs.
* Add bounded native observer plots and protected six-file CSV/initial-state/edit/event/model bundles.
  Paused edits have distinct edit_count rows; rewind stops a recording. Missing uncertainty is null.
* Add CLI model registry export and uniform-simulation-time experiments, plus coordinated native
  developer research/capture/profile exports. Add frozen-state definition replay and bounded independent parameter sweeps; no blanket scientific validation claimed.
* Add parent-relative orbital measurements and bounded dynamic-particle Newtonian diagnostics.
  Charts support nearest-state inspection, zoom/pan, log scale, fractional changes and event markers.
* Add cancellable native background studies with guided questions, saved prepared definitions,
  ordered sweep values and canonical-unit outcome comparison. CLI sweep/replay shares the runner.
  Frozen-state fingerprints reject a changed base; incomplete batches retain completed runs.
* Calculate native/headless analytic circular-orbit and cached JPL endpoint validation reports with actual residuals, initial/reference vectors, units, solver conditions and explicit coverage limits.
* Allow independent changed/measured objects and a persisted primary measurement. Surface completed branches from cancelled/failed studies without reporting success; open computed results automatically.
* Fit research scroll areas within the window, separate per-tab scroll state, add CLI help and graceful argument errors; research creation defaults to Solar Lab.
* Reopen observer bundles through a bounded shared numeric reader and CLI inspect; verify keys, units, finite rows and state ordering. Load up to eight completed study trajectories off the main thread.
* Add shared elapsed-time graph zoom/cursor, nearest-state keyboard inspection, multiple computed branches and explicit scale guards. Add saved graph/time choices, parameter favorites and Explore/Learn/Research guidance, including current formula inputs.
* Preserve invalid/newer workspace files and expose save/load errors; coordinate capture/quit with imported data and completed studies.
* Simulation state and save format unchanged; **golden fingerprint unchanged**.

## 1.0.0 — The whole universe (2026-10-04)
Plan and rationale: docs/PRODUCT_PLAN_1_0.md.
### Added
* **Life and death of stars**: main sequence → red giant (swelling, brightening, swallowing inner
  planets, shedding mass so surviving orbits widen) → planetary nebula and white dwarf, or supernova
  and neutron star / black hole by initial mass; supernova radiation fronts cross interstellar space
  at light speed and sterilise worlds in their path.
* **Compact objects you can create and throw**: black holes with ray-traced gravitational lensing,
  photon ring and Doppler-beamed accretion disks; neutron stars with pulsar beams; white dwarfs;
  extra stars.
* **Star formation**: stellar nurseries turn gas into new star systems until the cloud is spent; the
  real **Orion Nebula** with the Trapezium stars at 1,344 ly; ray-marched emission nebulae.
* **Galaxy scale**: the Milky Way seen from outside at its real orientation (bar, arms, dust, star-
  forming knots), the Local Group (Andromeda, Triangulum, Magellanic Clouds) at catalogued positions,
  and a deep field of galaxies and quasars along a cosmic web; the camera pivots to the Galactic Centre
  as you pull back.
* **Spectacle physics**: catastrophic disruption after Leinhardt & Stewart (2012) with debris, molten
  remnants and boiling oceans; moons inside the Roche limit become rings; impact flashes and shock
  rings; comets with sublimation-driven comae, ion tails and curved dust tails.
* **Throwing, Universe Sandbox style**: Throw (drag to fling, click for a circular orbit, live
  trajectory) and Grab tools, an object shelf, Aim and Create in a tool dock.
* **Civilizations across the stars**: Dyson swarms (Kardashev II), terraforming of colony worlds,
  generation ships that found new branch civilizations at stars scouted by probes.
* **Interface**: Inter + Phosphor icons, icon top bar with search, redesigned universe tree, time bar
  with speed slider, **guided tours** (four), **timeline with rewind**, **photo mode** (P),
  synthesised **sound**, decluttered labels.
* **Experiments**: Halley's Comet, Theia, a second sun, a feeding black hole, the Sun as a red giant
  or a dying supergiant, a Kardashev II humanity, and more.
### Changed
* Golden fingerprint re-pinned (new civilization fields and technologies); old saves load.
* Technologies that build on not-yet-invented ones can no longer leak into present-day Earth.
### Fixed
* Far-distance f32 overflow in direction normalisation (camera and galaxy shader).
* Inter's private-use glyphs shadowing Phosphor icons.

## 0.5.0 — Worlds and civilizations you can see (2026-10-04)
Details: docs/MILESTONE_G1_C1.md.
### Added
* **Appearance model**: every planet, moon and exoplanet gets a physically chosen style (cratered,
  volcanic, ice shell, grooved ice, tiger stripes, haze, cloud deck, lava/magma ocean, Sudarsky-class
  giants with belts, jets and storms); Jupiter's GRS and ovals, Saturn's hexagon, Neptune's dark vortex.
* **Rendering**: atmospheric single scattering, eclipse shadows from moons and planets, ring shadows,
  Saturn's measured ring structure, stellar limb darkening/granulation/spots, lava and thermal glow,
  procedural detail and bump mapping beyond texture resolution, mip-maps.
* **Present-day humanity** in the Solar System Lab: real cities and countries, exploration record,
  missions in flight (BepiColombo, Europa Clipper, JUICE), 11 000 satellites.
* **Space programmes**: missions to every world (flyby → orbiter → lander → crewed), colony ships,
  energy transition away from fossil fuels; Kardashev rating.
* **Visible civilizations**: at-a-glance card, space programme panel, planet badges, satellite shells
  incl. geostationary belt, stations, spacecraft in flight, colony sites and lights, cities and farmland
  on the ground.
* **Interventions**: Inspire, Signal, Hardship, Share knowledge, Teach a technology — with a 25-year
  cooldown and no skipping of prerequisites; undoable sandbox edits.
### Changed
* Only the Garden World template guarantees a habitable planet; hot Jupiters can now form.
* Golden fingerprint re-pinned (energy transition changes long runs).
### Fixed
* Stray terrain quads and one-frame flashes of newly spawned bodies.

## 0.4.0 — Sandbox Foundation (2026-10-03)
Cosmogon becomes a universe sandbox. Details: docs/MILESTONE_S1.md.
### Added
* **Home & sandboxes**: new application shell (intro, New Sandbox with seven templates, Continue, Load,
  Real Universe, Scenarios, Settings, Credits); sandbox documents with manifest, autosave (also on quit),
  Save As, Duplicate, Rename, Delete, checkpoints, thumbnails; legacy-save import; local profile.
* **Real data**: NASA/JPL Horizons pipeline (`tools/fetch_horizons.py`) and cached 2026-01-01 state vectors;
  Solar System Lab starts from them; Real Universe view (read-only) with Clone to sandbox; per-quantity
  provenance (measured / derived / estimated / procedural / user-modified) in the inspector.
* **N-body gravity**: 4th-order symplectic integrator with encounter substepping, collisions and mergers,
  optional 1PN relativity, rails moons, presets (Fast/Balanced/Accurate/Research/Custom), Kepler ↔ N-body;
  one-year integration matches JPL to ≤ 6 km for Venus–Neptune.
* **Editing**: object creator (planets, moons, giants, dwarf planets, asteroids, comets), editable inspector
  with units and scientific notation, star mass/age/metallicity, orbital elements, Δv pushes, delete,
  launch tool with live predicted trajectory and impact warning, undo/redo, warnings for unusual input.
* **Consequences**: orbit → insolation → climate → habitability → civilizations; impacts with crater,
  blast radius, impact winter, extinction and casualties; destroyed worlds end their civilizations.
* Overlays (osculating orbits, trails, predictions, velocity), calendar clock, command palette,
  seven curated "what if" scenarios built from ordinary edits; save format v2.
* Docs: GAP_ANALYSIS, SANDBOX_VISION, PHYSICS_ENGINE, ASTRONOMICAL_DATA, DATA_SOURCES, OBJECT_MODEL,
  SAVE_FORMAT, UI_SYSTEM, MILESTONE_S1.
### Changed
* One set of physical constants (IAU 2012/2015, CODATA 2018); insolation uses the annual-mean distance
  a(1−e²)^¼. Seeds replay slightly differently from 0.3 (golden fingerprint re-pinned); old saves load.
### Fixed
* `state_vectors_to_elements`: swapped arguments for the ascending node; equatorial orbits lost periapsis.

## 0.3.0 — Milestone 2 "Living worlds, rival nations" (2026-10-03)
### Added
* **Real Earth geography** (NOAA ETOPO5, public domain) with bicubic sampling; humanity starts in East Africa.
* **Rival nations** (`civ/polity.rs`): polities with territory, capitals, governments, relations, border wars,
  city captures, conquest, unions, secession, revolutions, world government; nuclear war between rivals.
* **Close-up terrain**: cube-sphere quadtree LOD of displaced, vertex-coloured patches (skirts, horizon culling,
  f64 placement), crater fields on airless worlds, path-length haze; terrain-aware camera.
* **Civilization level of detail**: per-civilization step sizes (1/10/100/1000 yr) with exact multi-year
  formulas; the civilization task sleeps when nothing is due.
* **Cross-platform determinism**: all simulation transcendental math via `libm`; golden-fingerprint test in CI.
* Windows cross-compilation from macOS; release workflow publishes a plain `.exe`, installer, portable zip, DMG.
* CLI `profile`; app flags `--latlon`, `--debug`.
### Changed
* Research pace recalibrated for the industrial era; crisis pressures scale with severity.
### Fixed
* Airless worlds (Moon, Mercury, most moons) were never lit and rendered black.
* Dry worlds measured height from a sentinel sea level and were drawn as ice; ice sheets now need water,
  frozen water covers realistic areas; no snow on peaks of waterless worlds.
* UI: opaque panels, markers occluded by planets, captures ignore user input.


## 0.2.0 — Milestone 1 "Living skeleton" (2026-10-03)
First end-to-end vertical slice, shipped as a double-clickable macOS app.

### Added
* `cosmogon_sim`: engine-independent, deterministic universe simulation — keyed RNG streams; multi-rate scheduler
  with CPU budget; stellar evolution; procedural neighbourhoods (IMF, binaries, frost-line architectures, moons,
  belts, rings); real Solar System as data; climate with greenhouse fit, carbonate–silicate cycle and glacial cycles;
  shared terrain and biomes; resources and deposits (fossil fuels made by life); multi-factor habitability; life
  stages with oxygenation and extinctions, simulated through prehistory; species; civilizations with knowledge,
  pressures, crises, collapse, settlements and networks; a 56-node causal technology graph; space age, radio
  detection and probes; history; versioned saves with migration hook.
* `cosmogon_cli`: `run`, `check` (determinism + save round trip), `survey`.
* `cosmogon` app (Bevy 0.18): f64 camera-relative rendering; astronomical camera; planet and atmosphere shaders;
  background texture baking re-run when worlds change; night-side city lights; procedural starfield; orbit lines and
  radio spheres; egui interface (menu with live showcase world, new-universe setup, save browser, settings,
  universe browser, chronicle, inspector, civilization dashboard with "why" explanations, timeline, toasts,
  developer overlay); autosave and quick save; GPU capture tool.
* Packaging: icon generator, `.app` + `.dmg` script, CI test and release workflows (Windows installer/zip, Linux).
* Documentation set.

### Changed
* `cosmogon_physics` no longer depends on an ECS.
* The Bevy prototype became the application crate `crates/cosmogon`; `big_space` replaced by an explicit f64 origin pass.

### Removed (moved to `legacy/`, not built)
* The hand-written wgpu renderer, its ECS and UI crates. See docs/ENGINE_ASSESSMENT.md.

### Fixed (in carried-over logic)
* Moons now orbit their planet's μ instead of the Sun's; time acceleration no longer applied twice;
  analytic orbits no longer fight an N-body integrator.

## 0.1.0 — Prototype (2026-07)
Original prototype; see docs/AUDIT.md.
- Paired habitat trajectories: native selectable stored-time response/contrast tables, signed discrete effect curves with unavailable gaps, protected all-time JSON/40-column CSV export and explicit CLI observation inspection. Loaded design identity stays separate from editable folder text. Original final analyses, simulation laws, saves and golden fingerprint remain unchanged.

### Recorded cabin balances (development)

* Shared retained endpoint stock/electricity/temperature/counter reports, all four independent paired ledgers, actual source/method context and protected CSV/JSON exports. Native cached views and immutable workers preserve recordings/live state; sparse sampled heat/power budgets are not extrapolated. No simulation law/save/dependency/golden change.

Selected cumulative cabin heat curves add eleven signed energy paths/storage, signed path/residual sums and executed/excluded time; protected native/CLI figures retain origin/coverage and four actual branch responses. Incomplete/different-origin comparisons stay unsupported. Existing recorded catalogs/analyses, physical laws, state/saves and golden unchanged.

Added separate source-pinned finite crop-supply feasibility studies: explicit liquid/CO₂/HNO₃-equivalent/lighting budgets, daily support brackets over unchanged maintained forecasts, retained source rounding residual, shared native/CLI numerical exports and verified frozen-context reopening. No physical crop or habitat stock credits, simulation/save change or golden re-pin.

Added separate conservative CHON crop material studies with pinned CIAAW nominal weights, original recipe/forecast retention, explicit inferred dry composition and oxygen differences, native element tables and strict protected JSON/CSV/source reopening. No simulation/save/golden/dependency/graphics change.
