# Stellar Legacy — current game design

This is the implemented design for version **0.2.1**, checked against the Rust
state, simulation, UI and authored data. It describes the full game and its
feature-gated tutorial demo. Unimplemented extensions and unresolved acceptance
work belong in [TODO.md](TODO.md), not in the current rules.

## 1. High concept

The player is the Custodian, the persistent intelligence aboard a generation
ship. Human captains age, councils change and promises outlive their creators.
One ship and its society travel through repeated departure, voyage, Homecoming
and drydock cycles. This is a single-player expedition strategy game, presented
through a command terminal with a ship schematic, dossiers, people and records.
It has no tactical combat, spatial empire simulation or account service.

## 2. Design pillars

- Generations create consequences: succession changes expertise, authority and
  inherited duties while the Custodian persists.
- Every choice must change real state. The log names what happened and who acted.
- Preparation and underway work compete for limited stores, time and project slots.
- Failures remain legible and recoverable while survival is still possible;
  terminal outcomes end that campaign.
- The interface supports complete touch/click play. Keyboard shortcuts supplement
  visible controls; neither sound nor color carries essential information alone.

## 3. Core loop and time

Found a dynasty by choosing a legacy and founding peoples. In port, compare a
charter, review its briefing and mandate, provision, recruit/train, service and
refit. Launch begins travel, followed by operation and return. Homecoming seals
the outcome and returns the same ship, people, obligations and damage to drydock.
A new hull purchase does not reset society. A new campaign creates new people;
Chronicle renown supplies its automatic Heritage head start.

The authoritative simulation advances in months, with annual economy/birthdays.
The live driver uses **1 real second per month at 1x**, half a second at 2x and
one third at 3x. In port, on explicit pause, or during a tutorial hold, simulation
time is frozen. Blocking decisions stop voyage advancement. Their separate
**90-second** countdown also stops on explicit pause. Pending authority reviews,
recovery notices and terminal outcomes are handled through their own visible
flows. Returning from a stop does not spend a hidden backlog of months.

There is no offline progress. The 30–60 minute ordinary-voyage experience remains
a pacing goal requiring human measurement, not an enforced duration. Charter
length, speed, reading time and pauses all matter; the tutorial is intentionally
shorter than the long campaign charters.

## 4. Identity, authority and commands

The Custodian performs routine operations and proposes strategic command posture.
The human captain may object when the proposed posture conflicts with that
captain's priority and the current ship state. The authority review offers
retaining the current policy, an available compromise, or a narrowly gated
emergency override. Eligibility is rechecked when the action is dispatched.
Ordinary repairs, allocation, trade and training do not all require ratification.

Captains' priorities derive deterministically from their authored identity.
Succession replaces the human partner while retaining mandate/history. Automatic
council choices name the acting officer or captain. Timed events choose the
highest-scoring available, affordable outcome; if none qualifies, the resolver
attempts the first outcome and dispatch validation can reject it. Authors must
provide a valid escape path. Dilemmas select the best success odds; posture
reviews accept an available compromise or retain the current posture. Delegation is category-based and legacy-neutral.

Memories cite durable Reign and Obligation records, not trimmed general log text.
Missing provenance produces neutral archival language. Perceived Custodian
conduct is not a claim about the player's private emotions. See
[event authoring](event_design_notes.md) for voice and authority requirements.

## 5. Simulation systems

### 5.1 Economy and ship

Credits, energy, minerals, food and influence are separate resource fields; fuel
and spare parts belong to ship state. The market trades credits against energy,
minerals, food and influence in port. Annual production, population consumption, farming,
spoilage, power shortages, maintenance, social state and fitted components affect
one another. Forecasts reuse authoritative calculations and cannot predict future
random events or population changes.

New campaigns start with 10,000 credits, 5,000 energy, 2,000 minerals, 1,500 food,
100 influence, 1,000 people and 300 spare parts. Heritage can add bonuses. Exact
rates and limits live in [game_config.json](assets/data/game_config.json).
Six subsystems have condition and institutional knowledge: life support,
engineering, agriculture, medical, education/culture and security. Underway
service/training uses ship projects; port service is immediate. Full refits,
component shopping and commissioning hulls are port actions. Salvage/fitting
eligibility remains governed by the ship and subsystem services.

### 5.2 Charters and posture

The full registry contains 23 charters across mining, colonization, exploration,
rescue, diplomacy and salvage. The tutorial, **Proving Run: Lumen Relay**, lasts
150 years: 50 travel, 50 operation and 50 return. The `demo` build offers only this
charter and ends its tutorial loop after Homecoming; the full build offers the
broader registry subject to normal eligibility gates.

