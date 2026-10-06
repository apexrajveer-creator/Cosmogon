# Master directive gap analysis

This is the active implementation order for the [master directive](MASTER_DEVELOPMENT_DIRECTIVE.md).
The old `GAP_ANALYSIS.md` describes the pre-sandbox baseline and is historical.

| Directive phases | Existing foundations | Remaining acceptance gates |
|---|---|---|
| 0 Audit | Release 1.0, current audit, baseline tests, native captures, checkpoint | Full GPU/VRAM profiling on supported hardware/tools; Windows manual QA |
| 1 Native foundation | Deterministic engine-independent sim, jobs, saves, logging, embedded assets | Resource/data-pack manager, measured budgets; performance report slice implemented here |
| 2 Application shell | Home, Continue, sandbox library, reference clone, settings, autosave | Coherent workspace persistence/docking, accessible navigation audit |
| 3 Camera + UX | Orbit/track, free flight, speed/inertia/sensitivity, object-relative viewpoints, command palette, inspector | Key rebinding, cinematic paths, complete reference-frame precision; notification foundation is present in phase 5 |
| 4 Photo | Clean native PNG, FOV/exposure/warmth/saturation, separate preferences, camera/dataset sidecars | DOF, advanced grading, high-resolution/tiled export, camera paths/video |
| 5 Audio/alerts | Realistic/cinematic/hybrid modes; music/effects/alert mix; inbox, routing, quiet settings and optional pause | Physical environment/interior/weather audio, custom sounds, durable read marks and new event producers |
| 6 Solar sandbox | Horizons states, N-body, edits, throw, launch, predictions, undo | Model-registry and research control completeness; validation of additional regimes |
| 7–8 Mars rendering/data | Quadtree surfaces, atmosphere; Earth relief pipeline | Licensed Mars terrain ingest/versioning/LOD, mission sites, surface travel and observed/generated labels |
| 9 Settlement | Civilization sites/resources | Actual engineering state/balances, habitat/power/water/air/thermal, hazards and explainable causes |
| 10 Supply chain | Abstract missions and travel | Vehicle design and dynamics, cargo, entry/landing, integration with the same settlement state |
| 11 Consequences | Orbit → environment → life/civ plus impact history | Validated transient climate/regional effects and explicit cause graph/WHY |
| 12 Civilization visuals | Lights, overlays, procedural land-use tint | Infrastructure geometry, visible roads/industry/ports/traffic driven by authoritative state |
| 13 Real universe | Embedded Horizons and ingestion script | Catalogue registry, spatial index, streaming, dataset manager and rights review |
| 14 Exotic universe | Stellar evolution, compact-object visuals, nurseries | Per-model validation and advanced effects; no wholesale reimplementation |
| 15 Interstellar | Reduced generation ships, terraforming, Dyson swarm | Richer shared consequences, branching civilizations and communication models |
| 16 Research | CLI, diagnostics and some graphs | PhysicsModelRegistry, uncertainty, experiments/sweeps, trajectories/CSV/manifests |
| 17–19 Polish/product | Native distribution pipeline and broad usable app | Cinematics/audio/accessibility/control completeness, signing, dataset delivery, product QA |

## Immediate sequence

1. **Performance foundation (built):** bounded samples; frame p50/p95/p99; simulation/load
   timing; optional supported process/render snapshots; panel and versioned reports.
2. **Camera navigation (built, partial phase):** f64 detached pose, six-axis motion, roll,
   adaptive/manual speed, inertia, sensitivity/inversion, orbit return and persistent viewpoints.
   Terrain follows nearby bodies. Input rebinding, swept collision and cinematic paths remain.
3. **Photo foundation (built, partial phase):** presentation-only lens/colour settings, separate
   persistence, clean PNG action and metadata sidecars. No DOF or tiled/high-resolution output yet.
4. **Research truth and controls:** typed model/source registries, provenance repair and consistent
   parameter descriptors before wide editing UI. Add only models backed by sources/validation.
5. **Notification/audio foundation (built, partial phase):** persisted category routing, thresholds,
   quiet options, searchable recent inbox and optional pauses; mode/channel mixer with realistic
   external silence. Environment models and future hazard producers remain unimplemented.

Phase numbers express dependencies, not permission to discard useful later-phase systems.
New work must meet the relevant definitions of done; partial implementations remain marked partial.
