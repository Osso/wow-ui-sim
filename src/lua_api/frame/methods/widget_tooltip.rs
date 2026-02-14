//! GameTooltip widget methods: SetOwner, AddLine, AddDoubleLine, tooltip queries, etc.

use super::methods_helpers::get_mixin_override;
use crate::lua_api::frame::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use crate::lua_api::tooltip::TooltipLine;
use crate::widget::{Anchor, AnchorPoint};
use mlua::{LightUserData, Lua, Result, Value};

pub fn add_tooltip_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_tooltip_core_methods(lua, methods)
}

/// SetOwner, ClearLines, AddLine, AddDoubleLine, spell/item stubs
fn add_tooltip_core_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_tooltip_setup_methods(lua, methods)?;
    add_tooltip_addline_methods(lua, methods)?;
    add_tooltip_doubleline_methods(lua, methods)?;
    add_tooltip_data_query_stubs(lua, methods)?;
    Ok(())
}

fn add_tooltip_setup_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_tooltip_owner_methods(lua, methods)?;
    add_tooltip_query_methods(lua, methods)?;
    add_tooltip_padding_override_methods(lua, methods)?;
    add_tooltip_settext_methods(lua, methods)?;
    add_tooltip_info_methods(lua, methods)?;
    add_tooltip_state_methods(lua, methods)?;
    Ok(())
}

fn add_tooltip_owner_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // SetOwner(owner, anchor, x, y) - Set the tooltip's owner and anchor
    methods.set("SetOwner", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        // Mixin overrides (e.g. CooldownViewerSettingsEditAlertMixin:SetOwner)
        // shadow this Rust method name. Delegate to the Lua override if present.
        if let Some((func, self_val)) = get_mixin_override(lua, id, "SetOwner") {
            let mut call_args = vec![self_val];
            call_args.extend(args);
            return func.call::<Value>(mlua::MultiValue::from_iter(call_args)).map(|_| ());
        }

        set_owner_impl(lua, id, args)
    })?)?;

    // ClearLines() - Clear all text lines from the tooltip
    methods.set("ClearLines", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(td) = state.tooltips.get_mut(&id) {
                td.lines.clear();
            }
        }
        fire_tooltip_script(lua, id, "OnTooltipCleared")?;
        Ok(())
    })?)?;

    Ok(())
}

/// Implementation of SetOwner after mixin override check.
fn set_owner_impl(lua: &Lua, id: u64, args: mlua::MultiValue) -> Result<()> {
    let mut args_iter = args.into_iter();
    let owner_val = args_iter.next().unwrap_or(Value::Nil);
    let anchor: String = match args_iter.next() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => "ANCHOR_NONE".to_string(),
    };

    let owner_id = extract_frame_id(&owner_val);

    // Clear lines and set owner
    {
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&id) {
            td.lines.clear();
            td.owner_id = owner_id;
            td.anchor_type = anchor.clone();
        }
        state.set_frame_visible(id, true);
        position_tooltip(&mut state, id, owner_id, &anchor);
    }

    // Fire OnTooltipCleared
    fire_tooltip_script(lua, id, "OnTooltipCleared")?;
    Ok(())
}

fn add_tooltip_addline_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // AddLine(text, r, g, b, wrap) - Add a line of text
    methods.set("AddLine", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
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

        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&id) {
            td.lines.push(TooltipLine {
                left_text: text,
                left_color: (r, g, b),
                right_text: None,
                right_color: (1.0, 1.0, 1.0),
                wrap,
            });
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_tooltip_doubleline_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // AddDoubleLine(leftText, rightText, lR, lG, lB, rR, rG, rB) - Add two-column line
    methods.set("AddDoubleLine", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        add_double_line_impl(lua, id, args)
    })?)?;
    Ok(())
}

/// Implementation of AddDoubleLine, extracted for line limit.
fn add_double_line_impl(lua: &Lua, id: u64, args: mlua::MultiValue) -> Result<()> {
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

    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    if let Some(td) = state.tooltips.get_mut(&id) {
        td.lines.push(TooltipLine {
            left_text: left,
            left_color: (lr, lg, lb),
            right_text: Some(right),
            right_color: (rr, rg, rb),
            wrap: false,
        });
    }
    Ok(())
}

fn add_tooltip_data_query_stubs(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // SetSpellByID(spellID) - Set tooltip to show spell info (no game data)
    methods.set("SetSpellByID", lua.create_function(|_, (_ud, _spell_id): (LightUserData, i32)| Ok(()))?)?;

    // SetItemByID(itemID) - Set tooltip to show item info (no game data)
    methods.set("SetItemByID", lua.create_function(|_, (_ud, _item_id): (LightUserData, i32)| Ok(()))?)?;

    // SetHyperlink(link) - Set tooltip from a hyperlink (no game data)
    methods.set("SetHyperlink", lua.create_function(|_, (_ud, _link): (LightUserData, String)| Ok(()))?)?;

    // SetUnitBuff/Debuff/Aura stubs (no game data)
    methods.set("SetUnitBuff", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetUnitDebuff", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetUnitAura", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetUnitBuffByAuraInstanceID", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetUnitDebuffByAuraInstanceID", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    // NumLines() - Get number of lines in tooltip
    methods.set("NumLines", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let count = state
            .tooltips
            .get(&id)
            .map(|td| td.lines.len())
            .unwrap_or(0);
        Ok(count as i32)
    })?)?;

    Ok(())
}

