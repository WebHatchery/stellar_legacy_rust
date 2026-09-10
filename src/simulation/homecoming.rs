//! Between-voyages recovery: a single, bounded intervention after homecoming.

use crate::data::{GameData, PopulationDelta, ResourceDelta};
use crate::simulation::institutions;
use crate::state::sim::{
    HomecomingChoice, HomecomingFocus, HomecomingRecovery, HomecomingRecoveryRecord,
    InstitutionRecord, InstitutionRecordKind, ObligationOperation, SimState,
};

/// Build the report's recovery brief from the state that actually came home.
/// Priority is deliberate: a due promise is time-sensitive, then the people,
/// then the weakest piece of institutional craft.
pub fn build_plan(sim: &SimState, data: &GameData) -> HomecomingRecovery {
    if let Some(obligation) = oldest_active_obligation(sim) {
        let timing = obligation.due_year.map_or_else(
            || "has no fixed date".to_owned(),
            |year| format!("is due in Year {year}"),
        );
        return HomecomingRecovery {
            focus: HomecomingFocus::Obligation,
            target_id: obligation.id.clone(),
            target_label: obligation.title.clone(),
            situation: format!(
                "{} {timing}. The next charter can wait, but the people who trusted this ship cannot.",
                obligation.beneficiary
            ),
            resolved: false,
            choice: None,
        };
    }

    if sim.population.unity < 0.6
        || sim.population.morale < 0.6
        || sim.aboard_approval_mean() < 0.45
    {
        let faction = sim
            .factions
            .iter()
            .filter(|f| f.is_aboard() && f.members > 0)
            .min_by(|left, right| {
                left.approval
                    .total_cmp(&right.approval)
                    .then_with(|| left.faction_id.cmp(&right.faction_id))
            });
        let (target_id, target_label, approval) = faction
            .map(|f| {
                let label = data
                    .factions
                    .get(&f.faction_id)
                    .map_or_else(|| f.faction_id.clone(), |def| def.name.clone());
                (f.faction_id.clone(), label, f.approval)
            })
            .unwrap_or_else(|| ("commons".to_owned(), "the common table".to_owned(), 0.5));
        return HomecomingRecovery {
            focus: HomecomingFocus::Cohesion,
            target_id,
            target_label,
            situation: format!(
                "The returning crew is carrying a divided common life: unity {:.0}% and the most strained people at {:.0}% trust.",
                sim.population.unity * 100.0,
                approval * 100.0
            ),
            resolved: false,
            choice: None,
        };
    }

    let target = weakest_subsystem(sim, data);
    let (target_id, target_label, condition, knowledge) = target.unwrap_or_else(|| {
        (
            "education_culture".to_owned(),
            "the ship's schools".to_owned(),
            1.0,
            0.5,
        )
    });
    HomecomingRecovery {
        focus: HomecomingFocus::Institutions,
        target_id,
        target_label,
        situation: format!(
            "The ship came home intact enough to continue, but its weakest craft is only {:.0}% sound and {:.0}% remembered.",
            condition * 100.0,
            knowledge * 100.0
        ),
        resolved: false,
        choice: None,
    }
}

/// The resource bill for a choice, using the target that the service would
/// actually affect right now. Defer is intentionally free.
pub fn choice_cost(sim: &SimState, data: &GameData, choice: HomecomingChoice) -> ResourceDelta {
    match choice {
        HomecomingChoice::ReconcilePeople => ResourceDelta {
            credits: -data.config.homecoming_recovery.reconcile_credits,
            influence: -data.config.homecoming_recovery.reconcile_influence,
            ..Default::default()
        },
        HomecomingChoice::PreserveCraft => ResourceDelta {
            credits: -school_cost(sim, data),
            ..Default::default()
        },
        HomecomingChoice::HonorPromise => ResourceDelta {
            credits: -data.config.homecoming_recovery.honor_credits,
            influence: -data.config.homecoming_recovery.honor_influence,
            ..Default::default()
        },
        HomecomingChoice::Defer => ResourceDelta::default(),
    }
}

