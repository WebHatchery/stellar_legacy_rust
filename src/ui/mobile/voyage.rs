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
        f.heading(&c.name);
        f.text(&format!(
            "{} · Year {} / {}\nVoyage {:.0}% · Objective {:.0} / {:.0} {}",
            c.phase.label(),
            c.months_elapsed / 12,
            c.target_duration_years,
            c.progress() * 100.0,
            c.objective_progress,
            c.objective_target,
            c.objective_unit
        ));
        fuel_status(ctx, f);
        for m in &c.milestones {
            f.text(&format!(
                "{} · {}",
                m.name,
                if m.reached { "Reached" } else { "Ahead" }
            ));
        }
        f.heading(&format!(
            "Charter approach · {}",
            c.approach.label_for(c.objective)
        ));
        f.text(&format!(
            "{}\n{}",
            c.approach.description_for(c.objective),
            crate::simulation::approach::effect_summary(c.approach)
        ));
        posture_summary(ctx, f);
        if contract::can_return_home(sim) {
            f.action("Review return home", true, UiAction::ReviewReturnHome);
        }
        return;
    }
    if let Some(t) = sim
        .selected_charter
        .as_ref()
        .and_then(|id| ctx.data.contracts.get(id))
    {
        f.heading(&t.name);
        f.text(&t.description);
        f.text(&format!(
            "{} years · Reward {} credits\nGoal: {:.0} {}",
            t.target_duration_years, t.reward.credits, t.objective_target, t.objective_unit
        ));
        for phase in &t.phases {
            f.text(&format!("{} · {} years", phase.kind.label(), phase.years));
        }
        for risk in &t.failure_risks {
            f.text(&format!("Risk: {}", risk.replace('_', " ")));
        }
        f.heading("Charter approach · fixed at launch");
        f.text(&crate::ui::charter_approach::mobile_summary(
            sim,
            t.objective,
        ));
        for candidate in crate::state::sim::CharterApproach::ALL {
            let active = sim.selected_charter_approach == candidate;
            let available =
                crate::simulation::approach::choice_available(sim, ctx.data, t, candidate);
            f.heading(candidate.label_for(t.objective));
            f.text(&format!(
                "{}\n{}",
                candidate.description_for(t.objective),
                crate::simulation::approach::effect_summary(candidate)
            ));
            if !available {
                f.text(
                    &crate::simulation::approach::unavailable_reason(sim, ctx.data, t, candidate)
                        .unwrap_or_else(|| "This approach is unavailable.".to_owned()),
                );
            }
            f.action(
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
        let forecast = contract::forecast::for_departure(sim, ctx.data, t);
        f.heading("Departure provisions");
        f.text(&format!("Food {} / {} recommended\nProjected end {} · Net {:+}/year\nEvents and deterioration can change this baseline.",sim.resources.food,forecast.recommended_food_store,forecast.projected_food_end,forecast.annual_food_net));
        let need = (forecast.recommended_food_store - sim.resources.food).max(0);
        let unit = market::buy_quote(sim, TradeResource::Food, 1).effective_unit_price;
        let amount = need.min(if unit > 0.0 {
            (sim.resources.credits as f32 / unit).floor() as i64
        } else {
            0
        });
        let cost = market::buy_quote(sim, TradeResource::Food, amount).total_credits;
        if need > 0 {
            f.action(
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
        f.text(&format!(
            "Spare parts {} / {} recommended",
            sim.ship.spare_parts, forecast.parts_upkeep
        ));
        if parts > 0 {
            f.action(
                &format!("Buy {parts} parts · {} credits", parts * price),
                true,
                UiAction::BuyParts(parts),
            );
        }
        f.text(&format!("Fuel aboard {:.0}% · Travel {} years\nTotal travel burn: {:.2} full tanks\nAnnual scoops: up to {:.1}% of a tank",sim.ship.fuel*100.0,forecast.travel_years,forecast.fuel_burn,forecast.fuel_regen_per_year * 100.0));
        f.text("The tank holds 100%. Scoops replenish fuel during the voyage, so total travel needs can exceed one tank. This baseline assumes current engineering condition.");
        f.text(&format!(
            "Route hull {:+.0}% · air {:+.0}%",
            forecast.route_hull_change * 100.0,
            forecast.route_life_support_change * 100.0
        ));
        let fuel_cost = (ctx.data.config.provisioning.fuel_cost_credits_per_point as f32
            * (1.0 - sim.ship.fuel)
            * 100.0)
            .ceil() as i64;
        f.action(
            &format!("Refuel tank · {fuel_cost} credits"),
            sim.ship.fuel < 1.0 && sim.resources.credits >= fuel_cost,
            UiAction::Refuel,
        );
        f.action("PROVISIONS REVIEWED", true, UiAction::ReviewProvisions);
        posture_summary(ctx, f);
        let conflicts = contract::obligation_conflicts(sim, t);
        for obligation in &conflicts {
            f.text(&format!(
                "! Launch will break: {} · owed to {}",
                obligation.title, obligation.beneficiary
            ));
        }
        f.text(&format!(
            "{} promises will be recorded at launch. Review History for inherited obligations.",
            t.launch_obligation_operations.len()
        ));
        let shortfalls = usize::from(need > 0)
            + usize::from(forecast.parts_upkeep > sim.ship.spare_parts)
            + usize::from(sim.ship.fuel < 0.999);
        f.action(
            &crate::ui::prep::launch_commit_label(conflicts.len(), shortfalls),
            true,
            UiAction::Launch,
        );
        f.action("Choose another charter", true, UiAction::CancelSelection);
        return;
    }
    f.heading("Choose a charter");
    let mut templates: Vec<_> = ctx
        .data
        .contracts
        .iter()
        .map(|(_, t)| t)
        .filter(|t| crate::data::contracts::is_available_in_build(t))
        .collect();
    templates.sort_by_key(|t| {
        (
            contract_systems::charter_lock(ctx, t).0,
            t.target_duration_years,
            t.name.clone(),
        )
    });
    for t in templates {
        let (locked, reason) = contract_systems::charter_lock(ctx, t);
        f.heading(&t.name);
        f.text(&format!(
            "{} years · {} · {} credits\nRoute: {} · {} launch promises",
            t.target_duration_years,
            t.objective.label(),
            t.reward.credits,
            if t.hazard > 0.0 {
                "more crisis-prone"
            } else {
                "ordinary crisis exposure"
            },
            t.launch_obligation_operations.len()
        ));
        if locked {
            f.text(&reason);
        }
        f.action(
            "Read briefing & prepare",
            !locked,
            UiAction::SelectCharter(t.id.clone()),
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
