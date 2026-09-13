//! The sealed report, including every metric and promise, at reading size.
use super::*;

pub(super) fn draw(ctx: &GameplayCtx<'_>, report: &VoyageDebrief, area: Rect, pointer: Pointer) {
    let text = if ctx.presentation.report_page.get() == 2 {
        let mut text = String::from("Defining moments\n");
        if report.highlights.is_empty() {
            text.push_str("\nNo notable council decisions or milestones recorded.");
        }
        for beat in &report.highlights {
            text.push_str(&format!(
                "\nY{}.{} · {}\n{}\n",
                beat.year,
                beat.month,
                beat.kind.tag(),
                beat.text
            ));
        }
        text
    } else {
        accounting(report)
    };
    crate::ui::reading::read_lines(
        &text,
        column_panel(area, "Homecoming record").inset(8.0),
        if ctx.presentation.report_page.get() == 2 {
            ctx.debrief_log_scroll
        } else {
            &ctx.presentation.report_scroll
        },
        pointer,
        term::dim(),
    );
}

pub(crate) fn accounting(r: &VoyageDebrief) -> String {
    let mut text = format!("{}\n\n{} years under way · {} generations passed · {} survivors aboard\nPopulation {} → {} ({:+}) · Y{}–Y{}\nCommand: {} · Charter approach: {} · Custodian: {} (perceived {:.0}%)\n\nFinal score {:.2} · {}\n", r.homecoming_line.as_deref().unwrap_or("The ship has returned."), r.duration_years, r.generations, r.population_end, r.population_start, r.population_end, r.population_change(), r.began_year, r.ended_year, r.command_posture.label(), r.approach.label(), r.custodian_disposition(), r.custodian_empathy.clamp(0.0,1.0)*100.0, r.score, r.outcome);
    let (reached, total) = r.milestones_reached();
    text.push_str(&format!(
        "Marks reached {reached}/{total} · {}\n",
        if r.unpaid() {
            "No payout"
        } else {
            "Payout received"
        }
    ));
    text.push_str(&format!(
        "\nCHARTER APPROACH\n{}\n{}\n",
        r.approach.label(),
        crate::simulation::approach::effect_summary(r.approach)
    ));
    text.push_str("\nHOMECOMING RECOVERY\n");
    text.push_str(&recovery_accounting(r));
    text.push('\n');
    let p = &r.payout;
    text.push_str(&format!("\nPAID\nCredits {:+} · Energy {:+} · Minerals {:+} · Food {:+} · Influence {:+}\n\nSCORE ACCOUNTING\n",p.credits,p.energy,p.minerals,p.food,p.influence));
    for m in &r.metrics {
        text.push_str(&format!(
            "{}: {:.2} / {:.2} · weight {:.2} · contribution +{:.3}\n",
            m.name,
            m.current,
            m.target,
            m.weight,
            m.contribution()
        ));
    }
    text.push_str("\nMILESTONES\n");
    for m in &r.milestones {
        text.push_str(&format!(
            "{} · {}\n",
            if m.reached { "Reached" } else { "Not reached" },
            m.name
        ));
    }
    text.push_str("\nPROMISES\n");
    if r.obligations.is_empty() {
        text.push_str("No promise changed this voyage.\n");
    }
    for o in &r.obligations {
        text.push_str(&format!(
            "{} · {}\nTo {} · responsible {}\nMaterial: {} · Reputation: {}\n",
            o.title,
            o.status.label(),
            o.beneficiary,
            o.responsible,
            o.stakes.material,
            o.stakes.reputation
        ));
        for h in &o.history {
            text.push_str(&format!("Y{} · {}\n", h.year, h.note));
        }
        text.push('\n');
    }
    text.push_str("\nCRAFT & SCHOOLS\n");
    if r.institutions.is_empty() {
        text.push_str("No institutional turning this voyage.");
    }
    for i in &r.institutions {
        text.push_str(&format!(
            "Y{} · {:?} · {} · {} · knowledge {:+.0}%\n",
            i.year,
            i.kind,
            i.subject,
            i.discipline,
            i.knowledge_change * 100.0
        ));
    }
    text
}

pub(crate) fn recovery_accounting(r: &VoyageDebrief) -> String {
    let Some(recovery) = r.recovery.as_ref() else {
        return "No recovery brief recorded.".to_owned();
    };
    if recovery.resolved {
        return format!(
            "Recorded · {} · {}\nTarget: {}",
            recovery.focus.label(),
            recovery
                .choice
                .map_or("Choice unavailable".to_owned(), |choice| choice
                    .label()
                    .to_owned()),
            recovery.target_label
        );
    }
    format!(
        "Pending review · {}\nTarget: {}\n{}",
        recovery.focus.label(),
        recovery.target_label,
        recovery.situation
    )
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/debrief/report/tests.rs"
    ));
}
