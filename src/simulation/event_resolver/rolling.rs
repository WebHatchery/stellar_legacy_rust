//! Which event fires: the category weighting, the gate every template
//! must pass for the ship's current state, and the weighted draw itself.

use crate::data::events::{EventCategory, EventTemplate};
use crate::data::{GameConfig, GameData};
use crate::simulation::subsystems;
use crate::state::sim::{PendingEvent, SimState};

/// `event_chance = min(cap, base + years_since_event*0.1 + contract_progress*0.2)`.
pub fn event_chance(config: &GameConfig, years_since_event: u32, contract_progress: f32) -> f32 {
    (config.event_chance_base + years_since_event as f32 * 0.1 + contract_progress * 0.2)
        .min(config.event_chance_cap)
}

/// Category weights, scaled up by ship/population distress (GDD §5.4).
pub fn category_weights(sim: &SimState, data: &GameData) -> [(EventCategory, f32); 4] {
    let config = &data.config;
    let mut crisis = 0.3;
    if sim.resources.food < config.low_food_threshold {
        crisis += 0.2;
    }
    if sim.resources.energy < config.low_energy_threshold {
        crisis += 0.2;
    }
    if sim.ship.hull_integrity < config.hull_warning_threshold {
        crisis += 0.2;
    }
    if sim.ship.life_support < config.life_support_warning_threshold {
        crisis += 0.2;
    }
    if sim.population.morale < 0.5 {
        crisis += 0.15;
    }
    if sim.population.unity < 0.4 {
        crisis += 0.15;
    }
    // Route hazard (content-depth charters round 11): a dangerous writ breeds more
    // crises for its whole voyage — the charter's risk profile, not just the ship's
    // present distress. A well-armed ship, though, makes a lawless route think twice
    // (content-depth charters round 27): the ship's *combat loadout* cuts into the route's
    // own hazard — scavengers and raiders keep their distance from guns — the direct-firepower
    // twin of the security corps' internal-order mitigation just below (guns deter the *route's*
    // danger; the corps quiets *every* crisis). Floored at 0, so firepower can neutralize a
    // route's added risk but never drive a hazardous writ below the ship's own base crisis rate.
    if let Some(contract) = &sim.contract {
        let combat = crate::simulation::ship::loadout_stats(sim, data)
            .combat
            .max(0) as f32;
        let deterred = (contract.hazard - combat * config.ship.hazard_combat_mitigation).max(0.0);
        crisis += deterred;
    }
    // A well-kept security/justice corps (content-depth subsystems round 21) defends
    // the ship against the crises a dangerous route and a distressed hull breed —
    // fewer boardings, riots, and breaches reach the council. The corps' condition
    // dampens the crisis weight (the subsystem-side twin of the charters-round-21
    // combat coupling), floored so even a perfect corps only quiets danger, never
    // silences it.
    let mitigation = config.subsystems.security_crisis_mitigation;
    if mitigation > 0.0 {
        let security = sim.subsystems.get("security").map_or(0.0, |s| s.condition);
        crisis = (crisis - security * mitigation).max(config.subsystems.crisis_weight_floor);
    }

    let milestone = match &sim.contract {
        Some(contract) => {
            let progress = contract.progress();
            if !(0.2..=0.8).contains(&progress) {
                0.4
            } else {
                0.15
            }
        }
        None => 0.05,
    };

    let legacy = (0.1 + (sim.year() / 25) as f32 * 0.05).min(0.3);

    [
        (EventCategory::ImmediateCrisis, crisis),
        (EventCategory::GenerationalChallenge, 0.3),
        (EventCategory::MissionMilestone, milestone),
        (EventCategory::LegacyMoment, legacy),
    ]
}

/// True if `template` clears its W6 phase + voyage gates for the current state:
/// an empty `phases` fires in any phase, otherwise the contract must be active
/// and its current phase listed; year / generation / cultural-drift gates must
/// all be met.
pub(crate) fn passes_gate(sim: &SimState, template: &EventTemplate) -> bool {
    !template.scheduled_only
        && phase_and_history_gates(sim, template)
        && faction_gates(sim, template)
        && subsystem_gates(sim, template)
        && resource_gates(sim, template)
        && campaign_gates(sim, template)
}

