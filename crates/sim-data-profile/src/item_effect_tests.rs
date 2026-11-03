//! Full effect-catalog preservation and profile selection; Lua dispatch is tested separately.

use crate::item_effects::{
    HAS_PLAYER_CONDITION_IDS, HAS_SPECIALIZATION_IDS, ITEM_EFFECT_METADATA, ITEM_EFFECTS,
    ITEM_SPELL_NAMES,
};

#[test]
fn catalogs_support_binary_search_without_collapsing_effect_rows() {
    assert!(ITEM_EFFECTS.windows(2).all(|rows| rows[0] <= rows[1]));
    assert!(
        ITEM_SPELL_NAMES
            .windows(2)
            .all(|rows| rows[0].0 < rows[1].0)
    );
    assert!(
        ITEM_EFFECT_METADATA
            .windows(2)
            .all(|rows| rows[0].0 < rows[1].0)
    );
}

#[test]
fn sparse_metadata_preserves_nonzero_values_for_associated_effects() {
    let effects: std::collections::BTreeSet<_> = ITEM_EFFECTS.iter().map(|row| row.1).collect();
    for &(effect, specialization, condition) in ITEM_EFFECT_METADATA {
        assert!(
            effects.contains(&effect),
            "metadata for unassociated effect {effect}"
        );
        assert!(
            specialization != 0 || condition != 0,
            "redundant zero metadata for {effect}"
        );
        assert!(HAS_SPECIALIZATION_IDS || specialization == 0);
        assert!(HAS_PLAYER_CONDITION_IDS || condition == 0);
    }
}

#[cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wowforever"
))]
mod selected_profile {
    use super::*;

    fn rows(item: u32) -> &'static [(u32, u32, i32, i32, i32)] {
        let start = ITEM_EFFECTS.partition_point(|row| row.0 < item);
        let end = ITEM_EFFECTS.partition_point(|row| row.0 <= item);
        &ITEM_EFFECTS[start..end]
    }

    fn name(spell: u32) -> Option<&'static str> {
        ITEM_SPELL_NAMES
            .binary_search_by_key(&spell, |row| row.0)
            .ok()
            .map(|index| ITEM_SPELL_NAMES[index].1)
    }

    fn assert_use_association(item: u32, spell: i32, expected_name: &str) {
        assert!(
            rows(item).iter().any(|row| row.2 == spell && row.3 == 0),
            "missing item {item} use-spell association {spell}"
        );
        assert_eq!(name(spell as u32), Some(expected_name));
    }

    fn assert_no_use_association(item: u32, spell: i32) {
        assert!(rows(item).iter().all(|row| row.2 != spell || row.3 != 0));
    }

    #[test]
    fn profile_retains_non_use_effects_and_spells_without_names() {
        assert!(ITEM_EFFECTS.iter().any(|row| row.3 != 0));
        assert!(ITEM_EFFECTS.iter().any(|row| {
            row.2 > 0
                && ITEM_SPELL_NAMES
                    .binary_search_by_key(&(row.2 as u32), |name| name.0)
                    .is_err()
        }));
    }

    #[test]
    fn profile_keeps_known_use_associations_and_its_own_spell_ids() {
        assert_use_association(6948, 8690, "Hearthstone");
        let drink = if cfg!(any(feature = "profile-retail", feature = "client-ptr")) {
            1291950
        } else {
            430
        };
        assert_use_association(159, drink, "Drink");

        if cfg!(feature = "client-wrath") {
            assert_use_association(33447, 43185, "Healing Potion");
            assert_no_use_association(76097, 105708);
            assert_no_use_association(233986, 1213946);
        } else if cfg!(feature = "client-mists") {
            assert_use_association(76097, 105708, "Healing Potion");
            assert_no_use_association(233986, 1213946);
        } else if cfg!(any(feature = "client-era", feature = "client-anniversary")) {
            assert_use_association(233986, 1213946, "Obsidian Bomb");
            assert_no_use_association(33447, 43185);
            assert_no_use_association(76097, 105708);
        }
    }

    #[test]
    fn sparse_metadata_records_captured_values_and_source_schema() {
        assert!(HAS_SPECIALIZATION_IDS);
        assert_eq!(HAS_PLAYER_CONDITION_IDS, !cfg!(feature = "client-wrath"));
        let expected_rows = if cfg!(any(feature = "profile-retail", feature = "client-ptr")) {
            80
        } else if cfg!(any(feature = "client-wrath", feature = "client-mists")) {
            0
        } else if cfg!(any(feature = "client-era", feature = "client-anniversary")) {
            1
        } else {
            3
        };
        assert_eq!(ITEM_EFFECT_METADATA.len(), expected_rows);
    }

    #[cfg(feature = "profile-retail")]
    #[test]
    fn retail_keeps_existing_rows_and_new_item_associations_in_one_catalog() {
        assert_use_association(278743, 55884, "Learning");
        assert_use_association(281372, 1310965, "Collect Zhevraskin Rug");
        assert_use_association(282657, 55884, "Learning");
        assert_use_association(285065, 1317439, "Mythical Soul Fragment");
        assert_eq!(rows(1165), &[(1165, 53, 7291, 0, 0)]);
        assert_eq!(name(7291), None);
        assert_eq!(
            rows(124513),
            &[
                (124513, 25299, 184897, 1, 0),
                (124513, 25300, 184898, 1, 1),
                (124513, 25301, 184983, 1, 2),
            ]
        );
        assert_eq!(
            ITEM_EFFECT_METADATA
                .binary_search_by_key(&25299, |row| row.0)
                .map(|index| ITEM_EFFECT_METADATA[index]),
            Ok((25299, 250, 0))
        );
    }

    #[cfg(feature = "client-wrath")]
    #[test]
    fn wrath_retains_negative_source_spell_ids() {
        assert_eq!(ITEM_EFFECTS.iter().filter(|row| row.2 < 0).count(), 110);
    }
}

#[cfg(not(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wowforever"
)))]
#[test]
fn unselected_profile_has_no_borrowed_rows_or_claimed_source_columns() {
    assert!(ITEM_EFFECTS.is_empty());
    assert!(ITEM_EFFECT_METADATA.is_empty());
    assert!(ITEM_SPELL_NAMES.is_empty());
    assert!(!HAS_SPECIALIZATION_IDS);
    assert!(!HAS_PLAYER_CONDITION_IDS);
}
