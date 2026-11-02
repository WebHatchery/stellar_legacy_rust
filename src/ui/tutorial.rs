//! Action-driven guidance occupying the header, leaving the game interactive.
use crate::ui::{term, term_button, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

struct Guide {
    heading: String,
    tip: String,
    complete: bool,
}

fn guide(data: &crate::data::GameData, step: usize) -> Option<Guide> {
    let steps = &data.config.tutorial.guided_steps;
    if steps.is_empty() {
        return None;
    }
    Some(if let Some(lesson) = steps.get(step) {
        Guide {
            heading: format!("Guide {}/{} · {}", step + 1, steps.len(), lesson.label),
            tip: lesson.tip.clone(),
            complete: false,
        }
    } else {
        Guide {
            heading: "Custodian guide complete".into(),
            tip: "Ship prepared, work queued, time held and the council answered. Help stays in the cog menu.".into(),
            complete: true,
        }
    })
}

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(guide) = guide(ctx.data, ctx.sim.tutorial_step) else {
        return;
    };
    // The guide takes the whole header band up to the time controls, so its
    // one-line tip reads at body size instead of squeezing beside the readout.
    let panel = Rect::new(16.0, 12.0, 1526.0, 58.0);
    // Panel tone is slightly translucent; back it with solid black so the
    // covered header readout does not ghost through the guide copy.
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, BLACK);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, term::panel());
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, term::accent());
    // The guide occupies the shared header band. Register its opaque surface
    // so the covered resource readout is not audited against guide copy.
    let _modal_region = Region::on(panel, term::panel());
    draw_ui_text_ex(
        &guide.heading.to_uppercase(),
        30.0,
        33.0,
        TextStyle::new(15.0, term::accent()).params(),
    );
    draw_ui_text_ex(
        &guide.tip,
        30.0,
        59.0,
        TextStyle::new(17.0, term::primary()).params(),
    );
    if term_button(
        Rect::new(panel.right() - 124.0, 19.0, 112.0, 44.0),
        if guide.complete { "FINISH" } else { "SKIP" },
        true,
        pointer,
    ) {
        actions.push(if guide.complete {
            UiAction::NextTutorial
        } else {
            UiAction::SkipTutorial
        });
    }
}
