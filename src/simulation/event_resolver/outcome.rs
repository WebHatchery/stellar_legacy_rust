//! What a choice does: scoring an outcome for the autoplayer, landing its
//! deltas on the sim, and resolving an event with no player at the helm.

use crate::data::events::{Complication, EventOutcome, EventTemplate};
use crate::data::{GameConfig, GameData};
use crate::simulation::issues;
use crate::simulation::subsystems;
use crate::state::sim::SimState;

use super::rolled_pop_count;
use super::{active_complication, outcome_affordable, outcome_available};

/// Score an outcome for auto-resolution (GDD §5.4). Higher is better.
pub fn score_outcome(outcome: &EventOutcome, sim: &SimState, config: &GameConfig) -> f32 {
    let food_weight = if sim.resources.food < config.low_food_threshold {
        2.0
    } else {
        1.0
    };
    let ship_distressed = sim.ship.hull_integrity < config.hull_warning_threshold
        || sim.ship.life_support < config.life_support_warning_threshold;
    let ship_weight = if ship_distressed { 1000.0 } else { 100.0 };

    let subsystem_value: f32 = outcome
        .subsystem_deltas
        .iter()
        .map(|delta| delta.condition * 700.0 + delta.knowledge * 600.0)
        .sum();
    let faction_value: f32 = outcome
        .faction_approval_deltas
        .iter()
        .map(|delta| delta.delta * 300.0)
        .sum::<f32>()
        + outcome.faction_approval_smallest * 300.0;
    let reputation_value: f32 = outcome
        .reputation_deltas
        .iter()
        .map(|delta| delta.delta * 250.0)
        .sum();
    let irreversible_cost = if outcome.force_return { 700.0 } else { 0.0 }
        + if outcome.faction_loss.is_some() {
            1200.0
        } else {
            0.0
        }
        + if outcome.faction_merge_id.is_some() {
            350.0
        } else {
            0.0
        };

    outcome.resource_delta.food as f32 * food_weight
        + (outcome.ship_delta.hull_integrity + outcome.ship_delta.life_support) * ship_weight
        + outcome.ship_delta.fuel * 500.0
        + outcome.ship_delta.spare_parts as f32 * 12.0
        + outcome.resource_delta.credits as f32 * 0.1
        + outcome.resource_delta.energy as f32 * 0.2
        + outcome.resource_delta.minerals as f32 * 0.3
        + outcome.resource_delta.influence as f32 * 1.5
        + outcome.population_delta.count as f32 * 2.0
        + outcome.population_delta.morale * 500.0
        + outcome.population_delta.unity * 600.0
        + outcome.population_delta.stability * 600.0
        + outcome.population_delta.legacy_loyalty * 350.0
        + outcome.objective_progress_delta * 1000.0
        + subsystem_value
        + faction_value
        + reputation_value
        + if outcome.grant_component.is_some() {
            600.0
        } else {
            0.0
        }
        + if outcome.grant_fitting.is_some() {
            600.0
        } else {
            0.0
        }
        + if outcome.designate_heir { 250.0 } else { 0.0 }
        - 100.0 * outcome.long_term_consequences.len() as f32
        - irreversible_cost
}

