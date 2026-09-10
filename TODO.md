# Open work

Current behavior is documented in [gdd.md](gdd.md). The Custodian identity pass,
obligations, officers/institutions, Agenda and responsive five-destination UI are
implemented. Completed milestone diaries are not open tasks.

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
