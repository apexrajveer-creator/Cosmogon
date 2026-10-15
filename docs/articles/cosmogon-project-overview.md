# Cosmogon: A Reproducible Living-Universe Sandbox

**Author:** Ashwin Dandotiya
**Project:** Cosmogon  
**Document type:** Project and software overview (not a peer-reviewed research paper)  
**Version:** Draft  
**Date:** 15 October 2026

**Release context:** Cosmogon 1.0.0 was released on 4 October 2026. The newer vehicle and research work described here is post-release development on the public `main` branch, described at commit `9eb830d` (15 October 2026); it is not part of the 1.0.0 release.

> Before public release, verify every linked source and version-specific claim, decide the AI-assistance disclosure, and export a versioned copy alongside the code release. This article describes the software and its engineering evidence; it does not claim that all of its scientific models are validated research instruments.

## Abstract

Cosmogon is a native, open-source sandbox for exploring how physical and biological processes can connect across astronomical scales. It combines a three-dimensional universe viewer with a deterministic simulation core, allowing users to inspect or alter planetary systems, stars, climates, life, and civilizations. A separate research workflow exposes typed inputs, model and data provenance, controlled comparisons, recorded trajectories, and reproducible numeric exports. The project includes real-data pathways such as NASA/JPL Solar System states and NASA Mars Orbiter Laser Altimeter topography, alongside explicitly generated and simplified models. This overview describes the software architecture, current research-oriented workflows, verification evidence, and limits. Cosmogon is an actively developed exploratory simulator: deterministic execution, regression tests, and selected benchmark comparisons support reproducibility, but do not establish empirical accuracy across its full range of physical, biological, or societal models.

**Keywords:** scientific visualization; simulation software; reproducibility; planetary science; space exploration; open-source software

## 1. Project and motivation

Cosmogon is built around a simple interaction: begin with a universe, inspect how it is represented, change a condition, and follow the consequences through the simulation. A user can start with the Solar System, view planetary and stellar systems at different scales, construct scenarios, and inspect how modeled climate, habitability, life, and civilization respond over time. The desktop application is written in Rust and uses Bevy for rendering and interaction; simulation and physics crates are designed to run independently of the graphical application. The workspace declares MIT OR Apache-2.0 licensing; the repository audit notes that root license files still need to be reconciled before public release.

The project serves several levels of use. A newcomer can follow guided scenarios and observe a change such as a different orbit. An advanced user can edit parameters, save a sandbox, or compare controlled branches. A researcher can inspect model definitions and source context, replay a saved study, and export the actual numeric observations. These modes share one authoritative simulation state, while the research interface makes assumptions and evidence more explicit.

Cosmogon is both an educational visualization and an engineering experiment in connecting many domains in one persistent sandbox. Its breadth is also a limitation: the presence of a domain in the software does not mean that domain is complete or calibrated for prediction.

## 2. Software design

The active Cargo workspace separates core numerical utilities, orbital physics, the simulation, a command-line research runner, and the native application. The simulation uses deterministic math helpers and keyed random streams. Its regression suite includes a golden state fingerprint intended to detect unintended changes to authoritative simulation state. Deliberate state-model changes require an explicitly reviewed fingerprint update and corresponding change record.

This separation allows experiments to use the same simulation mechanisms without relying on a rendered frame or a particular user-interface path. Research definitions can pin their model and data context; completed results can be reopened and inspected through native or command-line tools. Numeric exports are intended to preserve actual computed values and identify unavailable observations rather than fill gaps with inferred values.

The rendering layer has a different purpose from the scientific model. It supports scale-spanning exploration and communicates object identity and activity. Some views use observational inputs; other surface detail and activity are generated or illustrative. Visual richness should therefore not be interpreted as measurement resolution or physical validation.

## 3. Data and model categories

The project distinguishes measured, derived, estimated, procedural, and user-modified information in its user-facing research workflows. Examples of data pathways documented in the repository include NASA/JPL Horizons Solar System states and NASA Planetary Data System MOLA topography for Mars. Dataset versions, source identifiers, checksums, frames, coverage, and known registration limits are recorded where the workflow supports them. See the project’s [data-source register](../DATA_SOURCES.md) for source-specific details.

