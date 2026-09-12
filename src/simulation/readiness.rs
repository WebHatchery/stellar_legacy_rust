//! Shared readiness forecasts for the dashboard, Agenda, and autoplay.

use crate::data::GameData;
use crate::simulation::{command, crew, ship, subsystems};
use crate::state::sim::SimState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessBand {
    Strong,
    Stable,
    Vulnerable,
    Critical,
}

impl ReadinessBand {
    pub fn label(self) -> &'static str {
        match self {
            Self::Strong => "STRONG",
            Self::Stable => "STABLE",
            Self::Vulnerable => "VULNERABLE",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FoodReadiness {
    pub annual_spoilage: i64,
    pub annual_output: i64,
    pub annual_consumption: i64,
    pub annual_route_toll: i64,
    pub net_per_year: i64,
    pub gross_reserve_years: f32,
    pub net_deficit_years: Option<f32>,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuelReadiness {
    pub remaining_travel_months: u32,
    pub remaining_burn: f32,
    pub annual_scoop: f32,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadinessRow {
    pub score: f32,
    pub id: String,
    pub concern: String,
    pub band: ReadinessBand,
    pub evidence: String,
    pub trend: String,
    pub recommended_project: Option<String>,
    pub recommended_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadinessModel {
    pub food: FoodReadiness,
    pub fuel: FuelReadiness,
    pub rows: Vec<ReadinessRow>,
}

pub fn band(score: f32, data: &GameData) -> ReadinessBand {
    let cfg = data.config.readiness;
    if score >= cfg.strong_threshold {
        ReadinessBand::Strong
    } else if score >= cfg.stable_threshold {
        ReadinessBand::Stable
    } else if score >= cfg.vulnerable_threshold {
        ReadinessBand::Vulnerable
    } else {
        ReadinessBand::Critical
    }
}

pub fn forecast(sim: &SimState, data: &GameData) -> ReadinessModel {
    let crew_mult = crew::production_multipliers(sim, data);
    let output = (sim.production.food
        * crew_mult.food
        * (1.0 + subsystems::agriculture_food_bonus(sim, data))
        * subsystems::agriculture_condition_food_factor(sim, data))
    .floor() as i64;
    let consumption = (sim.population.count as f32 * data.config.food_per_person_per_year)
        .ceil()
        .max(1.0) as i64;
    let route_toll = active_route_food_toll(sim, data);
    let annual_consumption = consumption + (-route_toll).max(0);
    let after_food =
        (sim.resources.food.saturating_add(route_toll).max(0) + output - consumption).max(0);
    let spoilage = food_spoilage(after_food, &data.config);
    let net = output - consumption + route_toll - spoilage;
    let gross_years = sim.resources.food as f32 / consumption.max(1) as f32;
    let deficit_years = (net < 0).then(|| sim.resources.food as f32 / (-net) as f32);
    let food_score = if net >= 0 {
        (gross_years / 10.0).clamp(0.0, 1.0)
    } else {
        (deficit_years.unwrap_or(0.0) / 10.0).clamp(0.0, 1.0)
    };

    let (travel_months, remaining_burn) = remaining_fuel_need(sim, data);
    let stats = ship::loadout_stats(sim, data);
    let annual_scoop = stats.fuel_regen.max(0) as f32
        * data.config.ship.fuel_regen_per_point
        * subsystems::engineering_fuel_regen_factor(sim, data);
    let fuel_score = if remaining_burn <= 0.0 {
        1.0
    } else {
        ((sim.ship.fuel + annual_scoop * (travel_months as f32 / 12.0)) / remaining_burn)
            .clamp(0.0, 1.0)
    };
    let food = FoodReadiness {
        annual_spoilage: spoilage,
        annual_output: output,
        annual_consumption,
        annual_route_toll: route_toll,
        net_per_year: net,
        gross_reserve_years: gross_years,
        net_deficit_years: deficit_years,
        score: food_score,
    };
    let fuel = FuelReadiness {
        remaining_travel_months: travel_months,
        remaining_burn,
        annual_scoop,
        score: fuel_score,
    };
    let engineering_score = sim
        .subsystems
        .get("engineering_bay")
        .map_or(sim.ship.hull_integrity, |state| {
            state.condition.min(sim.ship.hull_integrity)
        });
    let air_score = sim.ship.life_support.min(
        sim.subsystems
            .get("life_support_habitat")
            .map_or(1.0, |state| state.condition),
    );
    let knowledge_score = if sim.subsystems.is_empty() {
        0.0
    } else {
        GameData::sorted_ids(&data.subsystems)
            .iter()
            .filter_map(|id| sim.subsystems.get(id))
            .map(|state| state.knowledge)
            .sum::<f32>()
            / sim.subsystems.len() as f32
    };
    let cohesion_score =
        (sim.population.morale + sim.population.unity + sim.population.stability) / 3.0;
    let mut rows = vec![
        row(
            "food",
            "FOOD",
            band(food.score, data),
            format_food(&food),
            trend(food.score),
            "optimise_hydroponics",
            Some("agriculture"),
        ),
        row(
            "engineering",
            "ENGINEERING",
            band(engineering_score, data),
            format!(
                "Hull {:.0}% · bay {:.0}%",
                sim.ship.hull_integrity * 100.0,
                sim.subsystems
                    .get("engineering_bay")
                    .map_or(0.0, |bay| bay.condition)
                    * 100.0
            ),
            trend(engineering_score),
            if sim.ship.hull_integrity < data.config.readiness.stable_threshold {
                "restore_hull"
            } else {
                "service_subsystem"
            },
            if sim.ship.hull_integrity < data.config.readiness.stable_threshold {
                None
            } else {
                Some("engineering_bay")
            },
        ),
        row(
            "life_support",
            "LIFE SUPPORT",
            band(air_score, data),
            format!("Air {:.0}%", sim.ship.life_support * 100.0),
            trend(air_score),
            "overhaul_life_support",
            None,
        ),
        row(
            "knowledge",
            "KNOWLEDGE / LEADERSHIP",
            band(knowledge_score, data),
            format!("Average craft {:.0}%", knowledge_score * 100.0),
            trend(knowledge_score),
            "train_replacement_cohort",
            weakest_knowledge(sim),
        ),
        row(
            "cohesion",
            "SOCIAL COHESION",
            band(cohesion_score, data),
            format!(
                "Morale {:.0}% · unity {:.0}%",
                sim.population.morale * 100.0,
                sim.population.unity * 100.0
            ),
            trend(cohesion_score),
            "restore_crew_quarters",
            None,
        ),
    ];
    {
        rows.push(row(
            "fuel",
            "FUEL",
            band(fuel.score, data),
            format!(
                "{} travel months · burn {:.2}",
                fuel.remaining_travel_months, fuel.remaining_burn
            ),
            trend(fuel.score),
            "",
            None,
        ));
    }
    for row in &mut rows {
        row.score = match row.id.as_str() {
            "food" => food.score,
            "fuel" => fuel.score,
            "engineering" => engineering_score,
            "life_support" => air_score,
            "knowledge" => knowledge_score,
            _ => cohesion_score,
        };
        let previous = sim.projects.readiness_history.get(&row.id);
        if let Some(previous) = previous {
            let rank = band_rank(row.band);
            if rank > previous.band {
                let threshold = match rank {
                    3 => data.config.readiness.strong_threshold,
                    2 => data.config.readiness.stable_threshold,
                    _ => data.config.readiness.vulnerable_threshold,
                };
                if row.score < threshold + data.config.readiness.recovery_hysteresis {
                    row.band = rank_band(previous.band);
                }
            }
            let delta = row.score - previous.score;
            row.trend = if delta.abs() < 1e-5 {
                previous.trend.clone()
            } else if delta > 0.0 {
                "IMPROVING".into()
            } else {
                "FALLING".into()
            };
        } else {
            row.trend = "NO PRIOR READING".into();
        }
        if row.id == "food" {
            row.trend = if food.net_per_year > 0 {
                "STORES GROWING"
            } else if food.net_per_year < 0 {
                "STORES DEPLETING"
            } else {
                "STORES BALANCED"
            }
            .into();
        }
        if row.band == ReadinessBand::Strong || row.band == ReadinessBand::Stable {
            row.recommended_project = None;
            row.recommended_target = None;
        }
        if row.id == "food" && sim.issues.has_active("agriculture_blight") {
            row.recommended_project = Some("sterilise_damaged_growing_systems".into());
            row.recommended_target = Some("agriculture".into());
        }
    }
    rows.sort_by_key(|row| match row.band {
        ReadinessBand::Critical => 0,
        ReadinessBand::Vulnerable => 1,
        ReadinessBand::Stable => 2,
        ReadinessBand::Strong => 3,
    });
    ReadinessModel { food, fuel, rows }
}

fn row(
    id: &str,
    concern: &str,
    band: ReadinessBand,
    evidence: String,
    trend: String,
    project: &str,
    target: Option<&str>,
) -> ReadinessRow {
    ReadinessRow {
        score: 0.0,
        id: id.to_owned(),
        concern: concern.to_owned(),
        band,
        evidence,
        trend,
        recommended_project: (!project.is_empty()).then(|| project.to_owned()),
        recommended_target: target.map(str::to_owned),
    }
}

fn format_food(food: &FoodReadiness) -> String {
    let net = food.net_deficit_years.map_or_else(
        || "Surplus at current conditions".to_owned(),
        |years| format!("{years:.1} years to depletion at current deficit"),
    );
    format!("{:.1} gross years · {net}", food.gross_reserve_years)
}

fn trend(_value: f32) -> String {
    "NO PRIOR READING".to_owned()
}

pub fn food_spoilage(stores: i64, config: &crate::data::GameConfig) -> i64 {
    if config.food_carrying_capacity <= 0 {
        return 0;
    }
    ((stores - config.food_carrying_capacity).max(0) as f32 * config.food_spoilage_fraction).round()
        as i64
}

fn band_rank(band: ReadinessBand) -> u8 {
    match band {
        ReadinessBand::Critical => 0,
        ReadinessBand::Vulnerable => 1,
        ReadinessBand::Stable => 2,
        ReadinessBand::Strong => 3,
    }
}
fn rank_band(rank: u8) -> ReadinessBand {
    match rank {
        0 => ReadinessBand::Critical,
        1 => ReadinessBand::Vulnerable,
        2 => ReadinessBand::Stable,
        _ => ReadinessBand::Strong,
    }
}

/// Store observations only at authoritative updates, never during UI drawing.
pub fn refresh(sim: &mut SimState, data: &GameData) {
    for row in forecast(sim, data).rows {
        sim.projects.readiness_history.insert(
            row.id,
            crate::state::sim::projects::ReadinessSample {
                score: row.score,
                band: band_rank(row.band),
                trend: row.trend,
            },
        );
    }
}

fn weakest_knowledge(sim: &SimState) -> Option<&str> {
    sim.subsystems
        .iter()
        .min_by(|left, right| {
            left.1
                .knowledge
                .total_cmp(&right.1.knowledge)
                .then_with(|| left.0.cmp(right.0))
        })
        .map(|(id, _)| id.as_str())
}

fn active_route_food_toll(sim: &SimState, data: &GameData) -> i64 {
    sim.contract
        .as_ref()
        .and_then(|contract| data.contracts.get(&contract.template_id))
        .map_or(0, |template| template.annual_toll.resource.food)
}

fn remaining_fuel_need(sim: &SimState, data: &GameData) -> (u32, f32) {
    let Some(contract) = &sim.contract else {
        return (0, 0.0);
    };
    let mut months = 0u32;
    let mut elapsed = 0u32;
    for phase in &contract.phases {
        let phase_months = phase.years * 12;
        let start = elapsed;
        let end = elapsed + phase_months;
        let from = contract.months_elapsed.max(start);
        if phase.kind == crate::data::contracts::ContractPhase::Travel && from < end {
            months += end - from;
        }
        elapsed = end;
    }
    let burn = data.config.provisioning.fuel_burn_per_travel_month
        * months as f32
        * subsystems::engineering_fuel_burn_factor(sim, data)
        * command::fuel_burn_factor(sim.command_posture)
        * sim.contract.as_ref().map_or(1.0, |contract| {
            crate::simulation::approach::fuel_burn_factor(contract.approach)
        });
    (months, burn)
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/simulation/readiness/tests.rs"
    ));
}
