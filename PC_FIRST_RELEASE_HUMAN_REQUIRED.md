# Stellar Legacy — Human-Required Work for the First PC Release

**Audit date:** 2026-08-26

**Scope:** paid full release, English, Windows x86-64, Steam + itch.io.
**Companion file:** `PC_FIRST_RELEASE_AI_AGENT_WORK.md` contains the technical audit,
revised estimate, and work an AI agent can execute.

An AI agent can draft, automate, upload with authorised access, and present evidence.
It cannot truthfully replace a legal rights-holder, taxpayer, bank-account owner,
credential holder, product owner, or human player. The items below therefore require a
person even when an agent prepares most of the surrounding work.

## Human launch gates at a glance

| Gate | Human must do | Why it cannot be delegated away |
| --- | --- | --- |
| Product authority | Name the legal publisher and confirm it owns or has permission for every shipped element | This is a legal representation, not a repository inference |
| Money and tax | Choose price/business model; supply bank/tax identity; pay fees; accept payout terms | Requires identity, financial authority, and contracts |
| Content truth | Approve mature-content, generative-AI, language, feature, and accessibility disclosures | Only the owner can attest to provenance and intended claims |
| Quality | Personally play the RC and approve readability, pacing, feel, and value | Automated tests cannot judge the lived experience |
| Store control | Control credentials/2FA, submit for review, answer reviewers, and press the final release control | These are authenticated and externally consequential actions |
| Launch risk | Choose date, scope, signing, support policy, go/no-go, rollback, and emergency response | These decisions commit money, reputation, and customer obligations |

## 1. Define ownership and commercial intent

- [ ] **Name the publishing party.** Decide whether the legal publisher is an individual
  or an entity and ensure the name matches bank and tax records.
- [ ] **Confirm the right to publish the game.** Resolve ownership of the port, the
  original web version, the `Stellar Legacy` name, all narrative text, all code, and all
  artwork/audio/fonts/name data.
- [ ] **Decide the stores.** Confirm Steam + itch.io, or explicitly reduce scope to one.
  The 5–8 week fastest credible estimate assumes both and a fresh Steam onboarding clock;
  itch-only can plausibly be 2–4 weeks.
- [ ] **Choose full release versus Early Access.** This plan assumes full release because
  the repository describes the game as mechanically complete. Do not choose Early Access
  merely to avoid a final quality decision.
- [ ] **Choose the business model and price.** Decide paid/free/pay-what-you-want, launch
  discount, regional pricing, refund posture, key policy, and whether itch purchases
  include Steam keys.
- [ ] **Choose the supported promise.** Approve Windows editions, minimum hardware,
  English-only status, input methods, offline behaviour, accessibility claims, and the
  absence of Steam-specific features not implemented in the build.
- [ ] **Choose a support commitment.** Name the contact/channel, response expectations,
  supported save migrations, hotfix policy, and minimum maintenance period.

## 2. Complete legal, financial, and account actions

These are store-mandated or legally sensitive launch gates.

### Steam

- [ ] Create or use the correct Steamworks partner account under the legal publishing
  party; keep credentials and Steam Guard/2FA under human control.
- [ ] Personally read and electronically sign the NDA and Steam Distribution Agreement.
- [ ] Supply accurate identity, address, company form, bank details, and tax information;
  respond to any verification request. Valve says tax verification may take 2–7 business
  days and the game cannot release without valid bank and tax information.
- [ ] Pay/allocate the **US$100 Steam Direct fee** for this product. If newly paid, record
  the date because Steam requires a **30-day wait** before release.
- [ ] Create/claim the app, record its App ID and Depot ID, and grant the minimum necessary
  permissions to any build account. Do not give an agent personal banking or tax access.
- [ ] Publish the approved Coming Soon page and record its public date; it must be live for
  **at least two weeks** before release.

### itch.io

- [ ] Create the real itch.io project page and choose its permanent owner/slug; butler
  cannot create the page for the agent.
