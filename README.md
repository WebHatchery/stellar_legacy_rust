# Stellar Legacy

A generational starship strategy game about keeping a living ship-civilization
alive through voyages that last decades or centuries.

You are the **Custodian**, the persistent intelligence aboard the ship. Captains
age, heirs inherit, and councils change. You remain, carrying the mission and the
promises made to people whose descendants will live with your decisions.

## What you do

Choose a charter, prepare your vessel, and guide its people through the outbound
journey, mission operations, and return home. Balance supplies, ship maintenance,
and crew welfare while responding to crises and competing demands. A successful
voyage gives your dynasty the means to undertake another; a costly one leaves
scars that the next generation must manage.

- **Prepare for the long haul.** Compare charter objectives and duration, refit
  the ship, and stock food, parts, and fuel before leaving port.
- **Keep a society together.** Watch morale, unity, cultural change, and the
  interests of the founding peoples. Survival is more than a working engine.
- **Guide successive captains.** Develop the crew and plan for leadership changes
  as generations grow old aboard the same vessel.
- **Make consequential choices.** Council dilemmas trade immediate relief against
  future costs. Recorded obligations can conflict with later charters.
- **Keep the quiet years useful.** Open the **CUSTODIAN AGENDA** to compare food,
  fuel, engineering, life support, knowledge, and cohesion readiness, then queue
  scarce ship projects. Work starts only when a slot and its full escrow are ready.
- **Build a legacy.** Review each Homecoming and the Chronicle, then prepare the
  next voyage. Chronicle renown grants an automatic Heritage head start when you
  found a new dynasty.

## How to play your first voyage

1. Tap or click **NEW GAME**. Choose a legacy and your founding peoples. Read
   **WHAT THIS CHANGES** to understand the legacy's effects before founding.
2. Open **Voyage / Drydock** and compare charters. **Proving Run: Lumen Relay** is the
   tutorial voyage: 150 years split into 50-year outbound, operation, and return
   legs. Tap **SELECT** to inspect a charter without launching.
3. Read the departure briefing and the **FOOD**, **PARTS**, and **FUEL** estimates.
   Use the stock-up controls or **MARKET** to provision the ship. Tap
   **PROVISIONS REVIEWED** when the guide asks you to.
4. Inspect **People**, **Ship / Loadout**, and **Ship / Systems**. Check the people,
   loadout, and equipment that must carry the mission. Time stays frozen in port,
   so take time to prepare.
5. Return to **Voyage / Drydock** and tap **LAUNCH**. Read any shortage or obligation
   warning carefully: launching understocked or breaking an existing promise is
   a real commitment.
6. Under way, use **Bridge** to watch the ship and **Voyage / Contract** to track the
   mission. Start at **1x** and tap **PAUSE** whenever you need to review or act.
   Tap **RESUME** to continue; **2x** and **3x** speed up quieter stretches.
7. Open **Ship / Agenda**. Read the readiness rows and any aftermath notices, choose a
   useful project, tap **QUEUE**, and tap **RESUME**. Two projects may run at once;
   queued work waits without charging until it starts.
8. When a council decision appears, read its choices and tap an option card.
   The voyage clock stops for blocking decisions, but the decision countdown can
   still run. Tap **PAUSE** to hold that countdown while you consider the choice.
9. If life support becomes critical, follow the visible recovery review. Food and
   fuel warnings are forecasts, while hull, population, dynasty, and prolonged
   zero-air failures can end the campaign.
10. At **Homecoming**, review the outcome and what the voyage cost your ship and
   people. Return to drydock, replenish and refit, and choose your next charter.

Follow the on-screen **CUSTODIAN GUIDE** for the opening steps. The full game
includes the tutorial charter alongside the wider campaign; the itch.io HTML5
demo is limited to that single tutorial voyage.

## Controls and useful screens

The game works with a mouse or touch. Use visible buttons and option cards;
a keyboard is optional. Drag lists to scroll.

