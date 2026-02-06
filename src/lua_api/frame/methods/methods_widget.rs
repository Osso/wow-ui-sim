//! Widget-specific methods: GameTooltip, EditBox, Slider, StatusBar, CheckButton,
//! Cooldown, ScrollFrame, Model, ColorSelect, dragging/moving, ScrollBox.

use super::methods_helpers::get_mixin_override;
use super::FrameHandle;
use crate::lua_api::tooltip::TooltipLine;
use crate::widget::{AttributeValue, Frame, WidgetType};
use mlua::{Result, UserDataMethods, Value};
use std::rc::Rc;

/// Add widget-specific methods to FrameHandle UserData.
pub fn add_widget_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_tooltip_methods(methods);
    add_message_frame_methods(methods);
    add_editbox_methods(methods);
    add_slider_methods(methods);
    add_statusbar_methods(methods);
    add_checkbutton_methods(methods);
    add_cooldown_methods(methods);
    add_scrollframe_methods(methods);
    add_model_methods(methods);
    add_model_scene_methods(methods);
    add_colorselect_methods(methods);
    add_drag_methods(methods);
    add_scrollbox_methods(methods);
    add_simplehtml_methods(methods);
}

fn add_tooltip_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetOwner(owner, anchor, x, y) - Set the tooltip's owner and anchor
    methods.add_method("SetOwner", |lua, this, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();
        let owner_val = args_iter.next().unwrap_or(Value::Nil);
        let anchor: String = match args_iter.next() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            _ => "ANCHOR_NONE".to_string(),
        };

        let owner_id = match &owner_val {
            Value::UserData(ud) => ud.borrow::<FrameHandle>().ok().map(|h| h.id),
            _ => None,
        };

        // Clear lines and set owner
        {
            let mut state = this.state.borrow_mut();
            if let Some(td) = state.tooltips.get_mut(&this.id) {
                td.lines.clear();
                td.owner_id = owner_id;
                td.anchor_type = anchor;
            }
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = true;
            }
        }

        // Fire OnTooltipCleared
        fire_tooltip_script(lua, this.id, "OnTooltipCleared")?;
        Ok(())
    });

    // ClearLines() - Clear all text lines from the tooltip
    methods.add_method("ClearLines", |lua, this, ()| {
        {
            let mut state = this.state.borrow_mut();
            if let Some(td) = state.tooltips.get_mut(&this.id) {
                td.lines.clear();
            }
        }
        fire_tooltip_script(lua, this.id, "OnTooltipCleared")?;
        Ok(())
    });

    // AddLine(text, r, g, b, wrap) - Add a line of text
    methods.add_method("AddLine", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let text = match it.next() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Integer(n)) => n.to_string(),
            _ => return Ok(()),
        };
        let r = val_to_f32(it.next(), 1.0);
        let g = val_to_f32(it.next(), 1.0);
        let b = val_to_f32(it.next(), 1.0);
        let wrap = match it.next() {
            Some(Value::Boolean(w)) => w,
            _ => false,
        };

        let mut state = this.state.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            td.lines.push(TooltipLine {
                left_text: text,
                left_color: (r, g, b),
                right_text: None,
                right_color: (1.0, 1.0, 1.0),
                wrap,
            });
        }
        Ok(())
    });

    // AddDoubleLine(leftText, rightText, lR, lG, lB, rR, rG, rB) - Add two-column line
    methods.add_method("AddDoubleLine", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let left = match it.next() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Integer(n)) => n.to_string(),
            _ => return Ok(()),
        };
        let right = match it.next() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Integer(n)) => n.to_string(),
            _ => String::new(),
        };
        let lr = val_to_f32(it.next(), 1.0);
        let lg = val_to_f32(it.next(), 1.0);
        let lb = val_to_f32(it.next(), 1.0);
        let rr = val_to_f32(it.next(), 1.0);
        let rg = val_to_f32(it.next(), 1.0);
        let rb = val_to_f32(it.next(), 1.0);

        let mut state = this.state.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            td.lines.push(TooltipLine {
                left_text: left,
                left_color: (lr, lg, lb),
                right_text: Some(right),
                right_color: (rr, rg, rb),
                wrap: false,
            });
        }
        Ok(())
    });

    // SetSpellByID(spellID) - Set tooltip to show spell info (no game data)
    methods.add_method("SetSpellByID", |_, _this, _spell_id: i32| Ok(()));

    // SetItemByID(itemID) - Set tooltip to show item info (no game data)
    methods.add_method("SetItemByID", |_, _this, _item_id: i32| Ok(()));

    // SetHyperlink(link) - Set tooltip from a hyperlink (no game data)
    methods.add_method("SetHyperlink", |_, _this, _link: String| Ok(()));

    // SetUnitBuff/Debuff/Aura stubs (no game data)
    methods.add_method("SetUnitBuff", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetUnitDebuff", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetUnitAura", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method(
        "SetUnitBuffByAuraInstanceID",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );
    methods.add_method(
        "SetUnitDebuffByAuraInstanceID",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );

    // NumLines() - Get number of lines in tooltip
    methods.add_method("NumLines", |_, this, ()| {
        let state = this.state.borrow();
        let count = state
            .tooltips
            .get(&this.id)
            .map(|td| td.lines.len())
            .unwrap_or(0);
        Ok(count as i32)
    });

    // GetUnit() - Get the unit this tooltip is showing info for (no game data)
    methods.add_method(
        "GetUnit",
        |_, _this, ()| -> Result<(Option<String>, Option<String>)> {
            Ok((None, None))
        },
    );

    // GetSpell() - Get the spell this tooltip is showing info for (no game data)
    methods.add_method(
        "GetSpell",
        |_, _this, ()| -> Result<(Option<String>, Option<i32>)> {
            Ok((None, None))
        },
    );

    // GetItem() - Get the item this tooltip is showing info for (no game data)
    methods.add_method(
        "GetItem",
        |_, _this, ()| -> Result<(Option<String>, Option<String>)> {
            Ok((None, None))
        },
    );

    // SetMinimumWidth(width) / GetMinimumWidth()
    methods.add_method("SetMinimumWidth", |_, this, width: f32| {
        let mut state = this.state.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            td.min_width = width;
        }
        Ok(())
    });
    methods.add_method("GetMinimumWidth", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state
            .tooltips
            .get(&this.id)
            .map(|td| td.min_width)
            .unwrap_or(0.0))
    });

    // SetPadding(padding) / GetPadding()
    // These check for Lua mixin overrides first (e.g., ScrollBoxBaseMixin:GetPadding)
    // because Rust add_method methods shadow mixin methods stored in __frame_fields.
    methods.add_method("SetPadding", |lua, this, args: mlua::MultiValue| {
        if let Some((func, ud)) = get_mixin_override(lua, this.id, "SetPadding") {
            let mut call_args = vec![ud];
            call_args.extend(args);
            return func.call::<()>(mlua::MultiValue::from_iter(call_args));
        }
        let padding = args
            .into_iter()
            .next()
            .and_then(|v| match v {
                Value::Number(n) => Some(n as f32),
                Value::Integer(n) => Some(n as f32),
                _ => None,
            })
            .unwrap_or(0.0);
        let mut state = this.state.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            td.padding = padding;
        }
        Ok(())
    });
    methods.add_method("GetPadding", |lua, this, ()| -> Result<Value> {
        if let Some((func, ud)) = get_mixin_override(lua, this.id, "GetPadding") {
            return func.call::<Value>(ud);
        }
        let state = this.state.borrow();
        Ok(Value::Number(
            state
                .tooltips
                .get(&this.id)
                .map(|td| td.padding as f64)
                .unwrap_or(0.0),
        ))
    });

    // AddTexture(texture) - Add a texture to the tooltip (stub)
    methods.add_method("AddTexture", |_, _this, _texture: String| Ok(()));

    // SetText(text, r, g, b, wrap) - Clear and set first line (tooltip), or set frame text
    // For SimpleHTML: strips HTML tags before storing plain text
    methods.add_method_mut("SetText", |_, this, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();
        if let Some(Value::String(text)) = args_iter.next() {
            let text_str = text.to_string_lossy().to_string();
            let r = val_to_f32(args_iter.next(), 1.0);
            let g = val_to_f32(args_iter.next(), 1.0);
            let b = val_to_f32(args_iter.next(), 1.0);
            // 5th arg can be alpha (number) or wrap (bool)
            let wrap = match args_iter.next() {
                Some(Value::Boolean(w)) => w,
                _ => false,
            };

            let mut state = this.state.borrow_mut();
            // If this is a tooltip, use tooltip data
            if let Some(td) = state.tooltips.get_mut(&this.id) {
                td.lines.clear();
                td.lines.push(TooltipLine {
                    left_text: text_str.clone(),
                    left_color: (r, g, b),
                    right_text: None,
                    right_color: (1.0, 1.0, 1.0),
                    wrap,
                });
            }
            // For SimpleHTML, strip HTML tags before storing
            let store_text = if state.simple_htmls.contains_key(&this.id) {
                strip_html_tags(&text_str)
            } else {
                text_str
            };
            // Always set frame.text too
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.text = Some(store_text);
            }
        }
        Ok(())
    });

    // AppendText(text) - Append to last line's left_text
    methods.add_method("AppendText", |_, this, text: String| {
        let mut state = this.state.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            if let Some(last) = td.lines.last_mut() {
                last.left_text.push_str(&text);
            }
        }
        Ok(())
    });

    // IsOwned(frame) - Check if tooltip is owned by a frame
    methods.add_method("IsOwned", |_, this, frame: Value| {
        let check_id = match &frame {
            Value::UserData(ud) => ud.borrow::<FrameHandle>().ok().map(|h| h.id),
            _ => None,
        };
        let state = this.state.borrow();
        let owned = state.tooltips.get(&this.id).is_some_and(|td| {
            td.owner_id.is_some() && td.owner_id == check_id
        });
        Ok(owned)
    });

    // GetOwner() - Return the owner frame
    methods.add_method("GetOwner", |lua, this, ()| {
        let owner_id = {
            let state = this.state.borrow();
            state.tooltips.get(&this.id).and_then(|td| td.owner_id)
        };
        match owner_id {
            Some(id) => {
                let key = format!("__frame_{}", id);
                let val: Value = lua.globals().get(key.as_str())?;
                Ok(val)
            }
            None => Ok(Value::Nil),
        }
    });

    // GetAnchorType() - Return the anchor type string
    methods.add_method("GetAnchorType", |_, this, ()| {
        let state = this.state.borrow();
        let anchor = state
            .tooltips
            .get(&this.id)
            .map(|td| td.anchor_type.clone())
            .unwrap_or_else(|| "ANCHOR_NONE".to_string());
        Ok(anchor)
    });

    // FadeOut() - Hide tooltip, clear owner
    methods.add_method("FadeOut", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.visible = false;
        }
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            td.owner_id = None;
        }
        Ok(())
    });
}

