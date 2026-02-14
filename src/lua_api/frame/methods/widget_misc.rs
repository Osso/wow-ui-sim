//! Miscellaneous widget methods: ColorSelect, drag/move/resize, SimpleHTML, and stubs.

use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use crate::widget::AttributeValue;
use mlua::{LightUserData, Lua, Value};

pub fn add_colorselect_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_colorselect_rgb_methods(lua, methods)?;
    add_colorselect_hsv_methods(lua, methods)?;
    Ok(())
}

pub fn add_drag_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_drag_move_methods(lua, methods)?;
    add_drag_movable_resizable_methods(lua, methods)?;
    add_drag_clamp_methods(lua, methods)?;
    add_drag_resize_methods(lua, methods)?;
    Ok(())
}

pub fn add_simplehtml_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_simplehtml_hyperlink_methods(lua, methods)?;
    add_simplehtml_content_methods(lua, methods)?;
    Ok(())
}

/// Miscellaneous widget method stubs referenced by Blizzard UI.
pub fn add_misc_widget_stubs(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_misc_stubs_simple(lua, methods)?;
    add_misc_stubs_mixin(lua, methods)?;
    Ok(())
}

// --- SimpleHTML ---

fn add_simplehtml_hyperlink_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetHyperlinkFormat(format)
    methods.set("SetHyperlinkFormat", lua.create_function(|lua, (ud, format): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(data) = state.simple_htmls.get_mut(&id) {
            data.hyperlink_format = format;
        }
        Ok(())
    })?)?;

    // GetHyperlinkFormat()
    methods.set("GetHyperlinkFormat", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let format = state
            .simple_htmls
            .get(&id)
            .map(|d| d.hyperlink_format.clone())
            .unwrap_or_else(|| "|H%s|h%s|h".to_string());
        Ok(format)
    })?)?;

    // SetHyperlinksEnabled(enabled)
    methods.set("SetHyperlinksEnabled", lua.create_function(|lua, (ud, enabled): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(data) = state.simple_htmls.get_mut(&id) {
            data.hyperlinks_enabled = enabled;
        }
        Ok(())
    })?)?;

    // GetHyperlinksEnabled()
    methods.set("GetHyperlinksEnabled", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let enabled = state
            .simple_htmls
            .get(&id)
            .map(|d| d.hyperlinks_enabled)
            .unwrap_or(true);
        Ok(enabled)
    })?)?;

    Ok(())
}

fn add_simplehtml_content_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // GetContentHeight() - estimate based on text length and font size
    methods.set("GetContentHeight", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let frame = match state.widgets.get(id) {
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
    })?)?;

    // GetTextData() - return empty table (no HTML parsing yet)
    methods.set("GetTextData", lua.create_function(|lua, _ud: LightUserData| {
        let table = lua.create_table()?;
        Ok(table)
    })?)?;

    Ok(())
}

// --- ColorSelect ---

fn add_colorselect_rgb_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetColorRGB(r, g, b) - Set the RGB color
    methods.set("SetColorRGB", lua.create_function(|lua, (ud, r, g, b): (LightUserData, f64, f64, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.attributes.insert("colorR".to_string(), AttributeValue::Number(r));
            frame.attributes.insert("colorG".to_string(), AttributeValue::Number(g));
            frame.attributes.insert("colorB".to_string(), AttributeValue::Number(b));
        }
        Ok(())
    })?)?;

    // GetColorRGB() - Get the RGB color
    methods.set("GetColorRGB", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            let get_num = |key: &str| -> f64 {
                match frame.attributes.get(key) {
                    Some(AttributeValue::Number(n)) => *n,
                    _ => 1.0,
                }
            };
            return Ok((get_num("colorR"), get_num("colorG"), get_num("colorB")));
        }
        Ok((1.0, 1.0, 1.0))
    })?)?;

    Ok(())
}

fn add_colorselect_hsv_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetColorHSV(h, s, v) - Set the HSV color
    methods.set("SetColorHSV", lua.create_function(|lua, (ud, h, s, v): (LightUserData, f64, f64, f64)| {
        let id = lud_to_id(ud);
        let (r, g, b) = hsv_to_rgb(h, s, v);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.attributes.insert("colorR".to_string(), AttributeValue::Number(r));
            frame.attributes.insert("colorG".to_string(), AttributeValue::Number(g));
            frame.attributes.insert("colorB".to_string(), AttributeValue::Number(b));
            frame.attributes.insert("colorH".to_string(), AttributeValue::Number(h % 360.0));
            frame.attributes.insert("colorS".to_string(), AttributeValue::Number(s));
            frame.attributes.insert("colorV".to_string(), AttributeValue::Number(v));
        }
        Ok(())
    })?)?;

    // GetColorHSV() - Get the HSV color
    methods.set("GetColorHSV", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            let get_num = |key: &str| -> Option<f64> {
                match frame.attributes.get(key) {
                    Some(AttributeValue::Number(n)) => Some(*n),
                    _ => None,
                }
            };
            if let (Some(h), Some(s), Some(v)) =
                (get_num("colorH"), get_num("colorS"), get_num("colorV"))
            {
                return Ok((h, s, v));
            }
            let r: f64 = get_num("colorR").unwrap_or(1.0);
            let g: f64 = get_num("colorG").unwrap_or(1.0);
            let b: f64 = get_num("colorB").unwrap_or(1.0);
            return Ok(rgb_to_hsv(r, g, b));
        }
        Ok((0.0, 0.0, 1.0))
    })?)?;

    Ok(())
}

// --- Drag/Move/Resize ---

