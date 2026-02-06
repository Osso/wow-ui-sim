//! UIDropDownMenu system implementation.
//!
//! WoW's dropdown menu system using global frames and functions.
//! This includes:
//! - DropDownList frames (DropDownList1, DropDownList2, DropDownList3)
//! - UIDropDownMenu_* functions for menu management
//! - Global constants for dropdown configuration

use crate::lua_api::frame::FrameHandle;
use crate::lua_api::SimState;
use crate::widget::{Frame, FrameStrata, WidgetType};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Get or create the `__frame_fields` table for a given frame ID.
fn get_or_create_frame_fields(lua: &Lua, frame_id: u64) -> Result<mlua::Table> {
    let fields_table: mlua::Table = lua
        .globals()
        .get::<mlua::Table>("__frame_fields")
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            lua.globals().set("__frame_fields", t.clone()).unwrap();
            t
        });
    let frame_fields = fields_table
        .get::<mlua::Table>(frame_id)
        .unwrap_or_else(|_| {
            let t = lua.create_table().unwrap();
            fields_table.set(frame_id, t.clone()).unwrap();
            t
        });
    Ok(frame_fields)
}

/// Register a frame widget as a Lua global, returning its ID.
fn create_and_register_global(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    frame: Frame,
    name: &str,
) -> Result<u64> {
    let id = frame.id;
    state.borrow_mut().widgets.register(frame);
    let ud = lua.create_userdata(FrameHandle {
        id,
        state: Rc::clone(state),
    })?;
    lua.globals().set(name, ud.clone())?;
    lua.globals()
        .set(format!("__frame_{}", id).as_str(), ud)?;
    Ok(id)
}

/// Register the UIDropDownMenu system.
pub fn register_dropdown_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_constants(lua)?;
    register_dropdown_list_frames(lua, &state)?;
    register_create_info(lua)?;
    register_initialize(lua, &state)?;
    register_add_button(lua, &state)?;
    register_width_and_text(lua, &state)?;
    register_selection_functions(lua)?;
    register_enable_disable(lua, &state)?;
    register_toggle_and_close(lua, &state)?;
    register_anchor_and_strata(lua, &state)?;
    register_separator_and_space(lua)?;
    register_query_functions(lua, &state)?;
    register_noop_functions(lua)?;
    Ok(())
}

