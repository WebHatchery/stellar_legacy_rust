# Release operations and acceptance

Current source version: **0.2.1** in Cargo and game configuration. This document
records available tooling and open gates; a successful build is not public-release
approval. There is no completed final store/clean-machine sign-off in the retained
repository records.

## Build scope

The game builds native Windows and full WebGL packages. The itch HTML5 channel is
a single-charter tutorial demo; the normal Windows build is the full game.
Language is English. Runtime progress is local, with no accounts, telemetry,
multiplayer, cloud saves or live generative-AI service. In-game achievements exist;
Steam achievements and other Steamworks features are not implemented promises.

The recorded PC distribution intent is a portable Windows x86-64 ZIP. Price,
legal publisher/copyright, support commitment, final hardware/Windows requirements,
signing and public release date still need owner decisions. The repository has
Steam app/depot templates and an itch target, but their existence does not prove
account approval, an upload or a public release. `itch.json` currently targets
`webhatchery/stellar-legacy` with `html5-demo` and `windows` channels; its default
`user_version` remains 0.1.0, so pass the candidate version explicitly when packaging.

## Required validation

Run from the Stellar Legacy project directory:

```powershell
.\publish.ps1
```

The wrapper delegates to the RustGames parent publisher. With no parameters it
builds/packages Windows and WebGL and deploys Preview, then runs
[test_release_package.ps1](../../scripts/test_release_package.ps1) against
`dist/stellar_legacy_windows.zip` and `dist/webgl`. Package checks include the exact
Windows executable's rendered capture and real-browser initialization, not just
compilation. `-Production` is a separate deployment choice; `-DryRun` avoids the
normal deployment path. Do not substitute a local development server for this gate.

