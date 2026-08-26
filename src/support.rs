//! Player-support helpers that bridge the game UI to native platform services.

/// Reveal the per-user folder containing saves, settings, and `crash_log.txt`.
/// The folder is created first so this also works before the first campaign save.
#[cfg(not(target_arch = "wasm32"))]
pub fn reveal_save_folder(game_name: &str) -> Result<(), String> {
    let marker = macroquad_toolkit::persistence::get_app_data_path(game_name, "save_autosave.json")
        .ok_or_else(|| "Windows could not determine the local save folder.".to_owned())?;
    let folder = marker
        .parent()
        .ok_or_else(|| "Windows returned an invalid local save folder.".to_owned())?;
    std::fs::create_dir_all(folder)
        .map_err(|error| format!("Could not create the save folder: {error}"))?;

    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");

    command
        .arg(folder)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open {}: {error}", folder.display()))
}

#[cfg(target_arch = "wasm32")]
pub fn reveal_save_folder(_game_name: &str) -> Result<(), String> {
    Err("Browser saves are managed by the browser and have no Windows folder.".to_owned())
}
