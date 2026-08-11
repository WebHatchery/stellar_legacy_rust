//! Crew roster management: recruit/train verbs, skill-driven modifiers,
//! and generational turnover (GDD §4, PLAN item 2).
//!
//! One post per archetype. Crew skills feed the tick as data-driven
//! modifiers declared on `crew_archetypes.json` — production multipliers,
//! famine mitigation (medic), and unity recovery (security chief).

use crate::data::{FlavorConfig, GameData, ProductionRates, ResourceDelta};
use crate::state::sim::{generate_crew_member, SimState};

/// The officer currently holding an archetype's post, if any.
pub fn post_holder<'a>(
    sim: &'a SimState,
    archetype_id: &str,
) -> Option<&'a crate::state::sim::CrewMember> {
    sim.crew.iter().find(|c| c.archetype_id == archetype_id)
}

/// Recruit an officer into a vacant post. Deducts the configured cost.
pub fn recruit(sim: &mut SimState, data: &GameData, archetype_id: &str) -> Result<String, String> {
    if post_holder(sim, archetype_id).is_some() {
        return Err("That post is already held.".to_owned());
    }
    let cost = ResourceDelta {
        credits: -data.config.crew.recruit_cost_credits,
        ..Default::default()
    };
    if !sim.resources.can_afford(&cost) {
        return Err("The treasury cannot cover a recruitment bounty.".to_owned());
    }

    let crew_cfg = &data.config.crew;
    let age_span = (crew_cfg.recruit_age_max - crew_cfg.recruit_age_min + 1) as usize;
    let age = crew_cfg.recruit_age_min + sim.rng.below(age_span) as u32;
    let legacy_id = sim.legacy.legacy_id.clone();
    let faction_id = sim.dominant_faction_id().unwrap_or_default().to_owned();
    let Some(member) = generate_crew_member(
        data,
        &legacy_id,
        archetype_id,
        age,
        faction_id,
        &mut sim.rng,
        &mut sim.next_crew_id,
    ) else {
        return Err("No such post exists.".to_owned());
    };

    sim.resources.apply(&cost);
    let name = member.name.clone();
    // Name the post the way the ship would, not by its raw id, and vary the
    // line so re-crewing a roster over the centuries never reads as a form
    // letter (content-depth voice round 7). Indexed by the officer's id.
    let post = data
        .crew_archetypes
        .iter()
        .find(|a| a.id == archetype_id)
        .map_or(archetype_id, |a| a.name.as_str());
    let line = FlavorConfig::line_with_name_post(
        &data.config.flavor.appointment,
        member.id as usize,
        &name,
        post,
    )
    .unwrap_or_else(|| format!("{name} took up the post of {post}."));
    sim.push_log(line);
    sim.crew.push(member);
    Ok(name)
}

/// Train the holder of a post, raising skill toward the archetype cap.
pub fn train(sim: &mut SimState, data: &GameData, archetype_id: &str) -> Result<String, String> {
    let Some(archetype) = data.crew_archetypes.iter().find(|a| a.id == archetype_id) else {
        return Err("No such post exists.".to_owned());
    };
    let holder_skill = post_holder(sim, archetype_id)
        .ok_or_else(|| "No officer holds that post.".to_owned())?
        .skill;
    if holder_skill >= archetype.skill_max {
        return Err("That officer has nothing left to learn.".to_owned());
    }
    let cost = ResourceDelta {
        credits: -data.config.crew.train_cost_credits,
        ..Default::default()
    };
    if !sim.resources.can_afford(&cost) {
        return Err("The treasury cannot cover the training program.".to_owned());
    }

    sim.resources.apply(&cost);
    let gain = data.config.crew.train_skill_gain;
    let member = sim
        .crew
        .iter_mut()
        .find(|c| c.archetype_id == archetype_id)
        .expect("holder existence checked above");
    member.skill = (member.skill + gain).min(archetype.skill_max);
    let name = member.name.clone();
    let skill = member.skill;
    let line = FlavorConfig::line_with_name_post(
        &data.config.flavor.training,
        skill as usize,
        &name,
        &archetype.name,
    )
    .map(|l| l.replace("{skill}", &skill.to_string()))
    .unwrap_or_else(|| {
        format!(
            "{name} completed advanced training as {} (skill {skill}).",
            archetype.name
        )
    });
    sim.push_log(line);
    Ok(name)
}

/// Per-resource production multipliers from the serving crew's skills.
pub fn production_multipliers(sim: &SimState, data: &GameData) -> ProductionRates {
    let mut mult = ProductionRates {
        credits: 1.0,
        energy: 1.0,
        minerals: 1.0,
        food: 1.0,
        influence: 1.0,
    };
    for member in &sim.crew {
        let Some(archetype) = data
            .crew_archetypes
            .iter()
            .find(|a| a.id == member.archetype_id)
        else {
            continue;
        };
        let skill = member.skill as f32;
        let per = &archetype.production_per_skill;
        mult.credits += per.credits * skill;
        mult.energy += per.energy * skill;
        mult.minerals += per.minerals * skill;
        mult.food += per.food * skill;
        mult.influence += per.influence * skill;
    }
    mult
}

/// Fraction of famine losses prevented by the serving medical staff (0-0.9).
pub fn famine_loss_reduction(sim: &SimState, data: &GameData) -> f32 {
    sim.crew
        .iter()
        .filter_map(|member| {
            data.crew_archetypes
                .iter()
                .find(|a| a.id == member.archetype_id)
                .map(|a| a.famine_loss_reduction_per_skill * member.skill as f32)
        })
        .sum::<f32>()
        .clamp(0.0, 0.9)
}

/// Yearly unity recovery from the serving security staff. Only applies while
/// unity sits below the configured ceiling — good policing steadies a
/// fractious ship, it doesn't manufacture harmony.
pub fn unity_recovery(sim: &SimState, data: &GameData) -> f32 {
    if sim.population.unity >= data.config.crew.unity_recovery_ceiling {
        return 0.0;
    }
    sim.crew
        .iter()
        .filter_map(|member| {
            data.crew_archetypes
                .iter()
                .find(|a| a.id == member.archetype_id)
                .map(|a| a.unity_recovery_per_skill * member.skill as f32)
        })
        .sum()
}

#[cfg(test)]
mod tests;
