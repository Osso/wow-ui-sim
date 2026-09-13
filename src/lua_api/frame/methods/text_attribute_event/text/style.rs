use super::super::helpers::{store_simple_attribute, val_to_f32};
use super::frame_text_scale_value;
use super::simple_html::{
    get_simple_html_text_color, is_simple_html_frame, set_simple_html_text_color,
    with_simple_html_data_mut,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_string_static, frame_id_from_stack,
    get_or_create_frame_fields, table_get, table_set, val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Copy, Clone)]
enum JustifyAxis {
    Horizontal,
    Vertical,
}

pub(crate) fn set_hyperlinks_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = matches!(stack_val(state, 2), Val::Bool(true));
    if with_simple_html_data_mut(state, id, |data| {
        data.hyperlinks_enabled = enabled;
    })
    .is_some()
    {
        return Ok(0);
    }
    store_simple_attribute(state, id, "__hyperlinks_enabled", Val::Bool(enabled))?;
    Ok(0)
}

pub(crate) fn get_hyperlinks_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(enabled) = with_simple_html_data_mut(state, id, |data| data.hyperlinks_enabled) {
        state.push(Val::Bool(enabled));
        return Ok(1);
    }
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.attributes.get("__hyperlinks_enabled"))
        .is_some_and(|value| matches!(value, crate::widget::AttributeValue::Boolean(true)));
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub(crate) fn set_hyperlink_format(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if with_simple_html_data_mut(state, id, |data| {
        data.hyperlink_format = value.clone();
    })
    .is_some()
    {
        return Ok(0);
    }
    let stored_value = create_string(state, &value);
    store_simple_attribute(state, id, "__hyperlink_format", stored_value)?;
    Ok(0)
}

pub(crate) fn get_hyperlink_format(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if let Some(value) = with_simple_html_data_mut(state, id, |data| data.hyperlink_format.clone())
    {
        let format_value = create_string(state, &value);
        state.push(format_value);
        return Ok(1);
    }
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.attributes.get("__hyperlink_format"))
        .and_then(|value| match value {
            crate::widget::AttributeValue::String(value) => Some(value.clone()),
            _ => None,
        });
    match value {
        Some(value) => {
            let format_value = create_string(state, &value);
            state.push(format_value);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(crate) fn set_indented_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let enabled = matches!(stack_val(state, 3), Val::Bool(true));
    let fields = get_or_create_frame_fields(state, id);
    let key = format!("__indented_word_wrap_{text_type}");
    table_set(state, fields, &key, Val::Bool(enabled));
    Ok(0)
}

pub(crate) fn get_indented_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let fields = get_or_create_frame_fields(state, id);
    let key = format!("__indented_word_wrap_{text_type}");
    let enabled = table_get(state, fields, &key) == Val::Bool(true);
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub(crate) fn set_text_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        set_simple_html_text_color(
            state,
            id,
            text_type,
            (
                val_to_f32(stack_val(state, 3), 1.0),
                val_to_f32(stack_val(state, 4), 1.0),
                val_to_f32(stack_val(state, 5), 1.0),
                val_to_f32(stack_val(state, 6), 1.0),
            ),
        );
        return Ok(0);
    }
    let r = val_to_f32(stack_val(state, 2), 1.0);
    let g = val_to_f32(stack_val(state, 3), 1.0);
    let b = val_to_f32(stack_val(state, 4), 1.0);
    let a = val_to_f32(stack_val(state, 5), 1.0);
    let new_color = crate::widget::Color::new(r, g, b, a);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id)
        && frame.text_color != new_color
    {
        frame.text_color = new_color;
    }
    Ok(0)
}

pub(crate) fn set_fixed_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fixed = matches!(stack_val(state, 2), Val::Bool(true));
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.text_color_fixed = fixed;
    }
    Ok(0)
}

pub(crate) fn get_text_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id)
        && let Some((r, g, b, a)) = get_simple_html_text_color(state, id, text_type)
    {
        state.push(Val::Num(r as f64));
        state.push(Val::Num(g as f64));
        state.push(Val::Num(b as f64));
        state.push(Val::Num(a as f64));
        return Ok(4);
    }
    let sim = borrow_state(state)?;
    let (r, g, b, a) = sim
        .widgets
        .get(id)
        .map(|f| {
            (
                f.text_color.r,
                f.text_color.g,
                f.text_color.b,
                f.text_color.a,
            )
        })
        .unwrap_or((1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32));
    drop(sim);
    state.push(Val::Num(r as f64));
    state.push(Val::Num(g as f64));
    state.push(Val::Num(b as f64));
    state.push(Val::Num(a as f64));
    Ok(4)
}

pub(crate) fn set_shadow_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = val_to_f32(stack_val(state, 2), 0.0);
    let g = val_to_f32(stack_val(state, 3), 0.0);
    let b = val_to_f32(stack_val(state, 4), 0.0);
    let a = val_to_f32(stack_val(state, 5), 1.0);
    let new_color = crate::widget::Color::new(r, g, b, a);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.shadow_color = new_color;
    }
    Ok(0)
}

