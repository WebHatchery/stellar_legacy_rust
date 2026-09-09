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
            f.text(&format!("Available credits: {}", sim.resources.credits));
            for role in &ctx.data.crew_archetypes {
                f.heading(&role.name);
                f.text(&role.description);
                if let Some(p) = crate::simulation::crew::post_holder(sim, &role.id) {
                    f.portrait(&p.name);
                    f.text(&format!(
                        "Age {} · Skill {} / {}",
                        p.age, p.skill, role.skill_max
                    ));
                    f.text(&format!(
                        "Retirement age {} · {} years away",
                        cfg.retirement_age,
                        cfg.retirement_age.saturating_sub(p.age),
                    ));
                    if let Some(a) = sim.apprenticeships.iter().find(|a| a.post_id == role.id) {
                        f.text(&format!(
                            "Prepared successor: {} · Skill {}",
                            a.apprentice_name, a.skill
                        ));
                        f.text("The apprentice takes over when this officer leaves and reduces the loss of expertise.");
                    } else {
                        f.text("Appoint an apprentice to prepare a successor and preserve more expertise when this officer leaves.");
                        if sim.resources.credits < cfg.apprentice_cost_credits {
                            f.text(&format!(
                                "Apprenticeship needs {} more credits.",
                                cfg.apprentice_cost_credits - sim.resources.credits
                            ));
                        }
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
                        if sim.resources.credits < cfg.train_cost_credits {
                            f.text(&format!(
                                "Training needs {} more credits.",
                                cfg.train_cost_credits - sim.resources.credits
                            ));
                        }
                        f.action(
                            &format!(
                                "Train to skill {} · {} credits",
                                (p.skill + cfg.train_skill_gain).min(role.skill_max),
                                cfg.train_cost_credits
                            ),
                            sim.resources.credits >= cfg.train_cost_credits,
                            UiAction::TrainCrew(role.id.clone()),
                        );
                    } else {
                        f.text("Skill mastered");
                    }
                } else {
                    f.text("! Officer post vacant");
                    if sim.resources.credits < cfg.recruit_cost_credits {
                        f.text(&format!(
                            "Recruitment needs {} more credits.",
                            cfg.recruit_cost_credits - sim.resources.credits
                        ));
                    }
                    f.action(
                        &format!("Recruit · {} credits", cfg.recruit_cost_credits),
                        sim.resources.credits >= cfg.recruit_cost_credits,
                        UiAction::RecruitCrew(role.id.clone()),
                    );
                }
            }
        }
        "factions" => {
            f.text(&format!(
                "Ship approval: {:.0}% · {}",
                sim.aboard_approval_mean() * 100.0,
                crate::state::sim::factions::approval_band_label(sim.aboard_approval_mean()),
            ));
            for people in &sim.factions {
                if let Some(d) = ctx.data.factions.get(&people.faction_id) {
                    f.heading(&d.name);
                    f.text(&d.description);
                    f.text(people.status.label());
                    if !people.is_aboard() {
                        continue;
                    }
                    f.text(&format!(
                        "{} people · {:.0}% approval · {}\nCare: {}",
                        people.members,
                        people.approval * 100.0,
                        crate::state::sim::factions::approval_band_label(people.approval),
                        ctx.data
                            .subsystems
                            .get(&d.tended_subsystem)
                            .map_or("No assigned system", |system| system.name.as_str()),
                    ));
                    for (label, ids) in [("Rivals", &d.rivals), ("Allies", &d.allies)] {
                        let names = ids
                            .iter()
                            .filter(|id| sim.is_faction_aboard(id))
                            .filter_map(|id| ctx.data.factions.get(id))
                            .map(|d| d.name.as_str())
                            .collect::<Vec<_>>();
                        f.text(&format!(
                            "{label} aboard: {}",
                            if names.is_empty() {
                                "None aboard".to_owned()
                            } else {
                                names.join(", ")
                            },
                        ));
                    }
                }
            }
            if sim.contract.is_none()
                && sim.aboard_faction_count() < ctx.data.config.factions.starting_count
            {
                f.heading("Recruit a people");
                f.text("Choose a group to fill an open founding berth while in port.");
                for id in sim.recruitable_faction_ids(ctx.data) {
                    let name = ctx
                        .data
                        .factions
                        .get(&id)
                        .map_or(id.as_str(), |d| d.name.as_str());
                    let cost = ctx.data.config.factions.recruit_group_cost_credits;
                    if let Some(definition) = ctx.data.factions.get(&id) {
                        f.heading(name);
                        f.text(&definition.description);
                    }
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
