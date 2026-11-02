//! Derived item-specialization records (WM42 and its structural-trait
//! follow-up) built from build-70391 DB2 rows.
//!
//! Rule and override traversal uses CSV row order; native order is UNKNOWN,
//! so this order is INFERRED.

use sim_data::item_spec_inputs::{ITEMS, OVERRIDES, RULES, SPEC_CLASSES};

use super::item_spec::ItemSpecRecord;

const WILDCARD: u8 = 40;
const TRAIT_CAPACITY: usize = 10;
const WEAPON_CATEGORY: u8 = 5;
const GEM_CATEGORY: u8 = 7;
/// Marker initial values: no rule ItemType is 8 and no specific primary is 40.
const NO_SPECIFIC_CATEGORY: u8 = 8;
const NO_TWO_TRAIT_PRIMARY: u8 = WILDCARD;

/// ItemSparse and Item fields read by the builder.
#[derive(Debug, Clone, Copy)]
pub struct ItemSpecInput {
    pub class: u8,
    pub subclass: u8,
    pub inventory_type: u8,
    pub allowable_classes: i32,
    /// GemProperties.Type selected by ItemSparse.Gem_properties.
    pub gem_type: u32,
    /// ItemSparse.StatModifier_bonusStat slots; -1 is empty.
    pub stats: [i8; 10],
}

/// ItemSpec row: MinLevel, MaxLevel, ItemType, PrimaryStat, SecondaryStat,
/// SpecializationID.
pub type ItemSpecRule = (u8, u8, u8, u8, u8, u32);

type ItemRow = (u32, u8, u8, u8, i32, u32, [i8; 10], u8, bool);

fn find_item_row(item_id: u32) -> Option<&'static ItemRow> {
    let index = ITEMS.binary_search_by_key(&item_id, |row| row.0).ok()?;
    Some(&ITEMS[index])
}

fn item_input(row: &ItemRow) -> ItemSpecInput {
    let &(_, class, subclass, inventory_type, allowable_classes, gem_type, stats, ..) = row;
    ItemSpecInput {
        class,
        subclass,
        inventory_type,
        allowable_classes,
        gem_type,
        stats,
    }
}

/// ItemSparse.Bonding keyed by item ID (field name INFERRED; WM48).
pub(super) fn item_bonding(item_id: u32) -> Option<u8> {
    find_item_row(item_id).map(|row| row.7)
}

/// Build the derived record, or None when ItemSparse/Item data is missing.
/// Normal derivation copies ItemSparse flag 134217728 into itemFlag27;
/// accepted overrides leave it false (WM48).
pub(super) fn derive_record(item_id: u32, level: i32) -> Option<ItemSpecRecord> {
    let row = find_item_row(item_id)?;
    let input = item_input(row);
    Some(
        derive_overrides(item_id, &input).unwrap_or_else(|| ItemSpecRecord {
            item_flag_27: row.8,
            ..derive_rule_record(&input, RULES, level)
        }),
    )
}

/// ChrSpecialization ClassID and OrderIndex (INFERRED field name, WM48).
pub(super) fn specialization_row(spec_id: u32) -> Option<(u8, u8)> {
    let index = SPEC_CLASSES
        .binary_search_by_key(&spec_id, |row| row.0)
        .ok()?;
    let (_, class, order_index) = SPEC_CLASSES[index];
    Some((class, order_index))
}

fn specialization_class(spec_id: u32) -> u8 {
    specialization_row(spec_id).map_or(0, |(class, _)| class)
}

/// Tests the ChrSpecialization.ClassID-minus-one bit; a missing row tests bit 31.
fn allows_specialization(mask: i32, spec_id: u32) -> bool {
    let bit = specialization_class(spec_id).wrapping_sub(1) & 31;
    (mask as u32) & (1_u32 << bit) != 0
}

/// Accepted overrides replace rule matching; none accepted falls through.
fn derive_overrides(item_id: u32, input: &ItemSpecInput) -> Option<ItemSpecRecord> {
    let start = OVERRIDES.partition_point(|row| row.0 < item_id);
    let end = OVERRIDES.partition_point(|row| row.0 <= item_id);
    let specialization_ids: Vec<u32> = OVERRIDES[start..end]
        .iter()
        .filter_map(|&(_, spec)| {
            allows_specialization(input.allowable_classes, spec).then_some(spec)
        })
        .collect();
    (!specialization_ids.is_empty()).then_some(ItemSpecRecord {
        no_traits_and_specializations: false,
        no_stat_traits: false,
        reject_unmatched: true,
        item_flag_27: false,
        specialization_ids,
    })
}

fn weapon_trait(subclass: u8) -> Option<u8> {
    match subclass {
        0 => Some(7),
        1 => Some(8),
        2 => Some(16),
        3 => Some(15),
        4 => Some(11),
        5 => Some(12),
        6 => Some(19),
        7 => Some(9),
        8 => Some(10),
        9 => Some(28),
        10 => Some(18),
        13 => Some(14),
        15 => Some(13),
        16 => Some(20),
        18 => Some(17),
        19 => Some(21),
        _ => None,
    }
}

