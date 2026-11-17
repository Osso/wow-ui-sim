//! Native `bit` library.
//!
//! rilua ships no `bit` library, so the simulator used to install a Lua
//! fallback in `compat_overrides.rs`. Questie's login coroutine spent about
//! 96% of its sampled instructions inside that fallback's `bit.band` loop
//! (handoff `questie-login-perf`). These Rust functions return the same values
//! for every input:
//!
//! - an argument converts as `tonumber(v) or 0`, then floors and wraps into
//!   `[0, 2^32)` with Lua's `%` (`fma(floor(a / b), -b, a)` in rilua);
//! - `band` and `bor` fold the arguments up to the `#{...}` border, so a
//!   trailing nil hole ends the fold where the old table length did;
//! - `^` is `powf`, and every other step uses the same f64 operations in the
//!   same order as the old Lua.
//!
//! The client ships `bit` as native code. Its argument errors for non-number
//! input are UNVERIFIED and are not modelled here.

use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const UINT32_MODULO: f64 = 4_294_967_296.0;
const UINT32_MASK: f64 = UINT32_MODULO - 1.0;

/// Install `bit` unless a global of that name already exists.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    if !matches!(crate::c_api::global_val(state, "bit"), Val::Nil) {
        return Ok(());
    }
    let table = state.gc.alloc_table(rilua::vm::table::Table::new());
    // Rooted in `_G` before the closure allocations below.
    crate::c_api::set_global_val(state, "bit", Val::Table(table));
    table_set_rust_fn_static(state, table, "band", band)?;
    table_set_rust_fn_static(state, table, "bor", bor)?;
    table_set_rust_fn_static(state, table, "bxor", bxor)?;
    table_set_rust_fn_static(state, table, "bnot", bnot)?;
    table_set_rust_fn_static(state, table, "lshift", lshift)?;
    table_set_rust_fn_static(state, table, "rshift", rshift)?;
    table_set_rust_fn_static(state, table, "mod", modulo)?;
    alias_arshift(state, table)
}

/// The old table assigned `arshift = rshift`, so both keys hold one function.
fn alias_arshift(
    state: &mut LuaState,
    table: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    let rshift_key = state.gc.intern_string_static(b"rshift");
    let arshift_key = state.gc.intern_string_static(b"arshift");
    let strings = &state.gc.string_arena;
    let Some(bit) = state.gc.tables.get_mut(table) else {
        return Err(rilua::runtime_error("bit table has been collected"));
    };
    let rshift = bit.get_str(rshift_key, strings);
    bit.raw_set(Val::Str(arshift_key), rshift, strings)?;
    state.gc.barrier_back(table);
    Ok(())
}

/// Lua 5.1 `a % b` as rilua evaluates it.
fn lua_mod(a: f64, b: f64) -> f64 {
    (a / b).floor().mul_add(-b, a)
}

/// The old `normalize` after its `tonumber(v) or 0` step.
fn wrap_u32(value: f64) -> f64 {
    let mut value = value.floor();
    if value < 0.0 {
        value += UINT32_MODULO;
    }
    lua_mod(value, UINT32_MODULO)
}

/// `tonumber(v)` with one argument: numbers, numeric strings, otherwise nil.
fn to_number(state: &mut LuaState, value: Val) -> LuaResult<Option<f64>> {
    match value {
        Val::Num(number) => Ok(Some(number)),
        Val::Str(_) => crate::lua_api::wrath_arguments::convert_number(state, value),
        _ => Ok(None),
    }
}

fn normalized_arg(state: &mut LuaState, index: usize) -> LuaResult<f64> {
    let value = stack_val(state, index as i32 + 1);
    Ok(wrap_u32(to_number(state, value)?.unwrap_or(0.0)))
}

/// `#{...}` for the call's arguments: rilua sizes that array part to the
/// argument count and binary-searches a border when its last slot is nil.
fn argument_border(state: &LuaState) -> usize {
    let count = state.top.saturating_sub(state.base);
    let is_nil = |index: usize| stack_val(state, index as i32 + 1).is_nil();
    if count == 0 || !is_nil(count - 1) {
        return count;
    }
    let (mut low, mut high) = (0, count);
    while high - low > 1 {
        let middle = usize::midpoint(low, high);
        if is_nil(middle - 1) {
            high = middle;
        } else {
            low = middle;
        }
    }
    low
}

/// The old bit-by-bit loop, with `keep` choosing which bit pairs set a bit.
fn bit_loop(mut a: f64, mut b: f64, keep: fn(f64, f64) -> bool) -> f64 {
    let (mut result, mut bit_value) = (0.0, 1.0);
    while a > 0.0 || b > 0.0 {
        if keep(lua_mod(a, 2.0), lua_mod(b, 2.0)) {
            result += bit_value;
        }
        a = (a / 2.0).floor();
        b = (b / 2.0).floor();
        bit_value *= 2.0;
    }
    result
}

fn both_set(a: f64, b: f64) -> bool {
    a == 1.0 && b == 1.0
}

fn either_set(a: f64, b: f64) -> bool {
    a == 1.0 || b == 1.0
}

fn differ(a: f64, b: f64) -> bool {
    a != b
}

fn fold(state: &mut LuaState, identity: f64, keep: fn(f64, f64) -> bool) -> LuaResult<u32> {
    let mut result = identity;
    for index in 0..argument_border(state) {
        let value = normalized_arg(state, index)?;
        result = bit_loop(wrap_u32(result), wrap_u32(value), keep);
    }
    push_number(state, wrap_u32(result))
}

fn push_number(state: &mut LuaState, value: f64) -> LuaResult<u32> {
    state.push(Val::Num(value));
    Ok(1)
}

fn band(state: &mut LuaState) -> LuaResult<u32> {
    fold(state, UINT32_MASK, both_set)
}

fn bor(state: &mut LuaState) -> LuaResult<u32> {
    fold(state, 0.0, either_set)
}

fn bxor(state: &mut LuaState) -> LuaResult<u32> {
    let a = normalized_arg(state, 0)?;
    let b = normalized_arg(state, 1)?;
    push_number(state, bit_loop(a, b, differ))
}

fn bnot(state: &mut LuaState) -> LuaResult<u32> {
    let a = normalized_arg(state, 0)?;
    push_number(state, UINT32_MASK - a)
}

fn lshift(state: &mut LuaState) -> LuaResult<u32> {
    let a = normalized_arg(state, 0)?;
    let n = normalized_arg(state, 1)?;
    push_number(state, wrap_u32(a * 2.0_f64.powf(n)))
}

fn rshift(state: &mut LuaState) -> LuaResult<u32> {
    let a = normalized_arg(state, 0)?;
    let n = normalized_arg(state, 1)?;
    push_number(state, (a / 2.0_f64.powf(n)).floor())
}

/// A divisor that truncates to zero returns `1 / divisor` (3.3.5a lbitlib;
/// Retail 12.1.5 observation in Meorawr/elune#12).
fn modulo(state: &mut LuaState) -> LuaResult<u32> {
    let divisor = to_number(state, stack_val(state, 2))?;
    if let Some(divisor) = divisor.filter(|divisor| *divisor > -1.0 && *divisor < 1.0) {
        return push_number(state, 1.0 / divisor);
    }
    let a = normalized_arg(state, 0)?;
    let b = normalized_arg(state, 1)?;
    push_number(state, lua_mod(a, b))
}
