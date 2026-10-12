# Cosmogon master engineer prompt — complete handoff

Copy this entire document into the next engineering agent together with repository access. It contains the full original master directive saved in this repository, the owner-approved graphics supplement, and the previous agent’s current-state handoff and build instructions. The owner may also provide the original pasted prompt; if there is any mismatch, preserve all requirements and ask the owner only if they materially conflict. The master directive below remains the product specification. The continuation section supplies current state and sequencing; it does not reduce the scope.

You are taking over the ongoing Cosmogon engineering effort. Continue the full long-term directive to completion through successive integrated, verified builds. Read AGENTS.md and HANDOFF.md in the repository before editing. Building and integrated product progress have priority over separate audits. Preserve useful work, keep the product native, keep the simulation deterministic, be honest about model validity, and persist through the work instead of stopping when one component passes.

The full original master directive follows verbatim from docs/MASTER_DEVELOPMENT_DIRECTIVE.md.

---

# COSMOGON
## FINAL MASTER DEVELOPMENT DIRECTIVE

You are taking over continued development of an existing project called **Cosmogon**.

This document is the primary long-term product and engineering directive for the project.

Treat it as the project's constitutional specification.

It supersedes earlier high-level prompts where they conflict with this document.

Preserve useful existing work.

Preserve implemented features.

Inspect the repository before changing architecture.

Never rewrite something merely because you would have built it differently.

Do not sacrifice a working application for architectural elegance.

Do not pretend unfinished or approximate systems are more accurate than they are.

Cosmogon already has a prototype/base.

Your task is to progressively transform that base into the application described below.

---

# 1. PRODUCT DEFINITION

Cosmogon is intended to become a **native desktop living-universe simulator**.

It combines:

- universe exploration;
- astronomical databases;
- universe sandbox physics;
- planetary simulation;
- planetary surfaces;
- spacecraft engineering;
- orbital mechanics;
- spaceflight;
- space settlements;
- resource management;
- environmental survival;
- geospatial analysis;
- machine-learning-assisted remote-sensing analysis;
- ecosystems;
- evolution;
- civilizations;
- technological development;
- interplanetary expansion;
- interstellar civilization;
- emergent history;
- cinematic universe visualization;
- astrophotography-like creative tools;
- scientific visualization;
- education;
- experimentation;
- research-oriented simulation.

But Cosmogon must NOT feel like fifteen disconnected applications glued together.

There is one universe.

There is one authoritative simulation state.

Everything participates in the same causal system.

The fundamental relationship is:

ASTROPHYSICS  
↓  
ORBITS  
↓  
STELLAR ENVIRONMENT  
↓  
PLANETARY PHYSICS  
↓  
ATMOSPHERE  
↓  
CLIMATE  
↓  
GEOLOGY / RESOURCES  
↓  
BIOSPHERE  
↓  
SETTLEMENTS  
↓  
CIVILIZATIONS  
↓  
TECHNOLOGY  
↓  
SPACEFLIGHT  
↓  
INTERPLANETARY CIVILIZATION  
↓  
INTERSTELLAR CIVILIZATION  
↓  
MEGASTRUCTURES

Changes near the top must be able to propagate downward.

Changes lower in the hierarchy may also affect higher layers where physically meaningful.

Example:

A civilization constructs an enormous Dyson swarm.

That changes:

- intercepted stellar radiation;
- visible stellar luminosity;
- infrared waste heat;
- available energy;
- orbital infrastructure;
- detectable technosignatures.

Cosmogon should understand those effects as part of one simulation.

---

# 2. DEFINING EXPERIENCE

Other space applications commonly answer questions such as:

"What happens to Earth's orbit if Jupiter becomes twice as massive?"

Cosmogon should eventually answer:

"What happens to Earth?"

Meaning:

- orbit;
- Moon;
- tides;
- seasons;
- received solar radiation;
- atmosphere;
- climate;
- oceans;
- ice;
- ecosystems;
- agriculture;
- cities;
- populations;
- economies;
- governments;
- technology;
- spaceflight;
- history.

A single physical intervention may create centuries or millennia of consequences.

That causal continuity is Cosmogon's primary differentiator.

---

# 3. THE SECOND DEFINING ADVANTAGE: CONTROL

Cosmogon's second major competitive advantage should be:

> **Extraordinary graphics combined with extraordinary control.**

The application should be approachable for somebody who simply wants to explore space.

But the same object should expose enough control that an advanced user can inspect and manipulate almost every meaningful quantity affecting it.

The goal is not merely:

"beautiful universe."

It is:

> "beautiful universe with an absurdly capable control system."

Think of the user as potentially being:

- a student;
- an astronomy enthusiast;
- a content creator;
- a photographer;
- a physics student;
- an aerospace engineer;
- a researcher;
- a sandbox player;
- a modder.

The interface must progressively reveal complexity instead of overwhelming everyone with it immediately.

---

# 4. NATIVE APPLICATION — ABSOLUTE REQUIREMENT

Cosmogon is NOT a browser application.

Do not convert it into a browser application.

Do not make WebAssembly, WebGPU, browser storage, browser workers or PWA architecture fundamental dependencies.

Primary targets:

## Windows

Cosmogon executable.

Professional installer.

## macOS

Cosmogon.app.

DMG distribution.

## Linux

Future supported target.

Core simulation should work offline once required datasets are installed.

No Python.

No Node.

No compiler.

No IDE.

No terminal.

No browser.

The user installs Cosmogon and launches it like a high-end simulator or game.

---

# 5. AUDIT BEFORE REWRITE

Before substantial development:

1. Read the repository.
2. Determine languages.
3. Determine renderer.
4. Determine simulation architecture.
5. Determine UI framework.
6. Determine physics implementation.
7. Determine build system.
8. Run tests.
9. Build release mode.
10. Launch application.
11. Profile CPU.
12. Profile GPU.
13. Profile RAM/VRAM.
14. Profile load times.
15. Inspect saves.
16. Catalogue current features.
17. Identify incomplete systems.
18. Identify prototype shortcuts.
19. Create a safe git checkpoint.

Create:

`docs/CURRENT_STATE_AUDIT.md`

---

# 6. ONE AUTHORITATIVE WORLD

Mars in:

Explorer Mode

Settlement Mode

Mission Mode

Civilization Mode

Research Mode

must be the same Mars.

A city exists on a planet.

A settlement occupies actual coordinates.

A spacecraft actually travels between locations.

A mine extracts material from a resource deposit.

A civilization's industry changes visible infrastructure.

Avoid disconnected minigame implementations.

---

# 7. SIMULATION LOD

Never simulate everything at maximum fidelity.

Use levels.

Cosmic scale:
catalogue/statistical representation.

Galactic scale:
catalogues and aggregated dynamics.

Star system:
hierarchical orbital propagation.

Active sandbox:
N-body physics.

Close encounter:
higher-accuracy numerical integration.

Planet:
reduced climate/environment.

Selected region:
high-resolution terrain/environment.

Settlement:
engineering simulation.

Civilization:
regional/macroscopic simulation.

Selected city:
detailed infrastructure.

Focus receives computational fidelity.

---

# 8. PHYSICS MODEL REGISTRY

Create:

`PhysicsModelRegistry`

Every important model stores:

model ID

domain

equations/algorithm

source

units

assumptions

validity range

known limitations

numerical method

expected error

validation cases

fidelity

runtime cost.

Research Mode should expose these.

---

# 9. SCIENTIFIC DATA STATES

Every scientifically relevant value may be classified as:

MEASURED

DERIVED

ESTIMATED

MODELLED

ML INFERENCE

PROCEDURAL

USER MODIFIED

UNKNOWN.

Never fabricate certainty.

---

# 10. SCIENTIFIC SOURCE PRIORITY

Prefer:

1. NASA / JPL / NAIF / NIST / USGS / NOAA / ESA / ESO / IAU and equivalent scientific institutions.
2. Peer-reviewed literature.
3. Authoritative textbooks.
4. Established scientific software/reference implementations.
5. General web resources only for orientation.

Do not implement scientific models from random blogs or unsourced summaries.

---

# 11. ASTRONOMICAL DATA SOURCES

Evaluate and integrate where appropriate:

NASA/JPL Horizons

NAIF SPICE

JPL Small-Body Database

Minor Planet Center

NASA Exoplanet Archive

ESA Gaia

SIMBAD

VizieR

NASA/IPAC NED

SDSS

2MASS

Pan-STARRS

MAST

NASA PDS

USGS Astrogeology.

Build ingestion pipelines rather than manually entering large datasets.

---

