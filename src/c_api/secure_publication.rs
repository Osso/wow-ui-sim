//! Retail secure publication driven by the verified R88y registration table. Evidence: pr46-49-decomp-answers.md, sections R88y.y1.

use super::secure_publication_data::{
    CONTEXT_BOTH, CONTEXT_GAME, CONTEXT_GLUE, INFERRED_ROW_COUNT, SOURCE_ROW_COUNT, SOURCE_SHA256,
    VERIFIED_MEMBERS, VERIFIED_STRING_LIBRARY_COPY_ALIASES,
};
use super::{
    global_val, register_namespace_function_with_flags, register_namespace_value_with_flags,
    register_namespace_values_with_flags, set_global_val,
};
use crate::lua_api::globals::security::{
    SECURE_ENV_BASE_NAMES_KEY, native_registration_destinations, secure_env_table,
    set_secure_env_key_state,
};
use crate::lua_api::methods::{create_table, registry_get, registry_set, table_get, table_set};
use crate::screen::ScreenKind;
use rilua::vm::closure::{Closure, LuaClosure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn prepare_verified_string_library_copy(state: &mut LuaState) {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return;
    }

    let string = global_val(state, "string");
    if !matches!(string, Val::Table(_)) {
        return;
    }
    for &(member, global_name) in VERIFIED_STRING_LIBRARY_COPY_ALIASES {
        let function = global_val(state, global_name);
        if matches!(function, Val::Function(_)) {
            table_set(state, string, member, function);
        }
    }
}

pub(crate) fn register_global_function_from_r88y(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Err(runtime_error(
            "R88y publication is available only for Retail 12.1.5",
        ));
    }
    let flag = verified_r88y_flag("_G", name)
        .ok_or_else(|| runtime_error(format!("R88y has no VERIFIED flag for _G.{name}")))?;
    crate::lua_api::globals::security::register_native_function_with_flags(
        lua, name, function, flag,
    )
}

pub(crate) fn register_global_function_if_missing_from_r88y(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Err(runtime_error(
            "R88y publication is available only for Retail 12.1.5",
        ));
    }
    let flag = verified_r88y_flag("_G", name)
        .ok_or_else(|| runtime_error(format!("R88y has no VERIFIED flag for _G.{name}")))?;
    crate::lua_api::globals::security::register_native_function_if_missing_with_flags(
        lua, name, function, flag,
    )
}

pub(crate) fn register_namespace_function_from_r88y(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Err(runtime_error(
            "R88y publication is available only for Retail 12.1.5",
        ));
    }
    let flag = verified_r88y_flag(namespace, member).ok_or_else(|| {
        runtime_error(format!(
            "R88y has no VERIFIED flag for {namespace}.{member}"
        ))
    })?;
    register_namespace_function_with_flags(state, namespace, member, function, flag)
}

const ORDINARY_SNAPSHOT_KEY: &str = "__wow_r88y_ordinary_snapshot";
const SECURE_SNAPSHOT_KEY: &str = "__wow_r88y_secure_snapshot";

pub(crate) fn publish_verified_secure_namespace_members(
    state: &mut LuaState,
    screen_kind: ScreenKind,
) -> LuaResult<usize> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(0);
    }

    debug_assert_eq!(
        VERIFIED_MEMBERS.len() + INFERRED_ROW_COUNT,
        SOURCE_ROW_COUNT
    );
    let (ordinary_snapshots, secure_snapshots) = registry_snapshot_tables(state)?;
    let (unmodeled, context_counts) =
        publish_verified_manifest_rows(state, screen_kind, ordinary_snapshots, secure_snapshots)?;
    trace_r88y_manifest(context_counts);
    Ok(unmodeled)
}

fn registry_snapshot_tables(state: &mut LuaState) -> LuaResult<(Val, Val)> {
    Ok((
        registry_snapshot_table(state, ORDINARY_SNAPSHOT_KEY)?,
        registry_snapshot_table(state, SECURE_SNAPSHOT_KEY)?,
    ))
}

