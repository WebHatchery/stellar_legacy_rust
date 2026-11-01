# Critical review — Steam readiness (2 November 2026)

Build reviewed: 0.2.1 (`master`), Windows debug build. **Target display:
1920×1080**, the most common Steam desktop resolution, at the default 100 % UI
scale and 100 % text size.

**Question:** is this ready to sell on Steam at full price?
**Answer: not yet.** The systems are deep and the tutorial works. At 1080p,
though, the presentation looks unfinished:

- the interface draws at a fixed pixel size, so on a 1080p monitor the text is
  small and the panels are mostly empty;
- there is no music, and the visual style changes between screens;
- the in-game screens are flat amber panels, so store screenshots would look
  like a spreadsheet.

Most of the work left is scaling, presentation, feedback and PC conventions,
not new systems.

## How this review was done (and its limits)

- I captured 25 scenes at 1920×1080 with `scripts/capture_ui.ps1`, directly
  into `docs/verification/ui_<scene>.png`. The images named below are those
  files.
- I checked the content and assets: `assets/`, `assets/events/*.json`,
  `src/audio.rs`, `src/achievements.rs`, and the release docs.
- I did not play a full campaign. The findings about pacing and repetition
  (§4) are risks to measure, not things I observed.
- Capture scenes put the game into staged states, so some odd numbers may come
  from the staging. These are marked **(verify)**.
- This review does not repeat `TODO.md` UI-01…UI-08. Where an item overlaps,
  it cites the task and says what goes further.

## 1. What already works (keep it)

- **The premise is strong and easy to market.** "You are the ship's mind; the
  captains die, you remain" works as a hook. The welcome text and store copy say
  it well.
- **The systems are deep.** There are 343 authored events across 11 themes,
  23 charters, 3 legacies, 6 founding peoples, obligations that outlive
  captains, Agenda projects, subsystem knowledge decay, Heritage carry-over and
  Homecoming recovery choices.
- **Decisions have integrity.** Events show you choices, then KNOWN effects,
  then an explicit Commit, with officer advice and a fallback timer
  (`ui_event.png`). Obligations that conflict with a new charter are named
  before launch (`ui_prep.png`).
- **The Chronicle's multi-voice accounts are the best writing in the game**
  (`ui_chronicle.png`). One deed is told as a fact, a command log entry, a
  house's view and a faction's view. Build more of the game around this.
- **The main-menu key art is store-grade** (`ui_menu.png`).
- **Some screens use the 1080p width well.** The Ship Loadout catalogue
  (`ui_ship.png`) shows all 18 components in three readable columns, and
  Drydock (`ui_drydock.png`) fits a two-column charter grid. Both are better
  than at smaller sizes.

## 2. Release blockers

### B0 — The UI does not scale to 1080p

This is the single biggest 1080p problem. At the default settings the
interface draws at the same pixel size as at 1280×720. Larger panels simply
stretch to fill the extra space.

- **Text is too small for a 24″ monitor viewed from a desk.**
  - The resource and status header is roughly 11 px.
  - Secondary lines are about 10 px. These include the modifier rows, trait
    lines, "LOCKED · PROVE WRIT · Requires…", the Route & Mission list and the
    subsystem culture block.
  - The resource header ("Credits 10000 Energy 5000 …") is the information read
    most often, and it is one of the smallest strings on screen.
- **Most panels are left largely empty.** Content stays at the top while the
  frame stretches:

  | Capture | Empty area |
  | --- | --- |
  | `ui_gameplay.png` | The lower half: the Ship & people and Recent developments panels are about 90 % empty |
  | `ui_event.png` | About 70 % of the decision modal, including the speaker column under the text |
  | `ui_prep.png` | Below the provisioning lines; LAUNCH sits ~450 px below the content it acts on |
  | `ui_contract_active.png` | The left column below Success Metrics; the middle of Route & Mission |
  | `ui_crew.png` | About 60 % of the roster area. The NAME HEIR buttons sit 1,400 px from the names they act on |
  | `ui_ship_underway.png` | The thin schematic in the middle of a ~900 px tall Ship Layout frame, with long empty leader lines |
  | `ui_subsystems.png` | The schematic floats in the lower half, with empty leader lines above and below it |

