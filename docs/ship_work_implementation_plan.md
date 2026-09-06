# Stellar Legacy: ship work implementation plan

Status: proposed implementation, not shipped. Recorded 2026-09-06.

## 1. Intended experience

Stellar Legacy is a semi-idle voyage survival game. The player watches generations
pass, makes consequential decisions, and uses the quiet years to keep the next
disaster survivable. Waiting should carry anticipation: a repair is progressing,
a reserve is recovering, or a successor is learning while another weakness grows.

The loop becomes **event → consequences → assess readiness → queue ship work →
watch recovery and preparation → respond to the next interruption**. Keep the
existing voyage, council, succession, Homecoming, and Chronicle structure.

Success means returning with a viable ship and society. Failure must remain real,
legible, and final for that campaign. Semi-idle means freedom from constant input;
it does not mean guaranteed survival or progress while the application is closed.

Ship work is not crafting, room placement, tactical assignments, repair minigames,
or a separate mode. Do not add a second production economy or require routine clicks
to collect completed work. Port refits retain their distinct role.

## 2. Verified foundation and gaps

| Existing implementation | Reuse or change |
| --- | --- |
| `src/simulation/tick.rs`, `src/game/realtime.rs` | Monthly simulation, annual economy, 1x/2x/3x, and blocking decisions already exist. Advance projects on this clock; preserve hard stops. |
| `src/simulation/subsystems/verbs.rs` | Repairs and institutional training currently apply immediately; convert underway actions into projects. Keep port servicing immediate. Tier upgrades and recovered fittings remain port-only. |
| `src/state/sim/subsystems.rs` | Condition and institutional knowledge already model wear and expertise. Avoid parallel health or specialist-population meters. |
| `src/simulation/tick/economy/`, `src/simulation/subsystems.rs` | Food production, consumption, fabrication, spoilage, wear, and knowledge already interact. Projects modify these existing calculations. |
| `src/data/events.rs`, `src/simulation/event_resolver/` | Consequence tags, subsystem damage, outcome requirements, and scheduled follow-ups exist. Add actionable issue records and project capabilities without duplicating event chains. |
| `src/simulation/contract/forecast.rs`, `src/simulation/advice.rs` | Extend shared forecasts and recommendations for readiness; the UI must not invent its own economic calculations. |
| `src/simulation/mortality.rs`, `src/game/realtime.rs` | Dynasty extinction already ends advancement. Add explicit vessel/population failure handling while preserving commission-ending identity. |
| `src/state/sim/authority.rs`, `src/game/actions/authority.rs` | Preserve the Custodian/captain authority contract and existing review flow. |
| `src/save.rs`, `src/save/tests.rs` | Toolkit persistence and migration exist; explicitly migrate new runtime state. |
| `src/simulation/autoplay.rs`, `docs/food_balance.md` | Existing maintenance policy completed all 288 reported reserve-test cases. This establishes viability, not meaningful danger or human engagement. |

The supplied project costs, durations, reserve years, and illustrative deck names
are design examples, not values validated against this game's units or content.
The current design documents describe extinction as the terminal rule; any new
vessel endings must update those documents with the implementation.

## 3. First release scope

Ship the following together as the minimum complete experience:

1. Custodian Agenda with **two concurrent projects** and up to four queued jobs.
2. Readiness covering food, engineering, life support, knowledge/leadership, and
   social cohesion, with fuel shown using the existing route forecast.
3. Maintenance notices derived from existing wear, plus persistent event aftermath.
4. Explicit survival endings, warning/recovery paths, and a touch-only tutorial.

Keep two slots fixed initially. A third/fourth slot is a later balance decision,
not an automatic progression reward that removes the central tradeoff. Defer
protected stores, component inventories, named trainee selection, and standing
orders until the first release proves that choosing projects is worthwhile.

### Initial project catalogue

Durations below are initial tuning hypotheses in simulation years. Derive prices
from existing repair/training prices and measured annual parts/credit production;
record final numbers in data after the baseline runs in milestone A.

| Project | Initial duration | Cost and eligibility | Outcome |
| --- | ---: | --- | --- |
| Service subsystem | 4–8 | Existing parts/minerals repair cost; sufficient knowledge; damaged target | Existing field repair gain and ceiling, applied at completion |
| Restore hull / overhaul life support | 6–10 | Existing applicable repair resources; target below field ceiling | Restore the existing ship meter through its authoritative service |
| Train replacement cohort | 8 | Existing training credit cost; knowledge below cap | Existing academy-adjusted knowledge gain; restores repair eligibility |
| Optimise hydroponics | 12 | Parts/minerals; working agricultural subsystem | One capped, modest production modifier; does not grant a drydock tier |
| Restore crew quarters | 5 | Parts; social recovery need | Bounded morale recovery over a defined period, not permanent stacking |
| Sterilise damaged growing systems | 4 | Parts; matching agricultural aftermath | Clear that issue and its penalty; do not refund the event's original food loss |
| Establish seed programme | 18 | Parts and food; agriculture knowledge floor | Persistent capability enabling one authored blight response |

