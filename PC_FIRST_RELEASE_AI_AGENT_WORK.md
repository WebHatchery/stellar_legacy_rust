# Stellar Legacy — AI-Agent Work for the First PC Release

> **Execution reconciliation (2026-08-26):** all work that can be completed from the
> repository without owner decisions, credentials, storefront state, new bug reports,
> or human/clean-machine judgement is implemented or prepared. See
> `docs/release/AI_AGENT_EXECUTION_STATUS.md` for evidence and the precise external gates.
> The original boxes below are retained as the audit decomposition; they are not a claim
> that an agent may complete the human-dependent portions of a mixed item.

**Audit date:** 2026-08-26
**Launch scope used for this plan:** a paid, full (not Early Access), English-language
Windows x86-64 release on Steam and itch.io. Browser, Linux, macOS, console, mobile,
localisation, multiplayer, online accounts, and new gameplay systems are out of scope.

This file lists work an AI coding agent can perform or substantially complete. The
separate `PC_FIRST_RELEASE_HUMAN_REQUIRED.md` lists the decisions, attestations, account
actions, and play judgements that must remain with a person.

## Verdict on the 4–7 month estimate

**Four to seven months is not supported as the remaining production estimate for the
defined Windows release.** It is defensible only as a deliberately slow, part-time
commercial launch runway or if the scope expands to major art replacement, localisation,
controller/Steam Deck certification, a demo/festival campaign, or substantial redesign.

A more evidence-based range from the repository's current state is:

| Release target | Elapsed time | Assumptions |
| --- | ---: | --- |
| itch.io-only Windows soft launch | **2–4 weeks** | One focused owner, one stabilisation pass, no long marketing campaign |
| Steam + itch.io Windows, fastest credible path | **5–8 weeks** | Steam onboarding starts immediately; technical work, store work, and testing overlap |
| Steam + itch.io Windows, prudent first commercial release | **6–10 weeks** | Includes external playtest feedback, a second RC, and store-review contingency |
| Marketing-led launch with a polished trailer and wishlist runway | **8–12 weeks** | Still no major new features or localisation |

The Steam lower bound is administrative, not code-driven. A new product has a 30-day
wait after paying the Steam Direct fee, its Coming Soon page must be public for at least
two weeks, and Valve recommends allowing at least seven business days for each review
submission. If the app fee was paid more than 30 days ago and a compliant Coming Soon
page has already been live for two weeks, the fastest Steam range can shrink by roughly
2–4 weeks.

Estimated hands-on work, assuming the current data defect is the only cascading test
root cause:

- **AI-agent engineering/content-production work:** about 7–15 focused working days.
- **Human owner and tester time:** about 30–70 hours, much of it parallel with agent work.
- **Contingency:** add 1–3 weeks if fresh-machine testing exposes graphics, audio,
  persistence, antivirus, DPI, or long-voyage defects.

Lines of code are a poor driver here. This is a large codebase, but most of the costly
game-design work already exists and is unusually well instrumented. Remaining release
risk is dominated by integration truth, hands-on play, platform setup, rights review,
store presentation, and launch operations.

## Evidence behind the estimate

### Strong existing evidence

- About **44,422 physical Rust lines in 190 `.rs` files**, with no Rust file above the
  workspace's 800-line ceiling; **285 tracked files** in total.
- The game has a complete documented loop from New Game through Homecoming and a second
  charter, local saves and migration plumbing, persistent Chronicle/Heritage data,
  settings, procedural audio, and touch/mouse/keyboard UI paths.
- Content is already deep: **327 events, 22 charters, three legacies, six founding
  factions, and six maintainable subsystems**.
- There are **44 checked-in verification captures** and a deterministic balance report
  generated from **49,500 simulated voyages**.
- The repository has Windows and WebGL CI jobs, a shared release builder, a Windows zip
  artifact shape, and a project-level itch publisher wrapper.
- `TODO.md` says no discrete gameplay tasks remain, and the 2026-08-05 release-readiness
  plan records its major gameplay/UI/audio phases as complete.

### Release blockers and unproven claims found by this audit

