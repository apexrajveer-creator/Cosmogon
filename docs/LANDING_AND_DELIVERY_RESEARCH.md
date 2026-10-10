# Landing and finite delivery — next connected mechanisms

Research notes, 2026-10-10. This is not implemented landing or delivery, and does not change the current flight/propulsion models.

The current spacecraft retains first spherical contact with its incoming velocity and finite cargo. A terminal impact record is insufficient evidence for unloading into a geographic habitat. Next builds need actual surface-relative position/velocity, supported terrain/source registration, a defined contact response and retained inventory exchanges. Preserve the verified design bench and versioned flight recordings when adding those mechanisms.

[NASA's Apollo landing-gear experience report, NASA TN D-6850 (1972)](https://ntrs.nasa.gov/api/citations/19720018253/downloads/19720018253.pdf), introductory discussion and landing-performance appendix, treats touchdown as a stability, energy absorption, structural loading and surface-condition problem. This supports implementing contact response and geometry instead of declaring a landing from a low centre-of-mass speed alone. Its hardware-specific envelopes are not universal safe limits for arbitrary authored vehicles.

[NASA SPLICE](https://www.nasa.gov/safe-and-precise-landing-integrated-capabilities-evolution-splice/) describes integrated surface navigation, terrain/hazard assessment and descent guidance. That motivates exposing target-frame observations and measured-versus-modeled support before an automatic descent controller. It does not supply calibration or accuracy guarantees for this application's terrain or vehicle model.

The next mechanism should establish local rotating surface-relative observations, approach geometry, explicit finite touchdown energy/force assumptions and an honest supported contact/attachment regime. Airless descent is a useful connected starting point while atmosphere/entry remains incomplete, but must still include actual gravity, finite burns and surface motion. Do not reinterpret a retained spherical impact as a safe landing, teleport a craft to a site, or replace descent with a timer.

Physical transfer must require supported co-location with the exact habitat body/site and bounded relative motion, withdraw from the same retained craft inventory, respect habitat capacities, retain rejected/excess stock, and apply atomic ordinary edits with exact undo/save/replay. Inspect the actual habitat inventory semantics before mapping all four cargo materials; equal display names are insufficient. Initial authored cargo remains distinct from later stock loading. No implicit settlement grant, waste sink, doubled stock or food/chemical-energy credit.

Keep the graphics supplement queued. These mechanisms need scientific state, controls and numerical evidence first; detailed vessel/planet assets are a separate rendering build. Landing hardware certification, crew safety, atmospheric entry, uncertainty and complete supply-chain acceptance remain unproven.