Charters own their phases, route, loadout/reputation/deed requirements, milestones,
objective accrual, success metrics, rewards and content biases. Outcome scoring
combines normalized weighted metrics into complete, partial, pyrrhic or failure
bands; consult [contract simulation](src/simulation/contract.rs) and data for
exact scoring. Launch warnings distinguish hard eligibility gates from risks the
player may accept, including shortages and conflicting obligations.

Steady, Expeditionary and Civic are implemented **global command postures**.
They affect objective work, event pressure, fuel and social change, and appear in
preparation, the active voyage and sealed records. Underway posture changes have
an annual lock and can trigger captain review. They are not the proposed
charter-specific operating-approach system described in the backlog.

### 5.3 Dynasty, crew and institutions

Living relatives and officers age on annual Founding Day. Mortality is checked
monthly with an age-dependent hazard, and severe events can claim named people.
Retirement, death, eligible/designated heirs and surviving family determine
succession. A depleted line can become extinct. Founding peoples, faction
membership, approval and relationships continue to evolve during the voyage.

Seven crew archetypes supply named expertise. Vacancies and untrained posts have
real effects. Relevant serving officers provide state-aware council advice;
advice does not claim an optimal choice. Apprenticeships preserve some expertise
through turnover. Schools, archives and faction custody support subsystem
knowledge with their costs and political tradeoffs. Homecoming retains important
appointments, losses and continuity outcomes.

### 5.4 Events, obligations and aftermath

Events combine category, family, gates, complications and outcomes. The registry
is embedded from eleven family files, guarded against duplicate IDs. At launch,
a seeded campaign skeleton schedules major beats using configured phase/era
pools and charter biases; monthly reactive rolls and threshold, recovery,
succession and scheduled consequences add state-dependent encounters.

Outcomes can affect resources, social state, reputation, components, knowledge,
faction loss, force-return, obligations and persistent issues. Eligibility,
affordability and authority determine which responses can be committed.
Complications can target particular outcomes, and later events read durable facts.

Obligations are structured duties with creator, responsible captain/office,
beneficiary, due date, status and history. They can cross succession, constrain a
later charter, and be fulfilled, renegotiated, defaulted or voided. They are distinct
from historical consequence flags. Event aftermath is likewise distinct: active
issues can impose ongoing penalties and require specific recovery projects.
Clearing an issue does not refund losses from the original event.

### 5.5 Legacies and society

Preservers, Adaptors and Wanderers each have six authored dilemmas and distinct
social pressure. Tradition, adaptation, loyalty, cultural drift, unity, reputation
and other tracked values affect real gates and consequences. The six founding
peoples are a separate faction system; founding selection, migration, births,
losses, mood and political relationships change who lives aboard.

### 5.6 Determinism

`SimState` owns a serializable toolkit `SeededRng`. Simulation rolls, choices and
month advancement must replay from the same state and inputs. Rendering and
previews do not consume that stream. Seed alone does not reproduce a live played
session because action timing changes its inputs. The cosmetic elapsed-voyage
clock and display settings do not feed simulation results.

### 5.7 Custodian Agenda

The current limit is **two running jobs and four waiting jobs**; queued and
suspended work share the waiting cap. Queueing is free. Starting rechecks the
target, knowledge, affordability and capability limits, then escrows the full
cost. Work commits materials monthly and delivers stages exactly once. A blocked
job remains visible with its reason. Eligible waiting jobs follow player priority.

| Project | Duration (months) | Delivery stages |
| --- | ---: | ---: |
| Service subsystem | 72 | 2 |
| Restore hull | 96 | 2 |
| Overhaul life support | 96 | 2 |
| Train replacement cohort | 96 | 2 |
| Optimise hydroponics | 144 | 2 |
| Restore crew quarters | 60 | 10 |
| Sterilise damaged growing systems | 48 | 1 |
| Establish seed programme | 216 | 3; capability at completion |

[projects.json](assets/projects.json) owns costs, requirements, refund masks and
effects. Pause releases the slot and keeps investment. Expertise loss also
suspends affected jobs. Paused work has a 12-month grace, then configured
restoration debt; the current debt rate is 1% per month, capped at 25%.
Cancellation keeps delivered results and refunds the configured recoverable
uncommitted escrow (80% before per-resource eligibility), not the whole purchase.
The review shows refund, retained deliveries, remaining work and restoration cost.
Fractional settlement is persisted; repeated actions must not mint or lose stores
through rounding. The housing/food-work pivot retains only legitimate delivery.

Work advances only on voyage months. Homecoming suspends unfinished work; port
has no clock-driven deterioration or instant project completion. Repair effects
apply bounded gains to current damage. Hydroponics affects actual production;
seed preparation unlocks its authored blight response only when complete.
Readiness compares food, fuel, engineering, air, knowledge and cohesion using
shared forecasts, issue state, observed trends and recovery hysteresis.

