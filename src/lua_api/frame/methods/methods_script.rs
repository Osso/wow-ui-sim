//! Script handler methods: SetScript, GetScript, HookScript, HasScript, etc.

use super::FrameHandle;
use mlua::{UserDataMethods, Value};

/// Add script handler methods to FrameHandle UserData.
pub fn add_script_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetScript(handler, func)
    methods.add_method("SetScript", |lua, this, (handler, func): (String, Value)| {
        let handler_type = crate::event::ScriptHandler::from_str(&handler);

        if let Some(h) = handler_type {
            if let Value::Function(f) = func {
                // Store function in a global Lua table for later retrieval
                let scripts_table: mlua::Table =
                    lua.globals().get("__scripts").unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__scripts", t.clone()).unwrap();
                        t
                    });

                let frame_key = format!("{}_{}", this.id, handler);
                scripts_table.set(frame_key.as_str(), f)?;

                // Mark that this widget has this handler
                let mut state = this.state.borrow_mut();
                state.scripts.set(this.id, h, 1); // Just mark it exists

                // Track OnUpdate registrations for efficient dispatch
                if h == crate::event::ScriptHandler::OnUpdate {
                    state.on_update_frames.insert(this.id);
                }
            } else {
                // nil func: remove the handler
                let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();
                if let Some(table) = scripts_table {
                    let frame_key = format!("{}_{}", this.id, handler);
                    table.set(frame_key.as_str(), mlua::Value::Nil)?;
                }

                let mut state = this.state.borrow_mut();
                state.scripts.remove(this.id, h);

                if h == crate::event::ScriptHandler::OnUpdate {
                    state.on_update_frames.remove(&this.id);
                }
            }
        }
        Ok(())
    });

    // GetScript(handler)
    methods.add_method("GetScript", |lua, this, handler: String| {
        let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();

        if let Some(table) = scripts_table {
            let frame_key = format!("{}_{}", this.id, handler);
            let func: Value = table.get(frame_key.as_str()).unwrap_or(Value::Nil);
            Ok(func)
        } else {
            Ok(Value::Nil)
        }
    });

    // SetOnClickHandler(func) - WoW 10.0+ method for setting OnClick handler (used by Edit Mode)
    methods.add_method("SetOnClickHandler", |lua, this, func: Value| {
        if let Value::Function(f) = func {
            let scripts_table: mlua::Table = lua.globals().get("__scripts").unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.globals().set("__scripts", t.clone()).unwrap();
                t
            });

            let frame_key = format!("{}_OnClick", this.id);
            scripts_table.set(frame_key.as_str(), f)?;

            let mut state = this.state.borrow_mut();
            state
                .scripts
                .set(this.id, crate::event::ScriptHandler::OnClick, 1);
        }
        Ok(())
    });

    // HookScript(handler, func) - Hook into existing script handler
    methods.add_method("HookScript", |lua, this, (handler, func): (String, Value)| {
        if let Value::Function(f) = func {
            // Store hook in a global table
            let hooks_table: mlua::Table =
                lua.globals()
                    .get("__script_hooks")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__script_hooks", t.clone()).unwrap();
                        t
                    });

            let frame_key = format!("{}_{}", this.id, handler);
            // Get existing hooks array or create new
            let hooks_array: mlua::Table = hooks_table
                .get::<mlua::Table>(frame_key.as_str())
                .unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    hooks_table.set(frame_key.as_str(), t.clone()).unwrap();
                    t
                });
            // Append the new hook
            let len = hooks_array.len().unwrap_or(0);
            hooks_array.set(len + 1, f)?;
        }
        Ok(())
    });

    // WrapScript(frame, scriptType, preBody, postBody) - Wraps a secure script handler
    methods.add_method(
        "WrapScript",
        |_, _this, (_target, _script, _pre_body): (mlua::Value, String, String)| {
            // Stub for secure script wrapping - not implemented in simulator
            Ok(())
        },
    );

    // UnwrapScript(frame, scriptType) - Removes script wrapping
    methods.add_method(
        "UnwrapScript",
        |_, _this, (_target, _script): (mlua::Value, String)| Ok(()),
    );

    // HasScript(scriptType) - Check if frame supports a script handler
    methods.add_method("HasScript", |_, _this, script_type: String| {
        // Most frames support common script types
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
        ];
        Ok(common_scripts
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&script_type)))
    });
}
