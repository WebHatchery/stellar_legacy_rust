# Stellar Legacy — Review Resolution Plan

**Review date:** 2026-08-05

**Status:** Proposed implementation plan

**Scope:** Balance, feature coherence, pacing, UI/UX, documentation, and release validation

## 1. Outcome

Bring the current mechanically complete game to a release-ready state without adding
another broad content pass. The work should preserve the deterministic simulation and
the ship-console identity while resolving the known interaction gaps, establishing
evidence for balance, and modernising the interface around clearer priorities and more
readable decisions.

The review found that the principal systems are genuinely connected: ship fittings,
subsystem condition and knowledge, crew, factions, resources, voyage drift, events,
contract scoring, rewards, the Chronicle, Heritage, and drydock persistence all affect
one another. The automated suite also exercises these connections extensively. The
remaining risk is not a missing core loop; it is the gap between mechanical correctness
and demonstrated balance, plus presentation that makes a rich simulation feel flatter
and older than it is.

## 2. Baseline and known issues

The 2026-08-05 review established this baseline:

- `cargo test` passes all 388 tests (387 crate tests plus the source-size gate).
- `publish.ps1` builds Windows and WebGL packages and deploys Preview successfully.
- The worktree is clean after validation.
- Full-voyage soak tests establish solvency and state invariants under selected seeds
  and a fixed first-choice policy, but do not establish parity across strategies,
  legacies, loadouts, charters, or random seeds.
- Displayed 1x/2x/3x speeds use internal multipliers of 3/6/9. At five seconds per
  month, a 340-year charter therefore has about 113/57/38 minutes of uninterrupted
  clock time before decisions and drydock. Longer charters can exceed the GDD's
  60-minute soft cap even at displayed 3x.
- Welcome and Help copy still describe the superseded Space/Enter advance control.
- The active-charter systems panel uses a static origin/waypoint/destination summary.
- Delegated event outcome scoring omits the GDD's documented legacy-specific modifier.
- Heritage is implemented as an automatic renown-tier head start while the GDD still
  promises a modifier selection.
- The UI has a consistent terminal identity, but uniform amber framing, dense small
  type, repeated status information, and an equally weighted navigation row weaken
  hierarchy and make the interface feel dated.
- The planned audio pass remains outstanding.

## 3. Delivery principles

1. Fix truth and interaction gaps before tuning numbers or adding presentation polish.
2. Instrument balance before changing it. A passing soak proves legality, not fairness.
3. Preserve the terminal/CRT fiction, but use it as an art direction rather than a
   constraint that makes every element look equally important.
4. Prefer shared `macroquad-toolkit` upgrades when the required UI behaviour would
   benefit other games; keep Stellar Legacy-specific layout and fiction local.
5. Keep each change narrow, deterministic where practical, and covered by the nearest
   unit or capture test.
6. Run `publish.ps1` with no parameters after every meaningful implementation phase.

## 4. Phase 0 — Release correctness and design truth

Resolve objective mismatches before judging balance or redesigning their presentation.

### Work

- Rewrite Welcome and Help copy for real-time auto-advance, Pause, and displayed
  1x/2x/3x controls. Remove every suggestion that Space or Enter advances time.
- Extend charter data with the route information needed by the active voyage view, or
  derive it consistently from authored phases and tags where appropriate. Show actual
  origin, operation site, destination/return berth, objective subsystem, current phase,
  and next unreached milestone.
- Decide the delegated-outcome rule:
  - implement a data-backed legacy-specific modifier in outcome scoring, with tests; or
  - remove the stale modifier from the GDD if delegation is intentionally neutral.
- Decide Heritage's player contract:
  - **recommended:** offer one of a small set of modifiers unlocked by the current
    renown tier when starting a new campaign; or
  - retain the automatic tier grant and update the GDD, menu copy, and store-facing
    description so no choice is promised.
- Synchronise `gdd.md`, `content_depth.md`, `README.md`, and `TODO.md` with the actual
  real-time loop, content counts, screen flow, and Heritage behaviour.

### Acceptance gate

- No player-facing or design copy describes removed controls or unimplemented choices.
- The active-contract screen contains no static route placeholder.
- Delegated outcome behaviour and the GDD describe the same scoring rule.
- Heritage has one explicit, tested design.
- `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo fmt -- --check`, and `publish.ps1` pass.

