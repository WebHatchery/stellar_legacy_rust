//! Persistent consequences and maintenance transitions.

use crate::data::events::IssueSpec;
use crate::data::GameData;
use crate::state::sim::{Issue, IssueSeverity, SimState};

/// Create or merge an authored aftermath issue. Repeated event outcomes never
/// duplicate the same concern; the stronger severity and earliest due date win.
pub fn record_event_issue(sim: &mut SimState, spec: &IssueSpec, source: &str) {
    let due_month = spec
        .due_months
        .map(|months| sim.month_clock.saturating_add(months));
    if let Some(issue) = sim
        .issues
        .active
        .iter_mut()
        .find(|issue| issue.id == spec.id)
    {
        issue.food_production_penalty = issue
            .food_production_penalty
            .max(spec.food_production_penalty);
        issue.severity = issue.severity.max(spec.severity);
        issue.due_month = match (issue.due_month, due_month) {
            (Some(current), Some(next)) => Some(current.min(next)),
            (None, next) => next,
            (current, None) => current,
        };
        for project_id in &spec.recovery_project_ids {
            if !issue.recovery_project_ids.contains(project_id) {
                issue.recovery_project_ids.push(project_id.clone());
            }
        }
        return;
    }
    sim.issues.active.push(Issue {
        food_production_penalty: spec.food_production_penalty,
        id: spec.id.clone(),
        source: source.to_owned(),
        target: spec.target.clone(),
        created_month: sim.month_clock,
        severity: spec.severity,
        due_month,
        recovery_project_ids: spec.recovery_project_ids.clone(),
        acknowledged: false,
        overdue_effect_applied: false,
        resolved_month: None,
        resolution: None,
    });
    sim.push_log(format!(
        "Persistent concern opened: {}.",
        spec.id.replace('_', " ")
    ));
}

pub fn resolve_issue(sim: &mut SimState, id: &str, resolution: &str) -> bool {
    let Some(index) = sim.issues.active.iter().position(|issue| issue.id == id) else {
        return false;
    };
    let mut issue = sim.issues.active.remove(index);
    issue.resolved_month = Some(sim.month_clock);
    issue.resolution = Some(resolution.to_owned());
    sim.issues.resolved.push(issue);
    true
}

pub fn issue_allows_project(sim: &SimState, id: &str) -> bool {
    sim.issues.has_active(id)
}

/// Open one maintenance notice per subsystem/type when condition crosses the
/// authored threshold. Recovery hysteresis is provided by resolution: the same
/// id cannot reopen until it has genuinely been resolved.
pub fn refresh_maintenance_issues(sim: &mut SimState, data: &GameData) {
    let threshold = data.config.projects.maintenance_condition_threshold;
    let ids = GameData::sorted_ids(&data.subsystems);
    for id in ids {
        let Some(state) = sim.subsystems.get(&id) else {
            continue;
        };
        if state.condition > threshold {
            continue;
        }
        let issue_id = format!("maintenance:{id}");
        if sim.issues.has_active(&issue_id) {
            continue;
        }
        let spec = IssueSpec {
            food_production_penalty: 0.0,
            id: issue_id,
            target: id.clone(),
            severity: if state.condition <= threshold * 0.65 {
                IssueSeverity::Critical
            } else {
                IssueSeverity::Vulnerable
            },
            due_months: Some(data.config.projects.maintenance_due_months),
            recovery_project_ids: vec!["service_subsystem".to_owned()],
        };
        record_event_issue(sim, &spec, "maintenance threshold");
    }
}

/// Apply at most one authored overdue consequence per active maintenance issue.
/// The notice remains linked to the consequence so the player can see why the
/// new damage happened and still recover the target through Agenda work.
pub fn apply_overdue_maintenance(sim: &mut SimState, data: &GameData) {
    let due_ids: Vec<String> = sim
        .issues
        .active
        .iter()
        .filter(|issue| {
            issue.due_month.is_some_and(|due| sim.month_clock >= due)
                && !issue.overdue_effect_applied
        })
        .map(|issue| issue.id.clone())
        .collect();
    for issue_id in due_ids {
        let target = sim
            .issues
            .active
            .iter()
            .find(|issue| issue.id == issue_id)
            .map(|issue| issue.target.clone())
            .unwrap_or_default();
        if target == "engineering_bay" {
            sim.ship.hull_integrity =
                (sim.ship.hull_integrity - data.config.projects.overdue_hull_damage).max(0.0);
            sim.push_log(format!(
                "Maintenance warning {issue_id} became a linked hull failure; the earlier notice was not ignored by the record."
            ));
        } else if target == "life_support_habitat" {
            sim.ship.life_support =
                (sim.ship.life_support - data.config.projects.overdue_life_support_damage).max(0.0);
            sim.push_log(format!(
                "Maintenance warning {issue_id} became a linked air-system failure; the earlier notice was not ignored by the record."
            ));
        } else {
            sim.push_log(format!(
                "Maintenance warning {issue_id} became a linked {target} failure."
            ));
            if let Some(state) = sim.subsystems.get_mut(&target) {
                state.condition = (state.condition - 0.08).max(0.0);
            }
        }
        if let Some(issue) = sim
            .issues
            .active
            .iter_mut()
            .find(|issue| issue.id == issue_id)
        {
            issue.overdue_effect_applied = true;
            issue.severity = IssueSeverity::Critical;
        }
    }
}

#[cfg(test)]
mod tests;
