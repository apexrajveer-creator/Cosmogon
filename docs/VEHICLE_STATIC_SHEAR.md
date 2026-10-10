# Persistent terrain-foot shear

Explicit world-flight **version 3** and terrain-contact **version 2** add history-dependent tangential compliance to actual finite vehicle motion. Original spherical flight and version-2 smooth-sliding contact keep their exact formats and mechanics. This is a contact capability; support/stability, attachment, safe landing and cargo delivery remain unfinished.

## Actual response and evidence

Each foot retains its finite elastic tangent displacement, preceding normal and normal load. The terrain material frame transports that history; a bounded proper normal rotation aligns it to the new local tangent without erasing stored energy. Tangential force is `-stiffness × retained displacement` up to the authored Coulomb capacity. It persists at zero speed. Reduced capacity or separation releases remaining storage explicitly. Authored force/travel limits reject unsupported motion; structural limits are not another silent friction clamp.

The coupled rigid-body solver applies shear at its midpoint prediction, alongside the existing normal spring/damper, gravity and finite central thrust. Full endpoint history uses midpoint relative foot velocity and a linear within-step contact fraction. Free-flight displacement before contact does not create preload. Tangential stiffness/travel joins the finite substep bounds; unsupported source/geometry/stiffness/force conditions stop the whole macro at its prior supported state.

Normal spring storage and its instantaneous derivative remain in the original foot energy/rate fields. Optional shear state and `shear.csv` provide tangential storage/force separately. Original `contact.csv` adds explicitly prefixed shear counters only for the new capability. Endpoint constitutive work, plastic slip loss, unloading release, numerical endpoint term and residual are retained separately from actual midpoint mechanical force/torque work. Their coupling residual remains visible. A numerical term is not measured material friction or proof of mechanical accuracy.

Actual source context carries the reduced model, assumptions and primary references. Overview MOLA/height-field normals are not new measured meter-scale terrain. Calibration and uncertainty remain unavailable. The original complete branch and every numeric export are independently recomputed on reopening.

## Native use

Supply vehicles → World flight → **Use terrain and rigid-body contact model** → **Persistent foot shear (replaces smooth sliding friction)** enables the new version in the staged definition. Editable tangential stiffness, Coulomb coefficient, elastic travel and force limit are in **Contact hardware assumptions**. Existing normal gear and pose controls stay available. Enabling persistent shear explicitly sets the old smooth coefficient to zero; disabling it leaves that coefficient editable. Both laws cannot be applied together.

Creation follows the ordinary undoable edit and actual-terrain initial clearance checks. Live/recorded views expose storage, slip/release/numerical terms and complete per-foot history. Recorded graph choices separate normal energy/loss from tangential storage, plastic loss, release, endpoint numerical term, residual and accumulated slip distance. Frozen worker requests retain their original law independently of subsequent edits.

## Command-line use

Use the same world, target and geographic start for hardware and flight creation. For a saved origin pass `--initial WORLD` to hardware, definition and study:

```sh
./target/release/cosmogon-cli contact-hardware --target Mars --static-shear --geographic-start /tmp/contact-start.json --output /tmp/persistent-hardware.json
./target/release/cosmogon-cli flight-definition --target Mars --geographic-start /tmp/contact-start.json --contact-hardware /tmp/persistent-hardware.json --samples 9 --interval-seconds 0.5 --output /tmp/persistent-study.json
./target/release/cosmogon-cli flight-study --definition /tmp/persistent-study.json --output /tmp/persistent-recording
./target/release/cosmogon-cli flight-inspect --recording /tmp/persistent-recording
```

Generate an explicitly signed start with `contact-start --output NEW_FILE`, and choose an altitude that clears actual terrain and extended feet. Examples are authored initial conditions, not descent guidance or launched cargo. Outputs are protected. The additional `shear.csv` is also retained in nested approach reports; corrupt/missing histories, work, model pins or numeric files are refused.

## Verification and continuation

The integrated checkpoint passes **505 release tests plus one existing ignored**, all six release/package checks, **67 CLI/native cases**, six inspected packaged screenshots and a **48.448818-second** ordinary live run. All 171 frozen Rust files match. Four integrated simulation tests and a native frozen-law export test supplement the preceding eight component tests. Old spherical and smooth-sliding contact recordings replay unchanged. [Evidence and measured numerical limits](VEHICLE_STATIC_SHEAR_VALIDATION.md). The preceding contact checkpoint is `1c0c6e0`; the component checkpoint is `ea52bfd`.

The next connected build must establish actual footprint/load/stability and supported surface-operation evidence, then finite co-located cargo transfer to the same habitat stocks. General water/crew-food capacities, actual changing cargo mass, thermal/handling and external-transfer ledgers need explicit models. Cargo currently stays aboard. Atmospheric entry, physical hardware calibration, scene/camera-follow detail and the full directive remain incomplete. Graphics stay queued.

[Mechanics, primary research and inventory constraints](STATIC_FOOT_SUPPORT_DESIGN.md).
