# Release 4 — The Living Hull

## Goal

Make the ship a cultural geography as well as a collection of mechanical subsystems.
Its compartments should remember who tends them, what happened there, and what customs
formed around decades of work and crisis.

## Player-facing outcome

The ship schematic shows each major area as a community with custodians, condition,
knowledge, a developed identity, and remembered history. That identity changes later
events and makes two ships with similar numeric condition feel socially different.

## Dependencies

- Release 2 supplies institutions and officer relationships that can become local
  traditions.
- Release 3 supplies operating doctrines whose repeated use may shape compartment
  identities.
- Existing six subsystem areas remain the geographical foundation.

## Scope

### Lightweight compartment identity

Each subsystem area can record:

- its principal tending faction or people;
- one active cultural descriptor;
- one remembered crisis or triumph;
- an institutional tradition or custodianship;
- an unresolved local grievance or privilege.

This is not a separate population simulation. It is a compact identity layer derived
from existing faction, subsystem, event, and institutional state.

### Identity development

Descriptors emerge from repeated or decisive play, such as:

- guild-led;
- communal;
- hereditary;
- ritualized;
- ration-hardened;
- improvised;
- archive-dependent;
- security-dominated.

The player must be told when an identity forms, changes, or is displaced.

### Mechanical consequences

Compartment identity can affect:

- subsystem wear, repair, or knowledge retention;
- faction approval and rivalry;
- event complications and gated outcomes;
- institutional upkeep;
- the political cost of refitting or replacing a module.

Every descriptor must provide at least one advantage and one vulnerability or cost.

### Ship schematic

Selecting a compartment reveals:

- condition and knowledge;
- officer and institutional links;
- tending people and approval;
- current descriptor;
- remembered event;
- active local issue.

Use distinct glyphs and labels while preserving the terminal-native visual language.

### Content exemplars

- Provide at least two possible descriptors for every subsystem.
- Add one event or complication per subsystem that reads its local identity.
- Add at least one conflict involving a refit that threatens an established community.

## Implementation shape

- Store compact compartment identity state keyed by subsystem id.
- Derive ordinary display facts from existing state rather than copying values.
- Use data-driven identity definitions, formation gates, and effects.
- Limit active descriptors to keep the schematic readable and balance tractable.

## Non-goals

- Individual simulation for every resident.
- Freeform deck construction or room placement.
- Tactical crew movement.
- A second faction roster per compartment.
- Unlimited stacking traits.

## Validation

- Test identity formation, replacement, persistence, and effect application.
- Test faction departure and subsystem replacement against local custodianship.
- Verify every descriptor has both a benefit and a cost.
- Confirm schematic information remains readable at common browser sizes.
- Capture neutral, developed, contested, and refitted compartment states.
- Run `.\publish.ps1` successfully.

## Complete means

This release is complete when:

- [ ] All six subsystem areas can develop persistent cultural identities.
- [ ] Every area exposes its custodian, history, institution, and current issue on the
      ship schematic.
- [ ] Every descriptor has a real systemic advantage and tradeoff.
- [ ] Events can gate or branch on compartment identity through data.
- [ ] Every subsystem ships with at least two descriptors and one identity-aware event.
- [ ] Refitting can create a social consequence when it disrupts an established area.
- [ ] The system remains lightweight and does not duplicate population or faction state.
- [ ] Save compatibility, deterministic tests, UI captures, and `publish.ps1` pass.
