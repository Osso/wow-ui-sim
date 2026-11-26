#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn devtools_message_handler_receives_distinct_dump_inputs(env: &WowLuaEnv) {
    let messages: Vec<String> = env
        .eval(
            r#"
            local messages = {}
            DevTools_AddMessageHandler(function(message)
                messages[#messages + 1] = tostring(message)
            end)
            DevTools_Dump("first supplied value", "first-key")
            DevTools_Dump("second supplied value", "second-key")
            return messages
            "#,
        )
        .expect("original DevTools dump consumer should dispatch handler messages");

    let output = messages.join("\n");
    assert!(output.contains("first supplied value"));
    assert!(output.contains("second supplied value"));
}
}
