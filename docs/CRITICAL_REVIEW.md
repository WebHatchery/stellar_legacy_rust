# Critical review — Steam readiness (2 November 2026)

Build reviewed: 0.2.1 (`master` @ `25f1ae7`), Windows debug build at 1280×720.

**Question:** is this ready to sell on Steam at full price?
**Answer: not yet.** The systems are deep and the tutorial works. The problem is
the presentation: almost nothing in it shows that the game was finished on
purpose. A Steam buyer judges a game by its capsule, five screenshots, the
trailer and the first ten minutes. Right now each of those would show a
well-built spreadsheet in one colour. Most of the work left is presentation,
feedback and PC conventions, not new systems.

## How this review was done (and its limits)

- I took fresh captures of 26 scenes at 1280×720 with `scripts/capture_ui.ps1`
  (output went to the ignored `target/review_caps/`). The `ui_<scene>.png` names
  below match those captures and the files in `docs/verification/`.
- I checked the content and assets: `assets/`, `assets/events/*.json`,
  `src/audio.rs`, `src/achievements.rs`, and the release docs.
- I did not play a full campaign for this review. The findings about pacing and
  repetition (§4) are risks to measure, not things I observed.
- Capture scenes put the game into staged states. Some numbers that look odd may
  come from the staging and not from real play. These are marked **(verify)**.
- This review does not repeat `TODO.md` UI-01…UI-08. Where an item overlaps,
  it cites that task and says what goes further.

## 1. What already works (keep it)

- **The premise is strong and easy to market.** "You are the ship's mind; the
  captains die, you remain" works as a hook. The welcome text and store copy say
  it well.
- **The systems are deep.** There are 343 authored events across 11 themes,
  23 charters, 3 legacies, 6 founding peoples, obligations that outlive
  captains, Agenda projects, subsystem knowledge decay, Heritage carry-over and
  Homecoming recovery choices. Few games in this niche have this much.
- **Decisions have integrity.** Events show you choices, then KNOWN effects,
  then an explicit Commit, with officer advice and a fallback timer
  (`ui_event.png`). Obligations that conflict with a new charter are named
  before launch (`ui_prep.png`).
- **The Chronicle's multi-voice accounts are the best writing in the game**
  (`ui_chronicle.png`). One deed is told as a fact, a command log entry, a
  house's view and a faction's view. Build more of the game around this.
- **The main-menu key art is store-grade** (`ui_menu.png`).

## 2. Release blockers

These would get the game marked "unfinished" in reviews, or they break what PC
players expect by default.

### B1 — There is no music, and the soundscape is a placeholder

`src/audio.rs` synthesises 8 cues in code (ambience, button, council,
resolution, phase, succession, homecoming, game over). There are no audio
assets. A game whose sessions last hours and which is about centuries passing
leans on its score more than most. Silence plus synth blips is the strongest
single "prototype" signal.

- Commission or license an ambient score: 3–5 loops for port, underway calm,
  crisis, succession/elegy and Homecoming. Add authored SFX for commit,
  succession, death, milestone, launch and arrival.
- Split the volume setting into Master, Music and SFX, and use sliders instead
  of Smaller/Reset/Larger buttons (`ui_settings.png`).

### B2 — Three visual identities, and illustration on only one screen

- The main menu is painted key art. Founding (`ui_heritage.png`) is a CRT bezel
  with scanlines and a different logo treatment. Everything in play is a flat
  amber panel. Going from the menu to Founding feels like opening a different
  product.
- Events, which are the emotional core, show one generic two-tone silhouette
  for every speaker (`ui_event.png`). 343 events share one picture.
- The ship, the game's protagonist, is drawn three different ways: as a Bridge
  schematic (`ui_gameplay.png`), as a thin outline with "AGR 0" labels
  (`ui_ship.png`), and as a larger layout with a legend (`ui_ship_underway.png`).
  None of them looks like the ship on the title screen.

