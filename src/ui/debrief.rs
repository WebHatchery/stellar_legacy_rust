//! HOMECOMING: the full-screen report a charter comes home with (GDD §5.2).
//!
//! The counterpart to `game_over` — extinction already got a takeover screen
//! and a final readout, while success, the run's actual climax, was two lines
//! pushed into a scrolling log that scrolls away. This screen is where the
//! voyage is paid, scored, and remembered: what the writ paid, how the score
//! was arrived at, which marks were made, what the council decided along the
//! way, and every captain who held the chair between launch and homecoming.
//!
//! Pure view, like every screen here: it reads `sim.debrief` and returns a
//! single [`UiAction::FileReport`] when the player is done reading.

mod columns;
mod report;

use crate::state::sim::debrief::VoyageDebrief;
use crate::ui::{
    term, term_button, term_panel, GameplayCtx, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

/// Column geometry. Three panels under the banner: the report itself, the
/// chain of command, and the voyage log.
const MARGIN: f32 = 18.0;
const FOOTER_H: f32 = 54.0;

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(report) = ctx.sim.debrief.as_ref() else {
        return;
    };
    draw_rectangle(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT, term::bg());
    draw_banner(report);

    for (index, label) in ["Outcome & accounting", "Captains", "Defining moments"]
        .into_iter()
        .enumerate()
    {
        let button = Rect::new(MARGIN + index as f32 * 414.0, 100.0, 400.0, 44.0);
        if term_button(button, label, true, pointer) {
            ctx.presentation.report_page.set(index);
            ctx.presentation.report_scroll.set(ScrollArea::new());
        }
        if ctx.presentation.report_page.get() == index {
            draw_rectangle(
                button.x,
                button.bottom() - 3.0,
                button.w,
                3.0,
                term::primary(),
            );
        }
    }
    let area = Rect::new(
        MARGIN,
        158.0,
        LOGICAL_WIDTH - MARGIN * 2.0,
        LOGICAL_HEIGHT - 158.0 - FOOTER_H - MARGIN,
    );
    match ctx.presentation.report_page.get() {
        1 => columns::draw_commanders(ctx, report, area, pointer),
        _ => report::draw(ctx, report, area, pointer),
    }

    // One way out. Filing the report clears it and returns the ship to the
    // drydock board, where the next charter is chosen.
    let btn = Rect::new(
        LOGICAL_WIDTH / 2.0 - 190.0,
        LOGICAL_HEIGHT - FOOTER_H - 2.0,
        380.0,
        44.0,
    );
    let caret = if blink(get_time() as f32, 2.5) {
        ">"
    } else {
        " "
    };
    let label = if crate::data::contracts::is_demo_build() {
        format!("{caret} COMPLETE THE DEMO")
    } else {
        format!("{caret} FILE THE REPORT")
    };
    if term_button(btn, &label, true, pointer) {
        actions.push(UiAction::FileReport);
    }
}

/// The banner: what came home, and how it went.
fn draw_banner(report: &VoyageDebrief) {
    let band = outcome_tone(&report.outcome);
    draw_text_glow(
        "HOMECOMING",
        LOGICAL_WIDTH / 2.0 - 148.0,
        52.0,
        TextStyle::new(42.0, band),
        0.14,
        3.0,
    );
    let subtitle = format!(
        "// {} · {} //",
        report.contract_name.to_uppercase(),
        report.outcome.to_uppercase()
    );
    // Centred by measurement rather than a magic offset: charter names run from
    // "The Long Tow" to "Founding Charter: Meridian Reach", and a fixed nudge
    // that suits one leaves the other hanging off the edge.
    let width = measure_text(&subtitle, None, 16, 1.0).width;
    draw_ui_text_ex(
        &subtitle,
        (LOGICAL_WIDTH - width) / 2.0,
        78.0,
        TextStyle::new(16.0, term::dim()).params(),
    );
}

/// Warm-red for a defaulted charter, the tube's own primary for anything the
/// ship can be proud of — the same signal language `game_over` uses.
pub(super) fn outcome_tone(outcome: &str) -> Color {
    match outcome.to_lowercase().as_str() {
        "failure" => term::alert(),
        "pyrrhic" => term::dim(),
        _ => term::primary(),
    }
}

/// A panel with its header, returning the inset content rect — every column
/// here opens the same way.
pub(super) fn column_panel(area: Rect, title: &str) -> Rect {
    term_panel(area, Some(title));
    let content = area.inset(18.0);
    Rect::new(content.x, content.y + 26.0, content.w, content.h - 26.0)
}

#[cfg(test)]
mod tests;
