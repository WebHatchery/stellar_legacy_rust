use super::*;

#[test]
fn displayed_speeds_are_truthful_multipliers() {
    assert_eq!(GameSpeed::Paused.multiplier(), 0.0);
    assert_eq!(GameSpeed::X1.multiplier(), 1.0);
    assert_eq!(GameSpeed::X2.multiplier(), 2.0);
    assert_eq!(GameSpeed::X3.multiplier(), 3.0);
}

#[test]
fn pause_remembers_speed_through_save_reload() {
    let data = crate::data::GameData::load().unwrap();
    let mut sim = super::super::SimState::new_campaign(
        &data,
        "preservers",
        81,
        &super::super::founding_faction_ids(&data),
    );
    sim.set_speed(GameSpeed::X3);
    sim.toggle_pause();
    assert_eq!(sim.speed, GameSpeed::Paused);
    let mut restored: super::super::SimState =
        serde_json::from_str(&serde_json::to_string(&sim).unwrap()).unwrap();
    restored.toggle_pause();
    assert_eq!(restored.speed, GameSpeed::X3);
}
