//! PTR-only default. Retail and Forever use the supplied C_QuestLog model.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if ACTIVE == ClientProfile::Ptr {
        lua.exec(
            r#"
            -- User-approved PTR default; native behavior remains unknown.
            function C_QuestLog.GetZoneStoryInfo(_uiMapID) end
        "#,
        )?;
    }
    Ok(())
}
