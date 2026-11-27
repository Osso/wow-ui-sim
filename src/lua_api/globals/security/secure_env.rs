//! Secure environment (secure_env.rs counterpart).
//!
//! `create_secure_environment` seeds one shared secure table from the fixed
//! 39-name native base, then `mark_secure` retargets secure addon chunks to it.
//! Table-valued base entries are shallow-copied. This is not a full `_G` copy
//! or a live overlay; later globals need secure registration or secure file
//! execution to appear in the table. Only Retail 12.1.5 keeps this strict
//! base; other profiles then receive master's temporary `_G` snapshot
//! (`workarounds::temporary::secure_env_global_snapshot`).

use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

use crate::lua_api::methods::{registry_get, registry_set};
use crate::lua_bridge::{table_set_rust_fn, table_set_rust_fn_unpaired};

const NATIVE_REGISTRATION_FLAG_MODULUS: u8 = 4;
pub(crate) const SECURE_ENV_BASE_NAMES_KEY: &str = "__secureenv_base_names";

const CREATE_SECURE_ENV_LUA: &str = r##"
    local genv = _G
    local recordPublication = rawget(genv, "__wow_record_secure_global_publication")
    rawset(genv, "__wow_record_secure_global_publication", nil)

    local baseNames = {
        "assert", "bit", "date", "error", "forceinsecure",
        "GetCurrentEnvironment", "geterrorhandler", "getfenv",
        "GetGlobalEnvironment", "getmetatable", "ipairs", "issecure",
        "math", "next", "pairs", "pcall", "pcallwithenv", "rawget",
        "rawset", "securecall", "securecallfunction", "secureexecuterange",
        "select", "setfenv", "setmetatable", "string", "strcmputf8i",
        "strconcat", "strjoin", "strlenutf8", "strsplit", "table", "time",
        "tonumber", "tostring", "type", "unpack", "IsInGlobalEnvironment",
        "SwapToGlobalEnvironment",
    }
    local secureenv = {}
    local baseNameSet = {}
    for _, name in ipairs(baseNames) do
        baseNameSet[name] = true
        local value = rawget(genv, name)
        if type(value) == "table" then
            local clone = {}
            for key, field in pairs(value) do
                clone[key] = field
            end
            value = clone
        end
        rawset(secureenv, name, value)
    end
    secureenv._G = secureenv
    setmetatable(secureenv, {
        __newindex = function(t, key, value)
            rawset(t, key, value)
            if type(key) == "string" and value ~= nil and key:sub(1, 2) ~= "C_"
                    and type(recordPublication) == "function" then
                recordPublication(key)
            end
        end,
    })
    return secureenv, baseNameSet
	"##;

