//! Save integrity (CODE_STANDARDS §11.2, data loss): a campaign survives a save
//! and reload through a non-ASCII profile path, and an unreadable save is
//! preserved aside instead of being overwritten by the next autosave.
#![cfg(not(target_arch = "wasm32"))]

use macroquad_toolkit::persistence::get_app_data_path;
use stellar_legacy::{founding_faction_ids, load_campaign, save_campaign, GameData, SimState};

fn isolated_data(name: &str) -> GameData {
    let mut data = GameData::load().expect("embedded game data");
    data.config.game_name = format!("stellar_legacy_{name}_{}", std::process::id());
    data
}

#[test]
fn campaign_round_trips_atomically_with_a_non_ascii_profile_path() {
    let data = isolated_data("雪_save_test");
    let sim = SimState::new_campaign(&data, "preservers", 0x5A7E, &founding_faction_ids(&data));

    save_campaign(&data.config, &sim).expect("save");
    let loaded = load_campaign(&data).expect("load");
    assert_eq!(loaded.seed, sim.seed);
    assert_eq!(loaded.dynasty.generation, sim.dynasty.generation);
    assert_eq!(loaded.dynasty.members.len(), sim.dynasty.members.len());

    let path = get_app_data_path(&data.config.game_name, "save_autosave.json").unwrap();
    let temporary = path.with_file_name(format!(
        ".{}.tmp",
        path.file_name().unwrap().to_string_lossy()
    ));
    assert!(path.exists());
    assert!(
        !temporary.exists(),
        "atomic-save temporary file was left behind"
    );

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(path.parent().unwrap());
}

#[test]
fn corrupt_campaign_is_quarantined_before_a_new_save_can_replace_it() {
    let data = isolated_data("quarantine_test");
    let path = get_app_data_path(&data.config.game_name, "save_autosave.json").unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{not valid json").unwrap();

    let error = load_campaign(&data).unwrap_err();
    assert!(error.contains("Preserved as autosave_corrupt"));
    assert!(!path.exists());
    let quarantine =
        get_app_data_path(&data.config.game_name, "save_autosave_corrupt.json").unwrap();
    assert_eq!(
        std::fs::read_to_string(&quarantine).unwrap(),
        "{not valid json"
    );

    let _ = std::fs::remove_file(&quarantine);
    let _ = std::fs::remove_dir(path.parent().unwrap());
}