- **Fixed-size dialogs float in the centre.**
  - Founding (`ui_heritage.png`) is a fixed box about 860 px wide using about
    45 % of the screen, so its blurbs clip even with space all around.
  - Welcome (`ui_welcome.png`) and Help are also fixed-size boxes over a grey
    field.

What to do:
- Set the default UI scale from the window height. 1080 / 720 gives 150 %, and
  the setting already exists in Settings.
- Separately, define how layouts use extra space at 1080p: wider columns,
  more items per row, side detail panes. Stretching empty frames is not enough.
- Anchor actions next to their content.

This extends UI-01: make 1920×1080 the primary supported canvas, and treat the
other sizes as secondary.

### B1 — There is no music, and the soundscape is a placeholder

`src/audio.rs` synthesises 8 cues in code. There are no audio assets. In a
game about centuries passing, with sessions that last hours, silence plus synth
blips is the strongest "prototype" signal after B0.

- Commission or license an ambient score: 3–5 loops for port, underway calm,
  crisis, succession/elegy and Homecoming. Add authored SFX for commit,
  succession, death, milestone, launch and arrival.
- Split volume into Master, Music and SFX, and use sliders instead of
  Smaller/Reset/Larger buttons (`ui_settings.png`).

### B2 — Three visual identities, and illustration on only one screen

- The main menu is painted key art. Founding (`ui_heritage.png`) is a CRT
  bezel with a different logo treatment. Everything in play is a flat amber
  panel.
- Events show one generic two-tone silhouette for every speaker, and at 1080p
  it is about 50 px tall in a 1,900 px modal (`ui_event.png`).
- The ship is drawn three different ways:
  - as a Bridge schematic (`ui_gameplay.png`);
  - as a thin 600 px outline with "AGR 0" labels (`ui_ship.png`);
  - as a large layout with a legend (`ui_ship_underway.png`).

  None of them looks like the ship on the title screen.

What to do:
- Pick one art direction and apply it from the menu to Homecoming.
- Draw one hero ship view that changes visibly with hull type, damage and the
  modules installed. At 1080p there is room for it on the Bridge.
- Add illustrated vignettes per event theme (11 themes means about 11–20
  images). The empty event space at 1080p is where they belong.
- Generate captain portraits procedurally, with ageing.

### B3 — PC conventions are missing because the UI is touch-first

- I found no fullscreen, windowed or resolution option in Settings or the game
  code. A 1080p Steam player expects fullscreen and borderless at a minimum.
- Help is titled **TOUCH CONTROLS** and the copy says "Tap" everywhere
  (`ui_help.png`, `ui_welcome.png`, "Tap a compartment to inspect" in
  `ui_subsystems.png`). On a Windows build, write "Click" instead.
- There are no hover states or hover tooltips. On PC, hover is the main way
  people learn a strategy game, and §3.4 needs it.
- Space pauses play (`src/game.rs`), but the game never tells the player.
  There is no shortcut for 1/2/3 speed, Esc to close or tab cycling, and hints
  such as BEGIN VOYAGE [ENTER] and BACK [ESC] appear on one screen only.
- The achievements in `src/achievements.rs` are local-only, and there is no
  Steam Cloud. Both are close to expected for a strategy game with long saves.

### B4 — Text clips and wraps even at 1920×1080

Every one of these is visible with plenty of free space nearby:

| Capture | Defect |
| --- | --- |
| `ui_ship.png` | Ion Drive blurb cut mid-sentence: "Scavenges enough reaction mass" |
| `ui_heritage.png` | Legacy blurbs cut ("until the mission") or touching the card border; WHAT THIS CHANGES cut at "let tradition rot too far" |
| `ui_prep.png` | "+315 PARTS · 3780 CR" and "REVIEW MANDATE" wrap inside their buttons |
| `ui_contract_active.png` | "REVIEW MANDATE" wraps; the "PROGRESS" label overlaps its own bar fill |
| `ui_crew.png` | "NAME HEIR" wraps inside a narrow button on a 1,900 px row |
| `ui_ship_underway.png` | "FIELD INSTALL — Solar Sail Array" wraps |
| `ui_subsystems.png` | "Closed Ecology" crowds the panel header |
| all schematics | The module icon collides with its label ("ΩEDU", "♡SEC") |
| `ui_chronicle.png` | Possessive casing bug: "Boro Chartwright'S HOUSE" |

