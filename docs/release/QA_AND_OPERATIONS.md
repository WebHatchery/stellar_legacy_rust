# Release-candidate QA and operations

## Severity ledger

| ID | Severity | Status | Summary |
| --- | --- | --- | --- |
| RC-001 | P0 | Fixed + regression tested | Duplicate `the_seized_works` ID prevented startup |
| RC-002 | P1 | Fixed | Publisher did not launch the exact packaged executable |
| RC-003 | P1 | Human gate | Clean-machine/store-client/save-survival matrix unperformed |
| RC-004 | P2 | Human gate | Hardware requirements unmeasured |
| RC-005 | P0 | Fixed + regression tested | Balance policy could choose an unaffordable outcome and stall a voyage |

## Agent validation record

- 465 active unit tests and the source-size integration gate pass; the one ignored
  49,500-voyage job passes separately and regenerates byte-identical report/matrix files.
- Formatting and clippy with warnings denied pass; no Rust file exceeds 800 lines.
- All 44 capture scenes render. The touch audit reports no ambiguity or undersized
  visible controls across the 43 interactive scenes (the boot frame has no controls).
- The exact Windows archive passes a two-file allow-list and launches its extracted
  executable to a rendered frame from a path containing spaces with read-only runtime
  files. The packaged WebGL runtime initializes in Chrome.
- `scripts/compare_release_builds.ps1` builds the Windows archive twice from one clean
  commit and rejects any difference in the extracted runtime payload. It records whether
  the ZIP containers themselves are byte-identical and explains timestamp-only variance.
- Windows Defender scanned the candidate ZIP on 2026-08-26; the release manifest records
  the scan start and the count of detections associated with the exact artifact path.
- Windows executable metadata reports Stellar Legacy and version 0.1.0.

P0: crash/data loss/cannot progress. P1: release-blocking install, launch, save, or severe
usability problem. P2/P3 may be scheduled only after explicit owner review.

## RC test record

| Tester/date | Artifact SHA-256 | Machine/Windows | GPU | Resolution/DPI | Input | Save scenario | Result/defects |
| --- | --- | --- | --- | --- | --- | --- | --- |
| UNASSIGNED | FROM MANIFEST | UNTESTED | UNTESTED | UNTESTED | UNTESTED | create/update/corrupt/reinstall | PENDING |

Required profiles/seeds: smoke `1001`; ordinary `2027`; harsh `99001`; succession
`314159`; forced-return `271828`; extinction stress `8675309`; second-charter `424242`.
The checked-in capture scenes provide deterministic UI states; the release balance test
provides deterministic voyage cohorts. Human play must use clean profiles without debug
commands.

## Reviewer route

1. Launch `stellar_legacy.exe`; tap/click through the welcome and choose NEW GAME.
2. Select a legacy and founding choices using visible controls.
3. Open CHARTERS, choose a starter charter, provision in PREP, and tap LAUNCH.
4. Use the visible pace controls until a council decision appears; choose an available
   option by clicking it.
5. Return to port, verify Homecoming and Chronicle, quit, relaunch, and choose CONTINUE.

## Rollback/hotfix procedure

Never edit a store artifact. Rebuild from the known-good commit, run the full validation
and package smoke, compare SHA-256, then assign the recorded Steam manifest or itch build
to the restricted branch/channel. Public promotion, rollback, messaging, and visibility
remain explicit human actions. Hotfix branches use `codex/hotfix-<issue>` and must preserve
the 0.1.0 save shape or add a tested migration.

## Release stop line

The agent may stage, checksum, preview, and upload only when specifically authorised.
It stops before public Steam release, itch visibility changes, pricing, legal surveys,
content-rating attestations, announcements, or irreversible promotion controls.
