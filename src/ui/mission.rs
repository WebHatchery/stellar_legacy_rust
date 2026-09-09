//! Full charter reading and explicit return-home confirmation.
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, occlude, wrap_text_ex, RectExt};

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
    let detail = Rect::new(area.x, area.y, area.w, area.h - 190.0);
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
        Rect::new(area.x, area.bottom() - 172.0, area.w, 172.0),
        pointer,
        actions,
    );
}

pub(crate) fn build_abort(ctx: &GameplayCtx<'_>, form: &mut crate::ui::mobile::form::Form) {
    form.heading("Return home early?");
    form.action("Keep voyaging", true, UiAction::DismissReturnHome);
    if !crate::simulation::contract::can_return_home(ctx.sim) {
        form.text("An early return is no longer available. Close this review to continue.");
        return;
    }
    let contract = ctx.sim.contract.as_ref().unwrap();
    let index = contract.first_return_index().unwrap();
    let years: u32 = contract
        .phases
        .iter()
        .skip(index)
        .map(|phase| phase.years)
        .sum();
    form.heading(&contract.name);
    form.text(&format!(
        "Objective work stops now. The return leg still takes {years} years."
    ));
    form.text(&format!("Pay is proportional to the objective banked ({:.0}% now; zero if none). Spent stores, losses and promises remain. There is no instant refund or teleport to port.", contract.objective_fraction() * 100.0));
    form.text("The clock waits while you choose. Tap Keep voyaging to close this review, or Confirm return home to turn back. Your selected time speed is preserved.");
    form.action("Confirm return home", true, UiAction::AbortMission);
}

pub fn draw_abort(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let width = crate::ui::logical_width();
    let height = crate::ui::logical_height();
    let shade = Rect::new(0.0, 72.0, width, height - 72.0);
    draw_rectangle(
        shade.x,
        shade.y,
        shade.w,
        shade.h,
        Color::new(0.0, 0.0, 0.0, 0.82),
    );
    occlude(shade);
    let panel_width = (width - 48.0).min(760.0);
    let panel = Rect::new(
        (width - panel_width) * 0.5,
        128.0,
        panel_width,
        height - 152.0,
    );
    term_panel(panel, None);
    let mut form = crate::ui::mobile::form::Form::new();
    build_abort(ctx, &mut form);
    form.draw(
        panel.inset(20.0),
        ctx.presentation,
        pointer,
        "return-home-review",
        actions,
    );
}
