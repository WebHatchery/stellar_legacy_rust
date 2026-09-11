# Open work

Current behavior is documented in [gdd.md](gdd.md). The Custodian identity pass,
obligations, officers/institutions, Agenda and responsive five-destination UI are
implemented. Completed milestone diaries are not open tasks.

## Standards alignment

The updated shared [agent checklist](AGENTS.md) and
[coding standards](CODE_STANDARDS.md) expose the following implementation debt.
The 800-line source gate currently passes, and the project has no `mod.rs` files,
but the newer function, module-documentation, test-placement, and validation rules
are not yet fully met.

- Migrate the legacy in-source tests as one deliberate change. Add `src/lib.rs` as
  the testable library surface, make `main.rs` the runtime shell, move test
  modules/helpers out of `src/**` into the crate's `tests/` directory, and expose
  only intentional public seams. The current audit found tests in 108 Rust files
  under `src/`; do not expand coverage until this migration is complete.
- Refactor functions above the new 100-line absolute maximum before extending
  their areas. The current audit found 72 such functions, with the largest
  production hotspots in `src/game/capture_scenes.rs`, `src/game/actions.rs`,
  `src/ui/subsystems.rs`, `src/ui/prep.rs`, `src/state/sim/campaign.rs`,
  `src/simulation/event_resolver/outcome.rs`, and `src/simulation/contract.rs`.
  Preserve cohesive ownership rather than moving code only to reduce line count.
- Add `//!` module documentation to the 15 production modules that still lack it:
  `src/game/capture_scenes/{agenda,display,homecoming}.rs`,
  `src/simulation/projects/{accounting,effects}.rs`,
  `src/ui/agenda/readiness_panel.rs`, `src/ui/dashboard/status.rs`,
  `src/ui/mobile/{decisions,history,menu,overlays,people,ship,voyage}.rs`, and
  `src/ui/mobile/ship/systems.rs`. Add docs to migrated test modules as they move.
- Review feature suites against the new target of no more than five high-value
  `#[test]` cases per major feature. Forty-four current test files exceed five;
  consolidate related cases with table-driven assertions where appropriate and
  record a short rationale when distinct coverage genuinely needs more.
- Audit test-only JSON parsing against the toolkit boundary. In particular,
  `src/data/tests/content.rs` directly parses embedded event/config JSON; use the
  toolkit-backed loading path for game data and reserve direct Serde parsing for
  save/chronicle migration fixtures or other non-game-data test payloads.
- Restore project-specific architecture guidance in `README.md` or a new
  `PROJECT_AGENTS.md`. The shared document describes generic `engine/`, `screens/`,
  `lib.rs`, and `Option<UiAction>` examples, while this project deliberately uses
  `simulation/`, capture-scene modules, a batched `Vec<UiAction>`, and embedded
  `GameData` registries. Document the ownership boundaries and deliberate
  deviations so future shared-document syncs do not invite incorrect refactors.
- Strengthen the standards checks in `tests/code_standards.rs` or CI. The current
  source-size test already counts every physical line, including tests, but does
  not check function length, module docs, test placement, or direct game-data
  parsing. Make the new structural rules observable without weakening the
  existing 800-line gate.
- Turn the capture-time touch audit into a release-blocking check. `src/main.rs`
  currently reports undersized and overlapping targets during screenshot capture;
  assert the 44px target and no ambiguous overlaps, then add text-bound and modal
  occlusion checks for the responsive scenes before relying on captures as
  accessibility evidence.

## UI/UX visual redesign

- Redesign the visual hierarchy so primary actions, urgent conditions and key
  information are immediately distinguishable.
- Rework screen layouts to make better use of desktop fullscreen space.
- Improve information density through clear grouping, spacing and prioritization.
- Improve navigation so destinations, current location and return paths are clear.
- Show an early before-and-after of the redesigned screens to make the visual
  changes reviewable; wording and button-behavior fixes alone do not complete this work.

## Acceptance still required

- Complete live touch/click routes for multiple-job reorder, full succession,
  due-obligation resolution into history, Homecoming into a second charter, and
  all critical-air/terminal controls. Include save/reload at each major boundary.
- Obtain fresh-player five-second Bridge recognition and council
  cost/consequence comprehension results. Measure meaningful choices during
  ordinary 30–90 second stretches at 1x and total human voyage duration.
- Complete clean-profile, display/DPI, hardware/audio/focus, accessibility and
  store-client install/update/save-survival checks on the exact candidate.
- Refresh and approve store screenshots for the final presentation. Existing
  captures and historical audits are not blanket approval of the current build.
- Resolve owner release decisions, support contact, rights/notice review,
  hardware claims and final storefront approval in
  [release QA](docs/release/QA_AND_OPERATIONS.md).

## Balance analysis maintenance

The obsolete matrix and report have been removed. The ignored
[generate_release_balance_report test](src/simulation/balance/tests.rs) still
contains a 22-charter assertion and hard-coded historical interpretation prose,
although the current registry has 23 charters. It also writes its generated files
to the repository root. Update those assumptions and choose an appropriate output
location before running it again or treating its results as release evidence.

The ordinary suite includes comparable ship-work policies; the retained
[September 7 cohort](docs/ship_work_cohort.csv) is historical comparison data,
not current human balance or pacing acceptance. Detailed cohort rows can be
generated through STELLAR_SHIP_WORK_REPORT. Automated policies do not establish
human success rates or voyage duration; use the candidate-specific routes in
[release QA](docs/release/QA_AND_OPERATIONS.md).

## Deferred design intent

These are retained design directions, not scheduled releases. Charter-specific
approaches, competing historical accounts and bounded compartment culture are
implemented; reconsider the remaining directions after the current core passes
human acceptance.

| Direction | What remains beyond current behavior | Required boundary |
| --- | --- | --- |
| Further work-management options | Protected reserves, named development programmes/trainees, standing orders and additional project slots | Add only after measured scarcity and choice quality justify more complexity |

Every accepted extension needs typed data, save compatibility, deterministic
behavior tests, touch-accessible controls and its own publisher validation.
Existing visuals supply ship/person/faction identity. Build on those assets for
the UI/UX visual redesign above rather than restarting retired plans.