pub(crate) fn get_shadow_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (r, g, b, a) = sim
        .widgets
        .get(id)
        .map(|f| {
            (
                f.shadow_color.r,
                f.shadow_color.g,
                f.shadow_color.b,
                f.shadow_color.a,
            )
        })
        .unwrap_or((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32));
    drop(sim);
    state.push(Val::Num(r as f64));
    state.push(Val::Num(g as f64));
    state.push(Val::Num(b as f64));
    state.push(Val::Num(a as f64));
    Ok(4)
}

pub(crate) fn set_shadow_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = val_to_f32(stack_val(state, 2), 0.0);
    let y = val_to_f32(stack_val(state, 3), 0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.shadow_offset = (x, y);
    }
    Ok(0)
}

pub(crate) fn get_shadow_offset(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (x, y) = sim
        .widgets
        .get(id)
        .map(|f| f.shadow_offset)
        .unwrap_or((0.0_f32, 0.0_f32));
    drop(sim);
    state.push(Val::Num(x as f64));
    state.push(Val::Num(y as f64));
    Ok(2)
}

pub(crate) fn set_justify_h(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(justification) = val_to_string(state, stack_val(state, 2)) else {
        return Ok(0);
    };
    set_justify_value(state, id, &justification, JustifyAxis::Horizontal)
}

pub(crate) fn get_justify_h(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let justify = get_justify_value(state, id, JustifyAxis::Horizontal)?;
    let justify_val = create_string_static(state, justify);
    state.push(justify_val);
    Ok(1)
}

pub(crate) fn set_justify_v(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(justification) = val_to_string(state, stack_val(state, 2)) else {
        return Ok(0);
    };
    set_justify_value(state, id, &justification, JustifyAxis::Vertical)
}

fn set_justify_value(
    state: &mut LuaState,
    id: u64,
    justification: &str,
    axis: JustifyAxis,
) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let justify = crate::widget::TextJustify::from_wow_str(justification);
        match axis {
            JustifyAxis::Horizontal => frame.justify_h = justify,
            JustifyAxis::Vertical => frame.justify_v = justify,
        }
    }
    Ok(0)
}

pub(crate) fn get_justify_v(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let justify = get_justify_value(state, id, JustifyAxis::Vertical)?;
    let justify_val = create_string_static(state, justify);
    state.push(justify_val);
    Ok(1)
}

fn get_justify_value(state: &LuaState, id: u64, axis: JustifyAxis) -> LuaResult<&'static str> {
    let sim = borrow_state(state)?;
    Ok(sim
        .widgets
        .get(id)
        .map(|frame| match axis {
            JustifyAxis::Horizontal => frame.justify_h.as_h_str(),
            JustifyAxis::Vertical => frame.justify_v.as_v_str(),
        })
        .unwrap_or(match axis {
            JustifyAxis::Horizontal => "LEFT",
            JustifyAxis::Vertical => "TOP",
        }))
}

pub(crate) fn set_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let word_wrap = matches!(stack_val(state, 2), Val::Bool(true));
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.word_wrap = word_wrap;
    }
    Ok(0)
}

pub(crate) fn get_max_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max_lines = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.max_lines)
        .unwrap_or(0);
    state.push(Val::Num(max_lines as f64));
    Ok(1)
}

pub(crate) fn set_max_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let max_lines = match stack_val(state, 2) {
        Val::Num(value) if value >= 0.0 => value as u32,
        _ => 0,
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.max_lines = max_lines;
    }
    Ok(0)
}

pub(crate) fn get_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let word_wrap = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.word_wrap)
        .unwrap_or(true);
    state.push(Val::Bool(word_wrap));
    Ok(1)
}

pub(crate) fn can_word_wrap(state: &mut LuaState) -> LuaResult<u32> {
    get_word_wrap(state)
}

pub(crate) fn set_non_space_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = matches!(stack_val(state, 2), Val::Bool(true));
    store_simple_attribute(state, id, "__non_space_wrap", Val::Bool(enabled))?;
    Ok(0)
}

pub(crate) fn can_non_space_wrap(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.attributes.get("__non_space_wrap"))
        .is_some_and(|value| matches!(value, crate::widget::AttributeValue::Boolean(true)));
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
pub(crate) fn set_scale_animation_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    // Simulator validation policy; native coercion is unverified.
    let mode = match stack_val(state, 2) {
        Val::Num(value) if value == 0.0 || value == 1.0 => value as u8,
        _ => {
            return Err(rilua::runtime_error(
                "FontStringScaleAnimationMode (0 or 1) expected",
            ));
        }
    };
    let mut sim = borrow_state_mut(state)?;
    let frame = sim
        .widgets
        .get_mut(id)
        .ok_or_else(|| rilua::runtime_error("FontString frame no longer exists"))?;
    frame.font_string_scale_animation_mode = mode;
    Ok(0)
}

#[cfg(feature = "retail-12-0-0")]
pub(crate) fn get_scale_animation_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mode = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .ok_or_else(|| rilua::runtime_error("FontString frame no longer exists"))?
            .font_string_scale_animation_mode
    };
    state.push(Val::Num(mode as f64));
    Ok(1)
}

pub(crate) fn get_text_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    state.push(Val::Num(frame_text_scale_value(state, id)));
    Ok(1)
}

pub(crate) fn set_text_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_scale = match stack_val(state, 2) {
        Val::Num(value) => value.max(0.0),
        _ => return Ok(0),
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.text_scale = text_scale;
    }
    Ok(0)
}
