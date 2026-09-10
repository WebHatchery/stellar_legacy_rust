//! The between-voyages recovery review attached to a sealed report.

use super::*;
use crate::simulation::homecoming;
use crate::state::sim::debrief::VoyageDebrief;
use crate::state::sim::{HomecomingChoice, HomecomingRecovery};
use crate::ui::selection_marker;
use macroquad_toolkit::ui::draw_text_block;

pub(super) fn draw(
    ctx: &GameplayCtx<'_>,
    report: &VoyageDebrief,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(recovery) = report.recovery.as_ref() else {
        return;
    };
    term_panel(
        area,
        Some(&format!(
            "RECOVERY REVIEW · {} · choose one",
            recovery.focus.label()
        )),
    );
    draw_text_block(
        &format!("{}\nTarget: {}", recovery.situation, recovery.target_label),
        area.x + 14.0,
        area.y + 40.0,
        area.w - 28.0,
        42.0,
        12.0,
        4.0,
        term::dim(),
    );

    let top = area.y + 88.0;
    let gap = 8.0;
    let card_w = (area.w - gap * 3.0) / 4.0;
    let card_h = (area.bottom() - top - 8.0).max(48.0);
    for (index, choice) in HomecomingChoice::ALL.into_iter().enumerate() {
        let card = Rect::new(area.x + index as f32 * (card_w + gap), top, card_w, card_h);
        let available = homecoming::choice_available(ctx.sim, ctx.data, choice);
        draw_card(ctx, recovery, choice, card, available, pointer, actions);
    }
}

fn draw_card(
    ctx: &GameplayCtx<'_>,
    recovery: &HomecomingRecovery,
    choice: HomecomingChoice,
    card: Rect,
    available: bool,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    term_panel(card, None);
    draw_ui_text_ex(
        choice.label(),
        card.x + 10.0,
        card.y + 18.0,
        TextStyle::new(
            13.0,
            if available {
                term::primary()
            } else {
                term::faint()
            },
        )
        .params(),
    );
    draw_text_block(
        choice.description(),
        card.x + 10.0,
        card.y + 34.0,
        card.w - 20.0,
        (card.h - 86.0).max(24.0),
        11.0,
        4.0,
        term::dim(),
    );
    let cost = homecoming::choice_cost(ctx.sim, ctx.data, choice);
    let cost_text = if choice == HomecomingChoice::Defer {
        "No immediate cost".to_owned()
    } else {
        format_cost(cost)
    };
    draw_ui_text_ex(
        &cost_text,
        card.x + 10.0,
        card.bottom() - 58.0,
        TextStyle::new(11.0, term::accent()).params(),
    );
    let label = if available { "COMMIT" } else { "UNAVAILABLE" };
    let button = Rect::new(card.x + 8.0, card.bottom() - 42.0, card.w - 16.0, 34.0);
    if term_button(button, label, available, pointer) {
        actions.push(UiAction::ChooseHomecomingRecovery(choice));
    }
    if recovery.choice == Some(choice) {
        selection_marker(card);
    }
}

fn format_cost(cost: crate::data::ResourceDelta) -> String {
    let mut parts = Vec::new();
    if cost.credits != 0 {
        parts.push(format!("{} cr", cost.credits.abs()));
    }
    if cost.influence != 0 {
        parts.push(format!("{} influence", cost.influence.abs()));
    }
    if parts.is_empty() {
        "No material cost".to_owned()
    } else {
        format!("Cost: {}", parts.join(" · "))
    }
}
