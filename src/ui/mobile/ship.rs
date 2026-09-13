//! Mobile ship destinations and loadout sections.

use super::*;
use crate::simulation::projects;
use crate::state::sim::{ProjectAmounts, ProjectStatus};
mod catalogue;
mod institutions;
mod repairs;
mod salvage;
mod systems;

pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    f.actions(
        vec![
            ("Loadout", UiAction::SelectScreen(Screen::ShipBuilder)),
            ("Systems", UiAction::SelectScreen(Screen::Subsystems)),
            ("Agenda", UiAction::SelectScreen(Screen::Agenda)),
        ],
        match ctx.screen {
            Screen::Subsystems => 1,
            Screen::Agenda => 2,
            _ => 0,
        },
    );
    if ctx.screen == Screen::Agenda {
        agenda(ctx, f, section);
        return;
    }
    if ctx.screen == Screen::Subsystems {
        systems::build(ctx, f, section);
        return;
    }
    build_ship_status(ctx, f);
    let port = ctx.sim.contract.is_none();
    repairs::build(ctx, f);
    if port {
        catalogue::build(ctx, f);
    }
    salvage::build(ctx, f);
}

fn build_ship_status(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Your ship");
    form.vessel(ctx);
    let ship = &ctx.sim.ship;
    form.text(&format!(
        "Hull {:.0}% · Air {:.0}% · Fuel {:.0}%\nSpare parts {}",
        ship.hull_integrity * 100.0,
        ship.life_support * 100.0,
        ship.fuel * 100.0,
        ship.spare_parts
    ));
    form.action(
        "Inspect compartments",
        true,
        UiAction::SelectScreen(Screen::Subsystems),
    );
}

fn agenda(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    let sim = ctx.sim;
    if let Some(id) = ctx.project_cancel_confirm.get() {
        crate::ui::agenda::review::build(ctx, id, f);
        return;
    }
    f.heading(&format!(
        "Agenda · {} active · {} waiting",
        sim.projects.active_count(),
        sim.projects.waiting_count()
    ));
    f.sections(&[
        ("Project queue", ""),
        ("Available projects", "catalogue"),
        ("Readiness", "readiness"),
    ]);
    if section == "readiness" {
        build_readiness(ctx, f);
        return;
    }
    if section == "catalogue" {
        for choice in crate::ui::agenda::catalogue_choices(ctx) {
            crate::ui::agenda::build_choice(ctx, &choice, f);
        }
    } else {
        build_project_queue(ctx, f);
    }
}

fn build_readiness(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let forecast = crate::simulation::readiness::forecast(ctx.sim, ctx.data);
    for row in forecast.rows {
        form.heading(&format!("{} · {}", row.concern, row.band.label()));
        form.text(&format!("{}\n{}", row.evidence, row.trend));
        if let Some(id) = row.recommended_project {
            response(ctx, form, &id, row.recommended_target);
        }
    }
    crate::ui::recovery_warning::build_stabilisation(ctx, form);
    form.heading("Persistent aftermath");
    for issue in &ctx.sim.issues.active {
        form.text(&format!(
            "{} · {} · {}",
            issue.id.replace('_', " "),
            issue.severity.label(),
            issue
                .due_month
                .map_or("No deadline".to_owned(), |m| format!(
                    "Due year {} month {}",
                    m / 12,
                    m % 12 + 1
                ))
        ));
        for id in &issue.recovery_project_ids {
            response(ctx, form, id, Some(issue.target.clone()));
        }
    }
}

fn build_project_queue(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    if sim.projects.jobs.is_empty() {
        form.text("No projects queued. Tap Available projects to choose ship work.");
    } else if sim.projects.waiting_count() > 0 {
        form.text("Queued jobs are checked in waiting-list order; unavailable jobs are skipped. To resume paused work, tap Review project.");
    }
    for job in &sim.projects.jobs {
        let Some(def) = projects::definition_for(job, ctx.data) else {
            continue;
        };
        form.heading(&def.name);
        form.text(&format!(
            "Target: {}",
            crate::ui::agenda::target_label(ctx.data, job.target_id.as_deref())
        ));
        form.text(&format!(
            "{:?} · {:.0}% complete",
            job.status,
            job.progress(def.duration_months) * 100.0
        ));
        if let Some(reason) = job.pause_reason.as_deref().or(job.stop_reason.as_deref()) {
            form.text(reason);
        }
        if matches!(
            job.status,
            ProjectStatus::Running | ProjectStatus::Paused | ProjectStatus::Queued
        ) {
            form.action(
                "Review project",
                true,
                UiAction::PreviewCancelProject(job.sequence_id),
            );
        }
        if let Some((position, count)) = sim.projects.waiting_position(job.sequence_id) {
            form.text(&format!("Waiting position {position} of {count}"));
            form.action(
                "Move up",
                position > 1,
                UiAction::MoveProject {
                    sequence_id: job.sequence_id,
                    direction: -1,
                },
            );
            form.action(
                "Move down",
                position < count,
                UiAction::MoveProject {
                    sequence_id: job.sequence_id,
                    direction: 1,
                },
            );
        }
    }
}

fn response(ctx: &GameplayCtx<'_>, f: &mut Form, id: &str, target: Option<String>) {
    if let Some(def) = ctx.data.projects.get(id) {
        let target = if matches!(
            def.target,
            crate::data::projects::ProjectTarget::None
                | crate::data::projects::ProjectTarget::Social
        ) {
            None
        } else {
            target
        };
        let check = projects::queue_check(ctx.sim, ctx.data, def, target.as_deref());
        let existing = projects::live_job(ctx.sim, id, target.as_deref());
        f.text(&format!(
            "{} · {} months · {}",
            def.name,
            def.duration_months,
            crate::ui::agenda::format_cost_long(ProjectAmounts::from_cost(def.cost.clone()))
        ));
        if let Some(job) = existing {
            f.text("Recovery is already on the Agenda.");
            f.action(
                "Review existing project",
                true,
                UiAction::PreviewCancelProject(job.sequence_id),
            );
            return;
        }
        if !check.eligible {
            f.text(&check.reason);
        }
        f.action(
            "Queue recovery project",
            check.eligible,
            UiAction::QueueProject {
                project_id: id.to_owned(),
                target_id: target,
            },
        );
    }
}
