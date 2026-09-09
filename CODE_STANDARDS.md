# Stellar Legacy coding standards

These are the project's contributor rules. [AGENTS.md](AGENTS.md) contains the
workspace workflow; [gdd.md](gdd.md#11-code-ownership) maps the actual code.

## 1. Core philosophy

Prefer readable, explicit ownership and small modules. Keep focused changes
narrow. Game rules and authored balance belong here; evaluate reusable rendering,
input, assets and platform behavior as shared toolkit improvements first. Avoid
new dependencies unless they remove real complexity or follow an established pattern.

## 2. Project structure

### 2.1 Responsibilities

`main.rs`/`boot.rs` initialize the runtime. `game.rs` and `game/actions/` coordinate
and dispatch intents. `state/sim/` owns serialized campaign state; `simulation/`
mutates it through explicit services. `data/` owns schemas and validation; `ui/`
reads state and returns actions. The main file is not the sole mutation site.

### 2.2 File size

Target 200–400 lines and reconsider structure around 600. Every `.rs` file has an
800-total-physical-line limit, including tests, comments, whitespace, generated
source, examples and build scripts. No exceptions. Extract cohesive responsibilities;
do not strip useful spacing or compress code to pass.

### 2.3 Module filenames

Use `foo.rs` and `foo/bar.rs`, never create new `mod.rs` files. Do not keep both
root forms for a module. Follow existing small domain boundaries.

## 3. Naming

Use PascalCase types, snake_case functions/variables/modules, and
SCREAMING_SNAKE_CASE constants. Boolean names should read as facts. Match existing
public/action vocabulary and use the same fictional term for the same system.

## 4. Functions and methods

Keep functions focused, normally 20–50 lines. Split long operations by
responsibility. Use configuration/context structs when numerous related parameters
obscure intent. Use `Option` for absence, `Result` for failure, and named result
structures where tuples would hide meaning. Avoid needless variable shadowing.

## 5. Data and state

Keep balance, authored content and shared prose in JSON. Embed/parse game data
through toolkit `data_loader` and include macros; do not add game-local generic
loaders or direct string parsing for game-data files. Typed schemas and semantic
checks remain in `data/`. See [event authoring](event_design_notes.md).

Simulation randomness comes from `SimState.rng`. Rendering, previews and cosmetic
clocks must not consume or mutate simulation state. Keep saves, authority,
obligations and delivered project effects coherent across reload and succession.
Preserve compatible IDs; add explicit migrations when meaning changes.

## 6. Error handling

Propagate required asset and save failures with useful context. Keep recovery
visible and preserve invalid saves through quarantine where possible. Never hide
a failed save or required asset load behind a success message. Use panics for
unrecoverable invariants, not ordinary player mistakes.

## 7. UI and input

UI returns `UiAction` intents; dispatch rechecks eligibility and affordability.
Drawing should not apply gameplay effects. Shared widgets own presentation and
interaction state, while the game owns simulation transitions.

Every required start, tutorial, gameplay and recovery action must be possible by
visible tap/click controls or an explicit direct-touch gesture. Give required
controls at least 44x44 logical-pixel targets. Shortcuts are supplemental; a
player-facing shortcut string also names its visible control. Tutorial instructions
state the exact next target. Menus must not leak input into the underlying game.

Bound every text draw by width and height; wrap, shrink or truncate before overlap.
Keep pointer coordinates aligned with the drawing camera. Reflow or scroll for UI
scaling and small viewports; keep global pause/recovery reachable. Color and sound
must not be the sole carriers of essential state. Preserve the Custodian/human
actor distinction in all player copy.

## 8. Publishing

Run `.\publish.ps1` with no parameters after meaningful changes. The project wrapper
builds/packages Windows and WebGL, deploys Preview and runs exact-package smoke
checks. Report missing infrastructure or unrelated failures explicitly; do not
substitute an unrequested local game/server run. Production and store release are
separate actions. Keep `catalog_thumbnail.png` aligned with the title screen;
the publisher deploys it as `<game_slug>/catalog_thumbnail.png`.

## 9. Documentation

Explain why, ownership, invariants and failures. Keep current design in `gdd.md`,
authoring in `event_design_notes.md`, open work in `TODO.md`, and operational gates
in release QA. Remove completed planning diaries rather than maintaining parallel
claims. Generic toolkit guidance stays with the sibling dependency.

## 10. Formatting and tooling

Use package-scoped Rustfmt, tests and Clippy commands from [README](README.md).
Fix diagnostics; explain intentional allowances. Remove unused private code and
fields rather than hiding them with underscore prefixes. Trait-required unused
parameters may use underscore names. Public contracts require consumer review
before removal.

## 11. Testing

### 11.1 Coverage

Test observable simulation, authority, accounting, fallback, save and input/layout
rules. Use deterministic scenarios and failure cases. GPU/browser/hardware checks
and human play establish behavior that unit tests cannot.

### 11.2 Test style

Keep fixtures small and tests readable. Test outcomes and invariants, not copies
of the implementation. Use existing autoplay for comparative policy tests; do not
create an alternative economy simulator.

### 11.3 Test placement

Unit tests live in separate child files, declared by `#[cfg(test)] mod tests;`
in `foo.rs` with bodies in `foo/tests.rs`. This preserves private access through
`use super::*`. Never put inline unit-test module bodies in implementation files.
The 800-line limit also applies to every test source. Crate-root `tests/` is for
actual integration tests; `tests/code_standards.rs` enforces the source-size gate.

## 12. Verification artifacts

Store screenshots directly in `docs/verification/`, no subfolders. Replace earlier
images of the same state. Verify actual dimensions and describe the scope of each
capture; fixture screenshots do not imply live interaction or human acceptance.

## 13. Completion and commits

Follow [AGENTS.md](AGENTS.md) and the workspace commit convention. Finish, validate
and commit each major independent change. Stage all modified/untracked project
files as required, review the final diff, and report the commit and validation
result. Keep changes scoped to Stellar Legacy unless the task explicitly expands.