/// Apply one outcome of a pending event to the sim and log it.
pub fn apply_outcome(
    sim: &mut SimState,
    data: &GameData,
    template: &EventTemplate,
    outcome_index: usize,
) {
    let Some(outcome) = template.outcomes.get(outcome_index) else {
        return;
    };
    if !outcome_available(sim, outcome) || !outcome_affordable(sim, outcome) {
        return;
    }
    // Snapshot the riding complication (content-depth round 6) from the state as
    // it stood *before* this outcome — the same state the player saw the twist
    // in — so the outcome's own deltas can't move the gate out from under it.
    let complication = active_complication(sim, template).cloned();
    apply_primary_effects(sim, data, template, outcome);

    if let Some(issue) = &outcome.issue {
        issues::record_event_issue(sim, issue, &template.id);
    }
    for issue_id in &outcome.resolves_issues {
        issues::resolve_issue(sim, issue_id, &outcome.label);
    }

    let text = outcome_text(template, outcome);
    sim.push_log(text.clone());
    record_decision(sim, template, outcome, text);
    // The council's own answers are the voyage's most-worth-remembering beats,
    // so the homecoming can show the player what they decided a century ago.
    // Only events that actually asked count — an auto-resolved incident was
    // something that happened to the ship, not something it chose.
    if template.requires_decision {
        crate::simulation::debrief::remember(
            sim,
            data,
            crate::state::sim::debrief::HighlightKind::Decision,
            format!("{} — {}", template.title, outcome.label),
        );
    }
    apply_secondary_effects(sim, data, outcome, complication.as_ref());
    // Record this occurrence (content-depth round 11) *after* the complication
    // has read the prior count, so a recurrence complication rides on the Nth
    // time and not the (N+1)th.
    *sim.event_fire_counts
        .entry(template.id.clone())
        .or_default() += 1;
    sim.pending_event = None;
}

fn apply_primary_effects(
    sim: &mut SimState,
    data: &GameData,
    template: &EventTemplate,
    outcome: &EventOutcome,
) -> u32 {
    let (mut resources, ship, mut population) = subsystems::buffered_deltas(
        sim,
        data,
        &template.family,
        outcome.resource_delta,
        outcome.ship_delta,
        outcome.population_delta,
    );
    if outcome.requires_full_payment {
        resources = outcome.resource_delta;
    }
    population.count = rolled_pop_count(population.count, data.config.real_time, &mut sim.rng);
    sim.resources.apply(&resources);
    sim.ship.apply(&ship);
    sim.population.apply(&population);
    let population_lost = (-population.count).max(0) as u32;
    crate::simulation::mortality::event_claim(sim, data, population_lost);
    crate::simulation::mortality::event_claim(sim, data, population_lost);
    sim.consequences
        .extend(outcome.long_term_consequences.iter().cloned());
    for operation in &outcome.obligation_operations {
        sim.apply_obligation_operation(operation);
    }
    if outcome.designate_heir {
        sim.dynasty.designated_heir =
            crate::simulation::succession::planned_heir(&sim.dynasty, &data.config)
                .map(|member| member.id);
    }
    for delta in &outcome.reputation_deltas {
        sim.adjust_reputation(&delta.id, delta.delta);
    }
    if let Some(followup) = &outcome.schedule_followup {
        sim.scheduled_events
            .push(crate::state::sim::ScheduledEvent {
                template_id: followup.template_id.clone(),
                fire_year: sim.year() + followup.delay_years,
            });
    }
    if let Some(component_id) = &outcome.grant_component {
        sim.ship.salvage.push(component_id.clone());
    }
    if let Some(fitting_id) = &outcome.grant_fitting {
        if !sim.ship.unlocked_fittings.contains(fitting_id) {
            sim.ship.unlocked_fittings.push(fitting_id.clone());
        }
    }
    population_lost
}

fn outcome_text(template: &EventTemplate, outcome: &EventOutcome) -> String {
    if outcome.log.is_empty() {
        format!("{}: {}", template.title, outcome.label)
    } else {
        outcome.log.clone()
    }
}

fn record_decision(
    sim: &mut SimState,
    template: &EventTemplate,
    outcome: &EventOutcome,
    text: String,
) {
    let Some(record) = &outcome.record else {
        return;
    };
    let captain = sim
        .dynasty
        .leader()
        .map(|leader| leader.name.clone())
        .unwrap_or_else(|| "The vacant chair".to_owned());
    let affected_accounts = record
        .affected
        .iter()
        .map(|account| crate::state::sim::AffectedAccount {
            people: account.people.clone(),
            account: account.account.clone(),
        })
        .collect();
    sim.decision_records
        .push(crate::state::sim::DecisionRecord {
            year: sim.year(),
            month: sim.month(),
            event_id: template.id.clone(),
            event_title: template.title.clone(),
            outcome_id: outcome.id.clone(),
            outcome_label: outcome.label.clone(),
            fact: text,
            captain,
            official_account: record.official.clone(),
            dynasty_account: record.dynasty.clone(),
            affected_accounts,
        });
}

