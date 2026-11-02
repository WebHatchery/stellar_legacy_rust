//! Data integrity (CODE_STANDARDS §11.2): the embedded content and config must
//! load with every cross-reference resolved, and the save label must track the
//! release so a shipped build never writes saves it would call foreign.

use stellar_legacy::GameData;

#[test]
fn embedded_content_loads_with_resolved_references() {
    // `GameData::load` rejects duplicate event ids, unresolved project and
    // issue references, and config values the simulation cannot run on.
    let data = GameData::load().expect("embedded game data");
    assert!(data.events.ids().next().is_some(), "no events embedded");
    assert!(
        data.contracts.ids().next().is_some(),
        "no charters embedded"
    );
}

#[test]
fn save_label_matches_the_cargo_release() {
    let data = GameData::load().expect("embedded game data");
    assert_eq!(data.config.version, env!("CARGO_PKG_VERSION"));
}
