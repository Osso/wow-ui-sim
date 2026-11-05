//! WM97 DQ4 and WM104: relic type label and item-level contribution.
//!
//! The type label comes from `GemProperties.Type`. The contribution applies the
//! bonus lists named by `SpellItemEnchantment` effects 11 and 12 to one fresh
//! accumulator; the published number is the Type 42 absolute-level selection.

use super::{bonus, tables};

const RELIC_LEVEL_CURVE: u32 = 1718;
const DISPLAY_LEVEL_MIN: i32 = 1;
const DISPLAY_LEVEL_MAX: i32 = 1300;

/// Localized through `RELIC_SLOT_TYPE_<token>`; `label` is the English fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelicSlotType {
    pub token: &'static str,
    pub label: &'static str,
}

/// WM104 2: extra artifact socket count from enchantment entry index 6.
///
/// The item-state flag bit 8192 suppresses every enchant-derived scan here. A
/// zero identity ends the contribution; the first type-8 effect supplies its
/// EffectPointsMin.
pub fn instance_socket_extra(instance_flags: u32, enchantment_id: i32) -> u32 {
    const ENCHANT_SCAN_SUPPRESSED: u32 = 8192;
    if instance_flags & ENCHANT_SCAN_SUPPRESSED != 0 || enchantment_id <= 0 {
        return 0;
    }
    tables::spell_enchantment(enchantment_id as u32)
        .and_then(|enchantment| enchantment.effects.iter().find(|effect| effect.kind == 8))
        .map_or(0, |effect| effect.points_min as u32)
}

/// WM97 DQ4 relic composition inputs selected by the item's GemProperties row.
pub struct RelicMetadata {
    /// `None` means an exact `GemProperties.Type` value with no native token.
    pub slot_type: Option<RelicSlotType>,
    /// `RELIC_TOOLTIP_ILVL_INCREASE` value; zero suppresses the line (WM104).
    pub item_levels: u16,
}

/// WM97 DQ4: the native type values and their `RELIC_SLOT_TYPE_` tokens.
pub fn relic_slot_type(gem_type: u32) -> Option<RelicSlotType> {
    let (token, label) = match gem_type {
        64 => ("IRON", "Iron"),
        128 => ("BLOOD", "Blood"),
        256 => ("SHADOW", "Shadow"),
        512 => ("FEL", "Fel"),
        1024 => ("ARCANE", "Arcane"),
        2048 => ("FROST", "Frost"),
        4096 => ("FIRE", "Fire"),
        8192 => ("LIFE", "Life"),
        16384 => ("WATER", "Water"),
        32768 => ("WIND", "Storm"),
        65536 => ("HOLY", "Holy"),
        _ => return None,
    };
    Some(RelicSlotType { token, label })
}

/// WM97 DQ4 + WM104: relic lines for the socketed item.
///
/// `display_level` is the socketed relic's display item level, never the
/// artifact's. A missing GemProperties row contributes no relic lines; a missing
/// enchantment row keeps the type label and contributes no item levels.
pub fn relic_metadata(
    gem_properties: u32,
    display_level: i32,
) -> Result<Option<RelicMetadata>, String> {
    let Some(gem) = tables::gem_properties(gem_properties) else {
        return Ok(None);
    };
    let slot_type = relic_slot_type(gem.kind);
    let mut accumulator = bonus::contribution_accumulator();
    if let Some(enchantment) = tables::spell_enchantment(gem.enchant_id) {
        // WM104: effect slots run in fixed order 0, 1, 2.
        for effect in &enchantment.effects {
            match effect.kind {
                11 => {
                    if effect.arg != 0 {
                        bonus::apply_contribution_list(effect.arg as u32, &mut accumulator)?;
                    }
                }
                12 => {
                    if let Some(list_id) = level_delta_list(display_level)? {
                        bonus::apply_contribution_list(list_id, &mut accumulator)?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(Some(RelicMetadata {
        slot_type,
        item_levels: bonus::contribution_selected_level(&accumulator),
    }))
}

/// WM104: clamp, evaluate curve 1718, truncate toward zero, floor 1, then match
/// the delta row exactly. A miss skips that effect.
fn level_delta_list(display_level: i32) -> Result<Option<u32>, String> {
    let input = display_level.clamp(DISPLAY_LEVEL_MIN, DISPLAY_LEVEL_MAX);
    let curve = tables::curve(RELIC_LEVEL_CURVE)?;
    let value = tables::curve_value(curve, input as f32)?;
    let delta = (value.trunc() as i32).max(1);
    Ok(tables::level_delta_list(delta))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_delta_rows_match_the_wm104_examples() {
        assert_eq!(level_delta_list(50).unwrap(), Some(1488));
        assert_eq!(level_delta_list(44).unwrap(), Some(1486));
        assert_eq!(level_delta_list(999).unwrap(), Some(3262));
        assert_eq!(level_delta_list(1).unwrap(), Some(1473));
        assert_eq!(level_delta_list(2).unwrap(), Some(1473));
    }

    #[test]
    fn type_values_map_exactly_and_unrecognized_values_have_no_token() {
        assert_eq!(relic_slot_type(64).unwrap().label, "Iron");
        assert_eq!(relic_slot_type(8192).unwrap().token, "LIFE");
        assert_eq!(relic_slot_type(32768).unwrap().label, "Storm");
        assert!(relic_slot_type(14).is_none());
    }

    #[test]
    fn type_one_rows_do_not_supply_the_published_number() {
        let mut accumulator = bonus::contribution_accumulator();
        bonus::apply_contribution_list(708, &mut accumulator).unwrap();
        assert_eq!(bonus::contribution_selected_level(&accumulator), 0);
        bonus::apply_contribution_list(7180, &mut accumulator).unwrap();
        assert_eq!(bonus::contribution_selected_level(&accumulator), 55);
    }
}
