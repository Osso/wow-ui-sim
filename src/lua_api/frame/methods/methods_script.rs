//! Script handler methods: SetScript, GetScript, HookScript, HasScript, etc.

use super::FrameHandle;
use crate::lua_api::script_helpers::{
    get_or_create_hooks_table, get_scripts_table, remove_script, set_script,
};
use mlua::{UserDataMethods, Value};

/// Add script handler methods to FrameHandle UserData.
pub fn add_script_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_set_script_methods(methods);
    add_get_script_method(methods);
    add_hook_and_wrap_methods(methods);
    add_clear_scripts_method(methods);
    add_has_script_method(methods);
}

/// SetScript(handler, func) and SetOnClickHandler(func)
fn add_set_script_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetScript", |lua, this, (handler, func): (String, Value)| {
        let handler_type = crate::event::ScriptHandler::from_str(&handler);

        if let Some(h) = handler_type {
            if let Value::Function(f) = func {
                set_script(lua, this.id, &handler, f);

                let mut state = this.state.borrow_mut();
                state.scripts.set(this.id, h, 1);

                if h == crate::event::ScriptHandler::OnUpdate || h == crate::event::ScriptHandler::OnPostUpdate {
                    state.on_update_frames.insert(this.id);
                    state.visible_on_update_cache = None;
                }
            } else {
                // nil func: remove the handler
                remove_script(lua, this.id, &handler);

                let mut state = this.state.borrow_mut();
                state.scripts.remove(this.id, h);

                if h == crate::event::ScriptHandler::OnUpdate || h == crate::event::ScriptHandler::OnPostUpdate {
                    state.on_update_frames.remove(&this.id);
                    state.visible_on_update_cache = None;
                }
            }
        }
        Ok(())
    });

    // SetOnClickHandler(func) - WoW 10.0+ convenience for setting OnClick
    methods.add_method("SetOnClickHandler", |lua, this, func: Value| {
        if let Value::Function(f) = func {
            set_script(lua, this.id, "OnClick", f);

            let mut state = this.state.borrow_mut();
            state
                .scripts
                .set(this.id, crate::event::ScriptHandler::OnClick, 1);
        }
        Ok(())
    });
}

/// GetScript(handler) - retrieve a stored script handler function.
fn add_get_script_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("GetScript", |lua, this, handler: String| {
        match crate::lua_api::script_helpers::get_script(lua, this.id, &handler) {
            Some(f) => Ok(Value::Function(f)),
            None => Ok(Value::Nil),
        }
    });
}

/// HookScript, WrapScript, UnwrapScript - script chaining methods.
fn add_hook_and_wrap_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("HookScript", |lua, this, (handler, func): (String, Value)| {
        if let Value::Function(f) = func {
            let hooks_table = get_or_create_hooks_table(lua);

            let frame_key = format!("{}_{}", this.id, handler);
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
    });

    // WrapScript - stub for secure script wrapping
    methods.add_method(
        "WrapScript",
        |_, _this, (_target, _script, _pre_body): (mlua::Value, String, String)| Ok(()),
    );

    // UnwrapScript - stub for removing script wrapping
    methods.add_method(
        "UnwrapScript",
        |_, _this, (_target, _script): (mlua::Value, String)| Ok(()),
    );
}

/// ClearScripts() - remove all script handlers for this frame.
fn add_clear_scripts_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("ClearScripts", |lua, this, ()| {
        if let Some(table) = get_scripts_table(lua) {
            // Iterate all keys and remove those starting with "{id}_"
            let prefix = format!("{}_", this.id);
            let keys: Vec<String> = table
                .pairs::<String, Value>()
                .filter_map(|pair| {
                    if let Ok((k, _)) = pair
                        && k.starts_with(&prefix) {
                            return Some(k);
                        }
                    None
                })
                .collect();
            for key in keys {
                let _ = table.set(key.as_str(), Value::Nil);
            }
        }

        // Also clear from hooks table
        let hooks_table: Option<mlua::Table> =
            lua.named_registry_value("__script_hooks").ok();
        if let Some(table) = hooks_table {
            let prefix = format!("{}_", this.id);
            let keys: Vec<String> = table
                .pairs::<String, Value>()
                .filter_map(|pair| {
                    if let Ok((k, _)) = pair
                        && k.starts_with(&prefix) {
                            return Some(k);
                        }
                    None
                })
                .collect();
            for key in keys {
                let _ = table.set(key.as_str(), Value::Nil);
            }
        }

        // Clear script entries in state
        let mut state = this.state.borrow_mut();
        state.scripts.remove_all(this.id);
        if state.on_update_frames.remove(&this.id) {
            state.visible_on_update_cache = None;
        }

        Ok(())
    });
}

/// HasScript(scriptType) - check if frame supports a script handler type.
fn add_has_script_method<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("HasScript", |_, _this, script_type: String| {
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
    });
}
