# Browser interaction checks

## 9 September 2026 — founding and first charter

Used the publisher's packaged `dist/webgl-smoke` site in the in-app browser
at 1126 × 912. The inspected package was built from `708d12c`, before the
founding-picker changes accompanying this note.

Using visible clicks and a drag, completed: New Game → Begin Briefing →
select three founding peoples → Begin the voyage → Voyage → Read briefing &
prepare on Proving Run: Lumen Relay. The guide advanced to the provisioning
lesson, with the new instruction visible at the top of the reading view.

The founding picker showed names without descriptions and still enabled
unselected choices after reaching the limit. The accompanying change adds
outlooks, cared-for subsystems, a selected count, explicit Remove controls,
and disabled excess choices. Phone captures cover the revised picker; they
are layout evidence, not a live replay of the revised route.

Full voyage, save/reload at all major boundaries, succession, terminal recovery and fresh-player
comprehension acceptance remain open in TODO.md. This check does not close
those gates.

Continued the same loaded package through Provisions Reviewed → People → Ship →
Voyage → Launch → Ship / Agenda → Available projects → Queue project → Pause →
Resume. Each lesson advanced and returned its instruction to the top. The first
council event, The Slow Pocket, appeared; Pause changed to Resume and stopped
the countdown. Its displayed fuel consequences were +20000% and -18000%, which
triggered the fuel-preview correction below.

The departure report also mixed tank percentages with unlabelled fractions.
The next revision explicitly names full tanks and percentage recovery; its
desktop capture keeps Provisions Reviewed and Launch visible.

After the fuel-preview correction passed publishing, used Utilities / Save game,
reloaded the page and clicked Continue. The paused campaign and pending Slow
Pocket decision survived; the fallback countdown restarted at 90 seconds. Both
choices displayed the current 97.5% tank changing to 100% or 0%. Committed Cut
through the fold, observed 100% fuel and the guide-complete message, then tapped
FINISH and returned to the ordinary Bridge. This completes the sampled tutorial
click route across the inspected packages, including one save/reload boundary.

Fuel preview tests compare displayed results with actual resolution at tank
limits and with subsystem protection. Authored fuel deltas and gameplay balance
were not changed by the display correction.

Project review navigation exposed an invisible clock hold: Review project →
Bridge → Open project queue restored the old review. After the lifecycle fix
passed publishing, saved and reloaded the paused campaign, then repeated that
route and observed the queue. Review project → Utilities → Close utilities also
returned to the queue. The Resume control remained visible throughout, preserving
the player's pause. Focused tests cover both ordinary review and cancellation
state on navigation, while project pause/resume keep the review open.
