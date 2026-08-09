# Stellar Legacy — Similar Games and Competitive Design Report

*Research date: 9 August 2026*

## Executive summary

**Stellar Legacy occupies a real niche rather than reproducing one existing game.** Its
closest shorthand is “*The Pale Beyond* or *Six Ages* aboard a generation ship, presented
through a diegetic command terminal.” *FTL* contributes the single-ship attrition and
pauseable crisis rhythm, while *Seedship* contributes the exact generation-ship setting
and text-first presentation. None of the researched games combines all of Stellar
Legacy's defining elements:

- one persistent ship-city rather than a fleet, empire, or disposable run;
- succession across several human generations;
- society-scale morale, unity, loyalty, adaptation, and cultural drift;
- event decisions whose effects can outlive the decision-maker;
- a repeated departure → attrition → homecoming → drydock/refit loop; and
- a dense, mostly diegetic terminal interface with no tactical combat layer.

The best primary references are:

1. **The Pale Beyond** — expedition survival, scarce stores, crew loyalty, and hard calls.
2. **Six Ages: Ride Like the Wind** — multi-generational story simulation and delayed
   consequence.
3. **Seedship** — generation-ship premise, push-your-luck travel, and text-led economy.
4. **FTL: Faster Than Light** — ship-system readability, pausable time pressure, event
   cadence, and run-level attrition.

The strongest positioning is therefore **generational expedition management**, not
“another FTL-like” and not a conventional grand-strategy space game.

## 1. Stellar Legacy baseline

This comparison uses the implemented game and its project documents, particularly the
[README](README.md), [GDD](gdd.md), [content-depth direction](content_depth.md), and
[event-system notes](event_design_notes.md).

### Core systems

- A persistent generation ship containing a simulated society.
- Month-precise real-time travel with Pause / 1× / 2× / 3× speeds.
- Credits, energy, minerals, food, influence, ship condition, and social condition.
- Six ship subsystems, crew posts, recruitment, training, and field/drydock repair.
- Dynasty leaders, heirs, ageing, succession, and extinction.
- Founding peoples and legacy factions whose relationships change over time.
- Long-term charters with preparation, underway phases, outcomes, rewards, and return.
- State-gated events, dilemmas, complications, visible immediate costs, and delayed
  consequences.
- A Chronicle and Heritage layer that carries renown between dynasties.

### Current graphic and interface style

The visual language is a **retro-futurist CRT command terminal**: near-black panels,
amber structural lines and labels, green nominal readings, red critical states,
condensed/monospace typography, meters, logs, and a schematic ship cutaway. It resembles
an operations console rather than a cinematic bridge.

Its strengths are unusually strong thematic consistency, cheap content scalability,
and clear status colour semantics. Its risks are visual sameness between screens,
high simultaneous information density, and emotional distance when population or
dynasty changes are represented only as text and numbers.

## 2. Comparison matrix

Scores describe usefulness as a reference for Stellar Legacy, not overall quality.

