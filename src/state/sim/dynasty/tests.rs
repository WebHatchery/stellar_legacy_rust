use super::*;

fn dynasty_with_leader(name: &str) -> Dynasty {
    Dynasty {
        generation: 1,
        years_since_generation: 0,
        next_member_id: 1,
        members: vec![DynastyMember {
            id: 0,
            name: name.to_owned(),
            age: 45,
            leadership: 70,
            specialization: "Command".to_owned(),
            trait_name: "Steady".to_owned(),
            is_leader: true,
        }],
        reigns: Vec::new(),
        designated_heir: None,
        births_this_generation: 0,
        leader_reign_years: 0,
        long_reign_marked: false,
        dynasty_crisis_marked: false,
        extinct: false,
    }
}

#[test]
fn a_reign_opens_on_the_sitting_captain_and_closes_where_it_ended() {
    let mut dynasty = dynasty_with_leader("Boro Chartwright");
    dynasty.begin_reign(12);
    assert_eq!(dynasty.reigns.len(), 1);
    let open = &dynasty.reigns[0];
    assert_eq!(open.name, "Boro Chartwright");
    assert_eq!(open.began_year, 12);
    assert_eq!(open.ended_year, None, "a sitting captain has no end year");
    // An open reign is counted up to the present; a closed one to its end.
    assert_eq!(open.years_held(40), 28);
    dynasty.end_reign(40);
    assert_eq!(dynasty.reigns[0].ended_year, Some(40));
    assert_eq!(
        dynasty.reigns[0].years_held(90),
        28,
        "a closed reign is fixed"
    );
}

#[test]
fn closing_a_reign_twice_keeps_the_first_end_year() {
    // A handoff and an extinction can land in the same tick; the second
    // close must not overwrite the year the chair actually passed on.
    let mut dynasty = dynasty_with_leader("Ilsa Vance");
    dynasty.begin_reign(5);
    dynasty.end_reign(30);
    dynasty.end_reign(31);
    assert_eq!(dynasty.reigns[0].ended_year, Some(30));
}

#[test]
fn an_empty_chair_records_no_captain() {
    let mut dynasty = dynasty_with_leader("Ilsa Vance");
    dynasty.members.clear();
    dynasty.begin_reign(9);
    assert!(
        dynasty.reigns.is_empty(),
        "an extinct line records no phantom captaincy"
    );
}

#[test]
fn overlap_picks_out_the_captains_a_voyage_passed_through() {
    // The debrief asks the roster which captains held the chair between the
    // launch year and the homecoming year — including one still sitting.
    let closed = |began, ended| Reign {
        name: "x".to_owned(),
        began_year: began,
        ended_year: Some(ended),
        generation: 1,
        leadership: 50,
        trait_name: String::new(),
        inherited_obligations: 0,
    };
    // Voyage runs years 20..60.
    assert!(!closed(0, 19).overlaps(20, 60), "ended before the launch");
    assert!(closed(0, 25).overlaps(20, 60), "sat at the launch");
    assert!(closed(30, 40).overlaps(20, 60), "sat wholly within");
    assert!(
        closed(55, 80).overlaps(20, 60),
        "took the chair before docking"
    );
    assert!(!closed(61, 80).overlaps(20, 60), "took it after docking");
    let mut open = closed(55, 80);
    open.ended_year = None;
    assert!(open.overlaps(20, 60), "a sitting captain is aboard");
}