fn publish_verified_manifest_rows(
    state: &mut LuaState,
    screen_kind: ScreenKind,
    ordinary_snapshots: Val,
    secure_snapshots: Val,
) -> LuaResult<(usize, [usize; 3])> {
    let mut unmodeled = 0;
    let mut context_counts = [0usize; 3];
    for &(namespace, member, flag, context) in VERIFIED_MEMBERS {
        let context_is_active =
            count_registration_context(context, &mut context_counts, screen_kind)?;
        if !publish_and_snapshot_member(
            state,
            namespace,
            member,
            flag,
            context_is_active,
            ordinary_snapshots,
            secure_snapshots,
        )? {
            unmodeled += 1;
        }
    }
    Ok((unmodeled, context_counts))
}

fn count_registration_context(
    context: u8,
    counts: &mut [usize; 3],
    screen_kind: ScreenKind,
) -> LuaResult<bool> {
    match context {
        CONTEXT_GAME => {
            counts[0] += 1;
            Ok(screen_kind == ScreenKind::Game)
        }
        CONTEXT_GLUE => {
            counts[1] += 1;
            Ok(screen_kind.is_glue())
        }
        CONTEXT_BOTH => {
            counts[2] += 1;
            Ok(true)
        }
        _ => Err(runtime_error("unsupported R88y registration context")),
    }
}
fn publish_and_snapshot_member(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    context_is_active: bool,
    ordinary_snapshots: Val,
    secure_snapshots: Val,
) -> LuaResult<bool> {
    let (ordinary_history, secure_history, previous) = snapshot_member_history(
        state,
        ordinary_snapshots,
        secure_snapshots,
        namespace,
        member,
    )?;
    let implemented = if context_is_active {
        publish_verified_member(state, namespace, member, flag, previous.0, previous.1)?
    } else {
        preserve_inactive_member(state, namespace, member)?
    };
    snapshot_current_member(state, namespace, member, ordinary_history, secure_history)?;
    Ok(implemented)
}

fn snapshot_member_history(
    state: &mut LuaState,
    ordinary_snapshots: Val,
    secure_snapshots: Val,
    namespace: &str,
    member: &str,
) -> LuaResult<(Val, Val, (Val, Val))> {
    let ordinary = namespace_snapshot_table(state, ordinary_snapshots, namespace)?;
    let secure = namespace_snapshot_table(state, secure_snapshots, namespace)?;
    let previous = (
        table_get(state, ordinary, member),
        table_get(state, secure, member),
    );
    Ok((ordinary, secure, previous))
}

fn preserve_inactive_member(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
) -> LuaResult<bool> {
    let (ordinary, secure) = namespace_tables(state, namespace)?;
    let ordinary_implemented = function_value(table_get(state, ordinary, member)).is_some();
    let secure_implemented = function_value(table_get(state, secure, member)).is_some();
    let preserve_fixed_base = is_fixed_base_copy(state, namespace, member)?;
    remove_namespace_member(state, namespace, member, true, !preserve_fixed_base)?;
    Ok(ordinary_implemented || secure_implemented)
}

fn snapshot_current_member(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    ordinary_history: Val,
    secure_history: Val,
) -> LuaResult<()> {
    let (ordinary, secure) = namespace_tables(state, namespace)?;
    store_snapshot(
        state,
        ordinary_history,
        member,
        table_get(state, ordinary, member),
    );
    store_snapshot(
        state,
        secure_history,
        member,
        table_get(state, secure, member),
    );
    Ok(())
}

fn trace_r88y_manifest(context_counts: [usize; 3]) {
    tracing::trace!(
        source_sha256 = SOURCE_SHA256,
        source_rows = SOURCE_ROW_COUNT,
        verified_rows = VERIFIED_MEMBERS.len(),
        inferred_rows = INFERRED_ROW_COUNT,
        verified_game_context_rows = context_counts[0],
        verified_glue_context_rows = context_counts[1],
        verified_both_context_rows = context_counts[2],
        "apply Retail secure-publication flags"
    );
}

fn registry_snapshot_table(state: &mut LuaState, key: &'static str) -> LuaResult<Val> {
    match registry_get(state, key) {
        table @ Val::Table(_) => Ok(table),
        Val::Nil => {
            let table = create_table(state);
            registry_set(state, key, table);
            Ok(table)
        }
        value => Err(runtime_error(format!(
            "R88y registry entry {key} is {}, not a table",
            value.type_name()
        ))),
    }
}

