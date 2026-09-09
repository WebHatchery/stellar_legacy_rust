# Open work

Current behavior is documented in [gdd.md](gdd.md). The Custodian identity pass,
obligations, officers/institutions, Agenda and responsive five-destination UI are
implemented. Completed milestone diaries are not open tasks.

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

## Known static-check failures

The September 9 documentation-cleanup check with Clippy and warnings denied found
three existing UI diagnostics: nested formatting in
[debrief/report.rs](src/ui/debrief/report.rs) and
[mobile/history.rs](src/ui/mobile/history.rs), plus an unnecessary reference in
[responsive.rs](src/ui/responsive.rs). Resolve these before claiming a clean
Clippy gate. Default/demo tests, formatting and the normal publisher passed;
those results do not waive these diagnostics.

## Balance analysis maintenance

The saved 22-charter matrix predates the 23-charter registry, 1-second monthly
clock and ship-work survival rules. The ignored matrix generator still contains
a 22-charter assertion and hard-coded historical interpretation prose. Update
those before regenerating or treating the full matrix as release evidence.
Keep the ordinary test suite and comparable ship-work policies as separate
checks; neither demonstrates final human pacing or balance. See
[balance evidence](balance_report.md).

## Deferred design intent

These are retained design directions, not scheduled releases or implemented
features. Reconsider them after the current core passes human acceptance.

| Direction | What remains beyond current behavior | Required boundary |
| --- | --- | --- |
| Charter-specific approaches | Authored doctrine selection with requirements, tradeoffs and effects on at least two voyage systems across the six objective families; selected doctrine retained through debrief/history | Reuse global posture, gates, officers and obligations; compare paired policies and avoid a universally best approach |
| Compartment culture | A compact local custodian, descriptor, remembered event and grievance, with at least two tradeoff-bearing descriptors per subsystem | Derive from existing people/institutions; no second resident simulation, room placement or unlimited trait stacking |
| Homecoming recovery | A consequential context-sensitive social/institutional recovery choice between voyages, preserving unresolved wounds and historical facts | Extend the sealed debrief and existing port rules; current underway projects do not implement this planned choice |
| Competing historical accounts | Official, dynasty and affected-people interpretations of a single authoritative deed, distinguishable in the Chronicle | Keep mechanics factual; do not duplicate mechanical history or hide consequences behind unreliable prose |
| Further work-management options | Protected reserves, named development programmes/trainees, standing orders and additional project slots | Add only after measured scarcity and choice quality justify more complexity |

Every accepted extension needs typed data, save compatibility, deterministic
behavior tests, touch-accessible controls and its own publisher validation.
The completed visual redesign already supplies ship/person/faction identity;
do not restart it from the retired plans.
