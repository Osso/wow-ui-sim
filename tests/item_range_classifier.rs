//! WM43 range-index classifier over supplied restriction and effect rows,
//! with the generated 70391 Difficulty fallback chain.

use sim_data::item_spell_ranges::DIFFICULTY_FALLBACKS;
use wow_ui_sim::c_api::item_spell::item_range::{Reach, classify_reach, restriction_targets};

const HELPFUL_RESTRICTION: u32 = 256;
const HARMFUL_RESTRICTION: u32 = 128;

#[test]
fn restriction_lookup_follows_difficulty_fallbacks_to_the_first_match() {
    let base_only = [(0, HELPFUL_RESTRICTION)];
    assert_eq!(
        restriction_targets(&base_only, DIFFICULTY_FALLBACKS, 0),
        Some(256)
    );
    // Mythic dungeon 23 falls back 23 -> 2 -> 1 -> 0.
    assert_eq!(
        restriction_targets(&base_only, DIFFICULTY_FALLBACKS, 23),
        Some(256)
    );
    let heroic_and_base = [(2, HARMFUL_RESTRICTION), (0, HELPFUL_RESTRICTION)];
    assert_eq!(
        restriction_targets(&heroic_and_base, DIFFICULTY_FALLBACKS, 23),
        Some(128)
    );
    // A difficulty without a Difficulty row ends the lookup without a match.
    assert_eq!(
        restriction_targets(&base_only, DIFFICULTY_FALLBACKS, 9999),
        None
    );
    let duplicate_base = [(0, HARMFUL_RESTRICTION), (0, HELPFUL_RESTRICTION)];
    assert_eq!(
        restriction_targets(&duplicate_base, DIFFICULTY_FALLBACKS, 0),
        Some(128)
    );
}

#[test]
fn restriction_targets_override_effects_in_bit_priority_order() {
    let helpful_effect = [(0, 1, 0)];
    let harmful_effect = [(0, 6, 0)];
    assert_eq!(
        classify_reach(Some(256 | 128), &harmful_effect),
        Reach::Helpful
    );
    assert_eq!(
        classify_reach(Some(128 | 2), &helpful_effect),
        Reach::Harmful
    );
    assert_eq!(classify_reach(Some(2), &helpful_effect), Reach::Neither);
    // Other bits continue to the effect scans.
    assert_eq!(classify_reach(Some(64), &helpful_effect), Reach::Helpful);
}

#[test]
fn harmful_effect_evidence_wins_over_helpful_evidence() {
    assert_eq!(
        classify_reach(None, &[(0, 1, 0), (0, 0, 6)]),
        Reach::Harmful
    );
    // EffectAttributes 4096 forces harmful even on a helpful target.
    assert_eq!(classify_reach(None, &[(4096, 1, 0)]), Reach::Harmful);
    assert_eq!(classify_reach(None, &[(0, 0, 21)]), Reach::Helpful);
    // Target 86 is in neither table.
    assert_eq!(classify_reach(None, &[(0, 86, 0)]), Reach::Neither);
    assert_eq!(classify_reach(None, &[]), Reach::Neither);
}