fn namespace_snapshot_table(
    state: &mut LuaState,
    snapshots: Val,
    namespace: &str,
) -> LuaResult<Val> {
    match table_get(state, snapshots, namespace) {
        table @ Val::Table(_) => Ok(table),
        Val::Nil => {
            let table = create_table(state);
            table_set(state, snapshots, namespace, table);
            Ok(table)
        }
        value => Err(runtime_error(format!(
            "R88y snapshot for {namespace} is {}, not a table",
            value.type_name()
        ))),
    }
}

fn store_snapshot(state: &mut LuaState, snapshot: Val, member: &str, value: Val) {
    let stored = function_value(value).unwrap_or(Val::Bool(false));
    table_set(state, snapshot, member, stored);
}

fn snapshot_changed(previous: Val, current: Val) -> bool {
    match previous {
        Val::Nil => false,
        Val::Bool(false) => matches!(current, Val::Function(_)),
        Val::Function(_) => previous != current,
        _ => true,
    }
}
fn publish_verified_member(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    previous_ordinary: Val,
    previous_secure: Val,
) -> LuaResult<bool> {
    let (ordinary_namespace, secure_namespace) = namespace_tables(state, namespace)?;
    let ordinary = table_get(state, ordinary_namespace, member);
    let secure = table_get(state, secure_namespace, member);
    let destinations = native_registration_destinations(flag);
    route_verified_member_by_destinations(
        state,
        namespace,
        member,
        flag,
        destinations,
        (ordinary, secure),
        (previous_ordinary, previous_secure),
    )
}

fn route_verified_member_by_destinations(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    destinations: (bool, bool),
    values: (Val, Val),
    previous: (Val, Val),
) -> LuaResult<bool> {
    let (ordinary, secure) = values;
    match destinations {
        (true, true) => publish_to_both(state, namespace, member, flag, values, previous),
        (true, false) => {
            let secure_base_copy = is_fixed_base_copy(state, namespace, member)?;
            publish_ordinary_only(
                state,
                namespace,
                member,
                flag,
                ordinary,
                secure,
                secure_base_copy,
            )
        }
        (false, true) => publish_secure_only(state, namespace, member, flag, ordinary, secure),
        (false, false) => Ok(false),
    }
}
fn publish_to_both(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    values: (Val, Val),
    previous: (Val, Val),
) -> LuaResult<bool> {
    let handlers = (function_value(values.0), function_value(values.1));
    match handlers {
        (Some(ordinary), Some(secure)) => update_existing_dual_handler(
            state,
            namespace,
            member,
            flag,
            (ordinary, secure),
            previous,
        ),
        (Some(ordinary), None) => publish_from_ordinary(state, namespace, member, flag, ordinary),
        (None, Some(secure)) => {
            publish_from_secure(state, namespace, member, flag, secure, previous.1)
        }
        (None, None) => remove_missing_dual_handler(state, namespace, member, previous),
    }
}

fn update_existing_dual_handler(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    handlers: (Val, Val),
    previous: (Val, Val),
) -> LuaResult<bool> {
    update_changed_dual_handler(
        state, namespace, member, flag, handlers.0, handlers.1, previous.0, previous.1,
    )?;
    Ok(true)
}

fn remove_missing_dual_handler(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    previous: (Val, Val),
) -> LuaResult<bool> {
    remove_stale_secure_handler(state, namespace, member, previous.0, previous.1)?;
    Ok(false)
}

fn publish_from_ordinary(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    ordinary: Val,
) -> LuaResult<bool> {
    let secure = separate_flag3_destination_function_object(state, ordinary, flag)?;
    set_member_in_both(state, namespace, member, ordinary, secure, flag)?;
    Ok(true)
}