fn add_drag_move_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("StartMoving", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id)
                && frame.movable {
                    frame.is_moving = true;
                }
        Ok(())
    })?)?;

    methods.set("StopMovingOrSizing", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id) {
                frame.is_moving = false;
            }
        Ok(())
    })?)?;

    methods.set("SetMovable", lua.create_function(|lua, (ud, movable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id) {
                frame.movable = movable;
            }
        Ok(())
    })?)?;

    methods.set("IsMovable", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(s) = state_rc.try_borrow()
            && let Some(frame) = s.widgets.get(id) {
                return Ok(frame.movable);
            }
        Ok(false)
    })?)?;

    Ok(())
}

fn add_drag_movable_resizable_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetResizable", lua.create_function(|lua, (ud, resizable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id) {
                frame.resizable = resizable;
            }
        Ok(())
    })?)?;

    methods.set("IsResizable", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(s) = state_rc.try_borrow()
            && let Some(frame) = s.widgets.get(id) {
                return Ok(frame.resizable);
            }
        Ok(false)
    })?)?;

    Ok(())
}

fn add_drag_clamp_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetClampedToScreen", lua.create_function(|lua, (ud, clamped): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(mut s) = state_rc.try_borrow_mut()
            && let Some(frame) = s.widgets.get_mut_visual(id) {
                frame.clamped_to_screen = clamped;
            }
        Ok(())
    })?)?;

    methods.set("IsClampedToScreen", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(s) = state_rc.try_borrow()
            && let Some(frame) = s.widgets.get(id) {
                return Ok(frame.clamped_to_screen);
            }
        Ok(false)
    })?)?;

    methods.set("SetClampRectInsets", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| {
        Ok(())
    })?)?;

    Ok(())
}

fn add_drag_resize_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetResizeBounds", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetResizeBounds", lua.create_function(|_, _ud: LightUserData| {
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    })?)?;
    // Legacy resize bound methods (deprecated in favor of SetResizeBounds)
    methods.set("SetMinResize", lua.create_function(|_, (_ud, _w, _h): (LightUserData, f32, f32)| Ok(()))?)?;
    methods.set("SetMaxResize", lua.create_function(|_, (_ud, _w, _h): (LightUserData, f32, f32)| Ok(()))?)?;
    methods.set("StartSizing", lua.create_function(|_, (_ud, _point): (LightUserData, Option<String>)| Ok(()))?)?;
    methods.set("RegisterForDrag", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetUserPlaced", lua.create_function(|_, (_ud, _user_placed): (LightUserData, bool)| Ok(()))?)?;
    methods.set("IsUserPlaced", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("SetDontSavePosition", lua.create_function(|_, (_ud, _dont_save): (LightUserData, bool)| Ok(()))?)?;
    Ok(())
}

// --- Misc stubs ---

fn add_misc_stubs_simple(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // SetupMenu(generator) - dropdown/context menu setup (used everywhere)
    methods.set("SetupMenu", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    // SetAlertContainer(container) - alert frame system
    methods.set("SetAlertContainer", lua.create_function(|_, (_ud, _container): (LightUserData, Value)| Ok(()))?)?;

    // SetColorFill(r, g, b, a) - StatusBar fill color
    methods.set("SetColorFill", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    // SetTextToFit(text) - FontString method that auto-sizes
    methods.set("SetTextToFit", lua.create_function(|lua, (ud, text): (LightUserData, Option<String>)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.text = text;
        }
        Ok(())
    })?)?;

    // SetSelectionTranslator(func) - tab/dropdown selection
    methods.set("SetSelectionTranslator", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;

    // SetItemButtonScale(scale) - item button sizing
    methods.set("SetItemButtonScale", lua.create_function(|_, (_ud, _scale): (LightUserData, Value)| Ok(()))?)?;

    // UpdateItemContextMatching() - item slot context matching (PaperDoll)
    methods.set("UpdateItemContextMatching", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    // UpdateHeight() - recalculate height (UIWidget containers)
    methods.set("UpdateHeight", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;

    // SetDefaultText(text) - dropdown default text
    methods.set("SetDefaultText", lua.create_function(|_, (_ud, _text): (LightUserData, Value)| Ok(()))?)?;

    // SetVisuals(info) - UnitFrame spark/bar visual configuration
    methods.set("SetVisuals", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    // RegisterForWidgetSet(widgetSetID, layoutFunc, initFunc, attachedUnitInfo)
    methods.set("RegisterForWidgetSet", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    // UnregisterForWidgetSet()
    methods.set("UnregisterForWidgetSet", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    Ok(())
}

fn add_misc_stubs_mixin(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // Mixin override: ModelScenelRotateButtonMixin defines SetRotationIncrement(increment)
    methods.set("SetRotationIncrement", lua.create_function(|lua, (ud, inc): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        if let Some((func, frame_ud)) = super::methods_helpers::get_mixin_override(lua, id, "SetRotationIncrement") {
            return func.call::<()>((frame_ud, inc));
        }
        Ok(())
    })?)?;

    // Init() - DO NOT add a Rust stub here. ScrollBoxListMixin:Init (Lua) must
    // be callable via __index. A Rust method takes priority and would shadow it.

    // NOTE: Initialize is intentionally NOT defined here as a no-op.
    // It must be dispatched via Lua mixin methods (e.g. AlternatePowerBarBaseMixin,
    // EvokerEbonMightBarMixin) through the __index custom fields lookup.
    // A Rust add_method shadows __index, breaking mixin dispatch.

    Ok(())
}

// --- Color conversion helpers ---

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
