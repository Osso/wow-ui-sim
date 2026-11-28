//! Forever `C_TradeSkillUI.GetProfessionInfoBySkillLineID`.
//!
//! Forever 70707 `TradeSkillUIDocumentation.lua:488-501` declares one non-nil
//! `ProfessionInfo` return. The Camelot professions book reads it for each rank
//! bar (Blizzard_ProfessionsBook/Camelot/Blizzard_ProfessionsBook.lua:30). The row
//! comes from the shared profession catalogue; an unknown skill line yields the
//! catalogue's empty row (`professionID = 0`), INFERRED from the non-nil return.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::missing_surface::professions::profession_info_table;
use crate::lua_api::globals::profession_data;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetProfessionInfoBySkillLineID",
        profession_info_by_skill_line_id,
    )
}

fn profession_info_by_skill_line_id(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    let profession = profession_data::get_profession_by_skill_line_id(skill_line_id);
    let table = profession_info_table(state, profession);
    state.push(table);
    Ok(1)
}