fn add_message_frame_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // AddMessage(text, r, g, b, messageID, holdTime) - Add message to a MessageFrame
    methods.add_method("AddMessage", |_, this, args: mlua::MultiValue| {
        add_message_impl(this, args);
        Ok(())
    });

    // AddMsg(text, ...) - Alias for AddMessage (used by some addons like DBM)
    methods.add_method("AddMsg", |_, this, args: mlua::MultiValue| {
        add_message_impl(this, args);
        Ok(())
    });

    // BackFillMessage(text, r, g, b, ...) - Add message to back of history
    methods.add_method("BackFillMessage", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let text = match args_vec.first() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            _ => return Ok(()),
        };
        let r = val_to_f32_ref(args_vec.get(1), 1.0);
        let g = val_to_f32_ref(args_vec.get(2), 1.0);
        let b = val_to_f32_ref(args_vec.get(3), 1.0);
        let a = val_to_f32_ref(args_vec.get(4), 1.0);
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.messages.insert(0, crate::lua_api::message_frame::Message {
            text, r, g, b, a, message_id: None,
        });
        if data.messages.len() > data.max_lines {
            data.messages.pop();
        }
        Ok(())
    });

    // Clear() - Clear all messages (overrides tooltip Clear for MessageFrame)
    methods.add_method("Clear", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.message_frames.get_mut(&this.id) {
            data.messages.clear();
            data.scroll_offset = 0;
        }
        Ok(())
    });

    // GetNumMessages()
    methods.add_method("GetNumMessages", |_, this, ()| {
        let state = this.state.borrow();
        let count = state.message_frames.get(&this.id)
            .map(|d| d.messages.len())
            .unwrap_or(0);
        Ok(count as i32)
    });

    // SetMaxLines(maxLines)
    methods.add_method_mut("SetMaxLines", |_, this, max_lines: i32| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.max_lines = max_lines.max(1) as usize;
        data.messages.truncate(data.max_lines);
        Ok(())
    });

    // GetMaxLines()
    methods.add_method("GetMaxLines", |_, this, ()| {
        let state = this.state.borrow();
        let max = state.message_frames.get(&this.id)
            .map(|d| d.max_lines)
            .unwrap_or(120);
        Ok(max as i32)
    });

    // SetFading(fading) - override the stub in methods_core
    methods.add_method("SetFading", |_, this, fading: bool| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fading = fading;
        Ok(())
    });

    // GetFading()
    methods.add_method("GetFading", |_, this, ()| {
        let state = this.state.borrow();
        let fading = state.message_frames.get(&this.id)
            .map(|d| d.fading)
            .unwrap_or(true);
        Ok(fading)
    });

    // SetTimeVisible(secs) - override the stub in methods_core
    methods.add_method("SetTimeVisible", |_, this, secs: f64| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.time_visible = secs;
        Ok(())
    });

    // GetTimeVisible()
    methods.add_method("GetTimeVisible", |_, this, ()| {
        let state = this.state.borrow();
        let secs = state.message_frames.get(&this.id)
            .map(|d| d.time_visible)
            .unwrap_or(10.0);
        Ok(secs)
    });

    // SetFadeDuration(secs) - override the stub in methods_core
    methods.add_method("SetFadeDuration", |_, this, secs: f64| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fade_duration = secs;
        Ok(())
    });

    // GetFadeDuration()
    methods.add_method("GetFadeDuration", |_, this, ()| {
        let state = this.state.borrow();
        let secs = state.message_frames.get(&this.id)
            .map(|d| d.fade_duration)
            .unwrap_or(3.0);
        Ok(secs)
    });

    // SetFadePower(power)
    methods.add_method("SetFadePower", |_, this, power: f64| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fade_power = power;
        Ok(())
    });

    // GetFadePower()
    methods.add_method("GetFadePower", |_, this, ()| {
        let state = this.state.borrow();
        let power = state.message_frames.get(&this.id)
            .map(|d| d.fade_power)
            .unwrap_or(1.0);
        Ok(power)
    });

    // SetInsertMode(mode) - override the stub in methods_core
    methods.add_method("SetInsertMode", |_, this, mode: String| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.insert_mode = mode;
        Ok(())
    });

    // GetInsertMode()
    methods.add_method("GetInsertMode", |_, this, ()| {
        let state = this.state.borrow();
        let mode = state.message_frames.get(&this.id)
            .map(|d| d.insert_mode.clone())
            .unwrap_or_else(|| "BOTTOM".to_string());
        Ok(mode)
    });

    // Scroll methods (no-ops for now, no visual scrolling)
    methods.add_method("ScrollUp", |_, _this, ()| Ok(()));
    methods.add_method("ScrollDown", |_, _this, ()| Ok(()));
    methods.add_method("PageUp", |_, _this, ()| Ok(()));
    methods.add_method("PageDown", |_, _this, ()| Ok(()));
    methods.add_method("ScrollToTop", |_, _this, ()| Ok(()));
    methods.add_method("ScrollToBottom", |_, _this, ()| Ok(()));

    // AtTop() / AtBottom()
    methods.add_method("AtTop", |_, _this, ()| Ok(true));
    methods.add_method("AtBottom", |_, _this, ()| Ok(true));

    // SetScrollOffset(offset) / GetScrollOffset()
    methods.add_method("SetScrollOffset", |_, this, offset: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.message_frames.get_mut(&this.id) {
            data.scroll_offset = offset;
        }
        Ok(())
    });
    methods.add_method("GetScrollOffset", |_, this, ()| {
        let state = this.state.borrow();
        let offset = state.message_frames.get(&this.id)
            .map(|d| d.scroll_offset)
            .unwrap_or(0);
        Ok(offset)
    });

    // GetMaxScrollRange()
    methods.add_method("GetMaxScrollRange", |_, _this, ()| Ok(0_i32));

    // SetScrollAllowed(allowed) / IsScrollAllowed()
    methods.add_method("SetScrollAllowed", |_, this, allowed: bool| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.scroll_allowed = allowed;
        Ok(())
    });
    methods.add_method("IsScrollAllowed", |_, this, ()| {
        let state = this.state.borrow();
        let allowed = state.message_frames.get(&this.id)
            .map(|d| d.scroll_allowed)
            .unwrap_or(true);
        Ok(allowed)
    });

    // SetTextCopyable(copyable) / IsTextCopyable()
    methods.add_method("SetTextCopyable", |_, this, copyable: bool| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.text_copyable = copyable;
        Ok(())
    });
    methods.add_method("IsTextCopyable", |_, this, ()| {
        let state = this.state.borrow();
        let copyable = state.message_frames.get(&this.id)
            .map(|d| d.text_copyable)
            .unwrap_or(false);
        Ok(copyable)
    });

    // HasMessageByID(messageID)
    methods.add_method("HasMessageByID", |_, this, id: i64| {
        let state = this.state.borrow();
        let has = state.message_frames.get(&this.id)
            .map(|d| d.messages.iter().any(|m| m.message_id == Some(id)))
            .unwrap_or(false);
        Ok(has)
    });

    // GetMessageInfo(index) - 1-based
    methods.add_method("GetMessageInfo", |_, this, index: i32| {
        let state = this.state.borrow();
        if let Some(data) = state.message_frames.get(&this.id) {
            let idx = (index - 1) as usize;
            if let Some(msg) = data.messages.get(idx) {
                return Ok((msg.text.clone(), msg.r as f64, msg.g as f64, msg.b as f64, msg.a as f64));
            }
        }
        Ok((String::new(), 1.0_f64, 1.0_f64, 1.0_f64, 1.0_f64))
    });

    // Callback stubs
    methods.add_method("SetOnScrollChangedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("SetOnTextCopiedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("SetOnLineRightClickedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("AddOnDisplayRefreshedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("RemoveMessagesByPredicate", |_, _this, _func: Value| Ok(()));
    methods.add_method("TransformMessages", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("AdjustMessageColors", |_, _this, _func: Value| Ok(()));
    methods.add_method("GetFontStringByID", |_, _this, _id: i64| Ok(Value::Nil));
    methods.add_method("ResetMessageFadeByID", |_, _this, _id: i64| Ok(()));
}

fn add_editbox_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFocus", |_, this, ()| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            s.focused_frame_id = Some(this.id);
        }
        Ok(())
    });
    methods.add_method("ClearFocus", |_, this, ()| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if s.focused_frame_id == Some(this.id) {
                s.focused_frame_id = None;
            }
        }
        Ok(())
    });
    methods.add_method("HasFocus", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            return Ok(s.focused_frame_id == Some(this.id));
        }
        Ok(false)
    });
    methods.add_method("SetCursorPosition", |_, _this, _pos: i32| Ok(()));
    methods.add_method("GetCursorPosition", |_, _this, ()| Ok(0));
    methods.add_method("HighlightText", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("Insert", |_, _this, _text: String| Ok(()));
    methods.add_method("SetMaxLetters", |_, _this, _max: i32| Ok(()));
    methods.add_method("GetMaxLetters", |_, _this, ()| Ok(0));
    methods.add_method("SetMaxBytes", |_, _this, _max: i32| Ok(()));
    methods.add_method("GetMaxBytes", |_, _this, ()| Ok(0));
    methods.add_method("SetNumber", |_, this, n: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.text = Some(n.to_string());
        }
        Ok(())
    });
    methods.add_method("GetNumber", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(text) = &frame.text {
                return Ok(text.parse::<f64>().unwrap_or(0.0));
            }
        }
        Ok(0.0)
    });
    methods.add_method("SetMultiLine", |_, _this, _multi: bool| Ok(()));
    methods.add_method("IsMultiLine", |_, _this, ()| Ok(false));
    methods.add_method("SetAutoFocus", |_, _this, _auto: bool| Ok(()));
    methods.add_method("SetNumeric", |_, _this, _numeric: bool| Ok(()));
    methods.add_method("IsNumeric", |_, _this, ()| Ok(false));
    methods.add_method("IsPassword", |_, _this, ()| Ok(false));
    methods.add_method("SetPassword", |_, _this, _pw: bool| Ok(()));
    methods.add_method("SetBlinkSpeed", |_, _this, _speed: f64| Ok(()));
    methods.add_method("SetHistoryLines", |_, _this, _lines: i32| Ok(()));
    methods.add_method("AddHistoryLine", |_, _this, _text: String| Ok(()));
    methods.add_method("GetHistoryLines", |_, _this, ()| Ok(0));
    methods.add_method("SetTextInsets", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetTextInsets", |_, _this, ()| {
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    });
    methods.add_method("SetCountInvisibleLetters", |_, _this, _count: bool| Ok(()));
}

