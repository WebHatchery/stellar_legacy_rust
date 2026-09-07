//! HELP // CONTROLS overlay: a terminal-style touch legend, opened by F2 or by
//! the HELP button in the chrome row. Read-only — returns true on the frame the
//! player asks to close it.

use crate::ui::{draw_text_block, term, term_button, term_panel, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, occlude, RectExt};

const CONTROLS: &[(&str, &str)] = &[
    ("TABS", "Tap Bridge, Ship, People, Voyage or History"),
    ("TIME", "Tap PAUSE or 1x / 2x / 3x in the top right"),
    (
        "DECISIONS",
        "Select a council choice, read its effects, then tap Commit",
    ),
    ("DISPLAY", "Tap Utilities, then Display & sound"),
    ("HELP", "Tap Utilities, then Help"),
    ("CLOSE", "Tap CLOSE or the visible back button"),
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

    let panel = Rect::new(LOGICAL_WIDTH / 2.0 - 380.0, 24.0, 760.0, 672.0);
    term_panel(panel, Some("HELP // IDENTITY & CONTROLS"));
    let content = panel.inset(26.0);
    draw_ui_text_ex(
        "WHO AM I?",
        content.x,
        content.y + 24.0,
        TextStyle::new(15.0, term::accent()).params(),
    );
    draw_text_block(
        "You are the Custodian: the persistent intelligence aboard this generation ship. Captains age, councils change, and the Custodian carries the ship's recorded promises across the centuries.",
        content.x,
        content.y + 34.0,
        content.w,
        58.0,
        13.0,
        3.0,
        term::primary(),
    );
    draw_ui_text_ex(
        "WHO DECIDES?",
        content.x,
        content.y + 108.0,
        TextStyle::new(15.0, term::accent()).params(),
    );
    draw_text_block(
        "The Custodian executes routine operations under its standing mandate. The serving captain can object to a strategic posture when its cost conflicts with the ship's condition; the review always offers a current policy or a legal compromise. Human messages are attributed in the log.",
        content.x,
        content.y + 118.0,
        content.w,
        64.0,
        13.0,
        3.0,
        term::primary(),
    );

    draw_ui_text_ex(
        "TOUCH CONTROLS",
        content.x,
        content.y + 206.0,
        TextStyle::new(15.0, term::accent()).params(),
    );
    for (index, (key, desc)) in CONTROLS.iter().enumerate() {
        let column = index / 4;
        let row = index % 4;
        let x = content.x + column as f32 * 350.0;
        let y = content.y + 224.0 + row as f32 * 30.0;
        draw_ui_text_ex(key, x, y, TextStyle::new(14.0, term::accent()).params());
        draw_text_block(
            desc,
            x + 86.0,
            y - 12.0,
            250.0,
            30.0,
            12.0,
            2.0,
            term::dim(),
        );
    }
    draw_text_block(
        "Mouse or finger works everywhere. Drag a list to scroll it. Tap REVIEW MANDATE in Voyage / Contract for the authority rules. Keyboard shortcuts are optional.",
        content.x,
        content.y + 362.0,
        content.w,
        38.0,
        13.0,
        3.0,
        term::faint(),
    );
    draw_ui_text_ex(
        &format!("VERSION {version}  //  LOCAL SAVES  //  NO TELEMETRY"),
        content.x,
        content.y + 414.0,
        TextStyle::new(12.0, term::faint()).params(),
    );
    draw_ui_text_ex(
        "Windows: %LOCALAPPDATA%\\stellar_legacy  (includes crash_log.txt)",
        content.x,
        content.y + 436.0,
        TextStyle::new(11.0, term::faint()).params(),
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
