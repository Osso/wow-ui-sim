//! Master-era global compatibility overrides — rilua port of the five
//! `register_*` fns that used to live in `src/lua_api/globals_legacy.rs`.
//!
//! What gets overridden and why:
//!
//! - **`print`** — intercepted so Lua `print(...)` writes to
//!   `SimState.console_output` (the GUI log panel) and `eprintln!`s a
//!   copy to stderr. The native print silently drops output.
//! - **`A_Print`** — secure alias stored in the registry under
//!   `__sim_print`. Lua wrapper so calls from tainted code still land
//!   in the log panel. `self_test.rs` relies on this symbol (~7 sites).
//! - **`next`** — routed through the native rilua implementation so
//!   table-backed frame refs expose their real per-frame slots. Retail
//!   frames are iterable table-like values: `DevTools_Dump(frame)` can see
//!   array entries as well as frame identity slot `[0]`.
//! - **`ipairs`** — `ipairs(frame)` returns a children iterator that
//!   yields `(i, childFrame)` pairs in insertion order. Preserves the
//!   master-era affordance used by a handful of Blizzard UI paths.
//! - **`getmetatable` / `setmetatable`** — NOT intercepted; rilua's
//!   native userdata metatable handling already supports the
//!   `setmetatable(frame, {__index = Mixin})` pattern and the per-type
//!   metatable cloning from the mlua era is no longer required (the
//!   index helper is registered once per FrameRef type in rilua).
//!   Verified by the `frame_mixin_index` test below.