fn add_slider_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetMinMaxValues", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetMinMaxValues", |_, _this, ()| Ok((0.0_f64, 100.0_f64)));
    methods.add_method("SetValue", |_, _this, _value: f64| Ok(()));
    methods.add_method("GetValue", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("SetValueStep", |_, _this, _step: f64| Ok(()));
    methods.add_method("GetValueStep", |_, _this, ()| Ok(1.0_f64));
    methods.add_method("SetOrientation", |_, _this, _orientation: String| Ok(()));
    methods.add_method("GetOrientation", |_, _this, ()| {
        Ok("HORIZONTAL".to_string())
    });
    methods.add_method("SetThumbTexture", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetThumbTexture", |lua, this, ()| {
        // Create or return the thumb texture for slider
        let texture_key = format!("__frame_{}_ThumbTexture", this.id);
        if let Ok(existing) = lua.globals().get::<Value>(texture_key.as_str()) {
            if !matches!(existing, Value::Nil) {
                return Ok(existing);
            }
        }

        let texture = Frame::new(WidgetType::Texture, None, Some(this.id));
        let texture_id = texture.id;

        {
            let mut state = this.state.borrow_mut();
            state.widgets.register(texture);
            state.widgets.add_child(this.id, texture_id);
        }

        let handle = FrameHandle {
            id: texture_id,
            state: Rc::clone(&this.state),
        };

        let ud = lua.create_userdata(handle)?;
        lua.globals().set(texture_key.as_str(), ud.clone())?;

        let frame_key = format!("__frame_{}", texture_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(Value::UserData(ud))
    });
    methods.add_method("SetObeyStepOnDrag", |_, _this, _obey: bool| Ok(()));
    methods.add_method("SetStepsPerPage", |_, _this, _steps: i32| Ok(()));
    methods.add_method("GetStepsPerPage", |_, _this, ()| Ok(1));
}

