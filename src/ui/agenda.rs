//! The Custodian Agenda: readiness, persistent aftermath, and ship work.
//!
//! This is intentionally a command board rather than a crafting inventory.
//! Queueing is free, starting charges the authored budget, and every project
//! action is returned to the game dispatcher as a touch-safe intent.

use crate::data::projects::ProjectTarget;
use crate::simulation::{projects as project_sim, readiness};
use crate::state::sim::{ProjectAmounts, ProjectStatus};
use crate::ui::{
    spec_line, term, term_button, term_panel, GameplayCtx, UiAction, LOGICAL_HEIGHT, LOGICAL_WIDTH,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{
    draw_text_block, draw_ui_text_ex, is_fully_visible, occlude, Pointer, RectExt,
};

mod readiness_panel;
use readiness_panel::draw_readiness;

const GUTTER: f32 = 14.0;
const ROW_H: f32 = 160.0;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if let Some(sequence_id) = ctx.project_cancel_confirm.get() {
        draw_cancel_preview(ctx, sequence_id, pointer, actions);
        return;
    }
    let left_w = 420.0;
    let left = Rect::new(area.x, area.y, left_w, area.h);
    let right = Rect::new(
        area.x + left_w + 12.0,
        area.y,
        area.w - left_w - 12.0,
        area.h,
    );
    draw_readiness(ctx, left, pointer, actions);
    draw_work_board(ctx, right, pointer, actions);

    if let Some(sequence_id) = ctx.project_cancel_confirm.get() {
        draw_cancel_preview(ctx, sequence_id, pointer, actions);
    }
}

fn draw_work_board(
    ctx: &GameplayCtx<'_>,
    area: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let active = ctx.sim.projects.active_count();
    let waiting = ctx.sim.projects.waiting_count();
    term_panel(
        area,
        Some(&format!(
            "CUSTODIAN AGENDA // {} ACTIVE · {} WAITING",
            active, waiting
        )),
    );
    let view = Rect::new(area.x + 16.0, area.y + 44.0, area.w - 32.0, area.h - 60.0);
    let choices = catalogue_choices(ctx);
    let content_h =
        40.0 + ctx.sim.projects.jobs.len() as f32 * ROW_H + 38.0 + choices.len() as f32 * ROW_H;
    let mut scroll = ctx.agenda_scroll.get();
    scroll.update_at(view, content_h, pointer.position);
    let board_pointer = if scroll.absorbs_press() {
        pointer.suppressed()
    } else {
        pointer
    };
    let mut y = view.y - scroll.offset();
    draw_ui_text_ex(
        "RUNNING & WAITING WORK",
        view.x,
        y + 16.0,
        TextStyle::new(14.0, term::primary()).params(),
    );
    y += 26.0;
    if ctx.sim.projects.jobs.is_empty() {
        draw_ui_text_ex(
            "No projects queued. Start with a useful, eligible choice below.",
            view.x,
            y + 20.0,
            TextStyle::new(12.0, term::dim()).params(),
        );
        y += 42.0;
    } else {
        for job in &ctx.sim.projects.jobs {
            let row = Rect::new(view.x, y, view.w - GUTTER, ROW_H - 6.0);
            if is_fully_visible(row, view) {
                draw_job(ctx, job, row, board_pointer, actions);
            }
            y += ROW_H;
        }
    }
    draw_line(
        view.x,
        y - 5.0,
        view.right() - GUTTER,
        y - 5.0,
        1.0,
        term::faint(),
    );
    draw_ui_text_ex(
        "PROJECT CATALOGUE // QUEUEING COSTS NOTHING",
        view.x,
        y + 16.0,
        TextStyle::new(14.0, term::primary()).params(),
    );
    y += 28.0;
    for choice in choices {
        let row = Rect::new(view.x, y, view.w - GUTTER, ROW_H - 6.0);
        if is_fully_visible(row, view) {
            draw_choice(ctx, &choice, row, board_pointer, actions);
        }
        y += ROW_H;
    }
    scroll.draw_scrollbar_with(
        view,
        content_h,
        term::surface_inset(),
        term::dim(),
        term::primary(),
    );
    ctx.agenda_scroll.set(scroll);
}