# 12. DATA AND ASSET REGISTRY

Maintain:

`data/SOURCE_REGISTRY`

and

`assets/ASSET_REGISTRY`

Track:

source

provider

version

retrieval date

licence

attribution

units

epoch

coordinate system

processing

redistribution permissions.

Nothing with unclear rights enters a release build.

---

# 13. REAL + PROCEDURAL UNIVERSE

Represent the universe using:

OBSERVED OBJECTS

SURVEY-CONSTRAINED STRUCTURE

PROCEDURAL COMPLETION

SIMULATED STATE.

Do not pretend humanity possesses measurements that do not exist.

---

# 14. PLANETARY DATA

Use real data where available.

Primary sources:

NASA PDS.

USGS Astrogeology.

Potential examples:

Mars MOLA

HiRISE

CTX

THEMIS

CRISM

lunar LOLA

LROC

Apollo imagery

mission archives.

Real terrain overrides procedural terrain where appropriate.

Procedural detail fills unresolved scales.

---

# 15. SURFACE IMAGERY

Do not randomly paste mission photographs onto terrain.

Where metadata permits:

place image capture locations correctly;

show mission trails;

build panoramas;

use photogrammetry where practical;

display source metadata.

User should be able to distinguish:

OBSERVED IMAGERY

from

GENERATED DETAIL.

---

# 16. EARTH

Earth should eventually become the highest-data planet.

Potential open layers:

topography

cities

buildings

roads

night lights

land cover

coastlines

population

climate

biomes

transport

infrastructure.

Use appropriately licensed sources.

Real contemporary data becomes the initial condition.

Once simulation diverges, simulated development becomes authoritative.

---

# 17. RENDERING — MAJOR PRODUCT PILLAR

Visual quality is not an afterthought.

Target an exceptionally polished modern desktop application.

Renderer should progressively support:

PBR

HDR

physically based exposure

filmic tone mapping

bloom

temporal anti-aliasing or equivalent

high-quality shadows

contact shadows

volumetric lighting

volumetric fog

planetary atmosphere

clouds

ocean

terrain

nebulae

auroras

rings

city lights

rocket exhaust

re-entry plasma

comet tails

debris

accretion disks

black-hole lensing

large-scale galaxy rendering.

Graphics presets:

LOW

MEDIUM

HIGH

ULTRA

CINEMATIC.

Graphics accuracy and simulation accuracy remain separate.

---

# 18. CAMERA SYSTEM — EXTREMELY IMPORTANT

Cosmogon's camera must be one of the best parts of the application.

Do not treat the camera as merely:

orbit selected object.

Implement multiple camera systems.

## FREE ROAM

The user can detach completely from celestial objects and move anywhere.

Movement:

forward

backward

left

right

up

down

roll

pitch

yaw.

No requirement to remain attached to an orbital plane.

The user can fly:

through a city;

above Mars;

between Saturn's rings;

past a spacecraft;

inside a nebula;

between planets;

outside a galaxy.

Movement speed must adapt across scale.

Fine mode:

centimeters/meters per second if appropriate.

Surface mode:

walking/drone-like scales.

Planetary mode:

km/s.

System mode:

AU-scale movement.

Galactic mode:

light-year scale movement.

Cosmic mode:

large-scale travel.

Provide configurable speed multipliers.

Mouse wheel or similar input may smoothly adjust speed.

---

# 19. CAMERA MODES

Support eventually:

FREE CAMERA

ORBIT CAMERA

TRACK OBJECT

FOLLOW OBJECT

CHASE SPACECRAFT

SURFACE CAMERA

FIRST-PERSON HABITAT CAMERA if eventually useful

DRONE CAMERA

CINEMATIC CAMERA

LOCKED SCIENTIFIC CAMERA

PHOTO CAMERA.

Transition smoothly between modes.

---

# 20. CAMERA PHYSICS

Camera movement is usually a navigation tool rather than a physical spacecraft.

Allow:

INERTIAL CAMERA

and

GAME-LIKE CAMERA

depending on preference.

Possible options:

movement damping

acceleration

deceleration

inertia

instant stop

mouse smoothing

controller curves.

---

# 21. CAMERA COORDINATES

Extreme scale requires:

hierarchical reference frames

double precision

camera-relative rendering

floating origin/origin rebasing where useful.

The free camera must not jitter simply because it is far from coordinate origin.

---

# 22. CAMERA BOOKMARKS

Allow saved viewpoints.

Each bookmark may store:

position

orientation

FOV

exposure

target

simulation time

visual settings.

Examples:

Saturn Ring Sunrise

Earth Night Side

Sagittarius A*

Mars Colony Overview

Jupiter Storm.

---

# 23. CINEMATIC CAMERA PATHS

Eventually support:

keyframes

position interpolation

look-at targets

speed curves

FOV changes

focus changes

time acceleration synchronization.

Allow users to create cinematic flythroughs.

---

# 24. PHOTO MODE

Build a professional:

## COSMOGON PHOTO MODE

Simulation can optionally:

pause

continue

or run slowly.

Photo Mode exposes photographic/presentation controls.

Possible controls:

camera position

camera rotation

focal length

field of view

sensor/aspect simulation where appropriate

focus distance

depth of field

aperture-style control

shutter-style motion blur control

exposure compensation

white balance

contrast

highlights

shadows

saturation

tone curve

bloom strength

lens effects

vignette

chromatic aberration if desired

film grain optional

color grading

LUTs

star exposure

background exposure.

Do not make tacky effects default.

---

# 25. PHYSICAL LIGHTING VS PHOTO LIGHTING

CRITICAL RULE:

Do not silently change simulation physics when user edits photo settings.

There are two distinct concepts:

## PHYSICAL LIGHTING

Actual stellar illumination.

Used for:

rendering baseline

solar power

climate

temperature

physics.

## PRESENTATION / CAMERA EXPOSURE

Changes only what the camera sees.

Example:

Increasing camera exposure must NOT increase solar generation on Mars.

---

# 26. CREATIVE LIGHTING MODE

Allow optional nonphysical lighting tools for creators.

This mode must be clearly labeled:

CREATIVE LIGHTING.

Possible tools:

fill light

rim light

area light

spot light

directional light

ambient intensity

color temperature

light color

intensity

shadow enable

shadow softness.

These exist for photography and cinematics only.

They do NOT affect scientific simulation unless explicitly converted into physical simulation objects.

---

# 27. LIGHTING INSPECTOR

Provide an advanced panel showing:

real light sources

stellar luminosity

distance

incident flux

apparent magnitude where relevant

exposure

camera response

artificial creative lights.

Allow:

PHYSICAL

and

CINEMATIC

tabs.

---

# 28. SCREENSHOT SYSTEM

Support:

normal screenshot

UI-free screenshot

high-resolution screenshot

supersampled screenshot

custom aspect ratios

transparent background where technically appropriate

metadata sidecar.

Potential future support:

8K/16K tiled capture

360-degree panorama

cubemap capture

HDR image output.

---

# 29. SCREENSHOT METADATA

Optionally store:

Cosmogon version

simulation time

object

camera coordinates

FOV

exposure

dataset version

sandbox

visual preset.

This is particularly useful for scientific visualization.

---

# 30. RENDER PRESETS

Provide presets for:

REALISTIC

ASTROPHOTOGRAPHY

HUMAN EYE

CINEMATIC

HIGH CONTRAST

EDUCATIONAL

CUSTOM.

Do not claim astrophotography mode equals a specific telescope unless it actually models that instrument.

---

# 31. SOUND — MAJOR SYSTEM

Audio should receive a major dedicated implementation, not random loops attached to objects.

Create an:

## AUDIO ENVIRONMENT ENGINE

Audio depends on context.

Examples:

vacuum

atmosphere

habitat interior

spacecraft interior

city

surface

weather

industrial environment.

---

# 32. VACUUM AUDIO RULE

Ordinary sound does not propagate through a vacuum.

Therefore:

external physical space should be silent in REALISTIC audio mode.

Do NOT play literal rocket engine noise through vacuum simply because the player is nearby.

However users may want cinematic presentation.

Provide:

REALISTIC

CINEMATIC

HYBRID

audio modes.

---

# 33. CINEMATIC SPACE AUDIO

Cinematic mode may include designed:

low-frequency rumbles

impact effects

rocket sound

stellar ambience

dramatic environmental sound.

Clearly treat these as presentation.

They must not imply scientific acoustic propagation.

---

# 34. WIND AUDIO

Wind sound should respond to physical environmental properties where possible.

Inputs may include:

atmospheric density

