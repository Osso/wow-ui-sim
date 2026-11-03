//! Garrison combat spell information from pinned generated data.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_string, create_table, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        let namespace = super::ensure_namespace(state, "C_Garrison")?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetCombatLogSpellInfo",
            get_combat_log_spell_info,
        )?;
    }
    Ok(())
}

fn get_combat_log_spell_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let records = sim_data::garrison_auto_spell::GARR_AUTO_SPELLS;
    let Ok(index) = records.binary_search_by_key(&id, |record| record.0) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let (_, name, cooldown, duration, school, icon, preview, tutorial, thorns) = records[index];
    let result = create_table(state);
    let name = create_string(state, name);
    let description = create_string(state, "");
    table_set(state, result, "autoCombatSpellID", Val::Num(id as f64));
    table_set(state, result, "name", name);
    table_set(state, result, "description", description);
    for (key, value) in [
        ("cooldown", cooldown),
        ("duration", duration),
        ("schoolMask", school),
        ("icon", icon),
        ("previewMask", preview),
        ("spellTutorialFlag", tutorial),
    ] {
        table_set(state, result, key, Val::Num(value as f64));
    }
    table_set(state, result, "hasThornsEffect", Val::Bool(thorns));
    state.push(result);
    Ok(1)
}