- A fresh `cargo test --all-features` run is **red**. The binary suite started 461 tests
  and ended with 38 passed, 422 failed, and one ignored; the separate source-size test
  did not get a chance to run. The cascade begins with one confirmed duplicate embedded
  event ID, `the_seized_works`, at two entries in
  `assets/events/engineering.json`. The current toolkit correctly rejects the duplicate,
  so the game cannot load its content in the test suite. This is likely a small root fix,
  but all 460 active binary tests must be rerun before assuming it is the only failure.
- The required no-argument `publish.ps1` run nevertheless exited successfully on
  2026-08-26 and deployed Preview. A capture-mode launch of the newly built retail
  executable then panicked during startup with exit code 101 on that same duplicate ID.
  The publisher currently proves compilation/packaging, not that the packaged game can
  start, and therefore needs an explicit retail-binary smoke gate.
- The repository has an old `dist/stellar_legacy_windows.zip`, but it is not evidence of
  a release candidate built against the current toolkit. The archive contains only
  `stellar_legacy.exe` and `assets.zip`.
- There is no SteamPipe app/depot configuration, launch-option record, Steam test branch
  workflow, or Steam store asset set in the repository.
- `publish-itch.ps1` cannot target a page because the required `itch.json` is absent.
- No project licence, third-party notices, credits file, changelog/release notes,
  support instructions, privacy statement, EULA decision record, or dependency licence
  report is checked in.
- The freshly built Windows executable is unsigned and its Product, FileVersion, and
  Description fields are empty. No dedicated Windows app icon/resource metadata or
  Authenticode signing workflow was found. Signing is optional for distribution but
  materially affects Windows trust presentation; whether to buy and use a certificate
  is a human business decision.
- CI builds artifacts but does not preserve a versioned release artifact or publish to a
  PC storefront.
- Automated captures and a balance matrix do not prove fresh-machine launch, save
  survival across upgrades, readable long-session pacing, antivirus reputation, or a
  complete human campaign.
- The product remains versioned `0.1.0` in both Cargo and game data. A public versioning,
  save-compatibility, rollback, and patch policy has not been recorded.

## AI-agent execution checklist

The order is deliberate. Do not start storefront upload work until the release candidate
is reproducible, and do not call an upload a release until the human gates in the other
file are signed off.

### 1. Freeze the release definition

- [ ] Add a short release manifest recording: Windows x86-64 only, English only, full
  release, premium/free price decision supplied by the owner, no online services, and
  exactly which store features are promised.
- [ ] Reconcile `Cargo.toml`, `assets/data/game_config.json`, player-visible version text,
  save-format version, package filenames, and store build labels.
- [ ] Create release notes and a changelog entry for the first public build.
- [ ] Record minimum and recommended Windows requirements only after measurements on the
  final build; do not invent hardware claims.
- [ ] Remove Browser from release-facing metadata for this PC-only launch while keeping
  WebGL as an internal validation target if desired.

### 2. Restore a green, reproducible release candidate

- [ ] Rename or intentionally merge the two `the_seized_works` events, update every
  reference, and add a regression test proving IDs are globally unique.
- [ ] Run all 460 active binary tests plus the source-size integration gate; separately
  run the ignored 49,500-voyage balance job and compare its generated report with the
  checked-in baseline.
- [ ] Run formatting and clippy with warnings denied.
- [ ] Run every headless capture scene and review machine-detectable clipping, overlap,
  missing-glyph, missing-asset, and minimum-target reports.
- [ ] Run `publish.ps1` with no parameters, as required by the workspace, then run a
  Windows-only release build for the actual PC candidate.
- [ ] Add a post-package smoke gate that launches the exact packaged executable in
  capture mode, requires exit code zero, requires an output frame, and fails publishing
  on startup panic. Do the equivalent against the packaged WebGL build while it remains
  part of the workspace validation path.
- [ ] Produce a release manifest containing commit hash, Rust version, toolkit commit,
  build time, artifact names, sizes, and SHA-256 hashes.
- [ ] Build twice from the same clean commit and investigate unexplained artifact
  differences. Bit-for-bit reproducibility is desirable, but a recorded explanation is
  acceptable where timestamps prevent it.
- [ ] Verify that every generated Rust file remains below 800 physical lines and that no
  debug-only capture path or environment override changes ordinary startup.

