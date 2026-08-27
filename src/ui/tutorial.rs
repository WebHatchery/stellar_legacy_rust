//! Guided first-voyage tutorial overlay. One lesson at a time keeps the
//! player focused on the current command without changing the simulation.

use crate::ui::{
    term, term_button, term_panel, GameplayCtx, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex, occlude, RectExt};

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let steps = &ctx.data.config.tutorial.guided_steps;
    let Some(step) = steps.get(ctx.sim.tutorial_step.min(steps.len().saturating_sub(1))) else {
        return;
    };

    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.76),
    );
    occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    let panel = Rect::new(LOGICAL_WIDTH / 2.0 - 350.0, 92.0, 700.0, 536.0);
    term_panel(panel, Some("CUSTODIAN AI // FIRST VOYAGE"));
    let content = panel.inset(28.0);

    let eye_x = content.x + 46.0;
    let eye_y = content.y + 54.0;
    draw_circle(eye_x, eye_y, 42.0, term::surface_inset());
    draw_circle_lines(eye_x, eye_y, 42.0, 2.0, term::border());
    draw_circle(eye_x, eye_y, 16.0, term::accent());
    draw_ui_text_ex(
        "CUSTODIAN",
        content.x,
        content.y + 112.0,
        TextStyle::new(11.0, term::dim()).params(),
    );

    let text_x = content.x + 120.0;
    draw_ui_text_ex(
        &format!(
            "LESSON {:02} / {:02}",
            ctx.sim.tutorial_step + 1,
            steps.len()
        ),
        text_x,
        content.y + 24.0,
        TextStyle::new(14.0, term::dim()).params(),
    );
    draw_ui_text_ex(
        &step.label.to_uppercase(),
        text_x,
        content.y + 58.0,
        TextStyle::new(25.0, term::primary()).params(),
    );
    draw_text_block(
        &step.tip,
        text_x,
        content.y + 88.0,
        content.w - 120.0,
        180.0,
        16.0,
        4.0,
        term::primary(),
    );
    draw_text_block(
        "NEXT continues the lessons. SKIP TUTORIAL ends them for this voyage. CANCEL closes this panel so you can resume later from SETTINGS.",
        content.x,
        content.y + 250.0,
        content.w,
        72.0,
        13.0,
        4.0,
        term::dim(),
    );

    let button_y = content.bottom() - 44.0;
    let gap = 10.0;
    let skip_w = 172.0;
    let cancel_w = 140.0;
    let next_w = content.w - skip_w - cancel_w - gap * 2.0;
    if term_button(
        Rect::new(content.x, button_y, skip_w, 44.0),
        "SKIP TUTORIAL",
        true,
        pointer,
    ) {
        actions.push(UiAction::SkipTutorial);
    }
    if term_button(
        Rect::new(content.x + skip_w + gap, button_y, cancel_w, 44.0),
        "CANCEL",
        true,
        pointer,
    ) {
        actions.push(UiAction::CancelTutorial);
    }
    if term_button(
        Rect::new(content.right() - next_w, button_y, next_w, 44.0),
        "NEXT",
        true,
        pointer,
    ) {
        actions.push(UiAction::NextTutorial);
    }
}
