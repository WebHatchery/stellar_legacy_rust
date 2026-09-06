# Stellar Legacy: player AI identity implementation plan

Date: 2026-09-06
Status: proposed implementation; this document does not change the current design or game behavior.
Project: `D:/WebHatchery/RustGames/stellar_legacy`.
All paths below are relative to this project. Files marked **new** are proposed.

## 1. Outcome and scope

The player is the Custodian, the persistent intelligence aboard a generation ship. Captains and councils are human partners with their own authority, priorities, and mortality. The AI operates the vessel, advises its government, and remembers the consequences of choices across generations.

Canonical introduction:

> You are the intelligence aboard a generation ship. Captains age. Councils change. You remain—carrying their promises, mistakes, and hopes across the centuries.

Short description:

> Be the mind of a generation ship. Guide generations of captains and carry their promises across the stars.

“Conscience” is a responsibility the player interprets. The game must allow compassionate, pragmatic, and severe decisions without assigning the player an involuntary inner monologue. Human reactions may judge those decisions.

Required scope:

- Establish this identity consistently in design, onboarding, decisions, records, and public descriptions.
- Convert the existing Custodian from a separate character into the player identity.
- Give human leadership a small, explicit, enforceable role in major decisions.
- Connect succession to persistent relationships and recorded promises.
- Preserve existing campaigns and clearly explain continuity and campaign endings.

Excluded from this implementation: a conversational language model, free-text dialogue, a new avatar system, neural-network mechanics, a full parliamentary simulator, AI rebellion campaigns, and new cross-campaign immortality mechanics. Existing Rust simulation and authored JSON remain the implementation model.

## 2. Verified starting points

| Existing surface | What exists | Required change |
| --- | --- | --- |
| `gdd.md`, sections 1 and 4; `README.md` | Player explicitly described as the standing council | Replace the identity and document the division of authority |
| `game_page.json`; `docs/release/STORE_COPY.md` | Command-a-starship marketing language | Identify the player as the ship intelligence in the first sentence |
| `assets/data/game_config.json`; `src/data/config/onboarding.rs` | Data-driven welcome and tutorial content | Explain identity, founding mandate, captain relationship, and exact touch actions |
| `assets/events/mystery.json` | Sixteen Custodian-related events, including grief, refusal, personhood, and personality consequences | Rewrite viewpoint and remove cases where the player implicitly controls the humans judging a separate player AI |
| `src/state/sim.rs`; `src/ui/dashboard/status.rs` | Custodian disposition and empathy-related state/display | Present this as perceived conduct, preserving compatible state identifiers |
| `src/simulation/advice.rs`; `assets/crew_archetypes.json` | Named officer advice | Retain human voices and add a distinct captain position where needed |
| `src/state/sim/dynasty.rs` | Living members, designated heir, persistent `Reign` records, inherited obligation count | Preserve captain history; add only missing relationship/authority information |
| `src/state/sim/obligations.rs` | Creator, responsible captain, beneficiary, dates, status, succession count, history | Use these records for truthful AI memory callbacks |
| `src/simulation/command.rs` | Mechanical command postures and review timing | Make posture an initial, bounded place for human disagreement |
| `src/game/realtime.rs` | Countdown invokes automatic decision resolution; dynasty extinction stops advancement | Attribute automatic choices accurately and explain why an AI campaign can end |
| `src/save.rs` | Toolkit save persistence and migration hook | Add explicit semantic migration when new state cannot be handled by defaults |

Do not confuse the Custodian AI with school custodianship. `custodian_faction_id` in `src/state/sim/institutions.rs` describes human institutional stewardship and should retain that meaning.

## 3. Canonical design decisions

### 3.1 Identity and continuity

- One Custodian persists throughout one campaign, across captain deaths, retirement, voyages, and drydock.
- A new campaign commissions a new instance. Chronicle/Heritage is inherited archival knowledge, not proof that the current AI personally witnessed another campaign.
- Legacy and founding peoples describe the society that commissioned the AI and its founding mandate. They are not the AI's biological family or a mandatory personality selection.
- The AI knows recorded history and available instrumentation. It cannot know private thoughts or reconstruct unrecorded conversations.
- The terminal interface is the AI's operational view. Human messages are attributed; system notices remain factual.

