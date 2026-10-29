# Save protection verification — 29–30 October 2026

Scope: 30-second session checkpoints and immediate campaign-boundary saves;
manual/menu/Homecoming saves share failure handling. No save schema change.

## Native interaction checks

Ran the actual checkout through `..\rust_management\cargo.ps1 run -p
stellar_legacy --features demo`. The demo namespace had no existing files before
the run; the full-game profile was not modified. Used the visible 1280×720 game
controls and inspected the resulting native save, without editing campaign data.

- Founded a dynasty and observed a save immediately without using Save game.
- Launched Lumen Relay and observed its contract in the departure checkpoint.
- Paused at month 12. The periodic checkpoint saved the paused speed, month and
  current resources while simulation time stayed stopped.
- Made only this run's save file read-only to force an atomic replacement error.
  Return to menu retained the live voyage. Restored write access and observed
  automatic retry succeed and the saving-restored notification.
- Closed the window without using the save/menu controls, rebuilt/reopened the
  demo and used Continue. The same captain, Year 1 paused state and resources
  returned (10090 credits, 4579 energy, 2015 minerals, 1515 food, 101 influence).
- Repeated a manual-save failure on the final build. The full notice
  `Save failed. Keep game open.` was readable; History retained the error and
  retry instructions. Restored write access and used Save game successfully.
- Closed both task-launched game processes and confirmed successful exit. The
  normal demo save remains available, with its read-only attribute cleared.

## Engineering checks

- Formatting and `git diff --check`: passed.
- Existing `save::tests`: six passed, including atomic round-trip, quarantine,
  invalid data rejection and persisted project investments.
- Existing `code_standards` integration suite: six passed, including source size.
- Existing embedded registry loading check: passed.
- WASM demo target check: passed; no deployment performed.
- Strict Clippy with all targets/features: passed after fixing the 13 existing
  errors in a separate cleanup slice. No lint suppression was added.
- Cleanup verification: 41 existing contract tests and six readiness tests passed;
  five affected desktop captures and a 390×844 portrait capture were reviewed.
  Mobile action dispatch and scrolling were reviewed in source, not touch-tested.
- Re-ran all six save tests and the WASM demo check after cleanup: passed.

## Limits

No live browser storage/quota test, physical touch test, power-loss test, full
Homecoming/demo-completion failure walkthrough, succession or terminal playthrough
was performed. Homecoming failure retention and capture-mode exclusions were
reviewed in source. Campaign and Chronicle remain separate persistence operations;
this change does not introduce transactional multi-file recovery or backup slots.
Abrupt interruption may still lose changes since the last successful checkpoint.