use crate::lua_api::SimState;
use crate::lua_api::methods::{
    borrow_state_mut, extract_frame_id, frame_ref, registry_get, registry_set, state_handle,
    table_get_static, val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::stdlib::base::{lua_ipairs, lua_next};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use std::cell::RefCell;
use std::rc::Rc;

const SIM_PRINT_KEY: &str = "__sim_print";
const ORIGINAL_NEXT_KEY: &str = "__original_next";
const ORIGINAL_IPAIRS_KEY: &str = "__original_ipairs";
const FRAME_CHILDREN_ITER_KEY: &str = "__frame_children_iter";

const A_PRINT_LUA: &str = r#"
function A_Print(...)
    local p = debug.getregistry().__sim_print
    if p then p(...) end
end
"#;

const LEGACY_ALIAS_LUA: &str = r#"
if abs == nil and math ~= nil then abs = math.abs end
if ceil == nil and math ~= nil then ceil = math.ceil end
if floor == nil and math ~= nil then floor = math.floor end
if max == nil and math ~= nil then max = math.max end
if min == nil and math ~= nil then min = math.min end
if strlen == nil and string ~= nil then strlen = string.len end
if sort == nil and table ~= nil then sort = table.sort end

if strsplittable == nil then
  function strsplittable(delimiter, input, limit)
    return { strsplit(delimiter, input, limit) }
  end
end

if MergeTable == nil then
  function MergeTable(dest, src)
    if type(dest) ~= "table" or type(src) ~= "table" then
      return dest
    end
    for key, value in pairs(src) do
      dest[key] = value
    end
    return dest
  end
end

if tFilter == nil then
  function tFilter(t, predicate)
    if type(t) ~= "table" or type(predicate) ~= "function" then
      return t
    end
    local out = 1
    for index = 1, #t do
      local value = t[index]
      if value ~= nil and predicate(value, index, t) then
        if out ~= index then
          t[out] = value
        end
        out = out + 1
      end
    end
    for index = out, #t do
      t[index] = nil
    end
    return t
  end
end

if string ~= nil and string.split == nil then
  function string.split(self, delimiter, limit)
    if type(self) == "string" and type(delimiter) == "string" then
      local inputIsEmpty = #delimiter == 0
      local inputIsLonger = #self <= 4 and #delimiter > #self
      local receiverLooksLikeDelimiter = #self <= 4 and self:match("^%W+$") ~= nil
      if inputIsEmpty or inputIsLonger or receiverLooksLikeDelimiter then
        return strsplit(self, delimiter, limit)
      end
    end
    return strsplit(delimiter, self, limit)
  end
end

if bit == nil then
  local UINT32_MODULO = 2 ^ 32
  local UINT32_MASK = UINT32_MODULO - 1

  local function normalize(v)
    v = math.floor(tonumber(v) or 0)
    if v < 0 then
      v = UINT32_MODULO + v
    end
    return v % UINT32_MODULO
  end

  local function fold(values, identity, step)
    local result = identity
    for i = 1, #values do
      result = step(result, normalize(values[i]))
    end
    return normalize(result)
  end

  local function lshift(a, n)
    return normalize(normalize(a) * (2 ^ normalize(n)))
  end

  local function rshift(a, n)
    return math.floor(normalize(a) / (2 ^ normalize(n)))
  end

  local function band2(a, b)
    local result = 0
    local bitValue = 1
    a = normalize(a)
    b = normalize(b)
    while a > 0 or b > 0 do
      local abit = a % 2
      local bbit = b % 2
      if abit == 1 and bbit == 1 then
        result = result + bitValue
      end
      a = math.floor(a / 2)
      b = math.floor(b / 2)
      bitValue = bitValue * 2
    end
    return result
  end

  local function bor2(a, b)
    local result = 0
    local bitValue = 1
    a = normalize(a)
    b = normalize(b)
    while a > 0 or b > 0 do
      local abit = a % 2
      local bbit = b % 2
      if abit == 1 or bbit == 1 then
        result = result + bitValue
      end
      a = math.floor(a / 2)
      b = math.floor(b / 2)
      bitValue = bitValue * 2
    end
    return result
  end

  bit = {
    band = function(...)
      return fold({...}, UINT32_MASK, band2)
    end,
    bor = function(...)
      return fold({...}, 0, bor2)
    end,
    bxor = function(a, b)
      a = normalize(a)
      b = normalize(b)
      local result = 0
      local bitValue = 1
      while a > 0 or b > 0 do
        local abit = a % 2
        local bbit = b % 2
        if abit ~= bbit then
          result = result + bitValue
        end
        a = math.floor(a / 2)
        b = math.floor(b / 2)
        bitValue = bitValue * 2
      end
      return result
    end,
    bnot = function(a)
      return UINT32_MASK - normalize(a)
    end,
    lshift = lshift,
    rshift = rshift,
    arshift = rshift,
    mod = function(a, b)
      -- A divisor that truncates to zero returns 1 / divisor (3.3.5a lbitlib;
      -- Retail 12.1.5 observation in Meorawr/elune#12).
      local divisor = tonumber(b)
      if divisor ~= nil and divisor > -1 and divisor < 1 then
        return 1 / divisor
      end
      return normalize(a) % normalize(b)
    end,
  }
end
"#;

/// Install `print` / `A_Print` / `next` / `ipairs` overrides on the rilua
/// VM. Idempotent — safe to call from `register_globals` even if the
/// registrar runs more than once.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    install_print(lua)?;
    install_addframetext(lua)?;
    install_a_print(lua)?;
    install_dumpobject(lua)?;
    install_legacy_aliases(lua)?;
    install_nil_symbol_logger(lua)?;
    install_next(lua)?;
    install_ipairs(lua)?;
    Ok(())
}

// ── print + A_Print ──────────────────────────────────────────────────────────

fn install_print(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "print", sim_print)?;
    let state = lua.state_mut();
    let print_val = {
        let key = state.gc.intern_string_static(b"print");
        state
            .gc
            .tables
            .get(state.global)
            .map(|t| t.get_str(key, &state.gc.string_arena))
            .unwrap_or(Val::Nil)
    };
    registry_set(state, SIM_PRINT_KEY, print_val);
    Ok(())
}

fn install_addframetext(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "addframetext", sim_addframetext)?;
    Ok(())
}