/// The configured consequence summary shown beside each recovery choice.
pub fn choice_effects(data: &GameData, choice: HomecomingChoice) -> String {
    let cfg = &data.config.homecoming_recovery;
    match choice {
        HomecomingChoice::ReconcilePeople => format!(
            "Effect: {:+.0}% morale · {:+.0}% unity · {:+.0}% stability · {:+.0}% weakest trust",
            cfg.reconcile_morale * 100.0,
            cfg.reconcile_unity * 100.0,
            cfg.reconcile_stability * 100.0,
            cfg.reconcile_approval * 100.0
        ),
        HomecomingChoice::PreserveCraft => format!(
            "Effect: {:+.0}% knowledge on the weakest discipline",
            cfg.preserve_knowledge_gain * 100.0
        ),
        HomecomingChoice::HonorPromise => format!(
            "Effect: fulfill oldest promise · {:+.0}% beneficiary trust",
            cfg.honor_approval * 100.0
        ),
        HomecomingChoice::Defer => format!(
            "Effect: {:+.0}% morale · {:+.0}% unity",
            -cfg.defer_morale_loss * 100.0,
            -cfg.defer_unity_loss * 100.0
        ),
    }
}

/// Whether the choice has a valid target and its full bill can be paid.
pub fn choice_available(sim: &SimState, data: &GameData, choice: HomecomingChoice) -> bool {
    choice_unavailable_reason(sim, data, choice).is_none()
}

/// Explain why a recovery card cannot be committed, if it cannot.
pub fn choice_unavailable_reason(
    sim: &SimState,
    data: &GameData,
    choice: HomecomingChoice,
) -> Option<String> {
    let target_exists = match choice {
        HomecomingChoice::ReconcilePeople => sim.factions.iter().any(|f| f.is_aboard()),
        HomecomingChoice::PreserveCraft => weakest_subsystem(sim, data).is_some(),
        HomecomingChoice::HonorPromise => oldest_active_obligation(sim).is_some(),
        HomecomingChoice::Defer => true,
    };
    if !target_exists {
        return Some(
            match choice {
                HomecomingChoice::ReconcilePeople => {
                    "No aboard faction can receive the commons grant."
                }
                HomecomingChoice::PreserveCraft => "No subsystem is available for a school.",
                HomecomingChoice::HonorPromise => "No active promise can be honored.",
                HomecomingChoice::Defer => "",
            }
            .to_owned(),
        );
    }
    let cost = choice_cost(sim, data, choice);
    let mut missing = Vec::new();
    if cost.credits < 0 && sim.resources.credits < -cost.credits {
        missing.push(format!(
            "{} more credits",
            -cost.credits - sim.resources.credits
        ));
    }
    if cost.influence < 0 && sim.resources.influence < -cost.influence {
        missing.push(format!(
            "{} more influence",
            -cost.influence - sim.resources.influence
        ));
    }
    if missing.is_empty() {
        None
    } else {
        Some(format!("Needs {}.", missing.join(" and ")))
    }
}

/// Apply a recovery choice to the live campaign and mark the sealed report as
/// resolved. The plan is cloned before mutation so the report remains a clean
/// historical snapshot while the campaign receives the consequence.
pub fn apply_choice(
    sim: &mut SimState,
    data: &GameData,
    choice: HomecomingChoice,
) -> Result<String, String> {
    let plan = sim
        .debrief
        .as_ref()
        .and_then(|report| report.recovery.clone())
        .ok_or_else(|| "No homecoming recovery is waiting.".to_owned())?;
    if plan.resolved {
        return Err("The homecoming recovery has already been recorded.".to_owned());
    }
    if let Some(reason) = choice_unavailable_reason(sim, data, choice) {
        return Err(reason);
    }

    let note = match choice {
        HomecomingChoice::ReconcilePeople => reconcile_people(sim, data)?,
        HomecomingChoice::PreserveCraft => preserve_craft(sim, data)?,
        HomecomingChoice::HonorPromise => honor_promise(sim, data)?,
        HomecomingChoice::Defer => {
            sim.population.apply(&PopulationDelta {
                morale: -data.config.homecoming_recovery.defer_morale_loss,
                unity: -data.config.homecoming_recovery.defer_unity_loss,
                ..Default::default()
            });
            "Recovery deferred; the next charter inherits the unresolved wound.".to_owned()
        }
    };

    sim.push_log(format!("HOMECOMING RECOVERY — {note}"));
    sim.homecoming_recovery_history
        .push(HomecomingRecoveryRecord {
            year: sim.year(),
            focus: plan.focus,
            choice,
            target_label: plan.target_label.clone(),
            note: note.clone(),
        });
    if let Some(report) = sim.debrief.as_mut() {
        if let Some(recovery) = report.recovery.as_mut() {
            recovery.resolved = true;
            recovery.choice = Some(choice);
        }
    }
    Ok(note)
}