wind velocity

pressure

temperature

terrain

camera position

nearby structures.

Mars wind should not simply sound identical to Earth wind.

Dense Venus-like atmosphere should feel different.

Thin Martian atmosphere should be appropriately subdued/filtered.

Research real recorded/synthesized planetary acoustic characteristics where reliable data exists.

---

# 35. SURFACE AUDIO

Possible environmental components:

wind

sand/dust

rain

thunder

water

ice

volcanism

machinery

vehicles

settlement ambience.

They should depend on planet/environment.

---

# 36. HABITAT SOUND

Inside pressurized structures, support:

ventilation

pumps

life-support fans

electrical systems

alarms

doors

airlocks

machinery

footsteps if a first-person view eventually exists.

Sound may become an informational signal.

Example:

failing pump sounds different.

---

# 37. SPACECRAFT SOUND

Inside spacecraft:

engine vibration

RCS vibration

pumps

fans

alerts

structural creaks

docking

stage separation

re-entry vibrations.

External sound follows selected realistic/cinematic mode.

---

# 38. SPATIAL AUDIO

Use modern spatial audio where supported.

Features may include:

distance attenuation

occlusion

reverb zones

interior/exterior transitions

material response

doppler effect where appropriate.

Do not exaggerate Doppler where physically irrelevant.

---

# 39. AUDIO MIXER

Expose category control:

MASTER

MUSIC

UI

ALERTS

ENVIRONMENT

WEATHER

SPACECRAFT

SETTLEMENT

CITY

CINEMATIC EFFECTS

AMBIENCE.

Allow:

mute

solo where useful

volume

dynamic range.

---

# 40. AUDIO PRESETS

Potential presets:

REALISTIC

CINEMATIC

QUIET

IMMERSIVE

CONTENT CREATOR

CUSTOM.

---

# 41. MUSIC

Music should remain optional.

Possible approach:

dynamic atmospheric score based on:

exploration

danger

civilization milestone

disaster

discovery.

Users must be able to disable music completely.

Do not let music overwhelm scientific/environment audio.

---

# 42. ALERT SYSTEM

Alerts must be powerful but NEVER annoying.

Do not spam the user's screen.

Build:

## NOTIFICATION CENTER

Event levels:

INFORMATION

NOTICE

WARNING

CRITICAL.

Categories:

physics

spacecraft

settlement

civilization

climate

dataset

system

research

mission

construction.

---

# 43. ALERT CUSTOMIZATION

For every category users should eventually be able to select:

popup

notification center only

sound only

timeline only

pause simulation

no notification.

Allow severity thresholds.

Example:

Civilization routine events:
Chronicle only.

Settlement oxygen emergency:
Critical popup + alarm + optional pause.

---

# 44. DO NOT DISTURB

Support:

DO NOT DISTURB

CINEMATIC MODE

PHOTO MODE

STREAMER MODE.

These can suppress screen notifications.

Critical simulation information should remain accessible in the Notification Center.

---

# 45. ALERT SOUNDS

Allow:

all alert sounds off

critical only

custom volume

category-specific sounds.

No mandatory pings.

---

# 46. ALERT HISTORY

Notifications do not disappear forever.

Maintain searchable notification history.

Allow user to jump to:

object

event

location

Chronicle entry.

---

# 47. CLEAN UI — ABSOLUTE PRIORITY

Cosmogon must avoid the classic simulator problem:

500 buttons visible simultaneously.

The UI must be:

clean

modern

calm

fast

discoverable

high information density only when requested.

---

# 48. PROGRESSIVE DISCLOSURE

A beginner initially sees:

object name

basic data

important actions.

An expert may expand:

Advanced.

A researcher can open:

Research.

Do not display tensors, orbital state vectors, thermodynamic coefficients and civilization graphs to someone simply looking at Saturn.

---

# 49. UI MODES

Support:

## SIMPLE

Minimal controls.

## ADVANCED

Precise numerical editing and charts.

## RESEARCH

Complete model/data/provenance control.

## CINEMATIC

Minimal overlays.

## CUSTOM

User-created layout.

Underlying simulation remains the same.

---

# 50. WORKSPACES

Provide workspaces:

EXPLORER

SANDBOX

PHYSICS LAB

PLANET LAB

SETTLEMENT

SPACECRAFT DESIGNER

MISSION CONTROL

CIVILIZATION

RESEARCH

PHOTO

CINEMATIC.

Users may save their own.

---

# 51. DOCKABLE INTERFACE

Eventually support:

dock

undock

resize

reorder

tabs

floating panels

collapse

auto-hide

multiple monitors.

Save layout per user.

---

# 52. CONTROL PANEL — SIGNATURE FEATURE

Build a universal:

## COSMOGON CONTROL PANEL

This is one of the project's most important differentiators.

When the user selects something, Cosmogon should expose every meaningful editable quantity that the model permits.

The panel should feel like a combination of:

professional simulation software

mission-control software

a modern game inspector

a scientific instrument.

---

# 53. CONTROL PANEL STRUCTURE

Typical sections:

BASIC

PHYSICAL

ORBIT

ROTATION

COMPOSITION

ATMOSPHERE

THERMAL

MAGNETIC

GEOLOGY

CLIMATE

SURFACE

RESOURCES

BIOSPHERE

CIVILIZATION

SPACECRAFT

VISUAL

AUDIO

DATA

RESEARCH

depending on selected object.

---

# 54. EVERY PARAMETER CONTROL

A research-level parameter control may include:

name

current value

unit

slider where appropriate

exact text input

scientific notation

uncertainty

source

derived/manual status

lock

reset

formula

dependency graph

valid range

warning range

history graph.

---

# 55. DERIVED VALUES

Example:

Planet mass and radius determine surface gravity.

If gravity is derived:

show lock icon.

If user changes mass:

gravity updates.

If user manually overrides gravity:

mark:

USER OVERRIDE.

Show warning that state may no longer correspond to ordinary Newtonian mass/radius relationships.

---

# 56. PARAMETER DEPENDENCY GRAPH

Research Mode should expose:

WHAT AFFECTS THIS?

and

WHAT DOES THIS AFFECT?

Example:

Atmospheric Pressure

affected by:

atmospheric mass

surface gravity

planet radius.

affects:

boiling point

weather

aerodynamics

habitability

sound propagation

settlement design.

This can become an extremely powerful educational and debugging tool.

---

# 57. EVERYTHING EDITABLE WHERE MEANINGFUL

Users should eventually be able to edit essentially all meaningful exposed simulation parameters.

Examples:

planet mass

star metallicity

atmospheric CO2

black-hole spin

comet volatile fraction

spacecraft tank pressure

habitat scrubber capacity

civilization research investment

climate parameters

solver timestep

collision fidelity.

Do not expose dangerous internal implementation variables that would simply corrupt memory or crash the application.

"Everything" means simulation-relevant state, not raw engine internals.

---

# 58. BATCH EDIT

Advanced users should be able to select multiple compatible objects and batch-edit.

Example:

select 100 asteroids.

Set:

density distribution

albedo

initial velocity perturbation.

Useful for experiments.

---

# 59. RANDOMIZATION TOOLS

Allow controlled randomization.

Example:

Generate 100 Earth-like planets with:

Mass:
0.8–1.2 Earth.

Atmospheric CO2:
distribution.

Seed:
12345.

Research Mode records the distributions and seed.

---

# 60. PARAMETER PRESETS

Allow save/load presets.

Examples:

Early Earth

Modern Earth

Snowball Earth

Terraforming Mars

Hot Jupiter

Earth-like atmosphere

Industrial civilization

Closed-loop Mars base.

---

# 61. COMPARE MODE

Select multiple objects and compare:

mass

radius

density

atmosphere

orbit

climate

life

civilization

etc.

Allow synchronized graphs.

---

# 62. LIVE GRAPHS

Research panel should support live plotting.

Examples:

Earth eccentricity vs time

settlement oxygen

battery SOC

global temperature

population

atmospheric CO2

spacecraft velocity

fuel

radiation dose.

Allow multiple series.

---

# 63. GRAPH INSPECTOR

Graphs should support:

zoom

pan

cursor

units

log scale

export CSV

save layout

event overlays.

Example:

display asteroid impact as vertical marker on global-temperature graph.

---

# 64. EXPRESSION / FORMULA VIEW

Where practical, show the actual model.

Example:

Surface gravity:

g = GM/R².

Show:

input values

units

result.

Education Mode explains it.

Research Mode exposes full model references.

---

# 65. RESEARCH GRADE DOES NOT MEAN UNREADABLE