fn add_tooltip_query_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // GetUnit() - Get the unit this tooltip is showing info for (no game data)
    methods.set("GetUnit", lua.create_function(|_, _ud: LightUserData| -> Result<(Option<String>, Option<String>)> {
        Ok((None, None))
    })?)?;

    // GetSpell() - Get the spell this tooltip is showing info for (no game data)
    methods.set("GetSpell", lua.create_function(|_, _ud: LightUserData| -> Result<(Option<String>, Option<i32>)> {
        Ok((None, None))
    })?)?;

    // GetItem() - Get the item this tooltip is showing info for (no game data)
    methods.set("GetItem", lua.create_function(|_, _ud: LightUserData| -> Result<(Option<String>, Option<String>)> {
        Ok((None, None))
    })?)?;

    add_tooltip_minwidth_methods(lua, methods)?;

    // AddTexture(texture) - Add a texture to the tooltip (stub)
    methods.set("AddTexture", lua.create_function(|_, (_ud, _texture): (LightUserData, String)| Ok(()))?)?;

    Ok(())
}

fn add_tooltip_minwidth_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // SetMinimumWidth(width) / GetMinimumWidth()
    methods.set("SetMinimumWidth", lua.create_function(|lua, (ud, width): (LightUserData, f32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&id) {
            td.min_width = width;
        }
        Ok(())
    })?)?;

    methods.set("GetMinimumWidth", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state
            .tooltips
            .get(&id)
            .map(|td| td.min_width)
            .unwrap_or(0.0))
    })?)?;

    Ok(())
}

fn add_tooltip_padding_override_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // SetPadding(padding) / GetPadding()
    // These check for Lua mixin overrides first (e.g., ScrollBoxBaseMixin:GetPadding)
    // because Rust add_method methods shadow mixin methods stored in __frame_fields.
    methods.set("SetPadding", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        if let Some((func, self_val)) = get_mixin_override(lua, id, "SetPadding") {
            let mut call_args = vec![self_val];
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
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&id) {
            td.padding = padding;
        }
        Ok(())
    })?)?;

    methods.set("GetPadding", lua.create_function(|lua, ud: LightUserData| -> Result<mlua::MultiValue> {
        let id = lud_to_id(ud);
        if let Some((func, self_val)) = get_mixin_override(lua, id, "GetPadding") {
            return func.call::<mlua::MultiValue>(self_val);
        }
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let padding = Value::Number(
            state
                .tooltips
                .get(&id)
                .map(|td| td.padding as f64)
                .unwrap_or(0.0),
        );
        Ok(mlua::MultiValue::from_iter(std::iter::once(padding)))
    })?)?;

    Ok(())
}

fn add_tooltip_settext_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // SetText is registered in methods_text (handles tooltip, SimpleHTML, and button child propagation).

    // AppendText(text) - Append to last line's left_text
    methods.set("AppendText", lua.create_function(|lua, (ud, text): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(td) = state.tooltips.get_mut(&id)
            && let Some(last) = td.lines.last_mut() {
                last.left_text.push_str(&text);
            }
        Ok(())
    })?)?;
    Ok(())
}

fn add_tooltip_info_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // IsOwned(frame) - Check if tooltip is owned by a frame
    methods.set("IsOwned", lua.create_function(|lua, (ud, frame): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let check_id = extract_frame_id(&frame);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let owned = state.tooltips.get(&id).is_some_and(|td| {
            td.owner_id.is_some() && td.owner_id == check_id
        });
        Ok(owned)
    })?)?;

    // GetOwner() - Return the owner frame
    methods.set("GetOwner", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let owner_id = {
            let state_rc = get_sim_state(lua);
            let state = state_rc.borrow();
            state.tooltips.get(&id).and_then(|td| td.owner_id)
        };
        match owner_id {
            Some(oid) => Ok(frame_lud(oid)),
            None => Ok(Value::Nil),
        }
    })?)?;

    // GetAnchorType() - Return the anchor type string
    methods.set("GetAnchorType", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let anchor = state
            .tooltips
            .get(&id)
            .map(|td| td.anchor_type.clone())
            .unwrap_or_else(|| "ANCHOR_NONE".to_string());
        Ok(anchor)
    })?)?;

    Ok(())
}

fn add_tooltip_state_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // FadeOut() - Hide tooltip, clear owner
    methods.set("FadeOut", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        state.set_frame_visible(id, false);
        if let Some(td) = state.tooltips.get_mut(&id) {
            td.owner_id = None;
        }
        Ok(())
    })?)?;
    Ok(())
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
