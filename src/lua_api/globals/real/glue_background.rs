//! R52 Glue background lookup and bounded non-3D setter.
use crate::c_api::c_login::character_roster::slots_for_sim;
use crate::lua_api::methods::{borrow_state, table_set, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const GET_USAGE: &str = "Usage: GetSelectBackgroundModel(index)";
const SET_USAGE: &str = "Usage: SetCharSelectBackground(\"filename\" | file ID)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let glue = borrow_state(state)?.screen_kind.is_glue();
    for (name, function) in [
        ("GetSelectBackgroundModel", get_background as rilua::RustFn),
        ("SetCharSelectBackground", set_background as rilua::RustFn),
    ] {
        if glue {
            table_set_rust_fn_static(state, state.global, name, function)?;
        } else {
            table_set(state, Val::Table(state.global), name, Val::Nil);
        }
    }
    Ok(())
}

fn numeric_argument(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(value) => Some(value),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|text| crate::c_api::helpers::parse_numeric_string(text.data())),
        _ => None,
    }
}

fn bounded_integer(number: f64, function: &str) -> LuaResult<i32> {
    // Exact overflow diagnostics and nonfinite conversion are UNKNOWN.
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "{function}: numeric conversion outside simulator coverage (native behavior unknown)"
        )));
    }
    Ok(number.trunc() as i32)
}

fn get_background(state: &mut LuaState) -> LuaResult<u32> {
    let number =
        numeric_argument(state, stack_val(state, 1)).ok_or_else(|| runtime_error(GET_USAGE))?;
    let index = bounded_integer(number, "GetSelectBackgroundModel")?;
    let result = {
        let sim = borrow_state(state)?;
        let slots = slots_for_sim(&sim);
        let record = index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| slots.get(index))
            .and_then(|slot| slot.resolved_record());
        sim.glue_login.backgrounds.resolve(record)
    };
    state.push(
        result
            .map(|file| Val::Num(f64::from(file)))
            .unwrap_or(Val::Nil),
    );
    Ok(1)
}

fn set_background(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    if let Some(number) = numeric_argument(state, value) {
        let _file_bits = bounded_integer(number, "SetCharSelectBackground")? as u32;
    } else if matches!(value, Val::Str(_)) {
        // Empty strings return without application; nonempty names would resolve
        // into the omitted model path. The resolver policy is UNKNOWN.
        let _filename = val_to_string(state, value);
    } else {
        return Err(runtime_error(SET_USAGE));
    }
    // Do not retain a fake visual selection, change the roster override or emit
    // Forever's readiness event: its payload/timing/count remain UNKNOWN.
    Ok(0)
}
