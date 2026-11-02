pub(super) use crate::c_api::item_spell::current_item_upgrade_location;
pub(crate) use crate::c_api::item_spell::{
    item_class_name, parse_item_guid, parse_item_id_key, parse_prefixed_id, push_item_info,
    spell_link_for_id,
};

use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register_item_and_spell_surfaces(state: &mut LuaState) -> LuaResult<()> {
    crate::c_api::item_spell::register_item_and_spell_surfaces(state)
}

pub(crate) fn item_link_for_id(item_id: u32) -> Option<String> {
    crate::c_api::item_spell::item_link_for_id(item_id)
}
