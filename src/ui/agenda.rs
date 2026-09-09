//! The Custodian Agenda: readiness, persistent aftermath, and ship work.
//!
//! This is intentionally a command board rather than a crafting inventory.
//! Queueing is free, starting charges the authored budget, and every project
//! action is returned to the game dispatcher as a touch-safe intent.

use crate::data::projects::ProjectTarget;
use crate::simulation::{projects as project_sim, readiness};
use crate::state::sim::{ProjectAmounts, ProjectStatus};
use crate::ui::{term, term_button, term_panel, GameplayCtx, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_block, draw_ui_text_ex, is_fully_visible, Pointer};

mod readiness_panel;
pub(crate) mod review;
use readiness_panel::draw_readiness;

const GUTTER: f32 = 14.0;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if let Some(sequence_id) = ctx.project_cancel_confirm.get() {
        review::draw(ctx, sequence_id, pointer, actions);
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
        review::draw(ctx, sequence_id, pointer, actions);
    }
}

mod work_board;
use work_board::draw_work_board;

pub(crate) struct CatalogueChoice {
    pub(crate) project_id: String,
    pub(crate) target_id: Option<String>,
    pub(crate) eligible: bool,
    pub(crate) reason: String,
    pub(crate) existing_job: Option<u64>,
}

pub(crate) fn catalogue_choices(ctx: &GameplayCtx<'_>) -> Vec<CatalogueChoice> {
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
                project_sim::queue_check(ctx.sim, ctx.data, definition, target_id.as_deref());
            let existing_job = project_sim::live_job(ctx.sim, &project_id, target_id.as_deref())
                .map(|job| job.sequence_id);
            choices.push(CatalogueChoice {
                project_id: project_id.clone(),
                target_id,
                eligible: check.eligible,
                reason: if existing_job.is_some() {
                    "Already on the Agenda. Review the existing project's progress and options."
                        .to_owned()
                } else {
                    check.reason
                },
                existing_job,
            });
        }
    }
    choices.sort_by(|left, right| {
        right
            .eligible
            .cmp(&left.eligible)
            .then_with(|| {
                right
                    .existing_job
                    .is_some()
                    .cmp(&left.existing_job.is_some())
            })
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
        TextStyle::new(16.0, term::primary()).params(),
    );
    let status = match job.status {
        ProjectStatus::Running => format!(
            "RUNNING · {:.0}%",
            job.progress(definition.duration_months) * 100.0
        ),
        ProjectStatus::Paused | ProjectStatus::Queued => {
            let (position, count) = ctx
                .sim
                .projects
                .waiting_position(job.sequence_id)
                .expect("waiting job");
            format!("WAITING {position}/{count} · {:?}", job.status)
        }
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
        TextStyle::new(14.0, term::dim()).params(),
    );
    let manage = Rect::new(row.x + 10.0, row.y + 76.0, 164.0, 60.0);
    if matches!(
        job.status,
        ProjectStatus::Queued | ProjectStatus::Running | ProjectStatus::Paused
    ) && term_button(manage, "REVIEW PROJECT", true, pointer)
    {
        actions.push(UiAction::PreviewCancelProject(job.sequence_id));
    }
    if let Some((position, count)) = ctx.sim.projects.waiting_position(job.sequence_id) {
        let up = Rect::new(row.x + 184.0, row.y + 76.0, 116.0, 60.0);
        let down = Rect::new(row.x + 310.0, row.y + 76.0, 136.0, 60.0);
        if term_button(up, "MOVE UP", position > 1, pointer) {
            actions.push(UiAction::MoveProject {
                sequence_id: job.sequence_id,
                direction: -1,
            });
        }
        if term_button(down, "MOVE DOWN", position < count, pointer) {
            actions.push(UiAction::MoveProject {
                sequence_id: job.sequence_id,
                direction: 1,
            });
        }
        draw_text_block(
            "Unavailable jobs are skipped. To resume paused work, tap REVIEW PROJECT.",
            row.x + 10.0,
            row.y + 145.0,
            row.w - 20.0,
            38.0,
            12.0,
            3.0,
            term::dim(),
        );
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
        TextStyle::new(16.0, term::primary()).params(),
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
    let button = Rect::new(row.x + 10.0, row.y + 78.0, 264.0, 60.0);
    if term_button(
        button,
        if choice.existing_job.is_some() {
            "Review existing project"
        } else {
            "Queue project"
        },
        choice.eligible || choice.existing_job.is_some(),
        pointer,
    ) {
        if let Some(id) = choice.existing_job {
            actions.push(UiAction::PreviewCancelProject(id));
        } else {
            actions.push(UiAction::QueueProject {
                project_id: choice.project_id.clone(),
                target_id: choice.target_id.clone(),
            });
        }
    }
    if !choice.eligible {
        draw_text_block(
            &choice.reason,
            row.x + 10.0,
            row.y + 145.0,
            row.w - 20.0,
            38.0,
            12.0,
            3.0,
            term::dim(),
        );
    }
}

pub(crate) fn format_cost(amounts: ProjectAmounts) -> String {
    format_amounts(amounts, ["cr", "en", "min", " food", " inf", " parts"])
}

pub(crate) fn format_cost_long(amounts: ProjectAmounts) -> String {
    format_amounts(
        amounts,
        [
            " credits",
            " energy",
            " minerals",
            " food",
            " influence",
            " spare parts",
        ],
    )
}

fn format_amounts(amounts: ProjectAmounts, labels: [&str; 6]) -> String {
    let parts: Vec<_> = amounts
        .values()
        .into_iter()
        .zip(labels)
        .filter(|(value, _)| *value > 0.0)
        .map(|(value, unit)| format!("{value:.2}{unit}"))
        .collect();
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
