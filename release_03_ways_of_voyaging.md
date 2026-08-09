# Release 3 — Ways of Voyaging

## Goal

Make pre-launch preparation determine how a charter can be attempted, not only whether
the ship carries enough supplies or meets a loadout threshold.

## Player-facing outcome

After selecting a charter, the player chooses an operating approach. That choice opens
particular solutions, changes likely complications, and creates a recognizable mission
doctrine with clear costs and risks.

## Dependencies

- Release 1 obligations may be created or conflicted by an approach.
- Release 2 officers and institutions may qualify the ship for specialist approaches.
- Existing provisioning, loadout gates, charter tags, event gates, phase weights, and
  objective accrual should be extended rather than duplicated.

## Scope

### Operating approaches

Support data-driven charter approaches that can alter:

- event-family and complication weights;
- available event outcomes;
- objective progress rules;
- field-repair limits or resource consumption;
- discovery and reward opportunities;
- faction approval and obligation exposure.

An approach must always include a benefit, a cost, and a recognizable failure mode.

### Initial approach set

Provide at least one meaningful approach decision for each objective family:

- mining;
- colonization/greening;
- exploration;
- rescue/preservation;
- diplomacy;
- salvage.

Candidate doctrines include redundant systems, rapid execution, open-handed mandate,
embedded specialists, local partnership, and heavy recovery rigging. These are starting
points, not mandatory names.

### Preparation dossier

The dossier must explain:

- what the approach enables;
- its entry requirements;
- its additional provisioning or loadout demands;
- which risk it increases;
- any obligation or faction commitment it creates.

Unavailable approaches remain visible with a plain-language reason.

### Voyage and debrief integration

- The selected approach appears on the active charter screen.
- Events acknowledge the approach where it materially changes the situation.
- Homecoming evaluates whether the doctrine helped, failed, or imposed an unexpected
  cost.
- The Chronicle records the approach so repeated charters form an operating history.

## Implementation shape

- Add a reusable approach schema to charter data.
- Copy the selected approach onto active-contract state at launch.
- Express event and outcome differences through existing tags and gates where possible.
- Keep balance constants in data.
- Make autoplay choose approaches through an explicit policy rather than relying on
  array position accidentally.

## Non-goals

- A universal technology tree.
- Dozens of near-identical percentage bonuses.
- Mid-voyage loadout shopping.
- A route-node map or tactical mission layer.
- Approaches that only modify starting resource totals.

## Validation

- Test requirements, selection, persistence, and save round-trips.
- Verify each modifier actually changes the intended voyage system.
- Add paired deterministic tests comparing the same charter with different approaches.
- Extend the balance matrix to include approaches and check for a universally dominant
  choice.
- Capture available, selected, and locked dossier states.
- Run `.\publish.ps1` successfully.

## Complete means

This release is complete when:

- [ ] Every charter launch requires or deliberately defaults an operating approach.
- [ ] All six objective families have at least one authored approach choice.
- [ ] Every approach changes at least two voyage systems beyond starting inventory.
- [ ] Requirements, benefits, costs, and risks are visible before launch.
- [ ] At least one approach uses an officer or institutional requirement.
- [ ] At least one approach creates or conflicts with an obligation.
- [ ] The active-contract screen, Homecoming, and Chronicle retain the selected doctrine.
- [ ] Balance testing finds no approach that is universally best.
- [ ] Deterministic tests, autoplay, save compatibility, and `publish.ps1` pass.
