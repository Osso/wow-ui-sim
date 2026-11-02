//! User-approved offline Forever glue presentation defaults, not native policy.
//!
//! Retire the realm-button default when realm-list/account presentation is modeled.
//! Retire the nil display record when current game-mode display data is modeled.
//! Retire the empty subregion/district catalogs when login realm discovery is modeled.
//! Raw guards preserve modeled producers when they become available.

const OFFLINE_GLUE_DISPLAY_DEFAULTS: &str = r#"
local realms = rawget(_G, "C_RealmList")
if realms == nil then
    realms = {}
    rawset(_G, "C_RealmList", realms)
end
if rawget(realms, "ShouldShowRealmsAsCheatButton") == nil then
    rawset(realms, "ShouldShowRealmsAsCheatButton", function()
        return false
    end)
end

local login = rawget(_G, "C_Login")
if login == nil then
    login = {}
    rawset(_G, "C_Login", login)
end
if rawget(login, "GetSubRegions") == nil then
    rawset(login, "GetSubRegions", function()
        return {}
    end)
end

local districts = rawget(_G, "C_DistrictUtils")
if districts == nil then
    districts = {}
    rawset(_G, "C_DistrictUtils", districts)
end
if rawget(districts, "GetAvailableSuperDistricts") == nil then
    rawset(districts, "GetAvailableSuperDistricts", function()
        return {}
    end)
end

local rules = rawget(_G, "C_GameRules")
if rules == nil then
    rules = {}
    rawset(_G, "C_GameRules", rules)
end
if rawget(rules, "GetCurrentGameModeDisplayInfo") == nil then
    rawset(rules, "GetCurrentGameModeDisplayInfo", function()
        return nil
    end)
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        lua.exec(OFFLINE_GLUE_DISPLAY_DEFAULTS)?;
    }
    Ok(())
}
