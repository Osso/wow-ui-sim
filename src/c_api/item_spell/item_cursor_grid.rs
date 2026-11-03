//! Owned-item cursor grid eligibility (WM54 Q4).
//!
//! VERIFIED: after the staged item resolves, native class 17 is excluded.
//! Otherwise the grid shows when the item has a nonzero selected use spell or
//! its base Item record has a nonzero InventoryType; a missing record gives
//! zero. The use-spell selection is the GetItemSpell effect route (trigger 0).
//! Its enchantment route does not apply: cursor items carry no enchantments in
//! this model. Gap: the item-action sub-branch with native flags.

use super::item_effects::selected_use_spell;
use sim_data::item_cursor_grid::{CLASS_17_ITEMS, INVENTORY_TYPED_ITEM_RANGES};

pub(crate) fn owned_item_shows_grid(item: u32) -> bool {
    if CLASS_17_ITEMS.binary_search(&item).is_ok() {
        return false;
    }
    selected_use_spell(item).is_some() || has_inventory_type(item)
}

fn has_inventory_type(item: u32) -> bool {
    let next = INVENTORY_TYPED_ITEM_RANGES.partition_point(|&(start, _)| start <= item);
    next.checked_sub(1)
        .is_some_and(|index| item <= INVENTORY_TYPED_ITEM_RANGES[index].1)
}