Use the package-scoped tests/Clippy/formatting commands in [README](../../README.md).
The normal suite includes the comparable ship-work cohort and the 800-line source
gate. The separate ignored full balance matrix needs maintenance before use;
see [balance analysis maintenance](../../TODO.md#balance-analysis-maintenance). Test logs/captures from an older
commit do not certify the candidate being submitted.

## Captures and media

[scripts/capture_ui.ps1](../../scripts/capture_ui.ps1) wraps the toolkit batch
capture API with prefix `STELLAR_LEGACY`. It accepts actual game scene names,
frame count and window dimensions. Example supplemental capture command:

```powershell
.\scripts\capture_ui.ps1 -Scenes gameplay,event -WindowWidth 1280 -WindowHeight 720
```

The scene registry is [capture_scenes.rs](../../src/game/capture_scenes.rs) and its
children. Store verification images directly in `docs/verification/`, replacing
the same state; inspect actual PNG dimensions rather than trusting names such as
`narrow`. Review 1280x720, 1920x1080, 1024x768 and 390x844, long names, full/empty
queues, all schemes, UI scale 75–200%, text size 75–150% and reachable modal controls.

The repository retains screenshots and measurement artifacts from earlier UI
passes. These show sampled states; they do not establish every current interaction,
scale combination, human comprehension or full campaign route. Regenerate affected
store screenshots only after final presentation is stable. See
[media provenance](STORE_MEDIA_PROVENANCE.md) for source masters and generation.

## Candidate and distribution tooling

After ordinary validation, from a clean committed tree:

```powershell
.\scripts\create_release_candidate.ps1
```

This builds/checks Windows using the publisher's Windows-only dry-run mode, then
writes a versioned archive and manifest under `dist/releases/`. The manifest
records game/toolkit commits, dirty state, Rust version, lockfile hash, artifact
size/hash and build time. `-AllowDirty` is available for investigation, not a
clean release claim; `-ScanWithDefender` requests a local scan and records its result.
[compare_release_builds.ps1](../../scripts/compare_release_builds.ps1) builds Windows twice and compares the packaged payloads. It requires a clean
commit unless passed -AllowDirty; ZIP timestamps may differ without payload changes.

[generate_release_inventories.ps1](../../scripts/generate_release_inventories.ps1)
regenerates Windows dependency licenses, upstream notices and asset inventory from
Cargo metadata and repository files. It replaces inventory files: preserve any
owner-reviewed provenance before regeneration. Unknown permissions remain open.

[prepare_steam_build.ps1](../../scripts/prepare_steam_build.ps1) requires actual
`-AppId` and `-DepotId`, validates the archive and stages credential-free content
and templates. It does not infer IDs or approve a branch. Upload requires explicit
`-Upload` and `-TestBranch`; its output instructs the operator to record the Build
ID and assign it in Steamworks. Signing is optional through
[sign_windows_artifact.ps1](../../scripts/sign_windows_artifact.ps1), with the
owner's certificate authorization and subsequent signature verification.

To package the itch tutorial without uploading after the full build:

```powershell
.\publish-itch.ps1 -Channel html5 -UserVersion 0.2.1 -DryRun
```

Use `-Channel windows` for the normal Windows package. The HTML5 wrapper builds
with `demo`, uses `itch-index.html`, temporarily swaps package output and restores
the full WebGL package. Packaging does not change page visibility or approve
public distribution. Check current store requirements in the authenticated store
workflow before submission; old fee/timing/specification estimates are not kept
as project rules.

## Candidate verification — 14 September 2026

The release candidate was built from game commit `4929a2bd5c19494cadbe80618c62b78b8c60f5c2` and toolkit commit
`d97b7237f5eeeb775237a7e3795de4f446d795e7`. The published package hashes are:

- Windows ZIP: `97D4721FA7D76A0105C1BDDF6C55D5E543AD58EE69AE59EE8CCA6083D612461A`
- WebGL ZIP: `6035FE20D4DA8C7C8A07E9C2C18ECC734337ED920F3C41B558B0A555316B1DDC`

The required publisher completed Windows and WebGL builds, packaged the release,
deployed the WebGL Preview, and passed the packaged Windows smoke run. The
redesign matrix verified 104 captures: 61 baseline, 27 portrait, 8 compact and
8 desktop. The touch-audit entries passed, corrected modal/layout scenes were
spot-checked, and the standalone browser-process regression passed.

This records engineering and visual evidence for the candidate only. It is not
manual clean-machine, store-client, or owner approval; the outstanding acceptance
and owner-decision gates below remain open until an authorized tester records them.

## Outstanding acceptance

These items have no final sign-off recorded here. Prior agent walkthroughs and
screenshots are not substitutes for these observations on the candidate hash.

1. New profile: welcome → founding → Voyage/Drydock → briefing/provisions → People
   and Ship reviews → Launch → first council choice and explicit Commit.
2. Ship work: Bridge risk → Systems/Agenda → queue multiple jobs → review → pause,
   resume, reorder and cancel; compare actual refunds, retained stages and debt.
3. People/history: fill a vacancy, train, appoint an apprentice, review an heir,
   experience a full succession, resolve a due obligation and inspect its history.
4. Campaign: Homecoming report → next charter; save/reload in port, underway,
   during a decision and during Homecoming. Preserve resources, authority and jobs.
5. Survival: all critical-air actions, stabilisation and its reset after air recovery, each terminal route,
   and return to title/History without reviving the ended campaign.
6. Fresh-player judgment: identify Bridge objective/risk/next action within five
   seconds; explain council costs without optional advice. Measure choice cadence,
   full voyage duration, repetition, recovery clarity and audio fatigue.
7. Clean machines: supported Windows/GPU/DPI combinations, mouse/trackpad/touch,
   window/fullscreen, focus/minimize/sleep, monitor/audio changes, muted play,
   non-admin/non-ASCII paths, antivirus and abrupt-exit recovery.
8. Store-delivered builds: install, launch offline, update, uninstall/reinstall and
   confirm save survival. Include uninvolved strategy and novice testers.

Record tester/date, exact artifact SHA-256, game/toolkit commits, machine/OS/GPU,
resolution/DPI, input, scenario, result and defect for each run. P0 means crash,
data loss or inability to progress; P1 covers release-blocking launch, save or
severe usability defects. Resolve known P0/P1 defects before approval.

## Owner decisions and release record

Keep price/business model, legal identity, rights, AI/content disclosures, privacy,
support contact, hardware promises, store copy/media, signing and release timing
under explicit owner review. [Rights/notices](RIGHTS_AND_NOTICES.md),
[store copy](STORE_COPY.md) and [support/privacy](SUPPORT_AND_PRIVACY.md) contain
useful drafts; unknowns are unresolved decisions, not permission to invent values.

For the approved candidate record legal publisher, public version, game/toolkit
commits, archive hash, store App/Depot/Build/channel IDs, test results, approved
notices/media, price, support and rollback owner, known issues and final go/no-go
name/date. No empty sign-off table is treated as completed evidence.

Public uploads, promotion, visibility, pricing and announcements are separate
authorized actions. Do not claim storefront completion from Preview deployment.
For rollback/hotfix, rebuild the known-good source, preserve or explicitly migrate
saves, rerun validation and hash the artifact. Never edit a store archive by hand;
record the replacement store build and obtain approval for public promotion.
