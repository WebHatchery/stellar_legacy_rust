//! Mobile Chronicle and obligation-history sections.

use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    if let Some(id) = ctx.obligation_detail {
        build_obligation_detail(ctx, f, id);
        return;
    }
    f.sections(&[
        ("Timeline", ""),
        ("Obligations", "obligations"),
        ("Mission archive", "archive"),
        ("Milestones", "milestones"),
    ]);
    if let Some(index) = section
        .strip_prefix("record:")
        .and_then(|i| i.parse::<usize>().ok())
    {
        if let Some(r) = ctx.sim.decision_records.get(index) {
            build_record(f, r);
            return;
        }
    }
    match section {
        "obligations" => build_obligations(ctx, f),
        "archive" => {
            crate::ui::chronicle::archive::build(ctx, f);
        }
        "milestones" => {
            crate::ui::chronicle::milestones::build(ctx, f);
        }
        _ => build_timeline(ctx, f),
    }
}

fn build_obligation_detail(ctx: &GameplayCtx<'_>, form: &mut Form, id: &str) {
    form.action(
        "Back to obligations",
        true,
        UiAction::CloseObligationHistory,
    );
    if let Some(obligation) = ctx.sim.obligations.iter().find(|o| o.id == id) {
        form.heading(&obligation.title);
        form.text(&timing(obligation, ctx.sim.year()));
        form.text(&format!(
            "{} · To {} · Responsible {}\nMaterial: {}\nReputation: {}",
            obligation.status.label(),
            obligation.beneficiary,
            obligation.responsible,
            obligation.stakes.material,
            obligation.stakes.reputation
        ));
        for history in &obligation.history {
            form.text(&format!(
                "Year {} · {} · {}\n{}",
                history.year,
                history.captain,
                history.status.label(),
                history.note
            ));
        }
    } else {
        form.text("This obligation is no longer available.");
    }
}

fn build_record(form: &mut Form, record: &crate::state::sim::DecisionRecord) {
    form.section("Back to timeline", "");
    form.heading(&record.event_title);
    form.text(&format!(
        "{}\nFact: {}\nCommand log: {}\n{}'s house: {}",
        record.outcome_label,
        record.fact,
        record.official_account,
        record.captain,
        record.dynasty_account
    ));
    for account in &record.affected_accounts {
        form.text(&format!("{}: {}", account.people, account.account));
    }
}

fn build_obligations(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Obligations");
    if ctx.sim.obligations.is_empty() {
        form.text("No obligations recorded. Promises and duties from your decisions will appear here with their deadlines and history.");
        return;
    }
    let active = ctx
        .sim
        .obligations
        .iter()
        .filter(|obligation| obligation.status.is_active())
        .count();
    form.text(&format!(
        "{active} active · {} resolved · {} due",
        ctx.sim.obligations.len() - active,
        ctx.sim.due_obligations().len()
    ));
    form.text("Active duties appear first. Read an obligation's history to see earlier promises and changes of responsibility.");
    let mut obligations = ctx.sim.obligations.iter().collect::<Vec<_>>();
    obligations.sort_by_key(|obligation| {
        (
            !obligation.status.is_active(),
            obligation.due_year.unwrap_or(u32::MAX),
            obligation.created_year,
        )
    });
    for obligation in obligations {
        form.heading(&obligation.title);
        form.text(&timing(obligation, ctx.sim.year()));
        form.text(&format!(
            "{} · To {}\nResponsible {}\nMaterial: {}\nReputation: {}",
            obligation.status.label(),
            obligation.beneficiary,
            obligation.responsible,
            obligation.stakes.material,
            obligation.stakes.reputation
        ));
        if obligation.successions_crossed > 0 {
            form.text(&format!(
                "Inherited across {} successions",
                obligation.successions_crossed
            ));
        }
        form.action(
            "Read full obligation history",
            true,
            UiAction::OpenObligationHistory(obligation.id.clone()),
        );
    }
}

fn build_timeline(ctx: &GameplayCtx<'_>, form: &mut Form) {
    form.heading("Voyage timeline");
    for (index, record) in ctx.sim.decision_records.iter().enumerate().rev() {
        form.section(
            &format!("Year {} · {}", record.year, record.event_title),
            &format!("record:{index}"),
        );
    }
    form.heading("Ship's log");
    for entry in ctx.sim.log.iter().rev() {
        form.text(&format!("Year {} · {}", entry.year, entry.text));
    }
}

fn timing(o: &crate::state::sim::Obligation, year: u32) -> String {
    if !o.status.is_active() {
        return format!(
            "Closed in Year {}",
            o.history.last().map_or(o.created_year, |h| h.year)
        );
    }
    match o.due_year {
        None => "No fixed deadline".to_owned(),
        Some(due) if due < year => format!("Due Year {due} · {} years overdue", year - due),
        Some(due) if due == year => format!("Due now · Year {due}"),
        Some(due) => format!("Due Year {due} · in {} years", due - year),
    }
}