fn publish_from_secure(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    secure: Val,
    previous_secure: Val,
) -> LuaResult<bool> {
    if matches!(previous_secure, Val::Function(_)) {
        // Public-table cleanup does not revoke a prior secure registration.
        return Ok(true);
    }
    let ordinary = separate_flag3_destination_function_object(state, secure, flag)?;
    set_member_in_both(state, namespace, member, ordinary, secure, flag)?;
    Ok(true)
}
fn update_changed_dual_handler(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    ordinary: Val,
    secure: Val,
    previous_ordinary: Val,
    previous_secure: Val,
) -> LuaResult<()> {
    let Some((ordinary_value, secure_value)) = changed_dual_destinations(
        state,
        flag,
        ordinary,
        secure,
        previous_ordinary,
        previous_secure,
    )?
    else {
        return Ok(());
    };
    if ordinary_value != ordinary || secure_value != secure {
        set_member_in_both(state, namespace, member, ordinary_value, secure_value, flag)?;
    }
    Ok(())
}

fn changed_dual_destinations(
    state: &mut LuaState,
    flag: u8,
    ordinary: Val,
    secure: Val,
    previous_ordinary: Val,
    previous_secure: Val,
) -> LuaResult<Option<(Val, Val)>> {
    let ordinary_changed = snapshot_changed(previous_ordinary, ordinary);
    let secure_changed = snapshot_changed(previous_secure, secure);
    match (ordinary_changed, secure_changed) {
        (true, false) => clone_ordinary_destination(state, ordinary, flag).map(Some),
        (false, true) => clone_secure_destination(state, secure, flag).map(Some),
        (true, true) | (false, false) if ordinary == secure => {
            clone_ordinary_destination(state, ordinary, flag).map(Some)
        }
        _ => Ok(None),
    }
}

fn clone_ordinary_destination(
    state: &mut LuaState,
    ordinary: Val,
    flag: u8,
) -> LuaResult<(Val, Val)> {
    let secure = separate_flag3_destination_function_object(state, ordinary, flag)?;
    Ok((ordinary, secure))
}

fn clone_secure_destination(state: &mut LuaState, secure: Val, flag: u8) -> LuaResult<(Val, Val)> {
    let ordinary = separate_flag3_destination_function_object(state, secure, flag)?;
    Ok((ordinary, secure))
}

fn remove_stale_secure_handler(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    previous_ordinary: Val,
    previous_secure: Val,
) -> LuaResult<()> {
    if matches!(previous_ordinary, Val::Function(_)) || matches!(previous_secure, Val::Function(_))
    {
        remove_namespace_member(state, namespace, member, true, true)?;
    }
    Ok(())
}

fn publish_ordinary_only(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    ordinary: Val,
    secure: Val,
    secure_base_copy: bool,
) -> LuaResult<bool> {
    if let Some(function) = function_value(ordinary) {
        set_member_in_destination(state, namespace, member, function, flag, false)?;
        remove_namespace_member(state, namespace, member, true, !secure_base_copy)?;
        return Ok(true);
    }
    if secure_base_copy && function_value(secure).is_some() {
        return Ok(true);
    }
    remove_namespace_member(state, namespace, member, true, true)?;
    Ok(false)
}

fn publish_secure_only(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    ordinary: Val,
    secure: Val,
) -> LuaResult<bool> {
    let value = function_value(secure).or_else(|| function_value(ordinary));
    let Some(value) = value else {
        remove_namespace_member(state, namespace, member, false, true)?;
        return Ok(false);
    };
    set_member_in_destination(state, namespace, member, value, flag, true)?;
    remove_namespace_member(state, namespace, member, false, true)?;
    Ok(true)
}

fn set_member_in_both(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    ordinary_value: Val,
    secure_value: Val,
    flag: u8,
) -> LuaResult<()> {
    if namespace == "_G" {
        set_global_val(state, member, ordinary_value);
        set_secure_env_key_state(state, member, secure_value)?;
        return Ok(());
    }
    validate_target_namespace(state, namespace, true, true)?;
    register_namespace_values_with_flags(
        state,
        namespace,
        member,
        ordinary_value,
        secure_value,
        flag,
    )
}

fn set_member_in_destination(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    value: Val,
    flag: u8,
    secure: bool,
) -> LuaResult<()> {
    if namespace == "_G" {
        if secure {
            set_secure_env_key_state(state, member, value)?;
        } else {
            set_global_val(state, member, value);
        }
        return Ok(());
    }
    validate_target_namespace(state, namespace, !secure, secure)?;
    register_namespace_value_with_flags(state, namespace, member, value, flag)
}