fn install_a_print(lua: &mut rilua::Lua) -> LuaResult<()> {
    lua.exec(A_PRINT_LUA)
        .map_err(|e| runtime_error(format!("A_Print install: {e}")))?;
    Ok(())
}

fn install_dumpobject(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "dumpobject", dumpobject)?;
    Ok(())
}

fn dumpobject(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn install_legacy_aliases(lua: &mut rilua::Lua) -> LuaResult<()> {
    lua.exec(LEGACY_ALIAS_LUA)
        .map_err(|e| runtime_error(format!("legacy alias install: {e}")))?;
    Ok(())
}

fn install_nil_symbol_logger(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "__wow_record_nil_symbol_access",
        record_nil_symbol_access,
    )?;
    LuaApiMut::register_function(
        lua,
        "__wow_record_public_global_publication",
        record_public_global_publication_callback,
    )?;
    LuaApiMut::register_function(
        lua,
        "__wow_record_secure_global_publication",
        record_secure_global_publication_callback,
    )?;
    Ok(())
}

fn record_nil_symbol_access(state: &mut LuaState) -> LuaResult<u32> {
    let container = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    let key = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let source = val_to_string(state, stack_val(state, 3));
    let line = match stack_val(state, 4) {
        Val::Num(value) if value.is_finite() => Some(value as i32),
        _ => None,
    };
    let mut sim = borrow_state_mut(state)?;
    let addon_index = sim.loading_addon_index.or(sim.executing_addon_index);
    let environment = sim.loading_nil_symbol_environment.unwrap_or_else(|| {
        if container == "__secureenv" {
            crate::lua_api::state::NilSymbolEnvironment::Secure
        } else {
            crate::lua_api::state::NilSymbolEnvironment::Public
        }
    });
    let addon_name = addon_index
        .and_then(|index| sim.addons.get(index as usize))
        .map(|addon| addon.folder_name.clone());
    sim.nil_symbol_accesses
        .push(crate::lua_api::state::NilSymbolAccess {
            addon_index,
            addon_name,
            environment,
            container,
            key,
            source,
            line,
        });
    Ok(0)
}

fn record_public_global_publication_callback(state: &mut LuaState) -> LuaResult<u32> {
    let name = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    record_public_global_publication(state, &name)?;
    Ok(0)
}

fn record_secure_global_publication_callback(state: &mut LuaState) -> LuaResult<u32> {
    let name = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    record_secure_global_publication(state, &name)?;
    Ok(0)
}

pub(crate) fn record_public_global_publication(state: &mut LuaState, name: &str) -> LuaResult<()> {
    record_global_publication(
        state,
        name,
        crate::lua_api::state::NilSymbolEnvironment::Public,
    )
}

pub(crate) fn record_secure_global_publication(state: &mut LuaState, name: &str) -> LuaResult<()> {
    record_global_publication(
        state,
        name,
        crate::lua_api::state::NilSymbolEnvironment::Secure,
    )
}

fn record_global_publication(
    state: &mut LuaState,
    name: &str,
    environment: crate::lua_api::state::NilSymbolEnvironment,
) -> LuaResult<()> {
    if name.is_empty() || name.starts_with("C_") {
        return Ok(());
    }

    let mut sim = borrow_state_mut(state)?;
    let Some(addon_index) = sim.loading_addon_index else {
        return Ok(());
    };
    let publications = match environment {
        crate::lua_api::state::NilSymbolEnvironment::Public => &mut sim.global_publications,
        crate::lua_api::state::NilSymbolEnvironment::Secure => &mut sim.secure_global_publications,
    };
    publications.insert((addon_index, name.to_string()));
    Ok(())
}

fn sim_print(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = (state.top as i32 - state.base as i32).max(0) as usize;
    let mut output = String::new();
    for i in 0..nargs {
        if i > 0 {
            output.push('\t');
        }
        append_val(state, stack_val(state, (i + 1) as i32), &mut output);
    }
    eprintln!("{output}");
    if let Ok(sim) = state_handle(state) {
        write_console_line(&sim, output);
    }
    Ok(0)
}

