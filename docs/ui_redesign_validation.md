# UI redesign validation

## Increment 1: operational prototypes (2026-09-07)

The Bridge and council layout are implemented in the game, using deterministic
capture scenes and existing simulation state. These are the first reviewable
prototypes, not acceptance of the complete M0–M6 programme.

The Bridge leads with the charter and voyage phase, gives the ship most of the
space, and places a low-energy warning above healthy status. The existing
readiness forecast supplies other concerns; no predicted event is invented.
Full instruments and maintenance remain available. Council choices have a
separate selection and commitment action, with the selected description and
known effects fully scrollable. Advice is optional; captain fallback and global
time controls remain visible. This layout does not change countdown semantics.

Initial captures: gameplay (port), dashboard_risk (energy shortage), event
(The Last Engineer), event_obligation_unaffordable (insufficient stores).
Actual PNG dimensions: 1280 × 720. Reviewed by the implementing agent.
Initial review found overlapping schematic captions at 168px diagram height;
the diagram was enlarged before the final capture. Body contrast and the
condensed font still require the shared reading-layer increment.

Narrow and portrait support, every-screen review, interaction routes, and fresh
human acceptance remain outstanding. Screenshot capture alone does not verify
click behavior or human comprehension. No fresh human reviewer has participated.

## Action inventory and destination map

| Current screen / action family | Destination | Required interaction retained |
| --- | --- | --- |
| Dashboard | Bridge | Voyage objective, warnings, maintenance, demographic detail |
| Ship / loadout / modules | Ship | Buy component, install salvage, commission hull, inspect modules |
| Subsystems | Ship | Repair, upgrade, fitting, train knowledge, school, archive, custody |
| Agenda | Ship | Queue, review, pause, resume, reorder, cancel, emergency recovery |
| Crew & Dynasty | People | Recruit/train officer, apprentice, heir, factions, delegation |
| Drydock / selected charter | Voyage | Select/cancel charter, briefing, posture review |
| Preparation | Voyage | Fuel, parts, provisions review, launch |
| Active contract | Voyage | Progress, milestones, return-home confirmation |
| Market | Voyage (port only) | Buy/sell each traded resource |
| Chronicle | History | Current decisions, mission archive, obligations, full duty history |
| Homecoming | Blocking outcome | File report, captain chain, voyage log |
| Event / legacy dilemma | Blocking decision | Eligible choices, affordability, effects/odds, fallback |
| Authority review | Blocking decision | Human authority options and attribution |
| Survival warning | Blocking recovery | Stabilise air, review recovery, resume |
| Terminal / dynasty extinction | Blocking ending | History, retirement/menu |
| Tutorial / welcome | Guidance | Exact touch instruction, continue/skip |
| Save / menu / help / display | Utilities | Persist/reload, settings, help, return to title |
| Pause / resume / speed | Global | Reachable above blocking overlays |

Capture scene inventory is maintained in `src/game/capture_scenes.rs` and its
Agenda and blueprint children; the existing image inventory is
`docs/verification/ui_*.png`. Additional scenes must be documented with their
actual framebuffer size, not inferred from a filename containing “narrow”.

Direction: retain the procedural class-specific ship and gold headings. Use
warm-white prose, 18px decision text, 26px headings, 44–48px actions and 14–26px
section spacing. Select-and-commit council choices replace simultaneous tall
cards so their full consequences can be read without reducing text size.

Increment 1 validation: publish.ps1 passed (Windows and WebGL release packaging, preview deployment, packaged Windows render and real-browser WebGL smoke). Four refreshed 1280x720 captures reviewed; schematic caption collision corrected. Full responsive and human acceptance remain pending.
