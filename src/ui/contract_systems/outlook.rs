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

fn mission_time(months: u32) -> String {
    match (months / 12, months % 12) {
        (0, 0) => "NOW".to_owned(),
        (0, months) => format!("{months}m"),
        (years, 0) => format!("{years}y"),
        (years, months) => format!("{years}y {months}m"),
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
        rect.y + 18.0,
        TextStyle::new(13.0, term::primary()).params(),
    );
    draw_ui_text_ex(
        ctx.sim.command_posture.description(),
        rect.x + 10.0,
        rect.y + 38.0,
        TextStyle::new(11.0, term::dim()).params(),
    );
    draw_ui_text_ex(
        &format!(
            "WORK {:.0}% · EVENTS {:.0}% · FUEL {:.0}%",
            crate::simulation::command::objective_factor(ctx.sim.command_posture) * 100.0,
            crate::simulation::command::event_chance_factor(ctx.sim.command_posture) * 100.0,
            crate::simulation::command::fuel_burn_factor(ctx.sim.command_posture) * 100.0,
        ),
        rect.x + 10.0,
        rect.y + 58.0,
        TextStyle::new(11.0, term::accent()).params(),
    );
    let can_change = crate::simulation::command::posture_change_allowed(ctx.sim);
    if !can_change {
        draw_ui_text_ex(
            &format!(
                "REVIEW LOCKED · NEXT COUNCIL IN {}M",
                ctx.sim
                    .command_posture_locked_until
                    .saturating_sub(ctx.sim.month_clock)
            ),
            rect.x + 10.0,
            rect.y + 75.0,
            TextStyle::new(10.0, term::alert()).params(),
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
        let label = if active {
            format!("[ {} ]", posture.label())
        } else {
            posture.label().to_owned()
        };
        if term_button(button, &label, can_change || active, pointer) && !active && can_change {
            actions.push(UiAction::SetPosture(posture));
        }
    }
}

fn draw_campaign_outlook(contract: &ActiveContract, rect: Rect) {
    draw_surface(
        rect,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    draw_ui_text_ex(
        "CAMPAIGN OUTLOOK · EXACT EVENT WITHHELD",
        rect.x + 10.0,
        rect.y + 18.0,
        TextStyle::new(12.0, term::primary()).params(),
    );
    let upcoming: Vec<_> = contract
        .beats
        .iter()
        .filter(|beat| !beat.fired && beat.month_clock > contract.months_elapsed)
        .take(3)
        .collect();
    if upcoming.is_empty() {
        draw_ui_text_ex(
            "No major scheduled beats remain on the current heading.",
            rect.x + 10.0,
            rect.y + 44.0,
            TextStyle::new(12.0, term::dim()).params(),
        );
        return;
    }
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
            rect.y + 43.0 + index as f32 * 22.0,
            TextStyle::new(12.0, term::dim()).params(),
        );
    }
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
    let content = area.inset(20.0);
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
    let next_phase = contract
        .next_phase_eta()
        .map(|(phase, eta)| {
            format!(
                "{} — in {}",
                phase.label(),
                mission_time(eta).to_lowercase()
            )
        })
        .unwrap_or_else(|| "Final leg — no further change".to_owned());
    let next_milestone = contract
        .next_milestone_eta()
        .map(|(milestone, eta)| {
            format!(
                "{} — in {}",
                milestone.name,
                mission_time(eta).to_lowercase()
            )
        })
        .unwrap_or_else(|| "All authored milestones reached".to_owned());
    let (clock_status, clock_stalled) = super::mission_clock_status(ctx.sim, ctx.data);
    draw_ui_text_ex(
        "MISSION CLOCK STATUS",
        content.x,
        content.y + 20.0,
        TextStyle::new(14.0, term::primary()).params(),
    );
    draw_ui_text_ex(
        &clock_status,
        content.x,
        content.y + 41.0,
        TextStyle::new(
            13.0,
            if clock_stalled {
                term::alert()
            } else {
                term::accent()
            },
        )
        .params(),
    );
    let route = format!(
        "ORIGIN  Home Berth — departed\nOPERATION SITE  {operation}\nOBJECTIVE SYSTEM  {objective_system}\nCURRENT PHASE  {}\nNEXT PHASE  {next_phase}\nNEXT MILESTONE  {next_milestone}\nHOME BERTH  {} remaining\nFuel stalls extend calendar time.",
        contract.phase.label().to_uppercase(),
        mission_time(contract.mission_months_remaining()).to_lowercase()
    );
    draw_text_block(
        &route,
        content.x,
        content.y + 68.0,
        content.w,
        124.0,
        12.0,
        3.0,
        term::dim(),
    );
    draw_campaign_outlook(
        contract,
        Rect::new(content.x, content.y + 202.0, content.w, 102.0),
    );
    draw_posture(
        ctx,
        Rect::new(content.x, content.bottom() - 142.0, content.w, 132.0),
        pointer,
        actions,
    );
}
