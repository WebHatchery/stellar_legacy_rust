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