fn sim_addframetext(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    if matches!(value, Val::Nil) {
        return Ok(0);
    }
    let mut rendered = String::new();
    append_val(state, value, &mut rendered);
    crate::lua_api::script_helpers::report_addframetext_error(state, &rendered);
    Ok(0)
}

fn write_console_line(sim: &Rc<RefCell<SimState>>, line: String) {
    sim.borrow_mut().console_output.push(line);
}

fn append_val(state: &LuaState, val: Val, out: &mut String) {
    match val {
        Val::Nil => out.push_str("nil"),
        Val::Bool(b) => out.push_str(if b { "true" } else { "false" }),
        Val::Num(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                out.push_str(&format!("{}", n as i64));
            } else {
                out.push_str(&format!("{n}"));
            }
        }
        Val::Str(s) => {
            if let Some(bytes) = state.gc.string_arena.get(s).map(|ls| ls.data().to_vec()) {
                out.push_str(&String::from_utf8_lossy(&bytes));
            }
        }
        Val::Table(_) => out.push_str("table"),
        Val::Function(_) => out.push_str("function"),
        Val::Userdata(_) | Val::LightUserdata(_) => out.push_str("userdata"),
        Val::Thread(_) => out.push_str("thread"),
    }
}

// ── next(...) compatibility ──────────────────────────────────────────────────

fn install_next(lua: &mut rilua::Lua) -> LuaResult<()> {
    let existing = registry_get(lua.state_mut(), ORIGINAL_NEXT_KEY);
    if !matches!(existing, Val::Function(_)) {
        let original = LuaApiMut::get_global_val(lua, "next");
        if !matches!(original, Val::Function(_)) {
            return Err(runtime_error("next missing"));
        }
        registry_set(lua.state_mut(), ORIGINAL_NEXT_KEY, original);
    }
    LuaApiMut::register_function(lua, "next", custom_next)?;
    Ok(())
}

fn custom_next(state: &mut LuaState) -> LuaResult<u32> {
    lua_next(state)
}

// ── ipairs(frame) children iterator ──────────────────────────────────────────

fn install_ipairs(lua: &mut rilua::Lua) -> LuaResult<()> {
    let existing = registry_get(lua.state_mut(), ORIGINAL_IPAIRS_KEY);
    if !matches!(existing, Val::Function(_)) {
        let original = LuaApiMut::get_global_val(lua, "ipairs");
        if !matches!(original, Val::Function(_)) {
            return Err(runtime_error("ipairs missing"));
        }
        registry_set(lua.state_mut(), ORIGINAL_IPAIRS_KEY, original);
    }

    // Register a Rust iterator body once and stash it in the registry so
    // the dispatch custom_ipairs can return it without re-registering on
    // every call.
    LuaApiMut::register_function(lua, "__rilua_frame_children_iter", frame_children_iter)?;
    let iter_fn = LuaApiMut::get_global_val(lua, "__rilua_frame_children_iter");
    registry_set(lua.state_mut(), FRAME_CHILDREN_ITER_KEY, iter_fn);
    // Clear the public name so addons can't shadow the iterator.
    LuaApiMut::set_global_val(lua, "__rilua_frame_children_iter", Val::Nil)?;

    LuaApiMut::register_function(lua, "ipairs", custom_ipairs)?;
    Ok(())
}

fn custom_ipairs(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    if let Some(frame_id) = extract_frame_id(state, value) {
        let iter_fn = registry_get(state, FRAME_CHILDREN_ITER_KEY);
        let frame_val = frame_ref(state, frame_id)?;
        // ipairs contract: iter_fn, state, control(0)
        state.push(iter_fn);
        state.push(frame_val);
        state.push(Val::Num(0.0));
        return Ok(3);
    }
    if !matches!(value, Val::Table(_)) {
        let site = describe_ipairs_callsite(state);
        let value_type = value.type_name();
        return Err(runtime_error(format!(
            "bad argument #1 to 'ipairs' (table expected, got {value_type}){site}"
        )));
    }
    lua_ipairs(state)
}

