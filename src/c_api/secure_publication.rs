//! Retail secure publication driven by the verified R88y registration table.

use super::secure_publication_data::{
    CONTEXT_BOTH, CONTEXT_GAME, CONTEXT_GLUE, INFERRED_ROW_COUNT, SOURCE_ROW_COUNT, SOURCE_SHA256,
    VERIFIED_MEMBERS, VERIFIED_STRING_LIBRARY_COPY_ALIASES,
};
use super::{
    global_val, register_namespace_function_with_flags, register_namespace_value_with_flags,
    set_global_val,
};
use crate::lua_api::globals::security::{
    SECURE_ENV_BASE_NAMES_KEY, native_registration_destinations, secure_env_table,
    set_secure_env_key_state,
};
use crate::lua_api::methods::{create_table, registry_get, registry_set, table_get, table_set};
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

pub(crate) fn publish_verified_secure_namespace_members(state: &mut LuaState) -> LuaResult<usize> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(0);
    }

    debug_assert_eq!(
        VERIFIED_MEMBERS.len() + INFERRED_ROW_COUNT,
        SOURCE_ROW_COUNT
    );
    let (ordinary_snapshots, secure_snapshots) = registry_snapshot_tables(state)?;
    let (unmodeled, context_counts) =
        publish_verified_manifest_rows(state, ordinary_snapshots, secure_snapshots)?;
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
    ordinary_snapshots: Val,
    secure_snapshots: Val,
) -> LuaResult<(usize, [usize; 3])> {
    let mut unmodeled = 0;
    let mut context_counts = [0usize; 3];
    for &(namespace, member, flag, context) in VERIFIED_MEMBERS {
        count_registration_context(context, &mut context_counts)?;
        if !publish_and_snapshot_member(
            state,
            namespace,
            member,
            flag,
            ordinary_snapshots,
            secure_snapshots,
        )? {
            unmodeled += 1;
        }
    }
    Ok((unmodeled, context_counts))
}

fn count_registration_context(context: u8, counts: &mut [usize; 3]) -> LuaResult<()> {
    match context {
        CONTEXT_GAME => counts[0] += 1,
        CONTEXT_GLUE => counts[1] += 1,
        CONTEXT_BOTH => counts[2] += 1,
        _ => return Err(runtime_error("unsupported R88y registration context")),
    }
    Ok(())
}

fn publish_and_snapshot_member(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    ordinary_snapshots: Val,
    secure_snapshots: Val,
) -> LuaResult<bool> {
    let ordinary_history = namespace_snapshot_table(state, ordinary_snapshots, namespace)?;
    let secure_history = namespace_snapshot_table(state, secure_snapshots, namespace)?;
    let previous_ordinary = table_get(state, ordinary_history, member);
    let previous_secure = table_get(state, secure_history, member);
    let implemented = publish_verified_member(
        state,
        namespace,
        member,
        flag,
        previous_ordinary,
        previous_secure,
    )?;

    let (ordinary_namespace, secure_namespace) = namespace_tables(state, namespace)?;
    let ordinary_value = table_get(state, ordinary_namespace, member);
    let secure_value = table_get(state, secure_namespace, member);
    store_snapshot(state, ordinary_history, member, ordinary_value);
    store_snapshot(state, secure_history, member, secure_value);
    Ok(implemented)
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
    let ordinary_value = table_get(state, ordinary_namespace, member);
    let secure_value = table_get(state, secure_namespace, member);
    let (publish_global, publish_secure) = native_registration_destinations(flag);
    match (publish_global, publish_secure) {
        (true, true) => publish_to_both(
            state,
            namespace,
            member,
            flag,
            ordinary_value,
            secure_value,
            previous_ordinary,
            previous_secure,
        ),
        (true, false) => {
            let secure_base_copy = is_fixed_base_copy(state, namespace, member)?;
            publish_ordinary_only(
                state,
                namespace,
                member,
                flag,
                ordinary_value,
                secure_value,
                secure_base_copy,
            )
        }
        (false, true) => {
            publish_secure_only(state, namespace, member, flag, ordinary_value, secure_value)
        }
        (false, false) => Ok(false),
    }
}