struct CatalogueChoice {
    project_id: String,
    target_id: Option<String>,
    eligible: bool,
    reason: String,
}

fn catalogue_choices(ctx: &GameplayCtx<'_>) -> Vec<CatalogueChoice> {
    let mut choices = Vec::new();
    for project_id in crate::data::GameData::sorted_ids(&ctx.data.projects) {
        let Some(definition) = ctx.data.projects.get(&project_id) else {
            continue;
        };
        let targets = match definition.target {
            ProjectTarget::Subsystem => crate::data::GameData::sorted_ids(&ctx.data.subsystems)
                .into_iter()
                .map(Some)
                .collect(),
            ProjectTarget::Agriculture => vec![Some("agriculture".to_owned())],
            ProjectTarget::None | ProjectTarget::Social => vec![None],
        };
        for target_id in targets {
            let check =
                project_sim::eligibility(ctx.sim, ctx.data, definition, target_id.as_deref());
            choices.push(CatalogueChoice {
                project_id: project_id.clone(),
                target_id,
                eligible: check.eligible,
                reason: check.reason,
            });
        }
    }
    choices.sort_by(|left, right| {
        right
            .eligible
            .cmp(&left.eligible)
            .then_with(|| left.project_id.cmp(&right.project_id))
            .then_with(|| left.target_id.cmp(&right.target_id))
    });
    choices
}

fn draw_job(
    ctx: &GameplayCtx<'_>,
    job: &crate::state::sim::ProjectInstance,
    row: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(definition) = project_sim::definition_for(job, ctx.data) else {
        return;
    };
    draw_rectangle(row.x, row.y, row.w, row.h, term::surface_inset());
    draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, term::faint());
    let title = format!(
        "{}{}",
        definition.name,
        job.target_id
            .as_deref()
            .map(|id| format!(" · {}", id.replace('_', " ")))
            .unwrap_or_default()
    );
    draw_ui_text_ex(
        &title,
        row.x + 10.0,
        row.y + 17.0,
        TextStyle::new(13.0, term::primary()).params(),
    );
    let status = match job.status {
        ProjectStatus::Running => format!(
            "RUNNING · {}%",
            job.progress(definition.duration_months) * 100.0
        ),
        ProjectStatus::Paused => format!("PAUSED · {} months", job.paused_months),
        ProjectStatus::Queued => "WAITING FOR A SLOT".to_owned(),
        _ => format!("{:?}", job.status).to_uppercase(),
    };
    draw_ui_text_ex(
        &status,
        row.x + 10.0,
        row.y + 34.0,
        TextStyle::new(
            11.0,
            if job.status == ProjectStatus::Paused {
                term::alert()
            } else {
                term::accent()
            },
        )
        .params(),
    );
    let detail = job
        .pause_reason
        .as_deref()
        .or(job.stop_reason.as_deref())
        .map_or_else(
            || {
                format!(
                    "STAGES {}/{}",
                    job.delivered_stages,
                    definition.stage_count()
                )
            },
            str::to_owned,
        );
    draw_ui_text_ex(
        &detail,
        row.x + 10.0,
        row.y + 51.0,
        TextStyle::new(10.0, term::dim()).params(),
    );
    let manage = Rect::new(row.x + 10.0, row.y + 76.0, 164.0, 60.0);
    if matches!(
        job.status,
        ProjectStatus::Queued | ProjectStatus::Running | ProjectStatus::Paused
    ) && term_button(manage, "REVIEW PROJECT", true, pointer)
    {
        actions.push(UiAction::PreviewCancelProject(job.sequence_id));
    }
    if job.is_waiting() {
        let up = Rect::new(row.x + 184.0, row.y + 76.0, 116.0, 60.0);
        let down = Rect::new(row.x + 310.0, row.y + 76.0, 136.0, 60.0);
        if term_button(up, "MOVE UP", true, pointer) {
            actions.push(UiAction::MoveProject {
                sequence_id: job.sequence_id,
                direction: -1,
            });
        }
        if term_button(down, "MOVE DOWN", true, pointer) {
            actions.push(UiAction::MoveProject {
                sequence_id: job.sequence_id,
                direction: 1,
            });
        }
    }
}

