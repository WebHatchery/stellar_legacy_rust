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
  milestones and return.

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
