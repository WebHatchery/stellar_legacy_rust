# Ship-work audit validation - 2026-09-07

This records the 0.2.1 correction pass against `ship_work_implementation_plan.md`.
It is an implementation and automated-validation record, not a completed human release sign-off.

## Corrected audit findings

- Air grace advances once per voyage month. Critical warnings stop the real-time burst, and action/event observations do not spend another month.
- Expertise loss suspends investments with their progress and escrow. Manual pause refuses a full waiting list. Paused jobs have priority controls, and reordering skips completed/running entries.
- Materials become committed monthly, including unfinished stages. Cancellation uses the same authored refundable-resource mask and ledger as its preview. Fractional settlement change persists in the save; repeated restoration does not discard or create whole materials through rounding.
- New quarters projects deliver ten recovery stages. Half completion retains five stages through the food-work pivot and cancellation. Existing 0.2.0 quarters retain their original final-only delivery contract.
- Readiness includes actual spoilage, corrected fuel division, recovery hysteresis, and observed trends. Completed hydroponics changes real production. Authored blight aftermath reduces production until resolved; sterilisation does not refund original event losses.
- Emergency stabilisation remains reachable from Agenda after reviewing the initial warning. The engineering evidence shows actual bay condition independently of hull.
- The Agenda exposes scrollable recovery actions and project comparisons before pause/resume/cancel. Desktop and narrow rendered captures are in `docs/verification/`.
- Saves validate project references, sequence IDs, delivery histories, escrow bounds, and capabilities; invalid investments follow quarantine. Unsupported versions fail. Newly terminal legacy states receive a paused migration notice.
- The reserve cohort now compares three policies through the existing mission driver, with detailed per-run measurements in `ship_work_cohort.csv`.

## Comparable automated cohort

All policies use the same six reserve-baseline charter selections, three legacies,
and seeds 0-15: 288 campaigns per policy, 864 total. All use the existing affordable
council-choice policy, port preparation, and emergency food purchases. NoProjects
performs no underway ship work or emergency stabilisation. Reactive queues repair
below 50% and training below eligibility. Prepared acts below 70%, trains earlier,
and queues seed/hydroponics preparation when its waiting list is empty. These are
policy comparisons, not human win rates. Each advances the authoritative clock one
month at a time; there is no separate economy simulator.

| Measurement | No projects | Reactive | Prepared |
| --- | ---: | ---: | ---: |
| Completed | 252 / 288 | 288 / 288 | 288 / 288 |
| Dynasty endings | 0 | 0 | 0 |
| Hull endings | 35 | 0 | 0 |
| Population endings | 1 | 0 | 0 |
| Air endings | 0 | 0 | 0 |
| Timed out | 0 | 0 | 0 |
| Lowest food | 0 | 106 | 0 |
| Lowest spare parts | 0 | 0 | 0 |
| Critical months | 55,557 | 0 | 0 |
| Simulation months | 1,066,767 | 1,094,400 | 1,094,400 |
| Occupied slot-months | 0 | 847,594 | 1,202,771 |
| Two-slot utilisation | 0% | 38.7% | 55.0% |
| Distinct blocked jobs, summed across runs | 0 | 592 | 583 |
| Observed queue transitions | 0 | 13,128 | 18,125 |
| Completed recoveries | 122 | 0 | 1 |
| Total recovery months | 20,654 | 0 | 1 |
| Maximum completed recovery | 675 months | n/a | 1 month |

Critical months count end-of-month hull/air at or below the configured warning
threshold, or empty food. Minima and recovery entry are also sampled after event
resolution, before policy intervention: Prepared's zero-food minimum was a brief
intervention-time reading rather than an end-of-month critical month. Queue
transitions include starts, completions, and suspensions; they are not a count of
human clicks. Unresolved critical periods at loss are not counted as completed
recoveries. The raw report preserves each campaign for separate analysis.

Prepared play improves completion over neglect by 36 campaigns (12.5 percentage
points). It does not improve survival over Reactive in this cohort. Do not claim
that all preparation choices are necessary or that the balance is finally proven.
Both repair policies win this particular cohort; deterministic neglected-air tests
still lose at month 12, while stabilisation buys enough time for the full overhaul.
A separate paired blight test demonstrates the seed programme's lower event cost
and avoidance of persistent aftermath, with no premature partial unlock.

## Timing and layout evidence

The authored clock is one real second per simulation month at 1x: a simulation year
is 12 seconds at 1x, 6 at 2x, and 4 at 3x, before pauses or decisions. These are clock
values, not measured human decision intervals. Tutorial hold and explicit pause
freeze advancement; the critical-warning burst stop prevents queued real-time
months from consuming the recovery window.

Rendered Agenda and review captures were inspected at 1224x720 and 960x600.
Both columns scroll independently and project/recommendation controls are 60
logical pixels high (45 physical pixels at 960 width). Capture verification does
not prove every human interaction or accessibility need.

## Remaining human release gates

- Measure meaningful choices during ordinary 30-90 second stretches of 1x play.
- Complete an actual touch/click playthrough, including scrolling, service and training,
  the housing/food pivot, recovery review, tutorial, save/reload, and endings.
- Complete the clean-profile/storefront/accessibility matrix in release QA.

These gates remain open. They cannot be reported as completed by automated policy
runs, screenshots, or a successful publisher. Production deployment is a separately
authorised action and is not evidence that human acceptance has passed.


## Validation commands

`cargo fmt --check` passed. The complete Rust test run passed 521 tests, with one
pre-existing ignored test, plus the separate code-standards integration test.
Focused readiness tests passed after the final recovery-control/readout change.
Every Rust source and test file remains within 800 physical lines.
The parameterless publisher validates both platform packages and launches their
native/browser smoke checks. The final production FTP run is reported separately
in the task handoff.
