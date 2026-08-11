//! State-aware named council advice selected from authored officer prose.

use crate::data::events::{EventCategory, EventTemplate};
use crate::data::GameData;
use crate::simulation::institutions::discipline_for_post;
use crate::state::sim::SimState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouncilAdvice {
    pub officer_name: Option<String>,
    pub post_name: String,
    pub text: String,
}

pub fn for_event(sim: &SimState, data: &GameData, event: &EventTemplate) -> Vec<CouncilAdvice> {
    advisor_posts(event)
        .into_iter()
        .take(2)
        .filter_map(|post_id| advice_for_post(sim, data, post_id))
        .collect()
}

fn advisor_posts(event: &EventTemplate) -> Vec<&str> {
    if !event.advisor_posts.is_empty() {
        return event.advisor_posts.iter().map(String::as_str).collect();
    }
    if !event.family.is_empty() {
        let pair = match event.family.as_str() {
            "engineering" => ["engineer", "security_chief"],
            "biology_medical" => ["medic", "scientist"],
            "survival" => ["agronomist", "commander"],
            "diplomacy" => ["security_chief", "commander"],
            "science_anomaly" | "mystery" => ["scientist", "navigator"],
            "legacy_drift" | "ethics" => ["commander", "navigator"],
            _ => ["commander", "scientist"],
        };
        return pair.into_iter().collect();
    }
    match event.category {
        EventCategory::ImmediateCrisis => vec!["engineer", "medic"],
        EventCategory::GenerationalChallenge => vec!["agronomist", "security_chief"],
        EventCategory::MissionMilestone => vec!["navigator", "scientist"],
        EventCategory::LegacyMoment => vec!["commander", "navigator"],
    }
}

pub fn advice_for_post(sim: &SimState, data: &GameData, post_id: &str) -> Option<CouncilAdvice> {
    let archetype = data.crew_archetypes.iter().find(|a| a.id == post_id)?;
    let holder = sim.crew.iter().find(|c| c.archetype_id == post_id);
    let post = archetype.name.clone();
    let Some(officer) = holder else {
        return Some(CouncilAdvice {
            officer_name: None,
            post_name: post,
            text: substitute(
                &archetype.advice.vacant,
                "No serving officer",
                &archetype.name,
                "vacant",
                "no people",
            ),
        });
    };

    let skill_band = if officer.skill < 55 {
        "learning"
    } else if officer.skill >= 75 {
        "master"
    } else {
        "seasoned"
    };
    let faction_name = data
        .factions
        .get(&officer.faction_id)
        .map_or("unaffiliated", |f| f.name.as_str());
    let base = if officer.skill < 55 {
        &archetype.advice.novice
    } else if officer.skill >= 75 {
        &archetype.advice.expert
    } else {
        &archetype.advice.steady
    };
    let mut parts = vec![substitute(
        base,
        &officer.name,
        &post,
        skill_band,
        faction_name,
    )];
    let discipline = discipline_for_post(post_id);
    if sim
        .subsystems
        .get(discipline)
        .is_some_and(|state| state.condition < 0.45 || state.knowledge < 0.45)
    {
        parts.push(substitute(
            &archetype.advice.strained,
            &officer.name,
            &post,
            skill_band,
            faction_name,
        ));
    }
    if sim.dominant_faction_id() == Some(officer.faction_id.as_str()) {
        parts.push(substitute(
            &archetype.advice.faction_aligned,
            &officer.name,
            &post,
            skill_band,
            faction_name,
        ));
    }
    let reputation_is_defining = ["mercy", "wonder", "resolve"]
        .iter()
        .any(|trait_id| (sim.reputation(trait_id) - 0.5).abs() >= 0.2);
    if reputation_is_defining {
        parts.push(substitute(
            &archetype.advice.reputation,
            &officer.name,
            &post,
            skill_band,
            faction_name,
        ));
    }
    if let Some(obligation) = sim.active_obligations().next() {
        let line = substitute(
            &archetype.advice.obligation,
            &officer.name,
            &post,
            skill_band,
            faction_name,
        )
        .replace("{obligation}", &obligation.title);
        parts.push(line);
    }
    parts.retain(|part| !part.trim().is_empty());
    Some(CouncilAdvice {
        officer_name: Some(officer.name.clone()),
        post_name: post,
        text: parts.join(" "),
    })
}

fn substitute(text: &str, name: &str, post: &str, skill_band: &str, faction: &str) -> String {
    text.replace("{name}", name)
        .replace("{post}", post)
        .replace("{skill_band}", skill_band)
        .replace("{faction}", faction)
}

#[cfg(test)]
mod tests;
