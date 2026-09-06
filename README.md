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
- **Build a legacy.** Review each Homecoming and the Chronicle, then prepare the
  next voyage. Chronicle renown grants an automatic Heritage head start when you
  found a new dynasty.

## How to play your first voyage

1. Tap or click **NEW GAME**. Choose a legacy and your founding peoples. Read
   **WHAT THIS CHANGES** to understand the legacy's effects before founding.
2. Open **DRYDOCK** and compare charters. **Proving Run: Lumen Relay** is the
   tutorial voyage: 150 years split into 50-year outbound, operation, and return
   legs. Tap **SELECT** to inspect a charter without launching.
3. Read the departure briefing and the **FOOD**, **PARTS**, and **FUEL** estimates.
   Use the stock-up controls or **MARKET** to provision the ship. Tap
   **PROVISIONS REVIEWED** when the guide asks you to.
4. Inspect **CREW & DYNASTY**, **SHIP**, and **SUBSYSTEMS**. Check the people,
   loadout, and equipment that must carry the mission. Time stays frozen in port,
   so take time to prepare.
5. Return to **DRYDOCK** and tap **LAUNCH**. Read any shortage or obligation
   warning carefully: launching understocked or breaking an existing promise is
   a real commitment.
6. Under way, use **DASHBOARD** to watch the ship and **CONTRACT** to track the
   mission. Start at **1x** and tap **PAUSE** whenever you need to review or act.
   Tap **RESUME** to continue; **2x** and **3x** speed up quieter stretches.
7. When a council decision appears, read its choices and tap an option card.
   The voyage clock stops for blocking decisions, but the decision countdown can
   still run. Tap **PAUSE** to hold that countdown while you consider the choice.
8. At **Homecoming**, review the outcome and what the voyage cost your ship and
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
| **DASHBOARD** | Review resources, ship condition, and recent developments. |
| **SHIP / SUBSYSTEMS** | Inspect the loadout and manage the systems keeping the vessel running. |
| **CREW & DYNASTY** | Review your people, captain, and succession. |
| **DRYDOCK / MARKET** | Choose and prepare charters, and trade while in port. |
| **CONTRACT** | Follow mission progress; tap **REVIEW MANDATE** for authority rules. |
| **CHRONICLE** | Read the record of earlier voyages and the legacy they leave. |
| **SAVE / MENU** | Save your campaign or return to the menu; use **CONTINUE** to resume a saved game. |
| **HELP / DISPLAY** | Read controls and identity guidance, or adjust presentation, audio, and delegation settings. |

Saves are local. On Windows, **HELP → OPEN SAVE FOLDER** opens their location.
The game remains readable with audio muted.

## Advice for a lasting dynasty

- **Stock for the whole journey.** Farms offset food demand, but shortages can
  still become starvation. Parts support upkeep; running out of fuel stalls
  progress and strains the ship. The market is only available in port.
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
