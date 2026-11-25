//! Temporary unsupported priority query; retire when native arbitration is modeled.
//! Target storage and mutations belong exclusively to c_api::c_super_track.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }
    lua.exec(
        r#"
        C_SuperTrack = C_SuperTrack or __wow_namespace()
        function C_SuperTrack.GetHighestPrioritySuperTrackingType() return nil end
    "#,
    )?;
    Ok(())
}
