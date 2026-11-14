use crate::lua_bridge::{table_set_rust_fn, table_set_rust_fn_unpaired};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::ffi::{CStr, CString};

pub(crate) fn parse_numeric_string(bytes: &[u8]) -> Option<f64> {
    let text = CString::new(bytes).ok()?;
    let mut end = std::ptr::null_mut();
    // strtod reads the terminated CString and returns a pointer within it.
    let number = unsafe { libc::strtod(text.as_ptr(), &mut end) };
    if end == text.as_ptr().cast_mut() {
        return None;
    }
    let remaining = unsafe { CStr::from_ptr(end) }.to_bytes();
    remaining
        .iter()
        .all(u8::is_ascii_whitespace)
        .then_some(number)
}

pub(crate) fn bad_argument(index: i32, function: &str, reason: &str) -> rilua::LuaError {
    rilua::runtime_error(format!("bad argument #{index} to '{function}' ({reason})"))
}

fn read_untainted_value(
    state: &LuaState,
    index: i32,
    function: &str,
    usage: &str,
) -> LuaResult<(Val, bool)> {
    let original = crate::lua_bridge::stack_val(state, index);
    if matches!(original, Val::Nil) {
        return Err(bad_argument(index, function, usage));
    }
    let marked = rilua::table_security::is_secret_value(state, original);
    let value = rilua::table_security::unwrap_secret(state, original).map_err(|_| {
        let denied = format!(
            "{usage}. Secret values are only allowed during untainted execution for this argument."
        );
        bad_argument(index, function, &denied)
    })?;
    Ok((value, marked))
}

/// Read one required, untainted value using the shared argument-error format.
/// The signed-32 and truthiness converters use this same absence and access path.
pub(crate) fn read_untainted_signed32_argument(
    state: &LuaState,
    index: i32,
    function: &str,
    usage: &str,
) -> LuaResult<i32> {
    let (value, marked) = read_untainted_value(state, index, function, usage)?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) if !marked => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| parse_numeric_string(string.data())),
        _ => None,
    };
    let in_range = |number: &f64| (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(number);
    let range_error = || {
        let range = format!(
            "outside of expected range {} to {} - {usage}",
            i32::MIN,
            i32::MAX
        );
        bad_argument(index, function, &range)
    };
    number
        .filter(in_range)
        .map(|number| number.trunc() as i32)
        .ok_or_else(range_error)
}

#[cfg(feature = "client-retail")]
pub(crate) fn read_untainted_truthy_argument(
    state: &LuaState,
    index: i32,
    function: &str,
    usage: &str,
) -> LuaResult<bool> {
    let (value, _) = read_untainted_value(state, index, function, usage)?;
    if matches!(value, Val::Nil) {
        return Err(bad_argument(index, function, usage));
    }
    Ok(!matches!(value, Val::Bool(false)))
}

/// Reject nonnumeric inputs; unrepresented numeric identities cannot match supplied rows.
pub(crate) fn validate_supplied_relation_id(
    value: Val,
    error_message: &'static str,
) -> LuaResult<Option<i32>> {
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(error_message));
    };
    let in_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number);
    let represented = number.is_finite() && number.fract() == 0.0 && in_range;
    Ok(represented.then(|| number as i32))
}