fn namespace_tables(state: &mut LuaState, namespace: &str) -> LuaResult<(Val, Val)> {
    let secure = Val::Table(secure_env_table(state)?);
    if namespace == "_G" {
        return Ok((Val::Table(state.global), secure));
    }
    Ok((
        global_val(state, namespace),
        table_get(state, secure, namespace),
    ))
}

fn function_value(value: Val) -> Option<Val> {
    matches!(value, Val::Function(_)).then_some(value)
}

fn separate_flag3_function_object(state: &mut LuaState, value: Val, flag: u8) -> LuaResult<Val> {
    if flag != 3 {
        return Ok(value);
    }
    let Val::Function(reference) = value else {
        return Ok(value);
    };
    // Build a new C closure instead of copying the existing function value.
    let duplicate = match state.gc.closures.get(reference) {
        Some(Closure::Rust(handler)) if handler.upvalues.is_empty() => RustClosure {
            func: handler.func,
            upvalues: Vec::new(),
            name: handler.name.clone(),
            env: handler.env,
        },
        Some(Closure::Rust(handler)) => {
            return Err(runtime_error(format!(
                "flag-3 native function {} has captured values",
                handler.name
            )));
        }
        _ => return Ok(value),
    };
    Ok(Val::Function(
        state.gc.alloc_closure(Closure::Rust(duplicate)),
    ))
}

fn separate_flag3_destination_function_object(
    state: &mut LuaState,
    value: Val,
    flag: u8,
) -> LuaResult<Val> {
    if !crate::client_profile::is_active_retail_12_1_5() || flag != 3 {
        return Ok(value);
    }
    let value = separate_flag3_function_object(state, value, flag)?;
    let Val::Function(reference) = value else {
        return Ok(value);
    };
    let duplicate = match state.gc.closures.get(reference) {
        Some(Closure::Lua(function)) => LuaClosure {
            proto: function.proto.clone(),
            upvalues: function.upvalues.clone(),
            env: function.env.clone(),
        },
        _ => return Ok(value),
    };
    let taint = rilua::stdlib::taint::get_closure_taint(state, reference);
    let duplicate = state.gc.alloc_closure(Closure::Lua(duplicate));
    if let Some(taint) = taint.as_deref() {
        rilua::stdlib::taint::set_closure_taint(state, duplicate, Some(taint))?;
    }
    Ok(Val::Function(duplicate))
}

fn is_fixed_base_copy(state: &mut LuaState, namespace: &str, member: &str) -> LuaResult<bool> {
    let Val::Table(base_names) = registry_get(state, SECURE_ENV_BASE_NAMES_KEY) else {
        return Err(runtime_error("secure environment base-name set is missing"));
    };
    let name = if namespace == "_G" { member } else { namespace };
    Ok(matches!(
        table_get(state, Val::Table(base_names), name),
        Val::Bool(true)
    ))
}

fn validate_target_namespace(
    state: &mut LuaState,
    namespace: &str,
    publish_global: bool,
    publish_secure: bool,
) -> LuaResult<()> {
    let secure = Val::Table(secure_env_table(state)?);
    for (parent, target, label) in [
        (Val::Table(state.global), publish_global, "ordinary"),
        (secure, publish_secure, "secure"),
    ] {
        if !target {
            continue;
        }
        let value = table_get(state, parent, namespace);
        if !matches!(value, Val::Nil | Val::Table(_)) {
            return Err(runtime_error(format!(
                "R88y target {label} namespace {namespace} is {}",
                value.type_name()
            )));
        }
    }
    Ok(())
}
fn remove_namespace_member(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    secure: bool,
    remove: bool,
) -> LuaResult<()> {
    if !remove {
        return Ok(());
    }
    if namespace == "_G" {
        return remove_global_member(state, member, secure);
    }
    let parent = namespace_parent(state, secure)?;
    remove_namespace_function(state, parent, namespace, member);
    Ok(())
}

fn remove_global_member(state: &mut LuaState, member: &str, secure: bool) -> LuaResult<()> {
    if secure {
        set_secure_env_key_state(state, member, Val::Nil)?;
    } else {
        set_global_val(state, member, Val::Nil);
    }
    Ok(())
}

