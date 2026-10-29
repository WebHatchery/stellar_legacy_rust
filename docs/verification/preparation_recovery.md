# Preparation and recovery hierarchy

2026-10-30: preparation now starts with commitments, stores, credits and
provisioning costs. Launch retains explicit shortage/default labels and the
selected approach/posture. Briefing, approaches and forecast explanations
have separate phone views with visible Back controls. Bridge obligations
are independent of the resource-warning selection, with a separate review
action on desktop and precedence in the phone reading order.

Reviewed native captures from the shared hidden wrapper:

- `ui_prep_portrait.png`: 390×844, conflicting promise and mixed provisions;
  the parts purchase and its cost are visible in the initial view.
- `ui_prep_portrait_9999.png`: bottom of the same preparation flow, including
  launch consequences, selected approach/posture, and disclosure controls.
- `ui_prep_briefing_portrait.png`, `ui_prep_approach_portrait.png`, and
  `ui_prep_forecast_portrait.png`: detail views with visible return controls.
- `ui_prep_scale_100_text_150_portrait.png`: 390×844 with 150% text; wrapping
  and two-row navigation remain readable, with preparation continuing below.
- `ui_bridge_commitments.png`: 1280×800 with simultaneous energy shortage and
  overdue promise; both warnings and both actions are visible.
- `ui_bridge_commitments_portrait.png`: 390×844; obligation review precedes
  energy recovery in the scrolling flow.

Wrapper runs completed successfully and their game processes exited. Reviewed
the existing section-button dispatch and scroll-key reset paths in source;
the captures seed these states rather than clicking through them. Browser,
physical touch, and an interactive purchase/launch round trip were not tested.
No deployment was performed.

Validation: formatting, strict all-target/all-feature Clippy, all six existing
structural checks (including file/function size), embedded-registry loading,
and diff whitespace checks pass. No new tests were added for this UI change.
Simulation and transaction handlers are unchanged.
