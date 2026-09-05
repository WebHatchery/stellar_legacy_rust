//! Full charter reading and explicit return-home confirmation.
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex, occlude, wrap_text_ex, RectExt};

pub fn draw_selected(
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
    let detail = Rect::new(area.x, area.y, area.w, area.h - 150.0);
    term_panel(detail, Some("SELECTED CHARTER // FULL BRIEFING"));
    let content = detail.inset(18.0);
    let view = Rect::new(content.x, content.y + 32.0, content.w, content.h - 98.0);
    let lines = wrap_text_ex(&template.description, view.w - 18.0, None, 16.0);
    let height = lines.len() as f32 * 23.0;
    let mut scroll = ctx.description_scroll.get();
    scroll.update_at(view, height, pointer.position);
    for (i, line) in lines.iter().enumerate() {
        let y = view.y + i as f32 * 23.0 - scroll.offset();
        if y >= view.y && y + 23.0 <= view.bottom() {
            draw_ui_text_ex(
                line,
                view.x,
                y + 17.0,
                TextStyle::new(16.0, term::primary()).params(),
            );
        }
    }
    scroll.draw_scrollbar_with(
        view,
        height,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.description_scroll.set(scroll);
    if term_button(
        Rect::new(content.x, content.bottom() - 44.0, content.w, 44.0),
        "CANCEL SELECTION / CHOOSE ANOTHER",
        true,
        pointer,
    ) {
        actions.push(UiAction::CancelSelection);
    }
    crate::ui::contract_systems::outlook::draw_posture(
        ctx,
        Rect::new(area.x, area.bottom() - 132.0, area.w, 132.0),
        pointer,
        actions,
    );
}

pub fn draw_abort(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(contract) = &ctx.sim.contract else {
        ctx.abort_confirm.set(false);
        return;
    };
    let Some(index) = contract.first_return_index() else {
        ctx.abort_confirm.set(false);
        return;
    };
    let years: u32 = contract
        .phases
        .iter()
        .skip(index)
        .map(|phase| phase.years)
        .sum();
    let panel = Rect::new(300.0, 190.0, 680.0, 330.0);
    draw_rectangle(0.0, 72.0, 1280.0, 648.0, Color::new(0.0, 0.0, 0.0, 0.82));
    occlude(Rect::new(0.0, 72.0, 1280.0, 648.0));
    term_panel(panel, Some("CANCEL MISSION?"));
    draw_text_block(&format!("Turn {} for home?\n\nObjective work stops now. The return leg still takes {} years. Pay is proportional to the objective banked ({:.0}% now; zero if none). Spent stores, losses and promises remain. There is no instant refund or teleport to port.\n\nThe clock waits while you choose.", contract.name, years, contract.objective_fraction() * 100.0), panel.x + 24.0, panel.y + 58.0, panel.w - 48.0, 196.0, 16.0, 4.0, term::primary());
    let y = panel.bottom() - 64.0;
    if term_button(
        Rect::new(panel.x + 24.0, y, 304.0, 44.0),
        "KEEP MISSION",
        true,
        pointer,
    ) {
        ctx.abort_confirm.set(false);
    }
    if term_button(
        Rect::new(panel.x + 340.0, y, 316.0, 44.0),
        "CONFIRM RETURN HOME",
        true,
        pointer,
    ) {
        actions.push(UiAction::AbortMission);
    }
}