// temporary: lt-635e. Keep the full snapshot private to secure-mode and replay chunks.
const CREATE_SECURE_REPLAY_ENV_LUA: &str = r##"
    local genv = _G
    local secureenv = rawget(genv, "__secureenv")
    if type(secureenv) ~= "table" then
        error("secure replay environment requires initialized secureenv")
    end

    local debugTable = rawget(genv, "debug")
    local logNilSymbolAccess = rawget(genv, "__wow_log_nil_symbol_access")
    local snapshot = {}
    for key, value in pairs(genv) do
        if key ~= "__secureenv" then
            snapshot[key] = value
        end
    end
    if type(snapshot.Enum) == "table" then
        local enumCopy = {}
        for key, value in pairs(snapshot.Enum) do
            enumCopy[key] = value
        end
        snapshot.Enum = enumCopy
    end

    local replayEnv = {}
    local values = {}
    local assigned = {}
    local enumeration = {}
    local originalRawget = rawget
    local originalRawset = rawset
    local originalPairs = pairs
    local originalNext = next

    local function assign(key, value)
        assigned[key] = true
        values[key] = value
        enumeration[key] = value
        secureenv[key] = value
    end

    local replayControlGlobals = {
        _G = true,
        rawget = true,
        rawset = true,
        pairs = true,
        next = true,
    }

    local function isReplayControlGlobal(key)
        return replayControlGlobals[key] == true
    end

    local function replayValue(key)
        if assigned[key] then
            return originalRawget(values, key)
        end
        if not isReplayControlGlobal(key) then
            local secureValue = originalRawget(secureenv, key)
            if secureValue ~= nil then
                return secureValue
            end
        end
        return originalRawget(snapshot, key)
    end

    local function replayRawget(target, key)
        if target ~= replayEnv then
            return originalRawget(target, key)
        end
        return replayValue(key)
    end

    local function replayRawset(target, key, value)
        if target ~= replayEnv then
            return originalRawset(target, key, value)
        end
        assign(key, value)
        return target
    end

    local function replayPairs(target)
        if target == replayEnv then
            return originalPairs(enumeration)
        end
        return originalPairs(target)
    end

    local function replayNext(target, key)
        if target == replayEnv then
            return originalNext(enumeration, key)
        end
        return originalNext(target, key)
    end

    snapshot.rawget = replayRawget
    snapshot.rawset = replayRawset
    snapshot.pairs = replayPairs
    snapshot.next = replayNext
    snapshot._G = replayEnv
    for key, value in originalPairs(snapshot) do
        enumeration[key] = value
    end

    setmetatable(replayEnv, {
        __index = function(_, key)
            local value = replayValue(key)
            if value == nil and type(logNilSymbolAccess) == "function" then
                local isGlobalIndex = debugTable and debugTable.isglobalindex
                if type(isGlobalIndex) == "function" and isGlobalIndex() then
                    -- The proxy is the secure global scope; report its misses as
                    -- secure globals (nil_symbol_reports.rs classifies "__secureenv").
                    logNilSymbolAccess("__secureenv", key)
                end
            end
            return value
        end,
        __newindex = function(_, key, value)
            assign(key, value)
        end,
    })
    return replayEnv
    "##;

pub(crate) fn secure_env_table(
    state: &mut LuaState,
) -> LuaResult<rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>> {
    match registry_get(state, "__secureenv") {
        Val::Table(table_ref) => Ok(table_ref),
        _ => Err(runtime_error("secure environment not initialized")),
    }
}

pub(crate) fn set_secure_env_key_state(
    state: &mut LuaState,
    key: &str,
    value: Val,
) -> LuaResult<()> {
    let secureenv_ref = secure_env_table(state)?;
    let key_ref = state.gc.intern_string(key.as_bytes());
    let publishes_value = !matches!(value, Val::Nil);
    if let Some(secureenv) = state.gc.tables.get_mut(secureenv_ref) {
        let _ = secureenv.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(secureenv_ref);
    if publishes_value {
        crate::lua_api::globals::compat_overrides::record_secure_global_publication(state, key)?;
    }
    Ok(())
}

pub(crate) fn publish_secure_env_key_if_vacant(
    state: &mut LuaState,
    key: &str,
    value: Val,
) -> LuaResult<bool> {
    let secureenv_ref = secure_env_table(state)?;
    let key_ref = state.gc.intern_string(key.as_bytes());
    let existing = state
        .gc
        .tables
        .get(secureenv_ref)
        .map_or(Val::Nil, |secureenv| {
            secureenv.get_str(key_ref, &state.gc.string_arena)
        });
    if !matches!(existing, Val::Nil) {
        return Ok(false);
    }
    set_secure_env_key_state(state, key, value)?;
    Ok(true)
}

/// Create the secure environment from the fixed native base list.
///
/// Table-valued base entries are shallow-copied. Later `_G` registrations do
/// not fall through; secure-visible exports must be written directly into secureenv.
pub fn create_secure_environment(lua: &mut rilua::Lua) -> LuaResult<()> {
    if matches!(LuaApiMut::get_global_val(lua, "__secureenv"), Val::Table(_)) {
        return Ok(());
    }

    let chunk = LuaApiMut::load(lua, CREATE_SECURE_ENV_LUA)?;
    let mut values = lua.call_function(&chunk, &[])?;
    let base_names = values
        .pop()
        .ok_or_else(|| runtime_error("create_secure_environment: missing base-name set"))?;
    let secureenv = values
        .pop()
        .ok_or_else(|| runtime_error("create_secure_environment: missing secureenv table"))?;
    if !matches!(secureenv, Val::Table(_)) || !matches!(base_names, Val::Table(_)) {
        return Err(runtime_error(
            "create_secure_environment: expected secureenv table and base-name set",
        ));
    }

    {
        let state = lua.state_mut();
        registry_set(state, "__secureenv", secureenv);
        registry_set(state, SECURE_ENV_BASE_NAMES_KEY, base_names);
    }
    LuaApiMut::set_global_val(lua, "__secureenv", secureenv)?;
    Ok(())
}

/// Create the temporary full-snapshot proxy used by Retail 12.1.5 secure replays
/// and generated chunks. Other profiles use `__secureenv` directly.
///
/// Non-control reads prefer live secureenv values. Names absent there retain
/// snapshot behavior; assignments publish to the strict secure environment.
// temporary: lt-635e. Retail 12.1.5 only; other profiles replay directly in
// secureenv, as on master (their native behavior is UNKNOWN).
pub fn create_secure_replay_environment(lua: &mut rilua::Lua) -> LuaResult<()> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }
    if matches!(
        registry_get(lua.state_mut(), "__secureenv_replay"),
        Val::Table(_)
    ) {
        return Ok(());
    }

    let chunk = LuaApiMut::load(lua, CREATE_SECURE_REPLAY_ENV_LUA)?;
    let replay_env = lua
        .call_function(&chunk, &[])?
        .into_iter()
        .next()
        .ok_or_else(|| runtime_error("create_secure_replay_environment: missing return value"))?;
    let Val::Table(_) = replay_env else {
        return Err(runtime_error(
            "create_secure_replay_environment: expected replay table",
        ));
    };
    registry_set(lua.state_mut(), "__secureenv_replay", replay_env);
    Ok(())
}

