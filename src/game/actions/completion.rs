//! What happens when a charter concludes: the pay, the marks it leaves on the
//! ship and its people, the Chronicle entry, and the homecoming report the
//! player is shown. Each phase stays small so the order of conclusion remains
//! visible and testable.

use crate::chronicle::ChronicleEntry;
use crate::data::{GameData, ResourceDelta};
use crate::game::Game;
use crate::simulation::{contract, debrief};
use crate::state::GameState;
use macroquad::prelude::get_time;

impl Game {
    /// Conclude the active charter and hand control to the sealed homecoming report.
    pub(in crate::game) fn conclude_contract(&mut self, score: f32, level: contract::SuccessLevel) {
        self.last_mission_real_secs = self.mission_started.map(|t| (get_time() - t) as f32);
        self.mission_started = None;
        let GameState::Gameplay(gameplay) = &mut self.state else {
            return;
        };
        let sim = &mut gameplay.sim;
        let entry = completion_entry(sim, score, level);
        let homecoming_line = add_homecoming_log(sim, &self.data, &entry, score);
        let payout = apply_completion_payout(sim, &self.data, score);
        let template = sim
            .contract
            .as_ref()
            .and_then(|c| self.data.contracts.get(&c.template_id))
            .cloned();
        let legacy_line = template.as_ref().and_then(|template| {
            apply_charter_consequences(sim, &self.data, template, level, score)
        });
        sim.debrief = debrief::seal(
            sim,
            &self.data,
            score,
            level,
            payout,
            homecoming_line,
            legacy_line,
        );
        sim.contract = None;
        self.record_chronicle(entry);
    }

    fn record_chronicle(&mut self, entry: ChronicleEntry) {
        self.chronicle.record(entry);
        if self.instant_reveal {
            return;
        }
        if let Err(err) = self.chronicle.save(
            &self.data.config.game_name,
            &self.data.config.chronicle_slot,
            &self.data.config.version,
        ) {
            self.notifications
                .danger(format!("Chronicle write failed: {err}"));
        }
    }
}

fn add_homecoming_log(
    sim: &mut crate::state::sim::SimState,
    data: &GameData,
    entry: &ChronicleEntry,
    score: f32,
) -> Option<String> {
    let key = entry.outcome.to_lowercase();
    let line = data.config.flavor.homecoming_line(
        &key,
        entry.generation as usize,
        entry.duration_years,
        entry.generation,
    );
    if let Some(line) = &line {
        sim.push_log(line.clone());
    }
    sim.push_log(format!(
        "Contract concluded: {} — {} (score {score:.2}).",
        entry.contract_name, entry.outcome
    ));
    line
}

fn apply_completion_payout(
    sim: &mut crate::state::sim::SimState,
    data: &GameData,
    score: f32,
) -> ResourceDelta {
    let payout = sim
        .contract
        .as_ref()
        .and_then(|c| {
            data.contracts.get(&c.template_id).map(|template| {
                let reputation = contract::reputation_reward_multiplier(sim, template);
                contract::prorated_reward(&template.reward, c.objective_fraction() * reputation)
            })
        })
        .unwrap_or_default();
    sim.resources.apply(&payout);
    let shift = contract::mission_outcome_morale_shift(
        score,
        data.config.ship.mission_outcome_morale_scale,
    );
    if shift != 0.0 {
        sim.population.morale = (sim.population.morale + shift).clamp(0.0, 1.0);
    }
    payout
}

fn completion_entry(
    sim: &crate::state::sim::SimState,
    score: f32,
    level: contract::SuccessLevel,
) -> ChronicleEntry {
    ChronicleEntry {
        completed_year: sim.year(),
        contract_name: sim
            .contract
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default(),
        objective: sim
            .contract
            .as_ref()
            .map(|c| c.objective.label().to_owned())
            .unwrap_or_default(),
        legacy_id: sim.legacy.legacy_id.clone(),
        leader_name: sim
            .dynasty
            .leader()
            .map(|leader| leader.name.clone())
            .unwrap_or_else(|| "an empty chair".to_owned()),
        generation: sim.dynasty.generation,
        score,
        outcome: level.label().to_owned(),
        duration_years: sim
            .contract
            .as_ref()
            .map(|c| c.months_elapsed / 12)
            .unwrap_or_default(),
        command_posture: sim.command_posture,
        charter_approach: sim.contract.as_ref().map(|c| c.approach),
        homecoming_recovery: None,
    }
}

fn apply_charter_consequences(
    sim: &mut crate::state::sim::SimState,
    data: &GameData,
    template: &crate::data::contracts::ContractTemplate,
    level: contract::SuccessLevel,
    score: f32,
) -> Option<String> {
    let failed = level == contract::SuccessLevel::Failure;
    sim.apply_charter_outcome_faction_sentiment(data, template, failed);
    if failed {
        for operation in &template.failure_obligation_operations {
            sim.apply_obligation_operation(operation);
        }
        if let Some(line) = contract::apply_abandonment(sim, template) {
            sim.push_log(line);
        }
        add_consequence(sim, &template.failure_consequence);
        return None;
    }
    for operation in &template.completion_obligation_operations {
        sim.apply_obligation_operation(operation);
    }
    add_consequence(sim, &template.completion_consequence);
    let line = contract::apply_completion_reward(sim, template, score);
    if let Some(line) = &line {
        sim.push_log(line.clone());
    }
    line
}

fn add_consequence(sim: &mut crate::state::sim::SimState, consequence: &str) {
    if !consequence.is_empty() && !sim.consequences.iter().any(|tag| tag == consequence) {
        sim.consequences.push(consequence.to_owned());
    }
}
