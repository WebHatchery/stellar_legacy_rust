# Stellar Legacy UI redesign plan

Date: 2026-09-07
Status: implementation underway; see ui_redesign_validation.md for verified increments and remaining acceptance work.
Scope: presentation, navigation, and interaction clarity across the existing game.

## 1. Intended result

Make the game feel like the Custodian's view of a living generation ship. Players
should recognise their vessel and people, understand the current voyage, and find
the next useful action without reading several panels of statistics.

Retain the gold identity, dark spacecraft atmosphere, and instrumentation. Introduce
clearer type, selective colour, a prominent ship, and recognisable human speakers.
Keep simulation rules, balance, project accounting, authority, and save compatibility
intact. This work does not add ship placement, new production systems, or new lore.

Preserve the identity and survival requirements in the
[identity plan](player_ai_identity_plan.md) and
[ship work plan](ship_work_implementation_plan.md). Summaries must use existing
forecasts and records; presentation must not invent predictions or historical facts.

## 2. Screenshot evidence

Baseline: verification captures refreshed in commit `e6f2e3`. The initial review
inspected 14 representative images, not every capture or live interaction.

| Evidence | Problem | Required response |
| --- | --- | --- |
| [Dashboard](verification/ui_gameplay.png) | Many equally prominent meters; large, mostly empty log | Prioritise objective, attention, and next action; compact healthy status |
| [Risk dashboard](verification/ui_dashboard_risk.png) | Energy shortage is visually subordinate to bright healthy bars | Put urgent actionable risk near the primary objective |
| [Agenda](verification/ui_agenda.png), [narrow capture](verification/ui_agenda_narrow.png) | Large sparse cards with tiny supporting details | Compact project rows; selected detail; genuine viewport testing |
| [Ship](verification/ui_ship.png), [underway](verification/ui_ship_underway.png) | Schematic is promising but ship identity is visually thin | Promote an interactive, stateful ship schematic |
| [Subsystems](verification/ui_subsystems.png) | Repeated meters and disabled buttons obscure actionable differences | Overview plus selected system; status badges separate from actions |
| [Crew](verification/ui_crew.png) | People are mostly text rows; development reference appears in copy | Recognisable officers, lineage, plain player language |
| [Contracts](verification/ui_contracts.png) | Similar text cards; descriptions appear cut off | Comparable summaries with complete selected briefing |
| [Event](verification/ui_event.png) | Situation, advice, and options compete for reading attention | Short opening, identifiable speakers, clear choice consequences |
| [Chronicle](verification/ui_chronicle.png), [debrief](verification/ui_debrief.png) | History and emotional outcomes resemble accounting panels | Timeline, defining moments, outcome summary, expandable detail |
| [Tutorial](verification/ui_tutorial.png) | Instruction competes with an already busy screen | Highlight the exact next visible control in context |
| [Menu](verification/ui_menu.png) | Strong sense of scale disappears during gameplay | Carry ship identity and restrained atmosphere into play |

Cross-cutting defects: small condensed body text, low-contrast gold descriptions,
scanlines across content, excessive borders, ambiguous abbreviations, and buttons
that display only a missing cost instead of the action they would perform.

## 3. Design rules

- One clear focal subject and primary action per screen or blocking decision.
- Use warm off-white for body copy, gold for identity/selection, and restrained
  green for positive state. Warnings use labels and symbols as well as colour.
- Start with body text at 16-18 logical pixels at 1280x720 and headings at 22-28;
  verify rendered readability before freezing these as shared tokens. Do not shrink
  text to make a layout fit. Use sentence case for prose and ordinary labels.
- Retain the display font for titles and short instrument labels. Keep effects
  subtle around chrome; remove scanlines/glow from reading surfaces by default.
- Target 4.5:1 contrast for normal text and 3:1 for large text and essential control
  boundaries. Measure against the final composited background, including effects.
- Define primary, secondary, and quiet actions. Healthy status is a badge, never a
  disabled action. Disabled actions retain their verb and show a readable reason.
- Use visible touch targets at least 44x44 logical pixels with non-overlapping hit
  regions. Details open by tap/click, never hover alone. Drag scrolling remains.
- Keep critical risk, costs, known consequences, and timer/fallback attribution
  visible before commitment. Optional detail may hold lore, advice, and full logs.
- Express resource names clearly; explain any retained abbreviation on tap.
- Animate only meaningful changes: active work, voyage progress, damage, succession.
  Support reduced motion and static rendering without loss of information.

## 4. Proposed navigation and layout

Use five stable main destinations. Names below are proposed UI labels; update
tutorials and help in the same implementation increment that changes them.

