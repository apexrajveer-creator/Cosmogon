# Recorded cabin balances

Open Habitat engineering → Recordings → Cabin balances after opening one recording or a paired design. Choose two ordered retained observation indices; the initial view uses first and last. Stocks, Electricity, Temperature and Counters show the selected branch. A paired design keeps four independent ledgers; its branch selector changes the displayed ledger and export retains all four.

Stocks lists all sixteen cabin/isolated mass stores, four escaped gas components and battery energy, before/after/change with original units. Escaped gas stays outside the usable cabin. Component changes are summed independently against the endpoint accounted mass change; the signed residual and endpoint drift remain visible. An endpoint ledger does not identify every intervening transfer or edit.

Electricity uses differences of retained cumulative counters:

`delta generated + initial battery - final battery - delta delivered - delta conversion losses - delta curtailed`

The signed residual is retained. Negative battery withdrawal means a storage gain. Unserved requested energy is separate; it is not delivered output or conversion loss. Sampled minute watts are never integrated across the recording span.

Temperature shows endpoint kelvins and reference phase-energy changes. Effective heat capacity × temperature change is available only if every retained effective capacity is the same. These reference storage changes do not provide a full-span thermal closure: the current model does not retain a complete cumulative ledger of every heat pathway. Counters preserves all fifteen existing cumulative totals, including reaction/phase processing and escaped-gas reference enthalpy. Served and unserved person-seconds describe nominal model demand rather than physiology, health or nutrition.

Evidence & export retains checks of stored configuration/counter sequences, distinguishes an initial uncompleted budget from unsupported completed sampled intervals and saves to a new folder. `report.json` contains exact endpoint samples, original source summaries and paired plan/order/common-state/method context; `stocks.csv` and `counters.csv` contain seven columns with 17-digit original values. `manifest.json` is written last. Existing folders and original studies are protected. A completed bundle is not a cryptographic provenance guarantee, field calibration or independent replay.

```bash
./target/release/cosmogon-cli habitat-balance --recording /tmp/study \
  --from-index 0 --observation 24 --output /tmp/new-balance
```

Omit either index to use first or last respectively. Omit output to inspect JSON only. Paired roots reopen all four validated branches. No other study controls are silently accepted.

```bash
./dist/Cosmogon.app/Contents/MacOS/cosmogon --new lab --focus Mars \
  --habitat-recording /tmp/study --panel habitat-recording-balance-temperature \
  --habitat-balance-from 0 --habitat-sample 24 \
  --capture /tmp/cabin-temperature-balance.png --capture-after 240 --quit
```

Native aliases: `habitat-recording-balance`, `-electricity`, `-temperature`, `-counters`, `-files`. `--habitat-balance-output NEW_DIR` freezes and exports actual loaded inputs once the recording or native sweep finishes. Request-time metadata is separate from later display choices. Cached reports recompute when the range or loaded evidence changes; workers preserve the live world and recording inputs.

Retained checks cover retained rows only. They cannot exclude intervening edits or unsupported intermediate behavior, reconstruct a history or certify hardware. Original world/sample analyses, simulation laws, save schema and the golden fingerprint remain unchanged.