- [ ] Keep account credentials and 2FA under human control and authorise any upload token
  with the least privileges available.
- [ ] Choose direct payments or itch.io-collected payouts. Complete the seller terms,
  payment connection, and tax interview if accepting money.
- [ ] Approve itch's revenue-share setting, minimum price, currency presentation, sale
  terms, and any Steam-key policy.

### Rights and policy

- [ ] Review the agent-produced asset/provenance ledger. Find evidence for every unknown
  source; replace or remove anything whose commercial rights cannot be demonstrated.
- [ ] Decide the project's copyright/licence and approve third-party notices. Obtain legal
  advice where needed; an AI-generated inventory is not legal clearance.
- [ ] Confirm whether any shipped narrative, code-visible text, art, audio, localisation,
  or marketing material was generated or materially assisted by generative AI.
- [ ] Personally approve the Steam generative-AI disclosure and itch.io AI Disclosure.
  Steam specifically asks about pre-generated content consumed by players, including
  artwork, sound, narrative, and localisation. Do not infer “none” merely because the
  running game has no live AI service.
- [ ] Personally approve mature-content/content-rating answers and any violence, death,
  substance, discrimination, fear, or other descriptors reflected in the 327 events.
- [ ] Approve the privacy statement only after confirming the retail build has no
  analytics, account system, network service, or unexpected data collection.
- [ ] Decide whether to purchase a Windows code-signing certificate. If signing is used,
  a human or controlled signing service must retain the private key and authorise signing.

## 3. Make the product decisions an agent cannot make

- [ ] **Approve the duplicate-event resolution.** The two events called
  `the_seized_works` have different content and categories. A person should decide which
  canonical identity/name belongs to each before an agent updates references.
- [ ] **Approve first-release scope discipline.** Reject or accept proposals for new
  content, controller support, Steam achievements/cloud, localisation, visual overhaul,
  demo, or multiplayer. Each accepted addition invalidates the short estimate.
- [ ] **Approve the final version and save policy.** Decide whether the public release is
  `1.0.0` or another version, whether pre-release saves are supported, and what future
  compatibility is promised.
- [ ] **Approve price-to-value fit.** Judge whether the present terminal/text art profile,
  voyage length, content repetition, and replay value justify the chosen price.
- [ ] **Approve store positioning.** Select the final genre/tags, comparable games,
  audience promise, title treatment, capsule direction, screenshots, trailer, and short
  description. An agent can generate options; only the owner should decide what represents
  the product and business.
- [ ] **Approve Windows support boundaries.** Decide whether to promise Windows 10 only,
  Windows 10/11, portable zip, installer, unsigned binary, and any minimum GPU/resolution.
  Base this on actual test evidence.

## 4. Perform human play and usability validation

This is release-quality required even where a storefront does not mandate each test.
Use the exact RC hash downloaded from each store test channel, not a developer build.

### Owner acceptance playthrough

- [ ] On a clean Windows user profile, install/download, launch without development tools,
  start a new dynasty, choose peoples/legacy, select and provision a charter, launch,
  resolve council decisions, experience succession, reach Homecoming, return to drydock,
  and begin a second charter.
- [ ] Save and quit at drydock, underway, during a pending decision, and at Homecoming;
  relaunch after each and confirm exact recovery.
- [ ] Exercise pause and every speed, every main screen, touch/click-only onboarding,
  optional keyboard paths, scroll/drag, delegation, abort/force-return, repair, refit,
  recruitment/training, market, Chronicle, Heritage, Help, Display, mute, and reset paths.
- [ ] Judge whether event prose is comfortable to read for a full voyage, not merely in a
  screenshot. Record eye strain, decision overload, dead time, accidental clicks, unclear
  consequences, and repeated copy.
- [ ] Judge audio repetition and fatigue over a full session with speakers and headphones;
  verify nothing essential is conveyed by sound alone.
