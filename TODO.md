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

## Code review — 2026-11-03

Source review of the save/load path, real-time clock, decision timeouts, tick,
data loading and UI panics. Strict Clippy is clean and every `.rs` file is
under 800 lines (largest: `game/capture_scenes/routes.rs`, 766). Nothing was
run interactively; the findings below come from reading the code.

- [x] **CR-01 — A missing legacy dilemma locks the clock (agent).**
  `legacy::resolve_dilemma` returns `None` without clearing
  `sim.pending_dilemma` when `pending_dilemma_def` finds nothing (an unknown
  `legacy_id` or `dilemma_id` from an older save) or the option index is out of
  range. The event modal draws nothing for it, and once the timeout passes
  `Game::auto_resolve_decision` (`game/actions/decision.rs`) runs again every
  frame: a new "Captain … took the highest-probability legacy response" log
  line and warning toast each frame, and time never moves again.
  **Action:** clear an unresolvable pending dilemma the same way the event
  path clears an unknown template (`None => sim.pending_event = None`). Log the
  fallback only when `resolve_dilemma` returns `Some`.
- [x] **CR-02 — Save validation only checks the Agenda (agent).**
  `save/validation.rs` checks projects, capabilities and issues. It does not
  check `legacy.legacy_id`, `pending_dilemma`, `pending_event.template_id`,
  `contract.template_id`, the subsystem keys against `data.subsystems`, or the
  faction ids. A save that names removed content loads, then fails silently or
  locks up (see CR-01) instead of hitting the clear quarantine-and-error path.
  **Action:** reject unknown catalog ids in `validation::validate` so they
  take the existing quarantine path.
- [x] **CR-03 — Loading a save parses the game data again (agent).**
  `save::load_campaign` calls `GameData::load()` inside the `and_then`, and
  `migrate_save_value` calls it again for migrated saves. That is up to two
  extra full parses of every embedded JSON file. The validation data also skips
  the `build_mode::load_data` adjustments.
  **Action:** pass the `&GameData` that `Game` already owns into
  `load_campaign` / `migrate_save_value`, and validate once.
- [x] **CR-04 — Config values the code divides by are never checked (agent).**
  `GameData::load` validates `projects.json` and its references, but nothing
  checks `game_config` values. `campaign.months_per_window == 0` divides by
  zero in `event_resolver/skeleton.rs::generate_beats`.
  `crew.recruit_age_max < recruit_age_min` underflows a `u32` in
  `simulation/crew.rs`. `real_time.seconds_per_month <= 0` is quietly clamped
  to 0.01 s in `game/realtime.rs`, and `decision_timeout_secs <= 0`
  auto-resolves every decision on the first frame.
  **Action:** add a `GameConfig::validates()` that fails fast with labelled
  errors, and call it from `GameData::load`.
- [x] **CR-05 — Old-style test modules still live in `src/` (agent, separate change).**
  61 `#[allow(dead_code, unused_imports)] mod tests { include!(…) }` blocks
  pull `tests/unit/**` into `src/`, against CODE_STANDARDS §11.7. They have no
  `#[cfg(test)]`, so non-`#[test]` helpers and imports also compile into
  release builds; the lint allow hides this.
  **Action:** in a dedicated pass, delete the cases §11.3 rules out (UI
  layout, experimental rules, balancing, internals). Move the rest into
  `tests/` against the `lib.rs` public API, then remove the `include!` blocks.
  **Done 2026-11-03:** 61 include blocks and 107 suite files removed. Kept:
  `tests/data_integrity.rs` (content loads, save label matches Cargo) and
  `tests/save_round_trip.rs` (round trip, corrupt-save quarantine), reached
  through a narrow persistence seam in `lib.rs`. The autoplay soak and the
  release-balance matrix stay in place for now (see CR-09).
- [x] **CR-06 — Stale comments and dead code (agent).**
  `game/realtime.rs`: the doc comment for a deleted run-timer function
  ("The cosmetic run timer's elapsed seconds (PLAN M4.7)…") is now attached to
  `decision_remaining`. `simulation/tick.rs`: the `TickReport` doc comment
  starts mid-sentence ("to: log lines…"). `lib.rs::report_touch_targets`: the
  `for` loop that prints clashes comes after `assert!(clashes.is_empty())`, so
  it can never run.
  **Action:** delete the orphaned comment, rewrite the `TickReport` summary,
  and either print the clashes before asserting or drop the loop.
- [x] **CR-07 — UI code panics on conditions that other code guarantees (agent, low).**
  `ui/contract_systems.rs::draw_active` (`contract.as_ref().unwrap()`),
  `ui/mission.rs::build_abort` (two unwraps behind `can_return_home`) and
  `ui/agenda.rs` (`waiting_position(..).expect("waiting job")`) depend on
  checks made in another function. If a future caller skips one of those
  checks, the frame panics.
  **Action:** switch these to `let … else { return; }` so the draw call
  degrades to an empty panel instead of a crash.
- [ ] **CR-08 — The function-length gate fails on two draw functions (agent).**
  `cargo test --test code_standards` fails at HEAD:
  `production_functions_stay_under_the_limit` reports
  `ui/contract_systems/outlook.rs:45-145` (101 lines) and
  `ui/event_modal/council.rs:153-271` (119 lines), over the 100-line limit.
  **Action:** split each function into cohesive helpers (not compressed
  formatting), and recapture the active-contract and event screens.
- [ ] **CR-09 — The autoplay soak and balance matrix still live in `src/` (owner decision).**
  `simulation.rs` still uses `include!` to pull in
  `tests/unit/simulation/autoplay.rs` (the owner's playtest soak, 12 tests) and
  `balance.rs` (the ignored release-balance report) as uncfg'd modules, so they
  compile into release builds. They drive private services, so they can't move
  to `tests/` without a wide public API.
  **Action:** choose between a feature-gated `analysis` module or binary
  (`cargo run --features analysis --bin balance`) and deleting them. The crate
  also has no asset-registry integrity gate, which §11.2 says every crate
  keeps; add one alongside.
