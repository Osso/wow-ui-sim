//! Empty mission lists until the garrison mission model exists.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    lua.exec(
        r#"
        local function empty_missions(first, second)
            local result, followerTypeID
            if type(first) == "table" then
                result, followerTypeID = first, second
            elseif type(first) == "number" then
                result, followerTypeID = {}, first
            elseif first == nil then
                result, followerTypeID = {}, second
            else
                error("bad argument #1 (table, number or nil expected)", 2)
            end
            if type(followerTypeID) ~= "number" then
                error("bad argument (followerTypeID must be a number)", 2)
            end
            for key in pairs(result) do result[key] = nil end
            return result
        end
        C_Garrison.GetInProgressMissions = empty_missions
        C_Garrison.GetAvailableMissions = empty_missions
    "#,
    )?;
    Ok(())
}
