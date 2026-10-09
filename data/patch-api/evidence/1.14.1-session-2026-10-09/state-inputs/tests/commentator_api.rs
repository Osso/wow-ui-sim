use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn commentator_send_addon_message_returns_success() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local explicit = C_Commentator.SendAddonMessage("COOLDOWNBROADCASTER", "1,2,3", "RAID")
            if explicit ~= 0 then
                return "explicit_channel_should_return_success"
            end

            local implicit = C_Commentator.SendAddonMessage("COOLDOWNBROADCASTER", "1,2,3")
            if implicit ~= 0 then
                return "default_channel_should_return_success"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_Commentator.SendAddonMessage should return SendAddonMessageResult.Success"
    );
}
