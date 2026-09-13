//! Mobile subsystem systems and custody controls.

use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    let sim = ctx.sim;
    if let Some(id) = ctx.custody_picker {
        build_custody_picker(ctx, f, id);
        return;
    }
    f.heading("Compartments");
    for id in GameData::sorted_ids(&ctx.data.subsystems) {
        let def = ctx.data.subsystems.get(&id).expect("sorted registry id");
        let Some(s) = sim.subsystems.get(&id) else {
            continue;
        };
        f.section(
            &format!("{} · {:.0}% condition", def.name, s.condition * 100.0),
            &format!("system:{id}"),
        );
        if section != format!("system:{id}") {
            continue;
        }
        build_system_detail(ctx, f, &id, def, s);
    }
}

fn build_custody_picker(ctx: &GameplayCtx<'_>, form: &mut Form, id: &str) {
    form.action(
        "Cancel custody selection",
        true,
        UiAction::CancelDisciplineCustody,
    );
    form.heading("Choose the discipline's custodians");
    if let Some(definition) = ctx.data.subsystems.get(id) {
        form.text(&definition.name);
    }
    form.text("Choose one people aboard to care for this discipline. Their approval affects care while the school is funded; this appointment cannot be reassigned here.");
    let cost = ctx.data.config.crew.custody_influence_cost;
    form.text(&format!("Grant costs {cost} influence"));
    for people in ctx.sim.factions.iter().filter(|p| p.is_aboard()) {
        let name = ctx
            .data
            .factions
            .get(&people.faction_id)
            .map_or(people.faction_id.as_str(), |p| p.name.as_str());
        form.action(
            name,
            ctx.sim.resources.influence >= cost,
            UiAction::GrantDisciplineCustody {
                subsystem_id: id.to_owned(),
                faction_id: people.faction_id.clone(),
            },
        );
    }
}

fn build_system_detail(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    id: &str,
    definition: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
) {
    let sim = ctx.sim;
    let cfg = &ctx.data.config;
    form.text(&format!(
        "{} · Tier {}\nCondition {:.0}% · Knowledge {:.0}%\nRepair requires {:.0}% knowledge",
        definition.fitting_name(state.tier),
        state.tier,
        state.condition * 100.0,
        state.knowledge * 100.0,
        definition.repair_knowledge_required * 100.0
    ));
    form.text(&definition.description);
    build_culture_detail(ctx, form, id, state);
    let target = crate::simulation::subsystems::repair_target_condition(sim, ctx.data, id)
        .unwrap_or(state.condition);
    if target > state.condition {
        form.action(
            &format!(
                "Repair to {:.0}% · {} parts · {} minerals",
                target * 100.0,
                definition.repair_parts_cost,
                definition.repair_minerals_cost
            ),
            state.knowledge >= definition.repair_knowledge_required
                && sim.ship.spare_parts >= definition.repair_parts_cost
                && sim.resources.minerals >= definition.repair_minerals_cost,
            UiAction::RepairSubsystem(id.to_owned()),
        );
    } else {
        form.text("Condition sound for current repair capability");
    }
    build_fitting_detail(ctx, form, id, definition, state);
    let train = crate::simulation::subsystems::training_target_knowledge(sim, ctx.data, id)
        .unwrap_or(state.knowledge);
    if train > state.knowledge {
        form.action(
            &format!(
                "Train to {:.0}% · {} credits",
                train * 100.0,
                cfg.subsystems.train_cost_credits
            ),
            sim.resources.credits >= cfg.subsystems.train_cost_credits,
            UiAction::TrainSubsystemKnowledge(id.to_owned()),
        );
    }
    super::institutions::build(ctx, form, id);
}

fn build_culture_detail(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    id: &str,
    state: &crate::state::sim::subsystems::SubsystemState,
) {
    let Some(culture) =
        crate::simulation::culture::descriptor(ctx.data, id, &state.culture.descriptor_id)
    else {
        return;
    };
    let custodian = state
        .culture
        .custodian_faction_id
        .as_deref()
        .and_then(|faction_id| ctx.data.factions.get(faction_id))
        .map_or("No local custodian", |faction| faction.name.as_str());
    form.heading(&format!("Local culture · {}", culture.label));
    form.text(&format!(
        "Custodian: {custodian}\n{}\n{}",
        crate::simulation::culture::effect_summary(culture),
        culture.description
    ));
    form.text(&format!(
        "Remembered: {}\nGrievance: {}",
        state
            .culture
            .remembered_event
            .as_deref()
            .unwrap_or("No compartment memory recorded."),
        state
            .culture
            .grievance
            .as_deref()
            .unwrap_or("No active grievance."),
    ));
}

fn build_fitting_detail(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    id: &str,
    definition: &crate::data::subsystems::SubsystemDef,
    state: &crate::state::sim::subsystems::SubsystemState,
) {
    let sim = ctx.sim;
    let Some(next) = definition.next_fitting(state.tier) else {
        form.text("Highest fitting tier reached.");
        return;
    };
    form.heading(&format!(
        "Next fitting: {} · Tier {}",
        next.name,
        state.tier + 1
    ));
    if !next.description.is_empty() {
        form.text(&next.description);
    }
    form.text("Installing an upgrade changes the fitting's tier; it does not restore condition or knowledge.");
    if sim.contract.is_some() {
        form.text("Install in port between voyages.");
    }
    if next.acquisition.is_mission_only() {
        if sim.ship.unlocked_fittings.contains(&next.id) {
            form.text("Recovered fitting available. Installation is free in port.");
            form.action(
                "Install recovered fitting",
                sim.contract.is_none(),
                UiAction::InstallFitting(id.to_owned()),
            );
        } else {
            form.text(
                "This fitting must be recovered as a mission reward. It is not sold in drydock.",
            );
        }
    } else {
        super::catalogue::purchase(
            ctx,
            form,
            "Upgrade in port",
            next.cost,
            UiAction::UpgradeSubsystem(id.to_owned()),
        );
    }
}
