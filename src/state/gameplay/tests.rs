use super::Screen;

#[test]
fn tab_set_changes_with_voyage_state() {
    // Docked: the refit board (DRYDOCK + MARKET), no active CONTRACT.
    let docked = Screen::tabs(true);
    assert!(docked.contains(&Screen::Drydock));
    assert!(docked.contains(&Screen::Market));
    assert!(!docked.contains(&Screen::Contract));

    // Under way: the operations set (CONTRACT), no DRYDOCK board, no MARKET.
    let underway = Screen::tabs(false);
    assert!(underway.contains(&Screen::Contract));
    assert!(!underway.contains(&Screen::Drydock));
    assert!(!underway.contains(&Screen::Market));

    // The dashboard is always reachable in both states.
    assert!(docked.contains(&Screen::Dashboard));
    assert!(underway.contains(&Screen::Dashboard));
}
