//! Slider, StatusBar, CheckButton methods and shared SetValue/GetValue/SetMinMaxValues.

use super::widget_tooltip::val_to_f32;
use crate::lua_api::frame::handle::{frame_lud, get_sim_state, lud_to_id};
use crate::widget::{AttributeValue, Color, WidgetType};
use mlua::{LightUserData, Lua, Result, Value};

pub fn add_slider_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // Note: SetMinMaxValues/GetMinMaxValues/SetValue/GetValue are in add_shared_value_methods
    add_slider_step_methods(lua, methods)?;
    add_slider_orientation_methods(lua, methods)?;
    add_slider_thumb_methods(lua, methods)?;
    add_slider_drag_methods(lua, methods)?;
    Ok(())
}

pub fn add_statusbar_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    // Note: SetMinMaxValues/GetMinMaxValues/SetValue/GetValue are in add_shared_value_methods
    add_statusbar_texture_methods(lua, methods)?;
    add_statusbar_color_methods(lua, methods)?;
    add_statusbar_fill_methods(lua, methods)?;
    add_statusbar_desaturate_methods(lua, methods)?;
    Ok(())
}

pub fn add_checkbutton_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetChecked", lua.create_function(|lua, (ud, checked): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        // Skip if already the same value
        let already = state.widgets.get(id)
            .and_then(|f| f.attributes.get("__checked"))
            .map(|v| matches!(v, AttributeValue::Boolean(b) if *b == checked))
            .unwrap_or(false);
        if already {
            return Ok(());
        }
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.attributes.insert("__checked".to_string(), AttributeValue::Boolean(checked));
        }
        // Toggle CheckedTexture visibility
        let checked_tex_id = state.widgets.get(id)
            .and_then(|f| f.children_keys.get("CheckedTexture").copied());
        if let Some(tex_id) = checked_tex_id {
            state.set_frame_visible(tex_id, checked);
        }
        Ok(())
    })?)?;

    methods.set("GetChecked", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(AttributeValue::Boolean(checked)) = frame.attributes.get("__checked") {
                return Ok(*checked);
            }
        Ok(false)
    })?)?;

    methods.set("GetCheckedTexture", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        get_or_create_child_texture(lua, id, "CheckedTexture")
    })?)?;

    Ok(())
}

/// Shared SetValue/GetValue/SetMinMaxValues/GetMinMaxValues that dispatch by widget type.
/// Must be registered last so it overwrites both slider and statusbar individual registrations.
pub fn add_shared_value_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_shared_set_value(lua, methods)?;
    add_shared_get_value(lua, methods)?;
    add_shared_set_min_max(lua, methods)?;
    add_shared_get_min_max(lua, methods)?;
    Ok(())
}

// --- Slider methods ---

fn add_slider_step_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetValueStep", lua.create_function(|lua, (ud, step): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.slider_step = step;
        }
        Ok(())
    })?)?;

    methods.set("GetValueStep", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let step = state.widgets.get(id).map(|f| f.slider_step).unwrap_or(1.0);
        Ok(step)
    })?)?;

    Ok(())
}

fn add_slider_orientation_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetOrientation", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        if let Some((func, frame_ud)) = super::methods_helpers::get_mixin_override(lua, id, "SetOrientation") {
            let mut call_args = vec![frame_ud];
            call_args.extend(args);
            return func.call::<Value>(mlua::MultiValue::from_iter(call_args)).map(|_| ());
        }
        if let Some(Value::String(s)) = args.into_iter().next() {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.slider_orientation = s.to_str().map(|s| s.to_uppercase()).unwrap_or_else(|_| "HORIZONTAL".to_string());
            }
        }
        Ok(())
    })?)?;

    methods.set("GetOrientation", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let orientation = state.widgets.get(id)
            .map(|f| f.slider_orientation.clone())
            .unwrap_or_else(|| "HORIZONTAL".to_string());
        Ok(orientation)
    })?)?;

    Ok(())
}

