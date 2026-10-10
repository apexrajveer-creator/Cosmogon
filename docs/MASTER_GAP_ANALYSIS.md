# Master directive gap analysis

This is the active implementation order for the [master directive](MASTER_DEVELOPMENT_DIRECTIVE.md).
The old `GAP_ANALYSIS.md` describes the pre-sandbox baseline and is historical.

| Directive phases | Existing foundations | Remaining acceptance gates |
|---|---|---|
| 0 Audit | Release 1.0, current audit, baseline tests, native captures, checkpoint | Full GPU/VRAM profiling on supported hardware/tools; Windows manual QA |
| 1 Native foundation | Deterministic engine-independent sim, jobs, saves, logging, embedded assets | Resource/data-pack manager, measured budgets; performance report slice implemented here |
| 2 Application shell | Home, Continue, sandbox library, reference clone, settings, autosave | Coherent workspace persistence/docking, accessible navigation audit |
| 3 Camera + UX | Orbit/track, free flight, speed/inertia/sensitivity, object-relative and geographic surface viewpoints, authored timed camera paths, physical-key customization, command palette, inspector | Mouse/controller/chord rebinding and accessibility verification, advanced cinematic paths/video/time synchronization, complete reference-frame precision; notification foundation is present in phase 5 |
| 4 Photo | Clean native PNG, FOV/exposure/warmth/saturation, separate preferences, camera/dataset sidecars | DOF, advanced grading, high-resolution/tiled export, advanced camera paths/video |
| 5 Audio/alerts | Realistic/cinematic/hybrid modes; music/effects/alert/equipment mix; live habitat interior sonification; inbox, routing, quiet settings and optional pause | Physical environment/interior/weather audio, custom sounds, durable read marks and new event producers |
| 6 Solar sandbox | Horizons states, N-body, edits, throw, launch, predictions, undo | Shared parameter/model registry, source context and observer export built; broader control/regime validation remains |
| 7–8 Mars rendering/data | Quadtree surfaces, atmosphere; Earth relief pipeline; pinned NASA MOLA 4/16ppd overview heights/counts with explicit undoable source selection and version checks, source-cell probes, four historical mission references, tracked/snapshot surface travel and source-supported numeric terrain sections with native definition replay and sampled geographic traces | Higher-resolution licensed patch delivery/LOD, registered mission imagery/tracks, finer terrain analysis and broader observed/generated layers |
| 9 Settlement | Civilization sites/resources plus authoritative user-created geographic habitats, finite mass/power/battery/air/water/heat budgets, native controls/markers, optional finite water electrolysis/CO₂ reduction, cabin vapor/condensate, optional finite outward open-orifice breach and separated product storage, controlled source-based crop biomass/transpiration planner plus optional finite installed reference crop transfers/harvest, model/source metadata, actual protected observer exports, reopened habitat recordings/graphs and design-alert causes | Calibrated crop physiology, chamber HVAC, repeated planting/diet coupling, further reaction/phase/hardware fidelity, seal/rarefied/inward flow and EVA, radiation, structural/site engineering, equipment calibration/reliability, resources/construction/maintenance/logistics, resolved meshes and calibrated cabin acoustics |
| 10 Supply chain | Abstract missions/travel plus finite authored vehicle/cargo/propulsion, actual system-local gravity/burn flight, swept spherical contact, rotating geographic approach, finite terrain/rigid-body contact and exact complete world/approach recordings | Footprint stability/attachment, physical transfer, launch, entry/landing, hardware/power/thermal constraints and physical cargo transfer to the same settlement state |
| 11 Consequences | Orbit → environment → life/civ plus impact history; shared completed-interval habitat WHY | Validated transient climate/regional effects and universal historical cause graph/WHY |
| 12 Civilization visuals | Lights, overlays, procedural land-use tint | Infrastructure geometry, visible roads/industry/ports/traffic driven by authoritative state |
| 13 Real universe | Embedded Horizons and ingestion script | Catalogue registry, spatial index, streaming, dataset manager and rights review |
| 14 Exotic universe | Stellar evolution, compact-object visuals, nurseries | Per-model validation and advanced effects; no wholesale reimplementation |
| 15 Interstellar | Reduced generation ships, terraforming, Dyson swarm | Richer shared consequences, branching civilizations and communication models |
| 16 Research | Typed model/source/parameter registry, native controls/interactive plots, observer CSV/initial-state bundles, frozen definitions/replay/sweeps, cross-object measurements, computed analytic/JPL reports, saved graph/favorite choices, reopened recordings, synchronized trajectory comparison and guided explanations | Broader comparison workflows, uncertainty propagation and independent model validation |
| 17–19 Polish/product | Native distribution pipeline and broad usable app | Cinematics/audio/accessibility/control completeness, signing, dataset delivery, product QA |

