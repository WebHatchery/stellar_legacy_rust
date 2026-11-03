//! Settings page: display, sound, guidance and council defaults. Reachable
//! from the main menu, the in-game cog menu (Esc) and any screen (F1). Returns [`DisplayAction`] intents the game
//! applies to its `DisplaySettings`; it never mutates state here.

use crate::data::events::EventCategory;
use crate::settings::{DisplaySettings, Phosphor};
use crate::state::sim::DelegationSettings;
use crate::ui::{logical_height, logical_width, term, term_button, term_panel, Region};
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
    ToggleAmbience,
    ToggleTutorial,
    ToggleFullscreen,
    /// Flip whether this category is delegated by default in new voyages.
    ToggleDelegationDefault(EventCategory),
    Close,
}

pub fn draw(
    display: &DisplaySettings,
    fullscreen: bool,
    delegation: &DelegationSettings,
    presentation: &crate::ui::presentation::Presentation,
    pointer: Pointer,
) -> Vec<DisplayAction> {
    let bounds = Rect::new(0.0, 0.0, logical_width(), logical_height());
    draw_rectangle(0.0, 0.0, bounds.w, bounds.h, Color::new(0.0, 0.0, 0.0, 0.8));
    occlude(bounds);
    let layout = layout(bounds.w, bounds.h, display.text_scale);
    term_panel(layout.panel, Some("SETTINGS"));
    // The settings panel covers the underlying menu, while the dimmed margins
    // remain visible as orientation. Register the opaque panel for collision
    // and contrast auditing so covered menu labels are not counted as peers.
    let _modal_region = Region::on(layout.panel, term::panel());
    let entries = build_entries(display, delegation, fullscreen);
    draw_entries(entries, layout, presentation, pointer)
}

fn build_entries(
    display: &DisplaySettings,
    delegation: &DelegationSettings,
    fullscreen: bool,
) -> Vec<Entry> {
    let toggle = |label: &str, on: bool, action| {
        Entry::Row(Row {
            label: label.into(),
            choices: vec![(if on { "On" } else { "Off" }.into(), true, on, action)],
        })
    };
    let mut entries = vec![
        Entry::Header("Display"),
        Entry::Row(Row::scale(
            "Text size",
            display.text_scale,
            0.75,
            1.5,
            DisplayAction::SetTextScale,
        )),
        toggle("Fullscreen", fullscreen, DisplayAction::ToggleFullscreen),
        Entry::Row(Row {
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
        }),
        toggle(
            "Title effects",
            display.crt_enabled,
            DisplayAction::ToggleCrt,
        ),
        toggle(
            "Scanlines",
            display.scanlines,
            DisplayAction::ToggleScanlines,
        ),
        toggle("Flicker", display.flicker, DisplayAction::ToggleFlicker),
        Entry::Header("Sound"),
        Entry::Row(Row::scale(
            "Audio volume",
            display.audio_volume,
            0.0,
            1.0,
            DisplayAction::SetAudio,
        )),
        toggle(
            "Underway ambience",
            display.ambience,
            DisplayAction::ToggleAmbience,
        ),
        Entry::Header("Guidance"),
        toggle(
            "Guided tutorial",
            display.tutorial_enabled,
            DisplayAction::ToggleTutorial,
        ),
        Entry::Header("Council defaults for new voyages"),
    ];
    for category in EventCategory::ALL {
        let delegated = delegation.is_delegated(category);
        entries.push(Entry::Row(Row {
            label: category.label().to_owned(),
            choices: vec![(
                if delegated { "Delegated" } else { "Council" }.into(),
                true,
                delegated,
                DisplayAction::ToggleDelegationDefault(category),
            )],
        }));
    }
    entries
}

/// A section heading spans the full width; rows fill the columns beneath it.
enum Entry {
    Header(&'static str),
    Row(Row),
}

/// Content-space rects for every entry, plus the total scroll height.
fn place(entries: &[Entry], layout: &Layout) -> (Vec<Rect>, f32) {
    let width = (layout.view.w - 16.0 - 16.0 * (layout.columns - 1) as f32) / layout.columns as f32;
    let (mut y, mut column) = (0.0, 0usize);
    let mut rects = Vec::with_capacity(entries.len());
    for entry in entries {
        match entry {
            Entry::Header(_) => {
                if column > 0 {
                    y += layout.row_height;
                    column = 0;
                }
                rects.push(Rect::new(0.0, y, layout.view.w - 16.0, HEADER_HEIGHT));
                y += HEADER_HEIGHT;
            }
            Entry::Row(_) => {
                rects.push(Rect::new(
                    column as f32 * (width + 16.0),
                    y,
                    width,
                    layout.row_height - 12.0,
                ));
                column += 1;
                if column == layout.columns {
                    column = 0;
                    y += layout.row_height;
                }
            }
        }
    }
    if column > 0 {
        y += layout.row_height;
    }
    (rects, y)
}

const HEADER_HEIGHT: f32 = 40.0;

fn draw_entries(
    entries: Vec<Entry>,
    layout: Layout,
    presentation: &crate::ui::presentation::Presentation,
    pointer: Pointer,
) -> Vec<DisplayAction> {
    let mut actions = Vec::new();
    let (rects, total) = place(&entries, &layout);
    let mut scroll = presentation.overlay_scroll.get();
    scroll.update_at(layout.view, total, pointer.position);
    let pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    for (entry, offset) in entries.into_iter().zip(rects) {
        let rect = Rect::new(
            layout.view.x + offset.x,
            layout.view.y + offset.y - scroll.offset(),
            offset.w,
            offset.h,
        );
        if !macroquad_toolkit::ui::is_fully_visible(rect, layout.view) {
            continue;
        }
        let row = match entry {
            Entry::Header(title) => {
                draw_section_heading(title, rect, term::accent());
                draw_rectangle(rect.x, rect.bottom() - 6.0, rect.w, 1.0, term::faint());
                continue;
            }
            Entry::Row(row) => row,
        };
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
/// A left-aligned section heading, vertically centred above its rule.
fn draw_section_heading(text: &str, rect: Rect, color: Color) {
    macroquad_toolkit::ui::draw_ui_text_ex(
        text,
        rect.x,
        rect.y + rect.h * 0.5 + 4.0,
        TextStyle::new(18.0, color).params(),
    );
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
