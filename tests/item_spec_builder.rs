//! WM42 derived-record builder with synthetic item inputs and ItemSpec rows.

use wow_ui_sim::c_api::item_spell::item_spec_derive::{ItemSpecInput, derive_rule_record};

const LEVEL: i32 = 80;
const NO_STATS: [i8; 10] = [-1; 10];

fn gem(gem_type: u32, first_stat: i8) -> ItemSpecInput {
    let mut stats = NO_STATS;
    stats[0] = first_stat;
    ItemSpecInput {
        class: 3,
        subclass: 0,
        inventory_type: 0,
        allowable_classes: -1,
        gem_type,
        stats,
    }
}

#[test]
fn eleven_gem_traits_overflow_capacity_before_stats() {
    let all_bits = (6..=16).fold(0, |mask, bit| mask | (1 << bit));
    // Mage 62 needs trait 38 (bit 15), mage 63 trait 39 (bit 16), mage 64
    // trait 0 (stat 5, Intellect).
    let rules = [
        (1, 123, 7, 38, 40, 62),
        (1, 123, 7, 39, 40, 63),
        (1, 123, 7, 0, 40, 64),
    ];
    let full = derive_rule_record(&gem(all_bits, 5), &rules, LEVEL);
    assert_eq!(full.specialization_ids, vec![62]);
    assert!(!full.no_stat_traits);

    let one_bit = derive_rule_record(&gem(1 << 16, 5), &rules, LEVEL);
    assert_eq!(one_bit.specialization_ids, vec![63, 64]);
}

#[test]
fn class_rejected_specific_rule_still_suppresses_later_wildcard() {
    let mut stats = NO_STATS;
    stats[0] = 3; // Agility: trait 1.
    let warglaive = ItemSpecInput {
        class: 2,
        subclass: 9, // Structural trait 28.
        inventory_type: 13,
        allowable_classes: 1 << 7, // Mage only.
        gem_type: 0,
        stats,
    };
    let wildcard = (1, 123, 5, 40, 28, 62);
    let rejected_specific = (1, 123, 5, 1, 28, 577);

    let unsuppressed = derive_rule_record(&warglaive, &[wildcard], LEVEL);
    assert_eq!(unsuppressed.specialization_ids, vec![62]);

    let suppressed = derive_rule_record(&warglaive, &[rejected_specific, wildcard], LEVEL);
    assert!(suppressed.specialization_ids.is_empty());
    assert!(suppressed.reject_unmatched);
    assert!(!suppressed.no_traits_and_specializations && !suppressed.no_stat_traits);
}