At launch offer at least three useful eligible choices spanning immediate safety,
expertise, and preparation. Do not offer pointless maintenance on pristine systems
just to fill the list. Cap permanent benefits, and explain completed/ineligible
projects. Use repeatable work only when there is an actual deficit to address.

### Project lifecycle and accounting

- A queued job is an intention: no resources charged and no benefits granted.
  Starting rechecks cost, target, knowledge, unique-effect caps, and authority.
- Deduct the full authored cost once when work starts. No recurring hidden drain
  in the first release. The occupied slot is the ongoing opportunity cost.
- Store elapsed work in integer months. Completion applies its result exactly
  once and records the date. No collection button or completion decision modal.
- Running jobs can be suspended or cancelled through visible controls. Suspension
  preserves investment/progress and releases the slot; it does not restore costs.
  Cancellation discards progress and returns no resources; preview this plainly.
- Queued and suspended work share the four-job waiting cap. Reorder with visible
  MOVE UP / MOVE DOWN controls; dragging is optional. Resuming uses the next free
  slot and does not charge again.
- Start the first eligible waiting job in player priority order; leave blocked
  jobs visible with a reason. Never silently replace the player's selection.
- Evaluate eligibility and fixed monthly progress from the month's starting
  state. Loss of required expertise suspends affected work; training itself must
  remain possible at low knowledge to avoid a circular recovery lock.
- New event damage changes the target, not elapsed labour. Repair completion adds
  a bounded gain to current condition; it never restores a saved old condition or
  erases unrelated damage. An already repaired/removed target stops work with a
  clear explanation, with no duplicate payout.
- Projects advance only during unpaused voyage months. On Homecoming, unfinished
  work carries into port suspended and resumes next voyage if still eligible.
  Port actions revalidate affected jobs; no hidden instant project completion.
- Save running, waiting, suspended, and completed-capability state. No offline
  advancement or calendar-time catch-up in this release.

Use constant progress initially rather than another crew-capacity simulation.
Add speed modifiers only if later evidence warrants their forecasting complexity.

## 4. Readiness, aftermath, and maintenance

The dashboard gives a compact readiness summary and the two active jobs. Tap
AGENDA to inspect active work, recommended responses, and waiting jobs. A readiness
row names the concern, evidence, trend, and an exact action such as QUEUE SERVICE.
Recommendations are advisory and never spend resources.

Separate two food readings: **stored food / current annual consumption** is gross
reserve coverage; **stored food / net annual deficit** estimates depletion when
consumption exceeds production. If net production is positive, say “Surplus at
current conditions,” not infinite safety. Include route tolls and existing spoilage
where applicable. Label projections as current-condition estimates; show known
project effects separately until completed. Fuel uses remaining travel burn and
scoop assumptions, not a misleading constant-years conversion during operations.

Readiness states are Strong, Stable, Vulnerable, and Critical. Data defines their
thresholds and recovery hysteresis. Condition, knowledge, food flow, and existing
social statistics supply the evidence. Do not claim “only one physician” unless
the actual roster supports it. Recommendations rank imminent survival threats,
aftermath, then preventative work; show blocked options with their prerequisite.

An `Issue` records stable ID, source event/maintenance trigger, target, created
month, severity, due month when relevant, recovery project IDs, and resolution.
Event outcomes create/merge issues through the same resolver used by manual,
delegated, and timed choices. Event damage is applied once; an issue describes
that damage and adds only explicitly authored ongoing effects. Fixing an issue
removes its active penalty but preserves the historical decision/consequence.

Maintenance begins with existing subsystem condition and wear forecasts, not a
second wear meter. Crossing a service threshold opens one issue per target/type.
Service recommendation dates are estimates; explicit authored failure deadlines
must be labelled as deadlines. Overdue maintenance can raise bounded failure risk
or trigger a scheduled failure, and the resulting event cites the earlier notice.
It must not apply both a new failure and the old threshold event for the same cause.
Recovery hysteresis rearms notices only after meaningful recovery.

The first content slice connects one agricultural crisis to sterilisation and a
later seed-programme payoff, plus one engineering maintenance warning to a failure
and recovery. Use existing subsystem IDs rather than introducing a deck simulation.
Preserve at least one unconditional legal event response. Consume a consumable
capability only when its chosen outcome commits, never during preview.

Record meaningful starts, completions, cancellations, neglected failures, and
preparation payoffs in the ship log and selected Homecoming/Chronicle highlights.
Avoid monthly progress spam. Link outcomes to their source issue/project.

