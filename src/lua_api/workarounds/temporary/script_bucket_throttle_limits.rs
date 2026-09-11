//! Explicit PTR mock: zeros are placeholders, not native limits or disabled semantics.
//! Replace only when authoritative return values are available; enforcement is out of scope.

#[cfg(feature = "retail-12-1-5")]
pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(
        r#"
        if rawget(_G, "GetScriptBucketThrottleLimits") == nil then
            function GetScriptBucketThrottleLimits()
                return {
                    luaScriptBucketThrottleMaxMsPerSecondNormal = 0,
                    luaScriptBucketThrottleMaxMsPerSecondRestricted = 0,
                    luaScriptBucketThrottleMaxMsBurstNormal = 0,
                    luaScriptBucketThrottleMaxMsBurstRestricted = 0,
                }
            end
        end
        "#,
    )?;
    Ok(())
}

#[cfg(not(feature = "retail-12-1-5"))]
pub(crate) fn apply_bootstrap(_lua: &mut rilua::Lua) -> crate::Result<()> {
    Ok(())
}
