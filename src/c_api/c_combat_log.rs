//! Combat-log filter records and profile publication boundaries.
//!
//! Current-event data belongs to Internal/Secure, not public C_CombatLog. Mark
//! the public member absent before generic namespace lookup can synthesize it.

#[cfg(feature = "client-wowforever")]
use crate::lua_api::methods::{create_table, table_set};
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[cfg(feature = "rest-policy-flags")]
pub(crate) fn register_restriction(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_CombatLog")?;
    crate::lua_bridge::table_set_rust_fn_static(state, ns, "IsCombatLogRestricted", |s| {
        let restricted = crate::lua_api::methods::borrow_state(s)?.combat_log_restricted;
        s.push(Val::Bool(restricted));
        Ok(1)
    })
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_publication(state: &mut LuaState) -> LuaResult<()> {
    let public = super::ensure_namespace(state, "C_CombatLog")?;
    let removed = create_table(state);
    table_set(state, removed, "GetCurrentEventInfo", Val::Bool(true));
    table_set(state, Val::Table(public), "__wow_removed_keys", removed);
    Ok(())
}

/// Retain opaque filter requests; native matching and validation are UNKNOWN.
/// The documentation declares restricted Add/Clear functions without signatures.
/// See docs/specs/combat-log-event-filters.md before adding interpretation here.
pub(crate) fn register_filters(lua: &mut rilua::Lua) -> LuaResult<()> {
    super::ensure_namespace(lua.state_mut(), "C_CombatLog")?;
    super::ensure_namespace(lua.state_mut(), "C_CombatLogSecure")?;
    lua.exec(FILTER_RECORDS_LUA)
}

const FILTER_RECORDS_LUA: &str = r##"
local queries = {}

local function CopyFilterArguments(record)
    local copy = { n = record.n }
    for index = 1, record.n do
        copy[index] = record[index]
    end
    return copy
end

local function RegisterFilterRecords(namespace)
    local filters = {}
    namespace.AddEventFilter = function(...)
        filters[#filters + 1] = { n = select("#", ...), ... }
    end
    namespace.ClearEventFilters = function()
        filters = {}
    end
    queries[namespace] = function()
        local snapshot = {}
        for index, record in ipairs(filters) do
            snapshot[index] = CopyFilterArguments(record)
        end
        return snapshot
    end
end

RegisterFilterRecords(C_CombatLog)

-- Retail's secure namespace is a separate table from the public namespace.
-- Older profiles publish C_CombatLogSecure on the ordinary global table.
local secureNamespace = C_CombatLogSecure
if __secureenv and __secureenv.C_CombatLogSecure then
    secureNamespace = __secureenv.C_CombatLogSecure
end
RegisterFilterRecords(secureNamespace)

-- Simulator diagnostic only, not a native filter query API. Keep records rooted
-- in closures, not untraced Rust Val fields. Public/secure stores stay separate.
function __wow_get_combat_log_event_filters(namespace)
    local query = queries[namespace]
    assert(query, "combat-log filter query requires a registered namespace")
    return query()
end
"##;