### 5.8 Survival

Dynasty extinction, zero hull, zero population and sustained zero life support
are terminal conditions. Zero-air grace is **12 voyage months**, counted once per
month. The critical threshold is 20%; its review stops the real-time burst.
Emergency stabilisation adds 15% life support at a cost of 300 energy,
50 minerals and 10 spare parts. It is limited within an air crisis and rearmed
when warning/countdown recovery clears its used flag. It buys time rather than
curing every failure.
The Agenda retains a recovery path after the first warning is reviewed.
Terminal results are sealed and recorded; recovery controls do not revive a dead
campaign. Food/fuel warnings are forecasts, with shortages feeding other damage.

## 6. Data and assets

[GameData](src/data.rs) embeds typed JSON through `macroquad_toolkit::data_loader`
and include macros on both native and WASM. Editing loose JSON does not hot-reload
rules into a compiled build. Games own schema/semantic validation; the toolkit
owns generic loading. Sources are:

- `assets/events/*.json`: 343 events across 11 families.
- `assets/contracts.json`, `legacies.json`, `factions.json`, `subsystems.json`:
  charters, legacies, peoples and six systems.
- `assets/projects.json`: eight project definitions.
- `assets/ship_components.json`, `crew_archetypes.json`, `dynasty_names.json`:
  equipment, seven crew roles and name/identity pools.
- `assets/data/game_config.json`: tuning and shared authored interface/flavor text.
- `assets/data/texture_manifest.json`: title texture; runtime images are packaged.

The UI uses the bundled DejaVu font and procedural ship/person/faction graphics.
Title and store imagery have separate provenance records. Art is not absent merely
because the game is primarily a terminal interface.

## 7. Persistence and progression

The full game uses the `stellar_legacy` storage namespace and one `autosave`
campaign slot. Chronicle history uses its own slot and survives new campaigns.
Display, delegation and onboarding preferences use separate keys. The demo adds
`_demo` to the namespace, isolating saves and preferences.

Version 0.2.1 accepts 0.1.0 and 0.2.0 campaigns through explicit migration and
validation. Version 0.2.0 quarters investments retain final-only delivery;
0.1.0 survival state receives recovery migration. Unsupported or invalid saves
produce errors and attempt quarantine rather than silently replacing progress.
See [support](docs/release/SUPPORT_AND_PRIVACY.md) for storage and recovery.

Chronicle renown automatically chooses the highest unlocked of four Heritage
tiers when a dynasty is founded. There is no Heritage modifier-selection screen.
Sealed Homecoming records preserve departure/return comparisons, captain reigns,
obligations, institutions, posture and notable decisions after the active contract
and general log have changed. Saving during the report retains that report.

## 8. Content quality

Prefer distinct situations and delayed consequences to larger counts. Couple
existing systems, preserve phase/year/generation gates, and vary recurring events
with meaningful complications. Avoid filler options, anonymous authority and
fake history. [Event authoring](event_design_notes.md) is the content contract;
[release QA](docs/release/QA_AND_OPERATIONS.md) distinguishes automated checks
from human acceptance.

## 9. Presentation and navigation

