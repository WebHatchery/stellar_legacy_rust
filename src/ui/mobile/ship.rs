use super::*;
use crate::simulation::projects;
use crate::state::sim::{ProjectAmounts, ProjectStatus};
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
    f.heading("Your ship");
    f.vessel(ctx);
    let s = &ctx.sim.ship;
    f.text(&format!(
        "Hull {:.0}% · Air {:.0}% · Fuel {:.0}%\nSpare parts {}",
        s.hull_integrity * 100.0,
        s.life_support * 100.0,
        s.fuel * 100.0,
        s.spare_parts
    ));
    f.action(
        "Inspect compartments",
        true,
        UiAction::SelectScreen(Screen::Subsystems),
    );
    let port = ctx.sim.contract.is_none();
    repairs::build(ctx, f);
    if port {
        for kind in [
            ComponentKind::Hull,
            ComponentKind::Engine,
            ComponentKind::Weapon,
        ] {
            for part in ctx
                .data
                .ship_components
                .list(kind)
                .iter()
                .filter(|p| !p.acquisition.is_mission_only())
            {
                f.heading(&part.name);
                f.text(&part.description);
                f.text(&format!(
                    "Cargo {} · Crew {} · Speed {} · Combat {}\n{} credits · {} minerals",
                    part.stats.cargo,
                    part.stats.crew_capacity,
                    part.stats.speed,
                    part.stats.combat,
                    part.cost.credits,
                    part.cost.minerals
                ));
                if kind == ComponentKind::Hull {
                    let c = &ctx.data.config.commission;
                    f.action(
                        &format!(
                            "Commission with full refit · {} credits · {} minerals",
                            part.cost.credits + c.premium_credits,
                            part.cost.minerals + c.premium_minerals
                        ),
                        ctx.sim.resources.credits >= part.cost.credits + c.premium_credits
                            && ctx.sim.resources.minerals
                                >= part.cost.minerals + c.premium_minerals,
                        UiAction::CommissionShip(part.id.clone()),
                    );
                }
                f.action(
                    "Purchase and fit",
                    ctx.sim.resources.credits >= part.cost.credits
                        && ctx.sim.resources.minerals >= part.cost.minerals,
                    UiAction::PurchaseComponent(kind, part.id.clone()),
                );
            }
        }
    }
    salvage::build(ctx, f);
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
        let forecast = crate::simulation::readiness::forecast(sim, ctx.data);
        for row in forecast.rows {
            f.heading(&format!("{} · {}", row.concern, row.band.label()));
            f.text(&format!("{}\n{}", row.evidence, row.trend));
            if let Some(id) = row.recommended_project {
                response(ctx, f, &id, row.recommended_target);
            }
        }
        if !sim.survival.emergency_used {
            let cost = &ctx.data.config.survival.emergency_resource_cost;
            f.text(&format!(
                "Emergency stabilisation: {} energy, {} minerals, {} spare parts.",
                cost.energy, cost.minerals, ctx.data.config.survival.emergency_parts_cost
            ));
            f.action("Stabilise air", true, UiAction::EmergencyStabilise);
        }
        f.heading("Persistent aftermath");
        for issue in &sim.issues.active {
            f.text(&format!(
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
                response(ctx, f, id, Some(issue.target.clone()));
            }
        }
        return;
    }
    if section == "catalogue" {
        for choice in crate::ui::agenda::catalogue_choices(ctx) {
            crate::ui::agenda::build_choice(ctx, &choice, f);
        }
    } else {
        if sim.projects.jobs.is_empty() {
            f.text("No projects queued. Tap Available projects to choose ship work.");
        } else if sim.projects.waiting_count() > 0 {
            f.text("Queued jobs are checked in waiting-list order; unavailable jobs are skipped. To resume paused work, tap Review project.");
        }
        for job in &sim.projects.jobs {
            if let Some(def) = projects::definition_for(job, ctx.data) {
                f.heading(&def.name);
                f.text(&format!(
                    "Target: {}",
                    crate::ui::agenda::target_label(ctx.data, job.target_id.as_deref())
                ));
                f.text(&format!(
                    "{:?} · {:.0}% complete",
                    job.status,
                    job.progress(def.duration_months) * 100.0
                ));
                if let Some(reason) = job.pause_reason.as_deref().or(job.stop_reason.as_deref()) {
                    f.text(reason);
                }
                if matches!(
                    job.status,
                    ProjectStatus::Running | ProjectStatus::Paused | ProjectStatus::Queued
                ) {
                    f.action(
                        "Review project",
                        true,
                        UiAction::PreviewCancelProject(job.sequence_id),
                    );
                }
                if let Some((position, count)) = sim.projects.waiting_position(job.sequence_id) {
                    f.text(&format!("Waiting position {position} of {count}"));
                    for (label, direction) in [("Move up", -1), ("Move down", 1)] {
                        f.action(
                            label,
                            if direction < 0 {
                                position > 1
                            } else {
                                position < count
                            },
                            UiAction::MoveProject {
                                sequence_id: job.sequence_id,
                                direction,
                            },
                        );
                    }
                }
            }
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
