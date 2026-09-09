use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    f.sections(&[
        ("Timeline", ""),
        ("Obligations", "obligations"),
        ("Mission archive", "archive"),
        ("Milestones", "milestones"),
    ]);
    if let Some(id) = ctx.obligation_detail {
        if let Some(o) = ctx.sim.obligations.iter().find(|o| o.id == id) {
            f.heading(&o.title);
            f.text(&format!(
                "{} · To {} · Responsible {}\nMaterial: {}\nReputation: {}",
                o.status.label(),
                o.beneficiary,
                o.responsible,
                o.stakes.material,
                o.stakes.reputation
            ));
            for h in &o.history {
                f.text(&format!("Year {} · {}", h.year, h.note));
            }
            f.action(
                "Close obligation history",
                true,
                UiAction::CloseObligationHistory,
            );
            return;
        }
    }
    if let Some(index) = section
        .strip_prefix("record:")
        .and_then(|i| i.parse::<usize>().ok())
    {
        if let Some(r) = ctx.sim.decision_records.iter().rev().nth(index) {
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
            for o in &ctx.sim.obligations {
                f.heading(&o.title);
                f.text(&format!(
                    "{} · To {}\nResponsible {} · Due {}\nMaterial: {}\nReputation: {}",
                    o.status.label(),
                    o.beneficiary,
                    o.responsible,
                    o.due_year
                        .map_or("open".to_owned(), |y| format!("Year {y}")),
                    o.stakes.material,
                    o.stakes.reputation
                ));
                f.action(
                    "Read full obligation history",
                    true,
                    UiAction::OpenObligationHistory(o.id.clone()),
                );
            }
        }
        "archive" => {
            f.heading("Mission archive");
            for e in &ctx.chronicle.entries {
                f.text(&format!(
                    "{} · {} · Score {:.2}",
                    e.contract_name, e.outcome, e.score
                ));
            }
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
            for (index, r) in ctx.sim.decision_records.iter().rev().enumerate() {
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
