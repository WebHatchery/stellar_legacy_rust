use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    let sim = ctx.sim;
    let cfg = &ctx.data.config;
    if let Some(id) = ctx.custody_picker {
        f.heading("Choose the discipline's custodians");
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
        f.action(
            "Cancel custody selection",
            true,
            UiAction::CancelDisciplineCustody,
        );
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
            if next.acquisition.is_mission_only() && sim.ship.unlocked_fittings.contains(&next.id) {
                f.action(
                    "Install recovered fitting",
                    sim.contract.is_none(),
                    UiAction::InstallFitting(id.clone()),
                );
            }

            f.action(
                &format!(
                    "Upgrade in port · {} credits · {} minerals",
                    next.cost.credits, next.cost.minerals
                ),
                sim.contract.is_none()
                    && sim.resources.credits >= next.cost.credits
                    && sim.resources.minerals >= next.cost.minerals,
                UiAction::UpgradeSubsystem(id.clone()),
            );
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
        let school = sim
            .subsystem_schools
            .iter()
            .find(|school| school.subsystem_id == id);
        let archive = sim.procedure_archives.iter().any(|a| a.subsystem_id == id);
        let cost = if school.is_some() {
            cfg.crew.school_upkeep_credits
        } else {
            cfg.crew.school_cost_credits
        };
        f.action(
            &format!(
                "{} school · {cost} credits",
                if school.is_some() {
                    "Support"
                } else {
                    "Establish"
                }
            ),
            sim.resources.credits >= cost,
            UiAction::EstablishSchool(id.clone()),
        );
        if school.is_some() && !archive {
            f.action(
                &format!(
                    "Compile archive · {} credits",
                    cfg.crew.archive_cost_credits
                ),
                sim.resources.credits >= cfg.crew.archive_cost_credits,
                UiAction::CompileProcedureArchive(id.clone()),
            );
        }
        if school.is_some() {
            f.action(
                "Choose discipline custodians",
                sim.resources.influence >= cfg.crew.custody_influence_cost,
                UiAction::BeginDisciplineCustody(id.clone()),
            );
        }
    }
}