- [ ] Intentionally attempt unaffordable, unavailable, contradictory, and destructive
  actions; confirm recovery and messages make sense to a first-time player.
- [ ] Run one ordinary voyage without design foreknowledge, or recruit an uninvolved
  tester who can. The developer cannot simulate novice comprehension reliably.

### Hardware and operating-system matrix

- [ ] Test the minimum supported Windows version on real hardware or a representative VM.
- [ ] Test at least one integrated-GPU/laptop system and one discrete-GPU desktop from
  different vendors where feasible.
- [ ] Test 1280×720, 1920×1080, and 2560×1440; 100%, 125%, 150%, and 200% DPI where
  available; windowed/full-screen behaviour if claimed; and a multi-monitor move.
- [ ] Test mouse/trackpad and keyboard. If controller support is not implemented, ensure
  the store does not claim it. Touch is not a required PC-store feature but should remain
  functional because it is an established project standard.
- [ ] Test speakers/headphones, audio disabled, focus loss, sleep/wake, monitor change,
  abrupt termination, Alt+F4, and restart.
- [ ] Test under a non-administrator account, an install path with spaces, a non-ASCII
  Windows username, and standard antivirus/SmartScreen conditions.
- [ ] Verify uninstall/reinstall and store updates do not remove or corrupt saves.

### External playtest

- [ ] Recruit at least 3–5 people who did not implement the game; include at least two who
  enjoy strategy/narrative games and one who is relatively unfamiliar with the design.
- [ ] Provide no coaching beyond the shipped Help. Observe where they stop, misread state,
  fail to find a control, or quit.
- [ ] Collect explicit consent before recording sessions or retaining personal data.
- [ ] Triage feedback with the agent: block release for crashes, data loss, progression
  deadlocks, false store claims, unreadable primary flow, or repeatable severe input bugs;
  consciously defer ordinary preference requests.
- [ ] Personally approve the final pacing, difficulty, onboarding, readability, content
  tone, and price after seeing the results.

## 5. Review and submit storefront material

### Steam store presence

- [ ] Approve every capsule, library asset, icon, and screenshot. Confirm screenshots are
  real gameplay and capsules contain only permitted title/art elements.
- [ ] Approve the short/long descriptions, tags, supported languages, system requirements,
  developer/publisher identity, website/support links, release date, price, content
  descriptors, and all feature checkboxes.
- [ ] Check the page on desktop and mobile layouts and verify small-capsule title
  legibility.
- [ ] Personally submit the Store Presence for review and respond to Valve feedback.
  Valve says review typically takes 3–5 business days and recommends at least seven
  business days of lead time.
- [ ] After store approval, personally publish the Coming Soon page and verify wishlist
  and community links.

### Steam build

- [ ] Authorise the build account and initial SteamPipe test upload without exposing
  long-lived credentials in the repository or chat.
- [ ] Install the private-branch build through the Steam client on clean machines and
  complete the owner acceptance smoke test.
- [ ] Approve the launch option, depot/package ownership, default branch selection,
  reviewer instructions, and near-final build.
- [ ] Personally complete the content survey and ratings questions, then submit the Game
  Build for review. Resolve any Valve feedback with the agent and repeat the human smoke
  test on the revised build.

### itch.io page and build

- [ ] Approve the cover, screenshots, page presentation, classification, Windows-only
  platform flag, English language, tags, AI disclosure, price, payment mode, install
  instructions, and support information.
- [ ] Authorise butler once, keeping the login token under human control.
- [ ] Download and install the restricted-page Windows build through browser and itch app;
  verify its hash/version and complete the smoke test.
- [ ] Keep the page Draft or Restricted until the final coordinated release decision.

## 6. Decide go/no-go and perform the release

- [ ] Select a release date that respects the 30-day Steam fee wait, two-week Coming Soon
  minimum, review lead time, tester availability, and the owner's ability to support the
  game for the next several days.
