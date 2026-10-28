//! Retire when map-to-zone-story relationships are modeled.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        lua.exec(
            r#"
            -- User-approved simulator default, not evidence of native behavior.
            -- Missing system: map-to-zone-story achievement relationships.
            function C_QuestLog.GetZoneStoryInfo(_uiMapID) end
        "#,
        )?;
    }
    Ok(())
}
