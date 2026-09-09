use super::*;
use crate::state::sim::AuthorityChoice;

pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form) -> Option<String> {
    let sim = ctx.sim;
    if sim.terminal.is_some() || sim.dynasty.extinct {
        if ctx.screen == Screen::Chronicle {
            return None;
        }
        f.heading(
            sim.terminal
                .as_ref()
                .map_or("Dynasty extinction", |t| t.reason.label()),
        );
        if let Some(t) = &sim.terminal {
            f.text(&t.evidence);
        }
        f.text(&format!(
            "Year {} · Generation {}\n{} people aboard\nHull {:.0}% · Air {:.0}%",
            sim.year(),
            sim.dynasty.generation,
            sim.population.count,
            sim.ship.hull_integrity * 100.0,
            sim.ship.life_support * 100.0
        ));
        for entry in sim.log.iter().rev().take(4) {
            f.text(&entry.text);
        }
        f.action(
            "Read History",
            true,
            UiAction::SelectScreen(Screen::Chronicle),
        );
        f.action("Return to menu", true, UiAction::ToMenu);
        return Some("terminal".to_owned());
    }
    if let Some(report) = &sim.debrief {
        f.heading(&format!("Homecoming · {}", report.outcome));
        f.text(&report.contract_name);
        f.text(&debrief::report::accounting(report));
        f.heading("Captains");
        for captain in &report.commanders {
            f.portrait(&captain.name);
            f.text(&format!(
                "Generation {} · {} years in command · {} inherited duties\n{}",
                captain.generation,
                captain.years_held(report.ended_year),
                captain.inherited_obligations,
                captain.trait_name
            ));
        }
        f.heading("Defining moments");
        for beat in &report.highlights {
            f.text(&format!(
                "Year {} · {}\n{}",
                beat.year,
                beat.kind.tag(),
                beat.text
            ));
        }

        return Some("homecoming".to_owned());
    }
    if sim.survival.warning_active {
        f.heading("! The air line is failing");
        f.text(&format!(
            "Air {:.0}% · Zero-air clock {}/{} months. The voyage is paused for recovery review.",
            sim.ship.life_support * 100.0,
            sim.survival.air_zero_months,
            ctx.data.config.survival.air_grace_months
        ));
        let cost = &ctx.data.config.survival.emergency_resource_cost;
        f.text(&format!("Emergency stabilisation costs {} energy, {} minerals and {} spare parts. Emergency rescue {}.",cost.energy,cost.minerals,ctx.data.config.survival.emergency_parts_cost,if sim.survival.emergency_used{"already used"}else{"available"}));
        if let Some(notice) = &sim.survival.migration_notice {
            f.text(notice);
        }
        f.action_section("Review Agenda", UiAction::ReviewRecovery, "readiness");
        f.action(
            "Stabilise air",
            !sim.survival.emergency_used,
            UiAction::EmergencyStabilise,
        );
        f.action("Resume voyage", true, UiAction::ResumeAfterWarning);
        return Some("recovery".to_owned());
    }
    if let Some(review) = &sim.authority.pending_review {
        f.heading("Captain reviews the mandate");
        f.portrait(&review.captain);
        f.text(&format!(
            "Fallback in {:.0}s · {} priority\n{}",
            ctx.decision_remaining.ceil(),
            review.priority.label(),
            review.reason
        ));
        for (label, p) in [
            ("Proposed", review.proposed),
            ("Compromise", review.compromise),
        ] {
            f.text(&format!(
                "{label}: {} · Work {:.0}% · Events {:.0}% · Fuel {:.0}%",
                p.label(),
                crate::simulation::command::objective_factor(p) * 100.0,
                crate::simulation::command::event_chance_factor(p) * 100.0,
                crate::simulation::command::fuel_burn_factor(p) * 100.0
            ));
        }
        f.action(
            "Keep current posture",
            true,
            UiAction::ResolveAuthority(AuthorityChoice::KeepCurrent),
        );
        f.action(
            "Accept compromise",
            true,
            UiAction::ResolveAuthority(AuthorityChoice::AcceptCompromise),
        );
        f.action(
            "Emergency override",
            review.emergency_allowed,
            UiAction::ResolveAuthority(AuthorityChoice::EmergencyOverride),
        );
        return Some("authority".to_owned());
    }
    if let Some(pending) = &sim.pending_event {
        if let Some(event) = ctx.data.events.get(&pending.template_id) {
            f.heading(&event.title);
            f.portrait(sim.dynasty.leader().map_or("Captain", |p| p.name.as_str()));
            f.text(&format!(
                "Captain fallback in {:.0}s",
                ctx.decision_remaining.ceil()
            ));
            f.text(&crate::simulation::event_resolver::shown_description(
                sim, event,
            ));
            let section = ctx.presentation.mobile_section.borrow().clone();
            if section == "advice" {
                for advice in crate::simulation::advice::for_event(sim, ctx.data, event) {
                    f.text(&format!(
                        "{} · {}\n{}",
                        advice.officer_name.as_deref().unwrap_or(&advice.post_name),
                        advice.post_name,
                        advice.text
                    ));
                }
                f.section("Hide officer advice", "");
            } else {
                f.section("Read officer advice", "advice");
            }
            let available =
                crate::simulation::event_resolver::available_outcome_indices(sim, event);
            for index in available {
                let option = &event.outcomes[index];
                f.heading(&option.label);
                f.text(&option.description);
                let range = crate::simulation::event_resolver::outcome_pop_impact_range(
                    sim, ctx.data, event, index,
                );
                let fuel = crate::simulation::event_resolver::outcome_fuel_preview(
                    sim, ctx.data, event, index,
                );
                f.text(&event_modal::known_effects(option, range, fuel).0);
                let ok = crate::simulation::event_resolver::outcome_affordable(sim, option);
                if !ok {
                    f.text("Unavailable: insufficient stores for this choice.");
                }
                f.action(
                    &format!("Commit: {}", option.label),
                    ok,
                    UiAction::ResolveEvent(index),
                );
            }
            return Some(format!(
                "event:{}:{}:{section}",
                pending.template_id, pending.rolled_month_clock
            ));
        }
    }
    if let Some(pending) = &sim.pending_dilemma {
        if let Some(d) = crate::simulation::legacy::pending_dilemma_def(sim, ctx.data) {
            f.heading(&d.title);
            f.text(&format!(
                "Captain fallback in {:.0}s",
                ctx.decision_remaining.ceil()
            ));
            f.text(&d.description);
            for (index, o) in d.options.iter().enumerate() {
                f.heading(&o.label);
                f.text(&format!(
                    "Success odds {:.0}%\nSuccess: {}\nFailure: {}",
                    crate::simulation::legacy::dilemma_odds(sim, ctx.data, o) * 100.0,
                    o.success.log,
                    o.failure.log
                ));
                f.action(
                    &format!("Commit: {}", o.label),
                    true,
                    UiAction::ResolveDilemma(index),
                );
            }
            return Some(format!("dilemma:{}", pending.dilemma_id));
        }
    }
    None
}
