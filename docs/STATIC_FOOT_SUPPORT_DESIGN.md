# Persistent foot shear and supported surface operations — active build

The accepted terrain-contact checkpoint is `1c0c6e0`: coupled normal gear response and smooth sliding friction, 492 passing tests. It cannot hold a finite lateral load at zero relative speed. The next integrated build must add persistent shear, demonstrate rotating-ground support and footprint stability, and then connect actual retained cargo to the exact co-located habitat stores. Do not grant resources, freeze motion from a speed threshold, or reinterpret old records.

## Evidence and inference

[Kuhn, Suzuki and Daouadji (2020)](https://arxiv.org/html/2002.10231v1) describes contact springs limited by Coulomb friction, with force history in the tangent plane. It identifies fresh-contact timing, elastic-to-sliding transitions, changing slip directions and objective rotation as relevant to 3D force/work calculations. This supports retaining tangential history and testing rotations, reversals and energy. It does not calibrate spacecraft feet, terrain materials or the simplified algorithm below; we are not claiming to implement its complete within-step solution.

[NASA's Apollo 11 touchdown reconstruction (2026)](https://ntrs.nasa.gov/api/citations/20260000331/downloads/Apollo%2011%20LM%20Touchdown%20Dynamics%20VnV%20for%20HLS.pdf?attachment=true) distinguishes coupled landing loads and toppling from broader landing hazards, compares independent models, and documents terrain uncertainty. Its gear has separate crush elements and articulated footpads. Our fixed-offset compliant feet are an authored reduced mechanism, not Apollo hardware or human-rated landing validation. Regional overview relief does not resolve local hazards.

## Implementable mechanism

Start with an engine-independent, separately versioned finite shear law. Retain elastic tangential displacement and the preceding unit normal/load. The caller supplies actual material-frame rotation and only the supported post-contact relative tangential displacement. Rotate memory objectively, then align it to the new normal by a small proper rotation; reject excessive unresolved normal turns. Preserve strain magnitude during pure transport. Never reset a loaded spring because velocity becomes small.

For stiffness `k`, retained strain `e` and new capacity `C = mu * normal_load`, first relax old strain to the new friction disk if unloading reduces its radius. Record the released stored energy explicitly. Form `trial = transported_e + displacement`; return it radially to `|e_new| <= C/k`. Force is `-k * e_new`. Reject actual force/elastic travel exceeding authored hardware limits rather than silently clamping to a structural limit.

This is an authored endpoint/backward-Euler constitutive update. It is not the exact continuous changing-direction Coulomb solution. For pre-relaxed strain `e0`, plastic slip `p = trial - e_new` and displacement `d`, its discrete identity is:

`k * e_new dot d = delta(0.5*k*|e|²) + k*e_new dot p + 0.5*k*|e_new-e0|² + unloading_release`.

Retain Coulomb/plastic dissipation, unloading release, **numerical dissipation**, storage and the actual balance residual separately. Numerical damping cannot be sold as measured material friction. Separation relaxes remaining tangential storage with an explicit release term; the reduced interface assumes irreversible loss at contact loss rather than recovering it elsewhere. Test zero-speed force retention, fresh loading, sliding, reversal, shrinking capacity, separation, changing normals, common rigid rotation and step refinement. No uncontrolled energy grant or hidden history erase is allowed.

## Integration and acceptance still required

- Add a new explicit hardware/world capability version. Original v1 spherical flight and v2 smooth-sliding contact definitions and recordings must remain byte-exact. Optional new fields remain absent in old saves; enabled state has separate model pins and source context.
- Evolve actual foot histories with coupled translation/attitude and the derivative-aware rotating terrain. A fresh-contact displacement must exclude the part before contact. Bound tangential stiffness and changing contact normal in the same finite substep budget; integrate/retain actual force/torque/storage/work residuals.
- Save/reopen/independently replay every history and its numeric exports. Expose typed hardware and live/recorded shear/support state in native controls, with readable student explanations and complete research evidence.
- Establish actual support/load/footprint and local-frame motion evidence before any cargo operation. Static force retention alone does not establish stability, attachment or safe landing.
- Cargo requires a separately retained finite inventory and changing-mass epochs. Preserve immutable original designs, finite propulsion mass/ideal-capability closures, inertia/exhaust assumptions and carried momentum/energy. Transfer only actual available quantities to the same nearby habitat stocks within capacity; rejected material remains aboard. No timer arrival, store grant or silent discard.

The shear component is the current mechanical foundation; it is not yet enabled in world flight. Full integrated release tests, clippy, package/capture checks and an ordinary live session follow integration. Calibration, atmospheric entry, hardware/power/thermal hazards and the full directive remain incomplete. Graphics stay queued.

## Current component evidence

`vehicle_shear.rs` now implements the finite shear/history/work mechanism and actual derivative-aware terrain-frame transport. Eight development tests pass, including an independent implicit supported-mass equilibrium, 1D continuum-work refinement and actual tilted MOLA-frame transport without ephemeris slip. The mass test starts unloaded; its endpoint numerical damping is explicitly retained. These tests do not enable static support in a saved world, validate multi-foot stability or make delivery available. The full release workspace passes **500 tests plus one existing ignored** and release all-target clippy passes. All 171 frozen Rust files match; existing warnings remain with none attributed to the shear component. Evidence is retained under ignored `dist/vehicle-static-foot-support-foundation/`. The accepted packaged terrain flight remains the 492-test `1c0c6e0` checkpoint; no new packaged/visual claim is made for this component.

## Actual inventory constraints found before delivery

Inspection of `supply_vehicle::Cargo` and `habitat::State` confirms matching **reserve** oxygen, potable water, dry food and food-bound water fields. Reserve oxygen is separate from cabin gas; delivered gas must not be injected directly into cabin pressure. Crop harvest stores are also separate from the nominal crew diet. Transferring dry food must not invent a calorie or crew-energy credit beyond the existing authored crew model.

The original habitat configuration has finite initial inventories and equipment-specific product limits, but no general potable-water or crew-food tank capacity. Delivery therefore needs explicit authored storage/handling capacities; treating old numerical bounds as physical tanks or adding unlimited stores would be incorrect. Shared oxygen headroom must respect both crop and electrolysis product limits. Retained rejected cargo stays aboard.

Original spacecraft mass and ideal capability assume immutable cargo. A future transfer must preserve that original definition and introduce actual cargo state plus mass-change/propulsion epochs, rather than rewriting the design and retroactively changing all burn counters. The habitat's recorded heat/crop/breach ledgers have independent closure checks. External transferred mass/energy needs its own retained ledger, and any interaction with those existing closures must be explicit and replayed. Temperature equality or instantaneous isothermal handling is an engineering assumption unless finite thermal/handling equipment is actually modeled.