fn namespace_parent(state: &mut LuaState, secure: bool) -> LuaResult<Val> {
    if secure {
        Ok(Val::Table(secure_env_table(state)?))
    } else {
        Ok(Val::Table(state.global))
    }
}

fn remove_namespace_function(state: &mut LuaState, parent: Val, namespace: &str, member: &str) {
    let namespace = table_get(state, parent, namespace);
    let Val::Table(table) = namespace else { return };
    if matches!(
        table_get(state, Val::Table(table), member),
        Val::Function(_)
    ) {
        table_set(state, Val::Table(table), member, Val::Nil);
    }
}

pub(crate) fn verified_r88y_flag(namespace: &str, member: &str) -> Option<u8> {
    VERIFIED_MEMBERS
        .binary_search_by(|candidate| (candidate.0, candidate.1).cmp(&(namespace, member)))
        .ok()
        .map(|index| VERIFIED_MEMBERS[index].2)
}

#[cfg(test)]
mod tests {
    use super::super::secure_publication_data::VERIFIED_MEMBERS;
    use super::{publish_verified_secure_namespace_members, verified_r88y_flag};
    use crate::lua_api::WowLuaEnv;
    use crate::screen::ScreenKind;
    use rilua::LuaApiMut;

    const VERIFIED_MANIFEST_PROBE: &str = r#"
                local ping = rawget(_G, "C_Ping")
                local securePing = rawget(__secureenv, "C_Ping")
                local services = rawget(_G, "C_CharacterServices")
                local secureServices = rawget(__secureenv, "C_CharacterServices")
                local privateAuras = rawget(_G, "C_UnitAurasPrivate")
                local securePrivateAuras = rawget(__secureenv, "C_UnitAurasPrivate")
                local catalog = rawget(_G, "C_CatalogShop")
                local secureCatalog = rawget(__secureenv, "C_CatalogShop")
                local color = rawget(_G, "C_UIColor")
                local secureColor = rawget(__secureenv, "C_UIColor")
                local store = rawget(_G, "C_StoreSecure")
                local secureStore = rawget(__secureenv, "C_StoreSecure")
                -- These Lua fallback producers do not receive native C-closure identity checks.
                -- Apply those checks when ping_defaults.rs and character_services_defaults.rs
                -- are replaced by native C producers.
                local function hasPublishedFunction(ordinary, secure, member)
                    if type(ordinary) ~= "table" or type(secure) ~= "table" then
                        return false
                    end
                    return type(rawget(ordinary, member)) == "function"
                        and type(rawget(secure, member)) == "function"
                end
                local function hasDistinctHandlers(ordinary, secure, member)
                    if type(ordinary) ~= "table" or type(secure) ~= "table" then
                        return false
                    end
                    local ordinaryHandler = rawget(ordinary, member)
                    local secureHandler = rawget(secure, member)
                    if type(ordinaryHandler) ~= "function"
                        or type(secureHandler) ~= "function" then
                        return false
                    end
                    return not rawequal(ordinaryHandler, secureHandler)
                end
                return hasPublishedFunction(ping, securePing, "GetDefaultPingOptions"),
                    hasPublishedFunction(
                        services,
                        secureServices,
                        "HasRequiredBoostForClassTrial"
                    ),
                    hasDistinctHandlers(color, secureColor, "GetColors"),
                    hasDistinctHandlers(store, secureStore, "GetProductGroups"),
                    type(securePrivateAuras) == "table"
                        and type(rawget(securePrivateAuras, "GetPrivateAuraAnchors")) == "function",
                    type(privateAuras) == "table"
                        and rawget(privateAuras, "GetPrivateAuraAnchors") == nil,
                    (type(privateAuras) ~= "table"
                        or rawget(privateAuras, "AnchorPrivateAura") == nil)
                        and (type(securePrivateAuras) ~= "table"
                            or rawget(securePrivateAuras, "AnchorPrivateAura") == nil),
                    (type(catalog) ~= "table"
                        or rawget(catalog, "GetAvailableTransmogRaceInfos") == nil)
                        and (type(secureCatalog) ~= "table"
                            or rawget(secureCatalog, "GetAvailableTransmogRaceInfos") == nil),
                    rawget(__secureenv, "GetForbiddenObjectTable") ~= nil
                        and rawget(_G, "GetForbiddenObjectTable") == nil,
                    type(rawget(_G, "RegisterEventCallback")) == "function"
                        and type(rawget(__secureenv, "RegisterEventCallback")) == "function",
                    type(rawget(_G, "CreateFromMixins")) == "function"
                        and type(rawget(__secureenv, "CreateFromMixins")) == "function",
                    type(rawget(_G, "newproxy")) == "function"
                        and rawget(__secureenv, "newproxy") == nil
                "#;
    const SCREEN_SWITCH_INITIAL_PROBE: &str = r#"
                local ordinaryNamespace = rawget(_G, "C_CharacterCreation")
                local secureNamespace = rawget(__secureenv, "C_CharacterCreation")
                local ordinary = type(ordinaryNamespace) == "table"
                    and rawget(ordinaryNamespace, "GetTrialBoostStartingLevel")
                local secure = type(secureNamespace) == "table"
                    and rawget(secureNamespace, "GetTrialBoostStartingLevel")
                return type(ordinary) == "function", type(secure) == "function"
                "#;
    const SCREEN_SWITCH_GLUE_PROBE: &str = r#"
                local ordinaryNamespace = rawget(_G, "C_CharacterCreation")
                local secureNamespace = rawget(__secureenv, "C_CharacterCreation")
                local ordinary = type(ordinaryNamespace) == "table"
                    and rawget(ordinaryNamespace, "GetTrialBoostStartingLevel")
                local secure = type(secureNamespace) == "table"
                    and rawget(secureNamespace, "GetTrialBoostStartingLevel")
                local ordinaryLevel = type(ordinary) == "function" and ordinary()
                local secureLevel = type(secure) == "function" and secure()
                return type(ordinary) == "function",
                    type(secure) == "function",
                    not rawequal(ordinary, secure),
                    type(ordinaryLevel) == "number",
                    type(secureLevel) == "number",
                    ordinaryLevel == secureLevel
                "#;
    const SCREEN_SWITCH_REPLACED_PROBE: &str = r#"
                local ordinary = C_Ping.GetDefaultPingOptions
                local secure = __secureenv.C_Ping.GetDefaultPingOptions
                return type(ordinary) == "function",
                    type(secure) == "function",
                    ordinary(),
                    secure()
                "#;
    const SCREEN_SWITCH_FLAG1_PROBE: &str = r#"
                return type(rawget(_G, "newproxy")) == "function",
                    rawget(__secureenv, "newproxy") == nil
                "#;

