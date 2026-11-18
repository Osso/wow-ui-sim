//! temporary: master's shallow `_G` snapshot into `__secureenv` for profiles
//! without native secure-publication evidence.
//!
//! UNKNOWN: native secure publication for Mists, PTR, WoW Forever, Wrath, Era
//! and Anniversary. Only Retail 12.1.5.70688 has the verified fixed 39-name Evidence: pr46-49-decomp-answers.md, sections R88g.1, R88y.y1.
//! base plus the R88y publisher. Every other profile keeps master's behavior Evidence: pr46-49-decomp-answers.md, sections R88y.
//! (copy every `_G` pair after protected-global wrapping, shallow-copy `Enum`,
//! log global nil reads). Retire this module per profile once that profile has
//! native publication evidence and its own publisher.

use rilua::{LuaApiMut, LuaResult};

const COPY_GLOBALS_INTO_SECUREENV_LUA: &str = r##"
    local genv = _G
    local secureenv = rawget(genv, "__secureenv")
    if type(secureenv) ~= "table" then
        error("secure global snapshot requires initialized secureenv")
    end
    local logNilSymbolAccess = rawget(genv, "__wow_log_nil_symbol_access")
    local debugTable = rawget(genv, "debug")

    for key, value in pairs(genv) do
        if key ~= "__secureenv" then
            rawset(secureenv, key, value)
        end
    end
    if type(rawget(genv, "Enum")) == "table" then
        local enumCopy = {}
        for key, value in pairs(rawget(genv, "Enum")) do
            enumCopy[key] = value
        end
        rawset(secureenv, "Enum", enumCopy)
    end
    rawset(secureenv, "_G", secureenv)

    local metatable = getmetatable(secureenv)
    metatable.__index = function(_, key)
        local isGlobalIndex = debugTable and debugTable.isglobalindex
        if type(isGlobalIndex) == "function" and isGlobalIndex()
                and type(logNilSymbolAccess) == "function" then
            logNilSymbolAccess("__secureenv", key)
        end
        return nil
    end
"##;

/// Copy the ordinary globals into secureenv unless the active profile has the
/// verified Retail 12.1.5 publication model. Evidence: pr46-49-decomp-answers.md, sections R88g.1, R88y.y1.
pub(crate) fn copy_globals_into_secureenv_without_publication_evidence(
    lua: &mut rilua::Lua,
) -> LuaResult<()> {
    if crate::client_profile::is_active_retail_12_1_5() {
        return Ok(());
    }
    let chunk = LuaApiMut::load(lua, COPY_GLOBALS_INTO_SECUREENV_LUA)?;
    lua.call_function(&chunk, &[])?;
    Ok(())
}