fn phase_and_history_gates(sim: &SimState, template: &EventTemplate) -> bool {
    if !template.phases.is_empty()
        && !sim
            .contract
            .as_ref()
            .is_some_and(|contract| template.phases.contains(&contract.phase))
    {
        return false;
    }
    template
        .requires_consequence
        .iter()
        .all(|tag| sim.consequences.contains(tag))
        && !template
            .forbidden_consequence
            .iter()
            .any(|tag| sim.consequences.contains(tag))
        && (template.requires_charter_tag.is_empty()
            || sim.contract.as_ref().is_some_and(|contract| {
                template
                    .requires_charter_tag
                    .iter()
                    .all(|tag| contract.tags.contains(tag))
            }))
        && (template.requires_dominant_faction.is_empty()
            || sim.dominant_faction_id() == Some(template.requires_dominant_faction.as_str()))
        && template
            .requires_factions_aboard
            .iter()
            .all(|id| sim.is_faction_aboard(id))
}

fn faction_gates(sim: &SimState, template: &EventTemplate) -> bool {
    template.faction_approval_below.iter().all(|gate| {
        sim.factions
            .iter()
            .any(|f| f.faction_id == gate.id && f.is_aboard() && f.approval <= gate.below)
    }) && template.faction_approval_above.iter().all(|gate| {
        sim.factions
            .iter()
            .any(|f| f.faction_id == gate.id && f.is_aboard() && f.approval >= gate.at_least)
    })
}

fn subsystem_gates(sim: &SimState, template: &EventTemplate) -> bool {
    template.knowledge_below.iter().all(|gate| {
        sim.subsystems
            .get(&gate.id)
            .is_some_and(|state| state.knowledge <= gate.below)
    }) && template.condition_below.iter().all(|gate| {
        sim.subsystems
            .get(&gate.id)
            .is_some_and(|state| state.condition <= gate.below)
    })
}

fn resource_gates(sim: &SimState, template: &EventTemplate) -> bool {
    let below = template
        .food_below
        .is_none_or(|threshold| sim.resources.food <= threshold)
        && template
            .fuel_below
            .is_none_or(|threshold| sim.ship.fuel <= threshold)
        && template
            .spare_parts_below
            .is_none_or(|threshold| sim.ship.spare_parts <= threshold)
        && template
            .energy_below
            .is_none_or(|threshold| sim.resources.energy <= threshold);
    let above = template
        .food_above
        .is_none_or(|threshold| sim.resources.food >= threshold)
        && template
            .credits_above
            .is_none_or(|threshold| sim.resources.credits >= threshold);
    below && above
}

fn campaign_gates(sim: &SimState, template: &EventTemplate) -> bool {
    if template.max_year != 0 && sim.year() > template.max_year
        || template.max_generation != 0 && sim.dynasty.generation > template.max_generation
        || template.max_population > 0 && sim.population.count > template.max_population
        || template.max_dynasty_size > 0
            && sim.dynasty.members.len() as u32 > template.max_dynasty_size
        || template
            .hull_below
            .is_some_and(|threshold| sim.ship.hull_integrity > threshold)
        || template
            .life_support_below
            .is_some_and(|threshold| sim.ship.life_support > threshold)
        || template
            .adaptation_above
            .is_some_and(|threshold| sim.population.adaptation < threshold)
        || template
            .stability_above
            .is_some_and(|threshold| sim.population.stability < threshold)
        || sim.lean_food_years < template.min_lean_food_years
        || sim.fat_food_years < template.min_fat_food_years
        || template.max_legacy_loyalty > 0.0
            && sim.population.legacy_loyalty > template.max_legacy_loyalty
        || template.max_stability > 0.0 && sim.population.stability > template.max_stability
        || template
            .min_reputation
            .iter()
            .any(|gate| sim.reputation(&gate.id) < gate.threshold)
        || template
            .max_reputation
            .iter()
            .any(|gate| sim.reputation(&gate.id) > gate.threshold)
    {
        return false;
    }
    let objective_ready = template.min_objective_fraction <= 0.0
        || sim.contract.as_ref().is_some_and(|contract| {
            contract.objective_fraction() >= template.min_objective_fraction
        });
    objective_ready
        && sim.year() >= template.min_year
        && sim.dynasty.generation >= template.min_generation
        && sim.population.cultural_drift >= template.min_cultural_drift
        && sim.population.morale >= template.min_morale
        && sim.population.unity >= template.min_unity
}

