use super::*;

#[test]
fn positive_food_production_is_reported_as_surplus_not_infinite_safety() {
    let data = crate::data::GameData::load().unwrap();
    let ids = crate::state::sim::founding_faction_ids(&data);
    let sim = SimState::new_campaign(&data, "preservers", 12, &ids);
    let model = forecast(&sim, &data);
    assert!(model.food.net_per_year >= 0);
    assert!(model.food.net_deficit_years.is_none());
    assert!(format_food(&model.food).contains("Surplus"));
}

fn campaign() -> (GameData, SimState) {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        12,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.contract = Some(crate::simulation::contract::start_contract(
        data.contracts.get("deep_vein_survey").unwrap(),
        &sim,
    ));
    (data, sim)
}

#[test]
fn fuel_coverage_divides_all_available_fuel_by_route_burn() {
    let (data, mut sim) = campaign();
    sim.ship.fuel = 0.2;
    let model = forecast(&sim, &data);
    assert!(model.fuel.remaining_burn > 0.0);
    let expected = ((0.2
        + model.fuel.annual_scoop * model.fuel.remaining_travel_months as f32 / 12.0)
        / model.fuel.remaining_burn)
        .clamp(0.0, 1.0);
    assert!((expected - model.fuel.score).abs() < 1e-6);
}

#[test]
fn surplus_and_spoilage_match_the_authoritative_annual_food_tick() {
    let (mut data, mut sim) = campaign();
    data.config.event_chance_base = 0.0;
    data.config.event_chance_cap = 0.0;
    sim.resources.food = 100_000;
    sim.month_clock = 11;
    let before = sim.resources.food;
    let estimate = forecast(&sim, &data).food;
    assert!(estimate.annual_spoilage > 0);
    crate::simulation::tick::advance_months(&mut sim, &data, 1);
    assert_eq!(sim.resources.food - before, estimate.net_per_year);
}

#[test]
fn recovery_hysteresis_and_trend_follow_observations() {
    let (data, mut sim) = campaign();
    sim.ship.hull_integrity = data.config.readiness.vulnerable_threshold - 0.01;
    refresh(&mut sim, &data);
    sim.ship.hull_integrity = data.config.readiness.vulnerable_threshold + 0.01;
    let rows = forecast(&sim, &data).rows;
    let row = rows.iter().find(|r| r.id == "engineering").unwrap();
    assert_eq!(row.band, ReadinessBand::Critical);
    assert_eq!(row.trend, "IMPROVING");
    sim.ship.hull_integrity += data.config.readiness.recovery_hysteresis;
    let rows = forecast(&sim, &data).rows;
    assert_eq!(
        rows.iter().find(|r| r.id == "engineering").unwrap().band,
        ReadinessBand::Vulnerable
    );
}

#[test]
fn hydroponics_delivery_and_blight_recovery_change_shared_production() {
    let (data, mut sim) = campaign();
    let base = forecast(&sim, &data).food.annual_output;
    sim.projects.hydroponics_bonus = 0.06;
    let improved = forecast(&sim, &data).food.annual_output;
    assert!(improved > base);
    let event = data.events.get("crop_blight").unwrap();
    crate::simulation::event_resolver::apply_outcome(&mut sim, &data, event, 0);
    let blighted = forecast(&sim, &data).food.annual_output;
    let food_after_event = sim.resources.food;
    assert!(sim.issues.has_active("agriculture_blight"));
    let id = crate::simulation::projects::queue_project(
        &mut sim,
        &data,
        "sterilise_damaged_growing_systems",
        Some("agriculture".into()),
    )
    .unwrap();
    for _ in 0..48 {
        crate::simulation::projects::advance_projects(&mut sim, &data);
    }
    assert_eq!(
        sim.projects.find(id).unwrap().status,
        crate::state::sim::ProjectStatus::Completed
    );
    assert!(!sim.issues.has_active("agriculture_blight"));
    assert!(forecast(&sim, &data).food.annual_output > blighted);
    assert!(
        sim.resources.food < food_after_event,
        "the original event loss must not be refunded"
    );
}

#[test]
fn prepared_blight_response_is_legal_only_after_full_seed_delivery() {
    let (data, mut prepared) = campaign();
    let mut unprepared = prepared.clone();
    let event = data.events.get("crop_blight").unwrap();
    assert!(
        !crate::simulation::event_resolver::available_outcome_indices(&prepared, event)
            .contains(&2)
    );
    let id = crate::simulation::projects::queue_project(
        &mut prepared,
        &data,
        "establish_seed_programme",
        Some("agriculture".into()),
    )
    .unwrap();
    for _ in 0..215 {
        crate::simulation::projects::advance_projects(&mut prepared, &data);
    }
    assert!(!prepared.projects.has_capability("seed_programme"));
    crate::simulation::projects::advance_projects(&mut prepared, &data);
    assert_eq!(
        prepared.projects.find(id).unwrap().status,
        crate::state::sim::ProjectStatus::Completed
    );
    assert!(
        crate::simulation::event_resolver::available_outcome_indices(&prepared, event).contains(&2)
    );
    let prepared_food = prepared.resources.food;
    let unprepared_food = unprepared.resources.food;
    crate::simulation::event_resolver::apply_outcome(&mut prepared, &data, event, 2);
    crate::simulation::event_resolver::apply_outcome(&mut unprepared, &data, event, 0);
    assert!(prepared_food - prepared.resources.food < unprepared_food - unprepared.resources.food);
    assert!(!prepared.issues.has_active("agriculture_blight"));
    assert!(unprepared.issues.has_active("agriculture_blight"));
}