/// Register global dropdown constants.
fn register_constants(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("UIDROPDOWNMENU_MAXBUTTONS", 1)?;
    globals.set("UIDROPDOWNMENU_MAXLEVELS", 3)?;
    globals.set("UIDROPDOWNMENU_BUTTON_HEIGHT", 16)?;
    globals.set("UIDROPDOWNMENU_BORDER_HEIGHT", 15)?;
    globals.set("UIDROPDOWNMENU_OPEN_MENU", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_INIT_MENU", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_MENU_LEVEL", 1)?;
    globals.set("UIDROPDOWNMENU_MENU_VALUE", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_SHOW_TIME", 2)?;
    globals.set("UIDROPDOWNMENU_DEFAULT_TEXT_HEIGHT", Value::Nil)?;
    globals.set("UIDROPDOWNMENU_DEFAULT_WIDTH_PADDING", 25)?;
    globals.set("OPEN_DROPDOWNMENUS", lua.create_table()?)?;
    Ok(())
}

/// Create DropDownList frames (3 levels, 8 buttons each with NormalText children).
fn register_dropdown_list_frames(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");

    for level in 1..=3 {
        let list_name = format!("DropDownList{}", level);
        let list_id = create_dropdown_list_frame(lua, state, &list_name, ui_parent_id)?;

        let fields = get_or_create_frame_fields(lua, list_id)?;
        fields.set("numButtons", 0)?;
        fields.set("maxWidth", 0)?;

        for btn_idx in 1..=8 {
            create_dropdown_button(lua, state, level, btn_idx, list_id)?;
        }
    }
    Ok(())
}

/// Create a single DropDownList frame.
fn create_dropdown_list_frame(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    name: &str,
    parent_id: Option<u64>,
) -> Result<u64> {
    let mut frame = Frame::new(WidgetType::Button, Some(name.to_string()), parent_id);
    frame.visible = false;
    frame.width = 180.0;
    frame.height = 32.0;
    frame.frame_strata = FrameStrata::FullscreenDialog;
    frame.clamped_to_screen = true;
    create_and_register_global(lua, state, frame, name)
}

/// Create a button and its NormalText child for a dropdown list.
fn create_dropdown_button(
    lua: &Lua,
    state: &Rc<RefCell<SimState>>,
    level: i32,
    btn_idx: i32,
    list_id: u64,
) -> Result<()> {
    let btn_name = format!("DropDownList{}Button{}", level, btn_idx);
    let mut btn_frame = Frame::new(WidgetType::Button, Some(btn_name.clone()), Some(list_id));
    btn_frame.visible = false;
    btn_frame.width = 100.0;
    btn_frame.height = 16.0;
    let btn_id = btn_frame.id;
    create_and_register_global(lua, state, btn_frame, &btn_name)?;

    let text_name = format!("DropDownList{}Button{}NormalText", level, btn_idx);
    let mut text_frame = Frame::new(WidgetType::FontString, Some(text_name.clone()), Some(btn_id));
    text_frame.visible = true;
    create_and_register_global(lua, state, text_frame, &text_name)?;
    Ok(())
}

/// Register UIDropDownMenu_CreateInfo.
fn register_create_info(lua: &Lua) -> Result<()> {
    let func = lua.create_function(|lua, ()| lua.create_table().map(Value::Table))?;
    lua.globals().set("UIDropDownMenu_CreateInfo", func)?;
    Ok(())
}

/// Register UIDropDownMenu_Initialize.
fn register_initialize(lua: &Lua, _state: &Rc<RefCell<SimState>>) -> Result<()> {
    let func = lua.create_function(
        |lua,
         (frame, init_fn, _display_mode, _level, _menu_list): (
            Value,
            Option<mlua::Function>,
            Option<String>,
            Option<i32>,
            Option<mlua::Table>,
        )| {
            if let Value::UserData(ud) = &frame {
                if let Ok(handle) = ud.borrow::<FrameHandle>() {
                    let fields = get_or_create_frame_fields(lua, handle.id)?;
                    if let Some(ref func) = init_fn {
                        fields.set("initialize", func.clone())?;
                    }
                }
            }

            lua.globals()
                .set("UIDROPDOWNMENU_INIT_MENU", frame.clone())?;

            if let Some(func) = init_fn {
                let level = _level.unwrap_or(1);
                let _ = func.call::<()>((frame, level, _menu_list));
            }

            Ok(())
        },
    )?;
    lua.globals().set("UIDropDownMenu_Initialize", func)?;
    Ok(())
}

/// Register UIDropDownMenu_AddButton.
fn register_add_button(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let state = Rc::clone(state);
    let func = lua.create_function(move |lua, (info, level): (mlua::Table, Option<i32>)| {
        let level = level.unwrap_or(1);
        let list_name = format!("DropDownList{}", level);

        let list_ud = match lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
            Ok(ud) => ud,
            Err(_) => return Ok(()),
        };
        let handle = match list_ud.borrow::<FrameHandle>() {
            Ok(h) => h,
            Err(_) => return Ok(()),
        };

        let list_fields = get_or_create_frame_fields(lua, handle.id)?;
        let num_buttons: i32 = list_fields.get("numButtons").unwrap_or(0);
        let new_index = num_buttons + 1;
        list_fields.set("numButtons", new_index)?;

        let btn_name = format!("DropDownList{}Button{}", level, new_index);
        if let Ok(btn_ud) = lua.globals().get::<mlua::AnyUserData>(btn_name.as_str()) {
            if let Ok(btn_handle) = btn_ud.borrow::<FrameHandle>() {
                let btn_fields = get_or_create_frame_fields(lua, btn_handle.id)?;
                for pair in info.pairs::<String, Value>() {
                    if let Ok((k, v)) = pair {
                        btn_fields.set(k, v)?;
                    }
                }

                if let Ok(text) = info.get::<mlua::String>("text") {
                    let mut s = state.borrow_mut();
                    if let Some(btn_frame) = s.widgets.get_mut(btn_handle.id) {
                        btn_frame.text = Some(text.to_string_lossy().to_string());
                        btn_frame.visible = true;
                    }
                }
            }
        }

        Ok(())
    })?;
    lua.globals().set("UIDropDownMenu_AddButton", func)?;
    Ok(())
}

