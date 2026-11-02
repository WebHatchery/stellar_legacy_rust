# UI text review

Stellar Legacy is a text-heavy game by design: the player is reading a ship's
monitor. That makes every line expensive. This review covers all 25 screens in
`docs/verification/` (1920x1080 set, 2026-11-02).

## The test every line must pass

A line stays only if it is one of:

1. **Identity** — a name, a title, a ship class.
2. **A number the player acts on** — and only when a bar, gauge or lamp cannot
   show it better.
3. **Story** — prose that is the point of the screen (event text, chronicle,
   homecoming).
4. **An action label** — what a button does.

Everything else is cut, turned into a lamp or chart, or moved into the guided
tutorial.

## Cross-screen rules

1. **Hide zeros and empty states.** No `WORK +0% · EVENTS +0% · FUEL +0%`, no
   `ANNUAL PEOPLE +0.0% morale…`, no `0 ACTIVE · 0 WAITING`, no
   `Body-horror events: 0`. Non-zero modifiers become signed bars around a
   centre line.
2. **Status words become lamps.** `NO PRIOR READING`, `STABLE/VULNERABLE`,
   `SOUND`, `POST VACANT`, `INSTALLED`, `RESTLESS/NEUTRAL/DEVOTED`, `[ ]`
   milestones. A trend is an arrow, or nothing when there is no prior reading.
3. **No instructions on panels.** "Tap REVIEW PROJECT…", "Repair or refit on
   the SHIP screen…", "The market line is raw…". These live in the guided
   tutorial only.
4. **No internal numbers.** `(w 21%)`, `1.00/0.85`, `score 0.82`, `DECAY ×1.08`,
   `crisis weight +0.12`, `Existential dread 0.00`, `perceived 100%`,
   `SCHOOL→Y25`. Chart them (bar with target tick) or hide them.
5. **Say it once.** Bridge says "choose a charter" four ways; Homecoming prints
   the recovery sentence twice; "Who decides" lives on Welcome, Help and
   Council; Ship Overview repeats the status strip.

## Screen by screen

### Main menu
No changes.

### Welcome / orientation
About 150 words in four paragraphs. Cut to three one-line boot messages: what
you are, what you do, how it ends. "Tap Read briefing…" is tutorial content;
"this is a campaign ending, not the Custodian dying" is a designer's note.

### New game (heritage)
- Legacy descriptions and WHAT THIS CHANGES are truncated mid-sentence; use a
  short tagline per legacy.
- Founding peoples: replace six `tech-averse · traditional` lines with a 2-axis
  plot (tech ↔ averse, traditional ↔ radical), chosen peoples lit.
- Heritage line becomes a badge.

### Bridge
- One "choose a charter" message, not four.
- `Hull 100% · Air 100% · Fuel 100%` becomes three mini gauges.
- `DECKS 1-2` labels are clutter around already-named tiles.

### Instruments
- Status bars are good.
- Class/drive/cargo list duplicates the Ship tab.
- `CUSTODIAN CONDUCT BALANCED 50%` becomes a needle on a slider.
- Drop `CONSEQUENCES 0 / DELEGATED 0 / FROM THE FOUNDING 0%`.
- Empty centre space: population and morale history graph.
- Population breakdown: one stacked bar coloured by approval.
- Bottom strip: `PRIMARY RISK Sound`, `WEAKEST MODULE Sound`,
  `NO ACTIVE COUNCIL` become dim icons when nothing is wrong.
- Ship's log lines truncate.

### Ship: Loadout
- Card stats (`CARGO 200 · CREW 12`) become bars against the installed part
  with ▲/▼ deltas.
- Top-right summary repeats the INSTALLED cards.
- `AGR 0 / EDU 0` on the mini schematic means nothing to the player.

### Ship: Underway
- 11-line schematic key repeats names the schematic already shows; keep three
  swatches (manned/vacant, good/worn, critical).
- SPEED, COMBAT, CARGO appear three times.
- Spare parts: bar against recommended.

### Systems
- Left list becomes condition bar + tier pips, or goes (schematic is the
  selector).
- `MORALE +0.4% · UNITY +0.3% · STABILITY +0.0% · CRAFT -0.2%/y · DECAY ×1.08`
  becomes a signed bar chart.
