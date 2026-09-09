//! One project review for desktop, scaled panels and phones.
use super::*;
use crate::ui::{logical_height, logical_width, mobile::form::Form};
use macroquad_toolkit::ui::{occlude, RectExt};

pub(super) fn draw(ctx: &GameplayCtx<'_>, id: u64, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let width = (logical_width() - 48.0).min(920.0);
    let panel = Rect::new(
        (logical_width() - width) * 0.5,
        128.0,
        width,
        logical_height() - 144.0,
    );
    occlude(panel);
    term_panel(panel, Some("PROJECT REVIEW"));
    let mut view = panel.inset(20.0);
    view.y += 28.0;
    view.h -= 28.0;
    let mut form = Form::new();
    build(ctx, id, &mut form);
    let key = format!(
        "project-review:{id}:{:?}",
        ctx.presentation.project_cancellation.get()
    );
    form.draw(view, ctx.presentation, pointer, &key, actions);
}

pub(crate) fn build(ctx: &GameplayCtx<'_>, id: u64, form: &mut Form) {
    let Some(job) = ctx.sim.projects.find(id) else {
        form.heading("Project no longer available");
        form.action("Back to Agenda", true, UiAction::DismissCancelProject);
        return;
    };
    let Some(definition) = project_sim::definition_for(job, ctx.data) else {
        form.heading("Project definition unavailable");
        form.action("Back to Agenda", true, UiAction::DismissCancelProject);
        return;
    };
    form.heading(&definition.name);
    form.action("Back to Agenda", true, UiAction::DismissCancelProject);
    let status = match job.status {
        ProjectStatus::Queued => "Queued · awaiting its turn and full starting budget",
        ProjectStatus::Running => "Running · using an Agenda slot",
        ProjectStatus::Paused => "Paused · its Agenda slot is available",
        ProjectStatus::Completed => "Completed",
        ProjectStatus::Stopped => "Stopped",
        ProjectStatus::Cancelled => "Cancelled",
    };
    form.text(status);
    if let Some(target) = &job.target_id {
        form.text(&format!("Target: {}", target.replace('_', " ")));
    }
    form.text(&format!(
        "Delivered {} / {} stages · {:.0}% complete\n{} months of work remaining",
        job.delivered_stages,
        definition.stage_count(),
        job.progress(definition.duration_months) * 100.0,
        definition
            .duration_months
            .saturating_sub(job.elapsed_months),
    ));
    if !matches!(
        job.status,
        ProjectStatus::Queued | ProjectStatus::Running | ProjectStatus::Paused
    ) {
        form.text("This project has ended. Delivered improvements remain aboard.");
        return;
    }
    if ctx.presentation.project_cancellation.get() == Some(id) {
        form.heading("Review cancellation");
        form.text(if job.status == ProjectStatus::Queued {
            "This project has not started, so no stores have been charged. Cancelling removes it from the waiting list."
        } else {
            "Cancelling ends unfinished work. Delivered stages remain aboard; spent materials are not returned. Only the recoverable stores shown below are refunded."
        });
        form.text(&format!(
            "Stores returned: {}",
            super::format_cost_long(project_sim::refund_preview(job, ctx.data))
        ));
        form.action("Keep project", true, UiAction::PreviewCancelProject(id));
        form.action("Confirm cancellation", true, UiAction::CancelProject(id));
        return;
    }
    match job.status {
        ProjectStatus::Running => {
            form.text("Pausing releases the slot and keeps the project's escrow. Unfinished stages can accumulate restoration debt during voyage months.");
            let check = project_sim::pause_check(ctx.sim, ctx.data, id);
            if let Err(reason) = &check {
                form.text(reason);
            }
            form.action("Pause project", check.is_ok(), UiAction::PauseProject(id));
        }
        ProjectStatus::Paused => {
            form.text(&format!(
                "Resume cost: {}\nPaused for {} voyage months",
                super::format_cost_long(job.restoration_debt),
                job.paused_months
            ));
            form.text("Resuming pays the current restoration debt once and uses an available slot. It does not charge the original project budget again. Any fractional balance is kept.");
            let check = project_sim::resume_check(ctx.sim, ctx.data, id);
            if let Err(reason) = &check {
                form.text(reason);
            }
            form.action("Resume project", check.is_ok(), UiAction::ResumeProject(id));
        }
        ProjectStatus::Queued => {
            form.text(&format!(
                "Starting budget: {}",
                super::format_cost_long(ProjectAmounts::from_cost(definition.cost.clone()))
            ));
            form.text("No stores have been charged. Work starts on a voyage when a slot, the full budget and the project's requirements are ready.");
        }
        _ => {}
    }
    form.action(
        "Review cancellation",
        true,
        UiAction::ReviewCancelProject(id),
    );
}