## 5. Phase 1 — Balance evidence and pacing decision

Build a deterministic balance matrix around the existing autoplay services. Do not tune
individual costs until this report exists.

### Simulation matrix

Run every charter over at least 50 seeds for:

- each legacy;
- a starter/default loadout;
- speed-, cargo-, combat-, and fuel-oriented legal loadouts;
- a conservative survival policy;
- an objective-first policy;
- the existing first-choice policy.

Record per run:

- completion, partial, pyrrhic, failure, and extinction rates;
- final score and objective fraction;
- lowest credits, food, fuel, spare parts, hull, life support, morale, and unity;
- population and faction losses;
- number and category of decisions, delegated outcomes, dilemmas, and forced returns;
- repairs, training, market trades, and emergency purchases;
- subsystem condition/knowledge at departure and homecoming;
- renown and net economic gain;
- simulated duration and estimated wall-clock duration at each displayed speed.

Emit a human-readable report that compares charters and strategies. Keep the harness
test-only or behind a dedicated analysis entry point so it cannot affect release saves.

### Initial balance targets

Treat these as hypotheses to validate, not immutable rules:

- A correctly provisioned renown-0 charter should complete or partially complete in
  roughly 70–85% of reasonable-policy runs.
- Extinction should be possible but uncommon on starter charters and materially more
  likely on explicitly extreme late-game charters.
- No legacy or purchasable component should dominate success rate and net reward across
  most objective families.
- Objective-specialist loadouts should outperform generalists in their intended family
  without becoming mandatory everywhere.
- A first upgrade should be a meaningful choice rather than an automatic purchase.
- Full repair should remain a significant but normally affordable share of a successful
  charter reward.
- Partial and pyrrhic bands should occur often enough to be real outcomes rather than
  theoretical score labels.

### Pacing decision

Choose and document one cadence model:

- **recommended:** make displayed 1x the expected readable pace and tune charter clock
  duration so an ordinary voyage, including decisions, lands within 30–60 minutes;
- alternatively, declare 3x the expected campaign pace and redesign the labels/onboarding
  around that fact.

Measure event-reading and drydock time instead of using clock-only estimates. Rebalance
`seconds_per_month`, charter years, or decision density together; changing only the
multiplier can damage the intended generational scale.

### Human validation

Complete at least three fresh renown-0 voyages by hand:

1. conservative provisioning and repairs;
2. aggressive objective/loadout investment;
3. imperfect or reactive play without foreknowledge.

Capture friction, unclear consequences, dominant purchases, dead time, and recovery
opportunities. Re-run the matrix after every tuning change.

### Acceptance gate

- A checked-in balance report records the matrix, policies, seeds, and results.
- Starter and late-game outcome distributions match their intended difficulty bands.
- No unexplained dominant legacy, component, or strategy remains.
- The documented 30–60 minute target matches measured human voyages.

## 6. Phase 2 — UI modernisation

Retain the austere generation-ship console, but replace the uniform terminal grid with a
layered command interface. Homecoming is the current visual benchmark: it has a focal
point, a narrative hierarchy, and a clear concluding action.

### Visual system

- Establish three typographic roles: a distinctive display face for major moments, a
  strong interface face for labels and controls, and a highly readable body face for
  event prose and explanations.
- Use amber for structure and institutional voice, green for healthy/actionable state,
  red for danger, and a restrained cool accent for navigation or neutral information.
- Reduce border density. Reserve strong frames for modal decisions, selected objects,
  and critical status; use spacing, subtle fills, and rules for ordinary grouping.
- Increase body-text size and line spacing at the 1280x720 logical canvas. Avoid relying
  on glow or low-contrast amber for long passages.
- Define shared spacing, type, colour, focus, hover, disabled, warning, and selection
  tokens in one UI theme layer.

### Shell and navigation

- Separate screen navigation from utilities. Move Save, Menu, Help, and Display into a
  compact utility cluster or menu rather than giving them the same weight as game tabs.
- Make drydock/underway context explicit without renumbering or visually reshuffling the
  player's mental model more than necessary.
- Add a clear current-location treatment and visible keyboard focus state.
- Preserve mouse, keyboard, and touch reachability for every action.

### Dashboard

- Recompose the screen around four priorities: current objective/phase, next required
  decision, immediate survival risks, and time control.