- Make the capture audit fail on clipped or wrapped single-line labels.
- Size buttons to their text.

### B5 — Controls appear when they do nothing

- PAUSE/[1x]/2x/3x shows on Homecoming (`ui_debrief.png`) and Game Over
  (`ui_gameover.png`).
- It is also fully shown in port, where time is frozen.

This overlaps UI-03. Treat it as a blocker, because these are the screens that
end a run.

### B6 — Raw simulation numbers are shown to the player

- "Final score 0.83", "1.00/0.85 (w 21%)", "crisis weight +0.12", "DECAY
  ×1.08" and "Custodian: KIND (perceived 100%)".
- Whole lines of zero modifiers are printed:
  "WORK +0% · EVENTS +0% · FUEL +0%" and "ANNUAL PEOPLE +0.0% morale · +0.0%
  unity · PRESERVE LOSS +0%".
  - At 1080p, Homecoming prints them in large body type as part of the
    narrative record (`ui_debrief.png`). The same record repeats the recovery
    review text that the panel below already shows.
- Hide zero modifiers. Show outcomes as grades or words, and keep exact values
  in hover tooltips.
- **Food 424291 (verify)** in `ui_contract_active.png`. If real play can reach
  this, the food cap is broken. If it is staging, keep it out of store
  screenshots.
- **"Y41 after 40 yr" vs header Y000 (verify)** in `ui_drydock.png`.

## 3. UI composition and readability at 1080p

### 3.1 No visual hierarchy

Every region is the same bordered amber rectangle, and every action is the same
bordered amber button.

- LAUNCH, CANCEL MISSION / RETURN HOME, DELETE SAVE and "Inspect ship" all
  carry the same weight.
- DELETE SAVE sits next to BACK on Founding (`ui_heritage.png`).
- On Drydock, locked charters dim their titles but keep a full-strength
  "LOCKED · RENOWN 100" button (`ui_drydock.png`), so the locked state looks
  like an action.

Define 3 button tiers (primary, secondary, quiet) plus a destructive style,
with one primary action per screen. This extends UI-02/UI-04.

### 3.2 Type

- The game ships with DejaVu Sans only, which reads as a system default. Pair a
  display face for headings with a legible UI face.
- At 1080p the size spread runs from a ~10 px meta line to a 22 px title.
  Once B0 scaling is in, set a minimum body size of about 16 px at 1080p.

### 3.3 Terminal decoration reads as placeholder

The `[ ]` checkboxes, `[1x]`, `> NEW GAME`, `// … //` around titles,
`[LEADER]` and ALL-CAPS label soup look like debug UI. Either commit to a
diegetic Custodian terminal (consistent type, animated text, sound) or replace
them with real controls.

### 3.4 Jargon load

A new player meets Charter, Writ, Approach, Posture, Mandate, Obligation,
Agenda, Escrow, Knowledge, Institution, Custodianship, Heritage, Renown and
Tradition. Needed:

- a hover glossary on every term;
- an in-game Codex under Help;
- fewer synonyms (Charter / Writ / Contract).

### 3.5 Use the space at 1080p instead of stretching frames

| Screen | Use the space for |
| --- | --- |
| Bridge | Hero ship and route view on top; current concern beside it; a short change feed instead of a permanent log (UI-02) |
| Event | Illustration plus speaker portrait on the left; choices with consequences on the right; officer advice inline, not behind a button |
| Prep | Readiness checklist and LAUNCH together at the top (UI-04); the briefing beside them |
| People | A family tree or generations timeline with portraits, not a five-row list |
| Contract | A route map with phase and milestone markers in place of the empty column |

### 3.6 Navigation depth

- Ship → Loadout → LOADOUT/MODULES makes three stacked tab rows
  (`ui_ship.png`).
- At 1080p the tab bar uses about 55 % of the width and Utilities sits alone
  at the far right.
