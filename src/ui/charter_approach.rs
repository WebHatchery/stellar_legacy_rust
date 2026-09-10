//! PREP controls for the doctrine fixed when a charter launches.

use crate::simulation::approach;
use crate::state::sim::{CharterApproach, SimState};
use crate::ui::{draw_text_block, term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

pub(crate) fn draw(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(template) = ctx
        .sim
        .selected_charter
        .as_ref()
        .and_then(|id| ctx.data.contracts.get(id))
    else {
        return;
    };
    term_panel(area, Some("CHARTER APPROACH // FIXED AT LAUNCH"));
    let content = area.inset(10.0);
    let selected = ctx.sim.selected_charter_approach;
    draw_ui_text_ex(
        &format!(
            "{} · {}",
            selected.label_for(template.objective),
            selected.label()
        ),
        content.x,
        content.y + 18.0,
        TextStyle::new(12.0, term::accent()).params(),
    );
    draw_text_block(
        &format!(
            "{}\n{}",
            selected.description_for(template.objective),
            approach::effect_summary(selected)
        ),
        content.x,
        content.y + 26.0,
        content.w,
        60.0,
        10.0,
        3.0,
        term::dim(),
    );

    let gap = 6.0;
    let button_y = content.bottom() - 44.0;
    if let Some(reason) =
        approach::unavailable_reason(ctx.sim, ctx.data, template, CharterApproach::ProveTheWrit)
    {
        draw_ui_text_ex(
            &format!(
                "LOCKED · {} · {reason}",
                CharterApproach::ProveTheWrit.button_label()
            ),
            content.x,
            button_y - 6.0,
            TextStyle::new(9.0, term::alert()).params(),
        );
    }
    let button_w = (content.w - gap * 2.0) / 3.0;
    for (index, candidate) in CharterApproach::ALL.into_iter().enumerate() {
        let button = Rect::new(
            content.x + index as f32 * (button_w + gap),
            button_y,
            button_w,
            44.0,
        );
        let available = approach::choice_available(ctx.sim, ctx.data, template, candidate);
        let active = selected == candidate;
        if term_button(
            button,
            candidate.button_label(),
            available && !active,
            pointer,
        ) {
            actions.push(UiAction::SetCharterApproach(candidate));
        }
        if active {
            crate::ui::selection_marker(button);
        }
    }
}

pub(crate) fn mobile_summary(
    sim: &SimState,
    objective: crate::data::contracts::ContractObjective,
) -> String {
    let approach = sim.selected_charter_approach;
    format!(
        "{} · {}\n{}\n{}",
        approach.label_for(objective),
        approach.label(),
        approach.description_for(objective),
        approach::effect_summary(approach)
    )
}