- `TIER ●●● · SCHOOL→Y25 · ARCHIVED…` is cryptic.
- Cut "Tap a compartment to inspect".

### Agenda
- Ten identical text rows: add status lamp, in-row progress bar, module icon in
  place of `· Hydroponics & Agriculture`.
- `NO PRIOR READING` ×4 goes.
- "Paused by the Custodian" repeats "Paused"; footer instructions go.

### People: Family
- Roster rows become columns: lifespan bar, leadership bar.
- Empty lower two-thirds wants a family tree.

### People: Officers
- `Age 32 · 37 years left · Skill 50` becomes lifespan + skill bars; training
  target as a ghost segment.
- `POST VACANT` becomes an empty silhouette with a red lamp.
- `RECRUIT (800 / CR)` buttons wrap.

### People: Factions
Biggest chart opportunity: three lines of text, ~80% empty. Approval gauge per
people, population share bar, rivalry triangle with red edges, care module as
icon.

### People: Council
- "Who decides?" to one sentence.
- Rows should read as two-position switches.
- `Next generation in 25 years` becomes a countdown bar.
- Legacy condition (`Body-horror events`, `Existential dread`,
  `Piracy reputation`, `cultural collapse`) shows internal names; lamps, unused
  hidden.

### Drydock
- Homecoming condition sentence becomes gauges.
- Renown 82 beside `LOCKED · RENOWN 100` should be a progress bar.
- `Route: ordinary crisis exposure · 0 launch promises` becomes 1–3 risk pips,
  promise badge only when > 0.
- Charter length as a bar.

### Prep (departure)
Heaviest screen, biggest gain.
- Food/Parts/Fuel `stored X / recommended Y` become bars with a recommended
  tick.
- Food projection + fuel burn + scoop rate become one stock-over-voyage line
  chart across the travel / operation / return bands, showing where each runs
  dry.
- Cut "Food is a reserve; farms supply…".
- `ROUTE LOAD · crisis weight +0.12…` becomes risk pips.
- Charter approach description repeats its title.
- Posture `WORK 100% · EVENTS 100% · FUEL 100%` becomes three needles centred
  on 100.
- `LAUNCH · 2 SHORT · DEFAULT 1` is cryptic; lamps on the button.
- Obligation conflict stays as text in a warning box with an icon.

### Tutorial banner
Truncates ("Stock…") and draws over the faded top bar. One line per step;
highlight the target element instead of describing its location.

### Active contract
- Right panel's ten label lines become a route line: origin ● — ship — ◆
  milestones — site ●.
- `[ ]` milestones become pips on the phase bar.
- Success metrics: bar with target tick, no `(w 21%)`.
- Fuel stall: alarm lamp.
- Hide "Campaign outlook · exact event withheld" when empty.

### Event
- Effects line becomes signed chips.
- Better: both options' effects side by side as a bar chart.
- "Fallback in 90s" becomes a countdown bar.

### Market
- Quoted terms become buy/sell price only, modifier tags when they apply.
- Trend `— +0.00` becomes a price sparkline.
- Held becomes a bar against cargo.
- Cut footer and "QUOTES LOCK ON TAP".

### Homecoming
- Opening paragraph is earned story; keep it.
- `Population 1000 → 326 (-674)` becomes a population curve over the voyage.
- Final score as a gauge.
- Drop the all-zero charter approach block.
- Recovery sentence printed twice.
- `0% sound / 21% remembered` becomes two bars.

### History (timeline)
Text belongs here. Make the selector a horizontal timeline with year ticks and
generation bands.

### Help
Controls table states the obvious; "Who decides" is the third copy. Keep the
version/save-folder lines and anything non-obvious.

### Settings, Utilities, Game over
Fine. Game over could add a population sparkline.

## Bugs noticed

- Bar labels drawn under the fill (`PROGRESS` on Contract, `CULTURAL DRIFT` on
  Instruments).
- Contract top bar shows `Food 424291`.
- `Boro Chartwright'S HOUSE` casing in History.
- Truncation: heritage legacy text, tutorial banner, Instruments log lines.

## Order of work

1. Global hide-zeros / empty-state rule.
2. Prep provisioning charts.
3. Factions panel.
4. Contract route line.
5. People lifespan and skill bars.
6. Event options side by side.
7. Market sparklines.
8. Homecoming population curve.
9. Move instructions into the tutorial.
