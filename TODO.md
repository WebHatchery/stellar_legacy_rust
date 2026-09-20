# Remaining tasks

## UI_STYLE review — 2026-09-20

Audit against the updated `UI_STYLE.md` §§1–9, `CODE_STANDARDS.md` §7,
`GAME_DEVELOPMENT_GUIDE.md`, `AGENTS.md`, `PROJECT_AGENTS.md`, README and GDD.
This section is an implementation backlog, not a claim that changes are complete.
The existing file contained only its heading; no tasks or completion history were
removed. Existing release acceptance history remains in
`docs/release/QA_AND_OPERATIONS.md`.

### Evidence and scope

- Reviewed source for desktop, compact and phone dispatch, navigation, time
  controls, Bridge, preparation, active voyage, subsystem inspection, schematic
  layout/rendering, loadout preview and mobile form rendering.
- Visually inspected existing captures in `docs/verification/`: `ui_gameplay.png`,
  `ui_prep.png`, `ui_agenda.png`, `ui_ship.png`, `ui_event.png`,
  `ui_subsystems.png`, `ui_contract_active.png`, `ui_debrief.png` (1280×720);
  `ui_gameplay_desktop.png` (1920×1080); `ui_gameplay_portrait.png`,
  `ui_dashboard_risk_portrait.png`, `ui_prep_portrait.png`,
  `ui_tutorial_portrait.png` (390×844). These are retained captures, not fresh
  screenshots of this checkout. Source corroborates the findings called out below.
- No game run, new capture, browser interaction, physical touch test or timing
  test was performed for this planning-only pass. Screenshot inspection establishes
  composition in those images, not present-build interaction correctness.
- Retain successful existing patterns: stable five-destination navigation;
  instruments behind disclosure; separate desktop utilities drawer; event
  selection followed by explicit Commit and contextual officer advice; scrolling
  Homecoming records and explicit recovery decisions. Do not replace legitimate
  management comparisons with a decorative world view.
- No literal template demonstration copy was identified in the sampled screens.
  The repeated bordered surfaces and generic button treatment below are observed
  composition issues, not evidence that the game still uses an unadapted template.

### Verified findings and implementation tasks

Ordered by player impact and dependencies. UI-01 establishes the contract;
UI-02–04 address composition and core decisions before UI-05–07 refine inspection.

- [ ] **UI-01 — Record phase-specific screen briefs and supported canvas sizes.**
  **Scope/files:** `README.md`, `gdd.md` §9; dispatch in
  `src/ui/shell.rs::draw_gameplay`, `src/ui/mobile.rs::active`,
  `src/ui.rs::compact`, `src/game/render.rs::draw`.
  **Observed:** The GDD describes features and routes, but neither README nor GDD
  declares normal/minimum supported viewports or the seven-part UI_STYLE screen
  briefs. The release QA matrix lists sizes without establishing a minimum
  support contract. Three presentation paths make inconsistent hierarchy easy
  to introduce.
  **Change:** Before implementation, add briefs for port Bridge, underway Bridge,
  PREP, active voyage, ship inspection/refit, People, Agenda, History, decisions
  and Homecoming. Specify the current decision, dominant focus, primary action,
  supporting/deferred facts, layout/framing and touch feedback for each. Adopt
  or explicitly revise the existing QA candidates: normal 1280×720, expanded
  1920×1080, compact 1024×768 and phone 390×844. Declare the actual minimum
  supported canvas and display-scale combinations after testing, rather than
  inferring support from logical resolution.
  **Acceptance:** Each phase has one stated purpose, at most 2–3 strong attention
  regions, and an explicit home for costs, warnings, utilities and recovery.
  **Verify:** Measure native and browser/embedded canvas dimensions, including
  the chosen minimum and landscape equivalent. Record which renderer is active
  across the 1100px width/640px height and logical 1200×680 dispatch thresholds.