fn add_statusbar_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetStatusBarTexture", |_, _this, _texture: Value| Ok(()));
    methods.add_method("GetStatusBarTexture", |lua, this, ()| {
        let texture_key = format!("__frame_{}_StatusBarTexture", this.id);
        if let Ok(existing) = lua.globals().get::<Value>(texture_key.as_str()) {
            if !matches!(existing, Value::Nil) {
                return Ok(existing);
            }
        }

        let texture = Frame::new(WidgetType::Texture, None, Some(this.id));
        let texture_id = texture.id;

        {
            let mut state = this.state.borrow_mut();
            state.widgets.register(texture);
            state.widgets.add_child(this.id, texture_id);
        }

        let handle = FrameHandle {
            id: texture_id,
            state: Rc::clone(&this.state),
        };

        let ud = lua.create_userdata(handle)?;
        lua.globals().set(texture_key.as_str(), ud.clone())?;

        let frame_key = format!("__frame_{}", texture_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(Value::UserData(ud))
    });
    methods.add_method("SetStatusBarColor", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("GetStatusBarColor", |_, _this, ()| {
        Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
    });
    methods.add_method("SetRotatesTexture", |_, _this, _rotates: bool| Ok(()));
    methods.add_method("SetReverseFill", |_, _this, _reverse: bool| Ok(()));
    methods.add_method("SetFillStyle", |_, _this, _style: String| Ok(()));
}

fn add_checkbutton_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetChecked", |_, this, checked: bool| {
        {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame
                    .attributes
                    .insert("__checked".to_string(), AttributeValue::Boolean(checked));
            }
            // Also toggle CheckedTexture visibility if it exists
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(&checked_tex_id) = frame.children_keys.get("CheckedTexture") {
                    if let Some(tex) = state.widgets.get_mut(checked_tex_id) {
                        tex.visible = checked;
                    }
                }
            }
        }
        Ok(())
    });
    methods.add_method("GetChecked", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            if let Some(AttributeValue::Boolean(checked)) = frame.attributes.get("__checked") {
                return Ok(*checked);
            }
        }
        Ok(false)
    });
    methods.add_method("GetCheckedTexture", |lua, this, ()| {
        let texture_key = format!("__frame_{}_CheckedTexture", this.id);
        if let Ok(existing) = lua.globals().get::<Value>(texture_key.as_str()) {
            if !matches!(existing, Value::Nil) {
                return Ok(existing);
            }
        }

        let texture = Frame::new(WidgetType::Texture, None, Some(this.id));
        let texture_id = texture.id;

        {
            let mut state = this.state.borrow_mut();
            state.widgets.register(texture);
            state.widgets.add_child(this.id, texture_id);
        }

        let handle = FrameHandle {
            id: texture_id,
            state: Rc::clone(&this.state),
        };

        let ud = lua.create_userdata(handle)?;
        lua.globals().set(texture_key.as_str(), ud.clone())?;

        let frame_key = format!("__frame_{}", texture_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(Value::UserData(ud))
    });
}

