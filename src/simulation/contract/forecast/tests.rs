use super::*;

fn campaign(data: &GameData) -> SimState {
    SimState::new_campaign(
        data,
        "preservers",
        81,
        &crate::state::sim::founding_faction_ids(data),
    )
}

#[test]
fn productive_ship_recommends_a_reserve_not_gross_centuries_of_food() {
    let data = GameData::load().unwrap();
    let mut sim = campaign(&data);
    sim.production.food = 10_000.0;
    let template = data.contracts.get("deep_vein_survey").unwrap();

    let forecast = for_departure(&sim, &data, template);

    assert!(forecast.annual_food_net > 0);
    assert_eq!(
        forecast.recommended_food_store,
        data.config.low_food_threshold.max(forecast.annual_food_use)
    );
    assert!(
        forecast.recommended_food_store < forecast.annual_food_use * forecast.duration_years as i64,
        "production should prevent the dossier demanding gross voyage consumption"
    );
}

#[test]
fn failing_engineering_worsens_both_sides_of_the_fuel_forecast() {
    let data = GameData::load().unwrap();
    let sound = campaign(&data);
    let mut failing = sound.clone();
    failing
        .subsystems
        .get_mut("engineering_bay")
        .unwrap()
        .condition = 0.0;
    let template = data.contracts.get("deep_vein_survey").unwrap();

    let sound = for_departure(&sound, &data, template);
    let failing = for_departure(&failing, &data, template);

    assert!(failing.fuel_burn > sound.fuel_burn);
    assert!(failing.fuel_regen_per_year < sound.fuel_regen_per_year);
}

#[test]
fn route_tolls_are_projected_across_the_whole_charter() {
    let data = GameData::load().unwrap();
    let sim = campaign(&data);
    let template = data.contracts.get("coronal_tap").unwrap();
    let forecast = for_departure(&sim, &data, template);

    assert_eq!(
        forecast.route_hull_change,
        template.annual_toll.ship.hull_integrity * template.target_duration_years as f32
    );
    assert_eq!(
        forecast.route_life_support_change,
        template.annual_toll.ship.life_support * template.target_duration_years as f32
    );
}
