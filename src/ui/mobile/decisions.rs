//! Mobile decision cards and their visible commit controls.

use super::*;
use crate::state::sim::AuthorityChoice;

pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form) -> Option<String> {
    let sim = ctx.sim;
    if sim.terminal.is_some() || sim.dynasty.extinct {
        if ctx.screen == Screen::Chronicle {
            return None;
        }
        return build_terminal(ctx, f);
    }
    if let Some(report) = &sim.debrief {
        return build_homecoming(ctx, f, report);
    }
    if sim.survival.warning_active {
        return build_recovery(ctx, f);
    }
    if let Some(review) = &sim.authority.pending_review {
        return build_authority(ctx, f, review);
    }
    if let Some(pending) = &sim.pending_event {
        if let Some(event) = ctx.data.events.get(&pending.template_id) {
            return build_event(ctx, f, pending, event);
        }
    }
    if let Some(pending) = &sim.pending_dilemma {
        if let Some(d) = crate::simulation::legacy::pending_dilemma_def(sim, ctx.data) {
            return build_dilemma(ctx, f, pending, d);
        }
    }
    None
}

fn build_terminal(ctx: &GameplayCtx<'_>, form: &mut Form) -> Option<String> {
    let sim = ctx.sim;
    form.heading(
        sim.terminal
            .as_ref()
            .map_or("Dynasty extinction", |t| t.reason.label()),
    );
    if let Some(terminal) = &sim.terminal {
        form.text(&terminal.evidence);
    }
    form.text(&format!(
        "Year {} · Generation {}\n{} people aboard\nHull {:.0}% · Air {:.0}%",
        sim.year(),
        sim.dynasty.generation,
        sim.population.count,
        sim.ship.hull_integrity * 100.0,
        sim.ship.life_support * 100.0
    ));
    for entry in sim.log.iter().rev().take(4) {
        form.text(&entry.text);
    }
    form.action(
        "Read History",
        true,
        UiAction::SelectScreen(Screen::Chronicle),
    );
    form.action("Return to menu", true, UiAction::ToMenu);
    Some("terminal".to_owned())
}

fn build_homecoming(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    report: &crate::state::sim::debrief::VoyageDebrief,
) -> Option<String> {
    form.heading(&format!("Homecoming · {}", report.outcome));
    form.text(&report.contract_name);
    form.text(&debrief::report::accounting(report));
    if let Some(recovery) = report
        .recovery
        .as_ref()
        .filter(|recovery| !recovery.resolved)
    {
        build_homecoming_recovery(ctx, form, recovery);
    }
    form.heading("Captains");
    for captain in &report.commanders {
        form.portrait(&captain.name);
        form.text(&format!(
            "Generation {} · {} years in command · {} inherited duties\n{}",
            captain.generation,
            captain.years_held(report.ended_year),
            captain.inherited_obligations,
            captain.trait_name
        ));
    }
    form.heading("Defining moments");
    for beat in &report.highlights {
        form.text(&format!(
            "Year {} · {}\n{}",
            beat.year,
            beat.kind.tag(),
            beat.text
        ));
    }
    Some("homecoming".to_owned())
}

fn build_recovery(ctx: &GameplayCtx<'_>, form: &mut Form) -> Option<String> {
    let sim = ctx.sim;
    form.heading("! The air line is failing");
    form.text(&format!(
        "Air {:.0}% · Zero-air clock {}/{} months. The voyage is paused for recovery review.",
        sim.ship.life_support * 100.0,
        sim.survival.air_zero_months,
        ctx.data.config.survival.air_grace_months
    ));
    if let Some(notice) = &sim.survival.migration_notice {
        form.text(notice);
    }
    form.action_section("Review Agenda", UiAction::ReviewRecovery, "readiness");
    crate::ui::recovery_warning::build_stabilisation(ctx, form);
    form.action("Resume voyage", true, UiAction::ResumeAfterWarning);
    Some("recovery".to_owned())
}

fn build_authority(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    review: &crate::state::sim::AuthorityReview,
) -> Option<String> {
    let sim = ctx.sim;
    form.heading("Captain reviews the mandate");
    form.portrait(&review.captain);
    form.text(&format!(
        "{} · {} priority\n{}",
        time_controls::fallback_label(sim.speed, ctx.decision_remaining),
        review.priority.label(),
        review.reason
    ));
    for (label, posture) in [
        ("Proposed", review.proposed),
        ("Compromise", review.compromise),
    ] {
        form.text(&format!(
            "{label}: {} · Work {:.0}% · Events {:.0}% · Fuel {:.0}%",
            posture.label(),
            crate::simulation::command::objective_factor(posture) * 100.0,
            crate::simulation::command::event_chance_factor(posture) * 100.0,
            crate::simulation::command::fuel_burn_factor(posture) * 100.0
        ));
    }
    form.action(
        "Keep current posture",
        true,
        UiAction::ResolveAuthority(AuthorityChoice::KeepCurrent),
    );
    form.action(
        "Accept compromise",
        true,
        UiAction::ResolveAuthority(AuthorityChoice::AcceptCompromise),
    );
    form.action(
        "Emergency override",
        review.emergency_allowed,
        UiAction::ResolveAuthority(AuthorityChoice::EmergencyOverride),
    );
    Some("authority".to_owned())
}