### 3. Harden native Windows behaviour

- [ ] Install the toolkit crash-log hook before game initialisation and show the support
  path in a player-facing Help/About entry without collecting telemetry.
- [ ] Add a safe way to reveal the save folder and document its Windows location.
- [ ] Test and, where feasible, automate save creation, atomic replacement, corrupt-save
  quarantine, migration, missing permissions, non-ASCII Windows usernames, and read-only
  install directories.
- [ ] Verify window resizing, minimising/restoring, focus loss, Alt+F4, DPI scaling,
  multi-monitor movement, audio-device loss, and muted operation.
- [ ] Add Windows version metadata, product name, file description, semantic version,
  copyright holder supplied by the owner, and a proper icon to the executable.
- [ ] Decide whether the build is portable zip only or also has an installer. If an
  installer is approved, create and test uninstall behaviour that never removes saves.
- [ ] Add automated checks ensuring retail archives contain only runtime files—no PDBs,
  secrets, Steam credentials, source-only captures, or stale assets.
- [ ] Scan the final archive with locally available antivirus tools and record hashes and
  results. Submit false positives for human follow-up if any appear.
- [ ] Prepare (but do not personally purchase or control) an Authenticode signing flow
  that reads credentials from secure storage and verifies the resulting signature.

### 4. Complete release documentation and rights inventory

- [ ] Generate a direct and transitive dependency licence inventory from the exact lock
  file used for release and assemble required notices.
- [ ] Inventory every shipped image, font, sound, text corpus, name list, code component,
  and store asset with its source, creator, licence/permission, and whether generative AI
  assisted it. Flag unknowns for the human owner; an agent cannot establish ownership.
- [ ] Draft project copyright/licence text, third-party notices, credits, support page,
  troubleshooting guide, refund-neutral system requirements, save location, and known
  issues for human/legal approval.
- [ ] Draft a short privacy statement explaining that the scoped build uses local saves
  and no accounts/analytics/network services, after verifying the final binary. The
  owner must approve the truth of that statement.
- [ ] Audit the build and store copy for all claimed features, languages, input modes,
  operating systems, achievements, cloud saves, accessibility options, and AI use.
  Remove claims that are not present in the submitted build.

### 5. Make store-quality marketing material

- [ ] Write the short description, long description, feature bullets, developer/publisher
  description, content warning copy, support copy, release notes, and launch announcement.
- [ ] Select at least five genuine 1920×1080-or-larger 16:9 gameplay screenshots for
  Steam, without marketing text or concept art. The existing capture harness can create
  candidates, but a human must approve truthful representation.
- [ ] Produce the current Steam store capsules and library assets from approved key art:
  header 920×430, small 462×174, main 1232×706, vertical 748×896, shortcut icon
  256×256, app icon 184×184, library capsule 600×900, hero 3840×1240, transparent
  library logo up to 1280 wide/720 tall, and library header 920×430.
- [ ] Enforce Steam's capsule rule: base assets contain only game art, name, and official
  subtitle; the library hero contains no text; the logo stays legible at small size.
- [ ] Produce an itch.io cover at 630×500 and 3–5 representative screenshots, plus page
  colours and layout matching the game.
- [ ] Script, capture, edit, caption, and export a truthful gameplay trailer if the human
  owner chooses to make one. Use only cleared music, fonts, footage, and art.
- [ ] Export editable masters and a provenance manifest for all store media, not just
  flattened delivery files.

### 6. Prepare Steam delivery

- [ ] After the human supplies App ID and Depot ID, add parameterised SteamPipe app and
  depot VDF templates that contain no credentials.
- [ ] Stage only the final Windows runtime files and configure the launch option for the
  actual executable path.
- [ ] Add scripts for previewing a SteamPipe build, uploading to a password-protected
  test branch, recording Build ID, promoting with an explicit human-controlled step,
  and rolling back to a known good manifest.
- [ ] Verify installation and launch through the Steam client, offline launch, update
  from the previous RC, uninstall/reinstall, and save preservation on human-provided
  test machines/accounts.
- [ ] Fill draft Steamworks metadata from approved copy: Windows-only OS support,
  categories/features, system requirements, pricing proposal, support contacts,
  release date, content survey draft, and reviewer instructions.