fn add_cooldown_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetCooldown", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetCooldownUNIX", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetCooldownTimes", |_, _this, ()| Ok((0.0_f64, 0.0_f64)));
    methods.add_method("SetSwipeColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetDrawSwipe", |_, _this, _draw: bool| Ok(()));
    methods.add_method("SetDrawEdge", |_, _this, _draw: bool| Ok(()));
    methods.add_method("SetDrawBling", |_, _this, _draw: bool| Ok(()));
    methods.add_method("SetReverse", |_, _this, _reverse: bool| Ok(()));
    methods.add_method("SetHideCountdownNumbers", |_, _this, _hide: bool| Ok(()));
    // Note: Clear() for Cooldown frames is handled in __index to avoid conflicts
    // with addons that use frame.Clear as a field
}

fn add_scrollframe_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetScrollChild", |_, _this, _child: Value| Ok(()));
    methods.add_method("GetScrollChild", |_, _this, ()| Ok(Value::Nil));
    methods.add_method("SetHorizontalScroll", |_, _this, _offset: f64| Ok(()));
    methods.add_method("GetHorizontalScroll", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("SetVerticalScroll", |_, _this, _offset: f64| Ok(()));
    methods.add_method("GetVerticalScroll", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("GetHorizontalScrollRange", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("GetVerticalScrollRange", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("UpdateScrollChildRect", |_, _this, ()| Ok(()));
}

fn add_model_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetModel", |_, _this, _path: String| Ok(()));
    methods.add_method("GetModel", |_, _this, ()| -> Result<Option<String>> {
        Ok(None)
    });
    methods.add_method("SetModelScale", |_, _this, _scale: f64| Ok(()));
    methods.add_method("GetModelScale", |_, _this, ()| Ok(1.0_f64));
    methods.add_method("SetPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetPosition", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64))
    });
    methods.add_method("SetFacing", |_, _this, _radians: f64| Ok(()));
    methods.add_method("GetFacing", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("SetUnit", |_, _this, _unit: Option<String>| Ok(()));
    methods.add_method("SetAutoDress", |_, _this, _auto_dress: bool| Ok(()));
    methods.add_method("SetDisplayInfo", |_, _this, _display_id: i32| Ok(()));
    methods.add_method("SetCreature", |_, _this, _creature_id: i32| Ok(()));
    methods.add_method("SetAnimation", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetCamDistanceScale", |_, _this, _scale: f64| Ok(()));
    methods.add_method("GetCamDistanceScale", |_, _this, ()| Ok(1.0_f64));
    methods.add_method("SetCamera", |_, _this, _camera_id: i32| Ok(()));
    methods.add_method("SetPortraitZoom", |_, _this, _zoom: f64| Ok(()));
    methods.add_method("SetDesaturation", |_, _this, _desat: f64| Ok(()));
    methods.add_method("SetRotation", |_, _this, _radians: f64| Ok(()));
    methods.add_method("SetLight", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetSequence", |_, _this, _sequence: i32| Ok(()));
    methods.add_method("SetSequenceTime", |_, _this, (_seq, _time): (i32, i32)| {
        Ok(())
    });
    methods.add_method("ClearModel", |_, _this, ()| Ok(()));
    methods.add_method(
        "TransitionToModelSceneID",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );
    methods.add_method("SetFromModelSceneID", |_, _this, _scene_id: i32| Ok(()));
    methods.add_method("GetModelSceneID", |_, _this, ()| Ok(0i32));
}

/// Native ModelScene methods (C++ side in WoW, stubs here).
/// The Lua-side logic lives in ModelSceneMixin; these are the engine calls it invokes.
fn add_model_scene_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_model_scene_rendering_stubs(methods);
    add_model_scene_camera_stubs(methods);
    add_model_scene_light_stubs(methods);
    add_model_scene_fog_stubs(methods);
}

fn add_model_scene_rendering_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetAllowOverlappedModels", |_, _this, _allow: bool| Ok(()));
    methods.add_method("IsAllowOverlappedModels", |_, _this, ()| Ok(false));
    methods.add_method("SetPaused", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetDrawLayer", |_, _this, _args: mlua::MultiValue| Ok(()));
    // Project3DPointTo2D(x, y, z) -> screenX, screenY, depthScale
    methods.add_method(
        "Project3DPointTo2D",
        |_, _this, _args: mlua::MultiValue| -> Result<(f64, f64, f64)> {
            Ok((0.0, 0.0, 1.0))
        },
    );
    methods.add_method("SetViewInsets", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetViewInsets", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
    });
    methods.add_method("GetViewTranslation", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64))
    });
}

