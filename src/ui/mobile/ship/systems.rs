use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    let sim = ctx.sim;
    let cfg = &ctx.data.config;
    if let Some(id) = ctx.custody_picker {
        f.action(
            "Cancel custody selection",
            true,
            UiAction::CancelDisciplineCustody,
        );
        f.heading("Choose the discipline's custodians");
        if let Some(def) = ctx.data.subsystems.get(id) {
            f.text(&def.name);
        }
        f.text("Choose one people aboard to care for this discipline. Their approval affects care while the school is funded; this appointment cannot be reassigned here.");
        f.text(&format!(
            "Grant costs {} influence",
            cfg.crew.custody_influence_cost
        ));
        for people in sim.factions.iter().filter(|p| p.is_aboard()) {
            let name = ctx
                .data
                .factions
                .get(&people.faction_id)
                .map_or(people.faction_id.as_str(), |p| p.name.as_str());
            f.action(
                name,
                sim.resources.influence >= cfg.crew.custody_influence_cost,
                UiAction::GrantDisciplineCustody {
                    subsystem_id: id.to_owned(),
                    faction_id: people.faction_id.clone(),
                },
            );
        }
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
        f.text(&format!(
            "{} · Tier {}\nCondition {:.0}% · Knowledge {:.0}%\nRepair requires {:.0}% knowledge",
            def.fitting_name(s.tier),
            s.tier,
            s.condition * 100.0,
            s.knowledge * 100.0,
            def.repair_knowledge_required * 100.0
        ));
        f.text(&def.description);
        if let Some(culture) =
            crate::simulation::culture::descriptor(ctx.data, &id, &s.culture.descriptor_id)
        {
            let custodian = s
                .culture
                .custodian_faction_id
                .as_deref()
                .and_then(|faction_id| ctx.data.factions.get(faction_id))
                .map_or("No local custodian", |faction| faction.name.as_str());
            f.heading(&format!("Local culture · {}", culture.label));
            f.text(&format!(
                "Custodian: {custodian}\n{}\n{}",
                crate::simulation::culture::effect_summary(culture),
                culture.description
            ));
            f.text(&format!(
                "Remembered: {}\nGrievance: {}",
                s.culture
                    .remembered_event
                    .as_deref()
                    .unwrap_or("No compartment memory recorded."),
                s.culture
                    .grievance
                    .as_deref()
                    .unwrap_or("No active grievance.")
            ));
        }
        let target = crate::simulation::subsystems::repair_target_condition(sim, ctx.data, &id)
            .unwrap_or(s.condition);
        if target > s.condition {
            f.action(
                &format!(
                    "Repair to {:.0}% · {} parts · {} minerals",
                    target * 100.0,
                    def.repair_parts_cost,
                    def.repair_minerals_cost
                ),
                s.knowledge >= def.repair_knowledge_required
                    && sim.ship.spare_parts >= def.repair_parts_cost
                    && sim.resources.minerals >= def.repair_minerals_cost,
                UiAction::RepairSubsystem(id.clone()),
            );
        } else {
            f.text("Condition sound for current repair capability");
        }
        if let Some(next) = def.next_fitting(s.tier) {
            f.heading(&format!(
                "Next fitting: {} · Tier {}",
                next.name,
                s.tier + 1
            ));
            if !next.description.is_empty() {
                f.text(&next.description);
            }
            f.text("Installing an upgrade changes the fitting's tier; it does not restore condition or knowledge.");
            if sim.contract.is_some() {
                f.text("Install in port between voyages.");
            }
            if next.acquisition.is_mission_only() {
                if sim.ship.unlocked_fittings.contains(&next.id) {
                    f.text("Recovered fitting available. Installation is free in port.");
                    f.action(
                        "Install recovered fitting",
                        sim.contract.is_none(),
                        UiAction::InstallFitting(id.clone()),
                    );
                } else {
                    f.text("This fitting must be recovered as a mission reward. It is not sold in drydock.");
                }
            } else {
                super::catalogue::purchase(
                    ctx,
                    f,
                    "Upgrade in port",
                    next.cost,
                    UiAction::UpgradeSubsystem(id.clone()),
                );
            }
        } else {
            f.text("Highest fitting tier reached.");
        }
        let train = crate::simulation::subsystems::training_target_knowledge(sim, ctx.data, &id)
            .unwrap_or(s.knowledge);
        if train > s.knowledge {
            f.action(
                &format!(
                    "Train to {:.0}% · {} credits",
                    train * 100.0,
                    cfg.subsystems.train_cost_credits
                ),
                sim.resources.credits >= cfg.subsystems.train_cost_credits,
                UiAction::TrainSubsystemKnowledge(id.clone()),
            );
        }
        super::institutions::build(ctx, f, &id);
    }
}