- "Agenda (0)" lives under Ship, even though it is the main thing to do
  mid-voyage.

### 3.7 Modals

- The Welcome dialog renders over Founding, and the screen underneath shows
  through (`ui_welcome.png`).
- Its section headings have no top spacing, and it ends in an empty band.

## 4. Game feel and design gaps

1. **The journey is never shown.** A 340-year voyage appears as a progress
   bar. At 1080p there is room for:
   - a starfield and a route map with the ship's position;
   - destination art;
   - an arrival moment.

   This is the cheapest and biggest "this is a real game" upgrade.
2. **Feedback is minimal.** Add:
   - animated resource deltas;
   - transitions between screens;
   - a beat when a decision is committed;
   - births, deaths and milestones that register visibly.
3. **Succession plays like a spreadsheet.** A captain's death or retirement
   should be a scene: portrait, epitaph, the heir accepting, a Chronicle line
   and a music cue (`ui_crew.png`).
4. **Homecoming is an accounting screen** (`ui_debrief.png`). It needs an
   arrival illustration, a montage of generations and captains, a clear grade,
   and then the recovery choice.
5. **Game Over has no weight** (`ui_gameover.png`). It is a small stats box on
   a black 1080p screen.
6. **Meta-progression is thin for Steam.**
   - Heritage is a numeric head start, and there are only 3 legacies.
   - Add unlocks for runs 2–5: hulls, legacies, founding peoples, charters,
     scenarios and Chronicle-gated events.
   - Add difficulty or ironman settings and show the run seed on screen.
7. **Repetition risk (measure it).** Count events per run, and repeat rates
   within a run and across runs 1–5, over a seeded batch.
8. **Session pacing (measure it).** Drydock reports "played 38m" for one
   charter. Consider a "run until something needs me" speed.
9. **The touch-first paradigm has a cost.** On a 1080p desktop it means more
   clicks and less information density than players of this genre expect. Add
   a real desktop layer: hover, right-click, shortcuts and denser panels.

## 5. Store-page readiness

- **Screenshots:** store screenshots are 1920×1080. Today all five would be
  mostly empty amber panels with small text (B0, B2).
- **Trailer:** there are no music, motion or ship-travel visuals to cut one
  from.
- **Capsules:** the menu key art covers them; the library hero and logo need
  clean-plate versions.
- **Identity:** the developer/publisher legal names are still unresolved
  (`docs/release/STORE_COPY.md`).

## 6. Recommended order of work

Each item is a separate, committable slice. Verify each at 1920×1080.

**Tier 0 — required before a Steam page goes live**
1. B0: resolution-relative default UI scale, plus a 1080p layout pass that
   anchors actions to their content and uses the extra space.
2. B4: fix clipping and wrapping, size buttons to their text, and make the
   capture audit fail on clipped labels.
3. B5: hide time controls where they do nothing (finish UI-03).
4. B6: hide zero modifiers, use grades and words, and resolve the
   **(verify)** numbers.
5. B3: fullscreen/borderless and resolution options, "Click" wording,
   shortcuts, and hover tooltips with a glossary.
6. B1: music and authored SFX, plus separate volume sliders.
7. B2: one art direction, a hero ship view, event vignettes and portraits.

**Tier 1 — needed to earn good reviews**
8. Show the voyage moving: route map, starfield, arrival (§4.1).
9. Feedback animations and transitions (§4.2).
10. Succession, Homecoming and Game Over as scenes (§4.3–4.5).
11. Button tiers, a destructive style and type pairing (§3.1–3.2).
12. Steam achievements and Cloud through Steamworks.

**Tier 2 — what keeps players past run 3**
13. Unlocks, legacies and scenarios (§4.6).
14. Measure repetition and pacing; add event variants or content (§4.7–4.8).

## 7. Acceptance check before a Steam release

Have someone who has never seen the game play the first 30 minutes on a 1080p
monitor at normal desk distance, with sound on and no explanation.
Release-ready means:

- they never lean in to read text;
- they never ask "is this finished?";
- they can explain Posture and Approach without opening Help;
- they react to the first captain's death.

Record the session with the release acceptance in
`docs/release/QA_AND_OPERATIONS.md`.
