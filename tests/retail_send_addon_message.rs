#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_chat_info_send_addon_message_supports_dandersframes_call_shape() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().party_group_active = true;
    let result: String = env
        .eval(
            r#"
            assert(type(C_ChatInfo.SendAddonMessage) == "function")
            assert(select("#", C_ChatInfo.SendAddonMessage("DANDERSFRAMES", "H", "PARTY")) == 1)
            assert(C_ChatInfo.SendAddonMessage("DANDERSFRAMES", "H", "PARTY") == Enum.SendAddonMessageResult.Success)
            assert(C_ChatInfo.SendAddonMessage(123, 456, nil, nil) == Enum.SendAddonMessageResult.Success)
            assert(not pcall(C_ChatInfo.SendAddonMessage, "DANDERSFRAMES", "H", true))
            assert(not pcall(C_ChatInfo.SendAddonMessage, "DANDERSFRAMES"))
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
    let records = env.state().borrow();
    assert_eq!(records.message_log.len(), 3);
    assert_eq!(records.message_log[0].channel, "PARTY");
    assert_eq!(records.message_log[1].prefix, "123");
    assert_eq!(records.message_log[1].message, "456");
}
