# Native habitat recordings and graphs

Habitat engineering → Recordings reopens a protected habitat-study folder produced by CLI `habitat` or native Model & export. The recorded experiment is independent of the live world: opening a bundle does not replace, edit or rewind it. Imports run on a worker; success replaces the prepared view and failure preserves prior observations. A native snapshot is one observation, while a trajectory contains multiple actual requested world times.

Select one of fourteen stock, pressure, temperature, energy, power or reaction metrics. Graph axes show the metric's unit and elapsed terrestrial hours from the first recorded world time. Points are actual observations; connecting lines are visual guides, and unavailable active intervals create gaps. The pointer and observation slider select the nearest stored row without interpolation. Read the exact selected value, world time, completed habitat time and interval duration. Expand conditions/sources for configuration and constraints; the shared model workspace and CLI `models` describe equations. Copy the full selected report JSON with all stored quantities and source context.

The shared bounded reader checks manifest format/model/source, 1–1024 observations, a maximum 30-day interval, increasing finite world times, habitat identity/original definition, every row's finite model/configuration/budgets and closed mass, exact initial/final saved endpoints and fingerprints, the final report and all CSV cells. Source and optional reaction version requirements remain explicit. Legacy 38-/49-column CSV and current 66-column studies are supported; new outputs retain additive columns. Malformed, oversized, future/mismatched sources, corrupt budgets/endpoints/fingerprints/reports or inconsistent CSV fail explicitly. Files are bounded individually: manifest/definition 64 KiB, report 2 MiB, samples JSON/CSV 16 MiB each, initial/final worlds 32 MiB each. Original data is never rewritten by the reader.

These are structural numerical consistency checks. State fingerprints are FNV identity, not cryptographic authenticity; this viewer does not independently replay a trajectory or establish measured field accuracy. Recorded environments/equipment remain modelled or user-assumed with unquantified predictive uncertainty. A missing historic reaction source cannot be silently filled from the current live world.

## Shared CLI inspection and native verification

```sh
./target/release/cosmogon-cli habitat-inspect --recording /tmp/habitat-study
./target/release/cosmogon --new lab --focus Mars --habitat-recording /tmp/habitat-study \
  --habitat-metric methane --habitat-sample 24 --panel habitat-recordings --quiet \
  --capture /tmp/habitat-recording.png --capture-metadata --capture-after 240 --quit
```

`habitat-inspect` accepts only `--recording` and emits JSON summary/validation scope, exact state identities, source versions and available metric IDs. The native metric/sample flags require a recording; sample is a zero-based stored index and out-of-range requests fail. Combining a crop definition with a recording launch is rejected to avoid ambiguous panes. `--load` may show a different live world while the recording remains independent. Automated invalid studies fail before GPU startup. `--panel habitat-recordings` also opens the empty reader without a launch import.

Photo metadata retains recorded summary, chosen metric/unit and actual selected row/value, even if the panel is hidden. It labels the independent recorded scope separately from live `habitat_reports` and the crop forecast. Two-recording comparisons and controlled native trajectories are described in [HABITAT_COMPARISONS](HABITAT_COMPARISONS.md); broader statistical workflows and uncertainty propagation remain separate acceptance gates. No Universe/save state or golden fingerprint change.
