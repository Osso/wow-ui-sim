//! Forever fallback; its active TOC excludes the Mainline ShardBar consumer.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Forever UnitDocumentation.lua:2786. Retail uses the supplied state model.
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        lua.exec("function UnitPowerDisplayMod(powerType) return 1 end")?;
    }
    Ok(())
}
