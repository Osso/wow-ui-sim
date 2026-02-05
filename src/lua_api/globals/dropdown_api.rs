//! UIDropDownMenu system implementation.
//!
//! WoW's dropdown menu system using global frames and functions.
//! This includes:
//! - DropDownList frames (DropDownList1, DropDownList2, DropDownList3)
//! - UIDropDownMenu_* functions for menu management
//! - Global constants for dropdown configuration

use crate::lua_api::frame::FrameHandle;
use crate::widget::{Frame, FrameStrata, WidgetType};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

use crate::lua_api::SimState;

/// Register the UIDropDownMenu system.
pub fn register_dropdown_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // Global constants
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

    // Create DropDownList frames (DropDownList1, DropDownList2, DropDownList3)
    for level in 1..=3 {
        let list_name = format!("DropDownList{}", level);
        let ui_parent_id = state.borrow().widgets.get_id_by_name("UIParent");
        let mut list_frame = Frame::new(
            WidgetType::Button,
            Some(list_name.clone()),
            ui_parent_id,
        );
        list_frame.visible = false;
        list_frame.width = 180.0;
        list_frame.height = 32.0;
        list_frame.frame_strata = FrameStrata::FullscreenDialog;
        list_frame.clamped_to_screen = true;
        let list_id = list_frame.id;
        state.borrow_mut().widgets.register(list_frame);

        let handle = FrameHandle {
            id: list_id,
            state: Rc::clone(&state),
        };
        let list_ud = lua.create_userdata(handle)?;

        // Set numButtons and maxWidth fields
        {
            let fields_table: mlua::Table =
                globals.get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
                    let t = lua.create_table().unwrap();
                    globals.set("__frame_fields", t.clone()).unwrap();
                    t
                });
            let frame_fields = lua.create_table()?;
            frame_fields.set("numButtons", 0)?;
            frame_fields.set("maxWidth", 0)?;
            fields_table.set(list_id, frame_fields)?;
        }

        globals.set(list_name.as_str(), list_ud.clone())?;
        let frame_key = format!("__frame_{}", list_id);
        globals.set(frame_key.as_str(), list_ud)?;

        // Create buttons for each dropdown list (DropDownList1Button1, etc.)
        for btn_idx in 1..=8 {
            let btn_name = format!("DropDownList{}Button{}", level, btn_idx);
            let mut btn_frame = Frame::new(
                WidgetType::Button,
                Some(btn_name.clone()),
                Some(list_id),
            );
            btn_frame.visible = false;
            btn_frame.width = 100.0;
            btn_frame.height = 16.0;
            let btn_id = btn_frame.id;
            state.borrow_mut().widgets.register(btn_frame);

            let btn_handle = FrameHandle {
                id: btn_id,
                state: Rc::clone(&state),
            };
            let btn_ud = lua.create_userdata(btn_handle)?;
            globals.set(btn_name.as_str(), btn_ud.clone())?;
            let btn_frame_key = format!("__frame_{}", btn_id);
            globals.set(btn_frame_key.as_str(), btn_ud)?;

            // Create child elements for buttons (NormalText, Icon, etc.)
            let text_name = format!("DropDownList{}Button{}NormalText", level, btn_idx);
            let mut text_frame = Frame::new(
                WidgetType::FontString,
                Some(text_name.clone()),
                Some(btn_id),
            );
            text_frame.visible = true;
            let text_id = text_frame.id;
            state.borrow_mut().widgets.register(text_frame);

            let text_handle = FrameHandle {
                id: text_id,
                state: Rc::clone(&state),
            };
            let text_ud = lua.create_userdata(text_handle)?;
            globals.set(text_name.as_str(), text_ud)?;
        }
    }

    // UIDropDownMenu_CreateInfo() - returns empty table for info structure
    let create_info = lua.create_function(|lua, ()| {
        lua.create_table().map(Value::Table)
    })?;
    globals.set("UIDropDownMenu_CreateInfo", create_info)?;

    // UIDropDownMenu_Initialize(frame, initFunction, displayMode, level, menuList)
    let _state_init = Rc::clone(&state);
    let init_func = lua.create_function(
        move |lua,
              (frame, init_fn, _display_mode, _level, _menu_list): (
                  Value,
                  Option<mlua::Function>,
                  Option<String>,
                  Option<i32>,
                  Option<mlua::Table>,
              )| {
            // Store the initialize function on the frame
            if let Value::UserData(ud) = &frame {
                if let Ok(handle) = ud.borrow::<FrameHandle>() {
                    let frame_id = handle.id;
                    let fields_table: mlua::Table = lua
                        .globals()
                        .get::<mlua::Table>("__frame_fields")
                        .unwrap_or_else(|_| {
                            let t = lua.create_table().unwrap();
                            lua.globals().set("__frame_fields", t.clone()).unwrap();
                            t
                        });
                    let frame_fields: mlua::Table =
                        fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                            let t = lua.create_table().unwrap();
                            fields_table.set(frame_id, t.clone()).unwrap();
                            t
                        });
                    if let Some(ref func) = init_fn {
                        frame_fields.set("initialize", func.clone())?;
                    }
                }
            }

            // Set UIDROPDOWNMENU_INIT_MENU
            lua.globals().set("UIDROPDOWNMENU_INIT_MENU", frame.clone())?;

            // Call the init function if provided
            if let Some(func) = init_fn {
                let level = _level.unwrap_or(1);
                let _ = func.call::<()>((frame.clone(), level, _menu_list));
            }

            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_Initialize", init_func)?;

    // UIDropDownMenu_AddButton(info, level) - adds a button to the menu
    let state_add = Rc::clone(&state);
    let add_button = lua.create_function(move |lua, (info, level): (mlua::Table, Option<i32>)| {
        let level = level.unwrap_or(1);
        let list_name = format!("DropDownList{}", level);

        // Get the list frame and increment numButtons
        if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
            if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                let num_buttons: i32 = frame_fields.get("numButtons").unwrap_or(0);
                let new_index = num_buttons + 1;
                frame_fields.set("numButtons", new_index)?;

                // Get the button and configure it
                let btn_name = format!("DropDownList{}Button{}", level, new_index);
                if let Ok(btn_ud) = lua.globals().get::<mlua::AnyUserData>(btn_name.as_str()) {
                    if let Ok(btn_handle) = btn_ud.borrow::<FrameHandle>() {
                        // Store button properties from info table
                        let btn_id = btn_handle.id;
                        let btn_fields: mlua::Table =
                            fields_table.get::<mlua::Table>(btn_id).unwrap_or_else(|_| {
                                let t = lua.create_table().unwrap();
                                fields_table.set(btn_id, t.clone()).unwrap();
                                t
                            });

                        // Copy info properties to button fields
                        for pair in info.pairs::<String, Value>() {
                            if let Ok((k, v)) = pair {
                                btn_fields.set(k, v)?;
                            }
                        }

                        // Set the text if provided
                        if let Ok(text) = info.get::<mlua::String>("text") {
                            let mut s = state_add.borrow_mut();
                            if let Some(btn_frame) = s.widgets.get_mut(btn_id) {
                                btn_frame.text = Some(text.to_string_lossy().to_string());
                                btn_frame.visible = true;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    })?;
    globals.set("UIDropDownMenu_AddButton", add_button)?;

    // UIDropDownMenu_SetWidth(frame, width, padding)
    let state_width = Rc::clone(&state);
    let set_width = lua.create_function(
        move |_lua, (frame, width, _padding): (mlua::AnyUserData, f32, Option<f32>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_width.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.width = width;
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetWidth", set_width)?;

    // UIDropDownMenu_SetText(frame, text)
    let state_text = Rc::clone(&state);
    let set_text = lua.create_function(
        move |_lua, (frame, text): (mlua::AnyUserData, Option<String>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_text.borrow_mut();
                if let Some(f) = s.widgets.get_mut(handle.id) {
                    f.text = text;
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetText", set_text)?;

    // UIDropDownMenu_GetText(frame) -> string
    let state_get_text = Rc::clone(&state);
    let get_text_fn = lua.create_function(move |lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let s = state_get_text.borrow();
            if let Some(f) = s.widgets.get(handle.id) {
                if let Some(ref text) = f.text {
                    let lua_str = lua.create_string(text)?;
                    return Ok(Value::String(lua_str));
                }
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("UIDropDownMenu_GetText", get_text_fn)?;

    // UIDropDownMenu_SetSelectedID(frame, id, useValue)
    let set_selected_id = lua.create_function(
        |lua, (frame, id, _use_value): (mlua::AnyUserData, i32, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                frame_fields.set("selectedID", id)?;
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetSelectedID", set_selected_id)?;

    // UIDropDownMenu_GetSelectedID(frame) -> number
    let get_selected_id = lua.create_function(|lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let frame_id = handle.id;
            if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields") {
                if let Ok(frame_fields) = fields_table.get::<mlua::Table>(frame_id) {
                    if let Ok(id) = frame_fields.get::<i32>("selectedID") {
                        return Ok(Value::Integer(id as i64));
                    }
                }
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("UIDropDownMenu_GetSelectedID", get_selected_id)?;

    // UIDropDownMenu_SetSelectedValue(frame, value, useValue)
    let set_selected_value = lua.create_function(
        |lua, (frame, value, _use_value): (mlua::AnyUserData, Value, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                frame_fields.set("selectedValue", value)?;
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetSelectedValue", set_selected_value)?;

    // UIDropDownMenu_GetSelectedValue(frame) -> value
    let get_selected_value = lua.create_function(|lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let frame_id = handle.id;
            if let Ok(fields_table) = lua.globals().get::<mlua::Table>("__frame_fields") {
                if let Ok(frame_fields) = fields_table.get::<mlua::Table>(frame_id) {
                    if let Ok(value) = frame_fields.get::<Value>("selectedValue") {
                        return Ok(value);
                    }
                }
            }
        }
        Ok(Value::Nil)
    })?;
    globals.set("UIDropDownMenu_GetSelectedValue", get_selected_value)?;

    // UIDropDownMenu_SetSelectedName(frame, name, useValue)
    let set_selected_name = lua.create_function(
        |lua, (frame, name, _use_value): (mlua::AnyUserData, String, Option<bool>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                frame_fields.set("selectedName", name)?;
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetSelectedName", set_selected_name)?;

    // UIDropDownMenu_EnableDropDown(frame)
    let state_enable = Rc::clone(&state);
    let enable_dropdown = lua.create_function(move |_lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let mut s = state_enable.borrow_mut();
            if let Some(f) = s.widgets.get_mut(handle.id) {
                f.attributes.insert(
                    "__dropdown_enabled".to_string(),
                    crate::widget::AttributeValue::Boolean(true),
                );
            }
        }
        Ok(())
    })?;
    globals.set("UIDropDownMenu_EnableDropDown", enable_dropdown)?;

    // UIDropDownMenu_DisableDropDown(frame)
    let state_disable = Rc::clone(&state);
    let disable_dropdown = lua.create_function(move |_lua, frame: mlua::AnyUserData| {
        if let Ok(handle) = frame.borrow::<FrameHandle>() {
            let mut s = state_disable.borrow_mut();
            if let Some(f) = s.widgets.get_mut(handle.id) {
                f.attributes.insert(
                    "__dropdown_enabled".to_string(),
                    crate::widget::AttributeValue::Boolean(false),
                );
            }
        }
        Ok(())
    })?;
    globals.set("UIDropDownMenu_DisableDropDown", disable_dropdown)?;

    // UIDropDownMenu_Refresh(frame, useValue, dropdownLevel)
    let refresh_dropdown = lua.create_function(
        |_lua, (_frame, _use_value, _level): (Value, Option<bool>, Option<i32>)| {
            // No-op in simulation - would normally re-run initialize function
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_Refresh", refresh_dropdown)?;

    // UIDropDownMenu_SetAnchor(dropdown, xOffset, yOffset, point, relativeTo, relativePoint)
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
                    let frame_id = handle.id;
                    let fields_table: mlua::Table = lua
                        .globals()
                        .get::<mlua::Table>("__frame_fields")
                        .unwrap_or_else(|_| {
                            let t = lua.create_table().unwrap();
                            lua.globals().set("__frame_fields", t.clone()).unwrap();
                            t
                        });
                    let frame_fields: mlua::Table =
                        fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                            let t = lua.create_table().unwrap();
                            fields_table.set(frame_id, t.clone()).unwrap();
                            t
                        });
                    frame_fields.set("xOffset", x_offset)?;
                    frame_fields.set("yOffset", y_offset)?;
                    frame_fields.set("point", point)?;
                    frame_fields.set("relativeTo", relative_to)?;
                    frame_fields.set("relativePoint", relative_point)?;
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetAnchor", set_anchor)?;

    // ToggleDropDownMenu(level, value, dropDownFrame, anchorName, xOffset, yOffset, menuList, button, autoHideDelay, displayMode)
    let state_toggle = Rc::clone(&state);
    let toggle_dropdown = lua.create_function(
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

            // Toggle visibility of the dropdown list
            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
                if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                    let mut s = state_toggle.borrow_mut();
                    if let Some(f) = s.widgets.get_mut(handle.id) {
                        f.visible = !f.visible;
                    }
                }
            }

            // Set UIDROPDOWNMENU_OPEN_MENU
            lua.globals().set("UIDROPDOWNMENU_OPEN_MENU", dropdown_frame)?;

            Ok(())
        },
    )?;
    globals.set("ToggleDropDownMenu", toggle_dropdown)?;

    // CloseDropDownMenus(level) - close menus at specified level and above
    let state_close = Rc::clone(&state);
    let close_dropdown = lua.create_function(move |lua, level: Option<i32>| {
        let start_level = level.unwrap_or(1);

        for lvl in start_level..=3 {
            let list_name = format!("DropDownList{}", lvl);
            if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>(list_name.as_str()) {
                if let Ok(handle) = list_ud.borrow::<FrameHandle>() {
                    let mut s = state_close.borrow_mut();
                    if let Some(f) = s.widgets.get_mut(handle.id) {
                        f.visible = false;
                    }
                }
            }
        }

        // Clear UIDROPDOWNMENU_OPEN_MENU
        lua.globals().set("UIDROPDOWNMENU_OPEN_MENU", Value::Nil)?;

        Ok(())
    })?;
    globals.set("CloseDropDownMenus", close_dropdown)?;

    // UIDropDownMenu_HandleGlobalMouseEvent(button, event) - handles clicks outside to close
    let handle_mouse_event = lua.create_function(|_lua, (_button, _event): (String, String)| {
        // No-op in simulation
        Ok(())
    })?;
    globals.set("UIDropDownMenu_HandleGlobalMouseEvent", handle_mouse_event)?;

    // UIDropDownMenu_SetInitializeFunction(frame, initFunction)
    let set_init_func = lua.create_function(
        |lua, (frame, init_fn): (mlua::AnyUserData, Option<mlua::Function>)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let frame_id = handle.id;
                let fields_table: mlua::Table = lua
                    .globals()
                    .get::<mlua::Table>("__frame_fields")
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__frame_fields", t.clone()).unwrap();
                        t
                    });
                let frame_fields: mlua::Table =
                    fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        fields_table.set(frame_id, t.clone()).unwrap();
                        t
                    });
                if let Some(func) = init_fn {
                    frame_fields.set("initialize", func)?;
                } else {
                    frame_fields.set("initialize", Value::Nil)?;
                }
            }
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_SetInitializeFunction", set_init_func)?;

    // UIDropDownMenu_JustifyText(frame, justification)
    let justify_text = lua.create_function(
        |_lua, (_frame, _justify): (Value, String)| {
            // No-op in simulation
            Ok(())
        },
    )?;
    globals.set("UIDropDownMenu_JustifyText", justify_text)?;

    // UIDropDownMenu_SetFrameStrata(frame, strata)
    let state_strata = Rc::clone(&state);
    let set_frame_strata = lua.create_function(
        move |_lua, (frame, strata): (mlua::AnyUserData, String)| {
            if let Ok(handle) = frame.borrow::<FrameHandle>() {
                let mut s = state_strata.borrow_mut();
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
    globals.set("UIDropDownMenu_SetFrameStrata", set_frame_strata)?;

    // UIDropDownMenu_AddSeparator(level)
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
    globals.set("UIDropDownMenu_AddSeparator", add_separator)?;

    // UIDropDownMenu_AddSpace(level)
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
    globals.set("UIDropDownMenu_AddSpace", add_space)?;

    // UIDropDownMenu_GetCurrentDropDown() -> frame
    let get_current_dropdown = lua.create_function(|lua, ()| {
        lua.globals().get::<Value>("UIDROPDOWNMENU_OPEN_MENU")
    })?;
    globals.set("UIDropDownMenu_GetCurrentDropDown", get_current_dropdown)?;

    // UIDropDownMenu_IsOpen(frame) -> boolean
    let state_is_open = Rc::clone(&state);
    let is_open = lua.create_function(move |lua, frame: Option<Value>| {
        if let Some(Value::UserData(ud)) = frame {
            if let Ok(handle) = ud.borrow::<FrameHandle>() {
                let target_id = handle.id;
                // Check if this frame is the open menu
                if let Ok(open_menu) = lua.globals().get::<Value>("UIDROPDOWNMENU_OPEN_MENU") {
                    if let Value::UserData(open_ud) = open_menu {
                        if let Ok(open_handle) = open_ud.borrow::<FrameHandle>() {
                            if open_handle.id == target_id {
                                // Check if DropDownList1 is visible
                                if let Ok(list_ud) = lua.globals().get::<mlua::AnyUserData>("DropDownList1") {
                                    if let Ok(list_handle) = list_ud.borrow::<FrameHandle>() {
                                        let s = state_is_open.borrow();
                                        if let Some(f) = s.widgets.get(list_handle.id) {
                                            return Ok(f.visible);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(false)
    })?;
    globals.set("UIDropDownMenu_IsOpen", is_open)?;

    Ok(())
}
