//! Retire when external event promotion state and URL launching are modeled.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        lua.exec(
            r#"
            C_ExternalEventURL = C_ExternalEventURL or {}
            -- User-approved simulator defaults, not evidence of native behavior.
            -- Missing system: external event promotion and URL launching.
            function C_ExternalEventURL.HasURL() return false end
            function C_ExternalEventURL.IsNew() return false end
            function C_ExternalEventURL.LaunchURL() end
        "#,
        )?;
        if ACTIVE == ClientProfile::Ptr {
            lua.exec("function C_ExternalEventURL.CanShow() return false end")?;
        }
    }
    Ok(())
}