### 3.2 Authority contract

| Action | Player authority | Human authority and implementation rule |
| --- | --- | --- |
| Resource allocation, repairs, trade within existing rules, crew training | Execute under standing mandate | Ordinary actions remain immediate; no new per-click approval |
| Charter acceptance/abandonment | Recommend a commitment | Show the captain's ratification and reason; use existing eligibility/cancellation rules initially |
| Strategic posture | Propose policy and implement an approved posture | First mechanical disagreement surface; a captain can object under authored conditions |
| Social/ethical event options | Advise or exercise explicitly delegated authority | Option metadata determines whether human ratification is needed |
| Heir designation | Nominate an eligible successor | Initial version keeps existing selection rules and states that the charter permits nomination; do not advertise a simulated council vote |
| Emergency action | Invoke a narrow, defined emergency mandate | Only authored emergencies permit override; costs and review consequences shown before selection |
| Delegation | Entrust a domain to a human office | Automatic result names the acting office/person and records its reason |

Do not label an immediate, unconditional decision “Recommend” unless the UI also makes clear that standing authorization makes that recommendation effective. Do not display a captain veto that the action dispatcher ignores.

### 3.3 Death and failure

Preserve current dynasty-extinction gameplay in the first release of this identity change. Explain that the founding commission has ended and the Custodian is archived/decommissioned when the line can no longer sustain command. This is a campaign-ending rule, not the AI dying with a captain.

Update extinction notices, game-over text, debrief/Chronicle terminology, and GDD together. If continuing under another family is desired later, treat it as a separate succession redesign: replacement eligibility, legitimacy, inheritance, tutorial, autoplay, and ending rules all need implementation. Do not silently enable indefinite continuation in this work.

## 4. Work package A — authoritative design and terminology

Files: `gdd.md`, `README.md`, `content_depth.md`, `event_design_notes.md`, `TODO.md`.

Tasks:

- Rewrite GDD sections 1 and 4 and adjust pillars/core loop/succession/persistence descriptions wherever they imply the player changes bodies or is the council.
- Add the authority table, campaign continuity rule, and extinction explanation.
- Define narrative conventions: “you” means the Custodian; “captain” and “council” mean human actors; “we” only appears in attributed collective speech.
- Describe empathy as perceived conduct rather than an authoritative reading of the player's emotions.
- Add event-authoring rules for decision ownership, actor attribution, memories, and human objections.
- Add work packages from this plan to `TODO.md` with dependencies and completion criteria.
- Retain valid references to council meetings, human captains, and institutional custodians. Review occurrences manually rather than globally replacing words.

Done when a designer can assign every current core verb to an actor and explain what survives succession without consulting this proposal.

## 5. Work package B — identity in the playable interface

Files: `assets/data/game_config.json`, `src/data/config/onboarding.rs`, `src/ui/welcome.rs`, `src/ui/main_menu.rs`, `src/ui/shell.rs`, `src/ui/help.rs`, `src/ui/tutorial.rs`, `src/game/tutorial.rs`, `src/ui/prep.rs`, `src/ui/dashboard.rs`, `src/ui/dashboard/status.rs`, `src/ui/crew_dynasty.rs`, `src/ui/mission.rs`, `src/ui/game_over.rs`.

Tasks:

- Welcome: replace the new-commander address with commissioning of the Custodian. Explain its mandate in a short briefing.
- New game: describe legacy/founding peoples as the commissioning society. Preserve the current choices and mechanical effects.
- Shell/dashboard: expose an unobtrusive identity label such as `CUSTODIAN // SHIP INTELLIGENCE`; separately name the serving captain.
- Captain panel: distinguish the AI's mandate from the captain's skills, term, priorities, and inherited duties.
- Help: add “Who am I?” and “Who decides?” explanations using actual implemented rules.
- Tutorial: teach one direct operational action and one interaction with human authority. Prompts name visible controls, for example “Tap REVIEW MANDATE.” Add such a control before referencing it.
- Returning players: keep identity visible outside the once-per-install welcome. A migrated save must not depend on replaying onboarding to explain the new framing.
- Ending: identify loss of the commission or vessel rather than imply the player was the deceased captain.
- Keep prose in existing authored configuration where practical; add typed fields only when a new UI element requires them.

