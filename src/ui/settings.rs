//! CRT display-settings overlay — the monitor's on-screen "display" menu.
//! Reachable from any screen (F1). Returns [`DisplayAction`] intents the game
//! applies to its `DisplaySettings`; it never mutates state here.

use crate::data::events::EventCategory;
use crate::settings::{DisplaySettings, Phosphor};
use crate::state::sim::DelegationSettings;
use crate::ui::{logical_height, logical_width, term, term_button, term_panel};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::occlude;

/// A change the display overlay is requesting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayAction {
    ToggleCrt,
    ToggleScanlines,
    ToggleFlicker,
    SetPhosphor(Phosphor),
    SetAudio(f32),
    SetTextScale(f32),
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
    presentation: &crate::ui::presentation::Presentation,
    pointer: Pointer,
) -> Vec<DisplayAction> {
    let mut actions = Vec::new();
    let bounds = Rect::new(0.0, 0.0, logical_width(), logical_height());
    draw_rectangle(0.0, 0.0, bounds.w, bounds.h, Color::new(0.0, 0.0, 0.0, 0.8));
    occlude(bounds);
    let layout = layout(bounds.w, bounds.h, display.text_scale);
    term_panel(layout.panel, Some("DISPLAY & SOUND"));
    let mut rows = vec![
        Row::scale(
            "UI scale",
            display.ui_scale,
            0.75,
            2.0,
            DisplayAction::SetUiScale,
        ),
        Row::scale(
            "Text size",
            display.text_scale,
            0.75,
            1.5,
            DisplayAction::SetTextScale,
        ),
        Row {
            label: "Color scheme".into(),
            choices: Phosphor::ALL
                .into_iter()
                .map(|p| {
                    (
                        p.label().to_owned(),
                        true,
                        display.phosphor == p,
                        DisplayAction::SetPhosphor(p),
                    )
                })
                .collect(),
        },
        Row::scale(
            "Audio volume",
            display.audio_volume,
            0.0,
            1.0,
            DisplayAction::SetAudio,
        ),
    ];
    for (label, on, action) in [
        (
            "Title effects",
            display.crt_enabled,
            DisplayAction::ToggleCrt,
        ),
        (
            "Scanlines",
            display.scanlines,
            DisplayAction::ToggleScanlines,
        ),
        ("Flicker", display.flicker, DisplayAction::ToggleFlicker),
        (
            "Underway ambience",
            display.ambience,
            DisplayAction::ToggleAmbience,
        ),
        (
            "Guided tutorial",
            display.tutorial_enabled,
            DisplayAction::ToggleTutorial,
        ),
    ] {
        rows.push(Row {
            label: label.into(),
            choices: vec![(if on { "On" } else { "Off" }.into(), true, on, action)],
        });
    }
    for category in EventCategory::ALL {
        let delegated = delegation.is_delegated(category);
        rows.push(Row {
            label: format!("New voyages · {}", category.label()),
            choices: vec![(
                if delegated { "Delegated" } else { "Council" }.into(),
                true,
                delegated,
                DisplayAction::ToggleDelegationDefault(category),
            )],
        });
    }
    let total = rows.len().div_ceil(layout.columns) as f32 * layout.row_height;
    let mut scroll = presentation.overlay_scroll.get();
    scroll.update_at(layout.view, total, pointer.position);
    let pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    for (index, row) in rows.into_iter().enumerate() {
        let rect = layout.row(index, scroll.offset());
        if !macroquad_toolkit::ui::is_fully_visible(rect, layout.view) {
            continue;
        }
        draw_text_centered_in_box_ex(
            &row.label,
            rect.x,
            rect.y,
            rect.w,
            layout.label_height,
            TextStyle::new(16.0, term::dim()),
        );
        let count = row.choices.len();
        let width = (rect.w - 8.0 * (count - 1) as f32) / count as f32;
        for (i, (label, enabled, selected, action)) in row.choices.into_iter().enumerate() {
            let button = Rect::new(
                rect.x + i as f32 * (width + 8.0),
                rect.y + layout.label_height,
                width,
                44.0,
            );
            if enabled {
                if choice_button(button, &label, selected, pointer) {
                    actions.push(action);
                }
            } else {
                term_button(button, &label, false, pointer);
            }
        }
    }
    scroll.draw_scrollbar(layout.view, total);
    presentation.overlay_scroll.set(scroll);
    if term_button(layout.close, "Close settings", true, pointer) {
        actions.push(DisplayAction::Close);
    }
    actions
}

struct Row {
    label: String,
    choices: Vec<(String, bool, bool, DisplayAction)>,
}
impl Row {
    fn scale(
        label: &str,
        value: f32,
        min: f32,
        max: f32,
        action: fn(f32) -> DisplayAction,
    ) -> Self {
        Self {
            label: format!("{label} · {:.0}%", value * 100.0),
            choices: vec![
                (
                    "Smaller".into(),
                    value > min + 0.001,
                    false,
                    action((value - 0.05).max(min)),
                ),
                (
                    "Reset".into(),
                    true,
                    false,
                    action(if label == "Audio volume" { 0.35 } else { 1.0 }),
                ),
                (
                    "Larger".into(),
                    value < max - 0.001,
                    false,
                    action((value + 0.05).min(max)),
                ),
            ],
        }
    }
}

struct Layout {
    panel: Rect,
    view: Rect,
    close: Rect,
    columns: usize,
    row_height: f32,
    label_height: f32,
}
fn layout(width: f32, height: f32, text_scale: f32) -> Layout {
    let panel = Rect::new(
        (width - (width - 24.0).min(940.0)) * 0.5,
        (height - (height - 24.0).min(680.0)) * 0.5,
        (width - 24.0).min(940.0),
        (height - 24.0).min(680.0),
    );
    let view = Rect::new(
        panel.x + 16.0,
        panel.y + 40.0,
        panel.w - 32.0,
        panel.h - 100.0,
    );
    let label_height = 24.0 * text_scale.max(1.0);
    Layout {
        panel,
        view,
        close: Rect::new(panel.x + 16.0, panel.bottom() - 52.0, panel.w - 32.0, 44.0),
        columns: if panel.w >= 800.0 { 2 } else { 1 },
        row_height: label_height + 56.0,
        label_height,
    }
}
impl Layout {
    fn row(&self, index: usize, scroll: f32) -> Rect {
        let width = (self.view.w - 16.0 - 16.0 * (self.columns - 1) as f32) / self.columns as f32;
        Rect::new(
            self.view.x + (index % self.columns) as f32 * (width + 16.0),
            self.view.y + (index / self.columns) as f32 * self.row_height - scroll,
            width,
            self.row_height - 12.0,
        )
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
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/settings/tests.rs"
    ));
}