fn add_slider_thumb_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetThumbTexture", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    methods.set("GetThumbTexture", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        get_or_create_child_texture(lua, id, "ThumbTexture")
    })?)?;

    Ok(())
}

fn add_slider_drag_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetObeyStepOnDrag", lua.create_function(|lua, (ud, obey): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.slider_obey_step_on_drag = obey;
        }
        Ok(())
    })?)?;

    methods.set("SetStepsPerPage", lua.create_function(|lua, (ud, steps): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.slider_steps_per_page = steps;
        }
        Ok(())
    })?)?;

    methods.set("GetStepsPerPage", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let steps = state.widgets.get(id).map(|f| f.slider_steps_per_page).unwrap_or(1);
        Ok(steps)
    })?)?;

    Ok(())
}

// --- StatusBar methods ---

fn add_statusbar_texture_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_set_statusbar_texture(lua, methods)?;
    add_get_statusbar_texture(lua, methods)?;
    methods.set("SetRotatesTexture", lua.create_function(|_, (_ud, _rotates): (LightUserData, bool)| Ok(()))?)?;
    Ok(())
}

fn add_set_statusbar_texture(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetStatusBarTexture", lua.create_function(|lua, (ud, texture): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let (path, bar_id) = match &texture {
            Value::String(s) => (Some(s.to_string_lossy().to_string()), None),
            Value::LightUserData(lud) => (None, Some(lud_to_id(*lud))),
            _ => (None, None),
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.statusbar_texture_path = path.clone();
            if let Some(bid) = bar_id {
                frame.statusbar_bar_id = Some(bid);
            }
        }
        if let Some(ref tex_str) = path {
            apply_statusbar_texture_path(&mut state, id, tex_str);
        }
        if let Some(bid) = bar_id {
            anchor_bar_to_parent(&mut state.widgets, bid, id);
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_get_statusbar_texture(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("GetStatusBarTexture", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let bar_id = {
            let state_rc = get_sim_state(lua);
            let state = state_rc.borrow();
            state.widgets.get(id).and_then(|f| f.statusbar_bar_id)
        };
        if let Some(bar_id) = bar_id {
            return Ok(frame_lud(bar_id));
        }
        get_or_create_child_texture(lua, id, "StatusBarTexture")
    })?)?;
    Ok(())
}

fn add_statusbar_color_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetStatusBarColor", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let mut it = args.into_iter();
        let r = val_to_f32(it.next(), 1.0);
        let g = val_to_f32(it.next(), 1.0);
        let b = val_to_f32(it.next(), 1.0);
        let a = val_to_f32(it.next(), 1.0);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.statusbar_color = Some(Color::new(r, g, b, a));
        }
        Ok(())
    })?)?;

    methods.set("GetStatusBarColor", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(c) = &frame.statusbar_color {
                return Ok((c.r, c.g, c.b, c.a));
            }
        Ok((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32))
    })?)?;

    Ok(())
}

fn add_statusbar_fill_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetFillStyle", lua.create_function(|lua, (ud, style): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.statusbar_fill_style = style;
        }
        Ok(())
    })?)?;

    methods.set("SetReverseFill", lua.create_function(|lua, (ud, reverse): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.statusbar_reverse_fill = reverse;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_statusbar_desaturate_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetStatusBarDesaturated", lua.create_function(|_, (_ud, _desat): (LightUserData, bool)| Ok(()))?)?;
    methods.set("GetStatusBarDesaturated", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("SetStatusBarAtlas", lua.create_function(|_, (_ud, _atlas): (LightUserData, String)| Ok(()))?)?;
    methods.set("GetFillStyle", lua.create_function(|_, _ud: LightUserData| Ok("STANDARD"))?)?;
    methods.set("GetReverseFill", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("GetRotatesTexture", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    Ok(())
}

// --- Shared value methods ---

fn add_shared_set_value(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetValue", lua.create_function(|lua, (ud, value): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let wtype = {
            let s = state_rc.borrow();
            s.widgets.get(id).map(|f| f.widget_type)
        };
        match wtype {
            Some(WidgetType::Slider) => set_slider_value(lua, id, value)?,
            Some(WidgetType::StatusBar) => set_statusbar_value(lua, id, value)?,
            _ => {}
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_shared_get_value(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("GetValue", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            return Ok(match frame.widget_type {
                WidgetType::Slider => frame.slider_value,
                WidgetType::StatusBar => frame.statusbar_value,
                _ => 0.0,
            });
        }
        Ok(0.0_f64)
    })?)?;
    Ok(())
}

fn add_shared_set_min_max(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetMinMaxValues", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let (min, max) = parse_min_max_args(args);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if !min_max_changed(&state, id, min, max) {
            return Ok(());
        }
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            apply_min_max(frame, min, max);
        }
        Ok(())
    })?)?;
    Ok(())
}

