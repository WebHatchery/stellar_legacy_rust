//! CRT display-settings overlay — the monitor's on-screen "display" menu.
//! Reachable from any screen (F1). Returns [`DisplayAction`] intents the game
//! applies to its `DisplaySettings`; it never mutates state here.

use crate::data::events::EventCategory;
use crate::settings::{DisplaySettings, Phosphor};
use crate::state::sim::DelegationSettings;
use crate::ui::{term, term_button, term_panel, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, occlude};

/// A change the display overlay is requesting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayAction {
    ToggleCrt,
    ToggleScanlines,
    ToggleFlicker,
    SetPhosphor(Phosphor),
    AdjustAudio(f32),
    SetUiScale(f32),
    ToggleAmbience,
    ToggleTutorial,
    /// Flip whether this category is delegated by default in new voyages.
    ToggleDelegationDefault(EventCategory),
    Close,
}

pub fn draw(
    display: &DisplaySettings,
    delegation: &DelegationSettings,
    pointer: Pointer,
) -> Vec<DisplayAction> {
    let mut actions = Vec::new();

    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.8),
    );
    occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));

    let panel = Rect::new(LOGICAL_WIDTH / 2.0 - 470.0, 20.0, 940.0, 680.0);
    term_panel(panel, Some("DISPLAY // CRT MONITOR"));
    let content = Rect::new(panel.x + 28.0, panel.y + 28.0, 420.0, panel.h - 56.0);
    let mut y = content.y + 30.0;

    // On/off rows: label left, a single toggle button right.
    toggle_row(
        content.x,
        y,
        content.w,
        "CRT EFFECT",
        display.crt_enabled,
        pointer,
        DisplayAction::ToggleCrt,
        &mut actions,
    );
    y += 52.0;
    toggle_row(
        content.x,
        y,
        content.w,
        "SCANLINES",
        display.scanlines,
        pointer,
        DisplayAction::ToggleScanlines,
        &mut actions,
    );
    y += 52.0;
    toggle_row(
        content.x,
        y,
        content.w,
        "FLICKER",
        display.flicker,
        pointer,
        DisplayAction::ToggleFlicker,
        &mut actions,
    );
    y += 52.0;

    draw_ui_text_ex(
        "COLOR SCHEME",
        content.x,
        y + 18.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    y += 28.0;
    let gap = 8.0;
    let bw = (content.w - gap * 2.0) / 3.0;
    for (index, scheme) in Phosphor::ALL.into_iter().enumerate() {
        if choice_button(
            Rect::new(content.x + index as f32 * (bw + gap), y, bw, 44.0),
            &scheme.label().to_uppercase(),
            display.phosphor == scheme,
            pointer,
        ) {
            actions.push(DisplayAction::SetPhosphor(scheme));
        }
    }
    y += 62.0;

    draw_ui_text_ex(
        "AUDIO MIX",
        content.x,
        y + 22.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    let volume_label = format!("{:.0}%", display.audio_volume * 100.0);
    if choice_button(
        Rect::new(content.right() - 202.0, y, 54.0, 44.0),
        "−",
        false,
        pointer,
    ) {
        actions.push(DisplayAction::AdjustAudio(-0.1));
    }
    draw_text_centered_in_box_ex(
        &volume_label,
        content.right() - 142.0,
        y,
        70.0,
        44.0,
        TextStyle::new(15.0, term::accent()),
    );
    if choice_button(
        Rect::new(content.right() - 66.0, y, 54.0, 44.0),
        "+",
        false,
        pointer,
    ) {
        actions.push(DisplayAction::AdjustAudio(0.1));
    }
    y += 50.0;
    toggle_row(
        content.x,
        y,
        content.w,
        "UNDERWAY AMBIENCE",
        display.ambience,
        pointer,
        DisplayAction::ToggleAmbience,
        &mut actions,
    );
    y += 48.0;
    toggle_row(
        content.x,
        y,
        content.w,
        "GUIDED TUTORIAL",
        display.tutorial_enabled,
        pointer,
        DisplayAction::ToggleTutorial,
        &mut actions,
    );
    y = content.y + 30.0;
    // Delegation defaults: which council categories auto-resolve in new voyages.
    draw_ui_text_ex(
        "DELEGATION DEFAULTS // NEW VOYAGES",
        panel.x + 498.0,
        y,
        TextStyle::new(14.0, term::primary()).params(),
    );
    y += 24.0;
    for category in EventCategory::ALL {
        let delegated = delegation.is_delegated(category);
        draw_ui_text_ex(
            &category.label().to_uppercase(),
            panel.x + 498.0,
            y + 21.0,
            TextStyle::new(15.0, term::dim()).params(),
        );
        let bw = 120.0;
        if choice_button(
            Rect::new(panel.right() - 28.0 - bw, y, bw, 44.0),
            if delegated { "DELEGATED" } else { "COUNCIL" },
            delegated,
            pointer,
        ) {
            actions.push(DisplayAction::ToggleDelegationDefault(category));
        }
        y += 48.0;
    }
    draw_ui_text_ex(
        &format!("UI SCALE · {:.0}%", display.ui_scale * 100.0),
        panel.x + 498.0,
        y + 30.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    for (index, (label, scale)) in [
        ("Smaller", display.ui_scale - 0.05),
        ("Reset", 1.0),
        ("Larger", display.ui_scale + 0.05),
    ]
    .into_iter()
    .enumerate()
    {
        if term_button(
            Rect::new(
                panel.x + 498.0 + index as f32 * 134.0,
                y + 44.0,
                126.0,
                44.0,
            ),
            label,
            true,
            pointer,
        ) {
            actions.push(DisplayAction::SetUiScale(scale));
        }
    }
    draw_ui_text_ex(
        "Adjusted scales use the scrolling layout.",
        content.x,
        content.bottom() - 54.0,
        TextStyle::new(13.0, term::faint()).params(),
    );

    if term_button(
        Rect::new(content.x, content.bottom() - 44.0, content.w, 44.0),
        "CLOSE",
        true,
        pointer,
    ) {
        actions.push(DisplayAction::Close);
    }

    actions
}

#[allow(clippy::too_many_arguments)]
fn toggle_row(
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    on: bool,
    pointer: Pointer,
    action: DisplayAction,
    actions: &mut Vec<DisplayAction>,
) {
    draw_ui_text_ex(
        label,
        x,
        y + 22.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
    let rect = Rect::new(x + w - 92.0, y, 92.0, 44.0);
    if choice_button(rect, if on { "ON" } else { "OFF" }, on, pointer) {
        actions.push(action);
    }
}

/// A button whose fill/border brightens when it represents the active choice.
fn choice_button(rect: Rect, label: &str, active: bool, pointer: Pointer) -> bool {
    let hit = touch_area(rect);
    note_neighbour(rect);
    note_target(label, rect);
    let hovered = pointer.hovering_over(hit) || pointer.pressing(hit);
    let fill = if active {
        term::surface_active()
    } else if hovered {
        term::surface_hover()
    } else {
        term::surface_inset()
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(fill).with_border(
            1.0,
            if active {
                term::primary()
            } else {
                term::faint()
            },
        ),
    );
    draw_text_centered_in_box_ex(
        label,
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        TextStyle::new(15.0, if active { term::accent() } else { term::dim() }),
    );
    pointer.released_on(hit)
}
