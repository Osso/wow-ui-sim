use super::thumb_texture::{
    assign_texture_payload, ensure_named_child_texture, get_named_child_texture_id,
};
use crate::lua_api::methods::{
    borrow_state_mut, extract_frame_id, frame_id_from_stack, frame_ref, get_or_create_frame_fields,
    table_get_static, table_set_static,
};
use crate::lua_api::script_helpers::{
    call_error_handler_state, get_scripts_for_dispatch, protected_lua_pcall_state,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use super::super::shared::val_to_f64;

fn read_color_component(state: &mut LuaState, id: u64, key: &'static str, default: f64) -> f64 {
    let fields = get_or_create_frame_fields(state, id);
    match table_get_static(state, fields, key) {
        Val::Num(value) => value,
        _ => default,
    }
}

fn write_color_components(state: &mut LuaState, id: u64, rgba: (f64, f64, f64, f64)) {
    let fields = get_or_create_frame_fields(state, id);
    table_set_static(state, fields, "__color_r", Val::Num(rgba.0));
    table_set_static(state, fields, "__color_g", Val::Num(rgba.1));
    table_set_static(state, fields, "__color_b", Val::Num(rgba.2));
    table_set_static(state, fields, "__color_a", Val::Num(rgba.3));
}

fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let sat = if max == 0.0 { 0.0 } else { delta / max };
    (hue, sat, max)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0).rem_euclid(2.0)) - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h.rem_euclid(360.0) {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r1 + m, g1 + m, b1 + m)
}

fn get_colorselect_texture(state: &mut LuaState, id: u64, key: &str) -> LuaResult<u32> {
    match get_named_child_texture_id(state, id, key) {
        Some(child_id) => {
            let texture = frame_ref(state, child_id)?;
            state.push(texture);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn set_colorselect_texture(state: &mut LuaState, id: u64, key: &str, value: Val) -> LuaResult<u32> {
    if matches!(value, Val::Nil) {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.children_keys.remove(key);
        }
        return Ok(0);
    }
    if let Some(child_id) = extract_frame_id(state, value) {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.children_keys.insert(key.to_string(), child_id);
        }
        return Ok(0);
    }
    let child_id = ensure_named_child_texture(state, id, key)?;
    assign_texture_payload(state, child_id, value)?;
    Ok(0)
}

fn colorselect_set_color_rgb(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let old_rgb = (
        read_color_component(state, id, "__color_r", 1.0),
        read_color_component(state, id, "__color_g", 1.0),
        read_color_component(state, id, "__color_b", 1.0),
    );
    let rgb = (
        val_to_f64(stack_val(state, 2)),
        val_to_f64(stack_val(state, 3)),
        val_to_f64(stack_val(state, 4)),
    );
    let alpha = read_color_component(state, id, "__color_a", 1.0);
    write_color_components(state, id, (rgb.0, rgb.1, rgb.2, alpha));
    if rgb != old_rgb {
        fire_color_select(state, id, rgb)?;
    }
    Ok(0)
}

fn fire_color_select(state: &mut LuaState, id: u64, rgb: (f64, f64, f64)) -> LuaResult<()> {
    let handlers = get_scripts_for_dispatch(state, id, "OnColorSelect");
    if handlers.is_empty() {
        return Ok(());
    }
    let frame = frame_ref(state, id)?;
    let args = [frame, Val::Num(rgb.0), Val::Num(rgb.1), Val::Num(rgb.2)];
    for handler in handlers {
        if let Err(error) = protected_lua_pcall_state(state, handler, &args) {
            call_error_handler_state(state, &error);
        }
    }
    Ok(())
}

fn colorselect_get_color_rgb(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = read_color_component(state, id, "__color_r", 1.0);
    let g = read_color_component(state, id, "__color_g", 1.0);
    let b = read_color_component(state, id, "__color_b", 1.0);
    state.push(Val::Num(r));
    state.push(Val::Num(g));
    state.push(Val::Num(b));
    Ok(3)
}

fn colorselect_set_color_hsv(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let alpha = read_color_component(state, id, "__color_a", 1.0);
    let h = val_to_f64(stack_val(state, 2));
    let s = val_to_f64(stack_val(state, 3));
    let v = val_to_f64(stack_val(state, 4));
    let (r, g, b) = hsv_to_rgb(h, s, v);
    write_color_components(state, id, (r, g, b, alpha));
    Ok(0)
}

fn colorselect_get_color_hsv(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = read_color_component(state, id, "__color_r", 1.0);
    let g = read_color_component(state, id, "__color_g", 1.0);
    let b = read_color_component(state, id, "__color_b", 1.0);
    let (h, s, v) = rgb_to_hsv(r, g, b);
    state.push(Val::Num(h));
    state.push(Val::Num(s));
    state.push(Val::Num(v));
    Ok(3)
}

fn colorselect_set_color_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = read_color_component(state, id, "__color_r", 1.0);
    let g = read_color_component(state, id, "__color_g", 1.0);
    let b = read_color_component(state, id, "__color_b", 1.0);
    let a = val_to_f64(stack_val(state, 2));
    write_color_components(state, id, (r, g, b, a));
    Ok(0)
}

