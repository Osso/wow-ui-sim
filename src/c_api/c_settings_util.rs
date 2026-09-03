//! `C_SettingsUtil` settings-panel actions.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{call_function_state, table_get_static};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_settings_util_surface(state: &mut LuaState) -> LuaResult<()> {
    let settings_util = ensure_namespace(state, "C_SettingsUtil")?;
    table_set_rust_fn_static(
        state,
        settings_util,
        "OpenSettingsPanel",
        open_settings_panel,
    )
}

fn open_settings_panel(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = Option::<f64>::from_stack(state, 1)?.unwrap_or_default();
    let scroll_target = stack_val(state, 2);
    let settings_panel = crate::c_api::global_val(state, "SettingsPanel");
    let settings_panel_mixin = crate::c_api::global_val(state, "SettingsPanelMixin");
    let open_to_category = table_get_static(state, settings_panel_mixin, "OpenToCategory");

    if matches!(settings_panel, Val::Nil) || !matches!(open_to_category, Val::Function(_)) {
        return Ok(0);
    }

    call_function_state(
        state,
        open_to_category,
        &[settings_panel, Val::Num(category_id), scroll_target],
    )?;
    Ok(0)
}
