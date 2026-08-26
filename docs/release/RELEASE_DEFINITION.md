# Release definition and decision record

Status: **0.1.0 Windows release candidate; not authorised for public release**.

| Field | Recorded scope |
| --- | --- |
| Product | Stellar Legacy |
| Version/build label | 0.1.0 / `0.1.0-windows-x86_64` |
| Operating system | Windows x86-64 only |
| Language | English only |
| Release type | Full release, not Early Access |
| Services | Local saves; no accounts, analytics, telemetry, multiplayer, or network dependency |
| Input promise | Mouse/touch and keyboard; no controller claim |
| Store features promised | Single-player only; no achievements, cloud, cards, workshop, multiplayer, or Deck claim |
| Distribution | Portable ZIP; installer not approved |
| Price | **OWNER DECISION REQUIRED** |
| Copyright holder | **OWNER INPUT REQUIRED** |
| Steam/itch identifiers | **OWNER INPUT REQUIRED** |

`Cargo.toml`, `game_config.json`, player-visible Help, candidate filenames, and build
labels use 0.1.0. The save wrapper also records 0.1.0; future schema changes require a
real migration before the release number changes. Browser metadata is internal catalog
metadata only and is not part of the PC storefront promise.

Minimum and recommended hardware requirements are intentionally blank until measured on
the final signed-off artifact. Do not substitute guesses in store forms.