    #[test]
    fn verified_manifest_routes_implemented_functions_and_leaves_gaps_absent() {
        let env = WowLuaEnv::new().expect("lua env");
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }
        let unmodeled = publish_verified_secure_namespace_members(
            env.rilua_mut().state_mut(),
            ScreenKind::Game,
        )
        .expect("verified publication is idempotent");
        assert_manifest_route_count(unmodeled);
        assert_verified_manifest_destinations(&env);
        assert_verified_r88y_flags();
    }

    fn assert_manifest_route_count(unmodeled: usize) {
        assert!(
            VERIFIED_MEMBERS.len() > unmodeled + 2,
            "at least two verified implementations should be routed"
        );
    }

    fn assert_verified_manifest_destinations(env: &WowLuaEnv) {
        let routes_and_gap: (
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
        ) = env
            .eval(VERIFIED_MANIFEST_PROBE)
            .expect("verified R88y destinations should be observable");
        assert_eq!(
            routes_and_gap,
            (
                true, true, true, true, true, true, true, true, true, true, true, true
            )
        );
    }

    fn assert_verified_r88y_flags() {
        assert_eq!(
            verified_r88y_flag("C_Ping", "GetDefaultPingOptions"),
            Some(3)
        );
        assert_eq!(
            verified_r88y_flag("C_UnitAurasPrivate", "AnchorPrivateAura"),
            Some(2)
        );
        assert_eq!(verified_r88y_flag("_G", "GetForbiddenObjectTable"), Some(6));
        assert_eq!(
            verified_r88y_flag("C_CatalogShop", "GetAvailableCategoryIDs"),
            Some(3)
        );
    }

    #[test]
    fn screen_switch_publishes_late_flag3_and_keeps_flag1_ordinary_only() {
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }
        let env = WowLuaEnv::new().expect("lua env");
        assert_pre_screen_flag3_absent(&env);
        assert_glue_screen_flag3_published(&env);
        replace_flag3_and_assert_late_handler(&env);
        assert_flag1_remains_ordinary_only(&env);
    }

    fn assert_pre_screen_flag3_absent(env: &WowLuaEnv) {
        let before: (bool, bool) = env
            .eval(SCREEN_SWITCH_INITIAL_PROBE)
            .expect("read pre-screen registration state");
        assert_eq!(before, (false, false));
    }

    fn assert_glue_screen_flag3_published(env: &WowLuaEnv) {
        env.set_screen_mode(ScreenKind::CharacterSelect);
        let late_flag3: (bool, bool, bool, bool, bool, bool) = env
            .eval(SCREEN_SWITCH_GLUE_PROBE)
            .expect("read late flag-3 publication");
        assert_eq!(late_flag3, (true, true, true, true, true, true));
    }

    fn replace_flag3_and_assert_late_handler(env: &WowLuaEnv) {
        env.exec("rawset(C_Ping, 'GetDefaultPingOptions', function() return 'late' end)")
            .expect("replace ordinary flag-3 namespace member");
        env.set_screen_mode(ScreenKind::Game);
        let replaced_flag3: (bool, bool, String, String) = env
            .eval(SCREEN_SWITCH_REPLACED_PROBE)
            .expect("read late flag-3 replacement behavior");
        assert_eq!(replaced_flag3, (true, true, "late".into(), "late".into()));
    }

    fn assert_flag1_remains_ordinary_only(env: &WowLuaEnv) {
        env.exec("rawset(_G, 'newproxy', function() return 'late' end)")
            .expect("replace ordinary flag-1 global for route test");
        env.set_screen_mode(ScreenKind::Game);
        let flag1: (bool, bool) = env
            .eval(SCREEN_SWITCH_FLAG1_PROBE)
            .expect("read late flag-1 publication");
        assert_eq!(flag1, (true, true));
    }

    fn secure_screen_members(env: &WowLuaEnv) -> (bool, bool) {
        env.eval(
            r#"
            local ping = rawget(__secureenv, "C_Ping")
            local creation = rawget(__secureenv, "C_CharacterCreation")
            return type(ping) == "table"
                    and type(rawget(ping, "GetDefaultPingOptions")) == "function",
                type(creation) == "table"
                    and type(rawget(creation, "GetTrialBoostStartingLevel")) == "function"
            "#,
        )
        .expect("secure Game and Glue member availability should be observable")
    }

    fn secure_store_presence(env: &WowLuaEnv) -> (bool, bool) {
        env.eval(
            r#"
            local store = rawget(__secureenv, "C_StoreSecure")
            return rawget(_G, "C_StoreSecure") == nil,
                type(store) == "table"
                    and type(rawget(store, "GetProductGroups")) == "function"
            "#,
        )
        .expect("public cleanup and secure Store membership should be observable")
    }

    #[test]
    fn verified_flag3_secure_members_follow_game_and_glue_contexts() {
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }
        let env = WowLuaEnv::new().expect("lua env");
        assert_eq!(secure_screen_members(&env), (true, false));

        env.set_screen_mode(ScreenKind::CharacterSelect);
        assert_eq!(secure_screen_members(&env), (false, true));

        env.set_screen_mode(ScreenKind::Game);
        assert_eq!(secure_screen_members(&env), (true, false));
    }

    #[test]
    fn public_store_cleanup_preserves_secure_registration_on_screen_change() {
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }
        assert_eq!(
            verified_r88y_flag("C_StoreSecure", "GetProductGroups"),
            Some(3)
        );
        let env = WowLuaEnv::new().expect("lua env");
        assert_eq!(secure_store_presence(&env), (false, true));

        env.exec("C_StoreSecure = nil")
            .expect("ordinary Store namespace cleanup should succeed");
        env.restore_post_cleanup_globals();
        assert_eq!(secure_store_presence(&env), (true, true));

        env.set_screen_mode(ScreenKind::Game);
        assert_eq!(secure_store_presence(&env), (true, true));
    }
}