The Research Panel must be powerful without looking like legacy 1990s engineering software.

Maintain:

clear typography

spacing

search

filtering

favorites

collapsible sections

breadcrumbs

keyboard navigation.

---

# 66. PARAMETER SEARCH

Within Control Panel:

search:

"CO2"

and instantly find every relevant CO2 parameter.

Search:

"radiation"

and find relevant settings.

---

# 67. FAVORITES

Users can star parameters.

Example custom research panel:

Earth eccentricity

solar luminosity

CO2

global temperature

population

food production.

---

# 68. CONTROL PANEL HISTORY

Parameter edits support:

undo

redo

edit history.

Show:

what changed

old value

new value

simulation time.

---

# 69. RESEARCH EXPERIMENT MODE

Allow:

freeze initial state

define independent variables

define measurements

run simulation

record output.

Eventually enable:

parameter sweep.

Example:

Jupiter mass:
0.5–2.0 Jupiter masses.

Measure:

Earth orbital eccentricity after 1 million years.

---

# 70. EXPERIMENT BATCHING

Long-term Research Mode may run experiments headlessly or accelerated.

Example:

100 seeds.

100 asteroid impact angles.

Different habitat designs.

Record outcomes.

This could eventually make Cosmogon genuinely useful for exploratory studies.

Clearly distinguish exploratory simulation from validated scientific research.

---

# 71. COMMAND PALETTE

Ctrl/Cmd + K.

Commands:

Go to Mars.

Create asteroid.

Show gravity vectors.

Open Research Panel.

Enter Photo Mode.

Mute alerts.

Set simulation speed.

Change exposure.

Focus settlement.

Save experiment.

---

# 72. OBJECT INSPECTOR

Relevant tabs:

OVERVIEW

PHYSICS

ORBIT

COMPOSITION

ATMOSPHERE

CLIMATE

SURFACE

LIFE

CIVILIZATION

SETTLEMENT

SPACECRAFT

VISUAL

AUDIO

HISTORY

DATA

RESEARCH.

Only show applicable tabs.

---

# 73. WHY BUTTON

Universal:

WHY?

Example:

Why did this colony lose power?

Cause graph:

Dust storm

→ solar irradiance reaching panels decreased

→ solar output decreased

→ batteries discharged

→ critical threshold reached.

Or:

Why is Earth freezing?

Orbit changed.

Stellar flux decreased.

Ice expanded.

Albedo rose.

Positive ice feedback occurred.

---

# 74. CHRONICLE

Record meaningful history.

Examples:

impact

discovery

city founded

factory built

first aircraft

first satellite

first colony

mass extinction

terraforming milestone

civilization collapse

first contact

stellar death.

Do not record meaningless per-frame noise.

---

# 75. OBJECT CREATION

Allow creation of:

stars

brown dwarfs

white dwarfs

neutron stars

black holes

planets

moons

asteroids

comets

rings

debris

spacecraft

stations

megastructures.

Simple presets + advanced controls.

---

# 76. SANDBOX PHYSICS

Support progressively:

Keplerian propagation

N-body gravity

close encounters

collisions

fragmentation

Roche effects

tides

comets

meteors

impacts

radiation

selected relativistic effects.

---

# 77. SETTLEMENT SIMULATION

Settlements physically exist on worlds.

Model:

power

air

water

temperature

life support

food

radiation

resources

construction

maintenance

industry

logistics.

No "+10 oxygen building" logic without underlying meaning.

---

# 78. HABITAT EQUATIONS

Use differential/state equations where appropriate.

Atmosphere:

dM_i/dt.

Temperature:

C dT/dt.

Battery:

dE/dt.

Water:

dW/dt.

Every model documented.

---

# 79. SITE ANALYSIS

GIS-style layers:

terrain

slope

illumination

resources

radiation

temperature

communications

landing safety.

Use real planetary data where possible.

---

# 80. ML GEOSPATIAL SYSTEM

Use ML for:

hazard detection

terrain segmentation

crater detection

route planning

site ranking

resource inference

building detection.

Never replace well-known physics with ML.

Expose confidence and provenance.

---

# 81. SPACECRAFT DESIGN

Spacecraft should expose:

mass

propellant

tanks

engines

thrust

Isp

thermal

power

aerodynamics

shielding

communications

payload.

Tradeoffs must be physical.

---

# 82. SPACEFLIGHT

Support:

launch

staging

orbit

transfer

gravity assist

rendezvous

docking

entry

landing

supply missions.

Visual effects read real vehicle/environment state.

---

# 83. CIVILIZATIONS

Civilizations contain:

population

settlements

economy

knowledge

technology

resources

industry

infrastructure

space capability

history.

Avoid simulating every individual person.

---

# 84. VISIBLE CIVILIZATION PROGRESS

Progress must physically appear.

Agriculture:
farms.

Industry:
factories.

Electricity:
grids and lights.

Automobiles:
roads.

Aviation:
airports.

Space age:
spaceports and satellites.

Interplanetary:
orbital traffic and colonies.

Megastructure era:
stellar-scale infrastructure.

---

# 85. TECHNOLOGY

Use causal technology graphs.

Prerequisites include:

knowledge

materials

manufacturing

energy

existing systems

motivation

observations.

No arbitrary timer-only unlocks.

---

# 86. INTERSTELLAR CIVILIZATION

Eventually:

generation ships

colonies

civilization divergence

communication delays

first contact

terraforming

Dyson swarms

orbital rings

major megastructures.

---

# 87. RESEARCH MODE

Expose:

state vectors

frames

models

equations

solver

timestep

tolerance

units

uncertainty

source

dataset version

seed

model version.

Export:

CSV

JSON

plots

event logs

trajectories

reproducibility manifest.

---

# 88. EDUCATION MODE

Simplify interface, not physics.

Example:

"This comet is travelling too quickly to remain bound to the Sun."

SHOW THE MATH.

Then display the actual quantities.

---

# 89. PERFORMANCE

Use:

multithreading

job system

LOD

streaming

spatial indexes

GPU instancing

compute shaders where useful

asynchronous IO

compressed assets.

Never load all data simultaneously.

---

# 90. DATA PACKS

Possible optional packs:

Core Solar System

High Resolution Earth

Mars HD

Moon HD

Gaia Nearby Stars

Full Stellar Catalogue

Exoplanets

Mission Archive.

Core application remains usable without every optional dataset.

---

# 91. MODDING

Eventually support structured mods:

objects

materials

habitats

spacecraft

scenarios

civilizations

technologies

datasets

visual themes.

Do not execute untrusted native code casually.

---

# 92. SCENARIOS

Examples:

Build First Moon Base

Build Mars Colony

Apollo 11

Perseverance

Voyager

Move Earth

Second Moon

Double Jupiter

Chicxulub Today

Rogue Planet

Terraform Mars

Dyson Swarm

Sun Becomes Red Giant

Survive Nearby Supernova.

Scenarios use the real systems.

---

# 93. CAMERA + PHOTO SCENARIOS

Include exploration/photo presets such as:

Saturn Ring Flythrough

Earth Sunrise From Moon

Jupiter Storm

Mars Sunset

Black Hole Accretion Disk

Supernova Observation

Milky Way Exterior

Dyson Swarm Eclipse.

These are camera starting points, not fake rendered videos.

---

# 94. ACCESSIBILITY AND CONTROL CUSTOMIZATION

Support:

keyboard rebinding

mouse sensitivity

invert axes

controller support where appropriate

UI scaling

font scaling

high contrast

reduced motion

colorblind-safe visualization where relevant.

---

# 95. ALERT ACCESSIBILITY

Alerts must support:

visual only

audio only

both

neither

different severity colors

screen-reader-friendly text where architecture supports it.

---

# 96. DOCUMENTATION

Maintain:

README.md

ARCHITECTURE.md

ROADMAP.md

CHANGELOG.md

BUILDING.md

docs/CURRENT_STATE_AUDIT.md

docs/ENGINE_ASSESSMENT.md

docs/PRODUCT_VISION.md

docs/RENDERING.md

docs/CAMERA_SYSTEM.md

docs/PHOTO_MODE.md

docs/AUDIO_SYSTEM.md

docs/NOTIFICATION_SYSTEM.md

docs/UI_UX.md

docs/CONTROL_PANEL.md

docs/PHYSICS_MODEL_REGISTRY.md

docs/ASTRONOMICAL_DATA.md

docs/PLANETARY_DATA.md

docs/SETTLEMENT_SIMULATION.md

docs/SPACECRAFT_SIMULATION.md

docs/CIVILIZATION_MODEL.md

