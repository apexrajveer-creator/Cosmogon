# Atmospheric-entry heating and terminal landing limits

An entry-limited flight is an ordinary contact-capable craft that opts into
versioned stagnation-point heating limits and retained thermal evidence
(`cosmogon-vehicle-entry/1`). A landing-limited flight opts into versioned
first-touchdown speed limits and retained touchdown evidence
(`cosmogon-vehicle-landing/1`). Both close the last missing evidence links in
the authored mission chain (launch → orbit → transfer → entry → landing): the
craft can now *measure* entry heating against an authored bound and *record*
its first touchdown speeds against authored landing bounds, with an atomic
stop at the last supported state when a bound is exceeded.

The atmosphere is the parent body's existing modeled bulk composition through
an exponential height model; touchdown speeds are ground-relative through the
existing surface spin frame. There is no aerodynamic force, attitude
controller, trajectory guidance, radiation/ablation model, landing
certification, attachment or cargo-delivery grant.

## Entry limits (`cosmogon-vehicle-entry/1`)

`Definition.entry` (`Option<Entry>`) is opt-in; absent legacy definitions
serialize byte-exactly.

| Field | Meaning | Bounds | Example |
| --- | --- | --- | --- |
| `entry_interface_altitude_m` | Height above the parent's reference sphere counted as the entry interface (crossing is recorded evidence, not a physical boundary) | `0..=1e7`, finite | 110000 |
| `nose_radius_m` | Effective stagnation-point nose radius of the correlation | `1e-3..=1e4`, finite | 1 |
| `heat_flux_coefficient` | Sutton-Graves coefficient `q = k·sqrt(ρ/Rn)·v³` (W/m², kg/m³, m, m/s) — the classic air value does not transfer to other gas mixtures | `0..=1e-1`, finite | 1.7415e-4 |
| `maximum_heat_flux_w_m2` | Stagnation heat-flux bound | `1e-30..=1e12`, finite | 1e7 |
| `maximum_total_heat_j_m2` | Integrated stagnation fluence bound | `1e-30..=1e18`, finite | 1e10 |

Structural rules (enforced in `validate_structure` / `Flight::new`): entry
limits require authored contact landing gear, and the starting body must have a
modeled atmosphere.

`Flight.entry_ledger` retains `above_interface` (sticky so a crossing is
recorded across substep boundaries), `interface_elapsed_s` (first accepted
substep at or below the interface after any substep above it, on the craft
propulsion clock), `peak_heat_flux_w_m2`, `total_heat_j_m2` and
`substeps_observed`. Heating accrues at **every accepted contact substep** from
the exponential atmosphere, which is never exactly zero above the interface;
the interface timestamp is evidence, not a heat gate. Exceeding either bound
returns an atomic error so the world keeps the last supported macro state with
`Phase::Unsupported` carrying both numbers.

## Landing limits (`cosmogon-vehicle-landing/1`)

`Definition.landing` (`Option<Limits>`) is opt-in; absent legacy definitions
serialize byte-exactly.

| Field | Meaning | Bounds | Example |
| --- | --- | --- | --- |
| `maximum_touchdown_vertical_speed_m_s` | Ground-relative normal speed bound at first touchdown | `0..=1e4`, finite | 10 |
| `maximum_touchdown_horizontal_speed_m_s` | Ground-relative tangential speed bound at first touchdown | `0..=1e4`, finite | 5 |
| `liftoff_altitude_m` | Height above the authored start clearance that counts as airborne evidence | `0..=1e5`, finite | 0.05 |

Landing limits require authored contact landing gear. `Flight.landing_ledger`
retains `start_clearance_m` (terrain radial clearance at creation),
`airborne`, `touchdown_elapsed_s`, `touchdown_vertical_speed_m_s`,
`touchdown_horizontal_speed_m_s` and `substeps_observed`. Touchdown is the
first accepted contact substep where a foot meets the modeled terrain (≤ 1 cm
gap) after all feet were clear. A surface-pad craft must rise
`liftoff_altitude_m` above its start clearance before any later touchdown can
count — gear settling onto the pad is never a landing. A descent start above
the bound is airborne from creation. Exceeding either speed bound stops the
craft atomically at the last supported state.

## Evidence, saves and studies

* Both ledgers are evaluated after every accepted contact substep and
  revalidated on reopen (`validate_saved` requires each ledger exactly when its
  limits exist).
* Saves gain conditional `cosmogon-vehicle-entry` / `cosmogon-vehicle-landing`
  data-version pins when any flight carries limits or a ledger; absent legacy
  worlds keep the golden `0xf2c4140a45ae509b`.
* Flight studies with entry/landing craft gain conditional
  `manifest["entry_evidence"]` / `manifest["landing_evidence"]` blocks (final
  ledger values) with exact `flight-inspect` replay; the research registry
  gains `vehicle-entry` / `vehicle-landing` descriptors.

## CLI

```
cosmogon-cli entry-definition [--entry-interface-altitude M --nose-radius M \
    --heat-flux-coefficient K --maximum-heat-flux Q --maximum-total-heat H] \
    [--initial WORLD | --scenario S --seed N] --target Mars \
    --latitude N --longitude E --output NEW_FILE
cosmogon-cli landing-definition [--maximum-touchdown-vertical M_S \
    --maximum-touchdown-horizontal M_S --landing-liftoff-altitude M] \
    [--initial WORLD | --scenario S --seed N] --target Mars \
    --latitude N --longitude E --output NEW_FILE
cosmogon-cli flight-definition --entry FILE | --landing FILE \
    [--contact-hardware | --launch FILE] [--samples N --interval-seconds S] \
    --output NEW_FILE
```

`entry-definition` / `landing-definition` resolve a pad on actual local
terrain, author a coast plan on the example bench with authored landing gear,
attach the limits and write the **full definition file** (the same format
`flight-definition --entry/--landing` consumes: read the file as a complete
definition and use its `entry`/`landing` limits). `--entry`/`--landing` on
`flight-definition` are mutually exclusive with `--geographic-start`,
`--definition` and each other, and require `--contact-hardware` or
`--launch FILE`. `flight-study` runs the pinned study as usual; it does not
accept `--entry`/`--landing` flags.

Study budgets still apply: contact-craft studies in SolarSystemLab allow
roughly 39 one-second samples (`duration × work × sources ≤ 20_000_000`).

## Native UI

The world-vehicle pane shows, above the flight details: an "Entry heating
limits and evidence" readout (interface, nose radius, coefficient, bounds,
running peak flux/total heat/substeps, interface crossing time, Sutton-Graves
scope reminder) and a "Landing limits and touchdown evidence" readout (bounds,
liftoff bound, start clearance, airborne flag, touchdown time/speeds, gear
scope reminder). The recorded-flight pane shows the same evidence for the
selected observation; the files pane and staged-geography editor carry
conditional entry/landing disclaimers and carry-over notes. Craft with landing
limits show the conditional header note that authored touchdown limits add
evidence only.

## Scope

This is explicit authored mechanics on the existing bounded solvers. Heating is
stagnation-point convection on a bulk exponential atmosphere — not an entry
corridor, radiation, ablation or aerodynamic-force model. Touchdown evidence is
first-contact speeds — not a certified landing, static attachment, terrain
hazard, tip-over or structural certification, and it does not deliver cargo.
