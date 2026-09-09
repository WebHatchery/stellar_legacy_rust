//! Desktop panel reflow. Window-sized navigation stays anchored while the
//! document inside the content panel wraps and scrolls at the chosen UI size.
use super::*;

pub fn draw(ctx: &GameplayCtx<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();
    let width = logical_width();
    let pointer = ctx.pointer;
    draw_text_centered_in_box_ex(
        &format!(
            "STELLAR LEGACY · Year {} · Generation {}",
            ctx.sim.year(),
            ctx.sim.dynasty.generation
        ),
        12.0,
        4.0,
        width - 24.0,
        30.0,
        TextStyle::new(18.0, term::primary()),
    );
    time_controls::draw_at(ctx.sim, pointer, &mut actions, 12.0, 42.0);
    if term_button(
        Rect::new(width - 132.0, 42.0, 120.0, 44.0),
        "Utilities",
        true,
        pointer,
    ) {
        actions.push(UiAction::DismissReturnHome);
        actions.push(UiAction::DismissCancelProject);
        ctx.presentation
            .utilities
            .set(!ctx.presentation.utilities.get());
    }
    let blocked = ctx.sim.has_pending_decision()
        || ctx.sim.survival.warning_active
        || ctx.sim.debrief.is_some();
    let step = (width - 24.0) / 5.0;
    for (i, destination) in navigation::Destination::ALL.into_iter().enumerate() {
        let rect = Rect::new(12.0 + i as f32 * step, 94.0, step - 6.0, 44.0);
        if term_button(rect, destination.label(), !blocked, pointer) {
            ctx.presentation.mobile_section.borrow_mut().clear();
            ctx.presentation.utilities.set(false);
            actions.push(UiAction::SelectScreen(
                destination.home(ctx.sim.contract.is_none()),
            ));
        }
        if destination == navigation::Destination::of(ctx.screen)
            && !ctx.presentation.utilities.get()
        {
            selection_marker(rect);
        }
    }
    let panel = Rect::new(12.0, 146.0, width - 24.0, logical_height() - 158.0);
    term_panel(panel, None);
    if ctx.screen == Screen::Dashboard
        && panel.w >= 720.0
        && !blocked
        && !ctx.presentation.utilities.get()
        && !ctx.abort_confirm.get()
        && !(ctx.tutorial_enabled && ctx.tutorial_open && !ctx.sim.tutorial_dismissed)
    {
        draw_bridge(ctx, panel.inset(12.0), &mut actions);
    } else {
        actions.extend(mobile::draw_content(ctx, panel.inset(12.0)));
    }
    actions
}

fn draw_bridge(ctx: &GameplayCtx<'_>, area: Rect, actions: &mut Vec<UiAction>) {
    use mobile::form::Form;
    let subject = Rect::new(area.x, area.y, (area.w - 20.0) * 0.6, area.h);
    let details = Rect::new(
        subject.right() + 20.0,
        area.y,
        area.w - subject.w - 20.0,
        area.h,
    );
    let mut left = Form::new();
    left.heading(
        ctx.sim
            .contract
            .as_ref()
            .map_or("Choose the next voyage", |c| c.name.as_str()),
    );
    left.text(if ctx.sim.contract.is_none() {
        "In port · Review a charter and prepare provisions"
    } else {
        "Underway · Follow the mission and the ship's condition"
    });
    left.vessel(ctx);
    left.text(&format!(
        "Hull {:.0}% · Air {:.0}% · Fuel {:.0}%",
        ctx.sim.ship.hull_integrity * 100.0,
        ctx.sim.ship.life_support * 100.0,
        ctx.sim.ship.fuel * 100.0
    ));
    left.action(
        "Inspect ship & compartments",
        true,
        UiAction::SelectScreen(Screen::ShipBuilder),
    );
    left.draw_scrolled(
        subject,
        ctx.presentation,
        ctx.pointer,
        "bridge-subject",
        actions,
        &ctx.presentation.bridge_subject_scroll,
        &ctx.presentation.bridge_subject_key,
    );
    let mut right = Form::new();
    right.heading("Attention & next action");
    mobile::bridge_details(ctx, &mut right);
    right.draw_scrolled(
        details,
        ctx.presentation,
        ctx.pointer,
        "bridge-details",
        actions,
        &ctx.presentation.bridge_details_scroll,
        &ctx.presentation.bridge_details_key,
    );
}