/// Weighted pick among already gate-cleared candidates (sorted by id for
/// determinism): legacy affinity × the buffering subsystem's rarefying factor
/// (W5). Records the fire on the sim and returns the pending event, or `None`
/// when nothing survived the filter.
fn pick_weighted(
    sim: &mut SimState,
    data: &GameData,
    mut candidates: Vec<(&String, &EventTemplate)>,
) -> Option<PendingEvent> {
    candidates.sort_by(|a, b| a.0.cmp(b.0));
    if candidates.is_empty() {
        return None;
    }
    let legacy_id = sim.legacy.legacy_id.as_str();
    let template_weights: Vec<f32> = candidates
        .iter()
        .map(|(_, t)| {
            *t.legacy_weight_modifiers.get(legacy_id).unwrap_or(&1.0)
                * subsystems::family_weight_factor(sim, data, &t.family)
        })
        .collect();
    let weight_total: f32 = template_weights.iter().sum();
    let mut roll = sim.rng.next_f32() * weight_total;
    let mut chosen = candidates[0].1;
    for (i, weight) in template_weights.iter().enumerate() {
        if roll < *weight {
            chosen = candidates[i].1;
            break;
        }
        roll -= weight;
    }
    sim.last_event_month_clock = sim.month_clock;
    Some(PendingEvent {
        template_id: chosen.id.clone(),
        rolled_month_clock: sim.month_clock,
    })
}

/// Roll for a reactive/filler event (W6): the monthly chance, a category by
/// weight, then a gate-cleared template within it. Returns the pending event
/// without applying anything; the caller decides block vs auto-resolve.
pub fn roll_event(sim: &mut SimState, data: &GameData) -> Option<PendingEvent> {
    let progress = sim.contract.as_ref().map_or(0.0, |c| c.progress());
    // The ramp is still a per-year model; convert its whole-year gap and the
    // resulting yearly chance to a per-month roll so expected events per year
    // is preserved while events can now fire (and be dated) any month (W3).
    let years_since = sim.month_clock.saturating_sub(sim.last_event_month_clock) / 12;
    let monthly_chance = (event_chance(&data.config, years_since, progress)
        * crate::simulation::command::event_chance_factor(sim.command_posture)
        * sim.contract.as_ref().map_or(1.0, |contract| {
            crate::simulation::approach::event_chance_factor(contract.approach)
        })
        / 12.0)
        .min(1.0);
    if !sim.rng.chance(monthly_chance) {
        return None;
    }

    // Pick a category by weight; candidates are that category's gate-cleared
    // templates (W6 phase/year/generation/drift filters).
    let weights = category_weights(sim, data);
    let total: f32 = weights.iter().map(|(_, w)| w).sum();
    let mut pick = sim.rng.next_f32() * total;
    let mut category = EventCategory::ImmediateCrisis;
    for (cat, weight) in weights {
        if pick < weight {
            category = cat;
            break;
        }
        pick -= weight;
    }

    let candidates: Vec<(&String, &EventTemplate)> = data
        .events
        .iter()
        .filter(|(_, t)| t.category == category && passes_gate(sim, t))
        .collect();
    pick_weighted(sim, data, candidates)
}

/// Roll a scheduled beat's event (W6): no chance roll — a beat always fires —
/// filtering the catalog to `family` plus the W6 gates, then the normal
/// weighting. `None` when the family is over-gated (caller falls through).
pub fn roll_event_in_family(
    sim: &mut SimState,
    data: &GameData,
    family: &str,
) -> Option<PendingEvent> {
    let candidates: Vec<(&String, &EventTemplate)> = data
        .events
        .iter()
        .filter(|(_, t)| t.family == family && passes_gate(sim, t))
        .collect();
    pick_weighted(sim, data, candidates)
}