- [ ] **UI-02 — Recompose the Bridge around the ship/current concern and retire the permanent event panel.**
  **Scope/files:** Port, calm voyage and resource-risk Bridge;
  `src/ui/bridge.rs::{draw,draw_voyage_subject,draw_attention,draw_operational_summary}`,
  `src/ui/mobile.rs::{bridge,bridge_details}`, `src/ui/responsive.rs::draw_bridge`,
  `src/ui/widgets.rs`, `src/ui/mobile/form.rs`, `src/game/render.rs::draw_notifications`.
  **Observed:** `ui_gameplay.png` has four equally bordered regions: ship,
  attention, ship/people and recent developments. Choose a charter, inspection,
  queue and History share button treatment. At 1920×1080 the fixed 400px subject
  and 250px diagram leave almost half the screen to sparse lower panels. The
  recent-log panel persists old events indefinitely. Generation repeats in the
  header and summary; compact code repeats hull/air/fuel in both columns.
  **Change:** Use a dominant ship/voyage area, one current-concern/action area and
  quiet navigation. Merge essential hull/air/fuel state with the ship; remove the
  separate Ship & people box, duplicated generation/status and repeated port
  instructions. Replace the permanent Recent developments panel with brief
  change feedback and a quiet visible History route. Keep unresolved threats
  persistent next to their recovery action even after feedback expires. Let the
  subject grow with available height. Give the phase-appropriate action distinct
  emphasis; reduce queue/inspection utility weight without hiding them. Use
  toolkit styling/input facilities when introducing action roles, not a second
  button/input implementation. On phone, put the urgent concern and recovery
  action before the diagram so long names do not push recovery below the fold.
  **Acceptance:** Within a glance, the player can identify the current concern
  and next action. Calm play has no more than three strong attention regions;
  larger windows enlarge useful ship content rather than log containers. Old
  events are retrievable in History, while current consequences remain visible.
  **Verify:** Compare gameplay, dashboard-risk and calm underway captures at
  1280×720, 1920×1080, 1024×768 and 390×844. Exercise touch-only concern → recovery
  → Bridge and History → Bridge paths. Trigger a project completion/resource
  change, wait for feedback to expire, and verify persistent state and history.
  Include simultaneous reserve warning and due obligation; neither may disappear
  merely because only one primary action is emphasized. Depends on UI-01.

- [ ] **UI-03 — Separate time controls from utilities and show whether time can actually advance.**
  **Scope/files:** Phone shell, port, Homecoming and blocking decisions;
  `src/ui/mobile/layout.rs::Layout::new`,
  `src/ui/mobile.rs::{draw_mobile_chrome,draw_mobile_time_controls}`,
  `src/ui/time_controls.rs`, `src/ui/responsive.rs::draw`,
  `src/game/render.rs::draw_overlays`, `src/game/realtime.rs`.
  **Observed:** Phone captures place Pause and Utilities in the same equal-width
  row, visually mixing a gameplay control with the menu route. Pause and all
  speed buttons remain prominent in port and Homecoming captures, although
  those phases cannot advance voyage time (GDD §3).
  **Change:** Move Utilities to a clearly separated, quieter navigation location.
  Group Pause/Resume and speed together. In phases with no running clock, replace
  the full speed bank with a concise In port/Report review state; retain any
  relevant next-voyage speed preference in a discoverable disclosure. Distinguish
  explicit pause, tutorial hold, fuel stall and decision fallback. Keep a direct
  Pause control available when the separate decision countdown can run.
  **Acceptance:** The player does not mistake menu access for a voyage control
  or infer that choosing 3x will progress a docked ship. Decisions still permit
  immediate touch pause and clearly show whether fallback time is stopped.
  **Verify:** At 1280×720, 1024×768 and 390×844, tap through port → launch → pause
  → decision → resume → Homecoming. Check speed preservation and the fallback
  countdown with no keyboard. Repeat at 200% UI/150% text, using the minimum
  supported combination established by UI-01; utilities and recovery remain
  reachable without sharing the gameplay-control group.

