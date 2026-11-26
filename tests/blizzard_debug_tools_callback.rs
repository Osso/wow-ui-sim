#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
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

#[cfg(feature = "client-wowforever")]
#[test]
fn devtools_message_handler_receives_distinct_dump_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("WoW Forever UI cache should be available");
    let dump_source = std::fs::read_to_string(ui.join("Blizzard_SharedXML/Dump.lua"))
        .expect("selected SharedXML dump implementation should be readable");
    env.exec(&dump_source)
        .expect("original DevTools dump implementation should load");
    let messages: Vec<String> = env
        .eval(
            r#"local messages = {}
            DevTools_AddMessageHandler(function(message) messages[#messages + 1] = tostring(message) end)
            DevTools_Dump("first supplied value", "first-key")
            DevTools_Dump("second supplied value", "second-key")
            return messages"#,
        )
        .expect("original dump consumer should dispatch handler messages");
    let output = messages.join("\n");
    assert!(output.contains("first supplied value"));
    assert!(output.contains("second supplied value"));
}