Done when a fresh player and a returning-save player can identify themselves and the current captain from normal UI, with no mandatory keyboard input and no clipped welcome/tutorial text.

## 6. Work package C — convert the Custodian narrative

Primary files: `assets/events/mystery.json`, `assets/data/game_config.json`, `src/data/tests/content.rs`, `src/data/tests/voice.rs`, `src/data/tests/event_gates.rs`, `src/state/sim.rs`, `src/ui/dashboard/status.rs`.

Audit all sixteen current events:

| Event IDs | Rewrite requirement |
| --- | --- |
| `the_spare_calculation`, `the_emotional_module` | Human requests or proposed model revisions addressed to the player; the player chooses its response |
| `the_custodians_first_grief`, `the_machine_dream` | Present an opportunity to remember/create; do not assert grief or a private dream the player never chose |
| `mercy_without_orders`, `the_cold_equation` | Make the operational decision belong to the AI and identify human approval or objection |
| `the_right_to_refuse`, `the_override_habit` | Frame as negotiations over the player's mandate; human reaction must not be another player-controlled vote |
| `the_night_channel`, `the_deleted_names` | Make privacy, memory retention, and archival disclosure explicit player choices |
| `the_joke_that_hurt`, `the_mercy_bug` | Avoid unexplained involuntary player speech/actions; use a proposed response or a documented consequence of prior policy |
| `the_remembered_birthdays`, `the_gentle_alarm` | Gate automatic acts on an established player-approved policy, or make them selectable decisions |
| `the_queue_without_appeal`, `the_efficient_farewell` | Tie severe conduct to actual earlier choices; no personality score alone should invent a consequential action |

For every event, review description, option label, option description, outcome log, gates, variables, flags, scheduled follow-ups, and test expectations. Preserve IDs and compatible flags so queued events and old saves still resolve.

Rework `custodian_kind` and `custodian_severe` flavor pools as attributed crew observations or factual summaries. Replace unsolicited “I believe/I feel” statements with player-selected responses where an internal stance matters.

Review every other `assets/events/*.json` file for ambiguous “you,” captain/council impersonation, and AI-versus-player confusion. `assets/events/obligations.json`, `ethics.json`, and `diplomacy.json` are priority follow-ups. Review `assets/legacies.json`, `contracts.json`, and `crew_archetypes.json` for the same issue.

Done when the AI is never treated as a second, independently controlled protagonist and no rewritten event breaks its existing chain or save identity.

## 7. Work package D — enforce a bounded human authority model

Existing integration points: `src/data/events.rs`, `src/state/sim.rs`, `src/state/sim/dynasty.rs`, `src/state/sim/campaign.rs`, `src/simulation/advice.rs`, `src/simulation/command.rs`, `src/simulation/event_resolver.rs`, `src/simulation/event_resolver/outcome.rs`, `src/game/actions.rs`, `src/ui/event_modal.rs`, `src/ui/crew_dynasty.rs`.

Proposed new ownership:

- `src/state/sim/authority.rs`: serializable mandate and current captain relationship state.
- `src/data/authority.rs`: typed authored authority rules, if the initial rule set warrants a dedicated schema.
- `src/simulation/authority.rs`: pure eligibility/objection evaluation and explicit state transitions.
- `src/ui/event_modal/authority.rs`: actor, objection, and consequence presentation.
- Separate `foo/tests.rs` children for meaningful behavior tests.

Minimum model:

- A captain-specific priority derived deterministically from existing traits/specialization, with authored fallback for unknown traits.
- A small explicit mandate state, initially routine operations plus narrowly defined emergency powers.
- Event-option authority classification: standing mandate, human ratification, emergency override. Default legacy options to standing mandate to avoid mass migration breakage.
- An evaluation result containing permission, acting authority, objection/reason, and visible costs. Both UI and dispatch must use the same evaluation.
- Add a relationship metric only if a concrete rule consumes it. Existing morale/stability/influence must not be relabeled as “trust in you” because they describe different things.

First complete scenario: a captain objects to a proposed command posture under a documented condition. The player can retain the current posture or choose an available compromise. An override is offered only if a real emergency condition permits it. Show the exact existing/postulated costs before commitment; choose and document balance values during implementation rather than implying this plan contains validated tuning.

