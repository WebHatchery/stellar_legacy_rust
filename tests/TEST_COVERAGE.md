# Feature coverage review

`CODE_STANDARDS.md` §11 governs testing: strongly target no more than five
`#[test]` cases per major feature, counted across all files and suite names.
Tests should read like rules, use simple setups, and exercise the public API.
Keep all tests and test-only helpers in the crate's `tests/` directory beside
`Cargo.toml`; `src/` must not declare or include test modules.

The existing `tests/unit/` files are still included from source modules and
exercise private seams. That is unfinished legacy migration, not an approved
placement pattern. Migrate those suites separately before expanding coverage,
using intentional library APIs without exposing unrelated internals.

Larger existing suites preserve different game contracts: simulation timing
and state transitions, event effects, contract scoring, succession, content
loading, and save compatibility. Preserve those regressions during migration,
but review each cohesive responsibility independently and consolidate related
inputs with table-driven assertions. A directory or module split does not
create a new feature allowance. Existing UI/layout cases do not establish a
requirement for more UI or rendering unit tests; use visual/manual verification
where appropriate.

Before committing an affected feature with more than five cases, record the
specific additional rules and why they warrant separate cases in the change's
review or commit. Existing suite size alone is not an exception rationale, and
useful coverage must not be deleted merely to reach the target.