- [ ] Prepare exact reviewer steps that reach New Game, launch a charter, trigger a
  decision, save/load, and return home without debug commands.
- [ ] Do not claim Steam Achievements, Steam Cloud, controller support, Trading Cards,
  workshop, multiplayer, or Deck compatibility unless each is deliberately implemented
  and tested.

### 7. Prepare itch.io delivery

- [ ] Create `itch.json` after the human supplies the real `owner/game` target; configure
  a Windows channel and public version without storing authentication secrets.
- [ ] Validate `publish-itch.ps1 -Channel windows -Preview` and then a butler dry run.
- [ ] Draft accurate Windows-only metadata, tags, English-language declaration, AI
  disclosure, content classification, price, install instructions, and support link.
- [ ] Upload first to a restricted/draft page or test channel, download through both the
  browser and itch app, and verify install, launch, update, uninstall, and save survival.
- [ ] Record the channel build ID/version and SHA-256 hash for rollback and support.

### 8. Support human QA and release decisions

- [ ] Provide clean profiles and deterministic seeds for smoke, ordinary, harsh,
  succession, forced-return, extinction, and second-charter scenarios.
- [ ] Turn human bug reports into reproducible cases, regression tests, focused fixes,
  updated captures, and release-note entries.
- [ ] Maintain a severity-ranked defect ledger with P0/P1 release blockers separated
  from post-launch improvements.
- [ ] Generate an RC checklist that records tester, machine, Windows version, GPU,
  resolution/DPI, input method, save scenario, result, and artifact hash.
- [ ] Rebuild and rerun proportionate validation after every accepted fix; never patch a
  store artifact by hand.

### 9. Release and post-release operations an agent can assist with

- [ ] Prepare the final Steam and itch uploads, checksum them, compare them with the
  approved RC, and stop before irreversible public-release controls unless explicitly
  authorised at that moment.
- [ ] Prepare launch announcements, support macros, known-issue posts, rollback commands,
  and hotfix branches.
- [ ] Monitor permitted store/build dashboards and public crash/bug reports, summarise
  them without exposing player data, and propose severity-ranked responses.
- [ ] Build and validate hotfixes, preserve save compatibility, update notices, and
  prepare rollback. A human retains authority over public deployment and communications.

## AI-agent completion gate

The agent-owned portion is complete only when all of the following are true:

- Tests, formatting, clippy, source-size gate, capture audit, release balance run, and
  the required no-argument `publish.ps1` validation are green from one clean commit.
- A freshly built Windows artifact has recorded provenance and checksums and passes
  automated retail-content checks.
- Steam test-branch and itch restricted-page downloads match that artifact and launch on
  human-provided clean Windows machines.
- Store copy/assets, legal/rights inventories, reviewer instructions, support material,
  and rollback procedures are ready for explicit human approval.
- Every item in `PC_FIRST_RELEASE_HUMAN_REQUIRED.md` marked as a launch gate has an owner
  and status; no agent has silently answered a legal, financial, rights, rating, or
  subjective quality question on the owner's behalf.

## Current official platform references

Checked 2026-08-26; re-check immediately before submission because store rules change.

- Steamworks onboarding, fee, identity/bank/tax information, 30-day wait, and Coming Soon
  rule: <https://partner.steamgames.com/doc/gettingstarted/onboarding>
- Steam release and review timing: <https://partner.steamgames.com/doc/store/releasing>
  and <https://partner.steamgames.com/doc/store/review_process>
- Steam content and generative-AI survey:
  <https://partner.steamgames.com/doc/gettingstarted/contentsurvey>
- Steam graphical asset specifications:
  <https://partner.steamgames.com/doc/store/assets>
- SteamPipe setup and uploads: <https://partner.steamgames.com/doc/sdk/uploading>
- itch.io first-page and image guidance:
  <https://itch.io/docs/creators/getting-started>
- itch.io quality and AI-disclosure guidance:
  <https://itch.io/docs/creators/quality-guidelines>
- itch.io payments and tax interview: <https://itch.io/docs/creators/payments>
- itch.io butler upload channels: <https://itch.io/docs/butler/pushing.html>