Re-evaluate permission at dispatch. Apply cost, change, and record atomically; rejected actions consume nothing. Ensure all paths, including automatic resolution, use the same rules. Authority changes must never remove every valid response from a blocking decision.

Scope limit: ordinary allocations, repairs, and purchases remain immediate. Expand ratification to other major decisions only after the first scenario is playable and validated.

Done when the captain can affect at least one meaningful outcome, the player has a clear recovery choice, and displayed authority matches enforced behavior.

## 8. Work package E — succession and personal memory

Files: `src/simulation/succession.rs`, `src/simulation/mortality.rs`, `src/simulation/tick/beats/succession.rs`, `src/state/sim/dynasty.rs`, `src/state/sim/obligations.rs`, `src/simulation/debrief.rs`, `src/ui/debrief.rs`, `src/ui/debrief/columns.rs`, `src/ui/chronicle.rs`.

Tasks:

- At both retirement and death-driven succession, preserve AI/mandate history while replacing the captain-specific relationship and priority.
- Show a short handover: outgoing captain, incoming captain, one inherited duty, new priority, and any actual authority change.
- Keep existing heir eligibility and selection mechanics; reframe the player action as charter-authorized nomination.
- Generate callbacks from `Reign` and `Obligation` records, not the trimmed general log.
- Prefer active, relevant obligations; choose deterministically and deduplicate by source record, transition, and callback kind.
- Example template: “Captain {creator} recorded this promise in year {created_year}. Captain {responsible} now carries it. It has crossed {successions_crossed} successions.” Use an AI-witnessed version only when provenance supports it.
- Preserve creator, beneficiary, and resolution facts even after the original captain leaves the living roster.
- Add a compact memory-reference structure only for facts absent from existing records. Do not duplicate the full log or invent historical conversations.
- Homecoming: connect one promise or decision to the captains who carried it. Chronicle distinguishes current campaign experience from inherited records.
- Missing old-save provenance renders a neutral archival statement, not fabricated first-person recollection.

Proposed new module if needed: `src/simulation/memory.rs` for deterministic selection/formatting, with `src/simulation/memory/tests.rs`. Persistent provenance belongs in a small state child module only if existing records cannot represent it.

Done when one promise survives multiple successions, is recalled accurately, and reaches a debrief without duplicate callback spam.

## 9. Work package F — timeouts, delegation, and save compatibility

Files: `src/game/realtime.rs`, `src/game.rs`, `src/game/actions.rs`, `src/simulation/event_resolver.rs`, `src/simulation/autoplay.rs`, `src/save.rs`, `src/save/tests.rs`, `src/chronicle.rs`, `src/state/sim/campaign.rs`.

### Automatic decisions

- Trace `auto_resolve_decision` and every delegation path before changing behavior.
- Replace the unqualified random timeout choice for identity-sensitive decisions with an authored, legal human fallback or a held decision when no fallback exists.
- Label the countdown with who will act. After timeout, log the actual human actor and chosen response; never attribute a random response to the player's convictions.
- Preserve Pause and tutorial clock-hold behavior. Reset the countdown for each distinct decision, including repeated event templates; retain the recent countdown-reset fix.
- Do not consume RNG during rendering or option previews. Document any intentional replay/balance change from replacing random choices.
- Update autoplay so authority restrictions cannot create infinite loops or illegal resolutions.

### Persistence

- Keep serialized Custodian/event IDs where their meaning remains compatible.
- Default new optional fields; migrate any fields whose semantics change. The existing migration hook mostly deserializes and updates the version, so do not assume it already performs semantic conversions.
- Distinguish new-campaign initialization from old-save initialization. Do not reset empathy, active obligations, captain history, selected heir, or event-chain progress.
- Preserve a pending Custodian event across upgrade, and handle saves made before captain history existed.
- Persist any pending authority review/fallback information needed to resume exactly once.
- Keep Chronicle/Heritage format stable unless an explicit, tested extension is necessary.

Done when representative older campaigns load, pending decisions remain resolvable, and save/reload cannot duplicate costs, reviews, or memories.

## 10. Work package G — public copy and release evidence