- [ ] Review the final defect ledger. No known P0/P1 defect, data loss, progression
  blocker, startup failure, false disclosure, unresolved rights issue, or store/build hash
  mismatch may be waived silently.
- [ ] Confirm rollback artifacts and instructions work and identify the person authorised
  to withdraw a bad build or post an incident update.
- [ ] Freeze the exact commit and artifact hash. Personally sign the release decision and
  prevent unrelated changes from entering the build.
- [ ] Approve and trigger the final itch.io visibility change.
- [ ] In Steamworks, use the human-controlled release process to release the approved app.
  The agent may accompany the checklist but should not make the irreversible public
  release decision on implied permission.
- [ ] Buy/download the public product like a customer where practical and run the public
  smoke test on both stores.
- [ ] Publish the approved launch announcement and monitor support/store dashboards during
  the chosen coverage window.

## 7. Human post-release obligations

- [ ] Decide whether each reported problem requires support guidance, hotfix, rollback,
  refund accommodation, store-copy correction, or no action.
- [ ] Approve all public incident statements, promises, discounts, key grants, refunds,
  or changes to customer-facing dates and features.
- [ ] Protect player reports and personal information; do not paste private data, crash
  paths containing usernames, credentials, or tax/store records into agent prompts.
- [ ] Re-authorise release credentials only when needed and revoke compromised or
  unnecessary tokens.
- [ ] Review sales/tax/payout reports and fulfil local accounting and tax obligations with
  a qualified professional where appropriate.
- [ ] Decide the supported patch horizon and communicate any end-of-support decision.

## Human sign-off record

Complete this for the exact public candidate:

| Decision | Value |
| --- | --- |
| Legal publisher | |
| Rights/provenance approved by | |
| Steam App ID / itch owner-slug | |
| Public version | |
| Commit hash | |
| Windows archive SHA-256 | |
| Steam Build ID | |
| itch channel/version | |
| Price and launch discount | |
| Steam fee-paid date | |
| Coming Soon public date | |
| Store/build review approvals | |
| Clean-machine smoke-test machines | |
| Known blockers | |
| Rollback owner | |
| Release date/time/time zone | |
| Final go/no-go owner and date | |

## Minimum human completion gate

The game is ready to publish only when:

- Legal publisher, bank/tax/payment onboarding, rights provenance, AI/content disclosures,
  privacy truth, price, support contact, and store claims are explicitly approved.
- The owner and uninvolved testers have played the exact store-delivered RC far enough to
  prove the first charter, Homecoming, persistence, and second-charter loop.
- The supported Windows matrix and store-client installation/update/save paths have been
  exercised by people on clean environments.
- Both store pages and builds are approved, timing requirements are satisfied, and the
  release/rollback owner is available.
- The repository's automated release gate must remain green for the frozen release
  commit. The duplicate `the_seized_works` startup fault found on 2026-08-26 is fixed and
  regression-tested; the publisher now launches the exact packaged Windows executable
  and requires a rendered frame instead of trusting compilation alone.

## Current official platform references

Checked 2026-08-26; the person submitting must re-check the live requirements.

- Steamworks onboarding and commercial requirements:
  <https://partner.steamgames.com/doc/gettingstarted/onboarding>
- Steam review and release process:
  <https://partner.steamgames.com/doc/store/review_process> and
  <https://partner.steamgames.com/doc/store/releasing>
- Steam Coming Soon requirement: <https://partner.steamgames.com/doc/store/types>
- Steam content and generative-AI survey:
  <https://partner.steamgames.com/doc/gettingstarted/contentsurvey>
- Steam store assets: <https://partner.steamgames.com/doc/store/assets>
- itch.io page creation: <https://itch.io/docs/creators/getting-started>
- itch.io quality and AI-disclosure rules:
  <https://itch.io/docs/creators/quality-guidelines>
- itch.io payments and tax interview: <https://itch.io/docs/creators/payments>
- itch.io butler requirements: <https://itch.io/docs/butler/pushing.html>
