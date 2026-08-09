# Release 6 — Command Archive

## Goal

Give Stellar Legacy's deepened systems distinct, terminal-native visual identities and
allow important decisions to be remembered differently by the council, dynasty, and
affected peoples.

## Player-facing outcome

Major screens are recognizable at a glance, generational change is visible, and the
Chronicle can show competing interpretations of consequential decisions without losing
the game's restrained command-terminal style.

## Dependencies

- Releases 1–5 provide the obligations, institutions, approaches, compartment identity,
  and recovery history that this release presents.
- Existing CRT palette, accessibility cues, ship schematic, and code-drawn UI remain the
  visual foundation.

## Scope

### Competing records

Consequential outcomes may provide short perspective fragments for:

- the official ship log;
- the ruling dynasty's account;
- one or more affected peoples.

Perspective fragments can influence later event language, public trust, Heritage text,
or a truth-and-reckoning event. They must not create parallel copies of mechanical
history: the underlying deed remains one authoritative fact.

### Chronicle presentation

- Show dated deeds, obligations, successions, institutions, recovery projects, and
  compartment changes on a navigable timeline.
- Clearly distinguish fact from interpretation.
- Let the player inspect conflicting accounts without hiding the mechanical outcome.
- Add captain plaques and inherited-promise markers.

### Symbolic visual kit

Create a reusable code-drawn kit containing:

- six subsystem glyphs;
- six founding-people seals;
- obligation status markers;
- event-family marks;
- captain plaques;
- cultural-identity tags;
- ship wear and refit marks.

Every symbol must have a text or shape-based redundant cue; color alone is insufficient.

### Distinct screen silhouettes

- Dynasty: lineage and reign timeline.
- Chronicle: dated institutional record.
- Obligations: ledger composition.
- Factions: seals and relationships.
- Preparation: charter dossier and approach selection.
- Ship: schematic-led cultural geography.
- Homecoming: departure-versus-return comparison.

Reserve strong contrast and motion for new damage, threshold crossings, succession,
Homecoming, and other meaningful changes.

## Implementation shape

- Reuse toolkit drawing and UI primitives; treat generally useful glyph, timeline, or
  relationship widgets as potential `macroquad-toolkit` improvements.
- Keep symbols procedural and reusable rather than introducing bespoke event art.
- Store perspective text in content data and keep mechanical facts authoritative.
- Maintain keyboard, mouse, touch, scaling, reduced-motion, and audio-independent cues.

## Non-goals

- Full-screen bespoke character or event illustration.
- Decorative CRT distortion that reduces readability.
- Tactical combat presentation.
- A conventional galaxy map.
- Hiding exact mechanical consequences behind unreliable narration.

## Validation

- Capture every major screen at 1440×900, 1920×1080, and 2560×1440.
- Verify text, shape, and color cues under the existing display/accessibility modes.
- Test Chronicle filtering, perspective selection, and save/load persistence.
- Confirm mechanical history remains identical regardless of displayed interpretation.
- Verify symbols remain legible at capture scale and do not depend on unsupported glyphs.
- Run `.\publish.ps1` successfully.

## Complete means

This release is complete when:

- [ ] Major screens have distinct compositions while remaining recognizably part of the
      same terminal interface.
- [ ] The reusable symbol kit covers subsystems, peoples, obligations, events, captains,
      cultural identities, and ship wear.
- [ ] Important decisions can show official, dynasty, and affected-people accounts.
- [ ] Fact and interpretation are visually and mechanically distinct.
- [ ] The Chronicle communicates succession and institutional change across centuries at
      a glance.
- [ ] All new visuals retain redundant non-color cues and supported input paths.
- [ ] Required viewport captures pass visual review without clipping or illegible text.
- [ ] Save compatibility, deterministic tests, accessibility checks, and
      `publish.ps1` pass.
