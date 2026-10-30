//! The simulator does not expose host window attention through Lua state.

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Retail and Forever ClientDocumentation.lua:10-16. User-approved default,
    // not native evidence: the host window effect is intentionally unsupported.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        lua.exec("function FlashClientIcon(briefly) end")?;
    }
    Ok(())
}
