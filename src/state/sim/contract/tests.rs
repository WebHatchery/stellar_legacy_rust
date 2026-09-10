use super::*;

#[test]
fn objective_specific_approach_names_are_authored_for_each_family() {
    use crate::data::contracts::ContractObjective;

    for objective in [
        ContractObjective::Mining,
        ContractObjective::Colonization,
        ContractObjective::Exploration,
        ContractObjective::Rescue,
        ContractObjective::Diplomacy,
        ContractObjective::Salvage,
    ] {
        for approach in CharterApproach::ALL {
            assert!(!approach.label_for(objective).is_empty());
        }
    }
}