## Immediate sequence

1. **Performance foundation (built):** bounded samples; frame p50/p95/p99; simulation/load
   timing; optional supported process/render snapshots; panel and versioned reports.
2. **Camera navigation (built, partial phase):** f64 detached pose, six-axis motion, roll,
   adaptive/manual speed, inertia, sensitivity/inversion, orbit return and persistent viewpoints.
   Terrain follows nearby bodies. Mouse/controller/chord customization, swept collision and advanced cinematics remain.
3. **Photo foundation (built, partial phase):** presentation-only lens/colour settings, separate
   persistence, clean PNG action and metadata sidecars. No DOF or tiled/high-resolution output yet.
4. **Research truth and controls (built, partial phase):** typed model/source registries, provenance repair and consistent
   parameter descriptors before wide editing UI. Add only models backed by sources/validation.
5. **Notification/audio foundation (built, partial phase):** persisted category routing, thresholds,
   quiet options, searchable recent inbox and optional pauses; mode/channel mixer with realistic
   external silence. Physical cabin acoustics/weather sound remain unimplemented; habitat design-alert producers now feed the existing routing.

6. **Habitat engineering (built, partial phase 9):** bounded reference crew-demand exchanges and explicit finite power/mass/heat state on solid-world coordinates. Preserve ordinary edits/undo and exact worlds; native controls, source/model limits and protected trajectory/snapshot exports are connected. See [HABITAT_ENGINEERING](HABITAT_ENGINEERING.md) and [HABITAT_RESEARCH](HABITAT_RESEARCH.md). Greenhouse/reactions and resource/logistics mechanisms come next.

Phase numbers express dependencies, not permission to discard useful later-phase systems.
New work must meet the relevant definitions of done; partial implementations remain marked partial.

7. **Greenhouse planning (built, partial phase 9):** source-based controlled lettuce forecasts, assumptions and lighting demands next to actual habitat context. Physical installation/finite resource exchange still follows; see [GREENHOUSE_PLANNING](GREENHOUSE_PLANNING.md).

8. **Finite oxygen production (built, partial phase 9):** optional source-based water electrolysis uses finite water, allocated bus power and both product capacities, with a reaction enthalpy debit in the shared heat budget. See [HABITAT_ELECTROLYSIS](HABITAT_ELECTROLYSIS.md). Physical greenhouse coupling and the wider settlement remain incomplete.

9. **Finite CO₂ reduction (built, partial phase 9):** optional Sabatier processing consumes captured CO₂ and stored H₂, keeps finite methane/raw-water products, processes held raw water later with existing powered recycling, and adds reference heat. See [HABITAT_SABATIER](HABITAT_SABATIER.md). No unlimited disposal, instantaneous recycling or closed-loop recovery guarantee.

10. **Habitat recording graphs (built, partial phase 9/16):** bounded shared source/endpoint/CSV consistency reader, independent native observation curves/selected rows and CLI inspection. See [HABITAT_RECORDINGS](HABITAT_RECORDINGS.md). No field validation or unimplemented physical greenhouse claim.