fn add_model_scene_camera_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetCameraPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetCameraPosition", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64))
    });
    methods.add_method("SetCameraOrientationByYawPitchRoll", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetCameraOrientationByAxisVectors", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetCameraForward", |_, _this, ()| Ok((0.0_f64, 0.0_f64, 1.0_f64)));
    methods.add_method("GetCameraRight", |_, _this, ()| Ok((1.0_f64, 0.0_f64, 0.0_f64)));
    methods.add_method("GetCameraUp", |_, _this, ()| Ok((0.0_f64, 1.0_f64, 0.0_f64)));
    methods.add_method("SetCameraFieldOfView", |_, _this, _fov: f64| Ok(()));
    methods.add_method("GetCameraFieldOfView", |_, _this, ()| Ok(0.785_f64));
    methods.add_method("SetCameraNearClip", |_, _this, _clip: f64| Ok(()));
    methods.add_method("SetCameraFarClip", |_, _this, _clip: f64| Ok(()));
    methods.add_method("GetCameraNearClip", |_, _this, ()| Ok(0.1_f64));
    methods.add_method("GetCameraFarClip", |_, _this, ()| Ok(100.0_f64));
}

fn add_model_scene_light_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetLightType", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetLightPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetLightPosition", |_, _this, ()| Ok((0.0_f64, 0.0_f64, 0.0_f64)));
    methods.add_method("SetLightDirection", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetLightDirection", |_, _this, ()| Ok((0.0_f64, -1.0_f64, 0.0_f64)));
    methods.add_method("SetLightAmbientColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetLightDiffuseColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetLightVisible", |_, _this, _visible: bool| Ok(()));
    methods.add_method("IsLightVisible", |_, _this, ()| Ok(true));
}