/// Register UIDropDownMenu_SetWidth, SetText, GetText.
fn register_width_and_text(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let state_w = Rc::clone(state);
    let set_width = lua.create_function(
        move |_lua, (frame, width, _padding): (mlua::AnyUserData, f32, Option<f32>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_w.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.width = width;
                }
            }
            Ok(())
        },
    )?;
    lua.globals().set("UIDropDownMenu_SetWidth", set_width)?;

    let state_st = Rc::clone(state);
    let set_text = lua.create_function(
        move |_lua, (frame, text): (mlua::AnyUserData, Option<String>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_st.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.text = text;
                }
            }
            Ok(())
        },
    )?;
    lua.globals().set("UIDropDownMenu_SetText", set_text)?;

    let state_gt = Rc::clone(state);
    let get_text = lua.create_function(move |lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let s = state_gt.borrow();
            if let Some(f) = s.widgets.get(handle.id) {
                if let Some(ref text) = f.text {
                    return Ok(Value::String(lua.create_string(text)?));
                }
            }
        }
        Ok(Value::Nil)
    })?;
    lua.globals().set("UIDropDownMenu_GetText", get_text)?;
    Ok(())
}

/// Register SetSelectedID, GetSelectedID, SetSelectedValue, GetSelectedValue, SetSelectedName.
fn register_selection_functions(lua: &Lua) -> Result<()> {
    register_field_setter(lua, "UIDropDownMenu_SetSelectedID", "selectedID")?;
    register_field_getter(lua, "UIDropDownMenu_GetSelectedID", "selectedID")?;
    register_field_setter(lua, "UIDropDownMenu_SetSelectedValue", "selectedValue")?;
    register_field_getter(lua, "UIDropDownMenu_GetSelectedValue", "selectedValue")?;
    register_field_setter(lua, "UIDropDownMenu_SetSelectedName", "selectedName")?;
    Ok(())
}

/// Register a function that sets a field on a frame's __frame_fields entry.
fn register_field_setter(lua: &Lua, global_name: &str, field_name: &'static str) -> Result<()> {
    let func = lua.create_function(
        move |lua, (frame, value, _use_value): (mlua::AnyUserData, Value, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let fields = get_or_create_frame_fields(lua, handle.id)?;
                fields.set(field_name, value)?;
            }
            Ok(())
        },
    )?;
    lua.globals().set(global_name, func)?;
    Ok(())
}

/// Register a function that gets a field from a frame's __frame_fields entry.
fn register_field_getter(lua: &Lua, global_name: &str, field_name: &'static str) -> Result<()> {
    let func = lua.create_function(move |lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields") {
                if let Ok(frame_fields) = fields_table.get::<mlua::Table>(handle.id) {
                    if let Ok(value) = frame_fields.get::<Value>(field_name) {
                        return Ok(value);
                    }
                }
            }
        }
        Ok(Value::Nil)
    })?;
    lua.globals().set(global_name, func)?;
    Ok(())
}

/// Register UIDropDownMenu_EnableDropDown and DisableDropDown.
fn register_enable_disable(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let state_e = Rc::clone(state);
    let enable = lua.create_function(move |_lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let mut s = state_e.borrow_mut();
            if let Some(f) = s.widgets.get_mut(handle.id) {
                f.attributes.insert(
                    "__dropdown_enabled".to_string(),
                    crate::widget::AttributeValue::Boolean(true),
                );
            }
        }
        Ok(())
    })?;
    lua.globals().set("UIDropDownMenu_EnableDropDown", enable)?;

    let state_d = Rc::clone(state);
    let disable = lua.create_function(move |_lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let mut s = state_d.borrow_mut();
            if let Some(f) = s.widgets.get_mut(handle.id) {
                f.attributes.insert(
                    "__dropdown_enabled".to_string(),
                    crate::widget::AttributeValue::Boolean(false),
                );
            }
        }
        Ok(())
    })?;
    lua.globals()
        .set("UIDropDownMenu_DisableDropDown", disable)?;
    Ok(())
}

