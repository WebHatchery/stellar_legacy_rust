use super::*;

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn corrupt_campaign_is_quarantined_before_a_new_save_can_replace_it() {
    let mut config = crate::data::GameData::load().unwrap().config;
    config.game_name = format!("stellar_legacy_quarantine_test_{}", std::process::id());
    let path =
        macroquad_toolkit::persistence::get_app_data_path(&config.game_name, "save_autosave.json")
            .unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{not valid json").unwrap();

    let error = load_campaign(&config).unwrap_err();
    assert!(error.contains("Preserved as autosave_corrupt"));
    assert!(!path.exists());
    let quarantine = macroquad_toolkit::persistence::get_app_data_path(
        &config.game_name,
        "save_autosave_corrupt.json",
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(&quarantine).unwrap(),
        "{not valid json"
    );

    let _ = std::fs::remove_file(&quarantine);
    let _ = std::fs::remove_dir(path.parent().unwrap());
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn campaign_round_trips_atomically_with_a_non_ascii_profile_path() {
    let data = crate::data::GameData::load().unwrap();
    let mut config = data.config.clone();
    config.game_name = format!("stellar_legacy_雪_save_test_{}", std::process::id());
    let sim = SimState::new_campaign(
        &data,
        "preservers",
        0x5A7E,
        &crate::state::sim::founding_faction_ids(&data),
    );

    save_campaign(&config, &sim).unwrap();
    let loaded = load_campaign(&config).unwrap();
    assert_eq!(loaded.seed, sim.seed);
    assert_eq!(loaded.dynasty.generation, sim.dynasty.generation);
    assert_eq!(loaded.dynasty.members.len(), sim.dynasty.members.len());

    let path =
        macroquad_toolkit::persistence::get_app_data_path(&config.game_name, "save_autosave.json")
            .unwrap();
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
