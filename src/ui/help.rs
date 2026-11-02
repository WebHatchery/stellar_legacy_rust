//! HELP // CONTROLS overlay: a terminal-style touch legend, opened by F2 or by
//! the HELP button in the chrome row. Read-only — returns true on the frame the
//! player asks to close it.

use crate::ui::reading::flow_text;
use crate::ui::{logical_height, logical_width, term, term_button, term_panel};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, occlude};

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
        logical_width(),
        logical_height(),
        Color::new(0.0, 0.0, 0.0, 0.8),
    );
    occlude(Rect::new(0.0, 0.0, logical_width(), logical_height()));

    let panel = Rect::new((logical_width() - 1240.0) / 2.0, 90.0, 1240.0, 900.0);
    term_panel(panel, Some("HELP // IDENTITY & CONTROLS"));
    let content = Rect::new(
        panel.x + 48.0,
        panel.y + 72.0,
        panel.w - 96.0,
        panel.h - 112.0,
    );
    draw_help_sections(content, version);
    draw_help_actions(content, pointer)
}

const HEADING: f32 = 26.0;
const BODY: f32 = 20.0;

fn heading(text: &str, x: f32, top: f32) -> f32 {
    draw_ui_text_ex(
        text,
        x,
        top + HEADING,
        TextStyle::new(HEADING, term::accent()).params(),
    );
    top + HEADING + 14.0
}

fn draw_help_sections(content: Rect, version: &str) {
    // Two reading columns: who the Custodian is on the left, how to act on the
    // right, so the 1080p panel is filled by prose at a desk-readable size.
    let gutter = 56.0;
    let column = (content.w - gutter) / 2.0;
    let left = content.x;
    let right = content.x + column + gutter;

    let mut y = heading("WHO AM I?", left, content.y);
    y = flow_text(
        "You are the Custodian: the persistent intelligence aboard this generation ship. Captains age, councils change, and the Custodian carries the ship's recorded promises across the centuries.",
        left,
        y,
        column,
        BODY,
        term::primary(),
    );
    y = heading("WHO DECIDES?", left, y + 32.0);
    flow_text(
        "The Custodian executes routine operations under its standing mandate. The serving captain can object to a strategic posture when its cost conflicts with the ship's condition; the review always offers a current policy or a legal compromise. Human messages are attributed in the log.",
        left,
        y,
        column,
        BODY,
        term::primary(),
    );

    let mut y = heading("TOUCH CONTROLS", right, content.y);
    for (key, desc) in CONTROLS {
        draw_ui_text_ex(
            key,
            right,
            y + BODY,
            TextStyle::new(BODY, term::accent()).params(),
        );
        y = flow_text(desc, right + 150.0, y, column - 150.0, BODY, term::dim()) + 14.0;
    }
    flow_text(
        "Mouse or finger works everywhere. Drag a list to scroll it. Tap REVIEW MANDATE in Voyage / Contract for the authority rules. Keyboard shortcuts are optional.",
        right,
        y + 18.0,
        column,
        BODY,
        term::faint(),
    );

    let footer = content.bottom() - 150.0;
    let y = flow_text(
        &format!("VERSION {version}  //  LOCAL SAVES  //  NO TELEMETRY"),
        left,
        footer,
        content.w,
        18.0,
        term::faint(),
    );
    flow_text(
        "Windows: %LOCALAPPDATA%\\stellar_legacy  (includes crash_log.txt)",
        left,
        y + 4.0,
        content.w,
        18.0,
        term::faint(),
    );
}

fn draw_help_actions(content: Rect, pointer: Pointer) -> Option<HelpAction> {
    let button_y = content.bottom() - 56.0;
    let half = (content.w - 24.0) / 2.0;
    if term_button(
        Rect::new(content.x, button_y, half, 56.0),
        "OPEN SAVE FOLDER",
        true,
        pointer,
    ) {
        return Some(HelpAction::OpenSaveFolder);
    }
    if term_button(
        Rect::new(content.x + half + 24.0, button_y, half, 56.0),
        "CLOSE",
        true,
        pointer,
    ) {
        return Some(HelpAction::Close);
    }
    None
}
