//! `CreateSecureDelegate` and `securecopy`, backed by rilua's taint model.
//!
//! Contract: `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua`
//! (Retail/PTR/Forever: `CreateSecureDelegate` :113, `securecopy` :462,
//! `SecureCopyOptions` :589, `SecureDelegateOptions` :599; Mists/Era/
//! Anniversary: `CreateSecureDelegate` :93). Behaviour the documentation does
//! not state is best-guess; see `docs/specs/secure-helpers.md`.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::native_frame_id_from_val;
use crate::lua_api::taint::current_stack_taint;
use rilua::stdlib::taint::{call_with_stack_taint, get_closure_taint};
use rilua::vm::callinfo::LUA_MULTRET;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use std::collections::HashMap;

const DEFAULT_MAX_TRAVERSAL_DEPTH: f64 = 100.0;

/// Publish both globals on the profiles whose generated docs declare them.
///
/// Registers only missing globals: post-cleanup restoration calls this again
/// after `Blizzard_EnvironmentCleanup` nils `CreateSecureDelegate`.
pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Wrath {
        register_if_missing(lua, "CreateSecureDelegate", create_secure_delegate)?;
    }
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        register_if_missing(lua, "securecopy", securecopy)?;
    }
    Ok(())
}

fn register_if_missing(lua: &mut rilua::Lua, name: &str, func: rilua::RustFn) -> LuaResult<()> {
    if LuaApiMut::get_global_val(lua, name) == Val::Nil {
        LuaApiMut::register_function(lua, name, func)?;
    }
    Ok(())
}

fn arg(state: &LuaState, index: usize) -> Val {
    let slot = state.base + index;
    if slot < state.top {
        state.stack_get(slot)
    } else {
        Val::Nil
    }
}

fn option_field(state: &mut LuaState, options: Val, name: &str) -> Val {
    let Val::Table(table) = options else {
        return Val::Nil;
    };
    let key = state.gc.intern_string(name.as_bytes());
    state
        .gc
        .tables
        .get(table)
        .map_or(Val::Nil, |t| t.get_str(key, &state.gc.string_arena))
}

fn bool_option(state: &mut LuaState, options: Val, name: &str, default: bool) -> bool {
    match option_field(state, options, name) {
        Val::Nil => default,
        Val::Bool(value) => value,
        _ => true,
    }
}

fn check_options_type(options: Val, function: &str) -> LuaResult<()> {
    match options {
        Val::Nil | Val::Table(_) => Ok(()),
        other => Err(runtime_error(format!(
            "bad argument #2 to '{function}' (table expected, got {})",
            other.type_name()
        ))),
    }
}

fn new_rust_closure(
    state: &mut LuaState,
    func: rilua::RustFn,
    name: &str,
    upvalues: Vec<Val>,
) -> Val {
    let mut closure = RustClosure::new(func, name);
    closure.upvalues = upvalues;
    Val::Function(state.gc.alloc_closure(Closure::Rust(closure)))
}

fn current_upvalues(state: &LuaState) -> LuaResult<Vec<Val>> {
    let func = state.stack_get(state.call_stack[state.ci].func);
    if let Val::Function(func_ref) = func
        && let Some(Closure::Rust(closure)) = state.gc.closures.get(func_ref)
    {
        return Ok(closure.upvalues.clone());
    }
    Err(runtime_error(
        "secure helper closure is missing its upvalues",
    ))
}

/// Push `func` followed by this frame's arguments (mapped through `map_arg`)
/// and call it with `taint` as the only visible stack taint.
fn call_through(
    state: &mut LuaState,
    func: Val,
    taint: Option<String>,
    mut map_arg: impl FnMut(&mut LuaState, Val) -> Val,
) -> LuaResult<u32> {
    let nargs = state.top - state.base;
    let func_idx = state.top;
    state.push(func);
    for index in 0..nargs {
        let value = state.stack_get(state.base + index);
        let value = map_arg(state, value);
        state.push(value);
    }
    call_with_stack_taint(state, func_idx, LUA_MULTRET, taint)?;
    Ok((state.top - func_idx) as u32)
}

/// Untrusted = a function carrying a closure taint stamp.
fn wrap_if_untrusted(state: &mut LuaState, value: Val) -> Val {
    let Val::Function(func_ref) = value else {
        return value;
    };
    if get_closure_taint(state, func_ref).is_none() {
        return value;
    }
    new_rust_closure(
        state,
        call_with_original_taint,
        "SecureFunctionWrapper",
        vec![value],
    )
}

fn call_with_original_taint(state: &mut LuaState) -> LuaResult<u32> {
    let original = current_upvalues(state)?[0];
    let Val::Function(func_ref) = original else {
        return Err(runtime_error("secure function wrapper lost its function"));
    };
    let taint = get_closure_taint(state, func_ref);
    call_through(state, original, taint, |_, value| value)
}

fn create_secure_delegate(state: &mut LuaState) -> LuaResult<u32> {
    let func = arg(state, 0);
    if !matches!(func, Val::Function(_)) {
        return Err(runtime_error(format!(
            "bad argument #1 to 'CreateSecureDelegate' (function expected, got {})",
            func.type_name()
        )));
    }
    let options = arg(state, 1);
    check_options_type(options, "CreateSecureDelegate")?;
    let wrap = bool_option(state, options, "wrapUntrustedFunctions", true);
    let delegate = new_rust_closure(
        state,
        invoke_secure_delegate,
        "SecureDelegate",
        vec![func, Val::Bool(wrap)],
    );
    state.push(delegate);
    Ok(1)
}

