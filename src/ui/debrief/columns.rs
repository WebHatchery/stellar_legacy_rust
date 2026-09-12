//! The homecoming's three columns: the report (prose, tallies, pay, scorecard
//! and marks), the chain of command, and the voyage log. Split from
//! `debrief.rs` so neither file carries both the page layout and the contents
//! of every panel on it.

use super::column_panel;
use crate::state::sim::debrief::VoyageDebrief;
use crate::ui::{term, GameplayCtx};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, is_fully_visible};

/// Reserved at a scrolling panel's right edge for the scrollbar.
const GUTTER: f32 = 12.0;

/// Every captain who held the chair between launch and homecoming. On a long
/// charter this is the column that makes the generational premise land: the
/// commander who signed the writ is rarely the one who files the report.
pub(super) fn draw_commanders(
    ctx: &GameplayCtx<'_>,
    report: &VoyageDebrief,
    area: Rect,
    pointer: Pointer,
) {
    let content = column_panel(area, "CHAIN OF COMMAND");

    if report.commanders.is_empty() {
        draw_text_block(
            "No command record for this voyage.",
            content.x,
            content.y + 8.0,
            content.w,
            40.0,
            13.0,
            4.0,
            term::dim(),
        );
        return;
    }

    draw_ui_text_ex(
        &format!(
            "{} held the chair",
            plural(report.commanders.len(), "captain", "captains")
        ),
        content.x,
        content.y + 6.0,
        TextStyle::new(13.0, term::faint()).params(),
    );

    let view = Rect::new(
        content.x,
        content.y + 18.0,
        content.w,
        content.bottom() - (content.y + 18.0),
    );
    const STRIDE: f32 = 74.0;
    let content_h = report.commanders.len() as f32 * STRIDE;
    let mut scroll = ctx.debrief_commanders_scroll.get();
    scroll.update_at(view, content_h, pointer.position);

    let mut row_top = view.y - scroll.offset();
    for (i, reign) in report.commanders.iter().enumerate() {
        let row = Rect::new(view.x, row_top, view.w - GUTTER, STRIDE - 6.0);
        row_top += STRIDE;
        // macroquad has no scissor rect, so cull partly-scrolled rows rather
        // than letting them spill past the panel.
        if !is_fully_visible(row, view) {
            continue;
        }
        // The captain who was sitting when the writ was signed is marked, so a
        // long chain still reads as "this began under her".
        let marker = if i == 0 { ">" } else { " " };
        draw_ui_text_ex(
            &format!("{marker} {}", reign.name),
            row.x,
            row.y + 14.0,
            TextStyle::new(15.0, term::primary()).params(),
        );
        let held = reign.years_held(report.ended_year);
        draw_ui_text_ex(
            &format!("  gen {} - {held} yr in the chair", reign.generation),
            row.x,
            row.y + 31.0,
            TextStyle::new(12.0, term::dim()).params(),
        );
        if reign.inherited_obligations > 0 {
            draw_ui_text_ex(
                &format!("  inherited {} active duties", reign.inherited_obligations),
                row.x,
                row.y + 45.0,
                TextStyle::new(11.0, term::accent()).params(),
            );
        }
        if !reign.trait_name.is_empty() {
            draw_ui_text_ex(
                &format!("  {}", reign.trait_name),
                row.x,
                row.y + 61.0,
                TextStyle::new(11.0, term::faint()).params(),
            );
        }
    }

    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.debrief_commanders_scroll.set(scroll);
}

/// "1 captain" / "3 captains" — the count reads as a sentence in the header.
fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/debrief/columns/tests.rs"
    ));
}
