//! Mobile voyage, contract, and market sections.

use super::*;
use crate::simulation::{contract, market};

pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    let sim = ctx.sim;
    if sim.contract.is_none() {
        f.actions(
            vec![
                ("Drydock", UiAction::SelectScreen(Screen::Drydock)),
                ("Market", UiAction::SelectScreen(Screen::Market)),
            ],
            usize::from(ctx.screen == Screen::Market),
        );
    }
    if ctx.screen == Screen::Market {
        trade(ctx, f);
        return;
    }
    if section == "posture" {
        f.section("Back to voyage", "");
        posture(ctx, f);
        return;
    }
    if let Some(c) = &sim.contract {
        build_active_voyage(ctx, f, c);
        return;
    }
    if let Some(t) = sim
        .selected_charter
        .as_ref()
        .and_then(|id| ctx.data.contracts.get(id))
    {
        build_selected_charter(ctx, f, t);
        return;
    }
    build_available_charters(ctx, f);
}

fn build_active_voyage(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    contract_state: &crate::state::sim::ActiveContract,
) {
    form.heading(&contract_state.name);
    form.text(&format!(
        "{} · Year {} / {}\nVoyage {:.0}% · Objective {:.0} / {:.0} {}",
        contract_state.phase.label(),
        contract_state.months_elapsed / 12,
        contract_state.target_duration_years,
        contract_state.progress() * 100.0,
        contract_state.objective_progress,
        contract_state.objective_target,
        contract_state.objective_unit
    ));
    fuel_status(ctx, form);
    for milestone in &contract_state.milestones {
        form.text(&format!(
            "{} · {}",
            milestone.name,
            if milestone.reached {
                "Reached"
            } else {
                "Ahead"
            }
        ));
    }
    form.heading(&format!(
        "Charter approach · {}",
        contract_state.approach.label_for(contract_state.objective)
    ));
    form.text(&format!(
        "{}\n{}",
        contract_state
            .approach
            .description_for(contract_state.objective),
        crate::simulation::approach::effect_summary(contract_state.approach)
    ));
    posture_summary(ctx, form);
    if contract::can_return_home(ctx.sim) {
        form.action("Review return home", true, UiAction::ReviewReturnHome);
    }
}

fn build_selected_charter(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    template: &crate::data::contracts::ContractTemplate,
) {
    form.heading(&template.name);
    form.text(&template.description);
    form.text(&format!(
        "{} years · Reward {} credits\nGoal: {:.0} {}",
        template.target_duration_years,
        template.reward.credits,
        template.objective_target,
        template.objective_unit
    ));
    for phase in &template.phases {
        form.text(&format!("{} · {} years", phase.kind.label(), phase.years));
    }
    for risk in &template.failure_risks {
        form.text(&format!("Risk: {}", risk.replace('_', " ")));
    }
    build_approach_choices(ctx, form, template);
    let (conflict_count, shortfalls) = build_departure_provisions(ctx, form, template);
    form.action("PROVISIONS REVIEWED", true, UiAction::ReviewProvisions);
    posture_summary(ctx, form);
    form.action(
        &crate::ui::prep::launch_commit_label(conflict_count, shortfalls),
        true,
        UiAction::Launch,
    );
    form.action("Choose another charter", true, UiAction::CancelSelection);
}

fn build_approach_choices(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    template: &crate::data::contracts::ContractTemplate,
) {
    let sim = ctx.sim;
    form.heading("Charter approach · fixed at launch");
    form.text(&crate::ui::charter_approach::mobile_summary(
        sim,
        template.objective,
    ));
    for candidate in crate::state::sim::CharterApproach::ALL {
        let active = sim.selected_charter_approach == candidate;
        let available =
            crate::simulation::approach::choice_available(sim, ctx.data, template, candidate);
        form.heading(&candidate.label_for(template.objective));
        form.text(&format!(
            "{}\n{}",
            candidate.description_for(template.objective),
            crate::simulation::approach::effect_summary(candidate)
        ));
        if !available {
            form.text(
                &crate::simulation::approach::unavailable_reason(
                    sim, ctx.data, template, candidate,
                )
                .unwrap_or_else(|| "This approach is unavailable.".to_owned()),
            );
        }
        form.action(
            if active {
                "Selected"
            } else if available {
                "Choose approach"
            } else {
                "Unavailable"
            },
            available && !active,
            UiAction::SetCharterApproach(candidate),
        );
    }
}

