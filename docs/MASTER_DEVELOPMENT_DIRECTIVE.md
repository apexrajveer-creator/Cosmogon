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