docs/TECHNOLOGY_MODEL.md

docs/GEOSPATIAL_ML.md

docs/SAVE_FORMAT.md

docs/VALIDATION.md

docs/PERFORMANCE.md

docs/DATA_SOURCES.md

docs/ASSET_LICENSES.md.

---

# 97. DEVELOPMENT ORDER

Do NOT attempt everything simultaneously.

Use vertical slices.

---

# 98. PHASE 0 — AUDIT

Complete current release.

Tag stable version.

Run tests.

Audit.

Profile.

Document current architecture.

---

# 99. PHASE 1 — NATIVE ENGINE FOUNDATION

Ensure:

native builds

resource manager

dataset manager

simulation/render separation

job system

logging

save directories

performance instrumentation.

---

# 100. PHASE 2 — PREMIUM APPLICATION SHELL

Implement:

loading sequence

Home

Continue

New Sandbox

Real Universe

My Sandboxes

Settings

save/load

autosave.

Build the UI workspace architecture now.

---

# 101. PHASE 3 — CAMERA + UX FOUNDATION

This must happen EARLY rather than being postponed.

Implement:

free roam camera

orbit camera

tracking

smooth scale-aware movement

bookmarks

camera transitions

input configuration.

Also implement the initial clean:

Control Panel

Inspector

Notification Center

Command Palette.

This ensures every later feature uses the intended UX.

---

# 102. PHASE 4 — PHOTO / CINEMATIC FOUNDATION

Implement initial:

Photo Mode

physical vs presentation exposure separation

FOV/focal controls

DOF

exposure

white balance

color grading

UI hiding

high-resolution screenshot

camera bookmarks.

Add Creative Lighting only after physical/presentation separation is architecturally clear.

---

# 103. PHASE 5 — AUDIO FOUNDATION

Implement:

Audio Environment Engine

realistic/cinematic audio modes

audio mixer

surface atmosphere-based ambience

wind prototype

spacecraft interior prototype

settlement interior prototype

fully configurable alerts.

No mandatory notification sound.

---

# 104. PHASE 6 — SOLAR SYSTEM SANDBOX

Implement:

real initial conditions

clone to sandbox

object editor

N-body

launch tool

trajectory prediction

collision foundation

undo/redo.

Demonstration:

Create an Earth-mass planet.

Throw it through Solar System.

Watch gravity operate.

Inspect precise state in Research Panel.

---

# 105. PHASE 7 — HIGH-END PLANET RENDERING

Implement:

planet LOD

terrain

atmosphere

surface materials

ocean

cloud foundation

surface navigation.

Mars first major target.

---

# 106. PHASE 8 — MARS REAL-DATA SLICE

Real Mars terrain.

Surface travel.

Mission locations.

Real-data layers.

Photo Mode.

Free-camera flythrough.

This milestone should already be visually impressive enough for promotional footage.

---

# 107. PHASE 9 — MARS SETTLEMENT SLICE

Build habitat.

Power.

Battery.

Water.

Oxygen.

CO2.

Thermal state.

Greenhouse.

Hazard.

Chronicle.

Control Panel exposes engineering parameters.

Research Panel exposes equations.

Audio reflects habitat state.

Alerts are configurable.

---

# 108. PHASE 10 — SPACECRAFT SUPPLY CHAIN

Design supply vehicle.

Launch.

Orbit.

Transfer.

Approach.

Entry.

Landing.

Cargo reaches settlement.

Camera can freely follow from any perspective.

Photo Mode captures launch/re-entry/landing.

---

# 109. PHASE 11 — PHYSICAL CONSEQUENCE CHAIN

Orbit change.

Climate change.

Environment change.

Civilization response.

Chronicle explains it.

WHY panel explains causal path.

---

# 110. PHASE 12 — CIVILIZATION VISUALIZATION

Cities.

Roads.

Industry.

Lights.

Spaceports.

Ships.

Urban growth.

Technological visual transformation.

---

# 111. PHASE 13 — REAL UNIVERSE DATA PLATFORM

Catalogue importer.

Spatial index.

Streaming.

Data packs.

Gaia.

Exoplanets.

SIMBAD/VizieR as appropriate.

Extragalactic data.

---

# 112. PHASE 14 — EXOTIC UNIVERSE

Stellar evolution.

Supernovae.

Neutron stars.

Black holes.

Nebulae.

Advanced visual effects.

---

# 113. PHASE 15 — INTERSTELLAR CIVILIZATION

Colonization.

Branches.

First contact.

Terraforming.

Megastructures.

---

# 114. PHASE 16 — RESEARCH TOOLING

Parameter sweep.

Experiment definitions.

Advanced graphing.

Comparisons.

Exports.

Reproducibility.

Model validation.

---

# 115. PHASE 17 — AUDIO / CINEMATIC POLISH

Massive polish pass:

environmental sound

spacecraft interior

habitats

city ambience

weather

cinematic audio

photo tools

cinematic camera paths

high-res captures

visual presets.

---

# 116. PHASE 18 — UI / CONTROL POLISH

The UI deserves a dedicated completion phase.

Audit every feature.

Ensure:

Simple Mode remains clean.

Advanced Mode is powerful.

Research Mode is extraordinarily capable.

No feature requires memorizing obscure controls.

Control Panel search works.

Dependencies are visible.

Notification configuration is complete.

Layouts persist.

Multi-monitor works where supported.

---

# 117. PHASE 19 — PRODUCTIZATION

Windows installer.

macOS app/DMG.

Linux later.

Release pipeline.

Data-pack manager.

Third-party notices.

No development dependencies.

---

# 118. FEATURE DEFINITION OF DONE

Do not say:

"Comets complete"

because a particle trail was attached.

A properly implemented domain requires as applicable:

simulation

rendering

UI

Research Panel

save/load

data provenance

tests

validation

performance

documentation.

---

# 119. CAMERA DEFINITION OF DONE

Free camera is not complete merely because WASD works.

It must handle:

surface scale

planet scale

system scale

galactic scale

precision

smooth movement

input customization

focus transitions

bookmarks

photo mode

large-coordinate stability.

---

# 120. AUDIO DEFINITION OF DONE

Audio is not complete because there is a background MP3.

Require:

context-aware environment

mixer

realistic/cinematic modes

spatial behavior

alert customization

interior/exterior transitions

settings persistence.

---

# 121. CONTROL PANEL DEFINITION OF DONE

Research control is not complete because values appear in text boxes.

Require:

units

exact editing

validation

derivation status

source

history

reset

undo

dependencies

graphs where useful

provenance

warnings.

---

# 122. GRAPHICS DEFINITION OF DONE

A rendering feature must work:

near

far

different exposures

different stars

different worlds

different graphics presets

various resolutions

photo mode

cinematic mode.

Profile it.

---

# 123. SCIENCE DEFINITION OF DONE

A scientific model requires:

source

units

assumptions

validity range

tests

benchmark

known error/limitations.

---

# 124. COSMOGON'S CENTRAL PRODUCT EQUATION

The competitive thesis is:

## SUPER GRAPHICS
+
## SUPER CONTROL
+
## DEEP PHYSICS
+
## LIVING CIVILIZATIONS
+
## REAL SCIENTIFIC DATA
+
## ACCESSIBLE UX

No competitor advantage should be pursued in isolation.

Beautiful graphics without control becomes a viewer.

Control without visual quality becomes engineering software.

Physics without consequences becomes a toy.

Civilization without physics becomes a strategy game.

Research controls without accessibility become intimidating.

Cosmogon should unify them.

---

# 125. TARGET EXPERIENCE

Launch Cosmogon.

Home appears.

Choose:

REAL UNIVERSE.

Search Mars.

Free camera flies seamlessly from the Milky Way toward the Solar System.

Approach Mars.

Surface data loads.

Actual terrain resolves.

Switch to Photo Mode.

Adjust:

camera focal length

exposure

white balance

depth of field.

Mars sunset fills the frame.

No simulation physics changed.

Take an 8K screenshot.

Exit Photo Mode.

Fly freely down toward Jezero.

Mission markers appear.

Select Perseverance.

Real mission information is available.

Clone Mars into:

"Mars Colony Research."

Enter Settlement workspace.

Control Panel opens.

Use real terrain layers.

Run site analysis.

Build habitat.

Solar farm.

Battery.

Water recovery.

Oxygen generation.

Greenhouse.

Open Research Mode.

Inspect:

O2 mass-balance equation.

Battery charge curve.

Thermal model.

Radiation assumptions.

Wind begins outside.

Atmosphere-aware audio reflects Martian conditions.