| Control or screen | Use it to |
| --- | --- |
| Numbered tabs | Switch between the ship's screens. Available tabs change between port and travel. |
| **PAUSE / RESUME**, **1x / 2x / 3x** | Control the passage of time. |
| **Bridge** | Review resources, ship condition, and recent developments. |
| **Ship / Agenda** | Compare readiness, inspect aftermath, and queue, pause, resume, reorder, or cancel underway ship projects. |
| **Ship / Systems** | Inspect the loadout and manage the systems keeping the vessel running. |
| **People** | Review your people, captain, and succession. |
| **Voyage / Drydock / Market** | Choose and prepare charters, and trade while in port. |
| **Voyage / Contract** | Follow mission progress; tap **REVIEW MANDATE** for authority rules. |
| **History** | Read the record of earlier voyages and the legacy they leave. |
| **SAVE / MENU** | Save your campaign or return to the menu; use **CONTINUE** to resume a saved game. |
| **HELP / DISPLAY** | Read controls and identity guidance, or adjust presentation, audio, and delegation settings. |

Saves are local. On Windows, **HELP → OPEN SAVE FOLDER** opens their location.
The game remains readable with audio muted.

## Advice for a lasting dynasty

- **Stock for the whole journey.** Farms offset food demand, but shortages can
  still become starvation. Parts support upkeep; running out of fuel stalls
  progress and strains the ship. The market is only available in port.
- **Spend quiet years deliberately.** Project queues are free to create, but a
  project rechecks its target and charges its full material escrow when it starts.
  Paused work can accumulate deterioration debt, and cancellation refunds only
  recoverable stores.
- **Read readiness as a forecast.** Gross food reserve can hide a net deficit after
  crew demand, route tolls, and spoilage. The Agenda also surfaces maintenance and
  event aftermath before they become linked failures.
- **Recover before the warning becomes final.** A critical air notice pauses the
  voyage for review and offers one bounded emergency stabilisation. Terminal
  outcomes are recorded in the Chronicle and remain final for that campaign.
- **Treat warnings as decisions.** The launch button can allow an understocked
  departure or a default on obligations. Permission to launch is not assurance
  that the voyage is well prepared.
- **Watch people as closely as supplies.** A ship can carry enough food and still
  face a divided society or a difficult succession.
- **Know who is acting.** You are the Custodian, working with human captains and
  councils. Captains can object to strategic policies. Delegated choices and
  timed fallbacks name the acting authority in the log; pause if you want time
  to make the decision yourself.
- **Read the Homecoming report.** Use the last voyage's losses and achievements
  to decide what the next departure needs.

## Development and release notes

Built with Rust, Macroquad, and the shared `macroquad-toolkit`. Game rules and
content run locally; no account is required.

Design and contributor references:

- [Game design](gdd.md): systems, rules, and design goals.
- [Content direction](content_depth.md): guidelines for deepening the game.
- [Event authoring](event_design_notes.md): event structure and content rules.
- [Open work](TODO.md): outstanding tasks.
- [UI redesign plan](docs/ui_redesign_plan.md): screenshot findings, visual direction,
  phased screen improvements, and acceptance criteria.
- [Ship work implementation plan](docs/ship_work_implementation_plan.md): proposed
  semi-idle projects, readiness, aftermath, and survival rules.
- [Release documentation](docs/release/): packaging, QA, and release records.

From the project directory, build, validate, and deploy the normal Windows and
WebGL release with:

```powershell
.\publish.ps1
```

After validating the normal release, package the itch.io tutorial demo without
uploading it with:

```powershell
.\publish-itch.ps1 -Channel html5 -DryRun
```


### Ship-work verification

Version 0.2.1 corrects the audited survival, Agenda escrow, readiness, and save
issues. Open **Ship / Agenda**, then **REVIEW PROJECT** to compare retained deliveries,
remaining work, refunds, and restoration cost before pausing, resuming, or
confirming cancellation. New quarters recover morale in ten stages. Fractional
resource change remains in the saved ledger; older quarters retain their original
single delivery. Authored issue `food_production_penalty` values affect current
production only while the issue is active; `refundable` resource masks define
which project escrow can be recovered.

The [verification record](docs/ship_work_validation.md) includes the 864-run
policy comparison and remaining human acceptance gates. Passing builds and
publisher checks do not mean those human gates have passed.

Navigation stays in the same order in port and underway: Bridge, Ship, People, Voyage, History. Save, Help and Display & sound are in Utilities. Pause/Resume and speed remain visible above blocking decisions.