/// Environment for secure replays and generated secure chunks: the lt-635e
/// replay proxy on Retail 12.1.5, `__secureenv` itself elsewhere (master).
pub(crate) fn secure_replay_env_table(
    state: &mut LuaState,
) -> LuaResult<rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>> {
    if !crate::client_profile::is_active_retail_12_1_5() {
        return secure_env_table(state);
    }
    match registry_get(state, "__secureenv_replay") {
        Val::Table(table_ref) => Ok(table_ref),
        _ => Err(runtime_error("secure replay environment not initialized")),
    }
}

/// Mark a secure replay function for its profile-selected environment:
/// Retail 12.1.5 uses the lt-635e proxy; other profiles use `__secureenv`.
// temporary: lt-635e
pub(crate) fn mark_secure_replay_state(
    state: &mut LuaState,
    func: &rilua::Function,
) -> LuaResult<()> {
    let replay_env_ref = secure_replay_env_table(state)?;
    let replay_env = rilua::Table::from_gc_ref(replay_env_ref);
    rilua::api::state_set_fenv(state, func, &replay_env)
}

/// Mark a compiled function as running under the secure environment.
///
/// Swaps the function's fenv to the registry-stored `__secureenv` table so
/// the closure and every inner closure it creates see `secureenv` as their
/// globals table. Matches Blizzard's `setfenv(chunk, secureenv)` step for
/// `[LoadIntoEnvironment secure]` files — caller never passes secureenv
/// explicitly; it's looked up via the registry.
pub fn mark_secure(lua: &mut rilua::Lua, func: &rilua::Function) -> LuaResult<()> {
    mark_secure_state(lua.state_mut(), func)
}

/// Raw-state variant of [`mark_secure`] for callers holding `&mut LuaState`
/// (e.g. inside a `with_state` closure or a `RustFn`).
pub fn mark_secure_state(state: &mut LuaState, func: &rilua::Function) -> LuaResult<()> {
    let secureenv_ref = secure_env_table(state)?;
    let secureenv = rilua::Table::from_gc_ref(secureenv_ref);
    rilua::api::state_set_fenv(state, func, &secureenv)
}

/// Return the two publication destinations for a flag-bearing native entry.
pub(crate) const fn native_registration_destinations(flags: u8) -> (bool, bool) {
    match flags % NATIVE_REGISTRATION_FLAG_MODULUS {
        0 => (false, false),
        1 => (true, false),
        2 => (false, true),
        _ => (true, true),
    }
}