Inside habitat, environmental audio changes.

A dust event begins.

Solar output falls.

Alert system records:

WARNING — Solar generation decreased.

User configured this category not to create popups.

No intrusive notification appears.

The Notification Center receives the event.

Battery falls.

Oxygen production throttles.

A critical threshold is reached.

Critical warning is configured to:

sound alarm

show popup

pause simulation.

Cosmogon does so.

The user clicks:

WHY?

Cause graph appears:

dust

→ reduced panel irradiance

→ reduced power

→ battery depletion

→ oxygen generation throttled.

The user changes battery capacity through the Research Control Panel.

Simulation resumes.

Later create a supply spacecraft.

Follow launch from free camera.

Switch to tracking.

Then cinematic camera.

Capture stage separation.

Fly beside the craft.

In realistic sound mode, exterior space becomes silent.

Switch to cinematic audio if desired.

Travel to Mars.

Land.

Settlement grows.

Factories become visible.

Roads expand.

Cities form.

Centuries pass.

Mars civilization develops.

Zoom to orbit.

Night-side artificial lighting is visible.

Fly freely through orbital traffic.

Then clone Solar System.

Create rogue planet.

Open advanced control panel.

Mass:
2 Earths.

Position:
custom.

Velocity:
precisely defined state vector.

Throw it.

Gravity acts.

Earth orbit changes.

Climate responds.

Civilization reacts.

Chronicle records centuries of consequences.

Open Research workspace.

Plot:

Earth eccentricity

solar flux

global temperature

crop output

population

over the same timeline.

Impact markers and events appear on graph.

Export CSV.

Switch to Cinematic Mode.

Hide UI.

Set camera path.

Adjust presentation exposure.

Mute alerts.

Enable cinematic audio.

Capture a film-quality view of the altered Solar System.

Cosmogon can be both:

a playground

and

an instrument.

That is the intended application.

---

# 126. FINAL EXECUTION DIRECTIVE

Do not respond to this document only with enthusiasm.

Do not make mockups instead of systems.

Do not attempt everything simultaneously.

Do not destroy working code.

Do not fake scientific capability.

Do not spam the user with UI.

Do not couple presentation controls to physical simulation.

Do not force alerts.

Do not make scientific depth inaccessible.

Proceed as follows:

audit existing project;

research required systems;

identify what already exists;

write gap analysis;

define architecture;

preserve working features;

implement the next coherent vertical slice;

test it;

validate it;

profile it;

integrate UI;

integrate Control Panel;

integrate save/load;

integrate sound where appropriate;

document it;

produce a working release build;

continue.

The ambition is:

> **A native simulated universe where a user has extraordinary freedom to explore, modify, build, research, understand and photograph reality — combining top-tier visuals with an unusually deep research-grade control system over the universe itself.**

Build toward that.
---

# Owner graphics goal supplement (full text)

# Owner graphics goal supplement — 2026-10-09

This requirement extends the active development goal and the rendering, planetary exploration and civilization phases of the master directive. The owner explicitly asked to keep the systematic cabin/research build sequence first and defer graphics implementation.

Every cosmic body the user can visit should eventually support exceptionally detailed close-up rendering in Cosmogon's own style, comparable in richness to the supplied stellar reference. Apply this across stars, planets, moons, small bodies, rings, atmospheres and other visitable environments. Detail should remain coherent from distant navigation to close inspection and surface exploration.

Activity must visibly change during play. It can be simulated/procedural and need not reproduce present-day observations. Stars should show appropriate surface and atmospheric activity; applicable worlds should show terrain, mountains, water, clouds, hurricanes and other weather, volcanism, earthquakes and their visible consequences, buildings and civilizations. Avoid applying Earth-specific features to worlds that do not support them. Existing physical/environment/civilization state should drive presentation where implemented, while designed animation stays explicitly distinguishable from simulated physical events and observed data.

Use the attached Sun image as a visual richness reference, not a literal scene template, scientific map or new claim about the Sun. Preserve the product's visual identity, readable research views, deterministic simulation boundaries and real-versus-procedural provenance. Stream or scale detail by viewing distance, visibility and hardware budgets; retain stable transitions and responsive wide-scale travel. Verification requires real app captures and measured performance at multiple scales, with no claim that all bodies meet this target until demonstrated.

Status: **recorded and queued; no graphics implementation in this checkpoint**. Continue the systematic cabin temperature and research builds first; recorded figures and inventory/electrical balances have been checkpointed. The full target spans multiple integrated builds and remains incomplete until the visitable body families and applicable activity are covered.

---

# Previous agent continuation handoff (full text)

# Cosmogon continuation handoff

Prepared 2026-10-11 (Asia/Kolkata) for the next coding agent. The owner will provide the original master prompt separately. Read that prompt as the product specification, this repository's [`AGENTS.md`](../AGENTS.md), [`HANDOFF.md`](../HANDOFF.md), and [`docs/MASTER_DEVELOPMENT_DIRECTIVE.md`](MASTER_DEVELOPMENT_DIRECTIVE.md) before editing. This note transfers current implementation state and the work order; it does not narrow or replace the directive.

## Mission and working style

Continue the existing Cosmogon product in integrated, user-visible or research-usable builds. Implement substantial connected slices, verify them against the actual application and authoritative simulation, record evidence, and move directly to the next dependency. The owner explicitly prioritizes building over separate audits. Do not stop after one component or treat a passing test suite as the end of the product. Research primary sources when physical or scientific claims depend on them; separate source facts, engineering inference, model assumptions and unvalidated limits. Student-to-researcher accessibility and trustworthy, reproducible results both matter.

The owner also added [`GRAPHICS_GOAL_SUPPLEMENT.md`](GRAPHICS_GOAL_SUPPLEMENT.md): every visitable cosmic body should eventually have highly detailed, animated close-up presentation in Cosmogon's style. Procedural detail and activity are allowed if clearly labeled; examples include stellar activity and planetary terrain, weather, water, volcanoes, earthquakes and civilization. The owner explicitly queued graphics while the habitat/research/supply engineering work continued. Keep this as a real product requirement and return to it in the rendering/data work; do not silently drop it or claim it is already implemented.

## Current checkout and verified state

- Repository: `/Users/flamexmystix/Documents/Sidequests/Language by Projects/full blown projects/SpaceRender`
- Branch: `codex/finite-world-flight`
- Latest committed baseline: `c6686926d7f86eefec97e408e0e01c4d7ed7b111` — actual vehicle footprint and ground-hold research reports.
- The committed baseline has an accepted native package and complete evidence in ignored `dist/vehicle-footprint-support-validation/`. That evidence applies to the committed 173-Rust-file baseline, **not to the current uncommitted source**.
- Current uncommitted cargo-mass work adds `crates/cosmogon_sim/src/vehicle_cargo_mass.rs`, exports the module from `crates/cosmogon_sim/src/lib.rs`, and adds `docs/CARGO_MASS_EPOCHS.md`. Preserve and continue this work; it belongs to the user-authorized project. It is not yet committed, built into a new app/CLI release binary, integrated into authoritative world-flight state, or wired to cargo transfer.
- Verification on the current 174-Rust-file source snapshot completed: `cargo test --release --workspace` exited 0 with **522 passed, 1 existing ignored** (including 301 simulation tests and all 8 cargo-mass tests); `cargo clippy --release --workspace --all-targets` exited 0. Clippy reports existing warnings in untouched application code; no cargo module failure was reported. The frozen inventory is `/tmp/cosmogon-cargo-source.json`; logs and exact command durations are `/tmp/cosmogon-cargo-tests.log`, `/tmp/cosmogon-cargo-clippy.log`, and `/tmp/cosmogon-cargo-checks.json`.
- A targeted cargo test earlier also exited 0 (8 passed). No new `cargo build --release -p cosmogon -p cosmogon_cli`, native/package/signature/DMG check, visual capture or ordinary live-app run was made for the uncommitted 174-file source. Do not imply that the current source has those checks. The latest packaged visual evidence is for c668692.
- No push, tag or release was made. No other working-tree changes were present at handoff time.

The test totals reconcile to the workspace output: app 127, CLI 23, core 32, physics 38, simulation 301, and one physics doctest = 522 passed; one pre-existing ignored app test. The workspace-wide check was run after the cargo module was added, so it does cover the current module.

## Cargo mass epoch work already implemented

