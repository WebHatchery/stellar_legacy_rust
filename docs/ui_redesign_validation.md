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

## Increment 2: shared reading layer

DejaVu Sans is embedded locally through the toolkit font API. The font is reused
from the workspace's released auction_game assets; the upstream DejaVu licence
is bundled in assets/fonts/LICENSE-DejaVu.txt and registered in assets.zip.
Upstream licence: https://raw.githubusercontent.com/dejavu-fonts/dejavu-fonts/master/LICENSE
The existing title illustration is retained. No generated portrait asset or new
runtime dependency is introduced.

Gameplay uses opaque neutral reading panels, warm-white body text, gold identity,
muted positive status and labelled peach warnings. Title effects never overlay
gameplay, help, settings or welcome prose. Scanlines and flicker default off.
Reading-surface contrast, calculated using WCAG relative luminance: body 14.94:1,
secondary text 9.25:1, warning 9.41:1, panel boundary 4.29:1. These refer to the
opaque gameplay panel, not arbitrary text over title artwork.

All 54 existing captures were refreshed at actual 1280×720 and reviewed in
contact sheets; affected Bridge, event, People, Agenda, settings, preparation,
Homecoming, History, founding and ending screens were also inspected at full
size. The review corrected officer/action overlap, council stat collisions,
Agenda header collision, inherited-duty/trait overlap, milestone row spacing,
settings delegation/close overlap, preparation mandate overlap and founding
intro/selection overlap. Meter numbers now have their own opaque backing.

Dense secondary panels still use their existing smaller instrument sizes and
need the later selected-detail migration. Full portrait acceptance is not yet
claimed. Existing “narrow” filenames were found to capture 1280×720; this is a
verification gap, not evidence of portrait support.

Unit validation: 521 unit tests passed, one pre-existing ignored test, and one
integration test passed. No simulation or save schema was changed.

Increment 2 final publish.ps1 passed after the layout corrections, including packaged Windows and real-browser WebGL smoke tests. Every project Rust file remains below 800 physical lines. The catalog thumbnail was refreshed from the title capture.

## Increment 3: stable navigation

Bridge, Ship, People, Voyage and History keep identical positions in port and
underway. Ship holds Loadout, Systems and Agenda; Voyage holds Drydock/Market
in port and Contract underway. The action map above is now implemented.
Utilities holds Save game, Help, Display & sound and Return to menu. Project
counts and due-duty markers remain visible. Bridge warnings link to the related
work/system view (or the Market for docked energy shortages). The Instruments
section preserves the original maintenance actions and demographic readouts.

Tutorial, help and README navigation directions were updated in the same
increment. Two navigation tests cover every old screen and voyage restrictions.
Utilities and Instruments now have explicit capture scenes. Their desktop
captures were inspected along with the Bridge, Agenda and council overlays.

Increment 3 final publish.ps1 passed for Windows and WebGL, including packaged runtime checks. Navigation tests passed. Instruments was split by responsibility to keep source files comfortably within the 800-line limit.

## Increment 4: ship and work management

Systems now combines a selectable compartment rail, the selected subsystem's
operating controls, and the class-specific schematic. Tapping a subsystem on
the Bridge or underway ship opens the matching Systems detail. Condition,
staffing and running targeted work have explicit labels as well as colours.
Achieved states such as Sound, Mastered and Installed render as quiet badges.

Agenda uses compact selectable rows with a persistent selected detail. Running
work shows progress; the original queue, review, reorder, pause/resume and
cancellation intents and authored accounting are preserved. Critical air still
has a visible Stabilise air control and its cost before commitment.

Inspected Systems, Agenda, critical recovery, cancellation review and ark,
corvette and ring hull screenshots at 1280×720. Nine project simulation tests
passed. This does not yet establish the portrait touch acceptance routes.

Increment 4 publish.ps1 passed, including packaged Windows and WebGL runtime checks.

## Increment 5 — People and stable identities

People now separates Family, Officers, Factions, and Council. The family view connects the serving captain to the eligible heir, while the roster retains explicit heir selection. Officer rows show stable, name-derived silhouettes alongside role, age, remaining service, and skill. Faction emblems accompany approval and current relationships. The same captain silhouette appears in council decisions. These are original geometric drawings; no third-party portrait assets or runtime requests are required.

Reviewed the five baseline captures (Family, Officers, Factions, Council, event) at 1280×720. Corrected overlapping heir targets found in the first review and regenerated Family. Capture scene aliases were corrected so each new People image shows its intended page. Simulation and save schemas are unchanged.
Validation: `.\publish.ps1` passed: Windows and WebGL release builds, Preview deployment, packaged native rendering and real-browser WebGL smoke checks.

## Increment 6 — Voyage and living records

Charter summaries compare duration, reward, authored crisis exposure and launch promises, using stable destination emblems. Full selected briefings remain in preparation. History separates the dated decision timeline, obligations and milestones; selecting a deed opens every account at 18px with scrolling. Obligation rows now have readable body text and full-width history access. Homecoming leads with the actual outcome, generations and survivors, then provides scrollable accounting, captains and defining moments. No score or promise is dropped at the bottom of a fixed panel.

Reviewed charter/preparation, timeline, resolved obligations, obligation history, mission archive, and Homecoming outcome/accounting/captains/moments captures at 1280×720. The exposed route hazard is described as ordinary or more crisis-prone, matching its authored additive crisis weight rather than inventing a probability. Portrait acceptance remains pending.
Validation: two focused Homecoming tests passed. `.\publish.ps1` passed Windows/WebGL builds, Preview deployment and packaged native/browser smoke checks.
