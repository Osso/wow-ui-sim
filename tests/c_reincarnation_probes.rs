#![cfg(not(feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn reincarnation_defaults_start_and_stop_character_state() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_Reincarnation.IsReincarnating() ~= false then return "initial-active" end
            if C_Reincarnation.GetReincarnatingCharacter() ~= nil then return "initial-character" end
            if C_Reincarnation.StartReincarnation({ guid = "guid-1", name = "Ari" }) ~= true then return "start" end
            if C_Reincarnation.IsReincarnating() ~= true then return "active" end
            local character = C_Reincarnation.GetReincarnatingCharacter()
            if character.guid ~= "guid-1" or character.name ~= "Ari" then return "character" end
            if C_Reincarnation.StartReincarnation({ guid = "guid-2", name = "Bee" }) ~= false then return "double-start" end
            if C_Reincarnation.StopReincarnation() ~= true then return "stop" end
            if C_Reincarnation.IsReincarnating() ~= false then return "inactive" end
            if C_Reincarnation.GetReincarnatingCharacter() ~= nil then return "cleared" end
            if C_Reincarnation.StopReincarnation() ~= false then return "double-stop" end
            return "ok"
            "#,
        )
        .expect("reincarnation defaults should be callable");

    assert_eq!(result, "ok");
}

#[test]
fn reincarnation_start_rejects_non_table_and_uses_default_character() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_Reincarnation.StartReincarnation("bad") ~= false then return "bad" end
            if C_Reincarnation.StartReincarnation() ~= true then return "default-start" end
            local character = C_Reincarnation.GetReincarnatingCharacter()
            if character.guid ~= "reincarnation-guid" then return "default-guid" end
            if character.name ~= "Reincarnating Character" then return "default-name" end
            return "ok"
            "#,
        )
        .expect("reincarnation defaults should handle invalid/default starts");

    assert_eq!(result, "ok");
}
