# Remaining tasks

## UI_STYLE review — 2026-09-20 (revised 2026-11-02 for the single canvas)

Audit against `UI_STYLE.md` §§1–9, `CODE_STANDARDS.md` §7,
`GAME_DEVELOPMENT_GUIDE.md`, `AGENTS.md`, README and GDD. This section is an
implementation backlog, not a claim that changes are complete. The Steam
readiness review in `docs/CRITICAL_REVIEW.md` extends several of these tasks.

### Supported canvas

The game targets **one desktop canvas: 1920×1080**. `ui::canvas()` is a fixed
1920×1080 virtual UI, letterboxed into the host window or browser canvas; there
is no compact, phone or UI-scale layout path. Text size (75–150%) is the only
display-size preference. Every task below is designed, captured and reviewed at
1920×1080.

### Evidence and scope

- Reviewed source for desktop dispatch, navigation, time controls, Bridge,
  preparation, active voyage, subsystem inspection, schematic layout/rendering,
  loadout preview and the shared `ui::form` document view.
- Captures live in `docs/verification/` as one 1920×1080 set under stable
  `ui_<scene>.png` names.
- Retain successful existing patterns: stable five-destination navigation;
  instruments behind disclosure; the utilities drawer; event selection followed
  by explicit Commit and contextual officer advice; scrolling Homecoming records
  and explicit recovery decisions. Do not replace legitimate management
  comparisons with a decorative world view.

### Findings and implementation tasks

Ordered by player impact and dependencies. The contract is the screen briefs
in `docs/gdd.md` §9.1; UI-02–04 address composition and core decisions before UI-05–07 refine inspection.

- [ ] **UI-02 — Recompose the Bridge around the ship/current concern and retire the permanent event panel.**
  **Scope/files:** Port, calm voyage and resource-risk Bridge;
  `src/ui/bridge.rs::{draw,draw_voyage_subject,draw_attention,draw_operational_summary}`,
  `src/ui/widgets.rs`, `src/game/render.rs::draw_notifications`.
  **Observed:** `ui_gameplay.png` has four equally bordered regions: ship,
  attention, ship/people and recent developments. Choose a charter, inspection,
  queue and History share button treatment. The fixed 400px subject and 250px
  diagram leave almost half the screen to sparse lower panels. The recent-log
  panel persists old events indefinitely. Generation repeats in the header and
  summary.
  **Change:** Use a dominant ship/voyage area, one current-concern/action area and
  quiet navigation. Merge essential hull/air/fuel state with the ship; remove the
  separate Ship & people box, duplicated generation/status and repeated port
  instructions. Replace the permanent Recent developments panel with brief
  change feedback and a quiet visible History route. Keep unresolved threats
  persistent next to their recovery action even after feedback expires. Let the
  subject fill the available height. Give the phase-appropriate action distinct
  emphasis; reduce queue/inspection utility weight without hiding them.
  **Acceptance:** Within a glance, the player can identify the current concern
  and next action. Calm play has no more than three strong attention regions;
  the 1080p canvas is filled with useful ship content rather than log
  containers. Old events are retrievable in History.
  **Check:** Gameplay, dashboard-risk and calm underway captures; concern →
  recovery → Bridge and History → Bridge paths; simultaneous reserve warning
  and due obligation. Follows the GDD §9.1 briefs.

- [ ] **UI-03 — Separate time controls from utilities and show whether time can actually advance.**
  **Scope/files:** Port, Homecoming and blocking decisions;
  `src/ui/time_controls.rs`, `src/game/render.rs::draw_overlays`,
  `src/game/realtime.rs`.
  **Observed:** Pause and all speed buttons remain prominent in port and
  Homecoming, although those phases cannot advance voyage time (GDD §3).
  **Change:** Group Pause/Resume and speed together, apart from Utilities. In
  phases with no running clock, replace the speed bank with a concise In
  port/Report review state. Distinguish explicit pause, tutorial hold, fuel
  stall and decision fallback. Keep a direct Pause control available when the
  decision countdown can run.
  **Acceptance:** The player never infers that choosing 3x will progress a docked
  ship. Decisions permit immediate pause and clearly show whether fallback time
  is stopped.
  **Check:** Port → launch → pause → decision → resume → Homecoming, with speed
  preservation and the fallback countdown.

- [ ] **UI-04 — Put departure readiness before briefing detail and make launch the clear commitment.**
  **Scope/files:** Selected charter/PREP and tutorial provisions lesson;
  `src/ui/prep.rs`, `src/ui/mission.rs::draw_selected`, `src/ui/charter_approach.rs`,
  `src/ui/contract_systems/outlook.rs::draw_posture`, `src/ui/tutorial.rs`.
  **Observed:** PREP divides attention among provisioning, full briefing,
  approach and posture. Launch has the same styling as refuel/cancel. Approach
  explanations and the locked reason use the smallest type on screen.
  **Change:** Start with charter goal/duration, readiness and consequential
  warnings. Keep purchase costs beside food/parts/fuel controls. Place an
  emphasized launch action beside the aggregate shortages and obligation
  defaults, with access to every affected duty before commitment. Show selected
  approach/posture summaries with explicit comparison controls and readable
  effects, eligibility and locked-at-launch wording.
  **Acceptance:** Launch is distinct from cancel/refuel; full costs, fuel
  assumptions, shortages and default consequences remain available before
  launching. No completed tutorial instruction persists as ordinary chrome.
  **Check:** Long briefing, locked approach, insufficient funds, several
  conflicting obligations and tutorial on/off; 150% text. Depends on the action
  hierarchy from UI-02.