| Game | Setting fit | Systems fit | Structure fit | Presentation fit | Most useful lesson |
| --- | :---: | :---: | :---: | :---: | --- |
| [The Pale Beyond](https://store.steampowered.com/app/1266030/The_Pale_Beyond/) | 3/5 | 5/5 | 5/5 | 2/5 | Make an expedition's people and dwindling stores feel inseparable. |
| [Six Ages: Ride Like the Wind](https://store.steampowered.com/app/881420/Six_Ages_Ride_Like_the_Wind/) | 1/5 | 5/5 | 4/5 | 2/5 | Let hundreds of state-driven scenes accumulate into a generational history. |
| [Seedship](https://play.google.com/store/apps/details?id=com.johnayliff.seedship&hl=en) | 5/5 | 3/5 | 3/5 | 4/5 | Text and a few ship variables can carry a strong colony-ship fantasy. |
| [FTL: Faster Than Light](https://store.steampowered.com/app/212680/FTL_Faster_Than_Light/) | 4/5 | 4/5 | 3/5 | 3/5 | Keep ship damage, staffing, risk, and pause controls legible at a glance. |
| [Star Traders: Frontiers](https://store.steampowered.com/app/335620/Star_Traders_Frontiers/) | 4/5 | 4/5 | 2/5 | 3/5 | Specialised crew and modular ships create many viable operating identities. |
| [Citizen Sleeper 2: Starward Vector](https://store.steampowered.com/app/2442460/Citizen_Sleeper_2_Starward_Vector/) | 4/5 | 3/5 | 4/5 | 3/5 | Give contracts a preparation phase and use crew to humanise mechanical risk. |
| [Star Dynasties](https://store.steampowered.com/app/1194590/Star_Dynasties/) | 4/5 | 4/5 | 3/5 | 2/5 | Make an heir inherit relationships, obligations, and reputational damage. |
| [Crying Suns](https://store.steampowered.com/app/873940/Crying_Suns/) | 4/5 | 3/5 | 3/5 | 4/5 | A restrained pixel-art frame can give event-heavy space strategy a premium identity. |
| [Warsim: The Realm of Aslona](https://store.steampowered.com/app/659540/Warsim_The_Realm_of_Aslona/) | 1/5 | 3/5 | 2/5 | 5/5 | Text/ASCII presentation can support exceptional depth if navigation stays characterful. |
| [Frostpunk](https://store.steampowered.com/app/323190/Frostpunk/) | 1/5 | 4/5 | 3/5 | 1/5 | Social meters matter when policy choices visibly reshape the society behind them. |

## 3. Primary comparables

### The Pale Beyond — closest campaign-loop analogue

**Why it is similar.** The player leads an isolated expedition, manages limited food and
fuel, balances safety against morale, and makes choices that determine whether the group
continues to recognise their leadership. Its official description explicitly links
scarce resources, crew survival, morale, and consequential decisions. That is the same
dramatic engine as Stellar Legacy's “ship and people wear out together,” despite its
polar rather than space setting.

**Key systems.** Weekly expedition turns; provisioning; crew assignment; food and heat;
morale/loyalty; injuries; votes; story decisions; expedition milestones; survival and
multiple outcomes.

**Graphic style.** Hand-drawn 2D expedition scenes, muted icy colours, portrait-led
dialogue, physical notebooks/cards, and a highly authored historical-adventure feel.
The art constantly reminds the player that resources belong to particular people.

**Lesson for Stellar Legacy.** Preserve the drydock/underway split and make the
homecoming emotionally and mechanically decisive. Stellar Legacy need not copy the
illustration budget, but it can borrow the principle of attaching critical resource
changes to recognisable crew posts, peoples, or deck communities.

**Do not copy.** A fixed authored cast or almost entirely linear expedition would work
against multi-generation replayability.

### Six Ages: Ride Like the Wind — closest generational narrative-system analogue

**Why it is similar.** Six Ages describes itself as a storybook strategy game in which
hundreds of encounters form a multi-generational story of survival and alliances. Its
over 400 scenes are selected and resolved by simulation state, so narrative content and
resource management are not separate modes.

**Key systems.** Clan creation; a rotating council; food and herd economy; exploration;
diplomacy; war; ritual; family continuity; state-driven events; long-delayed callbacks;
and leaders whose strengths change what advice and options are available.

**Graphic style.** High-resolution hand-painted scene illustrations, character portraits,
parchment-like UI framing, and a storybook scene as the dominant focal point. Numbers
support the story instead of dominating the screen.

**Lesson for Stellar Legacy.** This is the best model for consequences that resurface
years later. Event chains should remember who promised what, which people benefited,
which faction objected, and whether the responsible leader is still alive. Council
advice can also express specialist competence without turning into an optimal-choice
tooltip.

**Do not copy.** Commissioning unique full-screen art for hundreds of events would break
Stellar Legacy's scalable, no-art-liability strength.

### Seedship — closest premise and minimal-presentation analogue

**Why it is similar.** Seedship casts the player as an AI carrying frozen colonists in
search of humanity's new home. Each generated planet creates a push-your-luck choice:
settle now or continue and risk further damage to the ship. Random travel events and
colony epilogues make each compact run different.

**Key systems.** Ship-system condition; colonist survival; planet scanning; generated
planet traits; travel events; settle/continue risk; colony scoring; and a historical log
of founded colonies.

**Graphic style.** Extremely minimal, text-based, mobile-first panels with a computer
terminal flavour. Its lack of spectacle concentrates attention on the decision and the
player's imagined colony.

**Lesson for Stellar Legacy.** The generation-ship concept remains legible with little
representational art. The danger is repetition: a text-first game needs large content
variety, sharp state gating, and sufficiently different outcomes. Stellar Legacy's event
families, phase pools, complications, and Chronicle directly address that problem.

**Do not copy.** Seedship resolves its fantasy in short, score-oriented runs. Stellar
Legacy's distinguishing value is living with a society after arrival/homecoming and
carrying damage, culture, and obligation forward.

### FTL: Faster Than Light — strongest ship-status and pacing reference

**Why it is similar.** FTL is a randomly generated single-ship survival journey with
crew, subsystems, upgrades, events, resource scarcity, pausable real time, and permadeath.
Its store description emphasises varied playthroughs, permanent defeat, and the ability
to pause to evaluate and issue orders.

**Key systems.** Ship rooms and power allocation; specialised crew; hull, fuel, missiles,
drones, and scrap; upgrades; generated routes and events; pausable real-time combat; and
run unlocks.

**Graphic style.** Clean top-down pixel/vector-like ship cutaways, compact status boxes,
colour-coded systems, readable crew tokens, space backdrops, and modal illustrated event
panels. The ship diagram doubles as the main play space and a diagnostic display.

**Lesson for Stellar Legacy.** The new schematic ship screen is a strong direction.
Subsystem condition, crew staffing, repairability, and cascading failure should remain
visible on one diagram. Pause must always feel like a legitimate planning tool, not a
failure to play quickly.

**Do not copy.** Tactical combat, route-node maps, and disposable 1–2 hour roguelike runs
would overpower the longer institutional story. Marketing Stellar Legacy primarily as
an FTL-like would also create the wrong combat expectation.

## 4. Secondary system comparables

### Star Traders: Frontiers

This is the best reference for **ship build breadth, specialist crew, careers, factions,
and a changing political economy**. It offers hundreds of ship upgrades, dozens of
hulls and jobs, individually tailored crew, and faction relationships in a procedural
galaxy. Its illustrated 2D UI mixes character art, ship profiles, maps, and dense lists.

Stellar Legacy should borrow the way a ship build expresses an operating doctrine, but
not the huge open-world breadth. Six highly consequential subsystems and a smaller set
of legible hull/loadout identities suit Stellar Legacy better than hundreds of parts.

### Citizen Sleeper 2: Starward Vector

Citizen Sleeper 2 combines a ship, recruited crew, and **prepared multi-cycle contracts**
whose execution must adapt to risk and bad rolls. Its interface resembles a stylish
tabletop board layered over 3D habitat maps, with bold comic-book character illustrations
and strong typography.

Its main lesson is contract dramaturgy: preparation, crew selection, mounting stress,
twists, and a decompression beat after the job. Stellar Legacy can make charter dossiers
and homecoming debriefs feel more distinct without adopting dice placement or a single
protagonist.

### Star Dynasties

Star Dynasties is a generational, turn-based space narrative about heirs, bloodlines,
vassals, alliances, obligations, honour, and reputation. Procedural characters pursue
their own goals, and consequences can affect the heir and house rather than only the
current ruler. Visually it uses a conventional grand-strategy star map, portraits,
relationship panels, and event windows.

It validates dynasty as a mechanical promise in science fiction. Stellar Legacy should
focus on **institutional inheritance**—shipboard posts, promises, faction grievances,
and cultural changes—rather than copying marriage politics or galaxy conquest.

## 5. Presentation references

### Crying Suns

Crying Suns combines a dark pixel-art science-fiction identity with procedural
exploration, over 300 story events, resource management, and tactical fleet battles.
Large character silhouettes, animated battleships, restrained palettes, and cinematic
event framing make a systems-heavy screen feel authored and premium.

Useful lesson: occasional high-impact visual anchors can carry many text events. A small
library of reusable faction seals, ship silhouettes, deck emblems, and event-family
backplates could add emotional texture to Stellar Legacy without requiring bespoke art
for 309+ events.

### Warsim: The Realm of Aslona

Warsim proves that a deliberately text-based strategy game with charming ASCII graphics
can support enormous procedural depth and strong player reception. It uses humour,
distinct locations, procedural imagery, and a sense of discovery to stop the format from
feeling like a spreadsheet.

Useful lesson: terminal presentation should be treated as an expressive art style, not
an excuse for every screen to share the same composition. Different subsystems can have
recognisable silhouettes while remaining entirely code-drawn.

### Frostpunk

Frostpunk is structurally much larger and visually unrelated, but it is a valuable
reference for **society survival**. Food, heat, infrastructure, hope, discontent, laws,
and faction demands combine so that expedient decisions visibly determine what kind of
community survives.

Useful lesson: cultural drift and legacy loyalty should unlock or close concrete rules,
behaviours, and future event branches. A social meter is most meaningful when crossing a
threshold changes how the ship is governed, not merely the odds on a later roll.

## 6. Cross-game system findings

### 6.1 The successful common loop is pressure → choice → visible cost → later callback

Every strong comparable makes scarcity a narrative author. The player first sees a
resource, relationship, or time pressure; chooses an imperfect response; pays an
immediate, understandable cost; and later encounters a callback. Stellar Legacy already
has all four pieces, but its signature depends on the **later callback crossing a
succession boundary**.

Recommended consequence record:

| Recorded fact | Near-term use | Generational use |
| --- | --- | --- |
| Leader/council that made the promise | Advice and event text | Descendant inherits credit or blame |
| Peoples/faction affected | Immediate loyalty shift | Changed event pool or secession pressure |
| Resource or subsystem sacrificed | Operational penalty | Maintenance debt or design tradition |
| Principle chosen | Current outcome modifier | Charter, doctrine, or cultural norm |
| Public/private nature | Morale and stability | Chronicle version of the event |

### 6.2 Crew should connect population-scale numbers to human stakes

The Pale Beyond, FTL, Star Traders, Six Ages, and Citizen Sleeper 2 all use named people
as the interface between systems and story. Stellar Legacy appropriately simulates most
citizens in aggregate, but its named dynasty and ship-post holders should appear in
event advice, repair outcomes, homecoming losses, and Chronicle entries more often. That
provides human texture without simulating a thousand individuals.

### 6.3 Preparation must alter the mission, not only starting totals

The best contract/expedition games let preparation open approaches and prevent specific
failure modes. Provisioning in Stellar Legacy should affect event options, complication
weights, field-repair ceilings, and which discoveries can be exploited—not just add more
food or spare parts.

### 6.4 A persistent campaign needs recovery stories as well as failure spirals

FTL and Seedship can end quickly because they are run-based. Stellar Legacy asks the
player to keep a damaged society for multiple missions. Pyrrhic homecomings, faction
reconciliation, obsolete-ship retirement, new leadership, and hard-won institutional
knowledge therefore need to create credible recovery arcs.

## 7. Graphic-style findings and recommendations

### What is already distinctive

- The amber/green CRT palette is immediately recognisable and consistent with the fiction.
- The ship schematic gives Stellar Legacy a visual object that belongs to this game,
  rather than to generic dashboard software.
- Status colours and labelled meters support rapid diagnosis.
- The interface scales to a large amount of authored event content without an art bottleneck.

### Highest-value visual improvements

1. **Give each major screen a distinct silhouette.** Keep the terminal skin, but let the
   ship be schematic-led, the dynasty be timeline/tree-led, contracts be dossier-led,
   factions be seal/relationship-led, and the Chronicle be log/timeline-led.
2. **Create a small reusable symbolic art kit.** Code-drawn faction seals, peoples'
   emblems, subsystem glyphs, ship-class silhouettes, and event-family marks would add
   identity at low production cost.
3. **Use event framing to show category and time horizon.** A hull breach, faction
   dispute, first contact, and legacy moment should be recognisable before the body text
   is read. Colour must remain a redundant cue, not the only cue.
4. **Make generational change visible.** A horizontal lineage, captain plaques, changing
   ship silhouette/wear, and inherited-promise markers would communicate the unique
   premise better than another resource panel.
5. **Reserve strong contrast and motion for change.** Most of the terminal can remain
   subdued; new damage, threshold crossings, succession, and homecoming should receive
   the brief visual emphasis.

### Styles that would weaken the project

- Full bespoke character/event illustration at Six Ages or The Pale Beyond scale.
- A conventional star map that implies an open-world 4X game.
- Tactical ship combat visuals that reposition the game as an FTL clone.
- Decorative CRT effects that reduce already dense text readability.
- More simultaneous dashboard metrics without stronger grouping and progressive detail.

## 8. Positioning and design opportunities

### Recommended public description

> A generational expedition strategy game. Lead one living starship across decades,
> survive the promises and failures of successive captains, and bring a changed people
> home to refit before the next voyage.

### Strong comparison language

- “The expedition pressure of *The Pale Beyond*, stretched across generations.”
- “The multi-generational consequence of *Six Ages*, aboard one persistent starship.”
- “A generation-ship strategy game for players who like FTL's ship triage more than its
  combat.”

These are design/positioning formulations, not suggested store-page quotations or
endorsements.

### Best opportunities to differentiate further

1. **Promises that outlive captains.** Make explicit obligations a first-class system,
   with owners, beneficiaries, due dates, public memory, and inherited consequences.
2. **The ship as cultural geography.** Decks and subsystems can develop identities,
   loyalties, damage histories, and competing customs over decades.
3. **Homecoming as reckoning.** Compare departure and return not just numerically but in
   terms of deaths, births, changed peoples, fulfilled promises, and irreversible norms.
4. **Institutional knowledge.** A dynasty may lose a gifted engineer, but the ship can
   preserve procedures, schools, or traditions—turning succession from pure stat loss
   into a strategic choice about what knowledge survives.
5. **Consequences with competing interpretations.** The official log, faction memory,
   and dynasty account need not describe the same decision in the same way.

## 9. Bottom line

Stellar Legacy should not chase the breadth of Star Traders, the combat of FTL/Crying
Suns, the empire layer of Star Dynasties, or the illustration volume of Six Ages. Its
advantage is the combination those games leave open: **a deeply simulated institution
travelling inside one ageing ship, where the timescale of strategy is longer than any
individual life**.

The most valuable next design emphasis is therefore not another generic space system.
It is stronger connective tissue between voyages, named crew, peoples, promises,
succession, and the Chronicle—supported by a more varied but still terminal-native visual
language.

## Sources

Primary product/developer pages consulted:

- [The Pale Beyond — Steam](https://store.steampowered.com/app/1266030/The_Pale_Beyond/)
- [Six Ages: Ride Like the Wind — Steam](https://store.steampowered.com/app/881420/Six_Ages_Ride_Like_the_Wind/)
- [Seedship — Google Play](https://play.google.com/store/apps/details?id=com.johnayliff.seedship&hl=en)
- [Space Goblin Games — developer page](https://spacegoblingames.itch.io/)
- [FTL: Faster Than Light — Steam](https://store.steampowered.com/app/212680/FTL_Faster_Than_Light/)
- [Star Traders: Frontiers — Steam](https://store.steampowered.com/app/335620/Star_Traders_Frontiers/)
- [Citizen Sleeper 2: Starward Vector — Steam](https://store.steampowered.com/app/2442460/Citizen_Sleeper_2_Starward_Vector/)
- [Star Dynasties — Steam](https://store.steampowered.com/app/1194590/Star_Dynasties/)
- [Crying Suns — Steam](https://store.steampowered.com/app/873940/Crying_Suns/)
- [Warsim: The Realm of Aslona — Steam](https://store.steampowered.com/app/659540/Warsim_The_Realm_of_Aslona/)
- [Frostpunk — Steam](https://store.steampowered.com/app/323190/Frostpunk/)

