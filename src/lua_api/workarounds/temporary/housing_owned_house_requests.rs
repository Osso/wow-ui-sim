//! Temporary asynchronous owned-house requests, available before dashboard load.
//! Missing system: authoritative player-owned housing service/catalog.
//! Retire when that service supplies PLAYER_HOUSE_LIST_UPDATED responses.

pub(crate) const OWNED_HOUSE_REQUESTS_LUA: &str = r#"
if type(C_Housing) == "table" and rawget(C_Housing, "GetPlayerOwnedHouses") == nil then
    function C_Housing.GetPlayerOwnedHouses()
        C_Timer.After(0, function()
            FireEvent("PLAYER_HOUSE_LIST_UPDATED", {})
        end)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    if ACTIVE == ClientProfile::Ptr {
        lua.exec(OWNED_HOUSE_REQUESTS_LUA)?;
    }
    Ok(())
}