fn reconcile_people(sim: &mut SimState, data: &GameData) -> Result<String, String> {
    let cost = choice_cost(sim, data, HomecomingChoice::ReconcilePeople);
    sim.resources.apply(&cost);
    sim.population.apply(&PopulationDelta {
        morale: data.config.homecoming_recovery.reconcile_morale,
        unity: data.config.homecoming_recovery.reconcile_unity,
        stability: data.config.homecoming_recovery.reconcile_stability,
        ..Default::default()
    });
    let faction = sim
        .factions
        .iter_mut()
        .filter(|f| f.is_aboard() && f.members > 0)
        .min_by(|left, right| {
            left.approval
                .total_cmp(&right.approval)
                .then_with(|| left.faction_id.cmp(&right.faction_id))
        })
        .ok_or_else(|| "No people remain aboard to reconcile.".to_owned())?;
    faction.adjust_approval(data.config.homecoming_recovery.reconcile_approval);
    Ok(format!(
        "{} received a commons grant: morale, unity, stability, and trust rose.",
        faction.faction_id
    ))
}

fn preserve_craft(sim: &mut SimState, data: &GameData) -> Result<String, String> {
    let (id, name, _, before) =
        weakest_subsystem(sim, data).ok_or_else(|| "No subsystem needs a school.".to_owned())?;
    institutions::establish_or_support_school(sim, data, &id)?;
    let gain = data.config.homecoming_recovery.preserve_knowledge_gain;
    let after = if let Some(state) = sim.subsystems.get_mut(&id) {
        state.knowledge = (state.knowledge + gain).clamp(0.0, 1.0);
        state.knowledge
    } else {
        before
    };
    sim.institution_records.push(InstitutionRecord {
        year: sim.year(),
        kind: InstitutionRecordKind::ExpertisePreserved,
        subject: format!("{name} school"),
        discipline: id,
        knowledge_change: after - before,
    });
    Ok(format!(
        "the {name} school carried knowledge from {:.0}% to {:.0}%.",
        before * 100.0,
        after * 100.0
    ))
}

fn honor_promise(sim: &mut SimState, data: &GameData) -> Result<String, String> {
    let obligation = oldest_active_obligation(sim)
        .cloned()
        .ok_or_else(|| "No active promise can be honored.".to_owned())?;
    sim.resources
        .apply(&choice_cost(sim, data, HomecomingChoice::HonorPromise));
    sim.apply_obligation_operation(&ObligationOperation::Fulfil {
        authored_id: obligation.authored_id.clone(),
        note: "The homecoming recovery fund honored the oldest active promise between voyages."
            .to_owned(),
    });
    let beneficiary = obligation.beneficiary.clone();
    let approval_gain = data.config.homecoming_recovery.honor_approval;
    for faction in sim.factions.iter_mut().filter(|f| f.is_aboard()) {
        if data.factions.get(&faction.faction_id).is_some_and(|def| {
            beneficiary.contains(&def.name) || beneficiary.contains(&def.log_name)
        }) {
            faction.adjust_approval(approval_gain);
        }
    }
    Ok(format!(
        "{} was fulfilled for {}.",
        obligation.title, obligation.beneficiary
    ))
}

fn oldest_active_obligation(sim: &SimState) -> Option<&crate::state::sim::Obligation> {
    sim.active_obligations().min_by_key(|obligation| {
        (
            obligation.due_year.unwrap_or(u32::MAX),
            obligation.created_year,
            obligation.id.as_str(),
        )
    })
}

fn weakest_subsystem(sim: &SimState, data: &GameData) -> Option<(String, String, f32, f32)> {
    GameData::sorted_ids(&data.subsystems)
        .into_iter()
        .filter_map(|id| {
            let state = sim.subsystems.get(&id)?;
            let name = data.subsystems.get(&id)?.name.clone();
            Some((id, name, state.condition, state.knowledge))
        })
        .min_by(|left, right| {
            let left_score = left.2 * 0.45 + left.3 * 0.55;
            let right_score = right.2 * 0.45 + right.3 * 0.55;
            left_score
                .total_cmp(&right_score)
                .then_with(|| left.0.cmp(&right.0))
        })
}

fn school_cost(sim: &SimState, data: &GameData) -> i64 {
    let id = weakest_subsystem(sim, data)
        .map(|(id, _, _, _)| id)
        .unwrap_or_default();
    if sim
        .subsystem_schools
        .iter()
        .any(|school| school.subsystem_id == id)
    {
        data.config.crew.school_upkeep_credits
    } else {
        data.config.crew.school_cost_credits
    }
}

#[cfg(test)]
mod tests;