fn colorselect_get_color_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let alpha = read_color_component(state, id, "__color_a", 1.0);
    state.push(Val::Num(alpha));
    Ok(1)
}

macro_rules! colorselect_texture_methods {
    ($set_fn:ident, $get_fn:ident, $key:literal) => {
        fn $set_fn(state: &mut LuaState) -> LuaResult<u32> {
            let id = frame_id_from_stack(state, 1)?;
            let value = stack_val(state, 2);
            set_colorselect_texture(state, id, $key, value)
        }

        fn $get_fn(state: &mut LuaState) -> LuaResult<u32> {
            let id = frame_id_from_stack(state, 1)?;
            get_colorselect_texture(state, id, $key)
        }
    };
}

colorselect_texture_methods!(
    colorselect_set_color_alpha_texture,
    colorselect_get_color_alpha_texture,
    "ColorAlphaTexture"
);
colorselect_texture_methods!(
    colorselect_set_color_alpha_thumb_texture,
    colorselect_get_color_alpha_thumb_texture,
    "ColorAlphaThumbTexture"
);
colorselect_texture_methods!(
    colorselect_set_color_value_texture,
    colorselect_get_color_value_texture,
    "ColorValueTexture"
);
colorselect_texture_methods!(
    colorselect_set_color_value_thumb_texture,
    colorselect_get_color_value_thumb_texture,
    "ColorValueThumbTexture"
);
colorselect_texture_methods!(
    colorselect_set_color_wheel_texture,
    colorselect_get_color_wheel_texture,
    "ColorWheelTexture"
);
colorselect_texture_methods!(
    colorselect_set_color_wheel_thumb_texture,
    colorselect_get_color_wheel_thumb_texture,
    "ColorWheelThumbTexture"
);

fn colorselect_clear_color_wheel_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    set_colorselect_texture(state, id, "ColorWheelTexture", Val::Nil)
}

const COLORSELECT_COLOR_METHODS: &[(&str, RustFn)] = &[
    ("SetColorRGB", colorselect_set_color_rgb),
    ("GetColorRGB", colorselect_get_color_rgb),
    ("SetColorHSV", colorselect_set_color_hsv),
    ("GetColorHSV", colorselect_get_color_hsv),
    ("SetColorAlpha", colorselect_set_color_alpha),
    ("GetColorAlpha", colorselect_get_color_alpha),
];

const COLORSELECT_TEXTURE_METHODS: &[(&str, RustFn)] = &[
    ("SetColorAlphaTexture", colorselect_set_color_alpha_texture),
    ("GetColorAlphaTexture", colorselect_get_color_alpha_texture),
    (
        "SetColorAlphaThumbTexture",
        colorselect_set_color_alpha_thumb_texture,
    ),
    (
        "GetColorAlphaThumbTexture",
        colorselect_get_color_alpha_thumb_texture,
    ),
    ("SetColorValueTexture", colorselect_set_color_value_texture),
    ("GetColorValueTexture", colorselect_get_color_value_texture),
    (
        "SetColorValueThumbTexture",
        colorselect_set_color_value_thumb_texture,
    ),
    (
        "GetColorValueThumbTexture",
        colorselect_get_color_value_thumb_texture,
    ),
    ("SetColorWheelTexture", colorselect_set_color_wheel_texture),
    ("GetColorWheelTexture", colorselect_get_color_wheel_texture),
    (
        "SetColorWheelThumbTexture",
        colorselect_set_color_wheel_thumb_texture,
    ),
    (
        "GetColorWheelThumbTexture",
        colorselect_get_color_wheel_thumb_texture,
    ),
    (
        "ClearColorWheelTexture",
        colorselect_clear_color_wheel_texture,
    ),
];

pub(in crate::lua_api::frame::methods::widgets) fn register_colorselect(
    state: &mut LuaState,
    metatable: GcRef<Table>,
) -> LuaResult<()> {
    register_colorselect_methods(state, metatable, COLORSELECT_COLOR_METHODS)?;
    register_colorselect_methods(state, metatable, COLORSELECT_TEXTURE_METHODS)?;
    Ok(())
}

fn register_colorselect_methods(
    state: &mut LuaState,
    metatable: GcRef<Table>,
    methods: &[(&'static str, RustFn)],
) -> LuaResult<()> {
    for &(name, function) in methods {
        table_set_rust_fn_static(state, metatable, name, function)?;
    }
    Ok(())
}