What to do:
- Pick one art direction and apply it from the menu to Homecoming.
- Draw one hero ship view that changes visibly with hull type, damage and the
  modules installed.
- Add illustrated vignettes per event theme (11 themes means about 11–20
  images, not 343).
- Generate captain portraits procedurally, with ageing, from the dynasty data.

### B3 — PC conventions are missing because the UI is touch-first

- I found no fullscreen, windowed or resolution option in Settings or the game
  code. The window is fixed at logical size. Steam players expect fullscreen
  and borderless at a minimum.
- Help is titled **TOUCH CONTROLS** and the copy everywhere says "Tap"
  (`ui_help.png`, README). On a Windows Steam build this reads as a mobile port.
  Write the wording per platform: "Click" on desktop.
- There are no hover states or hover tooltips. On PC, hover is the main way
  people learn a strategy game. The game has a lot of jargon (§3.4) and no
  hover glossary for it.
- Space pauses play (`src/game.rs`), but the game never tells the player.
  There is no shortcut for 1/2/3 speed, Esc to close or tab cycling, and
  hints such as BEGIN VOYAGE [ENTER] and BACK [ESC] appear on one screen
  only. Add the missing keys and a shortcut list in Help.
- The Steam features players assume are not wired up: the achievements in
  `src/achievements.rs` are local-only, and there is no Steam Cloud. The store
  copy is honest about this, but for a strategy game with long saves, Cloud and
  achievements are close to expected. Steam Deck "Verified" also needs a
  readable text size and controller input.

### B4 — Text clips and wraps at the default resolution

All of these are visible at 1280×720, the normal QA size:

| Screen | Defect |
| --- | --- |
| `ui_ship.png` | Hull blurb cut mid-sentence: "held together by" |
| `ui_heritage.png` | Legacy blurbs cut: "until the mission" |
| `ui_debrief.png` | Recovery cards cut: "+6%", "Unavailable: No active promise can be" |
| `ui_subsystems.png` | "Closed Ecology" title crowds the panel header; "EDU" overlaps "DECKS 3-4"; "EXTEND +25Y · NEED 500cr" wraps inside its button |
| `ui_prep.png` | "+315 PARTS · 3780 CR" and "REVIEW MANDATE" wrap inside their buttons |
| `ui_crew.png` | "NAME HEIR" wraps |
| `ui_contract_active.png` | "PROGRESS" label overlaps its own bar fill |
| `ui_ship_underway.png` | "FIELD INSTALL — Solar Sail Array" wraps |
| `ui_chronicle.png` | Possessive casing bug: "Boro Chartwright'S HOUSE" |

The collision audit in the capture harness should fail the build on clipped
text, not only on overlap. Every one of these is the kind of thing a reviewer
screenshots.

### B5 — Controls appear when they do nothing

- The PAUSE/[1x]/2x/3x cluster shows on Homecoming (`ui_debrief.png`) and on
  Game Over (`ui_gameover.png`).
- In port the cluster is fully shown even though "time stays frozen in port".

This overlaps UI-03, which is not done yet. Treat it as a blocker, because it
breaks trust on the two screens that end a run.

### B6 — Raw simulation numbers are shown to the player

- "PARTIAL (score 0.82)", "1.00/0.85 (w 21%)", "crisis weight +0.12",
  "+0.0% morale · +0.0% unity · PRESERVE LOSS +0%", "DECAY ×1.08" and
  "Custodian: KIND (perceived 100%)".
- Lines where every modifier is zero still appear (`ui_prep.png`,
  `ui_contract_active.png`). Hide zero modifiers.
- Express ratios as grades, words or a ★ outcome, and keep the exact numbers in
  a hover tooltip.
- `ui_contract_active.png` shows **Food 424291 (verify)**. If real play can
  reach that, food balance or the cap (`food_carrying_capacity`) is broken. If
  it comes from the staging, fix the capture so it doesn't end up in store
  screenshots.
- `ui_drydock.png` shows "Y41 after 40 yr" in the Homecoming line while the
  header shows Y000 **(verify)**.

