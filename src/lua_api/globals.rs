//! Global WoW API functions.

use super::SimState;
use crate::widget::{Frame, WidgetType};
use mlua::{Lua, MetaMethod, Result, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all global WoW API functions.
pub fn register_globals(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // CreateFrame(frameType, name, parent, template, id)
    let state_clone = Rc::clone(&state);
    let create_frame = lua.create_function(move |lua, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();

        let frame_type: String = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Frame".to_string());

        let name: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        let parent_id: Option<u64> = args_iter.next().and_then(|v| {
            if let Value::UserData(ud) = v {
                ud.borrow::<FrameHandle>().ok().map(|h| h.id)
            } else {
                None
            }
        });

        // Get parent ID (default to UIParent)
        let parent_id = parent_id.or_else(|| {
            let state = state_clone.borrow();
            state.widgets.get_id_by_name("UIParent")
        });

        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame = Frame::new(widget_type, name.clone(), parent_id);
        let frame_id = frame.id;

        let mut state = state_clone.borrow_mut();
        state.widgets.register(frame);

        if let Some(pid) = parent_id {
            state.widgets.add_child(pid, frame_id);
        }

        // Create userdata handle
        let handle = FrameHandle {
            id: frame_id,
            state: Rc::clone(&state_clone),
        };

        let ud = lua.create_userdata(handle)?;

        // Store reference in globals if named
        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        // Store reference for event dispatch
        let frame_key = format!("__frame_{}", frame_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(ud)
    })?;
    globals.set("CreateFrame", create_frame)?;

    // UIParent reference
    let ui_parent_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("UIParent").unwrap()
    };
    let ui_parent = lua.create_userdata(FrameHandle {
        id: ui_parent_id,
        state: Rc::clone(&state),
    })?;
    globals.set("UIParent", ui_parent)?;

    // print() - already exists in Lua but we can customize if needed

    // strsplit(delimiter, str, limit) - WoW string utility
    let strsplit = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let delimiter = args
            .first()
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| " ".to_string());

        let input = args
            .get(1)
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let limit = args
            .get(2)
            .and_then(|v| {
                if let Value::Integer(n) = v {
                    Some(*n as usize)
                } else if let Value::Number(n) = v {
                    Some(*n as usize)
                } else {
                    None
                }
            });

        let parts: Vec<&str> = if let Some(limit) = limit {
            input.splitn(limit, &delimiter).collect()
        } else {
            input.split(&delimiter).collect()
        };

        let mut result = mlua::MultiValue::new();
        for part in parts {
            result.push_back(Value::String(lua.create_string(part)?));
        }
        Ok(result)
    })?;
    globals.set("strsplit", strsplit)?;

    // wipe(table) - Clear a table in place
    let wipe = lua.create_function(|_, table: mlua::Table| {
        // Get all keys first to avoid modification during iteration
        let keys: Vec<Value> = table
            .pairs::<Value, Value>()
            .filter_map(|r| r.ok().map(|(k, _)| k))
            .collect();

        for key in keys {
            table.set(key, Value::Nil)?;
        }
        Ok(table)
    })?;
    globals.set("wipe", wipe)?;

    // tinsert - alias for table.insert
    let tinsert = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_insert: mlua::Function = lua.globals().get::<mlua::Table>("table")?.get("insert")?;
        table_insert.call::<()>(args)?;
        Ok(())
    })?;
    globals.set("tinsert", tinsert)?;

    // tremove - alias for table.remove
    let tremove = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_remove: mlua::Function = lua.globals().get::<mlua::Table>("table")?.get("remove")?;
        table_remove.call::<Value>(args)
    })?;
    globals.set("tremove", tremove)?;

    // hooksecurefunc(name, hook) or hooksecurefunc(table, name, hook)
    let hooksecurefunc = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let (table, name, hook) = if args.len() == 2 {
            // hooksecurefunc("FuncName", hookFunc)
            let name = if let Value::String(s) = &args[0] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[1].clone();
            (lua.globals(), name, hook)
        } else if args.len() >= 3 {
            // hooksecurefunc(someTable, "FuncName", hookFunc)
            let table = if let Value::Table(t) = &args[0] {
                t.clone()
            } else {
                lua.globals()
            };
            let name = if let Value::String(s) = &args[1] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[2].clone();
            (table, name, hook)
        } else {
            return Ok(());
        };

        // Get the original function
        let original: Value = table.get::<Value>(name.as_str())?;

        if let (Value::Function(orig_fn), Value::Function(hook_fn)) = (original, hook) {
            // Create a wrapper that calls original then hook
            let wrapper = lua.create_function(move |_, args: mlua::MultiValue| {
                // Call original
                let result = orig_fn.call::<mlua::MultiValue>(args.clone())?;
                // Call hook (ignoring its result)
                let _ = hook_fn.call::<mlua::MultiValue>(args);
                Ok(result)
            })?;

            table.set(name.as_str(), wrapper)?;
        }

        Ok(())
    })?;
    globals.set("hooksecurefunc", hooksecurefunc)?;

    // GetBuildInfo() - Return mock game version
    let get_build_info = lua.create_function(|lua, ()| {
        // Return: version, build, date, tocversion, localizedVersion, buildType
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.0")?),  // version
            Value::String(lua.create_string("99999")?),   // build
            Value::String(lua.create_string("Jan 1 2025")?), // date
            Value::Integer(110000),                        // tocversion
            Value::String(lua.create_string("11.0.0")?),  // localizedVersion
            Value::String(lua.create_string("Release")?), // buildType
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    // SlashCmdList table
    let slash_cmd_list = lua.create_table()?;
    globals.set("SlashCmdList", slash_cmd_list)?;

    // C_Timer namespace
    let c_timer = lua.create_table()?;
    // C_Timer.After(seconds, callback) - simplified version that calls immediately for testing
    let c_timer_after = lua.create_function(|_, (_seconds, callback): (f64, mlua::Function)| {
        // In a real implementation, this would schedule for later
        // For testing, we just store it (or call it immediately)
        // For now, just acknowledge it exists
        let _ = callback; // Would need an event loop to actually call this
        Ok(())
    })?;
    c_timer.set("After", c_timer_after)?;
    globals.set("C_Timer", c_timer)?;

    Ok(())
}

/// Userdata handle to a frame (passed to Lua).
#[derive(Clone)]
pub struct FrameHandle {
    pub id: u64,
    pub state: Rc<RefCell<SimState>>,
}

impl UserData for FrameHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Support custom field access via __index/__newindex
        // This allows addons to do: frame.customField = value

        methods.add_meta_function(MetaMethod::Index, |lua: &Lua, (ud, key): (mlua::AnyUserData, String)| {
            // Try to get from the custom fields table
            let frame_id: u64 = ud.borrow::<FrameHandle>()?.id;
            let fields_table: Option<mlua::Table> = lua.globals().get("__frame_fields").ok();

            if let Some(table) = fields_table {
                let frame_fields: Option<mlua::Table> = table.get::<mlua::Table>(frame_id).ok();
                if let Some(fields) = frame_fields {
                    let value: Value = fields.get::<Value>(key.as_str()).unwrap_or(Value::Nil);
                    if value != Value::Nil {
                        return Ok(value);
                    }
                }
            }

            // Not found in custom fields, return nil (methods are handled separately by mlua)
            Ok(Value::Nil)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua: &Lua, (ud, key, value): (mlua::AnyUserData, String, Value)| {
            let frame_id: u64 = ud.borrow::<FrameHandle>()?.id;

            // Get or create the fields table
            let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.globals().set("__frame_fields", t.clone()).unwrap();
                t
            });

            // Get or create the frame's field table
            let frame_fields: mlua::Table = fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                fields_table.set(frame_id, t.clone()).unwrap();
                t
            });

            frame_fields.set(key, value)?;
            Ok(())
        });
        // GetName()
        methods.add_method("GetName", |_, this, ()| {
            let state = this.state.borrow();
            let name = state
                .widgets
                .get(this.id)
                .and_then(|f| f.name.clone())
                .unwrap_or_default();
            Ok(name)
        });

        // GetWidth()
        methods.add_method("GetWidth", |_, this, ()| {
            let state = this.state.borrow();
            let width = state.widgets.get(this.id).map(|f| f.width).unwrap_or(0.0);
            Ok(width)
        });

        // GetHeight()
        methods.add_method("GetHeight", |_, this, ()| {
            let state = this.state.borrow();
            let height = state.widgets.get(this.id).map(|f| f.height).unwrap_or(0.0);
            Ok(height)
        });

        // SetSize(width, height)
        methods.add_method("SetSize", |_, this, (width, height): (f32, f32)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.set_size(width, height);
            }
            Ok(())
        });

        // SetWidth(width)
        methods.add_method("SetWidth", |_, this, width: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.width = width;
            }
            Ok(())
        });

        // SetHeight(height)
        methods.add_method("SetHeight", |_, this, height: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.height = height;
            }
            Ok(())
        });

        // SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
        methods.add_method("SetPoint", |_, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let point_str = args
                .first()
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "CENTER".to_string());

            let point =
                crate::widget::AnchorPoint::from_str(&point_str).unwrap_or_default();

            // Parse the variable arguments
            let (relative_to, relative_point, x_ofs, y_ofs) = match args.len() {
                1 => (None, point, 0.0, 0.0),
                2 | 3 => {
                    // SetPoint("CENTER", x, y) or SetPoint("CENTER", relativeTo)
                    let x = args.get(1).and_then(|v| {
                        if let Value::Number(n) = v {
                            Some(*n as f32)
                        } else {
                            None
                        }
                    });
                    let y = args.get(2).and_then(|v| {
                        if let Value::Number(n) = v {
                            Some(*n as f32)
                        } else {
                            None
                        }
                    });
                    if let (Some(x), Some(y)) = (x, y) {
                        (None, point, x, y)
                    } else {
                        (None, point, 0.0, 0.0)
                    }
                }
                _ => {
                    // Full form: SetPoint(point, relativeTo, relativePoint, x, y)
                    let rel_point_str = args.get(2).and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    });
                    let rel_point = rel_point_str
                        .and_then(|s| crate::widget::AnchorPoint::from_str(&s))
                        .unwrap_or(point);
                    let x = args
                        .get(3)
                        .and_then(|v| {
                            if let Value::Number(n) = v {
                                Some(*n as f32)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0.0);
                    let y = args
                        .get(4)
                        .and_then(|v| {
                            if let Value::Number(n) = v {
                                Some(*n as f32)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0.0);
                    (None, rel_point, x, y)
                }
            };

            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
            }
            Ok(())
        });

        // ClearAllPoints()
        methods.add_method("ClearAllPoints", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.clear_all_points();
            }
            Ok(())
        });

        // Show()
        methods.add_method("Show", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = true;
            }
            Ok(())
        });

        // Hide()
        methods.add_method("Hide", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = false;
            }
            Ok(())
        });

        // IsVisible() / IsShown()
        methods.add_method("IsVisible", |_, this, ()| {
            let state = this.state.borrow();
            let visible = state.widgets.get(this.id).map(|f| f.visible).unwrap_or(false);
            Ok(visible)
        });

        methods.add_method("IsShown", |_, this, ()| {
            let state = this.state.borrow();
            let visible = state.widgets.get(this.id).map(|f| f.visible).unwrap_or(false);
            Ok(visible)
        });

        // RegisterEvent(event)
        methods.add_method("RegisterEvent", |_, this, event: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.register_event(&event);
            }
            Ok(())
        });

        // UnregisterEvent(event)
        methods.add_method("UnregisterEvent", |_, this, event: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.unregister_event(&event);
            }
            Ok(())
        });

        // SetScript(handler, func)
        methods.add_method("SetScript", |lua, this, (handler, func): (String, Value)| {
            let handler_type = crate::event::ScriptHandler::from_str(&handler);

            if let (Some(h), Value::Function(f)) = (handler_type, func) {
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

        // GetParent()
        methods.add_method("GetParent", |lua, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(parent_id) = frame.parent_id {
                    let handle = FrameHandle {
                        id: parent_id,
                        state: Rc::clone(&this.state),
                    };
                    return Ok(Value::UserData(lua.create_userdata(handle)?));
                }
            }
            Ok(Value::Nil)
        });

        // GetObjectType()
        methods.add_method("GetObjectType", |_, this, ()| {
            let state = this.state.borrow();
            let obj_type = state
                .widgets
                .get(this.id)
                .map(|f| f.widget_type.as_str())
                .unwrap_or("Frame");
            Ok(obj_type.to_string())
        });
    }
}