fn draw_choice(
    ctx: &GameplayCtx<'_>,
    choice: &CatalogueChoice,
    row: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(definition) = ctx.data.projects.get(&choice.project_id) else {
        return;
    };
    draw_rectangle(row.x, row.y, row.w, row.h, term::surface_inset());
    draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, term::faint());
    let target = choice
        .target_id
        .as_deref()
        .map(|id| format!(" · {}", id.replace('_', " ")))
        .unwrap_or_default();
    draw_ui_text_ex(
        &format!("{}{}", definition.name, target),
        row.x + 10.0,
        row.y + 17.0,
        TextStyle::new(13.0, term::primary()).params(),
    );
    let cost = format_cost(ProjectAmounts::from_cost(definition.cost.clone()));
    draw_ui_text_ex(
        &format!("{} months / {}", definition.duration_months, cost),
        row.x + 10.0,
        row.y + 35.0,
        TextStyle::new(12.0, term::dim()).params(),
    );
    draw_ui_text_ex(
        &if definition.divisible {
            format!(
                "{} staged deliveries; completed recovery is retained.",
                definition.stage_count()
            )
        } else {
            "Final delivery only; partial work grants no capability.".to_owned()
        },
        row.x + 10.0,
        row.y + 69.0,
        TextStyle::new(11.0, term::dim()).params(),
    );
    let button = Rect::new(row.x + 10.0, row.y + 78.0, 116.0, 60.0);
    if term_button(
        button,
        if choice.eligible { "QUEUE" } else { "BLOCKED" },
        choice.eligible,
        pointer,
    ) && choice.eligible
    {
        actions.push(UiAction::QueueProject {
            project_id: choice.project_id.clone(),
            target_id: choice.target_id.clone(),
        });
    }
    if !choice.eligible {
        draw_ui_text_ex(
            &choice.reason,
            row.x + 10.0,
            row.y + 149.0,
            TextStyle::new(10.0, term::alert()).params(),
        );
    }
}

