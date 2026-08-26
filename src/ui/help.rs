//! HELP // CONTROLS overlay: a terminal-style key legend, opened by F2 or by
//! the HELP button in the chrome row. Read-only — returns true on the frame the
//! player asks to close it.

use crate::ui::{term, term_button, term_panel, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, occlude, RectExt};

/// Keyboard shortcuts, every one of which has a control on screen — a tablet
/// has no function keys, so anything reachable only from this list would be
/// unreachable there.
const KEYS: &[(&str, &str)] = &[
    ("1 - 7", "Switch screen tabs"),
    (
        "ON-SCREEN",
        "Pause or choose the displayed 1x / 2x / 3x pace",
    ),
    ("1 - 9", "Choose an option in a council modal"),
    ("F1", "Display & delegation settings (DISPLAY)"),
    ("F2", "This help screen (HELP)"),
    ("F10", "Toggle the CRT effect (in DISPLAY)"),
    ("ESC", "Close an open panel (CLOSE)"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpAction {
    Close,
    OpenSaveFolder,
}

pub fn draw(pointer: Pointer, version: &str) -> Option<HelpAction> {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.8),
    );
    occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    let panel = Rect::new(
        LOGICAL_WIDTH / 2.0 - 280.0,
        LOGICAL_HEIGHT / 2.0 - 260.0,
        560.0,
        520.0,
    );
    term_panel(panel, Some("HELP // CONTROLS"));
    let content = panel.inset(30.0);
    let mut y = content.y + 42.0;

    for (key, desc) in KEYS {
        draw_ui_text_ex(
            key,
            content.x,
            y,
            TextStyle::new(16.0, term::accent()).params(),
        );
        draw_ui_text_ex(
            desc,
            content.x + 190.0,
            y,
            TextStyle::new(15.0, term::dim()).params(),
        );
        y += 32.0;
    }
    y += 8.0;
    draw_ui_text_ex(
        "Mouse or finger works everywhere. Drag a list to scroll it.",
        content.x,
        y,
        TextStyle::new(13.0, term::faint()).params(),
    );
    y += 28.0;
    draw_ui_text_ex(
        &format!("VERSION {version}  //  LOCAL SAVES  //  NO TELEMETRY"),
        content.x,
        y,
        TextStyle::new(13.0, term::faint()).params(),
    );
    y += 22.0;
    draw_ui_text_ex(
        "Windows: %LOCALAPPDATA%\\stellar_legacy  (includes crash_log.txt)",
        content.x,
        y,
        TextStyle::new(12.0, term::faint()).params(),
    );

    let button_y = content.bottom() - 44.0;
    if term_button(
        Rect::new(content.x, button_y, 330.0, 44.0),
        "OPEN SAVE FOLDER",
        true,
        pointer,
    ) {
        return Some(HelpAction::OpenSaveFolder);
    }
    if term_button(
        Rect::new(content.x + 344.0, button_y, content.w - 344.0, 44.0),
        "CLOSE",
        true,
        pointer,
    ) {
        return Some(HelpAction::Close);
    }
    None
}