fn add_shared_get_min_max(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("GetMinMaxValues", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            return Ok(match frame.widget_type {
                WidgetType::Slider => (frame.slider_min, frame.slider_max),
                WidgetType::StatusBar => (frame.statusbar_min, frame.statusbar_max),
                _ => (0.0, 1.0),
            });
        }
        Ok((0.0_f64, 1.0_f64))
    })?)?;
    Ok(())
}

// --- Helper functions ---

fn set_slider_value(lua: &mlua::Lua, id: u64, value: f64) -> mlua::Result<()> {
    let clamped = {
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let Some(frame) = state.widgets.get(id) else { return Ok(()) };
        let clamped = value.clamp(frame.slider_min, frame.slider_max);
        if clamped == frame.slider_value {
            return Ok(());
        }
        let frame = state.widgets.get_mut_visual(id).unwrap();
        frame.slider_value = clamped;
        clamped
    };
    fire_value_changed(lua, id, clamped)
}

fn set_statusbar_value(lua: &mlua::Lua, id: u64, value: f64) -> mlua::Result<()> {
    let clamped = {
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let Some(frame) = state.widgets.get(id) else { return Ok(()) };
        let clamped = value.clamp(frame.statusbar_min, frame.statusbar_max);
        if clamped == frame.statusbar_value {
            return Ok(());
        }
        let frame = state.widgets.get_mut_visual(id).unwrap();
        frame.statusbar_value = clamped;
        clamped
    };
    fire_value_changed(lua, id, clamped)
}

/// Fire OnValueChanged script with the new value as argument.
fn fire_value_changed(lua: &mlua::Lua, frame_id: u64, value: f64) -> mlua::Result<()> {
    if let Some(func) = crate::lua_api::script_helpers::get_script(lua, frame_id, "OnValueChanged")
        && let Some(frame_ud) = crate::lua_api::script_helpers::get_frame_ref(lua, frame_id)
            && let Err(e) = func.call::<()>((frame_ud, value)) {
                crate::lua_api::script_helpers::call_error_handler(lua, &e.to_string());
            }
    Ok(())
}

fn parse_min_max_args(args: mlua::MultiValue) -> (f64, f64) {
    let mut it = args.into_iter();
    let min = match it.next() {
        Some(Value::Number(n)) => n,
        Some(Value::Integer(n)) => n as f64,
        _ => 0.0,
    };
    let max = match it.next() {
        Some(Value::Number(n)) => n,
        Some(Value::Integer(n)) => n as f64,
        _ => 1.0,
    };
    (min, max)
}

fn min_max_changed(state: &crate::lua_api::SimState, id: u64, min: f64, max: f64) -> bool {
    state.widgets.get(id).map(|frame| match frame.widget_type {
        WidgetType::Slider => frame.slider_min != min || frame.slider_max != max,
        WidgetType::StatusBar => frame.statusbar_min != min || frame.statusbar_max != max,
        _ => false,
    }).unwrap_or(false)
}