11. **Controlled habitat comparisons (built, partial phase 9/16):** ordinary-edit single-setting branches, exact common base and every run, worker-driven native experiments and shared stored-time/interval-supported differences. See [HABITAT_COMPARISONS](HABITAT_COMPARISONS.md). No interaction analysis, field calibration or global optimization claim.

12. **Habitat equipment sound (built, partial phase 5/9):** explicit live interior listening context, completed-interval power-driven textures, separate mixer and protected WAV/source exports. See [HABITAT_AUDIO](HABITAT_AUDIO.md). No physical noise levels, fault model, acoustic propagation or complete interior claim.

13. **Habitat WHY (built, partial phase 9/11/16):** shared live/recorded completed-interval explanations with actual power/water/heat/air terms, recorded constraints, units/formulas and protected source exports. See [HABITAT_DIAGNOSTICS](HABITAT_DIAGNOSTICS.md). No unique historical attribution, physical fault model, field calibration or full cause graph.

14. **Authored camera paths (built, partial phase 2/17):** shared f64 timed poses, orientation/look-at, lens interpolation, native authoring, explicit playback/scrub and protected definition files. See [CINEMATIC_PATHS](CINEMATIC_PATHS.md). Video, DOF, spline/arc-length control and simulation-rate synchronization remain incomplete.

15. **Native keyboard controls (built, partial phase 3/17):** shared physical-key bindings for camera and ordinary HUD actions, conflicts/cancellation, explicit persistence, protected files and dynamic help. Mouse/controller/chord rebinding and full accessibility verification remain.

16. **Controlled transpiration planning (built, partial phase 9/16):** shared source-pinned vapor/leaf conductance and water forecasts, explicit humidity assumptions, deterministic integration, native plots and numeric exports. Physical greenhouse coupling, humidity dynamics and facility equipment remain incomplete.

17. **Finite cabin humidity (built, partial phase 9):** optional finite vapor and isolated condensate, explicit phase heat/electricity, supported source-based saturation and downstream recycling. Physical greenhouse coupling, leaks, ice/wall condensation and equipment calibration remain incomplete.

18. **Controlled moisture equipment studies (built, partial phase 9/16):** typed one-setting sweeps of airborne partition, humidity setpoint and finite collector assumptions from the same retained world, actual time-supported phase/water comparisons and protected source-pinned exports. No calibration, optimal humidity or global optimization claim.

19. **Paired habitat experiments (built, partial phase 9/16):** two settings at two levels, all four exact common-state branches, actual conditional effects and interaction contrasts. Keep single-setting studies and make unavailable interval/source support explicit; no statistical-noise, global-optimization or field-calibration claim.

20. **Paired trajectory analysis (built, partial phase 9/16):** actual four-response/contrast curves and selectable stored-time tables with protected numeric exports, explicit gaps and interval support. Preserve completed final reports and ordinary studies; no interpolation, field-calibration or statistical-confidence claim.

21. **Recorded design replay (built, partial phase 9/16):** explicit independent recomputation of four source-pinned branches at every recorded time, exact full-sample/endpoint comparisons, native worker progress/cancellation and protected mismatch reports. Reproducibility is separate from empirical/hardware validation; keep original observations read-only.

22. **Recorded graph navigation (built, partial phase 16):** explicit time-window zoom/pan, truthful linear/log display support, actual point cursors and separately retained display layouts without altering numerical observations/exports. Apply to ordinary and paired habitat recordings; broader research graphs and accessibility remain incomplete.

23. **Finite cabin gas loss (built, partial phase 9):** source-based pressure-driven outward gas flow, finite cabin stocks and explicit escaped mass/heat budgets, ordinary edits, native/recorded diagnostics and protected studies. Establish supported assumptions and integration/conservation evidence before extending the physical model; no calibrated hardware or safety claim.

24. **Recorded pressure explanations (built, partial phase 9/11/16):** actual component partial pressures and explicit ordered inventory/temperature/volume accounting between retained observations, without claiming unique causal attribution or reconstructing missing history. Protected numerical evidence and native explanations retain scope/source identity.

