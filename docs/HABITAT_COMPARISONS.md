# Habitat experiments and comparisons

Habitat engineering → Recordings runs controlled experiments without changing the live world. Choose **Run experiment**, select a live habitat and a setting, enter distinct comma-separated values in the displayed unit, choose observations/interval and a new output folder, then run. Calculation uses a captured world clone on a worker. It opens the first and last trajectories when complete; all intermediate run folders remain available through the ordinary reader. Reference sessions must first be cloned to a sandbox.

Supported settings are external firm supply (W), horizontal solar area (m²), battery capacity (J), water recovery fraction, cabin temperature setpoint (K), and already enabled electrolysis/reduction power (W) or reaction rate (kg/s). These retain the existing model's assumed equipment bounds. Initial stored energy/mass remain fixed: raising capacity does not fill a battery, and a smaller capacity cannot discard held energy. Existing edit-time remainders finish under the old configuration before each branch's change.

Each branch starts from one common world, makes one ordinary configuration edit and advances through the normal deterministic scheduler. A bounded plan requires 2–16 distinct finite values, 1–1024 observations per branch, at most 8192 total observations, a positive interval of at least one second and a maximum 30-day horizon. Every branch edit is previewed before creating output. A protected output contains `base.cosmo`, `plan.json`, every numbered `run-000` study with its seven ordinary files, and a completion `manifest.json` written last. A folder without that completion manifest is incomplete. Existing results are never overwritten.

This measures conditional model sensitivity. A single-factor sweep cannot identify interactions between multiple changed inputs; see [NIST/SEMATECH experimental methods](https://www.itl.nist.gov/div898/handbook/pri/section2/pri212.htm). It does not provide confidence intervals, hardware calibration or proof of globally optimal settings.

Open a second study folder as B to compare any independently validated pair. A circles and B squares have distinct colours and share one metric/unit scale. Select **Elapsed time** to align durations from each recording's beginning or **World time** to align physical simulation coordinates. Only exact stored timestamps match. No nearest-time subtraction, interpolation or extrapolation is performed. Interval quantities additionally require equal positive completed durations and aligned habitat interval ends; a snapshot without an active interval stays unavailable. The selected A, B and B−A values expose missing time/value/support explicitly. Arbitrary studies can have different epochs, sites, source versions or initial conditions and do not establish a controlled intervention.

Expand comparison conditions to inspect both original configurations/stocks and source identities, copy full comparison JSON or export actual pairs as protected JSON/CSV. Expand all selected values for both supported actual reports. Photo metadata retains original A/B rows, support and the completed sweep manifest separately from live habitat reports. Failed imports/runs preserve prior views. No simulation schema or golden change.

## Headless and automated equivalents

```sh
./target/release/cosmogon-cli habitat-sweep --initial /tmp/common.cosmo \
  --parameter firm_power_w --from 0 --to 2000 --runs 3 \
  --samples 25 --interval-days 0.041666666666666664 --output /tmp/new-sweep
./target/release/cosmogon-cli habitat-sweep --initial /tmp/new-sweep/base.cosmo \
  --definition /tmp/new-sweep/plan.json --output /tmp/new-replay
./target/release/cosmogon-cli habitat-compare \
  --recording /tmp/new-sweep/run-000 --reference /tmp/new-sweep/run-002 \
  --measure methane --alignment elapsed --output /tmp/new-comparison
./target/release/cosmogon --load /tmp/common.cosmo \
  --habitat-sweep /tmp/new-sweep/plan.json --habitat-study-output /tmp/native-sweep \
  --habitat-metric methane --habitat-sample 24 --panel habitat-recordings --quiet \
  --capture /tmp/compared.png --capture-metadata --capture-after 240 --quit
./target/release/cosmogon --new lab --habitat-recording /tmp/new-sweep/run-000 \
  --habitat-compare /tmp/new-sweep/run-002 --habitat-alignment world \
  --habitat-metric battery --habitat-sample 24 --panel habitat-recordings
```

For multiple habitats use CLI `--habitat-id` or native launch `--habitat-study-id`. CLI saved plans and inline range/sample flags are mutually exclusive. `--panel habitat-experiments` opens the compact native experiment controls. Save headers retain actual wall-clock save times; numeric files and simulation states replay exactly. Native automatic sweeps use the exact `--load` saved world on the worker, independently of subsequent live advancement; interactive runs use the current captured world. Invalid automated plans/partners fail before GPU startup. Multi-parameter designs, statistical uncertainty, native long-running acquisition/cancellation, physical greenhouse exchange and independent field validation remain unfinished.