## 3. UI composition and readability

### 3.1 No visual hierarchy

Every region is the same bordered amber rectangle, and every action is the same
bordered amber button. As a result:

- LAUNCH, CANCEL MISSION / RETURN HOME, DELETE SAVE and "Inspect ship" all
  carry the same visual weight.
- Destructive actions are not set apart. DELETE SAVE sits next to BACK on the
  Founding screen (`ui_heritage.png`), which is a dangerous neighbour.
- Secondary tabs (Overview/Instruments, Drydock/Market) look like primary
  buttons.

Define 3 button tiers (primary, secondary, quiet) plus a destructive style, and
use one primary action per screen. This extends UI-02/UI-04 to every screen.

### 3.2 Type

- The game ships with DejaVu Sans only. It reads as a system default.
- Pair a display face for headings and titles (the menu logo already has one)
  with a highly legible UI face.
- Secondary text such as QUOTED TERMS, trait lines, LOCKED notes and recovery
  card bodies renders at about 10–11 px at 1280×720. That is below comfortable
  reading size, and on Steam Deck it would fail. Raise the minimum body size, or
  cut the text so the larger size fits.

### 3.3 Terminal decoration reads as placeholder

The `[ ]` checkboxes, `[1x]`, `> NEW GAME`, `// … //` around titles, `[LEADER]`
and ALL-CAPS label soup are meant as a terminal aesthetic. In practice they read
as debug UI. Either commit to a diegetic Custodian terminal (consistent
typography, animated text, sound) or replace them with real controls.

### 3.4 Jargon load

A new player meets Charter, Writ, Approach, Posture, Mandate, Obligation,
Agenda, Escrow, Knowledge, Institution, Custodianship, Heritage, Renown and
Tradition. The tutorial covers the opening, but the middle game relies on the
player remembering them. Needed:

- a hover glossary on every term;
- an in-game Codex under Help;
- fewer synonyms, for example merging "Charter / Writ / Contract" into a single
  term.

### 3.5 Empty space on some screens, overcrowding on others

- Mostly empty: Drydock shows 2 charters and leaves about 60 % of the screen
  blank (`ui_drydock.png`). The Crew roster, the Event modal and Game Over are
  sparse too.
- Overcrowded: Prep and Active Contract cram dense 10 px lists into the same
  area (`ui_prep.png`, `ui_contract_active.png`).

Fit the density to each screen:

- Use charter cards with art and key stats in a 3–4 wide grid.
- Use a family tree or a generations timeline for People.
- Give event art and the narrative more room.

### 3.6 Navigation depth

Ship → Loadout → LOADOUT/MODULES makes three stacked tab rows (`ui_ship.png`).
Each section's second row has a different meaning: views on Ship, family on
People, phase on Voyage. "Agenda (0)" lives under Ship rather than at the top
level, even though it is the main thing to do mid-voyage.

### 3.7 Modals

- The Welcome dialog renders over Founding, and the buttons underneath show
  through (`ui_welcome.png`).
- Its section headings have no top spacing, and the lower third of the dialog
  is empty.

## 4. Game feel and design gaps

1. **The journey is never shown.** A 340-year voyage appears as a progress
   bar. Nothing on the Bridge moves: no starfield, no route map with the ship's
   position, no destination art, and no arrival moment. *Seeing* the ship
   travel is the cheapest and biggest "this is a real game" upgrade available.
2. **Feedback is minimal.** Resources change without ticking, flashing or
   floating deltas. Screens change without transitions. A Commit lands with no
   beat. Add small animations for resource deltas, committed decisions, deaths
   and births, and milestones.
3. **Succession is the emotional core, but it plays like a spreadsheet.** A
   captain dying or retiring should be a scene: portrait, epitaph, the heir
   accepting, a Chronicle line and the succession music cue. Today it is a
   roster row and a log line (`ui_crew.png`).
