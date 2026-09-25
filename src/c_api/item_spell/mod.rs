mod c_container;
mod c_currency;
mod c_equipment_set;
mod c_item;
pub(crate) mod helpers;

use super::c_spell;
use super::c_spell_book;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

#[cfg(feature = "client-wowforever")]
pub(crate) use c_container::{BAG_SLOT_FLAG_EXCLUDE_JUNK_SELL, bag_slot_flag_is_set};
pub(crate) use c_container::{
    c_container_get_item_id, c_container_get_item_link, c_container_get_num_slots,
};
pub(crate) use c_item::{
    c_item_get_item_id, c_item_is_consumable_item, c_item_is_equippable_item,
    c_item_is_item_in_range, item_link_for_id, parse_item_guid, parse_item_id_from_val,
    parse_prefixed_id, push_item_info, spell_link_for_id,
};
pub(crate) use helpers::current_item_upgrade_location;
pub(crate) use helpers::item_class_name;

pub(crate) fn register_item_and_spell_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_item::register_c_item(state)?;
    c_container::register_c_item_upgrade(state)?;
    c_container::register_c_container(state)?;
    c_currency::register_c_currency_info(state)?;
    c_equipment_set::register_c_equipment_set(state)?;
    c_currency::register_c_bank(state)?;
    c_spell::register_c_spell_surface(state)?;
    c_spell_book::register_c_spell_book(state)?;
    Ok(())
}
