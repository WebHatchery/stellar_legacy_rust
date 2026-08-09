# Release 2 — Officers and Schools

## Goal

Turn named officers into the human face of the simulation and make expertise something
the ship deliberately preserves across generations.

The loss of a gifted officer should matter, but a well-built institution should allow
their craft to survive them.

## Player-facing outcome

Council decisions include concise, state-aware advice from relevant named officers.
Between voyages, the player can invest in apprenticeships, schools, and archives that
preserve particular posts or subsystem knowledge through retirement, death, and
succession.

## Dependencies

- Release 1's responsible-office model should be reused when officers advise on or
  inherit responsibility for obligations.
- Existing crew posts, subsystem condition, subsystem knowledge, faction approval, and
  continuous mortality remain authoritative.

## Scope

### Named council advice

- Events may name one or more relevant officer archetypes.
- The modal shows advice from up to two serving officers.
- Advice varies with skill, faction affiliation, subsystem state, ship reputation, and
  active obligations.
- Advice communicates priorities and risks; it does not label an optimal outcome.
- Vacant posts produce an explicit absence rather than anonymous replacement advice.

### Knowledge continuity

Add a small set of institutional programs:

- designate an apprentice for a post;
- establish a school attached to a subsystem;
- compile an emergency procedure archive;
- preserve a retiring or deceased expert's methods;
- grant an aboard faction custodianship of a discipline.

Programs require real costs and produce distinct tradeoffs. Faction custody, for
example, may protect knowledge while increasing that people's political leverage.

### Turnover consequences

- An officer who leaves without a continuity plan can reduce related knowledge or close
  advanced event outcomes.
- A prepared apprentice retains part of the predecessor's skill.
- A school slows knowledge decay but requires upkeep or periodic recommitment.
- The Chronicle and Homecoming credit the institution or identify the expertise lost.

### Content exemplars

- Add at least one advice set for every officer archetype.
- Add at least one succession-of-craft event for engineering, medicine, agriculture,
  security, science, and education/culture.
- Include disagreement between competent officers whose values differ.

## Implementation shape

- Keep advice prose in data, with reusable gates and substitutions for officer name,
  post, skill band, and faction.
- Keep skill transfer and knowledge preservation deterministic.
- Avoid simulating full family trees for officers; use the existing named roster and
  explicit apprentice relationships.
- Expose institutional state through a focused UI rather than adding more dashboard
  meters.

## Non-goals

- Fully autonomous officer personalities.
- Tactical crew movement or combat.
- Exact success-probability tooltips.
- A large individual relationship simulator.
- Removing the population-scale abstraction of ordinary citizens.

## Validation

- Test advice selection for skilled, unskilled, faction-aligned, and vacant posts.
- Test apprenticeship transfer on retirement and death.
- Test school upkeep, knowledge-decay reduction, and loss when support ends.
- Verify named officers appear correctly after save/load and succession.
- Capture event advice, institutional management, and Homecoming turnover summaries.
- Run `.\publish.ps1` successfully.

## Complete means

This release is complete when:

- [ ] Every decision domain can surface advice from a relevant named officer.
- [ ] Advice reacts to live game state and never directly identifies a best choice.
- [ ] Vacant posts are mechanically and visibly different from filled posts.
- [ ] Apprenticeships and subsystem schools are player-controlled, persistent systems.
- [ ] Officer loss can damage expertise, and preparation can materially reduce that
      damage.
- [ ] Every subsystem has at least one authored knowledge-succession situation.
- [ ] Homecoming identifies important appointments, losses, and preserved institutions.
- [ ] Save compatibility, deterministic tests, autoplay, and `publish.ps1` all pass.
