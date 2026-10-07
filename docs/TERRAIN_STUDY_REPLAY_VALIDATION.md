# Native terrain study replay verification

Date: 2026-10-08. Parent source-refinement checkpoint: `227a4b5`.

- `cargo test --release --workspace`: 233 passed, one existing ignored. New semantic check covers transactional imports, bounded/unsupported/antipodal refusal, pinned radius and sample count, source mismatch, stale-input trace suppression, selected-row clamping and photo metadata.
- Release workspace clippy/all targets succeeds with the existing warnings; no new warnings in changed code. Release native build succeeds. No dependency, simulation state or dataset change; golden remains `0xf2c4140a45ae509b`.
- Eight native/CLI QA cases pass. A 23-row 16ppd definition on radius 3,396,000 m reopens independently of the simulated 3,389,500 m Mars. Distance is 549,447.7745122695 m. Photo sidecars retain the exact definition and selected numeric row, equal to shared CLI output.
- Inspected all five native captures: Graph view on matching 16ppd, differing 4ppd, legacy procedural Mars, no Mars, and compact Map view. The sampled gold/coral trace and white row marker are visible in the geographic capture. Source-mismatch views preserve the graph and explain disabled geographic actions. Expanded content scrolls; setup/import collapses to keep graph visible.
- Unsupported version, oversized definition and antipodal endpoints fail before GPU startup, with no PNG. The app-level test proves earlier study preservation after each failed import.

Retained evidence: ignored `dist/terrain-study-replay-validation/` (definitions, expected numeric rows, exact checks, captures/sidecars, runners, results and ordinary-run profile). A packaged 48-second ordinary run uses normal pipelined rendering; local ad-hoc signature and disk-image verification are checked separately. No public release, push or notarization.

Limits: manual pointer/keyboard/assistive-technology interaction is unverified. Captures establish native composition and numeric context, not every interactive action. The trace connects stored rows, is not a continuous terrain profile or safe rover route, and does not resolve local slope or uncertainty. Study preferences are session-local. Photos may record a study whose window is hidden. Imported source/radius never changes the world automatically. Mars' axial-tilt/spin model remains the existing approximate geographic frame.