/// Register ToggleDropDownMenu and CloseDropDownMenus.
fn register_toggle_and_close(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let state_t = Rc::clone(state);
    let toggle = lua.create_function(
        move |lua,
              (level, _value, dropdown_frame, _anchor_name, _x_offset, _y_offset, _menu_list, _button, _auto_hide_delay, _display_mode): (
                  Option<i32>,
                  Option<Value>,
                  Option<Value>,
                  Option<String>,
                  Option<f32>,
                  Option<f32>,
                  Option<mlua::Table>,
                  Option<Value>,
                  Option<f32>,
                  Option<String>,
              )| {
            let level = level.unwrap_or(1);
            let list_name = format!("DropDownList{}", level);

            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
                if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                    let mut s = state_t.borrow_mut();
                    if let Some(f) = s.widgets.get_mut(handle.id) {
                        f.visible = !f.visible;
                    }
                }
            }

            lua.globals()
                .set("UIDROPDOWNMENU_OPEN_MENU", dropdown_frame)?;
            Ok(())
        },
    )?;
    lua.globals().set("ToggleDropDownMenu", toggle)?;

    let state_c = Rc::clone(state);
    let close = lua.create_function(move |lua, level: Option<i32>| {
        let start_level = level.unwrap_or(1);
        for lvl in start_level..=3 {
            let list_name = format!("DropDownList{}", lvl);
            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
                if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                    let mut s = state_c.borrow_mut();
                    if let Some(f) = s.widgets.get_mut(handle.id) {
                        f.visible = false;
                    }
                }
            }
        }
        lua.globals()
            .set("UIDROPDOWNMENU_OPEN_MENU", Value::Nil)?;
        Ok(())
    })?;
    lua.globals().set("CloseDropDownMenus", close)?;
    Ok(())
}

/// Register UIDropDownMenu_SetAnchor and SetFrameStrata.
fn register_anchor_and_strata(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let set_anchor = lua.create_function(
        |lua,
         (dropdown, x_offset, y_offset, point, relative_to, relative_point): (
            Value,
            f32,
            f32,
            String,
            Option<Value>,
            Option<String>,
        )| {
            if let Value::UserData(ud) = &dropdown {
                if let Ok(handle) = ud.borrow::<FrameHandle>() {
                    let fields = get_or_create_frame_fields(lua, handle.id)?;
                    fields.set("xOffset", x_offset)?;
                    fields.set("yOffset", y_offset)?;
                    fields.set("point", point)?;
                    fields.set("relativeTo", relative_to)?;
                    fields.set("relativePoint", relative_point)?;
                }
            }
            Ok(())
        },
    )?;
    lua.globals().set("UIDropDownMenu_SetAnchor", set_anchor)?;

    let state_s = Rc::clone(state);
    let set_strata = lua.create_function(
        move |_lua, (frame, strata): (mlua::AnyUserData, String)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_s.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.frame_strata = match strata.to_uppercase().as_str() {
                        "WORLD" | "BACKGROUND" => FrameStrata::Background,
                        "LOW" => FrameStrata::Low,
                        "MEDIUM" => FrameStrata::Medium,
                        "HIGH" => FrameStrata::High,
                        "DIALOG" => FrameStrata::Dialog,
                        "FULLSCREEN" => FrameStrata::Fullscreen,
                        "FULLSCREEN_DIALOG" => FrameStrata::FullscreenDialog,
                        "TOOLTIP" => FrameStrata::Tooltip,
                        _ => FrameStrata::Medium,
                    };
                }
            }
            Ok(())
        },
    )?;
    lua.globals()
        .set("UIDropDownMenu_SetFrameStrata", set_strata)?;
    Ok(())
}