- [ ] **UI-04 — Put departure readiness before briefing detail and make launch the clear commitment.**
  **Scope/files:** Selected charter/PREP and tutorial provisions lesson;
  `src/ui/prep.rs`, `src/ui/mission.rs::draw_selected`, `src/ui/charter_approach.rs`,
  `src/ui/contract_systems/outlook.rs::draw_posture`,
  `src/ui/mobile/voyage.rs::{build_selected_charter,build_approach_choices,build_departure_provisions}`,
  `src/ui/tutorial.rs`.
  **Observed:** Desktop PREP divides attention among provisioning, full briefing,
  approach and posture. Launch has the same styling as refuel/cancel. Approach
  explanations use 10px text and the locked reason 9px. On phone, the full
  narrative and every approach precede provisions; the selected approach is
  described again in the alternatives. The tutorial capture asks for provisions
  while displaying briefing prose. `PROVISIONS REVIEWED` is emitted
  unconditionally in the mobile builder, even outside its tutorial need.
  **Change:** Start with charter goal/duration, readiness and consequential
  warnings. Keep purchase costs beside food/parts/fuel controls. Place an
  emphasized launch/review-launch action beside the aggregate shortages and
  obligation defaults, with access to every affected duty before commitment.
  Collapse full narrative behind Read briefing; show selected approach/posture
  summaries with explicit comparison controls. Only expand alternatives while
  choosing, with readable effects, eligibility and locked-at-launch wording.
  Remove duplicate approach summaries and gate the tutorial acknowledgement to
  its lesson. Make the tutorial open/scroll to the actual provisioning section.
  Replace tiny prerequisite prose with a wrapping readable row near its action.
  **Acceptance:** The initial phone view identifies the departure decision and
  reaches provisioning directly. Launch is distinct from cancel/refuel; full
  costs, fuel replenishment assumptions, shortages and default consequences
  remain available before launching. No completed tutorial instruction persists
  as ordinary gameplay chrome.
  **Verify:** PREP at 1280×720, 1920×1080, 1024×768 and 390×844, with long briefing,
  locked approach, insufficient funds, several conflicting obligations and
  tutorial on/off. Touch path: select charter → compare approach → return → buy
  provisions → inspect conflicts → launch or choose another. Recheck 150% text
  and 200% UI reflow without requiring users to shrink text. Depends on UI-01
  and the action hierarchy introduced in UI-02.

- [ ] **UI-05 — Make the ship schematic useful and identifiable at each layout size.**
  **Scope/files:** Bridge, Systems and loadout preview;
  `src/ui/ship_schematic.rs::build`, `src/ui/ship_schematic/draw.rs`,
  `src/ui/subsystems/overview.rs::{draw,select_compartments}`,
  `src/ui/mobile/form.rs::{Form::vessel,draw_vessel}`,
  `src/ui/ship_builder/preview.rs`.
  **Observed:** Phone Bridge captures show unlabeled compartment outlines and
  color/pip signals. `draw_vessel` draws no labels and accepts no pointer/actions,
  unlike the desktop selectable schematic. Desktop labels are dimmed for vacant
  posts; in `ui_subsystems.png`, lower captions sit very close to their glyphs.
  The loadout preview scales a 1900×330 source into a shallow strip and labels
  its tiny modules at 10px. There is no demonstrated camera/picking defect:
  these are schematic framing and affordance issues.
  **Change:** After Bridge recomposition, fit the diagram to its actual container
  with reserved label bands and a readable minimum compartment size. Provide
  a visible Inspect compartments action on compact/phone Bridge that opens a
  named list or selected-system inspector; support direct selection only where
  targets fit. Provide names and textual condition/vacancy on selection, not
  color alone. Keep vacant compartment names readable. In refit comparison,
  allow a larger inspectable preview on demand and move tier-explanation prose
  to contextual help. Avoid adding free-pan/zoom unless fitting cannot serve the
  inspection task; any zoom introduced needs visible touch controls and reset.
  **Acceptance:** A phone player can identify and inspect a failing compartment
  without guessing colored boxes. Diagram labels do not crowd glyphs; all hull
  silhouettes fit. Catalogue comparison remains the dominant refit decision.
  **Verify:** Normal/minimum sizes from UI-01 plus 1920×1080; barge, corvette,
  ring and ark; healthy, damaged, vacant and active-project compartments. Tap
  inspection, select each system and return; repeat after resize and scale
  changes, checking visible target bounds against picking. Depends on UI-02.

- [ ] **UI-06 — Split immediate subsystem care from institutional and culture detail.**
  **Scope/files:** Ship/Systems selected compartment, port and underway;
  `src/ui/subsystems.rs::{draw_card,draw_card_identity,draw_card_bars,draw_card_actions}`,
  `src/ui/subsystems/overview.rs`,
  `src/ui/mobile/ship/systems.rs::build_system_detail`.
  **Observed:** The selected desktop card always combines condition, knowledge,
  tier, school, archive, custodian, culture effects, memory and grievance before
  a row of repair/upgrade/train/institution controls. Culture uses a fixed
  76px text block at 10px. Mobile also inserts full culture detail before repair.
  The subsystem is already selected contextually; the remaining problem is the
  depth of information shown for every selection.
  **Change:** Lead with condition, repair capability, knowledge prerequisite and
  relevant repair/train action including cost/shortfall. Move culture history,
  exact passive modifiers, school/archive management and custody comparison into
  explicit Continuity & culture disclosure. Keep an active grievance, expiring
  school or lost repair capability summarized next to the affected decision,
  with a direct route to its detail. Remove duplicate custodian/tier labels and
  empty memory/grievance prose from the default care view. Reflow detail instead
  of reducing font size to fit the fixed card.
  **Acceptance:** A new player can diagnose and address a damaged system without
  reading institutional history; advanced care remains discoverable and no
  urgent consequence or purchase prerequisite is concealed.
  **Verify:** At 1280×720, 1024×768 and 390×844, inspect a damaged low-knowledge
  system, a sound system and a late-game school/custody case. Touch repair/train,
  open continuity, review cost and return. Check long custodian/memory text at
  150% text and no accidental purchase during scroll. Depends on UI-05.

