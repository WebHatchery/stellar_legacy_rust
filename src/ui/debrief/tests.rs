use super::*;

#[test]
fn a_defaulted_charter_is_toned_apart_from_one_the_ship_can_be_proud_of() {
    // The band is the first thing read off the banner, so a failure must
    // not come home wearing the same color as a completed run.
    assert_eq!(outcome_tone("Failure"), term::alert());
    assert_ne!(outcome_tone("Complete"), term::alert());
    assert_ne!(outcome_tone("Partial"), term::alert());
    // The lookup is case-insensitive: the band arrives as a `SuccessLevel`
    // label ("Failure"), but the report stores whatever it was given.
    assert_eq!(outcome_tone("failure"), outcome_tone("FAILURE"));
}