- [ ] **UI-05 — Make the ship schematic useful and identifiable.**
  **Scope/files:** Bridge, Systems and loadout preview;
  `src/ui/ship_schematic.rs::build`, `src/ui/ship_schematic/draw.rs`,
  `src/ui/subsystems/overview.rs::{draw,select_compartments}`,
  `src/ui/ship_builder/preview.rs`.
  **Observed:** Labels are dimmed for vacant posts; in `ui_subsystems.png`, lower
  captions sit very close to their glyphs. The loadout preview scales a
  1900×330 source into a shallow strip with tiny module labels.
  **Change:** Fit the diagram to its container with reserved label bands and a
  readable minimum compartment size. Provide names and textual
  condition/vacancy on selection, not color alone. Keep vacant compartment
  names readable. In refit comparison, allow a larger inspectable preview and
  move tier-explanation prose to contextual help.
  **Acceptance:** A player can identify and inspect a failing compartment
  without guessing colored boxes. Labels do not crowd glyphs; all hull
  silhouettes fit. Catalogue comparison remains the dominant refit decision.
  **Check:** Barge, corvette, ring and ark; healthy, damaged, vacant and
  active-project compartments. Depends on UI-02.

- [ ] **UI-06 — Split immediate subsystem care from institutional and culture detail.**
  **Scope/files:** Ship/Systems selected compartment, port and underway;
  `src/ui/subsystems.rs::{draw_card,draw_card_identity,draw_card_bars,draw_card_actions}`,
  `src/ui/subsystems/overview.rs`.
  **Observed:** The selected card always combines condition, knowledge, tier,
  school, archive, custodian, culture effects, memory and grievance before a
  row of repair/upgrade/train/institution controls. Culture uses a fixed small
  text block.
  **Change:** Lead with condition, repair capability, knowledge prerequisite and
  the relevant repair/train action including cost/shortfall. Move culture
  history, exact passive modifiers, school/archive management and custody
  comparison into an explicit Continuity & culture disclosure. Keep an active
  grievance, expiring school or lost repair capability summarized next to the
  affected decision. Reflow detail instead of reducing font size.
  **Acceptance:** A new player can diagnose and address a damaged system without
  reading institutional history; no urgent consequence or prerequisite is
  concealed.
  **Check:** A damaged low-knowledge system, a sound system and a late-game
  school/custody case at 100% and 150% text. Depends on UI-05.

- [ ] **UI-07 — Focus the active voyage on progress and the current obstacle.**
  **Scope/files:** Voyage/Contract during travel, operation, fuel stall and return;
  `src/ui/contract_systems.rs::{draw_active,draw_contract_progress,draw_contract_milestones,draw_return_button}`,
  `src/ui/contract_systems/outlook.rs`.
  **Observed:** `ui_contract_active.png` devotes most emphasis to bright success
  metric bars and a full-width Cancel mission / Return home control. A real
  fuel stall appears as a small text row. Phase/milestones repeat across the
  two panels; outlook and posture add nested borders and small prose.
  **Change:** Make route/phase, objective and next milestone the dominant
  summary. Promote a current stall to an actionable obstacle with its recovery
  route. Disclose scoring weights, completed milestones, locked approach effects
  and posture alternatives on demand. Remove the empty outlook box and repeated
  phase facts. Keep a secondary Review return home action with its confirmation.
  **Acceptance:** A stalled player first sees why progress stopped and where to
  recover. A calm player sees mission progress.
  **Check:** Fuel stall with/without scoop recovery, calm operation, reached
  milestones and return. Depends on UI-02/03 and the disclosure from UI-04.

### Further inspection required

- [ ] **UI-08 — Review the revised hierarchy in play and in dense states.**
  **Scope/files:** All affected screens; `scripts/capture_ui.ps1`,
  `src/game/capture_scenes.rs` and children, `src/game/render.rs`.
  **Action:** After each affected implementation, regenerate the scenes at
  1920×1080 directly in `docs/verification/`, replacing equivalent captures.
  Sample text 75/100/150%, long names/large values, full queues, extended
  family/history, first-use guide, selected inspector, authority/event decision,
  critical-air recovery, Homecoming and terminal state. Exercise navigation,
  inspection, drag, purchase, pause/resume, confirmation/dismissal, Help
  reopening and save/menu recovery. Observe notifications at arrival and after
  expiry, including whether they cover actions.
  **Acceptance:** Reviewers can identify the current decision and primary action
  within roughly a second; no more than three strong regions compete during
  normal play. Costs, warnings and recovery survive disclosure. Record checked
  interactions and any untested combinations explicitly.
