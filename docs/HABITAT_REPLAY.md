# Verify a saved paired habitat experiment

The Replay page recomputes all four saved branches from the retained common world. It checks every complete stored observation, plus initial/final world fingerprints and source versions. This is an explicit reproducibility check with the current deterministic engine. It does not establish agreement with real equipment, predictive confidence or cryptographic authenticity. See the [design memo](HABITAT_REPLAY_DESIGN.md) and [verification evidence](HABITAT_REPLAY_VALIDATION.md).

Open a complete paired experiment folder in Habitat engineering → Recordings → Paired results → Replay. Choose **Verify all four branches**. The recorded graph and selected observation remain available while a worker computes the check; progress identifies the branch and observation. Cancel requests stop at a cooperative scheduler boundary. Closing the app also requests cancellation. Source loading is bounded but is not interrupted in the middle of a read. The independent live world is never edited by replay.

A completed report distinguishes **complete** (the verification finished) from **verified** (all observations, endpoints and sources matched). An altered interior row can fail even when structural checks and endpoint worlds agree. The page shows per-branch counts and endpoint/source outcomes, and the first differing row/time; exact details retain up to sixteen differing observations and eight fields each. Long text and compound values use bounded previews. Full comparison uses canonical typed Sample bytes, including signed-zero representation. Diagnostic hashes are noncryptographic. Input failure or cancellation produces no successful completed report; the previous report, if present, is preserved with its own source identity.

Copy the completed JSON or choose a **new** report folder and export. The shared exporter writes `verification.json`, then the completion `manifest.json` last, without overwriting a report. A report with differences can be exported and remains `verified: false`. Original records, final analyses and trajectory exports stay read-only. Native report exports use the completed report captured at request time; editing an unopened folder field cannot redirect the verified source.

The headless command uses the same engine-independent verifier:

```bash
./target/release/cosmogon-cli habitat-verify \
    --recording /tmp/my-paired-experiment \
    --output /tmp/new-replay-report
```

`--recording` requires a complete source-compatible paired root, including common base and all four branches. `--output` is optional and must name a new folder. Standard output is the completed JSON. Exit status is zero only for an exact match; differences and input/export errors return nonzero. Differences still print their completed JSON and can retain a protected report. Observation/metric/scenario/sample flags are refused because replay checks the whole design. Existing `habitat-inspect` remains a structural/derived-analysis operation and never silently starts replay.

For capture and startup validation:

```bash
./target/release/cosmogon --new lab --focus Mars --speed 0 \
    --habitat-recording /tmp/my-paired-experiment --habitat-verify \
    --panel habitat-paired-replay --capture /tmp/replay.png \
    --capture-metadata --capture-after 400 --quit
```

`--habitat-verify` queues explicit whole-design verification after loading (or after a paired `--habitat-sweep` worker completes). It requires a paired input; ordinary branches and single-setting plans are refused before renderer startup. `--panel habitat-paired-replay` opens the page but alone does not run verification. Capture timing is measured in rendered frames, so a slow verification may still be pending at capture; metadata truthfully retains progress/completion. This build preserves the simulation/save schema, source coefficients and golden fingerprint.

[ NASA-STD-7009B](https://standards.nasa.gov/sites/default/files/standards/NASA/B/1/NASA-STD-7009B-Final-3-5-2024.pdf) provides methodological context for distinguishing implementation verification from real-system validation. The replay algorithm is Cosmogon's engineering regression mechanism; no NASA-standard compliance is claimed. Empirical calibration, physical greenhouse exchange, uncertainty and the full master directive remain incomplete.