fn add_model_scene_fog_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFogNear", |_, _this, _near: f64| Ok(()));
    methods.add_method("SetFogFar", |_, _this, _far: f64| Ok(()));
    methods.add_method("SetFogColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("ClearFog", |_, _this, ()| Ok(()));
}

fn add_colorselect_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetColorRGB(r, g, b) - Set the RGB color
    methods.add_method("SetColorRGB", |_, this, (r, g, b): (f64, f64, f64)| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame
                .attributes
                .insert("colorR".to_string(), AttributeValue::Number(r));
            frame
                .attributes
                .insert("colorG".to_string(), AttributeValue::Number(g));
            frame
                .attributes
                .insert("colorB".to_string(), AttributeValue::Number(b));
        }
        Ok(())
    });

    // GetColorRGB() - Get the RGB color
    methods.add_method("GetColorRGB", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let get_num = |key: &str| -> f64 {
                match frame.attributes.get(key) {
                    Some(AttributeValue::Number(n)) => *n,
                    _ => 1.0,
                }
            };
            let r = get_num("colorR");
            let g = get_num("colorG");
            let b = get_num("colorB");
            return Ok((r, g, b));
        }
        Ok((1.0, 1.0, 1.0))
    });

    // SetColorHSV(h, s, v) - Set the HSV color
    methods.add_method("SetColorHSV", |_, this, (h, s, v): (f64, f64, f64)| {
        let (r, g, b) = hsv_to_rgb(h, s, v);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame
                .attributes
                .insert("colorR".to_string(), AttributeValue::Number(r));
            frame
                .attributes
                .insert("colorG".to_string(), AttributeValue::Number(g));
            frame
                .attributes
                .insert("colorB".to_string(), AttributeValue::Number(b));
            frame
                .attributes
                .insert("colorH".to_string(), AttributeValue::Number(h % 360.0));
            frame
                .attributes
                .insert("colorS".to_string(), AttributeValue::Number(s));
            frame
                .attributes
                .insert("colorV".to_string(), AttributeValue::Number(v));
        }
        Ok(())
    });

    // GetColorHSV() - Get the HSV color
    methods.add_method("GetColorHSV", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let get_num = |key: &str| -> Option<f64> {
                match frame.attributes.get(key) {
                    Some(AttributeValue::Number(n)) => Some(*n),
                    _ => None,
                }
            };
            // Check if we have stored HSV values
            if let (Some(h), Some(s), Some(v)) =
                (get_num("colorH"), get_num("colorS"), get_num("colorV"))
            {
                return Ok((h, s, v));
            }
            // Otherwise convert from RGB
            let r: f64 = get_num("colorR").unwrap_or(1.0);
            let g: f64 = get_num("colorG").unwrap_or(1.0);
            let b: f64 = get_num("colorB").unwrap_or(1.0);
            return Ok(rgb_to_hsv(r, g, b));
        }
        Ok((0.0, 0.0, 1.0))
    });
}

fn add_drag_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("StartMoving", |_, this, ()| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                if frame.movable {
                    frame.is_moving = true;
                }
            }
        }
        Ok(())
    });
    methods.add_method("StopMovingOrSizing", |_, this, ()| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                frame.is_moving = false;
            }
        }
        Ok(())
    });
    methods.add_method("SetMovable", |_, this, movable: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                frame.movable = movable;
            }
        }
        Ok(())
    });
    methods.add_method("IsMovable", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            if let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.movable);
            }
        }
        Ok(false)
    });
    methods.add_method("SetResizable", |_, this, resizable: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                frame.resizable = resizable;
            }
        }
        Ok(())
    });
    methods.add_method("IsResizable", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            if let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.resizable);
            }
        }
        Ok(false)
    });
    methods.add_method("SetClampedToScreen", |_, this, clamped: bool| {
        if let Ok(mut s) = this.state.try_borrow_mut() {
            if let Some(frame) = s.widgets.get_mut(this.id) {
                frame.clamped_to_screen = clamped;
            }
        }
        Ok(())
    });
    methods.add_method("IsClampedToScreen", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            if let Some(frame) = s.widgets.get(this.id) {
                return Ok(frame.clamped_to_screen);
            }
        }
        Ok(false)
    });
    methods.add_method("SetClampRectInsets", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("SetResizeBounds", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetResizeBounds", |_, _this, ()| {
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    });
    methods.add_method("StartSizing", |_, _this, _point: Option<String>| Ok(()));
    methods.add_method("RegisterForDrag", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetUserPlaced", |_, _this, _user_placed: bool| Ok(()));
    methods.add_method("IsUserPlaced", |_, _this, ()| Ok(false));
    methods.add_method("SetDontSavePosition", |_, _this, _dont_save: bool| Ok(()));
}

