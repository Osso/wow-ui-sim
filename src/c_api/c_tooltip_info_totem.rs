//! WM94 sections 3/9: stored totem name and native-duration-style time text.

use crate::c_api::helpers::global_val;
use crate::lua_api::globals::real::totems::Totem;
use crate::lua_api::methods::{borrow_state, call_function_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::table_security::unwrap_secret;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetTotem", get_totem)
}

fn get_totem(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let (totem, now_ms) = {
        let sim = borrow_state(state)?;
        let totem = slot
            .and_then(|slot| sim.totem_slots.get(slot))
            .and_then(Option::as_ref)
            .cloned();
        (totem, sim.cached_clock_ms())
    };
    let Some(totem) = totem else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let remaining_ms = remaining_milliseconds(&totem, now_ms)?;
    let remaining = format_remaining_time(state, remaining_ms)?;
    let data = crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_totem_text(
        state,
        &totem.name,
        &remaining,
    )?;
    state.push(data);
    Ok(1)
}

fn read_slot(state: &LuaState) -> LuaResult<Option<usize>> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    if let rilua::Val::Num(number) = value {
        if number.is_finite()
            && number.fract() == 0.0
            && (0.0..=f64::from(u32::MAX)).contains(&number)
        {
            return Ok((number as u32)
                .checked_sub(1)
                .filter(|index| *index < 8)
                .map(|index| index as usize));
        }
    }
    Err(rilua::runtime_error(
        "GetTotem requires an unsigned integer slot",
    ))
}

fn remaining_milliseconds(totem: &Totem, now_ms: u32) -> LuaResult<u32> {
    let rate = totem.mod_rate as f32;
    if !rate.is_finite() || rate <= 0.0 {
        return Err(rilua::runtime_error(
            "GetTotem: supplied mod_rate must be finite and positive; native invalid-rate behavior is unknown",
        ));
    }
    // Host state exposes seconds; native storage is unsigned milliseconds.
    let start = emulated_nearest_even_milliseconds(totem.start_time * 1000.0)?;
    let duration = emulated_nearest_even_milliseconds(totem.duration * 1000.0)?;
    let end = start.wrapping_add(duration);
    if end == 0 || now_ms.wrapping_sub(end) as i32 >= 0 {
        return Ok(0);
    }
    let remaining = end.wrapping_sub(now_ms) as f32 / rate;
    emulated_nearest_even_milliseconds(f64::from(remaining))
}

/// EMULATED: nearest-even is the inferred default FP control, not live-state proof.
/// Also converts host-supplied seconds to the native millisecond storage domain.
/// Unsupported host inputs fail explicitly rather than clamp or become expiry.
fn emulated_nearest_even_milliseconds(value: f64) -> LuaResult<u32> {
    let rounded = value.round_ties_even();
    if !value.is_finite() || value < 0.0 || rounded > f64::from(u32::MAX) {
        return Err(rilua::runtime_error(
            "GetTotem: supplied time is outside the modeled unsigned millisecond domain",
        ));
    }
    Ok(rounded as u32)
}

fn select_remaining_unit(milliseconds: u32) -> (&'static str, u32) {
    let seconds = milliseconds.div_ceil(1000);
    match seconds {
        129601.. => ("SPELL_TIME_REMAINING_DAYS", seconds.div_ceil(86400)),
        5401.. => ("SPELL_TIME_REMAINING_HOURS", seconds.div_ceil(3600)),
        91.. => ("SPELL_TIME_REMAINING_MIN", seconds.div_ceil(60)),
        _ => ("SPELL_TIME_REMAINING_SEC", seconds),
    }
}

/// Localized duration text shared with the brawl tooltip's EMULATED ends-in
/// row; the unit selection and upward rounding are WM94 §9's Totem contract.
pub(crate) fn format_remaining_time(state: &mut LuaState, milliseconds: u32) -> LuaResult<String> {
    let (key, count) = select_remaining_unit(milliseconds);
    let template = global_val(state, key);
    let formatter = global_val(state, "format");
    let value = call_function_state(state, formatter, &[template, rilua::Val::Num(count.into())])?;
    val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error("GetTotem: localized formatter did not return text"))
}