25. **Controlled geometry and thermal studies (built, partial phase 9/16):** typed cabin volume, effective heat capacity, envelope conductance and radiator area assumptions from the same actual initial stores/temperature, plan-dependent pressure comparisons and exact independent replay. Distinguish counterfactual geometry/capacity from physical expansion or equal initial energy; no equipment calibration or automatic scaling.

26. **Recorded research figures (built, partial phase 16):** protected vector diagrams backed by exact original numbers, support/scale omissions, display definitions, full retained observations/source identity and paired method context. Share native/CLI plotting without smoothing or invented uncertainty. Improve small-signal range/tick clarity while preserving raw results.

27. **Recorded habitat balances (built, partial phase 9/16):** exact endpoint stocks and retained cumulative electrical/crew counters, mass and electrical closure, all four independent paired branches, protected shared native/CLI exports. Do not integrate sparse sampled minute budgets into a complete history.

28. **Cumulative cabin heat accounting (built, partial phase 9/16):** retain actual signed heat transfers on every completed modeled interval, explicit coverage/legacy availability and shared native/CLI span explanations. Preserve existing thermal physics and expose residuals; never reconstruct a complete ledger from sparse minute samples.

## Owner graphics goal supplement (2026-10-09)

[Detailed graphics requirement](GRAPHICS_GOAL_SUPPLEMENT.md) is part of the continuing product goal. Preserve the original master document and add this requirement through the existing rendering/planet/civilization phases. Every visitable cosmic body needs rich animated close-up detail in Cosmogon's style; procedural activity is permitted, with honest data provenance. **Queued: cabin/research builds continue first**, as explicitly directed by the owner.

29. **Selected cumulative heat trajectories (built, partial phase 9/16):** actual retained signed heat/storage curves, explicit execution coverage/origins, supported four-branch contrasts and protected shared native/CLI research figures. Preserve immutable existing metric catalogs and final analyses.

30. **Finite crop-supply studies (built, partial phase 9/16):** explicit finite liquid/CO₂/nutrient-equivalent/lighting budgets, source-based recipe projections and daily support brackets over unchanged maintained-regime forecasts. Preserve legacy definitions and separate planning from physical greenhouse stocks/stress.

31. **Conservative crop material chemistry (built, partial phase 9/16):** separately pinned CHON surrogate material balances, explicit published-versus-derived oxygen and complete elemental residuals before any physical greenhouse stock transfer. Preserve original forecasts, finite supply studies and context; no measured tissue or full nutrient chemistry claim.

32. **Finite installed crop process (built, partial phase 9):** actual optional crop state, finite material/power allocation, explicit chemical/phase/thermal exchanges and harvest stores, ordinary native controls/recordings and conservative source-pinned replay. Preserve absent legacy state; do not credit unconstrained source forecasts or imply calibrated crop physiology.

33. **Recorded actual crop trajectories (built, partial phase 9/16):** explicitly selected retained progress, stocks, cumulative material/electric/storage graphs, support-aware comparisons and protected complete research figures. Preserve immutable saved catalogs, actual source context and absent legacy evidence; do not imply calibrated biology or reconstruct sparse operating history.

34. **Controlled installed crop studies (built, partial phase 9/16):** typed ideal chamber acknowledgement, auxiliary electricity and finite dry/shared oxygen capacity studies from original installed state, with exact common-state branches, plan-dependent actual crop response catalogs, strict no-grant/no-discard preflight and protected source-pinned results. Preserve old catalogs and distinguish engineering assumptions from crop physiology or calibrated chamber hardware.

35. **Saved single-setting habitat designs (built, partial phase 9/16):** reopen complete common-state sweep identity and every actual branch, restore typed rerun controls independently of completed inputs, select retained branches and independently replay every observation with protected evidence. Preserve old paired formats/catalogs, fixed-state edits and source identity; wrap long setting descriptions. No empirical calibration or uncertainty claim.