fn add_scrollbox_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("RegisterCallback", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("ForEachFrame", |_, _this, _callback: mlua::Function| Ok(()));
    methods.add_method("UnregisterCallback", |_, _this, _args: mlua::MultiValue| {
        Ok(())
    });
    methods.add_method("CanInterpolateScroll", |_, _this, ()| Ok(false));
    methods.add_method("SetInterpolateScroll", |_, _this, _enabled: bool| Ok(()));
}

fn add_simplehtml_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetHyperlinkFormat(format)
    methods.add_method("SetHyperlinkFormat", |_, this, format: String| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.simple_htmls.get_mut(&this.id) {
            data.hyperlink_format = format;
        }
        Ok(())
    });

    // GetHyperlinkFormat()
    methods.add_method("GetHyperlinkFormat", |_, this, ()| {
        let state = this.state.borrow();
        let format = state
            .simple_htmls
            .get(&this.id)
            .map(|d| d.hyperlink_format.clone())
            .unwrap_or_else(|| "|H%s|h%s|h".to_string());
        Ok(format)
    });

    // SetHyperlinksEnabled(enabled)
    methods.add_method("SetHyperlinksEnabled", |_, this, enabled: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.simple_htmls.get_mut(&this.id) {
            data.hyperlinks_enabled = enabled;
        }
        Ok(())
    });

    // GetHyperlinksEnabled()
    methods.add_method("GetHyperlinksEnabled", |_, this, ()| {
        let state = this.state.borrow();
        let enabled = state
            .simple_htmls
            .get(&this.id)
            .map(|d| d.hyperlinks_enabled)
            .unwrap_or(true);
        Ok(enabled)
    });

    // GetContentHeight() - estimate based on text length and font size
    methods.add_method("GetContentHeight", |_, this, ()| {
        let state = this.state.borrow();
        let frame = match state.widgets.get(this.id) {
            Some(f) => f,
            None => return Ok(0.0_f64),
        };
        let text = match &frame.text {
            Some(t) if !t.is_empty() => t,
            _ => return Ok(0.0_f64),
        };
        let font_size = frame.font_size.max(12.0) as f64;
        let line_height = font_size * 1.2;
        let width = frame.width.max(200.0) as f64;
        let chars_per_line = (width / (font_size * 0.6)).max(1.0);
        let estimated_lines = (text.len() as f64 / chars_per_line).ceil().max(1.0);
        Ok(estimated_lines * line_height)
    });

    // GetTextData() - return empty table (no HTML parsing yet)
    methods.add_method("GetTextData", |lua, _this, ()| {
        let table = lua.create_table()?;
        Ok(table)
    });
}

// --- Helper functions ---

/// Shared AddMessage implementation for AddMessage/AddMsg.
fn add_message_impl(this: &FrameHandle, args: mlua::MultiValue) {
    let args_vec: Vec<Value> = args.into_iter().collect();
    let text = match args_vec.first() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => return,
    };
    let r = val_to_f32_ref(args_vec.get(1), 1.0);
    let g = val_to_f32_ref(args_vec.get(2), 1.0);
    let b = val_to_f32_ref(args_vec.get(3), 1.0);
    let a = val_to_f32_ref(args_vec.get(4), 1.0);
    let message_id = match args_vec.get(5) {
        Some(Value::Integer(n)) => Some(*n),
        Some(Value::Number(n)) => Some(*n as i64),
        _ => None,
    };
    let mut state = this.state.borrow_mut();
    let data = state.message_frames.entry(this.id)
        .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
    if data.insert_mode == "TOP" {
        data.messages.insert(0, crate::lua_api::message_frame::Message {
            text, r, g, b, a, message_id,
        });
    } else {
        data.messages.push(crate::lua_api::message_frame::Message {
            text, r, g, b, a, message_id,
        });
    }
    if data.messages.len() > data.max_lines {
        if data.insert_mode == "TOP" {
            data.messages.pop();
        } else {
            data.messages.remove(0);
        }
    }
}

/// Extract f32 from a reference to a Lua Value.
fn val_to_f32_ref(val: Option<&Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => default,
    }
}

/// Strip HTML tags from a string, returning plain text.
pub(super) fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

/// Extract f32 from a Lua Value, returning default if nil/absent.
pub(super) fn val_to_f32(val: Option<Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => n as f32,
        Some(Value::Integer(n)) => n as f32,
        _ => default,
    }
}

/// Fire a script handler on a frame (e.g. OnTooltipCleared).
fn fire_tooltip_script(lua: &mlua::Lua, frame_id: u64, handler: &str) -> mlua::Result<()> {
    if let Ok(Some(scripts_table)) = lua.globals().get::<Option<mlua::Table>>("__scripts") {
        let key = format!("{}_{}", frame_id, handler);
        if let Ok(Some(func)) = scripts_table.get::<Option<mlua::Function>>(key.as_str()) {
            let frame_key = format!("__frame_{}", frame_id);
            if let Ok(frame_ud) = lua.globals().get::<Value>(frame_key.as_str()) {
                let _ = func.call::<()>(frame_ud);
            }
        }
    }
    Ok(())
}

/// Convert HSV to RGB.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r1 + m, g1 + m, b1 + m)
}

/// Convert RGB to HSV.
fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}
