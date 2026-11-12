use super::c_character_creation_trial_boost::{CharacterServiceInfoRow, TrialBoostLevelState};

fn row(service_type: i32, boost_type: i32, boost_level: i32) -> CharacterServiceInfoRow {
    CharacterServiceInfoRow {
        service_type,
        boost_type,
        boost_level,
    }
}

#[test]
fn host_selector_uses_the_first_matching_service_row() {
    let mut state = TrialBoostLevelState::default();
    state.replace_rows(vec![row(1, 2, 100), row(1, 4, 71), row(1, 4, 72)]);
    state.set_selector(4);

    assert_eq!(state.starting_level(), 71);
}

#[test]
fn zero_selector_matches_the_zero_service_and_boost_row() {
    let mut state = TrialBoostLevelState::default();
    state.replace_rows(vec![row(1, 2, 100), row(0, 0, 44)]);
    state.set_selector(0);

    assert_eq!(state.starting_level(), 44);
}

#[test]
fn missing_rows_and_unmatched_service_rows_return_zero() {
    let mut state = TrialBoostLevelState::default();
    state.replace_rows(Vec::new());
    state.set_selector(4);
    assert_eq!(state.starting_level(), 0);

    state.replace_rows(vec![row(2, 4, 83)]);
    assert_eq!(state.starting_level(), 0);
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_rows_match_the_installed_12_1_5_70783_export() {
    let state = TrialBoostLevelState::default();
    assert_eq!(
        state.character_service_info_rows(),
        &[
            row(1, 2, 100),
            row(1, 3, 48),
            row(1, 1, 90),
            row(0, 0, 0),
            row(1, 4, 50),
            row(1, 6, 60),
            row(2, 1, 0),
            row(2, 2, 0),
            row(2, 3, 0),
            row(2, 4, 0),
            row(1, 8, 70),
            row(1, 11, 80),
            row(2, 5, 0),
        ]
    );
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_has_no_unverified_character_service_rows() {
    let state = TrialBoostLevelState::default();
    assert!(state.character_service_info_rows().is_empty());
    assert_eq!(state.starting_level(), 0);
}
