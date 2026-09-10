//! A shared, wrapping archive for the long memory between campaigns.
use super::*;
use crate::ui::mobile::form::Form;

pub(crate) fn recovery_record_heading(
    record: &crate::state::sim::HomecomingRecoveryRecord,
) -> String {
    format!(
        "Year {} · {} · {}",
        record.year,
        record.focus.label(),
        record.choice.label()
    )
}

#[cfg(test)]
mod tests;

pub(super) fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer) {
    term_panel(area, Some("MISSION ARCHIVE"));
    let mut form = Form::new();
    build(ctx, &mut form);
    let mut view = area.inset(24.0);
    view.y += 24.0;
    view.h -= 24.0;
    form.draw(
        view,
        ctx.presentation,
        pointer,
        "mission-archive",
        &mut Vec::new(),
    );
}

pub(crate) fn build(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Mission archive");
    form.text("This archive survives across campaigns. Recorded voyage scores build renown; your heritage grants a head start when you found a new dynasty.");
    let heritage = crate::heritage::derive(ctx.chronicle, &ctx.data.config.heritage);
    form.heading(&format!(
        "{} heritage · {} renown",
        heritage.tier_name, heritage.renown
    ));
    form.text(&format!(
        "New-dynasty bonus: {} credits · {} influence · {} tradition",
        heritage.credits, heritage.influence, heritage.tradition
    ));
    if let Some(next) = crate::heritage::next_tier(heritage.renown, &ctx.data.config.heritage) {
        form.text(&format!(
            "Next: {} · {} more renown\nNew-dynasty bonus at that tier: {} credits · {} influence · {} tradition",
            next.name, next.min_renown - heritage.renown, next.credits, next.influence, next.tradition
        ));
    } else {
        form.text("Highest heritage tier reached.");
    }
    let mut recovery_records: Vec<&crate::state::sim::HomecomingRecoveryRecord> = ctx
        .chronicle
        .entries
        .iter()
        .filter_map(|entry| entry.homecoming_recovery.as_ref())
        .collect();
    for record in &ctx.sim.homecoming_recovery_history {
        if !recovery_records.contains(&record) {
            recovery_records.push(record);
        }
    }
    recovery_records.sort_by_key(|record| std::cmp::Reverse(record.year));
    form.heading("Between-voyages recovery");
    if recovery_records.is_empty() {
        form.text("No recovery interventions recorded yet. Resolve the homecoming brief after a voyage to begin this ledger.");
    } else {
        form.text(&format!(
            "{} interventions recorded · Most recent first",
            recovery_records.len()
        ));
        for record in recovery_records {
            form.heading(&recovery_record_heading(record));
            form.text(&format!("Target: {}\n{}", record.target_label, record.note));
        }
    }
    if ctx.chronicle.entries.is_empty() {
        form.heading("No voyages recorded yet");
        form.text("When a voyage ends, its report is recorded here. Your current voyage's decisions and promises are in Timeline and Obligations.");
        return;
    }
    let stats = ctx.chronicle.stats();
    form.heading("Voyage record");
    form.text(&format!(
        "{} voyages · {} complete · {} years flown\nAverage score {:.0}% · Most recent first",
        stats.voyages,
        stats.completed,
        stats.years_flown,
        stats.average_score * 100.0
    ));
    for entry in ctx.chronicle.entries.iter().rev() {
        form.heading(&format!(
            "Year {} · {}",
            entry.completed_year, entry.contract_name
        ));
        form.text(&format!(
            "{} · Score {:.0}%\n{} charter · {} years\nCaptain {} · Generation {}\nCommand posture: {} · Approach: {}",
            entry.outcome, entry.score * 100.0, entry.objective, entry.duration_years,
            entry.leader_name,
            entry.generation,
            entry.command_posture.label(),
            entry
                .charter_approach
                .map_or("OLDER RECORD", |approach| approach.label())
        ));
    }
}
