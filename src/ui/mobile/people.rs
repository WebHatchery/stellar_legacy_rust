//! Mobile People and dynasty sections.

use super::*;
pub(super) fn build(ctx: &GameplayCtx<'_>, f: &mut Form, section: &str) {
    f.sections(&[
        ("Family", ""),
        ("Officers", "officers"),
        ("Factions", "factions"),
        ("Council", "council"),
    ]);
    match section {
        "officers" => build_officers(ctx, f),
        "factions" => build_factions(ctx, f),
        "council" => crate::ui::crew_dynasty::council::build(ctx, f),
        _ => build_family(ctx, f),
    }
}

fn build_officers(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    let cfg = &ctx.data.config.crew;
    form.text(&format!("Available credits: {}", sim.resources.credits));
    for role in &ctx.data.crew_archetypes {
        form.heading(&role.name);
        form.text(&role.description);
        if let Some(person) = crate::simulation::crew::post_holder(sim, &role.id) {
            form.portrait(&person.name);
            form.text(&format!(
                "Age {} · Skill {} / {}",
                person.age, person.skill, role.skill_max
            ));
            form.text(&format!(
                "Retirement age {} · {} years away",
                cfg.retirement_age,
                cfg.retirement_age.saturating_sub(person.age),
            ));
            build_officer_actions(ctx, form, role, person);
        } else {
            form.text("! Officer post vacant");
            if sim.resources.credits < cfg.recruit_cost_credits {
                form.text(&format!(
                    "Recruitment needs {} more credits.",
                    cfg.recruit_cost_credits - sim.resources.credits
                ));
            }
            form.action(
                &format!("Recruit · {} credits", cfg.recruit_cost_credits),
                sim.resources.credits >= cfg.recruit_cost_credits,
                UiAction::RecruitCrew(role.id.clone()),
            );
        }
    }
}

fn build_officer_actions(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    role: &crate::data::crew::CrewArchetype,
    person: &crate::state::sim::CrewMember,
) {
    let sim = ctx.sim;
    let cfg = &ctx.data.config.crew;
    if let Some(apprentice) = sim.apprenticeships.iter().find(|a| a.post_id == role.id) {
        form.text(&format!(
            "Prepared successor: {} · Skill {}",
            apprentice.apprentice_name, apprentice.skill
        ));
        form.text(
            "The apprentice takes over when this officer leaves and reduces the loss of expertise.",
        );
    } else {
        form.text("Appoint an apprentice to prepare a successor and preserve more expertise when this officer leaves.");
        if sim.resources.credits < cfg.apprentice_cost_credits {
            form.text(&format!(
                "Apprenticeship needs {} more credits.",
                cfg.apprentice_cost_credits - sim.resources.credits
            ));
        }
        form.action(
            &format!(
                "Appoint apprentice · {} credits",
                cfg.apprentice_cost_credits
            ),
            sim.resources.credits >= cfg.apprentice_cost_credits,
            UiAction::DesignateApprentice(role.id.clone()),
        );
    }
    if person.skill < role.skill_max {
        if sim.resources.credits < cfg.train_cost_credits {
            form.text(&format!(
                "Training needs {} more credits.",
                cfg.train_cost_credits - sim.resources.credits
            ));
        }
        form.action(
            &format!(
                "Train to skill {} · {} credits",
                (person.skill + cfg.train_skill_gain).min(role.skill_max),
                cfg.train_cost_credits
            ),
            sim.resources.credits >= cfg.train_cost_credits,
            UiAction::TrainCrew(role.id.clone()),
        );
    } else {
        form.text("Skill mastered");
    }
}

fn build_factions(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    form.text(&format!(
        "Ship approval: {:.0}% · {}",
        sim.aboard_approval_mean() * 100.0,
        crate::state::sim::factions::approval_band_label(sim.aboard_approval_mean()),
    ));
    for people in &sim.factions {
        let Some(definition) = ctx.data.factions.get(&people.faction_id) else {
            continue;
        };
        form.heading(&definition.name);
        form.text(&definition.description);
        form.text(people.status.label());
        if people.is_aboard() {
            build_faction_detail(ctx, form, people, definition);
        }
    }
    if sim.contract.is_none()
        && sim.aboard_faction_count() < ctx.data.config.factions.starting_count
    {
        build_faction_recruitment(ctx, form);
    }
}