The distinction between source data and simulation is important. Imported orbital states or terrain samples do not make all subsequent model outputs observational. Climate, biological, civilization, and many engineering processes use reduced models, idealizations, heuristics, or user assumptions. The project’s [scientific validation notes](../SCIENTIFIC_VALIDATION.md) describe particular numerical checks and their bounded scope; they do not validate every subsystem.

## 4. Research-oriented workflows

Cosmogon’s research tools are intended to make a study inspectable and repeatable within the software. Depending on the model, workflows include typed parameter definitions, frozen starting states, controlled single- or paired-setting comparisons, stored trajectories, source-pinned inputs, deterministic replay, and CSV or JSON exports. Habitat and vehicle engineering work adds explicit inventories, budgets, and accounting ledgers to selected workflows. Their inclusion provides traceability and supports model interrogation; it does not by itself establish that the underlying assumptions represent a real spacecraft, habitat, or crew with validated accuracy.

One bounded example is orbital integration checking. A documented circular test-particle case reports return-position errors of approximately 7.93 × 10⁻⁹ and 4.96 × 10⁻¹⁰ of the initial orbital radius at two tested resolutions, with energy and angular-momentum changes below 10⁻¹¹ for those specific fixtures. A separate one-year Solar System run compares selected computed endpoints with locally cached, rounded JPL coordinates and reports per-body residuals. These are specific regression and numerical-behavior checks under stated conditions, not universal error bounds, covariance estimates, or proof of long-term ephemeris accuracy. The detailed conditions and omissions are documented in [Measured orbit checks](../SCIENTIFIC_VALIDATION.md).

The evolving habitat and vehicle studies similarly expose conditional outputs and their assumptions. For example, powered equipment may be represented by finite stores and explicit energy or heat accounting, while some device ratings and biological conditions remain idealized inputs. A recorded result is evidence of what the implemented model computed, not an independently measured outcome in the physical world.

The latest engineering arc connects several stages of finite spacecraft operation: an authored supply vehicle with dry/tank/propellant/payload budgets performs finite burns under system-local gravity, approaches a rotating planetary surface, and contacts modeled terrain through sprung feet with persistent friction-shear history. Further workflows expose per-foot loads, ground-hold evidence, cargo-mass epochs, and the duration for which a craft continuously satisfies an authored support envelope, measured from accepted contact-solver states. Flight and contact stages use explicit model/save versions and replayable recordings; cargo-mass epochs are a separate, replayable research workflow and are not yet integrated into authoritative flight mass or habitat inventory. In the documented end-to-end operation-window case, the craft remains in flight for 20 seconds; 5,800 of 10,240 solver samples support the operation window, which reaches 9.38671875 seconds against a 5-second requirement. A bounded static check reports per-foot loads of 1,961–2,067 N versus an independent prediction of about 2,013 N. These are results for a specific authored case, not validation against flight hardware or a lander mission. The chain does not yet implement attachment, cargo delivery, or a complete validated landing system. See the [operation-window method](../VEHICLE_OPERATION.md), [validation report](../VEHICLE_OPERATION_VALIDATION.md), and [cargo-mass epoch workflow](../CARGO_MASS_EPOCHS.md).

## 5. Verification and reproducibility

The repository’s engineering checks include workspace tests, linting, deterministic replay, strict input and export validation, and headless application captures for interface changes. On commit `9eb830d` (15 October 2026), `cargo test --release --workspace` passed **539 tests, with no failures and one pre-existing ignored test**; all-target clippy also exited successfully, with pre-existing warnings confined to untouched files. This is software regression evidence, not a measure of scientific validity. The exact checkpoint and its limitations are recorded in the [vehicle operation validation report](../VEHICLE_OPERATION_VALIDATION.md) and [project status audit](../OPENCODE_PROJECT_STATUS.md).

Reproducing a study requires more than the application version. Where provided, users should retain the study definition, starting-world identity, simulation/model versions, source dataset versions, requested settings, run conditions, and raw exported observations. The project’s documentation and release artifacts are the authoritative place to verify the exact workflow and version used.

