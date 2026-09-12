//! Button state, enable/disable, click, and related methods.

use crate::lua_api::frame::methods::forbidden_aspects::ensure_forbidden_aspect_absent;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, extract_frame_id,
    frame_id_from_stack, frame_ref, get_or_create_frame_fields, table_get, table_set,
    val_to_string,
};
use crate::lua_api::script_helpers::{call_error_handler_state, get_script as get_rilua_script};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Button attribute helpers ──────────────────────────────────────────────────

pub(super) fn button_enabled(frame: &crate::widget::Frame) -> bool {
    frame
        .attributes
        .get("__enabled")
        .and_then(|value| match value {
            crate::widget::AttributeValue::Boolean(value) => Some(*value),
            _ => None,
        })
        .unwrap_or(true)
}

pub(super) fn sync_button_slot_visibility(sim: &mut crate::lua_api::SimState, button_id: u64) {
    for key in [
        "NormalTexture",
        "PushedTexture",
        "DisabledTexture",
        "HighlightTexture",
        "CheckedTexture",
        "DisabledCheckedTexture",
    ] {
        let child_id = sim
            .widgets
            .get(button_id)
            .and_then(|button| button.children_keys.get(key).copied());
        if let Some(child_id) = child_id {
            let should_show = super::textures::button_texture_should_show(sim, button_id, key);
            sim.widgets.set_visible(child_id, should_show);
        }
    }
}

pub(super) fn set_button_enabled_value(
    state: &mut LuaState,
    id: u64,
    enabled: bool,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get(id)
        .map(|frame| button_enabled(frame) != enabled)
        .unwrap_or(false);
    if changed {
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.attributes.insert(
                "__enabled".to_string(),
                crate::widget::AttributeValue::Boolean(enabled),
            );
        }
        sync_button_slot_visibility(&mut sim, id);
    }
    Ok(())
}

// ── Button methods ────────────────────────────────────────────────────────────

pub(super) fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(button_enabled).unwrap_or(true)
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}

/// Fire the `OnEnable` or `OnDisable` Lua script for a button, if registered.
fn fire_enable_disable_script(state: &mut LuaState, id: u64, enabled: bool) {
    let handler_name = if enabled { "OnEnable" } else { "OnDisable" };
    let Some(handler) = get_rilua_script(state, id, handler_name) else {
        return;
    };
    let Ok(frame) = frame_ref(state, id) else {
        return;
    };
    if let Err(error) = call_function_state(state, handler, &[frame]) {
        call_error_handler_state(state, &error.to_string());
    }
}

pub(super) fn set_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = bool::from_stack(state, 2).ok().unwrap_or(true);
    let changed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| button_enabled(f) != enabled)
            .unwrap_or(false)
    };
    set_button_enabled_value(state, id, enabled)?;
    if changed {
        fire_enable_disable_script(state, id, enabled);
    }
    Ok(0)
}

pub(super) fn enable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let was_disabled = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| !button_enabled(f))
            .unwrap_or(false)
    };
    set_button_enabled_value(state, id, true)?;
    if was_disabled {
        fire_enable_disable_script(state, id, true);
    }
    Ok(0)
}

pub(super) fn disable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let was_enabled = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(button_enabled).unwrap_or(true)
    };
    set_button_enabled_value(state, id, false)?;
    if was_enabled {
        fire_enable_disable_script(state, id, false);
    }
    Ok(0)
}

pub(super) fn register_for_clicks(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let buttons = collect_click_registration_args(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.registered_click_buttons = buttons;
    }
    Ok(0)
}

fn collect_click_registration_args(
    state: &mut LuaState,
    start: i32,
) -> std::collections::HashSet<String> {
    let mut buttons = std::collections::HashSet::new();
    let mut index = start;
    loop {
        let value = stack_val(state, index);
        if value == Val::Nil {
            break;
        }
        if let Some(button) = val_to_string(state, value) {
            buttons.insert(button);
        }
        index += 1;
    }
    buttons
}

pub(super) fn set_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let state_name = String::from_stack(state, 2)?;
    let pushed = state_name.eq_ignore_ascii_case("PUSHED");
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.button_state = if pushed { 1 } else { 0 };
        }
        sync_button_slot_visibility(&mut sim, id);
    }
    Ok(0)
}

pub(super) fn get_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let pushed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| frame.button_state == 1)
            .unwrap_or(false)
    };
    let name = if pushed { "PUSHED" } else { "NORMAL" };
    let name_val = create_string(state, name);
    state.push(name_val);
    Ok(1)
}

pub(super) fn is_down(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let is_down = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| frame.button_state == 1)
            .unwrap_or(false)
    };
    state.push(Val::Bool(is_down));
    Ok(1)
}

pub(super) fn is_over(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let is_over = {
        let fields = get_or_create_frame_fields(state, id);
        matches!(table_get(state, fields, "over"), Val::Bool(true))
    };
    state.push(Val::Bool(is_over));
    Ok(1)
}

pub(super) fn is_down_over(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let is_down = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| frame.button_state == 1)
            .unwrap_or(false)
    };
    let is_over = {
        let fields = get_or_create_frame_fields(state, id);
        matches!(table_get(state, fields, "over"), Val::Bool(true))
    };
    state.push(Val::Bool(is_down && is_over));
    Ok(1)
}