fn publish_to_both(
    state: &mut LuaState,
    namespace: &str,
    member: &str,
    flag: u8,
    ordinary: Val,
    secure: Val,
    previous_ordinary: Val,
    previous_secure: Val,
) -> LuaResult<bool> {
    let ordinary = function_value(ordinary);
    let secure = function_value(secure);
    if let Some(ordinary) = ordinary {
        if let Some(secure) = secure {
            update_changed_dual_handler(
                state,
                namespace,
                member,
                flag,
                ordinary,
                secure,
                previous_ordinary,
                previous_secure,
            )?;
        } else {
            set_member_in_both(state, namespace, member, ordinary, flag)?;
        }
        return Ok(true);
    }
    if let Some(secure) = secure {
        if matches!(previous_ordinary, Val::Function(_)) {
            remove_namespace_member(state, namespace, member, true, true)?;
            return Ok(false);
        }
        set_member_in_both(state, namespace, member, secure, flag)?;
        return Ok(true);
    }

    remove_stale_secure_handler(state, namespace, member, previous_ordinary, previous_secure)?;
    Ok(false)
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
    let ordinary_changed = snapshot_changed(previous_ordinary, ordinary);
    let secure_changed = snapshot_changed(previous_secure, secure);
    let replacement = match (ordinary_changed, secure_changed) {
        (true, false) => Some(ordinary),
        (false, true) => Some(secure),
        _ => None,
    };
    if let Some(replacement) = replacement {
        set_member_in_both(state, namespace, member, replacement, flag)?;
    }
    Ok(())
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
    value: Val,
    flag: u8,
) -> LuaResult<()> {
    if namespace == "_G" {
        set_global_val(state, member, value);
        set_secure_env_key_state(state, member, value)?;
        return Ok(());
    }
    validate_target_namespace(state, namespace, true, true)?;
    register_namespace_value_with_flags(state, namespace, member, value, flag)
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
        if secure {
            set_secure_env_key_state(state, member, Val::Nil)?;
        } else {
            set_global_val(state, member, Val::Nil);
        }
        return Ok(());
    }

    let parent = if secure {
        Val::Table(secure_env_table(state)?)
    } else {
        Val::Table(state.global)
    };
    let namespace_value = table_get(state, parent, namespace);
    if let Val::Table(table) = namespace_value
        && matches!(
            table_get(state, Val::Table(table), member),
            Val::Function(_)
        )
    {
        table_set(state, Val::Table(table), member, Val::Nil);
    }
    Ok(())
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

    #[test]
    fn verified_manifest_routes_implemented_functions_and_leaves_gaps_absent() {
        let env = WowLuaEnv::new().expect("lua env");
        if !crate::client_profile::is_active_retail_12_1_5() {
            return;
        }

        let unmodeled = publish_verified_secure_namespace_members(env.rilua_mut().state_mut())
            .expect("verified publication is idempotent");
        assert!(
            VERIFIED_MEMBERS.len() > unmodeled + 2,
            "at least two verified implementations should be routed"
        );

        let routes_and_gap: (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
            .eval(
                r#"
                local ping = rawget(_G, "C_Ping")
                local securePing = rawget(__secureenv, "C_Ping")
                local services = rawget(_G, "C_CharacterServices")
                local secureServices = rawget(__secureenv, "C_CharacterServices")
                local privateAuras = rawget(_G, "C_UnitAurasPrivate")
                local securePrivateAuras = rawget(__secureenv, "C_UnitAurasPrivate")
                local catalog = rawget(_G, "C_CatalogShop")
                local secureCatalog = rawget(__secureenv, "C_CatalogShop")
                return type(ping) == "table"
                        and type(rawget(ping, "GetDefaultPingOptions")) == "function"
                        and type(securePing) == "table"
                        and type(rawget(securePing, "GetDefaultPingOptions")) == "function",
                    type(services) == "table"
                        and type(rawget(services, "HasRequiredBoostForClassTrial")) == "function"
                        and type(secureServices) == "table"
                        and type(rawget(secureServices, "HasRequiredBoostForClassTrial")) == "function",
                    type(securePrivateAuras) == "table"
                        and type(rawget(securePrivateAuras, "GetPrivateAuraAnchors")) == "function",
                    type(privateAuras) == "table"
                        and rawget(privateAuras, "GetPrivateAuraAnchors") == nil,
                    (type(privateAuras) ~= "table"
                        or rawget(privateAuras, "AnchorPrivateAura") == nil)
                        and (type(securePrivateAuras) ~= "table"
                            or rawget(securePrivateAuras, "AnchorPrivateAura") == nil),
                    (type(catalog) ~= "table" or rawget(catalog, "GetAvailableCategoryIDs") == nil)
                        and (type(secureCatalog) ~= "table"
                            or rawget(secureCatalog, "GetAvailableCategoryIDs") == nil),
                    rawget(__secureenv, "GetForbiddenObjectTable") ~= nil
                        and rawget(_G, "GetForbiddenObjectTable") == nil,
                    type(rawget(_G, "RegisterEventCallback")) == "function"
                        and type(rawget(__secureenv, "RegisterEventCallback")) == "function",
                    type(rawget(_G, "CreateFromMixins")) == "function"
                        and type(rawget(__secureenv, "CreateFromMixins")) == "function",
                    type(rawget(_G, "newproxy")) == "function"
                        and rawget(__secureenv, "newproxy") == nil
                "#,
            )
            .expect("verified R88y destinations should be observable");
        assert_eq!(
            routes_and_gap,
            (true, true, true, true, true, true, true, true, true, true)
        );

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
        let before: (bool, bool) = env
            .eval(
                r#"
                local ordinaryNamespace = rawget(_G, "C_CharacterCreation")
                local secureNamespace = rawget(__secureenv, "C_CharacterCreation")
                local ordinary = type(ordinaryNamespace) == "table"
                    and rawget(ordinaryNamespace, "GetTrialBoostStartingLevel")
                local secure = type(secureNamespace) == "table"
                    and rawget(secureNamespace, "GetTrialBoostStartingLevel")
                return type(ordinary) == "function", type(secure) == "function"
                "#,
            )
            .expect("read pre-screen registration state");
        assert_eq!(before, (false, false));

        env.set_screen_mode(ScreenKind::CharacterSelect);
        let late_flag3: (bool, bool, bool) = env
            .eval(
                r#"
                local ordinaryNamespace = rawget(_G, "C_CharacterCreation")
                local secureNamespace = rawget(__secureenv, "C_CharacterCreation")
                local ordinary = type(ordinaryNamespace) == "table"
                    and rawget(ordinaryNamespace, "GetTrialBoostStartingLevel")
                local secure = type(secureNamespace) == "table"
                    and rawget(secureNamespace, "GetTrialBoostStartingLevel")
                return type(ordinary) == "function",
                    type(secure) == "function",
                    rawequal(ordinary, secure)
                "#,
            )
            .expect("read late flag-3 publication");
        assert_eq!(late_flag3, (true, true, true));

        env.exec("rawset(C_Ping, 'GetDefaultPingOptions', function() return 'late' end)")
            .expect("replace ordinary flag-3 namespace member");
        env.set_screen_mode(ScreenKind::CharacterSelect);
        let replaced_flag3: bool = env
            .eval(
                "return rawequal(C_Ping.GetDefaultPingOptions, __secureenv.C_Ping.GetDefaultPingOptions)",
            )
            .expect("read late flag-3 replacement");
        assert!(
            replaced_flag3,
            "secureenv must receive the final ordinary handler"
        );

        env.exec("rawset(_G, 'newproxy', function() return 'late' end)")
            .expect("replace ordinary flag-1 global for route test");
        env.set_screen_mode(ScreenKind::Game);
        let flag1: (bool, bool) = env
            .eval(
                r#"
                return type(rawget(_G, "newproxy")) == "function",
                    rawget(__secureenv, "newproxy") == nil
                "#,
            )
            .expect("read late flag-1 publication");
        assert_eq!(flag1, (true, true));
    }
}