| Destination | Contents and current screens |
| --- | --- |
| Bridge | Dashboard, recent developments, urgent recovery links |
| Ship | Schematic, subsystems, loadout/refits, Agenda projects |
| People | Officers, dynasty/succession, factions, delegation |
| Voyage | Charter board/drydock, preparation, market, active contract |
| History | Chronicle, obligations, mission archive, milestones |

Keep Pause/Resume and speed controls visible. Put Save, Help, Display/settings, and
return-to-menu inside a labelled utility menu. Keep urgent project/obligation badges
and direct recovery shortcuts visible so grouping does not conceal time-sensitive work.
Preserve the current port/travel restrictions within destinations instead of shifting
the position or numbering of main navigation items.

Bridge composition: compact resource/time header; voyage phase and objective;
ship schematic as the largest visual; an adjacent attention area with the most
urgent issue and its action; compact condition summary; up to three recent entries
with an explicit link to History. In port, the primary action leads to charter
selection or preparation. Underway, it follows the active risk or mission state.
Show an honest quiet-state summary when nothing needs intervention.

On smaller viewports, stack the main subject and attention area, use a selected
detail sheet, and scroll content vertically. Keep global time controls and modal
actions reachable. Do not scale the entire desktop interface into tiny text.

## 5. Implementation milestones

Complete, validate, and commit each independently useful increment before starting
the next. Checkboxes describe future completion; this document completes none of them.

### M0 — Baseline and two representative prototypes

- [x] Inventory all current destinations, actions, modal variants, and capture scenes.
- [ ] Produce reviewable Bridge and council-event mockups using real baseline content.
- [ ] Cover quiet travel, energy shortage, and an event with long choices/advice.
- [ ] Establish type, spacing, colour, button, and responsive layout tokens.
- [x] Record the chosen direction and differences from this proposal in this document.

Exit: both prototypes make the objective/decision and next action immediately
identifiable, preserve mandatory information, and demonstrate desktop and narrow
layout. Review their appearance before rolling the design across every screen.

### M1 — Reading layer and shared controls

- [ ] Implement tokens, readable body font, semantic colours, and subdued effects.
- [ ] Separate badges, primary actions, secondary actions, and disabled reasons.
- [ ] Remove player-facing development references and clarify resource labels.
- [ ] Inspect every existing screen for wrapping or clipping after typography changes.

Starting points: `src/ui/widgets.rs`, `src/ui.rs`, and existing theme/font ownership
identified during M0. Inspect toolkit capabilities before adding generic UI helpers.

Exit: no new text clipping; controls remain touch accessible; critical text is
readable with effects on/off and colour is not the sole indication of state.

### M2 — Navigation and Bridge

- [ ] Introduce stable destination grouping with a full old-to-new action map.
- [ ] Implement the ship-centred Bridge, objective, attention, and compact log.
- [ ] Link warnings directly to the relevant recovery or project action.
- [ ] Update tutorial, help, and README navigation references with the new labels.

Starting points: `src/ui/shell.rs`, `src/ui/dashboard.rs`,
`src/ui/dashboard/status.rs`, `src/ui/time_controls.rs`, `src/ui/tutorial.rs`.

Exit: charter selection, urgent repairs, project queue, obligations, save, and help
remain discoverable. Risk captures visually prioritise the actual risk. Pause and
recovery stay reachable in every responsive state and blocking overlay.

### M3 — Ship and work management

- [ ] Make compartments selectable and distinguish damage, staffing, and active work.
- [ ] Reuse ship-class silhouettes; use labels/icons as well as status colour.
- [ ] Put selected subsystem details and valid actions beside or below the schematic.
- [ ] Replace oversized Agenda cards with progress/status rows and selected details.
- [ ] Keep queue/pause/resume/reorder/cancel and emergency recovery fully accessible.
- [ ] Show costs, escrow/waiting reasons, and existing forecast uncertainty truthfully.

Starting points: `src/ui/ship_schematic.rs`, `src/ui/ship_schematic/draw.rs`,
`src/ui/ship_builder.rs`, `src/ui/subsystems.rs`, `src/ui/agenda.rs`,
`src/ui/agenda/readiness_panel.rs`.

Exit: all current ship classes and damaged/refitted states remain recognisable;
the same work operations and recovery routes function without rule changes.

### M4 — People and council decisions

- [ ] Give named officers stable portraits or distinctive silhouettes and role icons.
- [ ] Show captain, heir, vacancies, and succession as a readable connected lineage.
- [ ] Present factions with distinct emblems and concise current concerns.
- [ ] Restructure council events into situation, speaker, choices, and optional advice.
- [ ] Keep known costs/outcomes visible on each choice; preserve authority and timeout.

Starting points: `src/ui/crew_dynasty.rs`, `src/ui/event_modal.rs`,
`src/ui/authority_modal.rs`, `src/ui/recovery_warning.rs`.

