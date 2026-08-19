use super::*;

#[test]
fn every_hold_offers_a_smaller_recovery_lot_and_a_full_cargo_lot() {
    assert_eq!(trade_lot_sizes(0), (12, 50));
    assert_eq!(trade_lot_sizes(50), (12, 50));
    assert_eq!(trade_lot_sizes(200), (50, 200));
    assert_eq!(trade_lot_sizes(1_000), (250, 1_000));
}
