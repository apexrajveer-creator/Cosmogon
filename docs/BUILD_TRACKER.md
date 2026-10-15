# Cosmogon build tracker

Live checklist of integrated builds for the owner. ✅ = done, verified and committed; ⬜ = remaining.
Updated **in the same commit as each build**. Numbering: entries 1–45 are the accepted history
([MASTER_GAP_ANALYSIS](MASTER_GAP_ANALYSIS.md) is the evidence record for each). Entries 47+ are the
forward plan; **47–51 are firm** (accepted dependency sequence in
[VEHICLE_SUPPORT_NEXT](VEHICLE_SUPPORT_NEXT.md) and
[CONTINUATION_HANDOFF](CONTINUATION_HANDOFF.md) → Recommended next work), later numbers are
provisional groupings that may renumber when a build starts.

---

## Done — 47 builds

| # | Build | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Performance foundation | ✅ | gap 1, VALIDATION.md |
| 2 | Camera navigation | ✅ | gap 2, NAVIGATION.md |
| 3 | Photo foundation | ✅ | gap 3 |
| 4 | Research truth and controls | ✅ | gap 4, RESEARCH.md |
| 5 | Notification/audio foundation | ✅ | gap 5, NOTIFICATION_SYSTEM.md, AUDIO_SYSTEM.md |
| 6 | Habitat engineering (finite stores/power/heat) | ✅ | gap 6, HABITAT_ENGINEERING.md |
| 7 | Greenhouse planning | ✅ | gap 7, GREENHOUSE_PLANNING.md |
| 8 | Finite oxygen production (electrolysis) | ✅ | gap 8, HABITAT_ELECTROLYSIS.md |
| 9 | Finite CO₂ reduction (Sabatier) | ✅ | gap 9, HABITAT_SABATIER.md |
| 10 | Habitat recording graphs | ✅ | gap 10, HABITAT_RECORDINGS.md |
| 11 | Controlled habitat comparisons | ✅ | gap 11, HABITAT_COMPARISONS.md |
| 12 | Habitat equipment sound | ✅ | gap 12, HABITAT_AUDIO.md |
| 13 | Habitat WHY explanations | ✅ | gap 13, HABITAT_DIAGNOSTICS.md |
| 14 | Authored camera paths | ✅ | gap 14, CINEMATIC_PATHS.md |
| 15 | Native keyboard controls | ✅ | gap 15 |
| 16 | Controlled transpiration planning | ✅ | gap 16 |
| 17 | Finite cabin humidity | ✅ | gap 17 |
| 18 | Controlled moisture equipment studies | ✅ | gap 18 |
| 19 | Paired habitat experiments | ✅ | gap 19 |
| 20 | Paired trajectory analysis | ✅ | gap 20 |
| 21 | Recorded design replay | ✅ | gap 21 |
| 22 | Recorded graph navigation | ✅ | gap 22 |
| 23 | Finite cabin gas loss | ✅ | gap 23 |
| 24 | Recorded pressure explanations | ✅ | gap 24 |
| 25 | Controlled geometry and thermal studies | ✅ | gap 25 |
| 26 | Recorded research figures | ✅ | gap 26 |
| 27 | Recorded habitat balances | ✅ | gap 27 |
| 28 | Cumulative cabin heat accounting | ✅ | gap 28 |
| 29 | Selected cumulative heat trajectories | ✅ | gap 29 |
| 30 | Finite crop-supply studies | ✅ | gap 30 |
| 31 | Conservative crop material chemistry | ✅ | gap 31 |
| 32 | Finite installed crop process | ✅ | gap 32 |
| 33 | Recorded actual crop trajectories | ✅ | gap 33 |
| 34 | Controlled installed crop studies | ✅ | gap 34 |
| 35 | Saved single-setting habitat designs | ✅ | gap 35 |
| 36 | Explicit frozen habitat study origins | ✅ | gap 36 |
| 37 | Finite supply-vehicle design and propulsion | ✅ | gap 37 |
| 38 | Physical finite vehicle flight | ✅ | gap 38 |
| 39 | Surface-relative arrival | ✅ | gap 39, LANDING_AND_DELIVERY_RESEARCH.md |
| 40 | Terrain contact and finite landing response | ✅ | gap 40, VEHICLE_CONTACT.md (492 tests) |
| 41 | Persistent foot shear | ✅ | gap 41, VEHICLE_STATIC_SHEAR.md (505 tests) |
| 42 | Actual footprint and ground-hold evidence | ✅ | gap 42, VEHICLE_FOOTPRINT_SUPPORT.md (514 tests) |
| 43 | Cargo mass epochs research workflow | ✅ | gap 43, CARGO_MASS_EPOCHS.md, `5943ead` (532 tests) |
| 44 | Solver-integrated surface-operation window | ✅ | gap 44, VEHICLE_OPERATION.md (539 tests) |
| 45 | Cargo mass in authoritative world flight | ✅ | gap 45, VEHICLE_CARGO_FLIGHT.md, `c03f9d3` (548 tests) |
| 46 | Finite handling equipment and recipient storage capacity | ✅ | gap 46, VEHICLE_HANDLING_CAPACITY.md (562 tests), `24eef17` |
| 47 | Atomic co-located habitat transfer | ✅ | gap 47, VEHICLE_TRANSFER.md (575 tests) |