/// Register UIDropDownMenu_AddSeparator and AddSpace.
fn register_separator_and_space(lua: &Lua) -> Result<()> {
    let add_separator = lua.create_function(|lua, level: Option<i32>| {
        let info = lua.create_table()?;
        info.set("hasArrow", false)?;
        info.set("isTitle", true)?;
        info.set("isUninteractable", true)?;
        info.set("notCheckable", true)?;
        info.set("iconOnly", true)?;

        let add_button: mlua::Function = lua.globals().get("UIDropDownMenu_AddButton")?;
        add_button.call::<()>((info, level))?;
        Ok(())
    })?;
    lua.globals()
        .set("UIDropDownMenu_AddSeparator", add_separator)?;

    let add_space = lua.create_function(|lua, level: Option<i32>| {
        let info = lua.create_table()?;
        info.set("hasArrow", false)?;
        info.set("isTitle", true)?;
        info.set("isUninteractable", true)?;
        info.set("notCheckable", true)?;

        let add_button: mlua::Function = lua.globals().get("UIDropDownMenu_AddButton")?;
        add_button.call::<()>((info, level))?;
        Ok(())
    })?;
    lua.globals().set("UIDropDownMenu_AddSpace", add_space)?;
    Ok(())
}

/// Register UIDropDownMenu_GetCurrentDropDown and IsOpen.
fn register_query_functions(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let get_current = lua.create_function(|lua, ()| {
        lua.globals().get::<Value>("UIDROPDOWNMENU_OPEN_MENU")
    })?;
    lua.globals()
        .set("UIDropDownMenu_GetCurrentDropDown", get_current)?;

    let state_io = Rc::clone(state);
    let is_open = lua.create_function(move |lua, frame: Option<Value>| {
        let ud = match frame {
            Some(Value::UserData(ud)) => ud,
            _ => return Ok(false),
        };
        let handle = match ud.borrow::<FrameHandle>() {
            Ok(h) => h,
            Err(_) => return Ok(false),
        };
        let target_id = handle.id;

        let open_menu = match lua.globals().get::<Value>("UIDROPDOWNMENU_OPEN_MENU") {
            Ok(Value::UserData(ud)) => ud,
            _ => return Ok(false),
        };
        let open_handle = match open_menu.borrow::<FrameHandle>() {
            Ok(h) => h,
            Err(_) => return Ok(false),
        };
        if open_handle.id != target_id {
            return Ok(false);
        }

        let list_ud = match lua.globals().get::<mlua::AnyUserData>("DropDownList1") {
            Ok(ud) => ud,
            Err(_) => return Ok(false),
        };
        let list_handle = match list_ud.borrow::<FrameHandle>() {
            Ok(h) => h,
            Err(_) => return Ok(false),
        };
        let s = state_io.borrow();
        Ok(s.widgets.get(list_handle.id).map_or(false, |f| f.visible))
    })?;
    lua.globals().set("UIDropDownMenu_IsOpen", is_open)?;
    Ok(())
}

/// Register SetInitializeFunction and no-op functions (Refresh, JustifyText, HandleGlobalMouseEvent).
fn register_noop_functions(lua: &Lua) -> Result<()> {
    let set_init_func = lua.create_function(
        |lua, (frame, init_fn): (mlua::AnyUserData, Option<mlua::Function>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let fields = get_or_create_frame_fields(lua, handle.id)?;
                match init_fn {
                    Some(func) => fields.set("initialize", func)?,
                    None => fields.set("initialize", Value::Nil)?,
                }
            }
            Ok(())
        },
    )?;
    lua.globals()
        .set("UIDropDownMenu_SetInitializeFunction", set_init_func)?;

    let refresh = lua.create_function(
        |_lua, (_frame, _use_value, _level): (Value, Option<bool>, Option<i32>)| Ok(()),
    )?;
    lua.globals().set("UIDropDownMenu_Refresh", refresh)?;

    let justify = lua
        .create_function(|_lua, (_frame, _justify): (Value, String)| Ok(()))?;
    lua.globals()
        .set("UIDropDownMenu_JustifyText", justify)?;

    let handle_mouse =
        lua.create_function(|_lua, (_button, _event): (String, String)| Ok(()))?;
    lua.globals()
        .set("UIDropDownMenu_HandleGlobalMouseEvent", handle_mouse)?;
    Ok(())
}
