# Terrain and rigid-body vehicle response

Verified local development checkpoint: **492 passing tests plus one existing ignored**, 57 workflow cases, six inspected packaged captures and a 48-second ordinary run. [Evidence and numerical limits](VEHICLE_CONTACT_VALIDATION.md). This is a mechanical response capability, not completed safe landing, attachment or physical delivery.

World-flight version 2 explicitly adds contact hardware. Original version-1 definitions, spherical impact behavior and recordings retain their original semantics. Optional contact state is absent from untouched worlds; enabling the capability adds separate save model pins without changing the legacy golden fingerprint.

## Actual mechanics

Each authored foot has a body-space offset, spring stiffness, damping, finite stroke, resultant load limit and bounded smooth sliding friction. Its location and velocity follow the vehicle's integrated orientation and angular velocity. The shared terrain sampler uses the existing body-fixed height field, finite-difference tangent normal, signed first-order penetration and derivative-aware moving surface. MOLA overview source/datum limits remain visible: a one-meter normal probe does not create one-meter observed terrain data.

Actual centre translation and principal-axis Euler rotation evolve together. Deterministic Hamilton quaternion increments track body orientation. Diagonal inertia is retained vehicle mass times fixed squared gyration radii, with inertia triangle and hull/gear bounding-extent checks. Symmetric co-rotating fuel removal carries corresponding angular momentum; the existing central thrust command uses inertial axes and adds no lever torque. This is an explicit authored distribution/exhaust assumption, not measured tank or engine behavior.

The ordinary one-second vehicle task resolves bounded midpoint substeps. Dyadic maximum steps, coupled stiffness/damping leverage near terrain, angular increments, local foot travel, stroke, burn boundaries and finite fuel cutoff constrain those steps. Supported precision and finite force/stroke/hull checks reject unsupported cases. Failure retains the prior supported macro state and an explicit reason. High time warp resolves active contact motion rather than skipping it.

Stored final feet and both last-macro and cumulative work include actual contact force/torque work, moving-ground work, elastic-storage changes, normal/sliding dissipation, translational/rotational kinetic changes, gravity/thrust work, pre-ejection co-moving/co-rotating removed-material kinetic energy and exhausted angular momentum. The kinetic removal terms are the body balance share, not exhaust-jet or fuel chemical energy; a complete engine energy budget remains unavailable. Numerical balances retain their actual residuals. Contact storage/work residuals include the changing fitted terrain geometry; they are not a claim of exact global conservation or calibrated safety. Subsecond observations keep the last completed world task and disclose both clocks.

## Native controls

Supply vehicles → World flight stages the existing vectors and burn plan. **Use terrain and rigid-body contact model** enables separately typed hardware. **Contact hardware assumptions** edits gyration radii, unit orientation, initial body angular velocity, hull radius, normal probe, maximum step and each foot's offsets/response. Explicit normalization does not silently replace an invalid zero quaternion.

Creation uses the normal undoable sandbox edit. Feet must initially clear actual modeled terrain; no preloaded spring energy or habitat stock withdrawal is invented. Live and recorded readings show loaded feet, compression/load, body angular speed, resolved steps and numerical residuals. Recorded graphs expose loaded feet, elastic energy, cumulative dissipation, angular speed and contact residual. Full numeric/source evidence remains available independently of the live world.

## Command-line workflow

After building the CLI, `contact-start --output NEW_FILE` emits an explicitly versioned geographic contact template. Its signed altitude may be below the spherical base while still above actual terrain; original `arrival-start` definitions keep their positive spherical-altitude rule. Without explicit contact hardware, the new geographic template is refused.

Emit an authored hardware file:

```sh
./target/release/cosmogon-cli contact-hardware --target Mars --output /tmp/contact-hardware.json
```

That example aligns its body +Z axis to the modeled terrain normal at the default circular starting position. For a geographic start, pass the same explicit `--geographic-start FILE` and original world/target when producing hardware and the flight definition. Hardware alignment is a starting assumption, not an autopilot. Geographic altitude is above the spherical base; clear the actual terrain and extended feet when selecting a start.

Enable it explicitly in a short pinned study:

```sh
./target/release/cosmogon-cli flight-definition --target Mars --contact-hardware /tmp/contact-hardware.json --samples 5 --interval-seconds 0.5 --output /tmp/contact-study.json
./target/release/cosmogon-cli flight-study --definition /tmp/contact-study.json --output /tmp/contact-recording
./target/release/cosmogon-cli flight-inspect --recording /tmp/contact-recording
```

For a saved origin use `--initial WORLD` during definition and study. Protected output never replaces existing files/folders. Contact-capable bundles add **contact.csv** to the original full-world files; every foot row carries world time, craft time, pose/force/torque/terrain numbers and explicitly named cumulative work. Inspection independently recomputes every observation, all CSVs, manifests and final world. New contact work uses a conservative per-second budget; long/stiff/large-fleet studies can be refused before allocation.

## Remaining scope

Smooth sliding friction does not supply static sticking. Loaded feet and slow motion alone do not establish footprint stability, supported attachment or cargo access. Static friction, persistent support, explicit same-site atomic cargo transfer, launch, staging, atmospheric entry/heating, hardware power/thermal/control constraints, measured calibration and uncertainty remain unfinished. Hull clearance uses a local fitted tangent; no resolved hull/other-body collision response or measured local hazard map is implemented. Craft scene geometry and camera-follow work are separate queued graphics requirements.

Mechanics and inference sources are recorded in [the design memo](VEHICLE_CONTACT_DESIGN.md) and the shared scientific-model registry. All original phase-10 and graphics requirements remain in scope.
