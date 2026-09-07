use super::*;

#[test]
fn every_existing_screen_has_a_reachable_destination_in_its_voyage_state() {
    for in_port in [true, false] {
        for &screen in Screen::tabs(in_port) {
            let destination = Destination::of(screen);
            assert!(Destination::ALL.contains(&destination));
            assert!(
                destination.home(in_port) == screen
                    || destination
                        .sections(in_port)
                        .iter()
                        .any(|(_, target)| *target == screen)
            );
        }
    }
}

#[test]
fn voyage_navigation_preserves_market_and_contract_restrictions() {
    assert_eq!(Destination::Voyage.home(true), Screen::Drydock);
    assert_eq!(Destination::Voyage.home(false), Screen::Contract);
    assert!(!Destination::Voyage
        .sections(false)
        .iter()
        .any(|(_, screen)| *screen == Screen::Market));
    assert_eq!(
        Destination::Ship.sections(true),
        Destination::Ship.sections(false)
    );
}