## 5. Loss, recovery, and attention contract

Preserve dynasty extinction: the founding commission ends and the Custodian is
archived/decommissioned. A captain's death alone does not end a campaign with a
valid successor. Add a shared terminal outcome with reason and causal evidence;
do not overload the dynasty-extinct flag to mean every kind of defeat.

Proposed vessel survival rules for the first release:

| Condition | Consequence |
| --- | --- |
| Hull integrity reaches zero | Immediate vessel loss after the authoritative mutation resolves |
| Total population reaches zero | Immediate campaign loss |
| Life support remains at zero for 12 consecutive simulation months | Vessel loss; show a dated emergency countdown on entry; restoration above zero clears it |
| Food runs out | Existing hunger/population consequences continue; no instant food-zero game over |
| Fuel runs out | Existing stall, wear, and morale consequences continue; no arbitrary fuel-zero game over |
| Low morale/unity or poor charter score | Pressure and mission consequences, not extra hidden terminal thresholds |

The 12-month air grace is provisional. Pair it with a visible emergency stabilise
action, available even with low expertise: a bounded one-use-per-crisis response
that consumes an authored available resource or exact disclosed population cost
and restores enough air to attempt recovery. It cannot fully repair the ship or
be repeatedly farmed. Validate that ordinary warned states have a reachable
recovery sequence; neglect or explicit costly risks can still make rescue impossible.

Show critical warnings before hull reaches zero. Audit event deltas so a healthy,
prepared ship cannot suffer an untelegraphed unavoidable terminal hit. Risky choices
must disclose potential vessel loss. Critical countdown entry automatically pauses
once and presents REVIEW RECOVERY and RESUME; ordinary project completions use a
nonblocking notice. No repeated pause while the same condition remains critical.

Evaluate terminal conditions after each authoritative event/action and simulation
step, before awarding Homecoming success. If completion and failure share a month,
failure wins. Stop further economy, project, event, and contract processing after
the terminal result. Record one ending and provide visible CHRONICLE, NEW GAME,
and MENU controls. Loading an ended campaign must preserve its terminal state.

While running at 1x/2x/3x, routine work proceeds unattended and existing delegated
events can resolve. Explicit PAUSE freezes project progress and decision countdowns.
Closing the game freezes the campaign. Never advertise unattended safety while
the simulation is running. The tutorial must teach both queueing and pausing.

## 6. Technical implementation boundaries

Add small modules beside the existing systems:

| Proposed module | Responsibility |
| --- | --- |
| `src/data/projects.rs` and `assets/projects.json` | Typed definitions, effects, prerequisites, catalogue validation |
| `src/state/sim/projects.rs` | Serializable project state, unique capabilities, stable sequence IDs |
| `src/state/sim/issues.rs` | Persisted active/resolved issue references and warning acknowledgement |
| `src/simulation/projects.rs` with child modules | Eligibility, payment, queue commands, progression, completion |
| `src/simulation/readiness.rs` | Pure readiness/forecast/recommendation models shared by UI and autoplay |
| `src/simulation/issues.rs` | Aftermath creation, deduplication, maintenance transitions |
| `src/simulation/survival.rs` | Failure checks, grace countdown, emergency stabilisation |
| `src/ui/agenda.rs` with child modules | Touch layout and commands, no simulation mutation during drawing |

Route new JSON through `macroquad_toolkit::data_loader` via `src/data.rs`, following
the existing catalogue pattern. Keep game-specific validation here. Reuse toolkit
panels, scrolling, input, persistence, and timing; propose a toolkit improvement
only for an actual shared gap. No new dependencies are expected.

Use stable ordering for simultaneous project completion and issue creation; never
let HashMap iteration determine outcomes. For each month: capture eligibility,
advance the existing annual economy when due, check terminal state, advance project
work/apply completions, evaluate air grace and terminal state, then proceed through
contract/mortality/events with terminal checks after mutations. Jobs started by a
completion earn their first month next month. Completion effects apply to future
economic ticks, not retroactively to the annual tick just settled. Regression tests
must pin this ordering, especially on Homecoming and year boundaries.

Version the save change and migrate old saves with empty queues/capabilities, no
invented historical issues, and maintenance recommendations derived on the next
safe update. Existing zero-air saves receive the full grace period and an initial
paused warning; other newly terminal legacy states receive an explicit migration
notice before resuming evaluation. Preserve resources and existing obligations.
Validate referenced IDs and duplicate runtime IDs; quarantine unsupported/corrupt
saves through existing toolkit behavior rather than silently deleting investments.

Keep every Rust source/test file at or below 800 physical lines. Put tests in
separate child files and use named module filenames, never new `mod.rs` files.

## 7. Delivery sequence and acceptance gates