fn draw_cancel_preview(
    ctx: &GameplayCtx<'_>,
    sequence_id: u64,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(job) = ctx.sim.projects.find(sequence_id) else {
        ctx.project_cancel_confirm.set(None);
        return;
    };
    let Some(definition) = project_sim::definition_for(job, ctx.data) else {
        ctx.project_cancel_confirm.set(None);
        return;
    };
    occlude(Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    let panel = Rect::new(LOGICAL_WIDTH / 2.0 - 460.0, 105.0, 920.0, 520.0);
    term_panel(panel, Some("PROJECT OPTIONS // EXACT ACCOUNTING"));
    let mut content = panel.inset(24.0);
    content.y += 16.0;
    content.h -= 16.0;
    draw_ui_text_ex(
        &definition.name,
        content.x,
        content.y + 18.0,
        TextStyle::new(20.0, term::primary()).params(),
    );
    draw_text_block(
        "Delivered stages remain aboard, but unfinished work will not be restored. The remaining escrow is recoverable at the published cancellation rate; any pause debt is deducted from that recovery.",
        content.x,
        content.y + 30.0,
        content.w,
        48.0,
        13.0,
        3.0,
        term::dim(),
    );
    let refund = project_sim::refund_preview(job, ctx.data);
    let debt = job.restoration_debt;
    let mut y = content.y + 106.0;
    spec_line(
        content.x,
        y,
        content.w,
        "DELIVERED",
        &format!(
            "{}/{} stages",
            job.delivered_stages,
            definition.stage_count()
        ),
        term::accent(),
    );
    y += 25.0;
    spec_line(
        content.x,
        y,
        content.w,
        "REMAINING",
        &format!(
            "{} months",
            definition
                .duration_months
                .saturating_sub(job.elapsed_months)
        ),
        term::primary(),
    );
    y += 25.0;
    spec_line(
        content.x,
        y,
        content.w,
        "REFUND",
        &format_cost(refund),
        term::accent(),
    );
    y += 25.0;
    spec_line(
        content.x,
        y,
        content.w,
        "RESTORATION DEBT",
        &format_cost(debt),
        if debt.nonzero() {
            term::alert()
        } else {
            term::dim()
        },
    );
    y += 25.0;
    let next = if job.paused_months < ctx.data.config.projects.pause_grace_months {
        format!(
            "after {} more paused months",
            ctx.data.config.projects.pause_grace_months + 1 - job.paused_months
        )
    } else {
        "accrues each voyage month".to_owned()
    };
    spec_line(
        content.x,
        y,
        content.w,
        "PAUSE DETERIORATION",
        &format!(
            "{}; cap {:.0}% per unfinished stage",
            next,
            ctx.data.config.projects.pause_debt_cap_fraction * 100.0
        ),
        term::dim(),
    );
    y += 30.0;
    draw_text_block(
        "CONTINUE keeps work and escrow. PAUSE releases the slot with no refund; unfinished stages age only during voyage months. RESUME pays the displayed debt once. Fractional change stays in the saved ledger.",
        content.x, y, content.w, 64.0, 13.0, 3.0, term::dim(),
    );
    let bottom = panel.bottom() - 68.0;
    let keep = Rect::new(content.x, bottom, 180.0, 60.0);
    if term_button(keep, "CONTINUE", true, pointer) {
        actions.push(UiAction::DismissCancelProject);
    }
    let pivot = Rect::new(content.x + 194.0, bottom, 210.0, 60.0);
    if job.status == ProjectStatus::Running && term_button(pivot, "PAUSE PROJECT", true, pointer) {
        actions.push(UiAction::PauseProject(sequence_id));
        actions.push(UiAction::DismissCancelProject);
    }
    if job.status == ProjectStatus::Paused && term_button(pivot, "RESUME PROJECT", true, pointer) {
        actions.push(UiAction::ResumeProject(sequence_id));
        actions.push(UiAction::DismissCancelProject);
    }
    let confirm = Rect::new(content.right() - 235.0, bottom, 235.0, 60.0);
    if term_button(confirm, "CONFIRM CANCEL", true, pointer) {
        actions.push(UiAction::CancelProject(sequence_id));
    }
}

fn format_cost(amounts: ProjectAmounts) -> String {
    let mut parts = Vec::new();
    if amounts.credits > 0.0 {
        parts.push(format!("{:.2}cr", amounts.credits));
    }
    if amounts.energy > 0.0 {
        parts.push(format!("{:.2}en", amounts.energy));
    }
    if amounts.minerals > 0.0 {
        parts.push(format!("{:.2}min", amounts.minerals));
    }
    if amounts.food > 0.0 {
        parts.push(format!("{:.2} food", amounts.food));
    }
    if amounts.influence > 0.0 {
        parts.push(format!("{:.2} inf", amounts.influence));
    }
    if amounts.spare_parts > 0.0 {
        parts.push(format!("{:.2} parts", amounts.spare_parts));
    }
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(" · ")
    }
}

fn band_color(band: readiness::ReadinessBand) -> Color {
    match band {
        readiness::ReadinessBand::Critical => term::alert(),
        readiness::ReadinessBand::Vulnerable => term::primary(),
        readiness::ReadinessBand::Stable | readiness::ReadinessBand::Strong => term::accent(),
    }
}
