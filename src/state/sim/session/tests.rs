use super::*;

#[test]
fn displayed_speeds_are_truthful_multipliers() {
    assert_eq!(GameSpeed::Paused.multiplier(), 0.0);
    assert_eq!(GameSpeed::X1.multiplier(), 1.0);
    assert_eq!(GameSpeed::X2.multiplier(), 2.0);
    assert_eq!(GameSpeed::X3.multiplier(), 3.0);
}
