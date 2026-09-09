# Balance evidence and limits

The current clock, survival rules and content counts are in [gdd.md](gdd.md).
The measurements below are retained comparison evidence, not a claim that the
current release has passed human balance or pacing acceptance.

## Current automated coverage

The ordinary tests under [autoplay/tests](src/simulation/autoplay/tests) exercise
legal decisions, maintenance, readiness, staged projects and survival through the
authoritative simulation. The ship-work comparison test optionally writes detailed
rows through `STELLAR_SHIP_WORK_REPORT`.

The checked-in [ship-work cohort](docs/ship_work_cohort.csv) records the 0.2.1
September 7 audit: six starter/long charter selections, three legacies and seeds
0–15 under three policies, 288 campaigns each. All share affordable decisions,
port preparation and emergency food purchasing. NoProjects omits underway work
and emergency stabilisation; Reactive repairs below 50%; Prepared acts below
70%, trains earlier and queues seed/hydroponics preparation.

| Measurement | No projects | Reactive | Prepared |
| --- | ---: | ---: | ---: |
| Completed | 252 / 288 | 288 / 288 | 288 / 288 |
| Hull endings | 35 | 0 | 0 |
| Population endings | 1 | 0 | 0 |
| Dynasty / air endings | 0 / 0 | 0 / 0 | 0 / 0 |
| Lowest observed food | 0 | 106 | 0 |
| Lowest spare parts | 0 | 0 | 0 |
| Critical months | 55,557 | 0 | 0 |
| Two-slot utilization | 0% | 38.7% | 55.0% |

Minima include event-resolution points before policy intervention; Prepared's
zero-food sample was not an end-of-month critical period. The cohort demonstrates
value over neglect, **not superiority of Prepared over Reactive**. Separate
neglected-air tests fail at month 12 and paired blight scenarios check seed
preparation's payoff. These policies are not human success rates.

The earlier starting-food comparison is superseded by this shared 1,500-food
cohort. It does not justify claiming starvation is impossible or multiplying
consumption without testing the actual economy and human experience.

## Historical full-charter matrix

[balance_matrix.csv](balance_matrix.csv) contains 990 aggregate cells from the
September 2 analysis: 22 charters × 3 legacies × 5 loadouts × 3 policies, each with
50 seeds (49,500 voyages). It omits the tutorial charter and predates current
ship-work/survival and timing behavior. Keep it for comparison; its old success,
extinction, economic and timing conclusions do not certify today's build.

The ignored `generate_release_balance_report` test in
[balance/tests.rs](src/simulation/balance/tests.rs) still asserts 22 charters even
though the current registry has 23, and its report renderer includes historical
interpretations as fixed prose. Regeneration would overwrite this document and
could fail after expensive simulation. Fix the generator and separate calculated
results from interpretation before rerunning it; this is explicitly tracked in
[TODO.md](TODO.md). No full-matrix rerun is implied by a normal test pass.

## Human acceptance

Measure meaningful decisions, comprehension, scarcity and recovery across real
voyages rather than treating simulation clock time as human play time. The current
one-second month gives 12/6/4 seconds per year at 1x/2x/3x before pauses and
reading; it does not establish a 30–60 minute voyage. Compare conservative,
objective-focused and imperfect novice play over successive charters. Record
repeat content, dominant purchases, long quiet periods, resource bottlenecks and
irreversible losses. The exact-candidate acceptance routes live in
[release QA](docs/release/QA_AND_OPERATIONS.md).
