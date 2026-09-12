# Stellar Legacy project guidance

This project follows the shared RustGames checklist, with these intentional
boundaries and names:

- `src/data.rs` owns the embedded `GameData` registries and their typed,
  project-specific validation. Game content is loaded through
  `macroquad_toolkit::data_loader`; save and Chronicle migrations may use
  Serde directly for their persisted payloads.
- `src/simulation.rs` is the engine boundary. Its `simulation/` children own
  deterministic calculations and state transitions; they do not render or
  read input. `state/` owns the serializable campaign and menu state.
- `src/game.rs` coordinates the state machine, action dispatch, persistence,
  and capture setup. Capture-scene modules are test/capture fixtures, not a
  second gameplay implementation.
- `src/ui.rs` and `ui/` are a batched-intent view layer. A frame may return a
  `Vec<UiAction>` because several independent visible controls can be pressed
  or released together; dispatch remains centralized in `game/actions`.
- `src/lib.rs` is the testable library surface. `src/main.rs` is only the
  Macroquad window/runtime shell and calls the library's `run` entry point.

The responsive mobile shell is a deliberate presentation layer rather than a
separate screen state. Desktop and compact views read the same `GameState` and
emit the same actions, so adding a destination or interaction requires updating
the shared `Screen`/`UiAction` model rather than duplicating simulation logic.

The one ignored test in `simulation/balance` is a release-analysis report, not
part of the ordinary regression suite. It must write outside the repository
root and describe the current charter registry before it is used as evidence.