fn describe_ipairs_callsite(state: &mut LuaState) -> String {
    let debug = table_get_static(state, Val::Table(state.global), "debug");
    let getinfo = table_get_static(state, debug, "getinfo");
    let Val::Function(_) = getinfo else {
        return String::new();
    };
    let what = Val::Str(state.gc.intern_string_static(b"Sl"));
    for level in [2.0, 3.0, 4.0] {
        let Ok(results) = crate::lua_api::script_helpers::protected_call_state(
            state,
            getinfo,
            &[Val::Num(level), what],
        ) else {
            continue;
        };
        let Some(info @ Val::Table(_)) = results.first().copied() else {
            continue;
        };
        let src = table_get_static(state, info, "short_src");
        let line = table_get_static(state, info, "currentline");
        let (Val::Str(src_ref), Val::Num(line_no)) = (src, line) else {
            continue;
        };
        if let Some(src) = state.gc.string_arena.get(src_ref) {
            return format!(
                " at {}:{}",
                String::from_utf8_lossy(src.data()),
                line_no as i64
            );
        }
    }
    String::new()
}

/// Iterator body for `ipairs(frame)`. Called as `iter(state, control)`
/// where `state` is the frame ref and `control` is the previous index.
/// Returns `(next_index, child_frame)` or terminator `nil`.
fn frame_children_iter(state: &mut LuaState) -> LuaResult<u32> {
    let iter_state = stack_val(state, 1);
    let control = stack_val(state, 2);
    let Some(frame_id) = extract_frame_id(state, iter_state) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let prev_idx = match control {
        Val::Num(n) => n as i64,
        _ => 0,
    };
    let next_idx = (prev_idx + 1) as usize;
    let child_id = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        sim.widgets
            .get(frame_id)
            .and_then(|f| f.children.get(next_idx - 1).copied())
    };
    let Some(child_id) = child_id else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let child_val = frame_ref(state, child_id)?;
    state.push(Val::Num(next_idx as f64));
    state.push(child_val);
    Ok(2)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_legacy_math_and_table_aliases() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: String = env
            .eval(
                r#"
                local values = { 3, 1, 2 }
                sort(values)

                if abs(-4) ~= 4 then return "bad_abs" end
                if ceil(1.2) ~= 2 then return "bad_ceil" end
                if floor(1.8) ~= 1 then return "bad_floor" end
                if max(1, 3, 2) ~= 3 then return "bad_max" end
                if min(1, 3, 2) ~= 1 then return "bad_min" end
                if strlen("abcd") ~= 4 then return "bad_strlen" end
                if values[1] ~= 1 or values[2] ~= 2 or values[3] ~= 3 then
                    return "bad_sort"
                end

                return "ok"
                "#,
            )
            .expect("legacy alias probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_legacy_bit_library() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: String = env
            .eval(
                r#"
                if bit.band(12, 10) ~= 8 then return "band" end
                if bit.bor(12, 10) ~= 14 then return "bor" end
                if bit.bxor(12, 10) ~= 6 then return "bxor" end
                if bit.bnot(0) ~= 4294967295 then return "bnot" end
                if bit.lshift(1, 3) ~= 8 then return "lshift" end
                if bit.rshift(16, 2) ~= 4 then return "rshift" end
                if bit.arshift(16, 2) ~= 4 then return "arshift" end
                if bit.mod(17, 5) ~= 2 then return "mod" end
                if bit.mod(5, 0.5) ~= 2 then return "mod_half" end
                if bit.mod(5, 0) ~= math.huge then return "mod_zero" end
                if bit.mod(5, -0.5) ~= -2 then return "mod_negative_half" end
                return "ok"
                "#,
            )
            .expect("bit compatibility probe should run");

        assert_eq!(result, "ok");
    }
}