Bridge, Ship, People, Voyage and History stay in a stable order in port and underway.
Ship contains Loadout, Systems and Agenda; People separates Family, Officers,
Factions and Council. Voyage offers Drydock/Market in port and Contract underway.
The compact Family view distinguishes the automatically planned successor from
a named heir, explains the eligible age range and retains a visible current-heir
marker after designation. Family entries include specialization and trait.
Compact officer entries explain each post, retirement timing and apprenticeship
succession. Training names its resulting skill, and unaffordable actions show
the missing credits alongside the available balance.
Utilities contains Save game, Help, Display & sound and Return to menu.
Pause/Resume and speed controls remain reachable above blocking decisions.
Responsive and phone navigation underline the active destination and section;
phone speed controls underline the current running speed. Touch presses visibly
light navigation targets, and Drydock/Market share a compact section row.
Section rows reflow to fit enlarged labels. Scrolling documents reserve a footer
for a reading hint that distinguishes more content from the end of the section.
Phone navigation uses a second row when enlarged labels need it. At extreme UI
scale or limited height, a persistent Navigate control opens the five destinations
as a scrollable list; Back returns to the current screen. The phone
header and time controls reserve space for the selected text size.
Council choices select a response before explicit Commit; full consequences scroll.
Immediate council fuel consequences show the tank before and after subsystem
protection and capacity limits, rather than an uncapped authored percentage.
Decision countdowns explicitly say when fallback is paused, alongside the
remaining seconds, consistently across council, mandate and legacy decisions.
Return-home reviews share a scrolling consequence summary across display sizes.
Keep voyaging, station navigation and Utilities close the review and release its
temporary clock hold; the selected time speed is preserved.
The phone posture comparison identifies the current policy, explains the 100%
baseline and annual social tradeoffs, and shows months until another proposal
is available. The current policy cannot be proposed again.
The phone voyage report exposes the shared travel fuel check. A fuel stall
explains current annual scoop recovery or its absence, port-only refuelling,
and provides a visible link to Agenda readiness. Fuel readiness is distinct
from whether the player has paused time.
Departure fuel forecasts name total burn in full tanks and annual scoop recovery
as a percentage of one tank, making replenishment across long voyages explicit.
The narrow founding picker shows each people's outlook and cared-for subsystem
before selection. It counts selected peoples, names Remove explicitly, disables
additional selections when full, and explains how many more are needed to begin.
The guide offers FINISH only after all lessons, including the council response,
are complete. Its completion message is visible on desktop and phone; advancing
a lesson returns a scrolling reading view to the next instruction.
Agenda project reviews share one reading flow across display sizes. Pause and
resume show the same availability checks used by the command, including slots,
port restrictions and exact restoration affordability. Cancellation has a
separate refund review with Keep project and Confirm cancellation controls.
Station navigation, Utilities and campaign transitions close project reviews
and release their temporary clock hold without changing the chosen game speed.
Catalogue entries, queued work and reviews use the same authored subsystem names
as the Systems screen, so the player can identify each project's target.
Project choices and reviews include their authored purpose. Desktop catalogue
details and phone entries share a scrolling description, budget and delivery
schedule before the queue action.
Successful additions return to the project queue and name the added work.
The desktop selection follows the new job; failed additions keep the current
choice visible with the failure reason.
Waiting positions count queued and paused jobs, skipping running and ended work.
Reorder buttons stop at the list boundaries, and desktop selection follows the
moved job. The queue explains that unavailable jobs are skipped and paused work
requires an explicit resume.
Catalogue and recovery controls check live duplicates and queue capacity before
offering new work. Existing jobs instead expose a direct review action. A missing
starting budget alone does not prevent creating an unpaid waiting job.

Amber is the default color scheme, with Green and Slate available. Scanlines and
flicker default off. Master sound and ambience are independently controlled.
UI scale is 75–200% and text size 75–150%, separately saved with visible resets.
Full-window responsive coordinates reflow/scroll panels; scale does not zoom a
world camera or force desktop into mobile navigation. Settings retains a fixed
Close settings control. Touch-only long-route acceptance remains open in release QA.

## 10. Shared toolkit boundary

Use toolkit input/UI/text, assets, persistence, RNG, capture and procedural audio
helpers. Game-specific screen composition, authority, simulation and saves stay
here. The dependency's [module guide](../macroquad-toolkit/MACROQUAD_TOOLKIT.md)
is authoritative for generic API usage; do not maintain a copied guide here.

## 11. Code ownership

| Source | Responsibility |
| --- | --- |
| [main.rs](src/main.rs), [boot.rs](src/boot.rs), [game.rs](src/game.rs) | Startup, loaded assets, frame loop and coordination |
| [game/actions.rs](src/game/actions.rs) and children | Dispatch player intents; recheck operations |
| [game/realtime.rs](src/game/realtime.rs) | Live clock and decision countdown |
| [state/sim.rs](src/state/sim.rs) and children | Serializable campaign state |
| [simulation.rs](src/simulation.rs) and children | Month/annual rules, events, projects, contracts and people |
| [data.rs](src/data.rs) and children | Embedded schemas and content validation |
| [ui.rs](src/ui.rs) and children | Read state, lay out screens, return actions |
| [save.rs](src/save.rs), [chronicle.rs](src/chronicle.rs), [heritage.rs](src/heritage.rs) | Local persistence, sealed history and new-campaign bonuses |
| [settings.rs](src/settings.rs), [audio.rs](src/audio.rs) | Local presentation preferences and synthesized playback |

## 12. Scope boundaries

No runtime generative AI, accounts, network simulation, cloud saves, tactical
crew movement, freeform deck building or galactic empire management is implemented.
Global command posture, the current Homecoming report and procedural emblems do
not imply that the deferred approach, compartment-culture or competing-record
systems exist. See [TODO.md](TODO.md) for their bounded retained intent.

## 13. Validation

Run `.\publish.ps1` with no parameters from this project. It builds/packages and
deploys Preview, then checks the exact Windows and browser packages. Unit tests,
source-size checks, deterministic cohorts and screenshot inspection complement
that path. No automated result is a substitute for clean-machine, store-client or
human pacing/quality acceptance. Procedures and open gates are in
[release QA](docs/release/QA_AND_OPERATIONS.md).
