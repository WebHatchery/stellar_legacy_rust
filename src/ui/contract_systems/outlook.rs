//! The active mission's forward-looking command panel.
//!
//! Campaign beats are laid out at launch, so showing the next few here gives
//! the player useful foresight without revealing the exact event or its answer.
//! The same panel owns the posture controls because both are planning tools.

use crate::state::sim::{ActiveContract, CommandPosture};
use crate::ui::{draw_text_block, term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text_ex, RectExt};

pub(super) fn mission_time(months: u32) -> String {
    match (months / 12, months % 12) {
        (0, 0) => "NOW".to_owned(),
        (0, months) => format!("{months}m"),
        (years, 0) => format!("{years}y"),
        (years, months) => format!("{years}y {months}m"),
    }
}

/// The current posture in one line with its effects, for the calm contract
/// view; alternatives open behind Posture options.
fn draw_posture_summary(ctx: &GameplayCtx<'_>, rect: Rect) {
    draw_ui_text_ex(
        &format!("COMMAND POSTURE · {}", ctx.sim.command_posture.label()),
        rect.x,
        rect.y + 20.0,
        TextStyle::new(17.0, term::primary()).params(),
    );
    let effects = crate::simulation::command::effect_summary(ctx.sim.command_posture);
    if !effects.is_empty() {
        draw_ui_text_ex(
            &effects,
            rect.x,
            rect.y + 46.0,
            TextStyle::new(16.0, term::accent()).params(),
        );
    }
}