36. **Explicit frozen habitat study origins (built, partial phase 9/16):** reuse the actual pinned original common world or explicitly choose the current live world for a new typed experiment, with retained origin context, request-time validation and protected fresh output. Preserve completed results and saved catalogs; distinguish restoring controls from rewinding/reusing an original world. Finish this reproducibility control, then reassess physical settlement/supply gaps against the master directive.

37. **Finite supply-vehicle design and propulsion (built, partial phase 9/10/16):** explicit dry/propellant/tank/payload mass, typed actual consumable cargo inventories, source-based finite thrust/Isp burn bookkeeping and protected design evidence. Build the shared vehicle mechanism before world-flight and physical delivery; distinguish ideal free-space capability from trajectory/launch/landing feasibility. Connect later transport to the same finite settlement state rather than granting resources on a timer. Preserve deterministic absent legacy worlds, source honesty and existing cabin studies; no graphics work in this build.


38. **Physical finite vehicle flight (built, partial phase 10):** connect retained propellant/cargo and explicit f64 position/velocity to existing world reference frames and gravitational state, with bounded deterministic dynamics and supported contact boundaries. Native and headless evidence must distinguish applied finite burns, gravitational evolution and ideal capability. Keep existing design recordings exact and require physically valid co-location before future settlement transfer; no automatic supply grants or graphics build.

39. **Surface-relative arrival (built; landing/delivery remain incomplete, partial phase 10):** expose actual rotating target-frame position/velocity, geometric/source support and explicit finite contact-response assumptions before claiming touchdown. Continue toward supported contact/attachment and co-located atomic transfer from retained craft cargo into the exact habitat stocks, with capacities, no grants/discards, ordinary undo/save/replay and honest hardware limits. Preserve existing versioned flight records and absent legacy state; graphics remain queued. See [research memo](LANDING_AND_DELIVERY_RESEARCH.md).

40. **Terrain contact and finite landing response (built; static support/delivery remain incomplete, partial phase 10):** evolve motion against the modeled rotating terrain with explicit finite stroke/force/damping/friction and structural support, retained energy/work/failure evidence and a separately versioned flight capability. Do not convert old spherical impact records or declare attachment from low speed alone. Preserve exact prior recordings and absent legacy state; physical co-located delivery follows supported contact, with no grants or timer completion. Graphics remain queued.

Accepted smooth-contact mechanism: [guide](VEHICLE_CONTACT.md), [design](VEHICLE_CONTACT_DESIGN.md), [verification](VEHICLE_CONTACT_VALIDATION.md), checkpoint `1c0c6e0`. Its final evidence is **492 release tests plus one existing ignored**, six release/package checks, 57 CLI/native cases, six inspected captures and a 48.261886-second live run. The subsequent persistent-shear integration is separately versioned below. Footprint stability/attachment, safe landing and cargo transfer remain incomplete; graphics are queued.

41. **Static foot support and finite surface delivery (in progress, partial phase 9/10):** verified component checkpoint `ea52bfd` supplies finite shear/history/work and actual rotating-frame transport. The active [version-3 integration](VEHICLE_STATIC_SHEAR.md) now evolves that history in actual motion, strict saved/replayed exports and native/CLI controls; all **505 release tests plus one existing ignored pass**, with six release/package checks, 67 CLI/native cases, six inspected captures and a 48.448818-second live run. [Final evidence and numerical limits](VEHICLE_STATIC_SHEAR_VALIDATION.md). Establish explicit footprint/load/stability and supported surface operations, then retained-cargo transfer to the same co-located habitat stores with authored capacities and mass/momentum/energy accounting. Preserve exact prior contact records and all absent legacy state; no timer, low-speed attachment shortcut or automatic cargo credit. [Mechanism and inventory constraints](STATIC_FOOT_SUPPORT_DESIGN.md). Graphics stay queued.