- [ ] **UI-07 — Focus the active voyage on progress and the current obstacle.**
  **Scope/files:** Voyage/Contract during travel, operation, fuel stall and return;
  `src/ui/contract_systems.rs::{draw_active,draw_contract_progress,draw_contract_milestones,draw_return_button}`,
  `src/ui/contract_systems/outlook.rs`,
  `src/ui/mobile/voyage.rs::{build_active_voyage,fuel_status,posture_summary}`.
  **Observed:** `ui_contract_active.png` devotes most emphasis to bright success
  metric bars and a full-width Cancel mission / Return home control. A real fuel
  stall appears as a small text row in the right column. Phase/milestones repeat
  across the two panels; outlook and posture add nested borders and small prose.
  The screenshot's empty outlook still occupies its own bordered panel.
  **Change:** Make route/phase, objective and next milestone the dominant summary.
  Promote a current stall to an actionable obstacle with the existing recovery
  route. Disclose full scoring weights, completed milestones, locked approach
  effects and posture alternatives on demand. Keep current posture and upcoming
  actionable forecast concise; remove the empty outlook box and repeated phase
  facts. Retain a visible secondary Review return home action and its explicit
  consequence confirmation, without making cancellation the normal focal action.
  **Acceptance:** A stalled player first sees why progress stopped and where to
  recover. A calm player sees mission progress; score analysis is one explicit
  disclosure away. Retreat stays discoverable and reversible until confirmation.
  **Verify:** 1280×720, 1920×1080, 1024×768 and 390×844; fuel stall with/without
  scoop recovery, calm operation, reached milestones and return. Touch through
  recovery, score disclosure, posture review and return-home/cancel-review paths.
  Confirm critical warnings remain after temporary feedback ends. Depends on
  UI-02/03 and reuse the disclosure treatment from UI-04.

### Further inspection required — not verified defects

- [ ] **UI-08 — Verify the revised hierarchy in the live browser and dense interaction states.**
  **Scope/files:** All affected screens; `scripts/capture_ui.ps1`,
  `src/game/capture_scenes.rs` and children, `src/ui/mobile/form.rs`,
  `src/game/render.rs`, `docs/release/QA_AND_OPERATIONS.md`.
  **Evidence gap/effect:** Existing still images cannot establish current touch
  target size after scaling, scroll-versus-tap behavior, focus/picking after
  resizing, tutorial dismissal, or temporary feedback timing. No such interaction
  failure is asserted by this audit. People/History/Agenda expanded collections,
  extreme scale combinations and current-build overlays need broader sampling.
  **Action:** After each affected implementation, regenerate supported scenes
  directly in `docs/verification/`, replacing equivalent captures. Review normal
  1280×720, expanded 1920×1080, compact 1024×768, phone 390×844 and the declared
  minimum canvas, including actual embedded browser dimensions. Sample UI scale
  75/100/200% and text 75/100/150%, long names/large values, full queues, extended
  family/history, first-use guide, selected inspector, authority/event decision,
  critical-air recovery, Homecoming and terminal state. Exercise touch-only
  navigation, inspection, drag, purchase, pause/resume, confirmation/dismissal,
  Help reopening and save/menu recovery. Check effective 44px targets after
  viewport conversion rather than relying on logical button heights. Observe
  notifications at arrival and after expiry, including whether they cover actions.
  Fix only reproduced issues; merge overlapping work into UI-02–07 and record
  residual findings with exact state, size and reproduction path.
  **Acceptance:** Reviewers can identify the current decision and primary action
  within roughly a second; no more than three strong regions compete during
  normal play. Costs, warnings and recovery survive disclosure/reflow. Text,
  targets and scroll endpoints remain usable; no necessary gesture is hidden.
  Record checked interactions and any untested combinations explicitly. Run
  `.\publish.ps1` without parameters after meaningful game changes, and report
  its result or blocker; geometry checks and compilation are supplementary.
  This audit itself changes documentation only and does not require publishing.
