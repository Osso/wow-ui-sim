//! Registry table initialisation and script-error reporter setup.

use crate::lua_api::methods::{registry_set, registry_table_or_create};
use rilua::LuaApiMut;
use std::cell::RefCell;
use std::rc::Rc;

use super::super::state::SimState;

/// Set up registry tables for event dispatch and taint fallback.
pub(super) fn init_registry_tables(
    lua: &mut rilua::Lua,
    state: &Rc<RefCell<SimState>>,
) -> crate::Result<()> {
    let lua_state = lua.state_mut();
    let _ = state;
    let _ = registry_table_or_create(lua_state, "__addon_names");
    let _ = registry_table_or_create(lua_state, "__addon_timing");
    let _ = registry_table_or_create(lua_state, "__event_individual");
    let _ = registry_table_or_create(lua_state, "__event_all");
    let _ = registry_table_or_create(lua_state, "__scripts");
    let _ = registry_table_or_create(lua_state, "__on_update_scripts");
    let _ = registry_table_or_create(lua_state, "__on_post_update_scripts");
    let _ = registry_table_or_create(lua_state, "__rilua_frame_envs");
    let _ = registry_table_or_create(lua_state, "__rilua_frame_fields");
    // Generated script wrappers capture this reporter through the ordinary
    // environment so secure chunks do not need `debug` in their own environment.
    register_script_error_reporter(lua_state);
    super::super::on_update::register(lua_state, state)
}

fn register_script_error_reporter(state: &mut rilua::vm::state::LuaState) {
    use rilua::vm::closure::{Closure, RustClosure};

    fn report_script_error(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
        let msg = match state.stack_get(state.base) {
            rilua::Val::Str(s) => state
                .gc
                .string_arena
                .get(s)
                .map(|ls| String::from_utf8_lossy(ls.data()).to_string())
                .unwrap_or_default(),
            other => format!("{other:?}"),
        };
        crate::lua_api::script_helpers::call_error_handler_state(state, &msg);
        Ok(0)
    }

    let closure = Closure::Rust(RustClosure::new(
        report_script_error,
        "__report_script_error",
    ));
    let closure_ref = state.gc.alloc_closure(closure);
    registry_set(
        state,
        "__report_script_error",
        rilua::Val::Function(closure_ref),
    );
}
