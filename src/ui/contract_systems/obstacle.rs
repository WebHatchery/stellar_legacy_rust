//! The current obstacle on the active voyage. A dry tank on a travel leg stops
//! all progress, so it leads the contract view with why progress stopped and
//! the route to recovery, ahead of the progress track.

use crate::ui::{term, term_button_primary, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex};

/// Height the obstacle block takes from the top of the panel content.
pub(super) const HEIGHT: f32 = 168.0;

pub(super) fn draw_fuel_stall(
    ctx: &GameplayCtx<'_>,
    content: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(content.x, content.y + 16.0, content.w, HEIGHT - 28.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(term::surface_inset()).with_border(2.0, term::alert()),
    );
    draw_ui_text_ex(
        "! FUEL STALL · the drive is dry and the voyage makes no progress",
        rect.x + 16.0,
        rect.y + 32.0,
        TextStyle::new(20.0, term::alert()).params(),
    );
    let scoop = crate::simulation::readiness::forecast(ctx.sim, ctx.data)
        .fuel
        .annual_scoop;
    let recovery = if scoop > 0.0 {
        format!(
            "The scoops recover about {:.1}% of a tank a year; travel resumes once the next burn is aboard. Fuel can be bought only in port.",
            scoop * 100.0
        )
    } else {
        "This drive has no scoops, and fuel can be bought only in port. Recovery depends on the Agenda's fuel readiness and what the voyage brings.".to_owned()
    };
    let button_w = 300.0;
    draw_text_block(
        &recovery,
        rect.x + 16.0,
        rect.y + 46.0,
        rect.w - button_w - 48.0,
        rect.h - 54.0,
        17.0,
        5.0,
        term::dim(),
    );
    let button = Rect::new(
        rect.right() - button_w - 16.0,
        rect.bottom() - 68.0,
        button_w,
        52.0,
    );
    if term_button_primary(button, "Review fuel readiness", true, pointer) {
        actions.push(UiAction::SelectScreen(crate::state::Screen::Agenda));
    }
}
