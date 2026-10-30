//! Temporary display scaling until unmodified unit power is modeled.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Retail UnitDocumentation.lua:2697 and Forever :2786.
    // User-approved default, not native evidence: the current UnitPower model
    // does not scale unmodified values. Retire when that model gains scaling.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        lua.exec("function UnitPowerDisplayMod(powerType) return 1 end")?;
    }
    Ok(())
}