The new standalone `vehicle_cargo_mass` simulation module defines `cosmogon-vehicle-cargo-mass-epochs/1`. It keeps the original validated vehicle definition immutable and separately records held oxygen reserve, potable water, dry food and food-bound water. A bounded removal ledger records clock, remaining propellant, requested and actually represented species quantities, before/after inventories, mass residual, and co-moving linear momentum/translational kinetic energy. It refuses overdraw, negative/nonfinite/tiny-unrepresented requests, too many events and removals below total-mass floating-point resolution atomically.

Future ideal propulsion increments are recomputed piecewise across actual fuel and cargo epochs. Prior burns retain their original denominators; cargo removal itself grants no rocket impulse and is not mislabeled as exhausted fuel. The module can apply a later finite burn against the held cargo mass without changing original design inputs. The tests independently check multiple burn/removal epochs, momentum closure, conservative per-species rounding, strict serialized corruption and design identity, legacy-equivalent no-removal behavior, event bounds, precision refusal and finite-fuel exhaustion.

Important limits: this module is not yet a field on `world_vehicle::Flight`; the world contact solver, rigid-body mass/inertia, angular momentum, save pins, flight recordings, CLI and UI do not yet use it. Its central co-moving removal has no rotational/inertia/distribution, plumbing impulse, heat, recipient, handling or habitat exchange model. It performs **no world delivery** and creates no settlement stocks. Its README/memo language should be kept precise. The current research memo describes this as a component in progress, not a completed supply chain.

## How far the original directive has been carried

Use the active phase table and numbered build history in [`MASTER_GAP_ANALYSIS.md`](MASTER_GAP_ANALYSIS.md) as the detailed checklist. At a high level, phases 0–8 and 16 have broad, working foundations (the application/simulation, solar sandbox, navigation/photo/audio/research, Mars data and native inspection), with material gaps in validation, datasets, accessibility and quality. Phase 9 has a substantial but explicitly reduced habitat/greenhouse/reaction/recording/experiment chain; field-calibrated life support, construction/logistics and other environment systems remain open. Phase 10 has finite authored vehicle design, world gravity/burns, approach, terrain/contact/shear and the latest footprint/instantaneous support evidence. **The directive has not reached sustained surface operations, attachment, cargo unloading, habitat delivery, full launch/entry/landing or an end-to-end supply mission.** Phases 11–15 and 17–19 have foundations, not their complete directive scope: validated consequences/history, visible civilization infrastructure, real-catalogue streaming, per-model exotic validation, richer interstellar consequences, high-end UX/accessibility/distribution and the graphics supplement remain substantial work. The master directive is not complete. This is a phase-level orientation, not a claim that every individual master requirement was re-audited in this handoff.

## Accepted vehicle sequence and useful files

The supply chain is the current active product priority. Inspect, in this order:

1. [`VEHICLE_SUPPORT_NEXT.md`](VEHICLE_SUPPORT_NEXT.md) — immediate dependency sequence and safety/accounting invariants for eventual delivery.
2. [`VEHICLE_FOOTPRINT_SUPPORT.md`](VEHICLE_FOOTPRINT_SUPPORT.md) and [`VEHICLE_FOOTPRINT_SUPPORT_VALIDATION.md`](VEHICLE_FOOTPRINT_SUPPORT_VALIDATION.md) — latest accepted support reading, replay/export and verification boundaries.
3. [`VEHICLE_STATIC_SHEAR.md`](VEHICLE_STATIC_SHEAR.md), [`VEHICLE_CONTACT.md`](VEHICLE_CONTACT.md), [`LANDING_AND_DELIVERY_RESEARCH.md`](LANDING_AND_DELIVERY_RESEARCH.md), and [`STATIC_FOOT_SUPPORT_DESIGN.md`](STATIC_FOOT_SUPPORT_DESIGN.md) — preceding mechanics, research and limits.
4. Code: `crates/cosmogon_sim/src/world_vehicle.rs`, `vehicle_contact_dynamics.rs`, `vehicle_arrival.rs`, `vehicle_support.rs`, `vehicle_support_recording.rs`, `world_vehicle_recording.rs`, `supply_vehicle.rs`, `habitat.rs`, `save.rs`; app controls are primarily in `crates/cosmogon/src/ui/vehicle.rs`, CLI in `crates/cosmogon_cli/src/main.rs`.

Current accepted baseline sequence: smooth terrain contact `1c0c6e0`; persistent shear integrated at `c9ab899`; footprint/support research reports at `c668692`. The c668 package check covered 514 tests plus one ignored, 6 packaged checks, 45 CLI/native cases, 8 inspected packaged screenshots and a 60.053035-second live run. Details and caveats are in the validation file and the ignored `dist` evidence. Never copy old evidence over new source checks.

## Recommended next work, in order

Keep these as dependencies of one eventual physical delivery path, not as disconnected demo panels. Reassess after each integrated build; the order may adapt when code reveals a prerequisite, while the destination stays the same.

1. **Complete the active cargo-mass component as a user-verifiable research workflow.** Build the app and CLI on the current source; add source/model registration and a bounded sequence definition/runner/reopener/export for burn → finite species removal → later burn. It must show per-epoch denominators, actual versus requested stock, mass residual and stated momentum/energy scope. Provide one shared simulation implementation for CLI and native UI; immutable protected output and independent replay. Keep the module deterministic (`DMath` for sim math) and preserve the golden because the present code has not changed serialized `Universe` state. Update the memo with implementation facts only after they exist.

2. **Implement actual supported surface-operation duration inside world-flight integration.** This is the closest prerequisite to safe unloading. A slow craft or recorded endpoint samples alone do not prove it. The contact solver already has actual accepted midpoint rigid-body steps and atomic one-second macro commits in `vehicle_contact_dynamics.rs`; use those actual solver states, including endpoints and midpoint, for operation evidence. Share/refactor support evaluation so it can evaluate the exact solver clock and source/frame state without calling a public live reader that assumes `world.time` has advanced. Retain a separately versioned opt-in operation definition/ledger and strict conditional save/model pins, validate monotonic clock/source continuity, bound samples/events, and make failures roll back at the previous supported macro state. Define authored support/structural/stroke/friction limits and a required current continuous sampled window. Reset current eligibility on any excluded step. Historical longest duration is evidence, never current permission. Be explicit that checks at numerical solver endpoints are a modeled envelope, not proof of continuous real hardware stability. Do not freeze `Flying` or set a landing/attachment phase.

3. **Connect changing cargo mass to authoritative flight.** Integrate held cargo, removal epochs and future propulsion into the retained `Flight`, contact translation, the documented inertia-per-mass model and its energy/angular-momentum ledger. Preserve the immutable design and every pre-removal burn denominator. Removal must not create a free impulse. Current rigid dynamics assume fixed authored gyration radii and symmetric co-rotating fuel removal; general cargo distribution changes inertia and angular momentum, so author or derive an explicit, bounded cargo layout/handling rule before allowing an eccentric removal. Strictly version flight state and source pins; absent legacy state and all v1–v3 serialization/replay must remain exact. Make native/CLI diagnostics and independent full-world recording replay agree.

4. **Add finite surface handling and recipient capacity.** Author bounded equipment rates, power/thermal assumptions, actual contact support points and per-species habitat storage limits. Inspect `Habitat::Definition`, `Initial`, `State`, `Habitat::validate`, heat, crop, electrolysis, Sabatier, moisture and breach ledgers before editing. Existing habitats do not have general unlimited oxygen/water/food tanks. Oxygen reserve is not cabin oxygen pressure; food-bound water is not potable water; dry food must not imply an energy credit. Keep requested, accepted and rejected material explicit; rejected stock remains on the craft. No timer delivery, invisible grant or discarded overflow.

5. **Transfer atomically into that exact existing habitat at true co-location.** Require the same body/system/local frame/site/time, a current supported operation window, finite equipment reach/rate/power/storage and a live craft clock. Apply species-specific `Edit`s that participate in undo and replay; retain transfer mass, linear/angular momentum and heat evidence and preserve both flight and habitat conservation validators. Validate recipient capacity and reaction/product capacities before committing either side; refusal changes neither. Design the bridge so saved/reopened world state, normal edits, flight/habitat observer recordings, CLI, native interface, context/source registry and CSV all agree. Demonstrate full state roundtrip and independent replay. Only then claim finite transfer.

6. **Continue missing phase-10 mission mechanics:** authored finite launch from the surface, orbital flight/transfer, approach, atmosphere/entry/thermal limits and lander/surface-contact/handling power. Reuse current components and exact source frames; do not call the reduced current contact solver a complete safe landing or mission feasibility model. Keep scope visible and build one end-to-end reachable transfer scenario.

