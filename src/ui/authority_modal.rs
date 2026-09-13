//! Blocking presentation for the one bounded human authority review.

use crate::state::sim::{AuthorityChoice, AuthorityReview, CommandPosture};
use crate::ui::{logical_height, logical_width, term, term_button, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex, occlude, RectExt};

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(review) = ctx.sim.authority.pending_review.as_ref() else {
        return;
    };
    draw_rectangle(
        0.0,
        0.0,
        logical_width(),
        logical_height(),
        Color::new(0.0, 0.0, 0.0, 0.78),
    );
    occlude(Rect::new(0.0, 0.0, logical_width(), logical_height()));

    let panel = Rect::new(logical_width() / 2.0 - 350.0, 120.0, 700.0, 480.0);
    draw_surface(
        panel,
        &SurfaceStyle::new(term::panel())
            .with_border(2.0, term::alert())
            .with_header(42.0, term::panel_header())
            .with_header_divider(1.0, term::alert()),
    );
    // The captain's review is an opaque modal over the active contract. Tell the
    // layout audit which surface is in front so covered contract copy is ignored.
    let _modal_region = Region::on(panel, term::panel());
    draw_review_header(ctx, panel);
    let content = panel.inset(24.0);
    draw_review_context(review, content);
    draw_review_choices(review, content, pointer, actions);
}

fn draw_review_header(ctx: &GameplayCtx<'_>, panel: Rect) {
    draw_text_centered_in_box_ex(
        "CAPTAIN REVIEW // MANDATE CHECK",
        panel.x,
        panel.y,
        panel.w,
        42.0,
        TextStyle::new(15.0, term::alert()),
    );
    draw_ui_text_ex(
        &crate::ui::time_controls::fallback_label(ctx.sim.speed, ctx.decision_remaining),
        panel.right() - 178.0,
        panel.y + 27.0,
        TextStyle::new(11.0, term::accent()).params(),
    );
}

fn draw_review_context(review: &AuthorityReview, content: Rect) {
    draw_ui_text_ex(
        &format!(
            "{} OBJECTS · {} PRIORITY",
            review.captain.to_uppercase(),
            review.priority.label()
        ),
        content.x,
        content.y + 42.0,
        TextStyle::new(15.0, term::accent()).params(),
    );
    draw_text_block(
        &review.reason,
        content.x,
        content.y + 58.0,
        content.w,
        56.0,
        14.0,
        4.0,
        term::primary(),
    );
    draw_text_block(
        "The Custodian proposed a policy under its standing mandate. The captain may object; the ship still has a legal next step.",
        content.x,
        content.y + 132.0,
        content.w,
        36.0,
        12.0,
        4.0,
        term::dim(),
    );
}

fn draw_review_choices(
    review: &AuthorityReview,
    content: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    draw_posture_costs(
        content.x,
        content.y + 170.0,
        content.w,
        "PROPOSED",
        review.proposed,
    );
    draw_posture_costs(
        content.x,
        content.y + 202.0,
        content.w,
        "COMPROMISE",
        review.compromise,
    );
    let mut y = content.bottom() - 92.0;
    let button_w = (content.w - 12.0) / 2.0;
    if term_button(
        Rect::new(content.x, y, button_w, 44.0),
        "KEEP CURRENT",
        true,
        pointer,
    ) {
        actions.push(UiAction::ResolveAuthority(AuthorityChoice::KeepCurrent));
    }
    if term_button(
        Rect::new(content.x + button_w + 12.0, y, button_w, 44.0),
        "ACCEPT COMPROMISE",
        true,
        pointer,
    ) {
        actions.push(UiAction::ResolveAuthority(
            AuthorityChoice::AcceptCompromise,
        ));
    }
    y += 52.0;
    let label = if review.emergency_allowed {
        "EMERGENCY OVERRIDE"
    } else {
        "NO EMERGENCY OVERRIDE"
    };
    if term_button(
        Rect::new(content.x, y, content.w, 44.0),
        label,
        review.emergency_allowed,
        pointer,
    ) && review.emergency_allowed
    {
        actions.push(UiAction::ResolveAuthority(
            AuthorityChoice::EmergencyOverride,
        ));
    }
}

fn draw_posture_costs(x: f32, y: f32, width: f32, label: &str, posture: CommandPosture) {
    draw_ui_text_ex(
        &format!(
            "{label} {} · WORK {:.0}% · EVENTS {:.0}% · FUEL {:.0}%",
            posture.label(),
            crate::simulation::command::objective_factor(posture) * 100.0,
            crate::simulation::command::event_chance_factor(posture) * 100.0,
            crate::simulation::command::fuel_burn_factor(posture) * 100.0,
        ),
        x,
        y,
        TextStyle::new(13.0, term::accent()).params(),
    );
    draw_line(x, y + 8.0, x + width, y + 8.0, 1.0, term::faint());
}
