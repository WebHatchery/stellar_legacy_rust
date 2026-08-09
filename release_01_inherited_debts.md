# Release 1 — Inherited Debts

## Goal

Make promises a visible, persistent campaign system. A commitment made during one
captain's reign must be capable of surviving succession, constraining later voyages,
and ending through fulfilment, renegotiation, or default.

This release strengthens Stellar Legacy's central promise: decisions create duties
that somebody else's descendants must eventually answer.

## Player-facing outcome

The player can inspect every active obligation, understand who is owed what and when,
and see how a new decision or charter will affect those obligations. Succession and
Homecoming explicitly show which debts were inherited, honoured, revised, or broken.

## Scope

### Structured obligations

Add a serializable obligation record with, at minimum:

- stable id and authored title;
- source event or charter;
- creator and current responsible captain or office;
- beneficiary faction, people, or external party;
- creation year and optional due year;
- public, private, or disputed visibility;
- pending, fulfilled, renegotiated, defaulted, or void status;
- material and reputational stakes;
- number of successions crossed.

Obligations must be deterministic, save-compatible, and data-driven where authored
content is involved.

### Lifecycle

- Event outcomes and charter outcomes can create obligations.
- Later events can inspect, advance, renegotiate, fulfil, or default them.
- Succession transfers responsibility and records the inheritance.
- Due obligations surface predictably rather than relying only on a random event roll.
- Conflicting charters or decisions warn the player before commitment.

### Content exemplars

Ship three complete obligation chains:

1. a sanctuary or refugee commitment;
2. a technical-aid or station-building promise;
3. an internal compact with an aboard people.

Each chain must cross a succession in ordinary play and support at least three endings:
honour, renegotiation, and default.

### Interface

- Add an obligations ledger to the Chronicle or command shell.
- Show beneficiary, owner, due date, status, and expected stakes.
- Mark inherited obligations on the dynasty/reign timeline.
- Show obligation conflicts in charter preparation.
- Add an obligation accounting section to Homecoming.

## Implementation shape

- Prefer a dedicated obligation state module over expanding the existing append-only
  `consequences` list into a mixed-purpose structure.
- Keep `consequences` for discrete historical facts; use obligations for stateful duties.
- Extend event and charter data only with reusable obligation operations, not
  chain-specific Rust branches.
- Add save defaults or migration handling for campaigns created before this release.

## Non-goals

- A general quest system unrelated to promises.
- Procedural generation of obligation prose.
- Hiding deadlines or penalties from the player.
- Replacing existing event consequence chains.
- Adding a diplomacy map or external empire simulation.

## Validation

- Unit-test creation, inheritance, due-date handling, fulfilment, renegotiation, and
  default.
- Test save round-trips and loading a pre-release save without obligations.
- Add deterministic tests for all three exemplar chains.
- Confirm autoplay can identify and resolve due obligations without deadlock.
- Capture the ledger, succession marker, charter warning, and Homecoming accounting at
  supported viewport sizes.
- Run `.\publish.ps1` successfully.

## Complete means

This release is complete when:

- [x] Obligations are first-class serialized state, distinct from consequence flags.
- [x] Events and charters can create and resolve obligations entirely through data.
- [x] An obligation can cross one or more captain successions without losing ownership
      or history.
- [x] The player can always see the duty, beneficiary, deadline, and stakes before it is
      resolved.
- [x] Charter selection identifies direct conflicts with active obligations.
- [x] Homecoming reports obligations created, inherited, fulfilled, renegotiated, and
      defaulted during the voyage.
- [x] Three multi-stage obligation chains ship with meaningfully different outcomes.
- [x] Old saves load safely, deterministic tests pass, and `publish.ps1` passes.

## Completion record

- 397 tests passed (396 unit tests plus the source-size gate; one release-balance
  analysis test remains intentionally ignored).
- `publish.ps1` built and packaged Windows and WebGL releases and deployed the preview.
- Chronicle ledger, PREP conflict warning, succession inheritance marker, and
  Homecoming accounting were captured and visually checked at 1600x900 and 1280x720.
