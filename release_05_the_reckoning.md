# Release 5 — The Reckoning

## Goal

Turn Homecoming into the campaign's decisive comparison and recovery moment. Returning
to port should reveal how the ship, its people, and its obligations changed—and force a
choice about what will be repaired before the next voyage.

## Player-facing outcome

The debrief compares departure with return across material, social, generational, and
institutional dimensions. After reading it, the player selects one major recovery
project that addresses a lasting wound or invests in a hard-won strength.

## Dependencies

- Release 1 supplies obligation accounting.
- Release 2 supplies officer and institutional continuity.
- Release 3 supplies voyage doctrine evaluation.
- Release 4 supplies compartment identity and local history.

## Scope

### Departure snapshot

Capture the state required to compare departure and return, including:

- ship, subsystem condition, and subsystem knowledge;
- resources and population;
- faction membership and approval;
- morale, unity, stability, loyalty, adaptation, and cultural drift;
- officers and institutions;
- obligations;
- compartment identities;
- reputation traits.

Only values that produce a meaningful comparison should be shown.

### Homecoming report

Organize the debrief around four questions:

1. Did the charter succeed?
2. What did the voyage cost?
3. Who and what changed?
4. What remains unresolved?

Show major changes, not an unfiltered dump of every number. Highlight deaths,
appointments, successions, faction movements, knowledge losses, promises, new customs,
and irreversible changes.

### Recovery projects

Offer context-sensitive projects such as:

- reconcile estranged peoples;
- rebuild a damaged school;
- restore confidence in an office;
- hold a public accounting over a broken promise;
- memorialize losses;
- protect or dismantle a compartment tradition;
- prioritize full material refit over social recovery.

Projects consume real resources, influence, political goodwill, or opportunity. They
must improve a damaged campaign without erasing its history.

### Carry-forward

- The chosen project applies before the next charter is accepted.
- Unaddressed wounds remain visible in drydock and can affect later content.
- Recovery outcomes enter the Chronicle.
- The next charter dossier surfaces relevant unresolved conditions.

## Implementation shape

- Extend the existing voyage snapshot and sealed debrief instead of creating a parallel
  report system.
- Define recovery projects through data with reusable gates, costs, and effects.
- Rank and group changes so the UI emphasizes meaningful deltas.
- Preserve the current port-only repair/loadout rules.

## Non-goals

- Resetting morale, culture, factions, or knowledge after every voyage.
- A free recovery action that always repairs the worst stat.
- Turning drydock into real-time simulation.
- Making every voyage end in catastrophe.
- Replacing the existing success scorecard.

## Validation

- Test departure/return snapshots across succession and save/load.
- Test project eligibility, cost, application, and persistence.
- Verify a recovery project cannot erase historical facts or completed obligations.
- Extend autoplay with an explicit recovery policy.
- Capture successful, pyrrhic, socially fractured, and knowledge-damaged homecomings.
- Run the balance matrix across multiple consecutive voyages.
- Run `.\publish.ps1` successfully.

## Complete means

This release is complete when:

- [ ] Homecoming compares meaningful departure and return state across all major systems.
- [ ] The report clearly distinguishes achievement, cost, change, and unresolved work.
- [ ] Obligations, officers, institutions, approaches, and compartment identities appear
      when relevant.
- [ ] The player chooses one consequential, context-sensitive recovery project.
- [ ] Recovery creates a credible path out of a damaged campaign without resetting it.
- [ ] Unresolved wounds carry into drydock, charter preparation, and later events.
- [ ] Multi-voyage balance tests demonstrate both deterioration and recovery arcs.
- [ ] Save compatibility, deterministic tests, autoplay, UI captures, and
      `publish.ps1` pass.
