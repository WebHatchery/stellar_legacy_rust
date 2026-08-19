//! Read-only departure forecasting for the PREP dossier.
//!
//! This is deliberately a baseline projection, not a promise: it holds current
//! population, crew, subsystem condition, and route terms steady. Events and
//! later deterioration remain the uncertainty a generational voyage is about.

use crate::data::contracts::{ContractPhase, ContractTemplate};
use crate::data::GameData;
use crate::simulation::{crew, ship, subsystems};
use crate::state::sim::SimState;

#[derive(Debug, Clone, PartialEq)]
pub struct DepartureForecast {
    pub duration_years: u32,
    pub travel_years: u32,
    pub annual_food_output: i64,
    pub annual_food_use: i64,
    pub annual_route_food: i64,
    pub annual_food_net: i64,
    pub projected_food_end: i64,
    pub recommended_food_store: i64,
    pub parts_upkeep: i64,
    pub fuel_burn: f32,
    pub fuel_regen_per_year: f32,
    pub route_hull_change: f32,
    pub route_life_support_change: f32,
}

pub fn for_departure(
    sim: &SimState,
    data: &GameData,
    template: &ContractTemplate,
) -> DepartureForecast {
    let duration_years = template.target_duration_years;
    let travel_years = template
        .phases
        .iter()
        .filter(|phase| phase.kind == ContractPhase::Travel)
        .map(|phase| phase.years)
        .sum::<u32>();
    let crew_mult = crew::production_multipliers(sim, data);
    let agriculture_bonus = subsystems::agriculture_food_bonus(sim, data);
    let agriculture_condition = subsystems::agriculture_condition_food_factor(sim, data);
    let annual_food_output =
        (sim.production.food * crew_mult.food * (1.0 + agriculture_bonus) * agriculture_condition)
            .floor() as i64;
    let annual_food_use =
        (sim.population.count as f32 * data.config.food_per_person_per_year).ceil() as i64;
    let annual_route_food = template.annual_toll.resource.food;
    let annual_food_net = annual_food_output - annual_food_use + annual_route_food;
    let voyage_food_change = annual_food_net.saturating_mul(duration_years as i64);
    let projected_food_end = sim.resources.food.saturating_add(voyage_food_change);
    // Keep a survival reserve at least as large as both the game's lean-food
    // line and one present-day year of consumption. If current production is a
    // surplus, the reserve—not the voyage's gross consumption—is what must sail.
    let food_reserve = data.config.low_food_threshold.max(annual_food_use);
    let recommended_food_store =
        food_reserve.saturating_add(voyage_food_change.saturating_neg().max(0));

    let parts_upkeep = data
        .config
        .parts_upkeep_per_year
        .saturating_mul(duration_years as i64);
    let engineering_burn = subsystems::engineering_fuel_burn_factor(sim, data);
    let fuel_burn = data.config.provisioning.fuel_burn_per_travel_month
        * (travel_years * 12) as f32
        * engineering_burn;
    let stats = ship::loadout_stats(sim, data);
    let fuel_regen_per_year = stats.fuel_regen.max(0) as f32
        * data.config.ship.fuel_regen_per_point
        * subsystems::engineering_fuel_regen_factor(sim, data);
    let years = duration_years as f32;

    DepartureForecast {
        duration_years,
        travel_years,
        annual_food_output,
        annual_food_use,
        annual_route_food,
        annual_food_net,
        projected_food_end,
        recommended_food_store,
        parts_upkeep,
        fuel_burn,
        fuel_regen_per_year,
        route_hull_change: template.annual_toll.ship.hull_integrity * years,
        route_life_support_change: template.annual_toll.ship.life_support * years,
    }
}

#[cfg(test)]
mod tests;