Files: `game_page.json`, `docs/release/STORE_COPY.md`, `README.md`, `docs/release/RELEASE_DEFINITION.md`, `docs/release/QA_AND_OPERATIONS.md`, `docs/release/STORE_MEDIA_PROVENANCE.md`, `src/game/capture_scenes.rs`, `docs/verification/`, `catalog_thumbnail.png`.

- Update descriptions to match completed mechanics. Do not advertise meaningful captain resistance until package D is implemented.
- Audit support/release text for commander/player identity assumptions.
- Refresh welcome, tutorial, menu, dashboard, Custodian event, succession, and debrief captures when their screens change; replace matching existing files directly under `docs/verification/`.
- Add a reproducible authority-disagreement capture. Split capture responsibilities into a cohesive child file if needed.
- Refresh the catalog thumbnail only if the title screen changes. Refresh affected store screenshots and provenance together.
- Public store upload remains a separate release action. This plan does not request it.

## 11. Validation and acceptance

For every meaningful implementation milestone, run `./publish.ps1` with no parameters from the project directory. Report an unrelated environment failure explicitly; do not substitute a local server. Use focused tests for behavior changed, plus the publisher's required checks.

Required behavior cases:

1. New game and resumed game both identify the player as the Custodian.
2. Routine operational actions still execute immediately.
3. A captain objection changes an available strategic action; a legal alternative always exists.
4. Dispatch rejects stale/illegal choices without cost; accepted actions apply exactly once.
5. Death and retirement both preserve AI continuity and update captain-specific state once.
6. Unknown or missing captain traits have a deterministic fallback.
7. A pending obligation retains its creator and truthful history across multiple successions.
8. Missing history yields neutral archival prose; no fabricated memory is emitted.
9. Timeout/delegation names its actor, obeys authority, respects Pause, and handles repeated decisions.
10. Legacy saves, pending rewritten events, and new saves round-trip without loss of progress.
11. Dynasty extinction gives the commission-ending explanation and retains existing terminal behavior.
12. Full game and Lumen Relay demo onboarding remain touch-completable.

UI verification: common desktop browser sizes, long captain/obligation names, wrapping, modal scroll/reachability, visible touch targets, and no keyboard-only next step. Use existing capture tooling for evidence, with no unrequested local server.

Store tests in separate child files. Every physical line counts toward the 800-line Rust limit, including blanks and comments. Before touching large files such as `src/game/actions.rs`, `src/game/capture_scenes.rs`, `src/ui/chronicle.rs`, and `src/ui/dashboard.rs`, count lines and extract cohesive responsibilities if needed. Do not compress formatting to create space.

Content review is also required: keyword checks can find candidates but cannot establish that every “council” or “captain” reference is wrong. Manually read complete changed event chains.

## 12. Delivery order and commit boundaries

| Milestone | Packages | Deliverable and exit gate |
| --- | --- | --- |
| 1. Canonical identity | A | Consistent documented identity, authority, and ending rules; no claims that mechanics are already implemented |
| 2. Playable perspective | B + C | UI and Custodian content consistently address the player AI; event-chain and visual checks pass |
| 3. Human authority | D + required F persistence/automatic-resolution work | One enforceable disagreement with legal fallback; save and timeout tests pass |
| 4. Generational continuity | E + remaining F | Succession, memory, migration, and ending cases pass |
| 5. Release alignment | G | Public copy matches behavior; refreshed evidence and publisher validation complete |

Finish, validate, and commit each independently useful milestone before starting the next. Stage all modified/untracked project files as required by project instructions; inspect the working tree first and report included pre-existing changes. Use the catalog's narrative subject with an explicit parenthetical subsystem tag and an honest explanatory body.

No shared toolkit changes are anticipated for identity, authority, or memory: these are game-specific responsibilities. If implementation reveals missing generic rendering/input/loading behavior, evaluate a toolkit improvement before adding a local substitute. JSON continues through `macroquad_toolkit::data_loader`; no new generic loader or dependency is needed.

Final player comprehension check: after onboarding and one succession, the player can answer “Who am I?”, “What can I decide?”, “What can the captain refuse?”, and “Why does this old promise matter to me?” using what the game has actually shown.