fn build_event(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    pending: &crate::state::sim::PendingEvent,
    event: &crate::data::events::EventTemplate,
) -> Option<String> {
    let sim = ctx.sim;
    form.heading(&event.title);
    form.portrait(sim.dynasty.leader().map_or("Captain", |p| p.name.as_str()));
    form.text(&time_controls::fallback_label(
        sim.speed,
        ctx.decision_remaining,
    ));
    form.text(&crate::simulation::event_resolver::shown_description(
        sim, event,
    ));
    let section = ctx.presentation.mobile_section.borrow().clone();
    if section == "advice" {
        for advice in crate::simulation::advice::for_event(sim, ctx.data, event) {
            form.text(&format!(
                "{} · {}\n{}",
                advice.officer_name.as_deref().unwrap_or(&advice.post_name),
                advice.post_name,
                advice.text
            ));
        }
        form.section("Hide officer advice", "");
    } else {
        form.section("Read officer advice", "advice");
    }
    let available = crate::simulation::event_resolver::available_outcome_indices(sim, event);
    for index in available {
        let option = &event.outcomes[index];
        form.heading(&option.label);
        form.text(&option.description);
        let range = crate::simulation::event_resolver::outcome_pop_impact_range(
            sim, ctx.data, event, index,
        );
        let fuel =
            crate::simulation::event_resolver::outcome_fuel_preview(sim, ctx.data, event, index);
        form.text(&event_modal::known_effects(option, range, fuel).0);
        let affordable = crate::simulation::event_resolver::outcome_affordable(sim, option);
        if !affordable {
            form.text("Unavailable: insufficient stores for this choice.");
        }
        form.action(
            &format!("Commit: {}", option.label),
            affordable,
            UiAction::ResolveEvent(index),
        );
    }
    Some(format!(
        "event:{}:{}:{section}",
        pending.template_id, pending.rolled_month_clock
    ))
}

fn build_dilemma(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    pending: &crate::state::sim::PendingDilemma,
    dilemma: &crate::data::legacies::DilemmaDef,
) -> Option<String> {
    let sim = ctx.sim;
    form.heading(&dilemma.title);
    form.text(&time_controls::fallback_label(
        sim.speed,
        ctx.decision_remaining,
    ));
    form.text(&dilemma.description);
    for (index, option) in dilemma.options.iter().enumerate() {
        form.heading(&option.label);
        form.text(&format!(
            "Success odds {:.0}%\nSuccess: {}\nFailure: {}",
            crate::simulation::legacy::dilemma_odds(sim, ctx.data, option) * 100.0,
            option.success.log,
            option.failure.log
        ));
        form.action(
            &format!("Commit: {}", option.label),
            true,
            UiAction::ResolveDilemma(index),
        );
    }
    Some(format!("dilemma:{}", pending.dilemma_id))
}

fn build_homecoming_recovery(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    recovery: &crate::state::sim::HomecomingRecovery,
) {
    form.heading(&format!("Recovery review · {}", recovery.focus.label()));
    form.text(&format!(
        "{}\nTarget: {}",
        recovery.situation, recovery.target_label
    ));
    for choice in crate::state::sim::HomecomingChoice::ALL {
        let available = crate::simulation::homecoming::choice_available(ctx.sim, ctx.data, choice);
        let cost = crate::simulation::homecoming::choice_cost(ctx.sim, ctx.data, choice);
        let bill = if choice == crate::state::sim::HomecomingChoice::Defer {
            "no immediate cost".to_owned()
        } else {
            format!(
                "{} credits · {} influence",
                cost.credits.abs(),
                cost.influence.abs()
            )
        };
        form.heading(choice.label());
        let status = if available {
            bill
        } else {
            crate::simulation::homecoming::choice_unavailable_reason(ctx.sim, ctx.data, choice)
                .unwrap_or_else(|| "This choice cannot be committed.".to_owned())
        };
        form.text(&format!(
            "{}\n{}\n{}",
            choice.description(),
            crate::simulation::homecoming::choice_effects(ctx.data, choice),
            status
        ));
        form.action(
            if available {
                "Commit recovery"
            } else {
                "Unavailable"
            },
            available,
            UiAction::ChooseHomecomingRecovery(choice),
        );
    }
}