/// Register one flag-bearing native function. Flag 3 constructs a separate closure per table;
/// secure publication requires an existing table at the shared `__secureenv` registry key.
pub(crate) fn register_native_function_with_flags(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
    flags: u8,
) -> LuaResult<()> {
    let (publish_global, publish_secure) = native_registration_destinations(flags);
    let state = lua.state_mut();
    let global_ref = state.global;
    if publish_global {
        table_set_rust_fn(state, global_ref, name, function)?;
        if publish_secure {
            if flags == 3 {
                register_flag3_handler_in_secure_environment(state, name, function)?;
            } else {
                let value = crate::lua_api::methods::table_get(state, Val::Table(global_ref), name);
                set_secure_env_value_if_present(state, name, value)?;
            }
        }
    } else if publish_secure && let Val::Table(secureenv_ref) = registry_get(state, "__secureenv") {
        table_set_rust_fn(state, secureenv_ref, name, function)?;
    }
    Ok(())
}

fn register_flag3_handler_in_secure_environment(
    state: &mut LuaState,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    let Val::Table(secureenv_ref) = registry_get(state, "__secureenv") else {
        return Ok(());
    };
    table_set_rust_fn_unpaired(state, secureenv_ref, name, function)
}

/// Install a missing native function or publish the existing function value by its flags.
pub(crate) fn register_native_function_if_missing_with_flags(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
    flags: u8,
) -> LuaResult<()> {
    match LuaApiMut::get_global_val(lua, name) {
        Val::Nil => register_native_function_with_flags(lua, name, function, flags),
        existing @ Val::Function(_) => publish_global_value_with_flags(lua, name, existing, flags),
        _ => Ok(()),
    }
}

fn publish_global_value_with_flags(
    lua: &mut rilua::Lua,
    name: &str,
    value: Val,
    flags: u8,
) -> LuaResult<()> {
    let (publish_global, publish_secure) = native_registration_destinations(flags);
    if publish_global {
        LuaApiMut::set_global_val(lua, name, value)?;
    }
    if publish_secure {
        set_secure_env_key_state(lua.state_mut(), name, value)?;
    }
    Ok(())
}

fn set_secure_env_value_if_present(state: &mut LuaState, name: &str, value: Val) -> LuaResult<()> {
    let Val::Table(secureenv_ref) = registry_get(state, "__secureenv") else {
        return Ok(());
    };
    let key_ref = state.gc.intern_string(name.as_bytes());
    let secureenv = rilua::Table::from_gc_ref(secureenv_ref);
    secureenv.raw_set(state, Val::Str(key_ref), value)?;
    state.gc.barrier_back(secureenv_ref);
    Ok(())
}

fn register_rust_function_and_secure_copy(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    LuaApiMut::register_function(lua, name, function)?;
    let value = LuaApiMut::get_global_val(lua, name);
    set_secure_env_value_if_present(lua.state_mut(), name, value)
}

/// Preserve a current dual-publication entry whose native flags are UNKNOWN.
pub(crate) fn register_unknown_allowlist_function(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    register_rust_function_and_secure_copy(lua, name, function)
}

/// Register a native pair-list entry with no per-name flags.
pub(crate) fn register_pair_list_function(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    register_rust_function_and_secure_copy(lua, name, function)
}

/// Publish a simulator-only helper to both environments. This is not a native
/// registration or evidence of a WoW global.
pub(crate) fn register_simulator_secure_helper(
    lua: &mut rilua::Lua,
    name: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    register_rust_function_and_secure_copy(lua, name, function)
}

#[cfg(test)]
mod tests {
    use super::{
        register_native_function_if_missing_with_flags, register_native_function_with_flags,
        register_pair_list_function, registry_get,
    };
    use crate::lua_api::WowLuaEnv;
    use rilua::vm::state::LuaState;
    use rilua::{LuaApiMut, LuaResult, Val};

