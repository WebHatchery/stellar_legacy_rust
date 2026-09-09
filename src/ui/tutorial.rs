//! Action-driven guidance occupying the header, leaving the game interactive.
use crate::ui::{term, term_button, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex};

struct Guide {
    heading: String,
    tip: String,
    complete: bool,
    needs_navigation: bool,
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
            needs_navigation: matches!(
                lesson.id.as_str(),
                "dashboard" | "people" | "systems" | "launch" | "agenda"
            ),
        }
    } else {
        Guide {
            heading: "Custodian guide complete".into(),
            tip: "You have prepared the ship, queued work, controlled time and answered the council. Tap FINISH to close the guide; Help remains in Utilities.".into(),
            complete: true,
            needs_navigation: false,
        }
    })
}

pub(crate) fn form(ctx: &GameplayCtx<'_>, form: &mut crate::ui::mobile::form::Form) {
    if let Some(guide) = guide(ctx.data, ctx.sim.tutorial_step) {
        form.heading(&guide.heading);
        if guide.needs_navigation && crate::ui::mobile::navigation_is_compact() {
            form.text("Tap Navigate to open the station list, then follow the instruction below.");
        }
        form.text(&guide.tip);
        form.action(
            if guide.complete {
                "FINISH"
            } else {
                "Skip tutorial"
            },
            true,
            if guide.complete {
                UiAction::NextTutorial
            } else {
                UiAction::SkipTutorial
            },
        );
    }
}

pub(crate) fn reading_step(ctx: &GameplayCtx<'_>) -> Option<usize> {
    (ctx.tutorial_enabled && ctx.tutorial_open && !ctx.sim.tutorial_dismissed)
        .then_some(ctx.sim.tutorial_step)
}

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(guide) = guide(ctx.data, ctx.sim.tutorial_step) else {
        return;
    };
    let panel = Rect::new(16.0, 12.0, 880.0, 58.0);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, term::panel());
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, term::accent());
    draw_ui_text_ex(
        &guide.heading.to_uppercase(),
        28.0,
        29.0,
        TextStyle::new(13.0, term::accent()).params(),
    );
    draw_text_block(
        &guide.tip,
        28.0,
        34.0,
        735.0,
        33.0,
        12.0,
        2.0,
        term::primary(),
    );
    if term_button(
        Rect::new(778.0, 19.0, 108.0, 44.0),
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

#[cfg(test)]
mod tests;