fn build_departure_provisions(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    template: &crate::data::contracts::ContractTemplate,
) -> (usize, usize) {
    let sim = ctx.sim;
    let forecast = contract::forecast::for_departure(sim, ctx.data, template);
    form.heading("Departure provisions");
    form.text(&format!(
        "Food {} / {} recommended\nProjected end {} · Net {:+}/year\nEvents and deterioration can change this baseline.",
        sim.resources.food,
        forecast.recommended_food_store,
        forecast.projected_food_end,
        forecast.annual_food_net
    ));
    let need = (forecast.recommended_food_store - sim.resources.food).max(0);
    let unit = market::buy_quote(sim, TradeResource::Food, 1).effective_unit_price;
    let amount = need.min(if unit > 0.0 {
        (sim.resources.credits as f32 / unit).floor() as i64
    } else {
        0
    });
    let cost = market::buy_quote(sim, TradeResource::Food, amount).total_credits;
    if need > 0 {
        form.action(
            &format!("Buy {amount} food · {cost} credits"),
            amount > 0,
            UiAction::Buy(TradeResource::Food, amount),
        );
    }
    let price = ctx.data.config.provisioning.part_cost_credits;
    let parts = (forecast.parts_upkeep - sim.ship.spare_parts)
        .max(0)
        .min(if price > 0 {
            sim.resources.credits / price
        } else {
            0
        });
    form.text(&format!(
        "Spare parts {} / {} recommended",
        sim.ship.spare_parts, forecast.parts_upkeep
    ));
    if parts > 0 {
        form.action(
            &format!("Buy {parts} parts · {} credits", parts * price),
            true,
            UiAction::BuyParts(parts),
        );
    }
    form.text(&format!(
        "Fuel aboard {:.0}% · Travel {} years\nTotal travel burn: {:.2} full tanks\nAnnual scoops: up to {:.1}% of a tank",
        sim.ship.fuel * 100.0,
        forecast.travel_years,
        forecast.fuel_burn,
        forecast.fuel_regen_per_year * 100.0
    ));
    form.text("The tank holds 100%. Scoops replenish fuel during the voyage, so total travel needs can exceed one tank. This baseline assumes current engineering condition.");
    form.text(&format!(
        "Route hull {:+.0}% · air {:+.0}%",
        forecast.route_hull_change * 100.0,
        forecast.route_life_support_change * 100.0
    ));
    let fuel_cost = (ctx.data.config.provisioning.fuel_cost_credits_per_point as f32
        * (1.0 - sim.ship.fuel)
        * 100.0)
        .ceil() as i64;
    form.action(
        &format!("Refuel tank · {fuel_cost} credits"),
        sim.ship.fuel < 1.0 && sim.resources.credits >= fuel_cost,
        UiAction::Refuel,
    );
    let conflicts = contract::obligation_conflicts(sim, template);
    for obligation in &conflicts {
        form.text(&format!(
            "! Launch will break: {} · owed to {}",
            obligation.title, obligation.beneficiary
        ));
    }
    form.text(&format!(
        "{} promises will be recorded at launch. Review History for inherited obligations.",
        template.launch_obligation_operations.len()
    ));
    let shortfalls = usize::from(need > 0)
        + usize::from(forecast.parts_upkeep > sim.ship.spare_parts)
        + usize::from(sim.ship.fuel < 0.999);
    (conflicts.len(), shortfalls)
}

fn build_available_charters(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Choose a charter");
    let mut templates: Vec<_> = ctx
        .data
        .contracts
        .iter()
        .map(|(_, template)| template)
        .filter(|template| crate::data::contracts::is_available_in_build(template))
        .collect();
    templates.sort_by_key(|template| {
        (
            contract_systems::charter_lock(ctx, template).0,
            template.target_duration_years,
            template.name.clone(),
        )
    });
    for template in templates {
        let (locked, reason) = contract_systems::charter_lock(ctx, template);
        form.heading(&template.name);
        form.text(&format!(
            "{} years · {} · {} credits\nRoute: {} · {} launch promises",
            template.target_duration_years,
            template.objective.label(),
            template.reward.credits,
            if template.hazard > 0.0 {
                "more crisis-prone"
            } else {
                "ordinary crisis exposure"
            },
            template.launch_obligation_operations.len()
        ));
        if locked {
            form.text(&reason);
        }
        form.action(
            "Read briefing & prepare",
            !locked,
            UiAction::SelectCharter(template.id.clone()),
        );
    }
}

