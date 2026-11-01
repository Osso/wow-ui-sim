//! Camera motion is outside the simulator's intentionally unsupported 3D domain.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Wrath
    ) {
        return Ok(());
    }
    lua.exec(
        r#"
        C_Commentator = C_Commentator or {}
        function C_Commentator.SetFollowCameraSpeeds(elasticSpeed, minSpeed)
            if type(elasticSpeed) ~= "number" then error("elasticSpeed must be a number", 2) end
            if type(minSpeed) ~= "number" then error("minSpeed must be a number", 2) end
        end
    "#,
    )?;
    Ok(())
}
