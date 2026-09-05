//! Action-driven guidance occupying the header, leaving the game interactive.
use crate::ui::{term, term_button, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex};

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let steps = &ctx.data.config.tutorial.guided_steps;
    let Some(step) = steps.get(ctx.sim.tutorial_step) else {
        return;
    };
    let panel = Rect::new(16.0, 12.0, 880.0, 58.0);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, term::panel());
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, term::accent());
    draw_ui_text_ex(
        &format!(
            "CUSTODIAN GUIDE {}/{} - {}",
            ctx.sim.tutorial_step + 1,
            steps.len(),
            step.label.to_uppercase()
        ),
        28.0,
        29.0,
        TextStyle::new(13.0, term::accent()).params(),
    );
    draw_text_block(
        &step.tip,
        28.0,
        34.0,
        735.0,
        33.0,
        12.0,
        2.0,
        term::primary(),
    );
    let last = ctx.sim.tutorial_step + 1 == steps.len();
    if term_button(
        Rect::new(778.0, 19.0, 108.0, 44.0),
        if last { "FINISH" } else { "SKIP" },
        true,
        pointer,
    ) {
        actions.push(if last {
            UiAction::NextTutorial
        } else {
            UiAction::SkipTutorial
        });
    }
}