fn build_faction_detail(
    ctx: &GameplayCtx<'_>,
    form: &mut Form,
    people: &crate::state::sim::factions::FactionState,
    definition: &crate::data::factions::FactionDef,
) {
    let sim = ctx.sim;
    form.text(&format!(
        "{} people · {:.0}% approval · {}\nCare: {}",
        people.members,
        people.approval * 100.0,
        crate::state::sim::factions::approval_band_label(people.approval),
        ctx.data
            .subsystems
            .get(&definition.tended_subsystem)
            .map_or("No assigned system", |system| system.name.as_str()),
    ));
    for (label, ids) in [
        ("Rivals", &definition.rivals),
        ("Allies", &definition.allies),
    ] {
        let names = ids
            .iter()
            .filter(|id| sim.is_faction_aboard(id))
            .filter_map(|id| ctx.data.factions.get(id))
            .map(|definition| definition.name.as_str())
            .collect::<Vec<_>>();
        form.text(&format!(
            "{label} aboard: {}",
            if names.is_empty() {
                "None aboard".to_owned()
            } else {
                names.join(", ")
            },
        ));
    }
}

fn build_faction_recruitment(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    let cost = ctx.data.config.factions.recruit_group_cost_credits;
    form.heading("Recruit a people");
    form.text("Choose a group to fill an open founding berth while in port.");
    for id in sim.recruitable_faction_ids(ctx.data) {
        let name = ctx
            .data
            .factions
            .get(&id)
            .map_or(id.as_str(), |definition| definition.name.as_str());
        if let Some(definition) = ctx.data.factions.get(&id) {
            form.heading(name);
            form.text(&definition.description);
        }
        form.action(
            &format!("Recruit {name} · {cost} credits"),
            sim.resources.credits >= cost,
            UiAction::RecruitFactionGroup(id),
        );
    }
}

fn build_family(ctx: &GameplayCtx<'_>, form: &mut Form) {
    let sim = ctx.sim;
    form.heading("Serving captain");
    if let Some(person) = sim.dynasty.leader() {
        form.portrait(&person.name);
        form.text(&format!(
            "Age {} · Leadership {}",
            person.age, person.leadership
        ));
    }
    form.heading("Next eligible heir");
    if let Some(person) =
        crate::simulation::succession::planned_heir(&sim.dynasty, &ctx.data.config)
    {
        form.portrait(&person.name);
        form.text(&format!(
            "Age {} · Leadership {}",
            person.age, person.leadership
        ));
        form.text(if sim.dynasty.designated_heir == Some(person.id) {
            "Named heir · the council's choice for the next captain."
        } else {
            "Automatic successor · highest leadership among eligible family members."
        });
    } else {
        form.text("No eligible successor.");
    }
    form.heading("Family roster");
    form.text(&format!(
        "Tap Name heir to choose the next captain; the serving captain remains in command. Eligible ages: {}–{}. The choice must still be alive and eligible at succession.",
        ctx.data.config.heir_min_age, ctx.data.config.heir_max_age,
    ));
    for person in &sim.dynasty.members {
        form.portrait(&person.name);
        form.text(&format!(
            "Age {} · Leadership {}",
            person.age, person.leadership
        ));
        form.text(&format!(
            "{} · {}",
            person.specialization, person.trait_name
        ));
        if person.is_leader {
            form.text("Serving captain");
        } else if sim.dynasty.designated_heir == Some(person.id) {
            form.text("Named heir");
        }
        if !person.is_leader
            && person.age >= ctx.data.config.heir_min_age
            && person.age <= ctx.data.config.heir_max_age
        {
            let named = sim.dynasty.designated_heir == Some(person.id);
            form.action(
                if named { "Current heir" } else { "Name heir" },
                !named,
                UiAction::SelectHeir(person.id),
            );
        } else if !person.is_leader {
            form.text(if person.age < ctx.data.config.heir_min_age {
                "Too young to be named heir."
            } else {
                "Outside the age range for naming an heir."
            });
        }
    }
}