4. **Homecoming is an accounting screen** (`ui_debrief.png`). The narrative
   line is good, but then come a decimal score, a population delta and four
   small cards. It should feel like an ending: an arrival illustration, a
   generations-and-captains montage, a clear outcome grade, then the recovery
   choice.
5. **Game Over has no weight** (`ui_gameover.png`). It is a stats table on
   black. Show the Chronicle and the faces of the generations lost.
6. **Meta-progression is thin for Steam.**
   - Heritage is a numeric head start (+3000 cr / +200 inf / +30 tradition).
     There are only 3 legacies.
   - Steam strategy players expect unlocks to pull them into run 2–5: new
     hulls, legacies, founding peoples, charters, starting scenarios and
     Chronicle-unlocked events.
   - Consider difficulty or ironman settings and a run seed shown on screen.
7. **Repetition risk (measure it).** Each charter lasts 150–450 in-game years,
   and runs are open-ended. Instrument a seeded batch of runs to measure:
   - how many events fire per run;
   - the repeat rate within a run and across runs 1–5.

   Use the numbers to decide whether event *variants* or more events are needed.
8. **Session pacing (measure it).** Drydock reports "played 38m" for one charter.
   Check that 1x/2x/3x lets quiet decades pass quickly enough. Consider a "run
   until something needs me" speed; the Agenda and the fallback timers make that
   safe.
9. **The underlying UI paradigm is touch-first, which has a cost.** It produced
   explicit Commit buttons and disclosure panels, which suit phones. On PC the
   same design means more clicks and less information density than players of
   this genre expect. Either keep one adaptive UI with a real desktop layer
   (hover, right-click, shortcuts, denser panels), or decide that Steam is
   desktop-first.

## 5. Store-page readiness

- **Screenshots:** today all five would be amber box layouts. B2 and §4.1 have
  to land first, or the capsule art will promise a game the screenshots don't
  show.
- **Trailer:** there are no motion, music or ship-travel visuals to cut a
  trailer from (B1, §4.1).
- **Capsules:** the menu key art can supply main and header capsules. The
  library hero and logo need clean-plate versions.
- **Identity:** the developer/publisher legal names are still unresolved
  (`docs/release/STORE_COPY.md`).
- **Price positioning:** with music, art and feedback fixed, this is a credible
  niche strategy title. Without them, it competes with free web games.

## 6. Recommended order of work

Each item below is a separate, committable slice.

**Tier 0 — required before a Steam page goes live**
1. B4: fix the text clipping, and make the capture audit fail on clipped text.
2. B5: hide time controls where they do nothing (finish UI-03).
3. B6: hide zero modifiers, replace raw ratios with grades and words, and
   resolve the two **(verify)** numbers.
4. B3: add fullscreen/borderless and resolution options, platform-aware
   "Click/Tap" wording, keyboard shortcuts, and hover tooltips with a glossary.
5. B1: music and authored SFX, plus separate volume sliders.
6. B2: one art direction from menu to Homecoming, one hero ship view, event
   theme vignettes and procedural portraits.

**Tier 1 — needed to earn good reviews**
7. Show the voyage moving: route map, starfield, arrival moment (§4.1).
8. Feedback animations and screen transitions (§4.2).
9. Succession, Homecoming and Game Over as scenes (§4.3–4.5).
10. Button tiers, a destructive style and type pairing (§3.1–3.2), plus the
    density pass (§3.5).
11. Steam achievements and Cloud through Steamworks.

**Tier 2 — what keeps players past run 3**
12. Unlocks and more legacies or starting scenarios (§4.6).
13. Measure event repetition and pacing, then add variants or content (§4.7–4.8).
14. Steam Deck pass: controller input and minimum text size.

## 7. Acceptance check before a Steam release

Have someone who has never seen the game play the first 30 minutes on a
1080p monitor with sound on, with no explanation. Release-ready means:

- they never say "is this finished?";
- they can explain what Posture and Approach do without opening Help;
- they react to the first captain's death.

Record that session alongside the release acceptance in
`docs/release/QA_AND_OPERATIONS.md`.
