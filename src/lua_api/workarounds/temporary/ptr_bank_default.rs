//! Retire when the bank bag-layout policy is modeled.

use crate::client_profile::{ACTIVE, ClientProfile};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if ACTIVE == ClientProfile::Ptr {
        lua.exec(
            r#"
            C_Bank = C_Bank or {}
            -- User-approved simulator default, not evidence of native behavior.
            -- Missing system: realm bank bag-layout policy.
            function C_Bank.ShouldUsePlayerBagsInBank()
                return false
            end
        "#,
        )?;
    }
    Ok(())
}