    const NATIVE_GLOBAL_FLAGS_PROBE: &str = r#"
                local function exists(table, name)
                    return type(rawget(table, name)) == "function"
                end
                return {
                    exists(_G, "flagordinary"), exists(__secureenv, "flagordinary"),
                    exists(_G, "flagsecure"), exists(__secureenv, "flagsecure"),
                    exists(_G, "flagboth"), exists(__secureenv, "flagboth"),
                    exists(_G, "flagbothseven"), exists(__secureenv, "flagbothseven"),
                    exists(_G, "flagneither"), exists(__secureenv, "flagneither"),
                }
                "#;
    const FLAG3_GLOBAL_HANDLERS_PROBE: &str = r#"
                local ordinary3 = rawget(_G, "flagshared3")
                local secure3 = rawget(__secureenv, "flagshared3")
                local ordinary7 = rawget(_G, "flagshared7")
                local secure7 = rawget(__secureenv, "flagshared7")
                return {
                    type(ordinary3) == "function",
                    type(secure3) == "function",
                    not rawequal(ordinary3, secure3),
                    ordinary3() == true,
                    secure3() == true,
                    type(ordinary7) == "function",
                    type(secure7) == "function",
                    ordinary7() == true,
                    secure7() == true,
                }
                "#;
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    const FIXED_BASE_COPY_PROBE: &str = r#"
                local secure = __secureenv
                local probe = "__secure_env_copy_probe"
                local before = rawget(math, probe)
                rawset(math, probe, true)
                local secureEnum = rawget(secure, "Enum")
                local secureTooltipDataType = secureEnum and rawget(secureEnum, "TooltipDataType")
                local secureBit = rawget(secure, "bit")
                local ordinaryBit = rawget(_G, "bit")
                local result = {
                    type(rawget(secure, "assert")) == "function",
                    type(secureEnum) == "table"
                        and secureEnum ~= Enum
                        and type(secureTooltipDataType) == "table"
                        and type(rawget(secureTooltipDataType, "Unit")) == "number",
                    rawget(secure, "TooltipDataProcessor") == nil,
                    rawget(secure, "math") ~= math,
                    rawget(secure, "math").abs == math.abs
                        and rawget(rawget(secure, "math"), probe) == nil,
                    type(secureBit) == "table"
                        and secureBit ~= ordinaryBit
                        and secureBit.band == ordinaryBit.band,
                }
                rawset(math, probe, before)
                return result[1], result[2], result[3], result[4], result[5], result[6]
                "#;
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    const SECURE_LIBRARY_ALIASES_PROBE: &str = r#"
                local secure = __secureenv
                local secureString = rawget(secure, "string")
                local secureMath = rawget(secure, "math")
                return type(string.concat) == "function"
                        and rawequal(string.concat, strconcat)
                        and rawequal(secureString.concat, string.concat),
                    type(string.split) == "function"
                        and rawequal(string.split, strsplit),
                    type(secureString.split) == "function"
                        and rawequal(secureString.split, strsplit),
                    type(string.join) == "function"
                        and rawequal(string.join, strjoin)
                        and rawequal(secureString.join, string.join),
                    secureString ~= string,
                    secureMath ~= math and rawequal(secureMath.random, math.random),
                    type(math.round) == "function"
                        and type(secureMath.round) == "function",
                    type(string.contains) == "function"
                        and type(secureString.contains) == "function",
                    type(fastrandom) == "function"
                        and rawequal(fastrandom, math.random)
                        and rawget(secure, "fastrandom") == nil
                "#;
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    const MISSING_SECURE_BASE_MEMBERS_PROBE: &str = r#"
                    local names = {__VERIFIED_BASE_NAMES__}
                    local missing = {}
                    for _, name in ipairs(names) do
                        if rawget(_G, name) ~= nil and rawget(__secureenv, name) == nil then
                            missing[#missing + 1] = name
                        end
                    end
                    return missing
                "#;
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    const SEED_SECURE_ENVIRONMENT_PROBE: &str = r#"
                local names = {__VERIFIED_BASE_NAMES__}
                for _, name in ipairs(names) do
                    if rawget(_G, name) == nil then rawset(_G, name, true) end
                end
                rawset(_G, "coroutine", {})
                rawset(_G, "os", {})
                rawset(_G, "__secureenv", nil)
            "#;
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    const SECURE_ENVIRONMENT_BASE_PROBE: &str = r#"
                local names = {}
                for name in pairs(__secureenv) do
                    names[#names + 1] = name
                end
                return names,
                    rawget(__secureenv, "coroutine") == nil
                        and rawget(__secureenv, "os") == nil
                "#;

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    // R88n.11 part 1: exact Retail 12.1.5.70688 constructor copy list. Evidence: pr46-49-decomp-answers.md, sections R88n.11.
    const VERIFIED_BASE_NAMES: [&str; 39] = [
        "assert",
        "bit",
        "date",
        "error",
        "forceinsecure",
        "GetCurrentEnvironment",
        "geterrorhandler",
        "getfenv",
        "GetGlobalEnvironment",
        "getmetatable",
        "ipairs",
        "issecure",
        "math",
        "next",
        "pairs",
        "pcall",
        "pcallwithenv",
        "rawget",
        "rawset",
        "securecall",
        "securecallfunction",
        "secureexecuterange",
        "select",
        "setfenv",
        "setmetatable",
        "string",
        "strcmputf8i",
        "strconcat",
        "strjoin",
        "strlenutf8",
        "strsplit",
        "table",
        "time",
        "tonumber",
        "tostring",
        "type",
        "unpack",
        "IsInGlobalEnvironment",
        "SwapToGlobalEnvironment",
    ];

    fn flag_probe(state: &mut LuaState) -> LuaResult<u32> {
        state.push(Val::Bool(true));
        Ok(1)
    }

    #[test]
    fn native_global_flags_select_publication_destinations() {
        let env = WowLuaEnv::new().expect("lua env");
        {
            let mut lua = env.rilua_mut();
            for (name, flags) in [
                ("flagordinary", 1),
                ("flagsecure", 2),
                ("flagboth", 3),
                ("flagbothseven", 7),
                ("flagneither", 4),
            ] {
                register_native_function_with_flags(&mut lua, name, flag_probe, flags)
                    .expect("register flagged function");
            }
        }
        let result: Vec<bool> = env
            .eval(NATIVE_GLOBAL_FLAGS_PROBE)
            .expect("flag destination probe");
        assert_eq!(
            result,
            vec![
                true, false, false, true, true, true, true, true, false, false
            ]
        );
    }

    #[test]
    fn flag3_global_handlers_are_distinct_and_flag7_handlers_still_work() {
        let env = WowLuaEnv::new().expect("lua env");
        {
            let mut lua = env.rilua_mut();
            register_native_function_with_flags(&mut lua, "flagshared3", flag_probe, 3)
                .expect("register flag-3 function");
            register_native_function_with_flags(&mut lua, "flagshared7", flag_probe, 7)
                .expect("register flag-7 function");
        }
        let routes: Vec<bool> = env
            .eval(FLAG3_GLOBAL_HANDLERS_PROBE)
            .expect("flagged function publication probe");
        assert_eq!(
            routes,
            vec![true, true, true, true, true, true, true, true, true]
        );
    }

    #[test]
    fn flagged_missing_registration_preserves_an_existing_function() {
        let env = WowLuaEnv::new().expect("lua env");
        env.exec("flagpreserved = function() return 37 end")
            .expect("install existing function");
        {
            let mut lua = env.rilua_mut();
            register_native_function_if_missing_with_flags(
                &mut lua,
                "flagpreserved",
                flag_probe,
                7,
            )
            .expect("publish existing function");
        }

        let preserved: bool = env
            .eval(
                r#"
                local ordinary = rawget(_G, "flagpreserved")
                local secure = rawget(__secureenv, "flagpreserved")
                return type(ordinary) == "function"
                    and type(secure) == "function"
                    and ordinary() == 37
                    and secure() == 37
                "#,
            )
            .expect("existing function behavior probe");

        assert!(preserved);
    }

    #[test]
    fn secure_flag_publication_requires_an_existing_secure_table() {
        let mut lua = rilua::Lua::new().expect("lua state");
        register_native_function_with_flags(&mut lua, "secureonlywithoutenv", flag_probe, 2)
            .expect("register secure-only function");
        register_native_function_with_flags(&mut lua, "bothwithoutenv", flag_probe, 3)
            .expect("register dual function");
        register_pair_list_function(&mut lua, "pairwithoutenv", flag_probe)
            .expect("register pair-list function");

        assert!(matches!(
            LuaApiMut::get_global_val(&mut lua, "secureonlywithoutenv"),
            Val::Nil
        ));
        assert!(matches!(
            LuaApiMut::get_global_val(&mut lua, "bothwithoutenv"),
            Val::Function(_)
        ));
        assert!(matches!(
            LuaApiMut::get_global_val(&mut lua, "pairwithoutenv"),
            Val::Function(_)
        ));
        assert!(!matches!(
            registry_get(lua.state_mut(), "__secureenv"),
            Val::Table(_)
        ));
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    #[test]
    fn secure_environment_uses_fixed_base_and_shallow_table_copies() {
        let env = WowLuaEnv::new().expect("lua env");
        let result: (bool, bool, bool, bool, bool, bool) = env
            .eval(FIXED_BASE_COPY_PROBE)
            .expect("secure environment base probe");
        assert_eq!(result, (true, true, true, true, true, true));
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    #[test]
    fn secure_environment_copies_retail_base_functions() {
        let env = WowLuaEnv::new().expect("lua env");
        let copied: bool = env
            .eval(
                r#"
                    local secure = __secureenv
                    local function secureCopiesGlobalFunction(name)
                        local globalFunction = rawget(_G, name)
                        return type(globalFunction) == "function"
                            and rawget(secure, name) == globalFunction
                    end
                    return secureCopiesGlobalFunction("strjoin")
                        and secureCopiesGlobalFunction("strsplit")
                        and secureCopiesGlobalFunction("time")
                "#,
            )
            .expect("retail secure base functions should be readable");

        assert!(copied);
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    #[test]
    fn secure_library_aliases_follow_base_copy_and_keep_random_identity() {
        let env = WowLuaEnv::new().expect("lua env");
        let contract: (bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
            .eval(SECURE_LIBRARY_ALIASES_PROBE)
            .expect("verified library initialization routes should be observable");
        assert_eq!(
            contract,
            (true, true, true, true, true, true, true, true, true)
        );
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    #[test]
    fn secure_environment_contains_exact_verified_base_list() {
        let env = WowLuaEnv::new().expect("lua env");
        let names = verified_base_names_argument();
        let missing_secure_members = read_missing_secure_base_members(&env, &names);
        assert_no_missing_secure_base_members(missing_secure_members);
        seed_secure_environment_constructor_inputs(&env, &names);
        super::create_secure_environment(&mut env.rilua_mut())
            .expect("recreate secure environment from seeded globals");
        let (actual, excluded_standard_libraries) = read_secure_environment_base(&env);
        let expected = expected_secure_environment_base();
        assert_secure_environment_matches_base(actual, expected, excluded_standard_libraries);
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn verified_base_names_argument() -> String {
        VERIFIED_BASE_NAMES
            .iter()
            .map(|name| format!("{name:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn read_missing_secure_base_members(env: &WowLuaEnv, names: &str) -> Vec<String> {
        let probe = MISSING_SECURE_BASE_MEMBERS_PROBE.replace("__VERIFIED_BASE_NAMES__", names);
        env.eval(&probe).expect("compare installed base globals")
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn assert_no_missing_secure_base_members(missing_secure_members: Vec<String>) {
        assert!(
            missing_secure_members.is_empty(),
            "ordinary base globals missing from secureenv: {missing_secure_members:?}"
        );
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn seed_secure_environment_constructor_inputs(env: &WowLuaEnv, names: &str) {
        let seed = SEED_SECURE_ENVIRONMENT_PROBE.replace("__VERIFIED_BASE_NAMES__", names);
        env.exec(&seed).expect("seed constructor inputs");
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn read_secure_environment_base(env: &WowLuaEnv) -> (Vec<String>, bool) {
        env.eval(SECURE_ENVIRONMENT_BASE_PROBE)
            .expect("read secure-environment base")
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn expected_secure_environment_base() -> Vec<String> {
        let mut expected = VERIFIED_BASE_NAMES
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>();
        expected.push("_G".to_string());
        expected.sort_unstable();
        expected
    }

    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    fn assert_secure_environment_matches_base(
        mut actual: Vec<String>,
        expected: Vec<String>,
        excluded_standard_libraries: bool,
    ) {
        actual.sort_unstable();
        assert_eq!(actual, expected);
        assert!(excluded_standard_libraries);
    }
}