## 6. Scope and limitations

Cosmogon is under active development. Its research tools are broadening, but the full product specification is not complete. Current documentation identifies important gaps in physical calibration, uncertainty propagation, independent validation, and end-to-end spacecraft and habitat operations. Graphics also mix source-supported data with generated presentation detail. Specific models may have their own further limitations, so users should consult the corresponding design and validation notes before interpreting an output.

The software should not be used as a flight-planning, medical, safety-certification, climate-prediction, or operational life-support authority. No result should be described as empirically validated unless a suitable reference comparison or independent validation is documented for that model and use case. A deterministic result can be repeatable and still be systematically wrong.

## 7. Development direction

The current development direction connects finite spacecraft operations to surface support and cargo delivery, while continuing habitat engineering and reproducible studies. The longer-term plan includes stronger validation and uncertainty treatment, richer source-backed datasets, more complete accessibility and interaction, and detailed animated close-up rendering for visitable cosmic bodies. The project tracks these as development goals; this overview does not present them as completed capabilities.

## 8. Availability and citation

Cosmogon source code, documentation, and release downloads are available from the [project repository](https://github.com/ashvyagni/SpaceRender) and its [release page](https://github.com/ashvyagni/SpaceRender/releases). A rendered copy of this overview is published at [ashvyagni.github.io/SpaceRender](https://ashvyagni.github.io/SpaceRender/). The latest release covered by this overview is **1.0.0**; the development snapshot described above is not a release. The Cargo workspace declares `MIT OR Apache-2.0`, but the 15 October 2026 repository audit found no corresponding license text files. The project’s license distribution therefore needs reconciliation before asserting that the repository includes those license terms. For reproducibility, cite a specific release or commit. A versioned Zenodo record can provide a persistent DOI for a released software snapshot or report; Zenodo assigns a DOI when a record is published ([Zenodo quick start](https://help.zenodo.org/docs/get-started/quickstart/)).

**Suggested software citation (complete before publication):**

> Dandotiya, A. (2026). *Cosmogon: A Reproducible Living-Universe Sandbox* (development snapshot `9eb830d`). [Project repository](https://github.com/ashvyagni/SpaceRender). [DOI, if archived].

## References and project records

1. NASA/JPL Solar System Dynamics. [*Horizons System*](https://ssd.jpl.nasa.gov/horizons/). Source and integration context are recorded in the project’s [data-source register](../DATA_SOURCES.md).
2. NASA Planetary Data System. [*MGS MOLA Derived Topography — MEGDR and SHADR Dataset Bundle*](https://pds.nasa.gov/ds-view/pds/viewBundle.jsp?identifier=urn%3Anasa%3Apds%3Amgs_mola_topography_derived&version=1.0). Dataset version, checksums, coverage and processing notes are recorded in the project’s [data-source register](../DATA_SOURCES.md).
3. Cosmogon contributors. *Measured orbit checks*. Project-specific methods, numerical results, and limitations: [SCIENTIFIC_VALIDATION.md](../SCIENTIFIC_VALIDATION.md).
4. Cosmogon contributors. *Research design and acceptance direction*: [RESEARCH_DESIGN.md](../RESEARCH_DESIGN.md).

## Discovery and author identity

Google Scholar is an index, not a general-purpose article-hosting service. It indexes scholarly literature from publishers and repositories, and its inclusion rules favor identifiable scholarly works with accessible full text or abstracts and extractable bibliographic information. A short project announcement or ordinary product blog post may not be indexed. Indexing is automated and cannot be guaranteed. If this overview is later reshaped into a scholarly technical report, publish the full text with a clear title, author, abstract, references, and stable public URL; then check whether Scholar has indexed it and add it to a public author profile if appropriate. See [Google Scholar’s inclusion guidance](https://scholar.google.com/intl/en/scholar/inclusion.html) and [profile help](https://scholar.google.com/intl/en/scholar/citations.html).

For a consistent public record, use the same chosen author name on the repository, an [ORCID iD](https://orcid.org/), the Zenodo record, and any later scholarly submission. The author should review and approve the final text and any AI-assistance disclosure before public release.