fn invoke_secure_delegate(state: &mut LuaState) -> LuaResult<u32> {
    let upvalues = current_upvalues(state)?;
    let wrap = matches!(upvalues[1], Val::Bool(true));
    call_through(state, upvalues[0], None, |state, value| {
        if wrap {
            wrap_if_untrusted(state, value)
        } else {
            value
        }
    })
}

struct SecureCopy {
    max_depth: f64,
    wrap_functions: bool,
    permit_script_objects: bool,
    taint: Option<String>,
    copies: HashMap<GcRef<Table>, GcRef<Table>>,
    /// Rooted on the Lua stack; keeps every new copy reachable while
    /// interning or allocation may run the collector.
    anchor: GcRef<Table>,
    anchored: usize,
}

impl SecureCopy {
    fn anchor(&mut self, state: &mut LuaState, value: Val) -> LuaResult<()> {
        self.anchored += 1;
        let key = Val::Num(self.anchored as f64);
        if let Some(anchor) = state.gc.tables.get_mut(self.anchor) {
            anchor.raw_set(key, value, &state.gc.string_arena)?;
        }
        state.gc.barrier_back(self.anchor);
        Ok(())
    }

    fn copy_value(&mut self, state: &mut LuaState, value: Val, depth: f64) -> LuaResult<Val> {
        match value {
            Val::Table(table) => self.copy_table(state, value, table, depth),
            Val::Function(_) if self.wrap_functions => {
                let wrapped = wrap_if_untrusted(state, value);
                self.anchor(state, wrapped)?;
                Ok(wrapped)
            }
            other => Ok(other),
        }
    }

    fn copy_table(
        &mut self,
        state: &mut LuaState,
        value: Val,
        source: GcRef<Table>,
        depth: f64,
    ) -> LuaResult<Val> {
        if native_frame_id_from_val(state, value).is_some() {
            if self.permit_script_objects {
                return Ok(value);
            }
            return Err(runtime_error("securecopy: cannot copy a script object"));
        }
        if let Some(copy) = self.copies.get(&source) {
            return Ok(Val::Table(*copy));
        }
        if depth > self.max_depth {
            return Err(runtime_error(
                "securecopy: maximum traversal depth exceeded",
            ));
        }
        let copy = state.gc.alloc_table(Table::new());
        self.anchor(state, Val::Table(copy))?;
        self.copies.insert(source, copy);
        for (key, entry) in table_entries(state, source)? {
            let key = self.copy_value(state, key, depth + 1.0)?;
            let entry = self.copy_value(state, entry, depth + 1.0)?;
            self.store(state, copy, key, entry)?;
        }
        Ok(Val::Table(copy))
    }

    fn store(
        &self,
        state: &mut LuaState,
        table: GcRef<Table>,
        key: Val,
        value: Val,
    ) -> LuaResult<()> {
        let key_bytes = match key {
            Val::Str(s) => state.gc.string_arena.get(s).map(|s| s.data().to_vec()),
            _ => None,
        };
        let Some(target) = state.gc.tables.get_mut(table) else {
            return Err(runtime_error("securecopy: copy table was collected"));
        };
        target.raw_set(key, value, &state.gc.string_arena)?;
        // Slot taint exists only for string and integer keys in rilua.
        if let Some(taint) = self.taint.as_deref() {
            match (key, key_bytes) {
                (_, Some(bytes)) => target.set_slot_taint_str(&bytes, taint),
                (Val::Num(n), None) if n.fract() == 0.0 && n.is_finite() => {
                    target.set_slot_taint_int(n as i64, taint)
                }
                _ => {}
            }
        }
        state.gc.barrier_back(table);
        Ok(())
    }
}

fn table_entries(state: &LuaState, table: GcRef<Table>) -> LuaResult<Vec<(Val, Val)>> {
    let source = state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error("securecopy: source table was collected"))?;
    let mut entries = Vec::new();
    let mut key = Val::Nil;
    while let Some((next_key, value)) = source.next(key, &state.gc.string_arena)? {
        entries.push((next_key, value));
        key = next_key;
    }
    Ok(entries)
}

fn max_depth_option(state: &mut LuaState, options: Val) -> LuaResult<f64> {
    match option_field(state, options, "maxTraversalDepth") {
        Val::Nil => Ok(DEFAULT_MAX_TRAVERSAL_DEPTH),
        Val::Num(depth) => Ok(depth),
        other => Err(runtime_error(format!(
            "securecopy: maxTraversalDepth must be a number, got {}",
            other.type_name()
        ))),
    }
}

fn securecopy(state: &mut LuaState) -> LuaResult<u32> {
    let value = arg(state, 0);
    let options = arg(state, 1);
    check_options_type(options, "securecopy")?;
    let max_depth = max_depth_option(state, options)?;
    let wrap_functions = bool_option(state, options, "wrapUntrustedFunctions", true);
    let permit_script_objects = bool_option(state, options, "permitScriptObjects", false);
    let anchor = state.gc.alloc_table(Table::new());
    state.push(Val::Table(anchor));
    let mut copier = SecureCopy {
        max_depth,
        wrap_functions,
        permit_script_objects,
        taint: current_stack_taint(state),
        copies: HashMap::new(),
        anchor,
        anchored: 0,
    };
    let copy = copier.copy_value(state, value, 1.0)?;
    state.push(copy);
    Ok(1)
}
