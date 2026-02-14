//! GameTooltip widget methods: SetOwner, AddLine, AddDoubleLine, tooltip queries, etc.

use super::methods_helpers::get_mixin_override;
use super::FrameHandle;
use crate::lua_api::tooltip::TooltipLine;
use crate::widget::{Anchor, AnchorPoint};
use mlua::{Result, UserDataMethods, Value};

pub fn add_tooltip_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_tooltip_core_methods(methods);
}

/// SetOwner, ClearLines, AddLine, AddDoubleLine, spell/item stubs
fn add_tooltip_core_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_tooltip_setup_methods(methods);
    add_tooltip_addline_methods(methods);
    add_tooltip_doubleline_methods(methods);
    add_tooltip_data_query_stubs(methods);
}

fn add_tooltip_setup_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_tooltip_owner_methods(methods);
    add_tooltip_query_methods(methods);
    add_tooltip_padding_override_methods(methods);
    add_tooltip_settext_methods(methods);
    add_tooltip_info_methods(methods);
    add_tooltip_state_methods(methods);
}

fn add_tooltip_owner_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetOwner(owner, anchor, x, y) - Set the tooltip's owner and anchor
    methods.add_method("SetOwner", |lua, this, args: mlua::MultiValue| {
        // Mixin overrides (e.g. CooldownViewerSettingsEditAlertMixin:SetOwner)
        // shadow this Rust method name. Delegate to the Lua override if present.
        if let Some((func, ud)) = get_mixin_override(lua, this.id, "SetOwner") {
            let mut call_args = vec![ud];
            call_args.extend(args);
            return func.call::<Value>(mlua::MultiValue::from_iter(call_args)).map(|_| ());
        }

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
                td.anchor_type = anchor.clone();
            }
            state.set_frame_visible(this.id, true);
            position_tooltip(&mut state, this.id, owner_id, &anchor);
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
}

fn add_tooltip_addline_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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
}

fn add_tooltip_doubleline_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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
}

fn add_tooltip_data_query_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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
}

fn add_tooltip_query_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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

    // AddTexture(texture) - Add a texture to the tooltip (stub)
    methods.add_method("AddTexture", |_, _this, _texture: String| Ok(()));
}

fn add_tooltip_padding_override_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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
    methods.add_method("GetPadding", |lua, this, ()| -> Result<mlua::MultiValue> {
        if let Some((func, ud)) = get_mixin_override(lua, this.id, "GetPadding") {
            return func.call::<mlua::MultiValue>(ud);
        }
        let state = this.state.borrow();
        let padding = Value::Number(
            state
                .tooltips
                .get(&this.id)
                .map(|td| td.padding as f64)
                .unwrap_or(0.0),
        );
        Ok(mlua::MultiValue::from_iter(std::iter::once(padding)))
    });
}

fn add_tooltip_settext_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetText is registered in methods_text (handles tooltip, SimpleHTML, and button child propagation).

    // AppendText(text) - Append to last line's left_text
    methods.add_method("AppendText", |_, this, text: String| {
        let mut state = this.state.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&this.id)
            && let Some(last) = td.lines.last_mut() {
                last.left_text.push_str(&text);
            }
        Ok(())
    });
}

fn add_tooltip_info_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
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
}

fn add_tooltip_state_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // FadeOut() - Hide tooltip, clear owner
    methods.add_method("FadeOut", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        state.set_frame_visible(this.id, false);
        if let Some(td) = state.tooltips.get_mut(&this.id) {
            td.owner_id = None;
        }
        Ok(())
    });
}

// --- Positioning ---

/// Set anchors on the tooltip frame based on anchor_type from SetOwner.
fn position_tooltip(
    state: &mut crate::lua_api::state::SimState,
    tooltip_id: u64,
    owner_id: Option<u64>,
    anchor_type: &str,
) {
    let frame = match state.widgets.get_mut_visual(tooltip_id) {
        Some(f) => f,
        None => return,
    };
    frame.anchors.clear();

    match anchor_type {
        "ANCHOR_CURSOR" => {
            // Position at mouse cursor + 20px Y offset
            let (mx, my) = state.mouse_position.unwrap_or((0.0, 0.0));
            frame.anchors.push(Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: None,
                relative_to_id: None,
                relative_point: AnchorPoint::TopLeft,
                x_offset: mx,
                y_offset: my + 20.0,
            });
        }
        "ANCHOR_NONE" => {
            // Addon will call SetPoint manually — don't set anchors
        }
        _ => {
            let owner = match owner_id {
                Some(id) => id,
                None => return,
            };
            let (tp, rp) = anchor_points_for_type(anchor_type);
            frame.anchors.push(Anchor {
                point: tp,
                relative_to: None,
                relative_to_id: Some(owner as usize),
                relative_point: rp,
                x_offset: 0.0,
                y_offset: 0.0,
            });
        }
    }
}

/// Map anchor_type string to (tooltip_point, owner_point).
fn anchor_points_for_type(anchor_type: &str) -> (AnchorPoint, AnchorPoint) {
    match anchor_type {
        "ANCHOR_RIGHT" => (AnchorPoint::TopLeft, AnchorPoint::TopRight),
        "ANCHOR_LEFT" => (AnchorPoint::TopRight, AnchorPoint::TopLeft),
        "ANCHOR_TOPLEFT" => (AnchorPoint::BottomLeft, AnchorPoint::TopLeft),
        "ANCHOR_TOPRIGHT" => (AnchorPoint::BottomLeft, AnchorPoint::TopRight),
        "ANCHOR_BOTTOMLEFT" => (AnchorPoint::TopLeft, AnchorPoint::BottomLeft),
        "ANCHOR_BOTTOMRIGHT" => (AnchorPoint::TopLeft, AnchorPoint::BottomRight),
        // Default to ANCHOR_RIGHT behavior
        _ => (AnchorPoint::TopLeft, AnchorPoint::TopRight),
    }
}

// --- Shared helpers ---

/// Fire a script handler on a frame (e.g. OnTooltipCleared).
pub(super) fn fire_tooltip_script(lua: &mlua::Lua, frame_id: u64, handler: &str) -> mlua::Result<()> {
    if let Some(func) = crate::lua_api::script_helpers::get_script(lua, frame_id, handler)
        && let Some(frame_ud) = crate::lua_api::script_helpers::get_frame_ref(lua, frame_id)
            && let Err(e) = func.call::<()>(frame_ud) {
                crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
            }
    Ok(())
}

/// Extract f32 from a Lua Value, returning default if nil/absent.
pub(super) fn val_to_f32(val: Option<Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => n as f32,
        Some(Value::Integer(n)) => n as f32,
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