fn armor_category(subclass: u8, inventory_type: u8) -> (u8, Option<u8>) {
    const HELD_IN_OFF_HAND: u8 = 23;
    const CLOAK: u8 = 16;
    match (subclass, inventory_type) {
        (0, HELD_IN_OFF_HAND) => (6, Some(23)),
        (1, CLOAK) => (0, Some(27)),
        (1..=4, _) => (subclass, None),
        (6, _) => (6, Some(22)),
        (7..=11, _) => (6, Some(23)),
        _ => (0, None),
    }
}

fn select_category(input: &ItemSpecInput) -> (u8, Option<u8>) {
    match input.class {
        2 => (WEAPON_CATEGORY, weapon_trait(input.subclass)),
        3 => (GEM_CATEGORY, None),
        4 => armor_category(input.subclass, input.inventory_type),
        _ => (0, None),
    }
}

/// One shared list: capacity is checked before the duplicate scan.
struct Traits {
    ids: Vec<u8>,
    stat_count: usize,
}

impl Traits {
    fn append(&mut self, trait_id: u8) -> bool {
        if self.ids.len() == TRAIT_CAPACITY || self.ids.contains(&trait_id) {
            return false;
        }
        self.ids.push(trait_id);
        true
    }

    fn append_stat(&mut self, trait_id: u8) {
        self.stat_count += usize::from(self.append(trait_id));
    }
}

fn stat_traits(stat: i8) -> &'static [u8] {
    match stat {
        3 => &[1],
        4 => &[2],
        5 => &[0],
        6 => &[3],
        13 => &[5],
        14 => &[6],
        19 | 20 | 21 | 32 => &[24],
        31 => &[4],
        36 => &[25],
        50 => &[26],
        71 => &[1, 2, 0],
        72 => &[1, 2],
        73 => &[1, 0],
        74 => &[2, 0],
        _ => &[],
    }
}

/// Structural trait, then gem property bits 6–16 (category 7 only), then
/// the ten stored stat slots in order.
fn collect_traits(input: &ItemSpecInput, category: u8, structural: Option<u8>) -> Traits {
    let mut traits = Traits {
        ids: Vec::with_capacity(TRAIT_CAPACITY),
        stat_count: 0,
    };
    if let Some(trait_id) = structural {
        traits.append(trait_id);
    }
    if category == GEM_CATEGORY {
        for bit in 6..=16_u8 {
            if input.gem_type & (1 << bit) != 0 {
                traits.append_stat(bit + 23);
            }
        }
    }
    for &stat in &input.stats {
        for &trait_id in stat_traits(stat) {
            traits.append_stat(trait_id);
        }
    }
    traits
}

/// Normal derivation: categories, traits and one ItemSpec traversal.
pub fn derive_rule_record(
    input: &ItemSpecInput,
    rules: &[ItemSpecRule],
    level: i32,
) -> ItemSpecRecord {
    let (category, structural) = select_category(input);
    let traits = collect_traits(input, category, structural);
    let mut matcher = RuleMatcher {
        category,
        traits: &traits.ids,
        level,
        allowable_classes: input.allowable_classes,
        last_specific_category: NO_SPECIFIC_CATEGORY,
        last_two_trait_primary: NO_TWO_TRAIT_PRIMARY,
        reject_unmatched: false,
        specialization_ids: Vec::new(),
    };
    for &rule in rules {
        matcher.visit(rule);
    }
    ItemSpecRecord {
        no_traits_and_specializations: traits.ids.is_empty()
            && matcher.specialization_ids.is_empty(),
        no_stat_traits: traits.stat_count == 0,
        reject_unmatched: matcher.reject_unmatched,
        item_flag_27: false,
        specialization_ids: matcher.specialization_ids,
    }
}

/// One category-keyed traversal sharing two last-value suppression markers.
struct RuleMatcher<'a> {
    category: u8,
    traits: &'a [u8],
    level: i32,
    allowable_classes: i32,
    last_specific_category: u8,
    last_two_trait_primary: u8,
    reject_unmatched: bool,
    specialization_ids: Vec<u32>,
}

impl RuleMatcher<'_> {
    fn visit(&mut self, (min, max, item_type, primary, secondary, spec): ItemSpecRule) {
        if item_type != self.category || !self.matches(primary, secondary) {
            return;
        }
        if self.suppressed(item_type, primary, secondary) {
            return;
        }
        self.reject_unmatched = true;
        if !(i32::from(min)..=i32::from(max)).contains(&self.level) {
            return;
        }
        let allowed = allows_specialization(self.allowable_classes, spec);
        if allowed && !self.specialization_ids.contains(&spec) {
            self.specialization_ids.push(spec);
        }
        // Writes follow the append attempt, even after class rejection.
        if primary != WILDCARD {
            self.last_specific_category = item_type;
            if secondary != WILDCARD {
                self.last_two_trait_primary = primary;
            }
        }
    }

    /// A wildcard primary ignores the secondary, except in category 5.
    /// 70391 category-5 wildcard-primary rows all carry a specific secondary.
    fn matches(&self, primary: u8, secondary: u8) -> bool {
        let secondary_matches = secondary == WILDCARD || self.traits.contains(&secondary);
        if primary == WILDCARD {
            return self.category != WEAPON_CATEGORY || secondary_matches;
        }
        self.traits.contains(&primary) && secondary_matches
    }

    fn suppressed(&self, item_type: u8, primary: u8, secondary: u8) -> bool {
        if primary == WILDCARD {
            return self.last_specific_category == item_type;
        }
        secondary == WILDCARD && self.last_two_trait_primary == primary
    }
}
