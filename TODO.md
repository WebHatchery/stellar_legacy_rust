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

### Review record — 2026-11-02

UI-01 to UI-07 are implemented (GDD §9.1 briefs, Bridge, time controls,
PREP, schematic, subsystem card, active contract). The UI-08 review then:

- Recaptured the full 25-scene stable set at 1920×1080 and read each back.
- Ran the capture text/modal audit at 150% for gameplay, prep, posture,
  subsystems, institutions, contract_active, event_fuel_paused, debrief,
  chronicle, people_officers, market, drydock and (after UI-09/10) event,
  agenda, ship, ship_refitted, ship_underway, ship_underway_ark, tutorial,
  settings and gameover; at 125% for agenda, gameplay, tutorial, event and
  settings; and at 75% for each of those. All pass after the fixes below
  and in the UI-09 and UI-10 commits.
- Fixed during review: schematic condition percentages moved below each
  box (they collided with codes at 150%); header status lines start past
  the brand block; the clock status line moved to the navigation row; the
  PREP charter title cleared its panel header; posture effects and lock
  lines anchor above the button row.

Not exercised (captures only, no interactive run): navigation, drag,
purchase, pause/resume, confirmation/dismissal, Help reopening, save/menu
recovery, notification arrival and expiry, long names and large values,
full queues, extended family/history, critical-air recovery, the live
guide-hold and fuel-stall status lines, a calm contract with reached
milestones, and the scoop-less stall wording.

### Follow-up tasks

- [ ] **UI-11 — Interactive pass in the running build (human).**
  **Action:** exercise the "not exercised" list above in the Windows or
  browser build and record outcomes here. Needs a person at the controls;
  capture scenes cannot press, drag or wait for notifications to expire.