pub(crate) fn draw_posture(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    draw_surface(
        rect,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    draw_ui_text_ex(
        &format!("COMMAND POSTURE · {}", ctx.sim.command_posture.label()),
        rect.x + 10.0,
        rect.y + 24.0,
        TextStyle::new(15.0, term::primary()).params(),
    );
    // PREP gives the posture room for two description lines; the underway
    // contract panel keeps it to one.
    let desc_h = if rect.h >= 220.0 { 48.0 } else { 24.0 };
    let effects_y = rect.y + 52.0 + desc_h + 18.0;
    draw_text_block(
        ctx.sim.command_posture.description(),
        rect.x + 10.0,
        rect.y + 52.0,
        rect.w - 20.0,
        desc_h,
        17.0,
        4.0,
        term::dim(),
    );
    let effects = crate::simulation::command::effect_summary(ctx.sim.command_posture);
    if !effects.is_empty() {
        draw_ui_text_ex(
            &effects,
            rect.x + 10.0,
            effects_y,
            TextStyle::new(16.0, term::accent()).params(),
        );
    }
    if term_button(
        Rect::new(rect.right() - 210.0, rect.y + 4.0, 200.0, 44.0),
        "REVIEW MANDATE",
        true,
        pointer,
    ) {
        actions.push(UiAction::OpenHelp);
    }
    let can_change = crate::simulation::command::posture_change_allowed(ctx.sim);
    if !can_change {
        draw_ui_text_ex(
            &format!(
                "REVIEW LOCKED · NEXT COUNCIL IN {}M",
                crate::simulation::command::review_wait_months(ctx.sim)
            ),
            rect.x + 10.0,
            effects_y + 22.0,
            TextStyle::new(15.0, term::alert()).params(),
        );
    }
    let gap = 6.0;
    // Keep the policy controls at the touch-first 44px target even in PREP,
    // where this panel is deliberately compact.
    let button_y = rect.bottom() - 50.0;
    let button_w = (rect.w - gap * 2.0) / 3.0;
    for (index, posture) in CommandPosture::ALL.into_iter().enumerate() {
        let button = Rect::new(
            rect.x + index as f32 * (button_w + gap),
            button_y,
            button_w,
            44.0,
        );
        let active = ctx.sim.command_posture == posture;
        if term_button(button, posture.label(), can_change && !active, pointer) {
            actions.push(UiAction::SetPosture(posture));
        }
        if active {
            crate::ui::selection_marker(button);
        }
    }
}

fn draw_campaign_outlook(contract: &ActiveContract, rect: Rect) {
    let upcoming: Vec<_> = contract
        .beats
        .iter()
        .filter(|beat| !beat.fired && beat.month_clock > contract.months_elapsed)
        .take(3)
        .collect();
    // An empty forecast is not news; the panel only appears when it has beats.
    if upcoming.is_empty() {
        return;
    }
    draw_surface(
        rect,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    draw_ui_text_ex(
        "CAMPAIGN OUTLOOK",
        rect.x + 10.0,
        rect.y + 22.0,
        TextStyle::new(16.0, term::primary()).params(),
    );
    for (index, beat) in upcoming.into_iter().enumerate() {
        let eta = beat.month_clock.saturating_sub(contract.months_elapsed);
        draw_ui_text_ex(
            &format!(
                "{}  ·  IN {}  ·  {}",
                index + 1,
                mission_time(eta).to_lowercase(),
                beat.family.to_uppercase()
            ),
            rect.x + 10.0,
            rect.y + 50.0 + index as f32 * 26.0,
            TextStyle::new(15.0, term::dim()).params(),
        );
    }
}

/// The route block; the approach effects line appears only when the approach
/// actually shifts the voyage.
fn route_text(ctx: &GameplayCtx<'_>, contract: &ActiveContract) -> String {
    let template = ctx.data.contracts.get(&contract.template_id);
    let operation = template
        .map(|t| t.operation_site())
        .unwrap_or_else(|| contract.name.clone());
    let objective_system = contract
        .objective_subsystem
        .is_empty()
        .then_some("No single subsystem")
        .or_else(|| {
            ctx.data
                .subsystems
                .get(&contract.objective_subsystem)
                .map(|s| s.name.as_str())
        })
        .unwrap_or(&contract.objective_subsystem);
    // Phase, position, milestones and berth time live on the voyage track.
    let approach_effects = crate::simulation::approach::effect_summary(contract.approach);
    let mut route_lines = vec![
        format!("OPERATION SITE  {operation}"),
        format!("OBJECTIVE SYSTEM  {objective_system}"),
        format!(
            "CHARTER APPROACH  {}",
            contract.approach.label_for(contract.objective)
        ),
    ];
    if !approach_effects.is_empty() {
        route_lines.push(approach_effects);
    }
    route_lines.join("\n")
}

pub(super) fn draw(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(contract) = ctx.sim.contract.as_ref() else {
        return;
    };
    term_panel(area, Some("ROUTE & MISSION"));
    let mut content = area.inset(20.0);
    content.y += 24.0;
    content.h -= 24.0;
    let (clock_status, clock_stalled) = super::mission_clock_status(ctx.sim, ctx.data);
    // The stall itself leads the left panel; here the check is one quiet line.
    draw_ui_text_ex(
        &format!("TRAVEL FUEL · {clock_status}"),
        content.x,
        content.y + 20.0,
        TextStyle::new(
            16.0,
            if clock_stalled {
                term::alert()
            } else {
                term::accent()
            },
        )
        .params(),
    );
    let route = route_text(ctx, contract);
    draw_text_block(
        &route,
        content.x,
        content.y + 52.0,
        content.w,
        130.0,
        16.0,
        5.0,
        term::dim(),
    );
    draw_campaign_outlook(
        contract,
        Rect::new(content.x, content.y + 196.0, content.w, 128.0),
    );
    let options = ctx.presentation.posture_options.get();
    let toggle = Rect::new(
        content.right() - 260.0,
        content.bottom() - 44.0,
        260.0,
        44.0,
    );
    if options {
        draw_posture(
            ctx,
            Rect::new(content.x, content.bottom() - 238.0, content.w, 180.0),
            pointer,
            actions,
        );
    } else {
        draw_posture_summary(
            ctx,
            Rect::new(content.x, content.bottom() - 112.0, content.w, 60.0),
        );
    }
    if term_button(
        toggle,
        if options {
            "Hide posture options"
        } else {
            "Posture options"
        },
        true,
        pointer,
    ) {
        ctx.presentation.posture_options.set(!options);
    }
}