fn apply_min_max(frame: &mut crate::widget::Frame, min: f64, max: f64) {
    match frame.widget_type {
        WidgetType::Slider => {
            frame.slider_min = min;
            frame.slider_max = max;
            frame.slider_value = frame.slider_value.clamp(min, max);
        }
        WidgetType::StatusBar => {
            frame.statusbar_min = min;
            frame.statusbar_max = max;
            frame.statusbar_value = frame.statusbar_value.clamp(min, max);
        }
        _ => {}
    }
}

/// Apply a texture path string to a StatusBar's bar child, creating it if needed.
fn apply_statusbar_texture_path(state: &mut crate::lua_api::SimState, id: u64, tex_str: &str) {
    let bar_child_id = find_bar_texture_child(&state.widgets, id)
        .unwrap_or_else(|| {
            let child_id = super::methods_helpers::get_or_create_button_texture(state, id, "StatusBarTexture");
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.statusbar_bar_id = Some(child_id);
            }
            child_id
        });
    apply_bar_texture(&mut state.widgets, bar_child_id, tex_str);
    anchor_bar_to_parent(&mut state.widgets, bar_child_id, id);
}

/// Find the bar texture child of a StatusBar by stored ID or children_keys.
fn find_bar_texture_child(widgets: &crate::widget::WidgetRegistry, parent_id: u64) -> Option<u64> {
    let frame = widgets.get(parent_id)?;
    frame.statusbar_bar_id
        .or_else(|| frame.children_keys.get("BarTexture").copied())
        .or_else(|| frame.children_keys.get("StatusBarTexture").copied())
        .or_else(|| frame.children_keys.get("Bar").copied())
}

/// Apply a texture path or atlas name to a bar texture child.
fn apply_bar_texture(widgets: &mut crate::widget::WidgetRegistry, child_id: u64, tex_str: &str) {
    // Try atlas lookup first
    if let Some(lookup) = crate::atlas::get_atlas_info(tex_str) {
        let info = lookup.info;
        if let Some(frame) = widgets.get_mut_visual(child_id) {
            frame.texture = Some(info.file.to_string());
            let uvs = (info.left_tex_coord, info.right_tex_coord, info.top_tex_coord, info.bottom_tex_coord);
            frame.atlas_tex_coords = Some(uvs);
            frame.tex_coords = Some(uvs);
            frame.horiz_tile = info.tiles_horizontally;
            frame.vert_tile = info.tiles_vertically;
            frame.atlas = Some(tex_str.to_string());
        }
    } else if let Some(frame) = widgets.get_mut_visual(child_id) {
        // Treat as a file path
        frame.texture = Some(tex_str.to_string());
        frame.atlas = None;
        frame.tex_coords = None;
        frame.tex_coords_quad = None;
        frame.atlas_tex_coords = None;
    }
}

/// Apply SetAllPoints-style anchors to make a bar texture fill its parent.
fn anchor_bar_to_parent(widgets: &mut crate::widget::WidgetRegistry, bar_id: u64, parent_id: u64) {
    use crate::widget::{Anchor, AnchorPoint};
    if let Some(bar) = widgets.get_mut_visual(bar_id) {
        bar.anchors = vec![
            Anchor {
                point: AnchorPoint::TopLeft,
                relative_to: None,
                relative_to_id: Some(parent_id as usize),
                relative_point: AnchorPoint::TopLeft,
                x_offset: 0.0,
                y_offset: 0.0,
            },
            Anchor {
                point: AnchorPoint::BottomRight,
                relative_to: None,
                relative_to_id: Some(parent_id as usize),
                relative_point: AnchorPoint::BottomRight,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        ];
    }
}

/// Look up or create a child texture by key and return it as a LightUserData Value.
/// Used by GetThumbTexture, GetStatusBarTexture, GetCheckedTexture, etc.
pub(super) fn get_or_create_child_texture(
    lua: &Lua,
    id: u64,
    key: &str,
) -> Result<Value> {
    let state_rc = get_sim_state(lua);
    let mut state = state_rc.borrow_mut();
    let tex_id = super::methods_helpers::get_or_create_button_texture(&mut state, id, key);
    Ok(frame_lud(tex_id))
}
