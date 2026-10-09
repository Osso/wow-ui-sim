//! Temporary behavioral messaging receipt defaults.
//!
//! The simulator does not model behavioral-notification server state yet.
//! Blizzard_BehavioralMessaging only needs the receipt acknowledgement to be
//! callable, so keep this no-op outside the state-backed C API surface.

const BEHAVIORAL_MESSAGING_DEFAULTS_LUA: &str = r#"
C_BehavioralMessaging = C_BehavioralMessaging or __wow_namespace()
if rawget(C_BehavioralMessaging, "SendNotificationReceipt") == nil then
    function C_BehavioralMessaging.SendNotificationReceipt(_notificationID, _receiptType)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(BEHAVIORAL_MESSAGING_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_receipt_acknowledgement_noop() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: bool = env
            .eval(
                r##"
                local result_count = select("#", C_BehavioralMessaging.SendNotificationReceipt(1, "shown"))
                return result_count == 0
                "##,
            )
            .expect("receipt acknowledgement should be callable");

        assert!(result);
    }

    #[test]
    fn preserves_existing_receipt_acknowledgement() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_BehavioralMessaging.SendNotificationReceipt()
                return "modeled"
            end
            "#,
        )
        .expect("fixture should install existing function");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval("return C_BehavioralMessaging.SendNotificationReceipt()")
            .expect("existing receipt acknowledgement should remain callable");

        assert_eq!(result, "modeled");
    }
}