- Collapse duplicated hull/life/fuel information into one primary status presentation.
- Turn the log into a legible chronological feed with stronger differentiation between
  structural milestones, council decisions, warnings, and ambient fiction.
- Surface why a system is changing: trend arrows or short causal labels should explain
  deterioration and recovery without requiring a trip to another tab.

### Charter and preparation flow

- Replace database-like charter rows with mission dossiers showing route, duration,
  operation, objective, reward, provisioning burden, major hazards, loadout gates, and
  likely subsystem pressure.
- Provide useful comparison without revealing hidden event outcomes.
- Keep the launch checklist, but integrate each failed check with the action that fixes
  it and explain whether it is a hard gate or accepted risk.
- Make the selected charter and Launch action visually dominant.

### Decisions and specialist screens

- Give event modals a clearer question, stakes summary, readable story text, and choices
  that are easy to compare. Show known costs and immediate effects; label uncertainty
  rather than exposing secret rolls.
- Redesign Ship and Subsystems around the schematic as the primary navigation surface,
  with contextual condition, knowledge, crew, and action details.
- Make Crew & Dynasty distinguish succession-critical people, staffed posts, vacancies,
  faction politics, and delegation instead of presenting them as one dense ledger.
- Preserve Homecoming's strong composition and bring the same hierarchy to Chronicle
  and Heritage.

### Accessibility and responsive validation

- Validate 1280x720, 1920x1080, 2560x1440, and representative desktop browser ratios.
- Test green/amber modes, CRT enabled/disabled, keyboard-only navigation, mouse, and
  touch-sized targets.
- Ensure status never depends on colour alone.
- Add or refresh headless captures for menu, dashboard, event, prep, ship, subsystems,
  crew, active contract, drydock, debrief, Chronicle, Help, and settings.

### Acceptance gate

- Critical state and the next meaningful action are identifiable within a few seconds on
  every primary screen.
- Event prose is comfortable to read at the logical minimum resolution.
- Navigation and utilities no longer compete visually with gameplay.
- Capture review shows consistent hierarchy and no clipping across target resolutions.
- The interface retains a recognisable Stellar Legacy console identity.

## 7. Phase 3 — Audio and feedback

Add audio only after layout and pacing stabilise, so cues reinforce final interactions.

### Work

- Add restrained underway ambience with accessible volume controls.
- Add high-value cues for button activation, council alert, event resolution, phase
  transition, succession, homecoming, and game over.
- Add subtle visual feedback for milestones, warnings, resource transactions, and
  successful repairs/refits.
- Avoid continuous alarm fatigue; critical warnings must remain understandable when
  muted.

### Acceptance gate

- Audio cues correspond to meaningful state changes and never carry unique information.
- Repeated cues remain tolerable over a 30–60 minute voyage.
- Settings persist and default to a restrained mix.

## 8. Phase 4 — Release validation

### Automated

- Run the complete test, clippy, formatting, source-size, balance-matrix, and capture
  suites.
- Run `publish.ps1` with no parameters from the project directory.
- Confirm Windows and WebGL packages contain the same current assets and data.

### Manual

- Complete a fresh renown-0 charter from New Game through Homecoming and drydock.
- Start a second charter to prove persistence, rewards, wear, dynasty, factions,
  unlocked salvage, Chronicle, and Heritage behave together.
- Exercise pause/speed changes, every main screen, delegated and council decisions,
  save/load during a voyage, save/load during Homecoming, abort/force-return, and
  extinction.
- Verify the catalog thumbnail represents the current title screen.

### Release gate

The plan is complete only when documentation matches play, the balance report supports
the intended difficulty and pacing, the refreshed UI passes capture and input review,
and `publish.ps1` succeeds from a clean worktree.

## 9. Recommended implementation order

1. Release correctness and documentation truth.
2. Balance matrix and pacing decision.
3. Numerical tuning backed by the matrix and manual voyages.
4. Shared UI theme/toolkit primitives, where needed.
5. Shell, dashboard, charter/prep, event, and specialist-screen redesigns.
6. Accessibility, resolution, and capture validation.
7. Audio and feedback.
8. Final end-to-end release validation.

Do not begin another large event-content expansion until these gates are met. The current
content inventory is already large enough to expose the balance and presentation issues;
more content would increase the cost of resolving them without addressing their cause.