fn apply_secondary_effects(
    sim: &mut SimState,
    data: &GameData,
    outcome: &EventOutcome,
    complication: Option<&Complication>,
) {
    if outcome.force_return {
        crate::simulation::contract::jump_to_return(sim);
    }
    if let Some(kind) = outcome.faction_loss {
        match &outcome.faction_loss_id {
            Some(id) => sim.apply_faction_loss_by_id(data, kind, id),
            None => sim.apply_faction_loss(data, kind),
        }
    }
    if let Some(id) = &outcome.faction_merge_id {
        sim.apply_faction_merge(data, id);
    }
    for delta in &outcome.subsystem_deltas {
        if let Some(state) = sim.subsystems.get_mut(&delta.id) {
            state.condition = (state.condition + delta.condition).clamp(0.0, 1.0);
            state.knowledge = (state.knowledge + delta.knowledge).clamp(0.0, 1.0);
        }
    }
    for delta in &outcome.faction_approval_deltas {
        if let Some(state) = sim
            .factions
            .iter_mut()
            .find(|faction| faction.faction_id == delta.id && faction.is_aboard())
        {
            state.adjust_approval(delta.delta);
        }
    }
    sim.apply_rival_approval_spillover(data, &outcome.faction_approval_deltas);
    sim.apply_ally_approval_spillover(data, &outcome.faction_approval_deltas);
    sim.apply_rival_approval_schadenfreude(data, &outcome.faction_approval_deltas);
    sim.apply_ally_approval_commiseration(data, &outcome.faction_approval_deltas);
    if outcome.faction_approval_smallest != 0.0 {
        sim.adjust_smallest_faction_approval(outcome.faction_approval_smallest);
    }
    if outcome.objective_progress_delta != 0.0 {
        if let Some(contract) = sim.contract.as_mut() {
            let shift = outcome.objective_progress_delta * contract.objective_target;
            contract.objective_progress = (contract.objective_progress + shift).max(0.0);
        }
    }
    apply_complication(sim, outcome, complication);
}

fn apply_complication(
    sim: &mut SimState,
    outcome: &EventOutcome,
    complication: Option<&Complication>,
) {
    let applies = complication.is_some_and(|complication| {
        complication.applies_to_outcomes.is_empty()
            || complication.applies_to_outcomes.contains(&outcome.id)
    });
    let Some(complication) = complication.filter(|_| applies) else {
        return;
    };
    sim.resources.apply(&complication.resource_delta);
    sim.ship.apply(&complication.ship_delta);
    sim.population.apply(&complication.population_delta);
    for delta in &complication.subsystem_deltas {
        if let Some(state) = sim.subsystems.get_mut(&delta.id) {
            state.condition = (state.condition + delta.condition).clamp(0.0, 1.0);
            state.knowledge = (state.knowledge + delta.knowledge).clamp(0.0, 1.0);
        }
    }
    if !complication.log.is_empty() {
        sim.push_log(complication.log.clone());
    }
}

/// Pick the best-scoring outcome and apply it (delegated/no-decision path).
/// Returns the applied outcome's label.
pub fn auto_resolve(sim: &mut SimState, data: &GameData, template: &EventTemplate) -> String {
    let best = template
        .outcomes
        .iter()
        .enumerate()
        .filter(|(_, outcome)| outcome_available(sim, outcome) && outcome_affordable(sim, outcome))
        .max_by(|(_, a), (_, b)| {
            score_outcome(a, sim, &data.config).total_cmp(&score_outcome(b, sim, &data.config))
        })
        .map(|(i, _)| i)
        .unwrap_or(0);
    let label = template
        .outcomes
        .get(best)
        .map(|o| o.label.clone())
        .unwrap_or_default();
    apply_outcome(sim, data, template, best);
    label
}