Released baselines: v0.3.0, v0.5.0, **v1.0.0** (`0411e9d`) — all preserved; tags untouched.

---

## Remaining — 48 onward

### A. Supply & landing chain (firm order — the owner's active priority)

| # | Build | Status |
| --- | --- | --- |
| 47 | ~~Atomic co-located habitat transfer: species `Edit`s, operation-window gate, conservation both sides, undo/save/replay, CLI/native agreement — the first real delivery~~ | ✅ **DONE** |
| 48 | Authored finite launch from a surface (engine, structural/thermal limits, trajectory evidence) | ⬜ **NEXT** |
| 49 | Orbital flight, transfer and rendezvous/approach | ⬜ |
| 50 | Entry, heating/thermal limits and terminal landing completion | ⬜ |
| 51 | End-to-end supply-mission closeout: one scripted launch → orbit → transfer → landing → delivery scenario with full evidence | ⬜ |

### B. Settlement realism (phase 9 remainder) — provisional, ~6 builds

| # | Build | Status |
| --- | --- | --- |
| 52 | Physical greenhouse installation + finite resource exchange with habitat stocks | ⬜ |
| 53 | Physical humidity/leak/wall-condensation coupling | ⬜ |
| 54 | Equipment calibration and maintenance model | ⬜ |
| 55 | Finite settlement construction and logistics | ⬜ |
| 56 | Life-support validation beyond reference assumptions (calibration/uncertainty) | ⬜ |
| 57 | Environmental hazard completeness (breach/radiation propagation) | ⬜ |

### C. Research & scientific validation — provisional, ~3 builds

| # | Build | Status |
| --- | --- | --- |
| 58 | Uncertainty and controlled-comparison framework across research panes | ⬜ |
| 59 | Independent scientific validation campaign (analytic/JPL cross-checks extended) | ⬜ |
| 60 | Student→researcher UX completeness (guided flows, provenance surfacing) | ⬜ |

### D. Consequences, history & civilizations — provisional, ~5 builds

| # | Build | Status |
| --- | --- | --- |
| 61 | Validated climate/consequence propagation between systems | ⬜ |
| 62 | Recorded history and Chronicle completeness | ⬜ |
| 63 | Visible civilization infrastructure and economic/spatial presence | ⬜ |
| 64 | Intervention consequences end-to-end with evidence | ⬜ |
| 65 | Interstellar consequence and contact modeling | ⬜ |

### E. Universe & data pipeline — provisional, ~9 builds

| # | Build | Status |
| --- | --- | --- |
| 66 | Real catalogue streaming (Gaia nearby stars) with provenance | ⬜ |
| 67 | Known exoplanet systems | ⬜ |
| 68 | Real nebulae/galaxy/deep-sky datasets | ⬜ |
| 69 | Dataset registry, licensing and citation audit | ⬜ |
| 70 | Per-model exotic validation (black holes, compact objects) | ⬜ |
| 71 | Galaxy-scale dynamics and collisions | ⬜ |
| 72 | Cosmology/large-scale structure presentation | ⬜ |
| 73 | Procedural-body seeding with honest provenance labels | ⬜ |
| 74 | Data pipeline performance and memory budgets | ⬜ |

### F. Owner graphics goal supplement (queued by owner; ~8–12 builds) — see [GRAPHICS_GOAL_SUPPLEMENT](GRAPHICS_GOAL_SUPPLEMENT.md)

| # | Build | Status |
| --- | --- | --- |
| 75 | Rendering architecture/LOD/GPU budget foundation from capture measurements | ⬜ |
| 76 | Star close-up activity (surface/atmosphere) to reference richness | ⬜ |
| 77 | Planet terrain close-ups: mountains, water, erosion detail | ⬜ |
| 78 | Atmospheres and weather: clouds, storms, hurricanes where applicable | ⬜ |
| 79 | Geological activity: volcanism, earthquakes and visible consequences | ⬜ |
| 80 | Civilizations and buildings visible from close range | ⬜ |
| 81 | Moons, rings, small bodies and encounter detail | ⬜ |
| 82 | Cross-scale coherence, streaming and measured performance passes | ⬜ |

### G. Product polish, platform & QA — provisional, ~6 builds

| # | Build | Status |
| --- | --- | --- |
| 83 | Workspace/persistence coherence and notification-audio polish | ⬜ |
| 84 | Physical cabin acoustics/weather sound | ⬜ |
| 85 | Accessibility audit and fixes (input, contrast, motion, AT) | ⬜ |
| 86 | Performance QA pass at scale (p50/p95 budgets, regression guards) | ⬜ |
| 87 | Video capture, DOF and cinematic-completion features | ⬜ |
| 88 | Distribution: signed installers, update check, localization scaffolding | ⬜ |

---

**Totals:** 47 done / ~42 remaining (≈89 builds total; remaining ≈240–340 focused hours, graphics
the dominant wildcard). Cluster sizes in sections B–G are estimates and renumber as builds land;
section A never renumbers. Update this file in the same commit as each build, and append the new
gap entry to [MASTER_GAP_ANALYSIS](MASTER_GAP_ANALYSIS.md) when the build is accepted.