Exit: long events and unavailable options remain readable; opening detail never
silently changes pause/countdown semantics; the acting captain/council is explicit.
Character identity remains stable across screen changes and save/load.

### M5 — Voyage and living history

- [ ] Give charters comparable duration/reward/risk summaries and destination visuals.
- [ ] Present the full selected briefing without clipped prose.
- [ ] Make departure a clear provisions/obligations review with existing launch rules.
- [ ] Lead Homecoming with outcome, generations, survivors, and defining decisions.
- [ ] Use a timeline for historical events with full records available on selection.
- [ ] Keep overdue obligations prominent and archives/milestones accessible.

Starting points: `src/ui/prep.rs`, `src/ui/mission.rs`, `src/ui/market.rs`,
`src/ui/contract_systems.rs`, `src/ui/chronicle.rs`, `src/ui/debrief.rs`.

Exit: players can compare charters, understand launch commitments, inspect inherited
promises, and read the full debrief without losing existing mechanical information.

### M6 — Complete acceptance and release evidence

- [ ] Run the end-to-end routes and visual matrix below.
- [ ] Replace affected verification captures and review every capture in the manifest.
- [ ] Refresh release screenshots only after the presentation has stabilised.
- [ ] Keep root `catalog_thumbnail.png` aligned with the title screen if it changes.
- [ ] Update design/help documentation and record validation results and open limitations.

## 6. Assets and implementation boundaries

First use the existing ship schematic, ship-class variants, and title-art identity.
Add a small consistent set of officer silhouettes/portraits, faction emblems, and
destination visuals in M4/M5. Record asset provenance and licensing, bundle assets
locally, and provide intentional fallbacks. Do not make asset generation a blocker
for the first layout prototype. Avoid full-screen decorative images behind prose.

Game-specific view models and layouts belong in Stellar Legacy. Generic text
measurement, clipping, scrolling, responsive layout, focus, and touch-target gaps
should first be evaluated as macroquad-toolkit upgrades. Keep draws separate from
game actions and reuse authoritative state/forecasts. No new dependency without a
concrete reduction in complexity. Keep every Rust file at or below 800 physical
lines and tests in separate child files; split cohesive responsibilities as needed.

## 7. Validation and definition of done

For every meaningful implementation increment run `.\publish.ps1` without
parameters from the project directory. Record pass/failure and any environment
blocker. Do not substitute a local development server for this validation path.
Use focused separate-file tests where action routing, selection, timers, or state
transitions change; do not write tests that merely repeat decorative constants.

Visual matrix: 1280x720 baseline, 1920x1080 desktop, 1024x768 compact desktop, and
390x844 portrait touch viewport. These are target acceptance sizes, not a claim of
existing support. Verify actual viewport dimensions rather than trusting a capture
whose filename says narrow. Check long names, large values, full queues, empty
lists, disabled actions, modal overflow, reduced motion, and display effects.

Store captures directly in `docs/verification/`; replace existing images for the
same state. Give genuinely additional viewport/state coverage distinct filenames
without subfolders. Record scenario/viewport and outcome in a future
`docs/ui_redesign_validation.md` so screenshot evidence is reproducible.

Required click/touch-only routes:

1. New game → founding → charter → provisions → launch → first council decision.
2. Risk on Bridge → recovery/project → queue → pause/resume → reorder/cancel.
3. Officer vacancy → appointment/training → heir review → succession record.
4. Due obligation → review → affordable or unaffordable choice → resulting history.
5. Homecoming → report detail → next charter; save → reload → same relevant state.
6. Critical air recovery and terminal failure, including reaching every modal action.

Human acceptance: ask a fresh reviewer to identify the current objective, most
urgent issue, and next action from the Bridge within five seconds. Ask them to
explain a council choice's known cost and consequence without opening optional
advice. Record observed results; automated captures cannot establish engagement
or discoverability. Target zero clipped essential labels, overlapping targets,
hover-only information, keyboard-required steps, or inaccessible recovery actions.

## 8. Tracking and sequencing

Dependency order: M0 → M1 → M2 → M3 → M4 → M5 → M6. Ship and event prototypes come
first to establish both operational and human identity before broad migration.
Each milestone may contain multiple focused commits with its own validation.
Do not bundle unrelated balance changes into this redesign.

The first implementation task is M0: produce the Bridge and council-event
prototypes and complete the action inventory. This planning change itself does
not alter game behavior, assets, screenshots, or the status of existing human QA.

Implementation direction: in-game Bridge and council prototypes now use the existing ship silhouette and authoritative state. Council options use selection followed by an explicit commitment, with full consequences in a scrollable reading area. See [validation and action inventory](ui_redesign_validation.md). Desktop prototypes precede responsive acceptance; unchecked items remain outstanding.
