# Event and content authoring

Use [gdd.md](gdd.md) for implemented systems and [TODO.md](TODO.md) for open
work. This is the authoring contract for current event data, not a content-pass
history or a request to add more events indefinitely.

## Ownership and voice

The player is the persistent Custodian. Captains, councils and officers are human
actors. Attribute speech, objections and fallback actions explicitly. Describe
observed state, selected policy and recorded consequences; do not invent private
Custodian feelings, unchosen conduct or human thoughts.

For every identity-sensitive choice, establish:

1. Whether it uses standing mandate, human ratification or authored emergency power.
2. Which human office acts on timeout, and which available outcome the resolver can select.
3. The durable consequence, obligation or reputation record used by later callbacks.

Automatic acts require an established policy gate or a selectable decision.
Severe actions need an actual prior choice/flag; a reputation score alone cannot
invent one. Preserve event IDs and compatible flags when rewriting prose because
pending events and chains can survive save upgrades. Use Reign/Obligation facts
for memory, with neutral archival wording when old saves lack provenance.

## Data layout and inventory

[GameData](src/data.rs) embeds each family file and merges it into one registry
with a duplicate-ID guard. Add events to the correct file; a new family also
requires registration in `EVENT_FILES`. All files are embedded on native/WASM,
so changing JSON requires rebuilding. Generic parsing belongs to toolkit
`data_loader`; game-specific schema and validation remain here.

| Family/file under `assets/events/` | Events |
| --- | ---: |
| biology_medical | 26 |
| comedy | 17 |
| diplomacy | 38 |
| engineering | 35 |
| ethics | 16 |
| exploration_first_contact | 16 |
| legacy_drift | 80 |
| mystery | 35 |
| obligations | 12 |
| science_anomaly | 18 |
| survival | 50 |
| **Total** | **343** |

These are the current authored counts, not expansion quotas. EventCategory has
four values: immediate crisis, generational challenge, mission milestone and
legacy moment. Category controls scoring/delegation; family organizes content
and selection. [data/events.rs](src/data/events.rs) defines the exact schema.

## Families, complications and gates

Build situations from reusable families, state-dependent complications and
meaningfully different outcomes. Gate century-specific writing by year,
generation, drift or durable history. Complications use the first matching gate;
`applies_to_outcomes` can attach the extra toll only to named choices while the
twist remains visible. Keep unknown rolls distinct from known costs/effects.

Outcome requirements can read expertise, reputation, consequences and capabilities.
Check affordability and authority at dispatch, not just in the UI. Preserve a
legal escape path for blocking decisions, and test automatic policies against
unavailable/unaffordable branches. Keep deliberate force-return and faction-loss
branches out of the first default outcome. Do not rely on array order as proof
that an outcome is legal.

The campaign skeleton's phase and era pools are authored in
`game_config.json` under `campaign_skeleton`; they are **not a hard-coded Rust
family table**. [skeleton.rs](src/simulation/event_resolver/skeleton.rs) combines
those pools with charter biases and seeded placement. Current windows are 240
months and skip the first 60 months. Threshold/recovery beats, scheduled follow-ups
and dead-air handling supplement that schedule. An over-gated family can fall
through to normal selection; test reachability rather than counting templates.

## Persistent consequences

Use consequence flags for historical facts, obligations for duties with ownership
and lifecycle, and issue records for ongoing aftermath. Include correct creator,
beneficiary, timing and succession facts. Do not duplicate the full general log.
Authored `food_production_penalty` applies while an issue is active; clearing the
issue does not refund the initiating event's loss. Project capabilities should
unlock a concrete authored response only after valid completion.

Give prepared and neglected ships recognizably different choices and aftermath.
Costs must be connected to existing production, consumption, training, maintenance
and scarcity. Do not introduce a second economy, speculative meters or reward
buttons whose effects are only flavor. Use local typed schemas and reusable
simulation operations rather than chain-specific branches in Rust.

## Review and validation

For a content pass, select one weakness in existing play: recurring situations,
weak consequence chains, flat charter differences, succession without payoff,
or unclear faction/ship relationships. Read the relevant complete chain before
editing. Prefer richer branches and cross-system effects over filler volume.

Check unique IDs, valid references, family registration, phase/era reachability,
legal choices, identity attribution, staged effects and save compatibility in the
separate child tests under `src/data/tests/` and the relevant simulation module.
Use deterministic paired scenarios where preparation is meant to change an outcome.
See [balance_report.md](balance_report.md) for current evidence limitations.

Run `.\publish.ps1` without parameters after meaningful changes. Review changed
screens at their actual dimensions, including full prose, long names, scrollable
consequences and touch-only recovery. Store captures directly in
`docs/verification/`, replacing earlier examples of the same state. Follow
[AGENTS.md](AGENTS.md) for commit boundaries; no separate rotation diary is needed.