7. **Reassess wider master gaps and return to owner graphics requirement.** `MASTER_GAP_ANALYSIS.md` is the live phase table (current supply sequence entries 37–42). Important remaining areas span finite settlement construction/maintenance/radiation and equipment calibration; validated climate/consequence/history propagation; registered real astronomical/planetary data and licenses; richer comparison/uncertainty and independent scientific validation; audio and accessibility; coherent workspace persistence; performance and product QA. The graphics supplement is a first-class gap: high-detail animated visitable stars and planetary/celestial bodies across viewing scales, with provenance and procedural labels. Plan rendering architecture, LOD and GPU budgets from source inspection and actual capture tests; do not launch this work until it is next in owner priority.

## Non-negotiable invariants

- Read and follow `AGENTS.md` and `HANDOFF.md` every session. Preserve the original master directive. `docs/MASTER_GAP_ANALYSIS.md` and the newest validation/checkpoint docs track progress, not a substitute for the master.
- `cargo test --release --workspace` must pass. Run `cargo clippy --release --workspace --all-targets` and avoid new warnings in touched code. Build both app and CLI for a connected source checkpoint; for visual UI work take headless PNG `--capture`s and actually inspect them; for sustained app changes run normally for 30–60 seconds and inspect logs.
- Simulation is deterministic. Use `cosmogon_core::dmath::DMath` for transcendental simulation calculations, keyed RNG if randomness is needed, and stable ordered iteration. Avoid new dependencies without a clear need.
- The released serialized-world golden is `0xf2c4140a45ae509b`. Re-pin only for a deliberate serialized-state change and explain the migration/intent in `CHANGELOG.md`. Prefer strictly optional fields with defaults/skipping to preserve old saves, fingerprints and byte-for-byte old recordings.
- Keep real measurements/data separate from procedural assumptions and simulation-derived values. Every new research model needs an explicit ID/version, source context, units, definition, bounds, unsupported regimes, validation evidence and uncertainty status. Do not claim research-grade physical accuracy from reproducibility alone.
- Protected outputs must never overwrite user data. Preserve exact original world/branch and complete observations; independently reopen/recompute reports. Native UI and CLI must use the same engine-independent model.
- Commits, when appropriate, must be authored only as GitHub user `ashvyagni`; never include `Co-Authored-By` lines. Push only as `ashvyagni` to `git@github.com:ashvyagni/SpaceRender.git`. If authentication is unavailable, stop before pushing and let the owner push. No push/tag/release is authorized merely by this handoff.
- Keep user changes and ignored evidence. Do not remove `dist`/cache artifacts, kill unknown processes, or detach the owner's mounted images. Do not create a new branch or worktree unless the owner requests it or the repo workflow needs it.

## Practical verification and known traps

Use the exact commands and capture argument explanations in `HANDOFF.md`. The usual app capture example is:

```bash
cargo build --release -p cosmogon
./target/release/cosmogon --new sol --focus Mars --distance 2 --speed 0 \
  --capture /tmp/cosmogon-next.png --capture-after 240 --quit
```

`--capture` disables keyboard input and sound; it does not prove interactive UI behavior. For app changes that need interaction, launch the app normally and inspect its behavior. Capture representative existing-body, empty/unsupported and new-feature states when useful. The project traps in `HANDOFF.md` include f32 overflow at galaxy scale, undefined reversed/equal-edge Metal `smoothstep`, expensive GPU shaders/timeouts, font private-use icon clashes, scheduler `NEVER` versus infinity, Bevy query conflicts, and stacked bottom-panel layering. Another prior long test was a Solar-System simulation advancing millions of years; redirect output to a file and verify the actual process/log rather than assuming silence is a hang. A QA harness SIGTERM (exit 15) is not a clean app quit. Keep versioned recordings exact and treat source edits as stopping at the last supported state, not a reason to reconstruct future state.

## Remaining-work estimate

The previous estimate was **roughly 60–75% of total effort still remaining**, a judgment about the full master directive plus graphics supplement, not a measured completion percentage. It is supported by the breadth of open work across supply/landing, settlement realism, scientific validation and uncertainty, the universe/data pipeline, the every-body close-up graphics requirement, accessibility and platform QA. The newly verified cargo module is only a foundation; it has not moved the project past the physical-delivery gap. Do not present this number as a project metric. Near-term, safe physical cargo transfer itself is several connected engineering builds (operation envelope, mass/inertia integration, finite equipment/storage, atomic recipient transfer, complete replay/UI/CLI verification) before the separate mission/launch/entry work.

## First actions for the next agent

1. Confirm current `git status`, branch and this handoff; read the owner-provided original prompt, `AGENTS.md`, `HANDOFF.md`, master, active gap analysis, and `VEHICLE_SUPPORT_NEXT.md`.
2. Preserve the three uncommitted cargo files. Read the full module and tests, run `git diff --check`, and inspect the verified current test/clippy logs in `/tmp` if they still exist. If not, rerun required checks from source; never infer success from this note alone.
3. Continue item 1 above until a user-visible/headless research workflow exists for piecewise cargo mass epochs and it is built, exported/reopened, visually checked, documented and checkpointed.
4. Immediately proceed to actual solver-integrated supported surface-operation duration, then connect cargo flight state, finite handling and same-habitat transfer in dependency order. Keep the long-term objective active throughout. Do not call it done when the cargo component or transfer alone passes.

## Prompt to send alongside the original directive

Continue the complete Cosmogon master directive that I am providing separately. Before editing, read `AGENTS.md`, `HANDOFF.md` and `docs/CONTINUATION_HANDOFF.md`; then inspect the current checkout and the active master gap analysis. The directive is the product specification; the handoff records what the previous agent actually changed, tested and had not yet started.

The previous agent brought the project to accepted checkpoint `c668692` for actual vehicle footprint and conditional ground-hold research reports. Before that, the project gained finite supply-vehicle design/propulsion, actual system-local gravitational flight, rotating-surface approach, terrain rigid-body contact, persistent foot shear and exact replayable research evidence. Broad earlier foundations include the native app, deterministic solar sandbox, habitat/greenhouse systems, experiments, research controls, datasets, navigation/photo/audio and partial civilization/interstellar systems. Read `docs/MASTER_GAP_ANALYSIS.md` for the phase breakdown. The directive is still far from complete: no sustained surface-operation gate, cargo transfer into a settlement, or complete end-to-end landing/delivery exists. The owner’s rich animated close-up graphics requirement for every visitable cosmic body is real, but the owner directed that cabin/supply work proceed first.

Continue the current uncommitted work; do not reset or overwrite it. It adds `vehicle_cargo_mass.rs` and its module export plus `docs/CARGO_MASS_EPOCHS.md`. It accounts for finite removal of four distinct cargo species and recomputes future ideal burns piecewise while preserving the immutable design and previous burn epochs. On 2026-10-11 its source passed `cargo test --release --workspace` (522 passed, 1 existing ignored) and `cargo clippy --release --workspace --all-targets`. It has **not** been built into a fresh app/CLI package, shown in UI, wrapped in a research definition/runner/export/reopen workflow, or connected to authoritative flight. The existing native package evidence is for the previous committed 173-file checkpoint, not this 174-file tree.

Your first build should turn those mass epochs into one shared, bounded, inspectable, independently replayable CLI/native research workflow with protected exports and explicit model/source limitations. Build and visually/runtime verify the current source. Then immediately implement the next dependency: a versioned supported surface-operation window evaluated from actual contact solver steps (including accepted midpoints), with source continuity, finite coverage and reset-on-failure. Do not infer it from low speed or sparse screenshots; do not freeze flight or imply attachment. Next connect cargo mass/inertia/energy/momentum to flight, then finite handling/power/rates and recipient capacities, then an atomic transfer into the exact co-located existing habitat with undo/save/replay/recording/CLI/UI support. No cargo grants, timer arrivals, untracked discards or relabelled species. Continue onward through launch, orbital transfer, entry/landing and the rest of the master gap list.

Treat scientific claims and product scope honestly. Preserve deterministic simulation, the golden fingerprint and old saves/flight recordings unless a deliberate, documented migration is necessary. Follow the evidence and verification workflow in `HANDOFF.md`: full release workspace tests, all-target clippy, fresh app/CLI build, actual inspected `--capture` images for visual changes and an ordinary 30–60 second run for sustained changes. Work in integrated builds and keep progressing; this is not a request to stop after the cargo workflow or delivery. Do not push, publish or declare the master complete without the required evidence and owner authorization. The previous planning estimate was that roughly 60–75% of total effort remains, as an uncalibrated judgment rather than a progress metric.