pub(crate) fn set_global_val(state: &mut LuaState, name: &str, value: Val) {
    let key = state.gc.intern_string(name.as_bytes());
    let global = state.global;
    if let Some(table) = state.gc.tables.get_mut(global) {
        let _ = table.raw_set(Val::Str(key), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
}

pub(crate) fn global_val(state: &mut LuaState, name: &str) -> Val {
    let key = state.gc.intern_string(name.as_bytes());
    state
        .gc
        .tables
        .get(state.global)
        .map(|table| table.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

pub(crate) fn ensure_global_table(state: &mut LuaState, name: &str) -> Val {
    match global_val(state, name) {
        table @ Val::Table(_) => table,
        _ => {
            let table = crate::lua_api::methods::create_table(state);
            set_global_val(state, name, table);
            table
        }
    }
}

pub(crate) fn ensure_enum_table(state: &mut LuaState) -> Val {
    let (ordinary, _) = ensure_namespace_pair(state, "Enum")
        .expect("secure environment must exist before Enum registration");
    Val::Table(ordinary)
}

pub(crate) fn ensure_namespace(
    state: &mut LuaState,
    name: &'static str,
) -> LuaResult<GcRef<Table>> {
    ensure_namespace_pair(state, name).map(|(ordinary, _)| ordinary)
}

pub(crate) fn register_namespace_function_with_flags(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    function: rilua::RustFn,
    flags: u8,
) -> LuaResult<()> {
    let (publish_global, publish_secure) =
        crate::lua_api::globals::security::native_registration_destinations(flags);
    let global_parent = state.global;
    let ordinary = publish_global.then(|| ensure_table_at_parent(state, global_parent, namespace));
    let secure_parent = if publish_secure {
        match crate::lua_api::methods::registry_get(state, "__secureenv") {
            Val::Table(parent) => Some(parent),
            _ => None,
        }
    } else {
        None
    };
    let secure = secure_parent.map(|parent| ensure_table_at_parent(state, parent, namespace));

    if let (Some(ordinary), Some(secure)) = (ordinary, secure) {
        crate::lua_bridge::set_namespace_peer(state, ordinary, secure)?;
    }
    match (ordinary, secure) {
        (Some(ordinary), Some(secure)) => {
            table_set_rust_fn_unpaired(state, ordinary, member, function)?;
            let value = crate::lua_api::methods::table_get(state, Val::Table(ordinary), member);
            crate::lua_api::methods::table_set(state, Val::Table(secure), member, value);
        }
        (Some(ordinary), None) => {
            table_set_rust_fn_unpaired(state, ordinary, member, function)?;
        }
        (None, Some(secure)) => {
            table_set_rust_fn_unpaired(state, secure, member, function)?;
        }
        (None, None) => {}
    }
    Ok(())
}

/// Register one existing namespace value with its per-member native flags.
pub(crate) fn register_namespace_value_with_flags(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    value: Val,
    flags: u8,
) -> LuaResult<()> {
    let (publish_global, publish_secure) =
        crate::lua_api::globals::security::native_registration_destinations(flags);
    let global_parent = state.global;
    let ordinary = publish_global.then(|| ensure_table_at_parent(state, global_parent, namespace));
    let secure_parent = if publish_secure {
        Some(crate::lua_api::globals::security::secure_env_table(state)?)
    } else {
        None
    };
    let secure = secure_parent.map(|parent| ensure_table_at_parent(state, parent, namespace));

    if let (Some(ordinary), Some(secure)) = (ordinary, secure) {
        crate::lua_bridge::set_namespace_peer(state, ordinary, secure)?;
    }
    if let Some(ordinary) = ordinary {
        crate::lua_api::methods::table_set(state, Val::Table(ordinary), member, value);
    }
    if let Some(secure) = secure {
        crate::lua_api::methods::table_set(state, Val::Table(secure), member, value);
    }
    Ok(())
}

/// Preserve an unclassified namespace entry on both sides of the current allowlist.
pub(crate) fn register_unknown_namespace_function(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    let (ordinary, _) = ensure_namespace_pair(state, namespace)?;
    table_set_rust_fn(state, ordinary, member, function)
}

pub(crate) fn ensure_namespace_pair(
    state: &mut LuaState,
    name: &str,
) -> LuaResult<(GcRef<Table>, GcRef<Table>)> {
    let ordinary_parent = state.global;
    let secure_parent = crate::lua_api::globals::security::secure_env_table(state)?;
    let ordinary = ensure_table_at_parent(state, ordinary_parent, name);
    let secure = ensure_table_at_parent(state, secure_parent, name);
    crate::lua_bridge::set_namespace_peer(state, ordinary, secure)?;
    Ok((ordinary, secure))
}

pub(crate) fn ensure_namespace_table_field(
    state: &mut LuaState,
    ordinary_parent: GcRef<Table>,
    name: &str,
) -> LuaResult<GcRef<Table>> {
    let secure_parent = crate::lua_bridge::namespace_peer(state, ordinary_parent)?
        .ok_or_else(|| rilua::runtime_error("secure namespace parent was not registered"))?;
    let ordinary = ensure_table_at_parent(state, ordinary_parent, name);
    let secure = ensure_table_at_parent(state, secure_parent, name);
    crate::lua_bridge::set_namespace_peer(state, ordinary, secure)?;
    Ok(ordinary)
}

pub(crate) fn ensure_paired_table_field(
    state: &mut LuaState,
    ordinary_parent: Val,
    name: &str,
) -> LuaResult<Val> {
    let Val::Table(parent_ref) = ordinary_parent else {
        return Err(rilua::runtime_error("namespace parent must be a table"));
    };
    ensure_namespace_table_field(state, parent_ref, name).map(Val::Table)
}

pub(crate) fn set_namespace_member(
    state: &mut LuaState,
    namespace: Val,
    name: &str,
    value: Val,
) -> LuaResult<()> {
    let Val::Table(namespace_ref) = namespace else {
        return Err(rilua::runtime_error("namespace must be a table"));
    };
    let key = Val::Str(state.gc.intern_string(name.as_bytes()));
    crate::lua_bridge::table_set_namespace_member(state, namespace_ref, key, value)
}

fn ensure_table_at_parent(state: &mut LuaState, parent: GcRef<Table>, name: &str) -> GcRef<Table> {
    let key_ref = state.gc.intern_string(name.as_bytes());
    let current = state
        .gc
        .tables
        .get(parent)
        .map(|table| table.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if let Val::Table(table_ref) = current {
        return table_ref;
    }

    let table_ref = state.gc.alloc_table(Table::new());
    if let Some(table) = state.gc.tables.get_mut(parent) {
        let _ = table.raw_set(
            Val::Str(key_ref),
            Val::Table(table_ref),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(parent);
    table_ref
}

#[cfg(test)]
mod tests {
    use super::{ensure_namespace_pair, register_namespace_function_with_flags};
    use crate::lua_api::WowLuaEnv;
    use crate::lua_api::methods::{registry_get, table_get_static};
    use rilua::vm::state::LuaState;
    use rilua::{LuaApiMut, LuaResult, Val};

    fn flag_probe(state: &mut LuaState) -> LuaResult<u32> {
        state.push(Val::Bool(true));
        Ok(1)
    }

    #[test]
    fn namespace_flags_select_tables_and_members() {
        let env = WowLuaEnv::new().expect("lua env");
        {
            let mut lua = env.rilua_mut();
            for (namespace, flags) in [
                ("flagnamespaceordinary", 1),
                ("flagnamespacesecure", 2),
                ("flagnamespaceboth", 3),
                ("flagnamespacebothseven", 7),
                ("flagnamespaceneither", 4),
            ] {
                register_namespace_function_with_flags(
                    lua.state_mut(),
                    namespace,
                    "probe",
                    flag_probe,
                    flags,
                )
                .expect("register flagged namespace member");
            }
        }

        let result: Vec<bool> = env
            .eval(
                r#"
                local function has(parent, namespace)
                    local table = rawget(parent, namespace)
                    return type(table) == "table" and type(rawget(table, "probe")) == "function"
                end
                local public = rawget(_G, "flagnamespaceboth")
                local secure = rawget(__secureenv, "flagnamespaceboth")
                return {
                    has(_G, "flagnamespaceordinary"), has(__secureenv, "flagnamespaceordinary"),
                    has(_G, "flagnamespacesecure"), has(__secureenv, "flagnamespacesecure"),
                    has(_G, "flagnamespaceboth"), has(__secureenv, "flagnamespaceboth"),
                    public ~= secure,
                    has(_G, "flagnamespacebothseven"), has(__secureenv, "flagnamespacebothseven"),
                    has(_G, "flagnamespaceneither"), has(__secureenv, "flagnamespaceneither"),
                }
                "#,
            )
            .expect("namespace flag destination probe");

        assert_eq!(
            result,
            vec![
                true, false, false, true, true, true, true, true, true, false, false
            ]
        );
    }

    #[test]
    fn flag_destinations_do_not_mirror_into_an_existing_namespace_peer() {
        let env = WowLuaEnv::new().expect("lua env");
        {
            let mut lua = env.rilua_mut();
            let state = lua.state_mut();
            let _ = ensure_namespace_pair(state, "existingpairedflagnamespace")
                .expect("create paired namespace");
            register_namespace_function_with_flags(
                state,
                "existingpairedflagnamespace",
                "ordinaryonly",
                flag_probe,
                1,
            )
            .expect("register ordinary-only member");
            register_namespace_function_with_flags(
                state,
                "existingpairedflagnamespace",
                "secureonly",
                flag_probe,
                2,
            )
            .expect("register secure-only member");
        }

        let result: Vec<bool> = env
            .eval(
                r#"
                local public = rawget(_G, "existingpairedflagnamespace")
                local secure = rawget(__secureenv, "existingpairedflagnamespace")
                return {
                    type(rawget(public, "ordinaryonly")) == "function",
                    rawget(secure, "ordinaryonly") == nil,
                    rawget(public, "secureonly") == nil,
                    type(rawget(secure, "secureonly")) == "function",
                }
                "#,
            )
            .expect("paired namespace flag probe");

        assert_eq!(result, vec![true, true, true, true]);
    }

    #[cfg(feature = "client-retail")]
    #[test]
    fn retail_c_timer_dual_publication_preserves_function_identity() {
        let env = WowLuaEnv::new().expect("lua env");
        {
            let mut lua = env.rilua_mut();
            let state = lua.state_mut();
            for member in ["After", "NewTicker", "NewTimer"] {
                register_namespace_function_with_flags(state, "C_Timer", member, flag_probe, 3)
                    .expect("register dual-published C_Timer member");
            }
        }

        let result: (bool, bool, bool) = env
            .eval(
                r#"
                local public = rawget(_G, "C_Timer")
                local secure = rawget(__secureenv, "C_Timer")
                return rawequal(rawget(public, "After"), rawget(secure, "After")),
                    rawequal(rawget(public, "NewTicker"), rawget(secure, "NewTicker")),
                    rawequal(rawget(public, "NewTimer"), rawget(secure, "NewTimer"))
                "#,
            )
            .expect("C_Timer function identity probe");

        assert_eq!(result, (true, true, true));
    }

    #[test]
    fn namespace_secure_publication_requires_an_existing_secure_table() {
        let mut lua = rilua::Lua::new().expect("lua state");
        register_namespace_function_with_flags(
            lua.state_mut(),
            "secureonlywithoutenv",
            "probe",
            flag_probe,
            2,
        )
        .expect("register secure-only namespace member");
        register_namespace_function_with_flags(
            lua.state_mut(),
            "bothwithoutenv",
            "probe",
            flag_probe,
            3,
        )
        .expect("register dual namespace member");

        assert!(matches!(
            LuaApiMut::get_global_val(&mut lua, "secureonlywithoutenv"),
            Val::Nil
        ));
        let ordinary = LuaApiMut::get_global_val(&mut lua, "bothwithoutenv");
        assert!(matches!(ordinary, Val::Table(_)));
        assert!(matches!(
            table_get_static(lua.state_mut(), ordinary, "probe"),
            Val::Function(_)
        ));
        assert!(!matches!(
            registry_get(lua.state_mut(), "__secureenv"),
            Val::Table(_)
        ));
    }
}

/// Keep ordinary namespace lookup from fabricating retired members.
pub(crate) fn mark_namespace_keys_removed(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    names: &[&str],
) {
    use crate::lua_api::methods::{create_table, table_get, table_set};
    let table = Val::Table(namespace);
    let existing = table_get(state, table, "__wow_removed_keys");
    let removed = if matches!(existing, Val::Table(_)) {
        existing
    } else {
        create_table(state)
    };
    for name in names {
        table_set(state, removed, name, Val::Bool(true));
    }
    table_set(state, table, "__wow_removed_keys", removed);
}

pub(crate) fn set_table_array(state: &mut LuaState, table: Val, index: i64, value: Val) {
    let Val::Table(table_ref) = table else { return };
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}
