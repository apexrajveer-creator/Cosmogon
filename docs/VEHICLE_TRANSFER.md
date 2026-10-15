# Atomic co-located habitat transfer

Build 47 of the supply sequence. This build connects the handling preflight (build 46) to a
single undoable world edit that **actually moves cargo**: a supported craft co-located with a
habitat within the authored handling reach commits both sides atomically, or refuses and
changes nothing. It follows the finite handling equipment and storage capacity (build 46),
the surface-operation window (build 44) and the cargo-mass world flight (build 45).

## The edit

`Edit::TransferVehicleCargoToHabitat { system, vehicle_id, habitat_id, requested }`
(`cosmogon-vehicle-transfer/1`) is one atomic sandbox edit: both sides or neither. Undo is
the ordinary snapshot-based sandbox undo; the transfer participates automatically.

## Plan: preflight + reach + rate

`vehicle_transfer::plan(u, flight, recipient, requested) -> Result<Plan>` layers three gates
on the build-46 preflight:

1. **Handling preflight** (authored equipment, flying phase, cargo ledgers and retained
   evidence, same body, continuous supported-operation window, finite nonnegative request,
   per-species / total / shared-oxygen-interface limits, recipient free space).
2. **Modeled spherical reach.** The craft's ground site (surface-frame position projected to
   the body-fixed latitude/longitude) must lie within `reach_m` of the habitat site along the
   great-circle arc on the modeled sphere. Default `reach_m = 1e7` m; authored values are
   bounded `0..=1e7`.
3. **Window-scaled mass-rate envelope.** `mass_rate_kg_s × current_window_s` bounds the total
   mass moved this operation. The accepted set is trimmed from the **end** of the fixed
   species order (O₂, potable water, dry food, food water) down to the envelope; the trimmed
   remainder joins the preflight's rejected stock and stays on the craft. Default
   `mass_rate_kg_s = 1e6`; authored values are bounded `0..=1e6`.

A final ulp clamp guarantees `held + accepted ≤ tank` for every species so the committed
habitat state passes its exact capacity comparison. If every species clamps to zero the plan
refuses with "accepts no material".

## Commit: both sides or neither

The apply arm steps the recipient to the edit time **first** (so plan and receipt see the
same post-step stocks), then:

* **Craft side:** `Flight::remove_cargo(accepted, Vec3d::ZERO)` — a co-moving, impulse-free
  removal. Craft velocity is unchanged; the zero-relative cargo event and `CargoEvent` record
  the removal for `validate_saved` and replay.
* **Flight record:** one `Transfer` (requested, accepted, rejected, window, separation, reach,
  rate envelope, ground-relative velocity, ground-absorbed linear momentum
  `v × accepted.mass` and kinetic energy `½ m v²`, declared power/thermal, craft mass before/
  after) appended to `Flight.transfers` (bounded 4096, conditional save pin).
* **Habitat side:** `vehicle_transfer::receive()` adds the accepted stocks and appends one
  `Receipt` to `Habitat.receipts` (same bounds, same pin). The habitat's ordinary `validate`
  then runs, so a receipt that would breach a tank is refused by the same rule as any other
  habitat state.

Ground-absorbed momentum/energy and the declared power/thermal values are retained **evidence
and assumptions**: no structural, thermal, attachment or power-bus integration is modeled.

## Retained records and replay

`validate_transfers` re-derives every retained record on load: ordered zero-relative cargo
event matching (`actual_removed == accepted == event.removal.requested`), habitat on the same
body, matching receipt by `(world_time_s, vehicle_id, accepted, requested)` and the
`accepted + rejected == requested` closure. A conditional `cosmogon-vehicle-transfer` save pin
appears only when a world contains transfers or receipts.

## Studies and exposure

* `world_vehicle_recording::Study.transfers` authors transfers at exact sample boundaries
  (applied after the world reaches the sample time, before its observation is recorded) and
  validates sample bounds, finite positive cargo and habitat existence up front. Studies that
  carry transfers gain a conditional `manifest["transfers"]` block and a `transfers.csv` file;
  the handling manifest scope reads "consumed by the retained transfer records".
* CLI: `habitat-setup` creates a protected world containing one finite habitat;
  `flight-definition --handling` opts a definition into the versioned handling equipment.
  Transfers are authored by adding a `transfers` array to the pinned study JSON.
* Native: the world-vehicle pane gains a co-located transfer section (habitat selector,
  per-species request editors, execute button, retained-record summary) and the habitat pane
  gains a received-transfers collapsing header.

## Limits

Instantaneous transfer, declared-but-unmodeled power/thermal draw, modeled spherical reach
(no terrain, curvature beyond the sphere, or obstruction), window evidence as the only
temporal gate, and ground absorption as evidence only. Rejected stock stays on the craft; no
tank is unbounded; no delivery is automatic; a committed transfer is not landing, attachment,
hardware feasibility or mission feasibility. Detailed body/terrain/civilization graphics remain
queued under the owner's supplement.
