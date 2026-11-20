//! INFERRED simulator compatibility; native SetMapForQuestLog is not established (R88f.F.7).
use super::LuaTableRef;
use crate::c_api::global_val;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state_mut, call_function_state};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn uses_rust_quest_log_selection() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

pub(super) fn register(state: &mut LuaState, table_ref: LuaTableRef) -> LuaResult<()> {
    if uses_rust_quest_log_selection() {
        table_set_rust_fn_static(state, table_ref, "SetMapForQuestLog", set_map_for_quest_log)?;
    }
    Ok(())
}

fn set_map_for_quest_log(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(map_id) = stack_val(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?.quest_log_selected_map_id = Some(map_id);
    for frame_name in ["WorldMapFrame", "QuestMapFrame"] {
        update_selected_map(state, frame_name, map_id)?;
    }
    Ok(0)
}

fn update_selected_map(state: &mut LuaState, frame_name: &str, map_id: f64) -> LuaResult<()> {
    let frame = global_val(state, frame_name);
    if matches!(frame, Val::Nil | Val::Bool(false)) {
        return Ok(());
    }
    let key = state.gc.intern_string(b"SetMapID");
    let method = state.gettable(frame, Val::Str(key))?;
    if matches!(method, Val::Function(_)) {
        call_function_state(state, method, &[frame, Val::Num(map_id)])?;
    }
    Ok(())
}
