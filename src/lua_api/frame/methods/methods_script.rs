//! Script handler methods: SetScript, GetScript, HookScript, HasScript, etc.

use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use crate::lua_api::script_helpers::{
    get_or_create_hooks_table, get_scripts_table, remove_script, set_script,
};
use mlua::{LightUserData, Lua, Value};

/// Add script handler methods to the shared methods table.
pub fn add_script_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_set_script_methods(lua, methods)?;
    add_get_script_method(lua, methods)?;
    add_hook_and_wrap_methods(lua, methods)?;
    add_clear_scripts_method(lua, methods)?;
    add_has_script_method(lua, methods)?;
    Ok(())
}

/// SetScript(handler, func) and SetOnClickHandler(func)
fn add_set_script_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetScript", lua.create_function(|lua, (ud, handler, func): (LightUserData, String, Value)| {
        let id = lud_to_id(ud);
        let handler_type = crate::event::ScriptHandler::from_str(&handler);

        if let Some(h) = handler_type {
            if let Value::Function(f) = func {
                set_script(lua, id, &handler, f);

                let state_rc = get_sim_state(lua);
                let mut state = state_rc.borrow_mut();
                state.scripts.set(id, h, 1);

                if h == crate::event::ScriptHandler::OnUpdate || h == crate::event::ScriptHandler::OnPostUpdate {
                    state.on_update_frames.insert(id);
                    state.visible_on_update_cache = None;
                }
            } else {
                // nil func: remove the handler
                remove_script(lua, id, &handler);

                let state_rc = get_sim_state(lua);
                let mut state = state_rc.borrow_mut();
                state.scripts.remove(id, h);

                if h == crate::event::ScriptHandler::OnUpdate || h == crate::event::ScriptHandler::OnPostUpdate {
                    state.on_update_frames.remove(&id);
                    state.visible_on_update_cache = None;
                }
            }
        }
        Ok(())
    })?)?;

    // SetOnClickHandler(func) - WoW 10.0+ convenience for setting OnClick
    methods.set("SetOnClickHandler", lua.create_function(|lua, (ud, func): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        if let Value::Function(f) = func {
            set_script(lua, id, "OnClick", f);

            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            state
                .scripts
                .set(id, crate::event::ScriptHandler::OnClick, 1);
        }
        Ok(())
    })?)?;

    Ok(())
}

/// GetScript(handler) - retrieve a stored script handler function.
fn add_get_script_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetScript", lua.create_function(|lua, (ud, handler): (LightUserData, String)| {
        let id = lud_to_id(ud);
        match crate::lua_api::script_helpers::get_script(lua, id, &handler) {
            Some(f) => Ok(Value::Function(f)),
            None => Ok(Value::Nil),
        }
    })?)?;

    Ok(())
}

/// HookScript, WrapScript, UnwrapScript - script chaining methods.
fn add_hook_and_wrap_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("HookScript", lua.create_function(|lua, (ud, handler, func): (LightUserData, String, Value)| {
        let id = lud_to_id(ud);
        if let Value::Function(f) = func {
            let hooks_table = get_or_create_hooks_table(lua);

            let frame_key = format!("{}_{}", id, handler);
            let hooks_array: mlua::Table = hooks_table
                .get::<mlua::Table>(frame_key.as_str())
                .unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    hooks_table.set(frame_key.as_str(), t.clone()).unwrap();
                    t
                });
            let len = hooks_array.len().unwrap_or(0);
            hooks_array.set(len + 1, f)?;
        }
        Ok(())
    })?)?;

    // WrapScript - stub for secure script wrapping
    methods.set("WrapScript", lua.create_function(|_, (_ud, _target, _script, _pre_body): (LightUserData, Value, String, String)| {
        Ok(())
    })?)?;

    // UnwrapScript - stub for removing script wrapping
    methods.set("UnwrapScript", lua.create_function(|_, (_ud, _target, _script): (LightUserData, Value, String)| {
        Ok(())
    })?)?;

    Ok(())
}

/// Remove matching keys from a Lua table by prefix.
fn remove_keys_with_prefix(table: &mlua::Table, prefix: &str) {
    let keys: Vec<String> = table
        .pairs::<String, Value>()
        .filter_map(|pair| {
            if let Ok((k, _)) = pair
                && k.starts_with(prefix) {
                    return Some(k);
                }
            None
        })
        .collect();
    for key in keys {
        let _ = table.set(key.as_str(), Value::Nil);
    }
}

/// ClearScripts() - remove all script handlers for this frame.
fn add_clear_scripts_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("ClearScripts", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let prefix = format!("{}_", id);

        if let Some(table) = get_scripts_table(lua) {
            remove_keys_with_prefix(&table, &prefix);
        }

        // Also clear from hooks table
        let hooks_table: Option<mlua::Table> =
            lua.named_registry_value("__script_hooks").ok();
        if let Some(table) = hooks_table {
            remove_keys_with_prefix(&table, &prefix);
        }

        // Clear script entries in state
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        state.scripts.remove_all(id);
        if state.on_update_frames.remove(&id) {
            state.visible_on_update_cache = None;
        }

        Ok(())
    })?)?;

    Ok(())
}

/// HasScript(scriptType) - check if frame supports a script handler type.
fn add_has_script_method(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("HasScript", lua.create_function(|_, (_ud, script_type): (LightUserData, String)| {
        let common_scripts = [
            "OnClick",
            "OnEnter",
            "OnLeave",
            "OnShow",
            "OnHide",
            "OnMouseDown",
            "OnMouseUp",
            "OnMouseWheel",
            "OnDragStart",
            "OnDragStop",
            "OnUpdate",
            "OnEvent",
            "OnLoad",
            "OnSizeChanged",
            "OnAttributeChanged",
            "OnEnable",
            "OnDisable",
            "OnTooltipSetItem",
            "OnTooltipSetUnit",
            "OnTooltipSetSpell",
            "OnTooltipCleared",
            "PostClick",
            "PreClick",
            "OnValueChanged",
            "OnMinMaxChanged",
            "OnEditFocusGained",
            "OnEditFocusLost",
            "OnTextChanged",
            "OnEnterPressed",
            "OnEscapePressed",
            "OnKeyDown",
            "OnKeyUp",
            "OnChar",
            "OnTabPressed",
            "OnSpacePressed",
            "OnReceiveDrag",
            "OnPostUpdate",
            "OnPostShow",
            "OnPostHide",
            "OnPostClick",
        ];
        Ok(common_scripts
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&script_type)))
    })?)?;

    Ok(())
}