pub(super) fn click(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    ensure_forbidden_aspect_absent(state, id, "ScriptedInput", "Click")?;
    if !begin_click(state, id)? {
        return Ok(0);
    }
    crate::lua_api::frame::methods::widgets::toggle_checkbutton_for_click(state, id)?;
    let Some(handler) = get_rilua_script(state, id, "OnClick") else {
        end_click(state, id);
        return Ok(0);
    };
    if matches!(handler, Val::Nil) {
        end_click(state, id);
        return Ok(0);
    }
    let self_ref = frame_ref(state, id)?;
    let button = create_string(state, "LeftButton");
    let args = [self_ref, button, Val::Bool(false)];
    if let Err(error) = call_function_state(state, handler, &args) {
        call_error_handler_state(state, &error.to_string());
    }
    end_click(state, id);
    Ok(0)
}

fn begin_click(state: &mut LuaState, id: u64) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get_mut(id) else {
        return Ok(false);
    };
    if frame.click_depth > 0 {
        return Ok(false);
    }
    frame.click_depth = 1;
    Ok(true)
}

fn end_click(state: &mut LuaState, id: u64) {
    if let Ok(mut sim) = borrow_state_mut(state)
        && let Some(frame) = sim.widgets.get_mut(id)
    {
        frame.click_depth = 0;
    }
}

pub(super) fn set_item_button_scale(state: &mut LuaState) -> LuaResult<u32> {
    let self_table = stack_val(state, 1);
    if let Some(self_id) = extract_frame_id(state, self_table) {
        let fields = get_or_create_frame_fields(state, self_id);
        let override_fn = table_get(state, fields, "SetItemButtonScale");
        if matches!(override_fn, Val::Function(_)) {
            let arg_count = state.top.saturating_sub(state.base) as i32;
            let args: Vec<Val> = (1..=arg_count)
                .map(|index| stack_val(state, index))
                .collect();
            let _ = call_function_state(state, override_fn, &args)?;
            return Ok(0);
        }
        table_set(state, fields, "itemButtonScale", stack_val(state, 2));
    }
    let scale = f64::from_stack(state, 2)?;
    let count = table_get(state, self_table, "Count");
    if let Some(count_id) = extract_frame_id(state, count) {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(count_id) {
            frame.scale = scale as f32;
        }
    }
    Ok(0)
}

pub(super) fn calculate_action(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let result = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        let button_id = frame.map(|widget| widget.user_id).unwrap_or(0);
        if button_id > 0 {
            button_id
        } else {
            frame
                .and_then(|widget| widget.attributes.get("action"))
                .and_then(|value| match value {
                    crate::widget::AttributeValue::Number(number) => Some(*number as i32),
                    _ => None,
                })
                .unwrap_or(1)
        }
    };
    state.push(Val::Num(result as f64));
    Ok(1)
}

// ── Button font objects ───────────────────────────────────────────────────────

use crate::lua_api::methods::registry_table_or_create;

fn get_or_create_button_font_store(state: &mut LuaState) -> Val {
    registry_table_or_create(state, "__button_font_objects")
}

pub(super) fn has_normal_font_object(state: &mut LuaState, id: u64) -> bool {
    let store = get_or_create_button_font_store(state);
    !matches!(table_get(state, store, &format!("{id}:normal")), Val::Nil)
}

pub(super) fn set_normal_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let font_object = resolve_font_object_arg(state, 2);
    let store = get_or_create_button_font_store(state);
    table_set(state, store, &format!("{id}:normal"), font_object);
    let Some(text_child_id) = super::font_strings::ensure_button_text_child(state, id)? else {
        return Ok(0);
    };
    if matches!(font_object, Val::Table(_)) {
        let fields = super::font_strings::read_font_object_fields(state, font_object);
        let mut sim = borrow_state_mut(state)?;
        if let Some(text_child) = sim.widgets.get_mut_visual(text_child_id) {
            super::font_strings::apply_font_object_snapshot(text_child, &fields);
        }
    }
    Ok(0)
}

fn resolve_font_object_arg(state: &mut LuaState, index: i32) -> Val {
    let raw = stack_val(state, index);
    if let Val::Str(_) = raw {
        if let Some(name) = String::from_stack(state, index).ok() {
            let resolved = table_get(state, Val::Table(state.global), &name);
            if matches!(resolved, Val::Table(_)) {
                return resolved;
            }
        }
    }
    raw
}

pub(super) fn get_normal_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let store = get_or_create_button_font_store(state);
    let font_object = table_get(state, store, &format!("{id}:normal"));
    state.push(font_object);
    Ok(1)
}

pub(super) fn set_highlight_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let font_object = resolve_font_object_arg(state, 2);
    let store = get_or_create_button_font_store(state);
    table_set(state, store, &format!("{id}:highlight"), font_object);
    Ok(0)
}

pub(super) fn get_highlight_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let store = get_or_create_button_font_store(state);
    let font_object = table_get(state, store, &format!("{id}:highlight"));
    state.push(font_object);
    Ok(1)
}

pub(super) fn set_disabled_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let font_object = resolve_font_object_arg(state, 2);
    let store = get_or_create_button_font_store(state);
    table_set(state, store, &format!("{id}:disabled"), font_object);
    Ok(0)
}

pub(super) fn get_disabled_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let store = get_or_create_button_font_store(state);
    let font_object = table_get(state, store, &format!("{id}:disabled"));
    state.push(font_object);
    Ok(1)
}

// ── Pushed text offset ────────────────────────────────────────────────────────

pub(super) fn set_pushed_text_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = f64::from_stack(state, 2)? as f32;
    let y = f64::from_stack(state, 3)? as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.pushed_text_offset = (x, y);
    }
    Ok(0)
}

pub(super) fn get_pushed_text_offset(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::IntoStack;
    let id = frame_id_from_stack(state, 1)?;
    let (x, y) = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| f.pushed_text_offset)
            .unwrap_or((0.0, 0.0))
    };
    (x as f64, y as f64).into_stack(state)
}
