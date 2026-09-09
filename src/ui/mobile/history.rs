use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    if let Some(id) = ctx.obligation_detail {
        f.action(
            "Back to obligations",
            true,
            UiAction::CloseObligationHistory,
        );
        if let Some(o) = ctx.sim.obligations.iter().find(|o| o.id == id) {
            f.heading(&o.title);
            f.text(&timing(o, ctx.sim.year()));
            f.text(&format!(
                "{} · To {} · Responsible {}\nMaterial: {}\nReputation: {}",
                o.status.label(),
                o.beneficiary,
                o.responsible,
                o.stakes.material,
                o.stakes.reputation
            ));
            for h in &o.history {
                f.text(&format!(
                    "Year {} · {} · {}\n{}",
                    h.year,
                    h.captain,
                    h.status.label(),
                    h.note
                ));
            }
        } else {
            f.text("This obligation is no longer available.");
        }
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
            f.section("Back to timeline", "");
            f.heading(&r.event_title);
            f.text(&format!(
                "{}\nFact: {}\nCommand log: {}\n{}'s house: {}",
                r.outcome_label, r.fact, r.official_account, r.captain, r.dynasty_account
            ));
            for a in &r.affected_accounts {
                f.text(&format!("{}: {}", a.people, a.account));
            }
            return;
        }
    }
    match section {
        "obligations" => {
            f.heading("Obligations");
            if ctx.sim.obligations.is_empty() {
                f.text("No obligations recorded. Promises and duties from your decisions will appear here with their deadlines and history.");
                return;
            }
            let active = ctx
                .sim
                .obligations
                .iter()
                .filter(|o| o.status.is_active())
                .count();
            f.text(&format!(
                "{active} active · {} resolved · {} due",
                ctx.sim.obligations.len() - active,
                ctx.sim.due_obligations().len()
            ));
            f.text("Active duties appear first. Read an obligation's history to see earlier promises and changes of responsibility.");
            let mut obligations = ctx.sim.obligations.iter().collect::<Vec<_>>();
            obligations.sort_by_key(|o| {
                (
                    !o.status.is_active(),
                    o.due_year.unwrap_or(u32::MAX),
                    o.created_year,
                )
            });
            for o in obligations {
                f.heading(&o.title);
                f.text(&timing(o, ctx.sim.year()));
                f.text(&format!(
                    "{} · To {}\nResponsible {}\nMaterial: {}\nReputation: {}",
                    o.status.label(),
                    o.beneficiary,
                    o.responsible,
                    o.stakes.material,
                    o.stakes.reputation
                ));
                if o.successions_crossed > 0 {
                    f.text(&format!(
                        "Inherited across {} successions",
                        o.successions_crossed
                    ));
                }
                f.action(
                    "Read full obligation history",
                    true,
                    UiAction::OpenObligationHistory(o.id.clone()),
                );
            }
        }
        "archive" => {
            crate::ui::chronicle::archive::build(ctx, f);
        }
        "milestones" => {
            f.heading("Milestones");
            for a in ctx.achievements.iter() {
                f.text(&format!(
                    "{}{} · {}",
                    if a.unlocked {
                        "Reached · "
                    } else {
                        "Not reached · "
                    },
                    a.name,
                    a.description
                ));
            }
        }
        _ => {
            f.heading("Voyage timeline");
            for (index, r) in ctx.sim.decision_records.iter().enumerate().rev() {
                f.section(
                    &format!("Year {} · {}", r.year, r.event_title),
                    &format!("record:{index}"),
                );
            }
            f.heading("Ship's log");
            for e in ctx.sim.log.iter().rev() {
                f.text(&format!("Year {} · {}", e.year, e.text));
            }
        }
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