Each milestone is a separate useful change: implement, validate with the project's
parameterless publisher, inspect the working tree, and commit before the next one.
Do not call the whole feature released until milestone E passes.

### A — Baseline and survival contract

- Capture fixed-seed baseline outcomes, food/parts curves, repairs per voyage,
  failure reasons, and current real-time years at all three speeds.
- Implement shared terminal state, explicit failure checks, air grace/recovery,
  ending UI, save migration, and critical warning pause behavior.
- Update GDD sections 3, 4, 7, and identity continuity documentation for new endings.
- Gate: extinction still works; terminal transitions occur once; no Homecoming
  reward after same-month loss; warned air failure has a demonstrated recovery.

### B — Queue and actionable interface

- Implement catalogue/state, two slots, four waiting jobs, lifecycle commands,
  save support, and basic Agenda/dashboard progress.
- Convert underway repair/training entry points, including autoplay, to queue
  commands so old instant actions cannot bypass duration. Keep port behavior.
- Gate: complete a service and training project through visible controls;
  suspension/reorder/reload preserve progress and costs; insufficient funds and
  lost expertise give precise reasons; no duplicate completion effects.

### C — Readiness and persistent consequences

- Add shared readiness projections, maintenance issues, agricultural aftermath,
  and the engineering warning/failure/recovery sequence.
- Integrate issues with existing event arbitration and log/debrief records.
- Gate: one event creates a persistent concern, its project resolves it, ignoring
  a maintenance warning produces a linked consequence, and all forecasts match
  simulation assumptions for surplus, deficit, fuel stall, and condition changes.

### D — Preparation pays off

- Add capped hydroponics optimisation, quarters recovery, seed programme, and a
  gated blight outcome that recognises preparation.
- Teach exact taps: OPEN AGENDA, choose a project, tap QUEUE, then RESUME.
- Gate: demonstrate prepared/unprepared versions of the same seeded crisis;
  both have legal choices but materially different costs. Quiet launch years
  offer at least three useful choices for two slots without compulsory chores.

### E — Balance, accessibility, and release

- Compare no-project, reactive-repair, and preparedness policies on the same
  seeds, legacies, and starter/long charters used by the reserve baseline.
- Report completion, each terminal reason, worst food/parts values, critical
  months, project utilisation, blocked jobs, queue changes, and recovery latency.
  Separate automated-policy results from human playtest findings.
- Require prepared play to improve survival or losses over neglect on the same
  cohort; include deterministic neglect cases that genuinely lose and crisis
  cases that prepared play rescues. Avoid targeting universal wins or a guessed
  global loss percentage. Keep the instructed tutorial reliably recoverable.
- Check long-voyage production so permanent bonuses do not erase scarcity. Tune
  capped benefits, costs, and upkeep before adding more resource drains. Review
  `docs/food_balance.md` with measured new results.
- Human playtest target: at least one meaningful choice in a typical 30–90 seconds
  of quiet 1x play, with no required repeated click cadence. Measure actual timings;
  adjust duration/content, not raw popup count. At 3x critical notices must stop
  before a player loses the recovery window.
- Verify all actions by touch/click at desktop and narrow browser sizes, with
  scrolling, readable reasons, exact tutorial control names, and visible pause.
  Replace matching screenshots directly under `docs/verification/`.
- Update README/GDD/event authoring/TODO to distinguish shipped scope from later
  work. Run `./publish.ps1` with no parameters and report its actual result.

### Required regression scenarios across milestones

Test annual/monthly completion boundaries; equivalent seeded outcomes at each
speed; no advancement while paused, in port, or on a blocking decision; event
damage during a repair; knowledge loss and training recovery; queue affordability
changes; cancel/resume without duplication; old/new save round-trips; delegated
and timed aftermath; warning deduplication; gate capability availability; and
failure during event resolution, project processing, and charter completion.
Extend the existing autoplay services rather than simulating a separate game.

## 8. Later extensions, only after the core is validated

1. **Protected reserves and spares:** use transfers from existing stores, never
   creation of free food. Define capacity, spoilage, draw permissions, and automatic
   spare consumption. A reserve target cannot fill without actual surplus.
2. **Development programmes:** build on institutional knowledge and real crew
   identities; define death/replacement and succession rules before candidate UI.
3. **Standing orders:** explicit opt-in, disclosed thresholds/cost ceilings,
   hysteresis, and named acting authority. Use existing captain objection review;
   automation cannot bypass affordability or protected stores. Avoid oscillating
   rationing and repeated objections for unchanged policy.
4. **Additional slots/content:** only if two slots produce excessive blocking
   rather than interesting choices. Keep slots scarce and modifiers bounded.

The first implementation task is milestone A. The complete first playable ship-work
slice is A through D; milestone E is the release gate, not optional polish.