fn fuel_status(ctx: &GameplayCtx<'_>, f: &mut Form) {
    let (status, stalled) = crate::ui::contract_systems::mission_clock_status(ctx.sim, ctx.data);
    f.heading(&format!("Fuel aboard · {:.1}%", ctx.sim.ship.fuel * 100.0));
    f.text(&status);
    if stalled {
        let scoop = crate::simulation::readiness::forecast(ctx.sim, ctx.data)
            .fuel
            .annual_scoop;
        if scoop > 0.0 {
            f.text(&format!("Travel waits for fuel while the ship's calendar and upkeep continue. The current scoops can restore up to {:.1}% of a tank each year; engineering condition can change that rate.", scoop * 100.0));
            f.text(if ctx.sim.speed == GameSpeed::Paused {
                "Tap Resume to let calendar time and annual fuel recovery advance. Fuel is purchased only in port."
            } else {
                "Leave time running for annual fuel recovery. Fuel is purchased only in port."
            });
        } else {
            f.text("The current loadout cannot regenerate fuel. Fuel is purchased only in port. Review fuel readiness for available ship work, or use Review return home below to consider ending the mission early.");
        }
        f.action_section(
            "Review fuel readiness",
            UiAction::SelectScreen(Screen::Agenda),
            "readiness",
        );
    }
}

fn posture_summary(ctx: &GameplayCtx<'_>, f: &mut Form) {
    f.heading(&format!(
        "Command posture · {}",
        ctx.sim.command_posture.label()
    ));
    f.text(ctx.sim.command_posture.description());
    f.section("Compare command postures", "posture");
}

fn posture(ctx: &GameplayCtx<'_>, f: &mut Form) {
    f.heading(&format!(
        "Command posture · {}",
        ctx.sim.command_posture.label()
    ));
    f.action("REVIEW MANDATE", true, UiAction::OpenHelp);
    let wait = crate::simulation::command::review_wait_months(ctx.sim);
    if wait > 0 {
        f.text(&format!("Current posture is committed. Another proposal becomes available in {wait} months. Advance voyage time to reach the next review."));
    } else if ctx.sim.contract.is_some() {
        f.text("A ratified change commits the ship for 12 months. The captain may require a mandate review before accepting a proposal.");
    } else {
        f.text("Postures can change freely in port. The captain may require a mandate review before accepting a proposal.");
    }
    f.text("Percentages compare each posture with STEADY at 100%. Work applies during operations; fuel use applies during travel.");
    for p in CommandPosture::ALL {
        let current = p == ctx.sim.command_posture;
        f.heading(&format!(
            "{}{}",
            p.label(),
            if current { " · Current" } else { "" }
        ));
        f.text(p.description());
        f.text(&format!(
            "Work {:.0}% · Event chance {:.0}% · Fuel use {:.0}%",
            crate::simulation::command::objective_factor(p) * 100.0,
            crate::simulation::command::event_chance_factor(p) * 100.0,
            crate::simulation::command::fuel_burn_factor(p) * 100.0
        ));
        f.text(match p {
            CommandPosture::Steady => "No additional annual social adjustment.",
            CommandPosture::Expeditionary => "Each year underway, this posture also reduces morale, unity and legacy loyalty.",
            CommandPosture::Civic => "Each year underway, this posture also improves morale, unity, stability and legacy loyalty.",
        });
        f.action(
            &format!(
                "{} {}",
                if current { "Current:" } else { "Propose" },
                p.label()
            ),
            !current && wait == 0,
            UiAction::SetPosture(p),
        );
    }
}

fn trade(ctx: &GameplayCtx<'_>, f: &mut Form) {
    f.heading(&format!("Market · {} credits", ctx.sim.resources.credits));
    let (small, large) = crate::ui::market::trade_lot_sizes(
        crate::simulation::ship::loadout_stats(ctx.sim, ctx.data).cargo as i64,
    );
    for resource in TradeResource::ALL {
        let held = crate::ui::market::held_amount(ctx, resource);
        f.heading(resource.label());
        f.text(&format!("{held} held · quotes settle on tap"));
        for amount in [small, large] {
            let buy = market::buy_quote(ctx.sim, resource, amount);
            let sell = market::sell_quote(ctx.sim, resource, amount);
            f.action(
                &format!("Buy {amount} · {} credits", buy.total_credits),
                ctx.sim.resources.credits >= buy.total_credits,
                UiAction::Buy(resource, amount),
            );
            f.action(
                &format!("Sell {amount} · receive {} credits", sell.total_credits),
                held >= amount,
                UiAction::Sell(resource, amount),
            );
        }
    }
}
