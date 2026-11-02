//! Read-only departure forecasting for the PREP dossier.
//!
//! This is deliberately a baseline projection, not a promise: it holds current
//! population, crew, subsystem condition, and route terms steady. Events and
//! later deterioration remain the uncertainty a generational voyage is about.

use crate::data::contracts::{ContractPhase, ContractTemplate};
use crate::data::GameData;
use crate::simulation::{command, crew, ship, subsystems};
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

/// Stores at the start of each charter year, index 0 = departure and the last
/// entry = berth. One point per year keeps the curve cheap to draw every frame.
#[derive(Debug, Clone, PartialEq)]
pub struct StockCurve {
    pub food: Vec<i64>,
    pub fuel: Vec<f32>,
    /// First charter year the stores run out, if they do.
    pub food_dry_year: Option<u32>,
    /// First Travel year the burn empties the tank before scoops refill it.
    pub fuel_dry_year: Option<u32>,
}

impl DepartureForecast {
    /// Baseline year-by-year projection from the given starting stores. Food
    /// follows the steady annual net; the tank burns through Travel years and
    /// scoops every year, capped at one full tank. Events are not forecast,
    /// matching the rest of this baseline.
    pub fn stock_curve(&self, food: i64, fuel: f32, template: &ContractTemplate) -> StockCurve {
        let years = self.duration_years as usize;
        let burn_per_travel_year = if self.travel_years > 0 {
            self.fuel_burn / self.travel_years as f32
        } else {
            0.0
        };
        let mut travel_year = Vec::with_capacity(years);
        for phase in &template.phases {
            for _ in 0..phase.years {
                travel_year.push(phase.kind == ContractPhase::Travel);
            }
        }
        travel_year.resize(years, false);
        let mut curve = StockCurve {
            food: Vec::with_capacity(years + 1),
            fuel: Vec::with_capacity(years + 1),
            food_dry_year: None,
            fuel_dry_year: None,
        };
        let (mut food_now, mut fuel_now) = (food, fuel.clamp(0.0, 1.0));
        curve.food.push(food_now);
        curve.fuel.push(fuel_now);
        for (index, travelling) in travel_year.into_iter().enumerate() {
            let year = index as u32 + 1;
            food_now = food_now.saturating_add(self.annual_food_net).max(0);
            if food_now == 0 && curve.food_dry_year.is_none() {
                curve.food_dry_year = Some(year);
            }
            if travelling {
                fuel_now -= burn_per_travel_year;
                if fuel_now <= 0.0 {
                    fuel_now = 0.0;
                    curve.fuel_dry_year.get_or_insert(year);
                }
            }
            fuel_now = (fuel_now + self.fuel_regen_per_year).min(1.0);
            curve.food.push(food_now);
            curve.fuel.push(fuel_now);
        }
        curve
    }
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
        * engineering_burn
        * command::fuel_burn_factor(sim.command_posture)
        * crate::simulation::approach::fuel_burn_factor(sim.selected_charter_approach);
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
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/contract/forecast/tests.rs"
    ));
}
