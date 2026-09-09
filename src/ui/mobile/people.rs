use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    f.sections(&[
        ("Family", ""),
        ("Officers", "officers"),
        ("Factions", "factions"),
        ("Council", "council"),
    ]);
    let sim = ctx.sim;
    let cfg = &ctx.data.config.crew;
    match section {
        "officers" => {
            for role in &ctx.data.crew_archetypes {
                f.heading(&role.name);
                if let Some(p) = crate::simulation::crew::post_holder(sim, &role.id) {
                    f.portrait(&p.name);
                    f.text(&format!(
                        "Age {} · Skill {} / {}",
                        p.age, p.skill, role.skill_max
                    ));
                    if let Some(a) = sim.apprenticeships.iter().find(|a| a.post_id == role.id) {
                        f.text(&format!(
                            "Apprentice: {} · Skill {}",
                            a.apprentice_name, a.skill
                        ));
                    } else {
                        f.action(
                            &format!(
                                "Appoint apprentice · {} credits",
                                cfg.apprentice_cost_credits
                            ),
                            sim.resources.credits >= cfg.apprentice_cost_credits,
                            UiAction::DesignateApprentice(role.id.clone()),
                        );
                    }
                    if p.skill < role.skill_max {
                        f.action(
                            &format!("Train · {} credits", cfg.train_cost_credits),
                            sim.resources.credits >= cfg.train_cost_credits,
                            UiAction::TrainCrew(role.id.clone()),
                        );
                    } else {
                        f.text("Skill mastered");
                    }
                } else {
                    f.text("! Officer post vacant");
                    f.action(
                        &format!("Recruit · {} credits", cfg.recruit_cost_credits),
                        sim.resources.credits >= cfg.recruit_cost_credits,
                        UiAction::RecruitCrew(role.id.clone()),
                    );
                }
            }
        }
        "factions" => {
            for people in &sim.factions {
                if let Some(d) = ctx.data.factions.get(&people.faction_id) {
                    f.heading(&d.name);
                    f.text(&format!(
                        "{} people · {:.0}% approval\nCare: {}",
                        people.members,
                        people.approval * 100.0,
                        d.tended_subsystem.replace('_', " ")
                    ));
                    for (label, ids) in [("Rivals", &d.rivals), ("Allies", &d.allies)] {
                        f.text(&format!(
                            "{label}: {}",
                            ids.iter()
                                .filter(|id| sim.is_faction_aboard(id))
                                .filter_map(|id| ctx.data.factions.get(id))
                                .map(|d| d.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
            if sim.contract.is_none()
                && sim.aboard_faction_count() < ctx.data.config.factions.starting_count
            {
                for id in sim.recruitable_faction_ids(ctx.data) {
                    let name = ctx
                        .data
                        .factions
                        .get(&id)
                        .map_or(id.as_str(), |d| d.name.as_str());
                    let cost = ctx.data.config.factions.recruit_group_cost_credits;
                    f.action(
                        &format!("Recruit {name} · {cost} credits"),
                        sim.resources.credits >= cost,
                        UiAction::RecruitFactionGroup(id),
                    );
                }
            }
        }
        "council" => {
            f.heading("Custodian & captain");
            f.text("Routine operations follow the standing mandate. Delegated event domains resolve through the council's advisors; outcomes remain in History.");
            for category in EventCategory::ALL {
                f.action(
                    &format!(
                        "{} · {} · tap to change",
                        category.label(),
                        if sim.delegation.is_delegated(category) {
                            "delegated"
                        } else {
                            "council"
                        }
                    ),
                    true,
                    UiAction::ToggleDelegation(category),
                );
            }
        }
        _ => {
            f.heading("Serving captain");
            if let Some(p) = sim.dynasty.leader() {
                f.portrait(&p.name);
                f.text(&format!("Age {} · Leadership {}", p.age, p.leadership));
            }
            f.heading("Next eligible heir");
            if let Some(p) =
                crate::simulation::succession::planned_heir(&sim.dynasty, &ctx.data.config)
            {
                f.portrait(&p.name);
                f.text(&format!("Age {} · Leadership {}", p.age, p.leadership));
                f.text(if sim.dynasty.designated_heir == Some(p.id) {
                    "Named heir · the council's choice for the next captain."
                } else {
                    "Automatic successor · highest leadership among eligible family members."
                });
            } else {
                f.text("No eligible successor.");
            }
            f.heading("Family roster");
            f.text(&format!(
                "Tap Name heir to choose the next captain; the serving captain remains in command. Eligible ages: {}–{}. The choice must still be alive and eligible at succession.",
                ctx.data.config.heir_min_age, ctx.data.config.heir_max_age,
            ));
            for p in &sim.dynasty.members {
                {
                    f.portrait(&p.name);
                    f.text(&format!("Age {} · Leadership {}", p.age, p.leadership));
                    f.text(&format!("{} · {}", p.specialization, p.trait_name));
                    if p.is_leader {
                        f.text("Serving captain");
                    } else if sim.dynasty.designated_heir == Some(p.id) {
                        f.text("Named heir");
                    }
                    if !p.is_leader
                        && p.age >= ctx.data.config.heir_min_age
                        && p.age <= ctx.data.config.heir_max_age
                    {
                        let named = sim.dynasty.designated_heir == Some(p.id);
                        f.action(
                            if named { "Current heir" } else { "Name heir" },
                            !named,
                            UiAction::SelectHeir(p.id),
                        );
                    } else if !p.is_leader {
                        f.text(if p.age < ctx.data.config.heir_min_age {
                            "Too young to be named heir."
                        } else {
                            "Outside the age range for naming an heir."
                        });
                    }
                }
            }
        }
    }
}
