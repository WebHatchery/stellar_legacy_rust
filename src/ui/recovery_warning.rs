//! Blocking life-support recovery review.

use crate::ui::{
    draw_text_block, draw_ui_text_ex, logical_height, logical_width, occlude, term, term_button,
    term_panel, GameplayCtx, Pointer, TextStyle, UiAction,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::RectExt;

pub fn draw(ctx: &GameplayCtx<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    occlude(Rect::new(0.0, 0.0, logical_width(), logical_height()));
    let panel = Rect::new(logical_width() / 2.0 - 370.0, 150.0, 740.0, 410.0);
    term_panel(panel, Some("LIFE SUPPORT // RECOVERY REVIEW REQUIRED"));
    let content = panel.inset(26.0);
    draw_ui_text_ex(
        "THE AIR LINE IS FAILING",
        content.x,
        content.y + 22.0,
        TextStyle::new(24.0, term::alert()).params(),
    );
    draw_text_block(
        &format!(
            "Life support is at {:.0}%. The voyage is paused once so the Custodian can review recovery. Zero air has a grace period of {} simulation months; food and fuel reserves are not instant-loss conditions.",
            ctx.sim.ship.life_support * 100.0,
            ctx.data.config.survival.air_grace_months
        ),
        content.x,
        content.y + 38.0,
        content.w,
        70.0,
        14.0,
        3.0,
        term::dim(),
    );
    draw_ui_text_ex(
        &format!(
            "ZERO-AIR CLOCK: {}/{} months · EMERGENCY: {}",
            ctx.sim.survival.air_zero_months,
            ctx.data.config.survival.air_grace_months,
            if ctx.sim.survival.emergency_used {
                "USED"
            } else {
                "AVAILABLE"
            }
        ),
        content.x,
        content.y + 126.0,
        TextStyle::new(14.0, term::primary()).params(),
    );
    draw_ui_text_ex(
        &format!(
            "STABILISATION COST: {} energy · {} minerals · {} spare parts",
            ctx.data.config.survival.emergency_resource_cost.energy,
            ctx.data.config.survival.emergency_resource_cost.minerals,
            ctx.data.config.survival.emergency_parts_cost
        ),
        content.x,
        content.y + 150.0,
        TextStyle::new(12.0, term::faint()).params(),
    );
    if let Some(notice) = &ctx.sim.survival.migration_notice {
        draw_text_block(
            notice,
            content.x,
            content.y + 184.0,
            content.w,
            72.0,
            13.0,
            3.0,
            term::alert(),
        );
    }
    let y = panel.bottom() - 60.0;
    let review = Rect::new(content.x, y, 210.0, 44.0);
    let rescue = Rect::new(content.x + 224.0, y, 220.0, 44.0);
    let resume = Rect::new(content.right() - 190.0, y, 190.0, 44.0);
    if term_button(review, "REVIEW AGENDA", true, pointer) {
        actions.push(UiAction::ReviewRecovery);
    }
    if term_button(
        rescue,
        "STABILISE AIR",
        !ctx.sim.survival.emergency_used,
        pointer,
    ) {
        actions.push(UiAction::EmergencyStabilise);
    }
    if term_button(resume, "RESUME VOYAGE", true, pointer) {
        actions.push(UiAction::ResumeAfterWarning);
    }
}
