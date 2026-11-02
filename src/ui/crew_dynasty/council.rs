//! One readable council brief across display sizes.
use super::*;
use crate::data::events::EventCategory;
use crate::ui::form::Form;

pub(super) fn draw(
    ctx: &GameplayCtx<'_>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    term_panel(rect, Some("CUSTODIAN & CAPTAIN"));
    let mut form = Form::new();
    build(ctx, &mut form);
    let mut view = rect.inset(18.0);
    view.y += 24.0;
    view.h -= 24.0;
    form.draw(view, ctx.presentation, pointer, "council-brief", actions);
}

pub(crate) fn build(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    form.heading("Who decides?");
    form.text(
        "Delegated kinds of event are settled by advisors; story dilemmas always come to you.",
    );
    for category in EventCategory::ALL {
        let delegated = sim.delegation.is_delegated(category);
        form.action(
            &format!(
                "{} · {}",
                category.label(),
                if delegated {
                    "Automatic · Return to council review"
                } else {
                    "Council review · Delegate"
                }
            ),
            true,
            UiAction::ToggleDelegation(category),
        );
    }
    form.heading("Command & succession");
    form.text("The Custodian handles routine operations under the standing mandate.");
    let captain = sim
        .dynasty
        .leader()
        .map_or("No captain", |p| p.name.as_str());
    form.text(&format!(
        "Captain: {captain}\nPriority: {}",
        sim.authority.captain_priority.label()
    ));
    form.text(&format!(
        "Generation {} · Next generation in {} years",
        sim.dynasty.generation,
        ctx.data
            .config
            .generation_interval_years
            .saturating_sub(sim.dynasty.years_since_generation)
    ));
    if let Some(heir) = planned_heir(&sim.dynasty, &ctx.data.config) {
        form.text(&format!(
            "{}: {} · Leadership {}",
            if sim.dynasty.designated_heir == Some(heir.id) {
                "Named heir"
            } else {
                "Automatic successor"
            },
            heir.name,
            heir.leadership
        ));
    } else {
        form.text("No eligible heir. Review the Family roster.");
    }
    let vacancies = ctx
        .data
        .crew_archetypes
        .iter()
        .filter(|post| post_holder(sim, &post.id).is_none())
        .count();
    form.text(&format!(
        "Prepared apprentices: {} · Vacant officer posts: {vacancies}",
        sim.apprenticeships.len()
    ));

    form.heading("Legacy condition");
    legacy_condition_lines(ctx, form);
}

/// Only conditions the voyage has actually accrued are listed; a clean record
/// reads as tradition alone, and a zero failure risk is not reported.
fn legacy_condition_lines(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    let legacy = &sim.legacy;
    let mut lines = vec![format!("Tradition points: {}", legacy.tradition_points)];
    if legacy.body_horror_events > 0 {
        lines.push(format!("Body-horror events: {}", legacy.body_horror_events));
    }
    if legacy.existential_dread >= 0.005 {
        lines.push(format!(
            "Existential dread: {:.2}",
            legacy.existential_dread
        ));
    }
    if legacy.piracy_reputation >= 0.005 {
        lines.push(format!(
            "Piracy reputation: {:.2}",
            legacy.piracy_reputation
        ));
    }
    form.text(&lines.join("\n"));
    let risk = crate::simulation::legacy::failure_risk(sim, &ctx.data.config);
    if risk.total == 0 && !risk.at_risk {
        return;
    }
    let risk_name = ctx
        .data
        .legacies
        .get(&legacy.legacy_id)
        .map(|definition| definition.failure_risk.replace('_', " "))
        .unwrap_or_else(|| "Legacy failure".to_owned());
    form.text(&format!(
        "{risk_name}: {} · {}",
        risk.total,
        if risk.at_risk { "AT RISK" } else { "STABLE" }
    ));
    for factor in &risk.factors {
        form.text(&format!("{} · +{} risk", factor.label, factor.points));
    }
}
