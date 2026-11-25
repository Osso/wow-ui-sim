#![cfg(not(feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn reincarnation_defaults_to_inactive_with_no_character() {
    let env = env();
    let (active, character_is_nil): (bool, bool) = env
        .eval(
            r#"
            return C_Reincarnation.IsReincarnating(),
                   C_Reincarnation.GetReincarnatingCharacter() == nil
            "#,
        )
        .unwrap();

    assert!(!active, "reincarnation should start inactive");
    assert!(character_is_nil, "no reincarnating character should be set");
}

#[test]
fn reincarnation_start_sets_default_character_state() {
    let env = env();
    let (started, active, has_guid, has_name): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local started = C_Reincarnation.StartReincarnation()
            local character = C_Reincarnation.GetReincarnatingCharacter()
            return started,
                   C_Reincarnation.IsReincarnating(),
                   character and type(character.guid) == "string",
                   character and type(character.name) == "string"
            "#,
        )
        .unwrap();

    assert!(started, "first StartReincarnation call should succeed");
    assert!(active, "state should become active after start");
    assert!(has_guid, "default character should include guid");
    assert!(has_name, "default character should include name");
}

#[test]
fn reincarnation_start_rejects_when_already_active() {
    let env = env();
    let (first_started, second_started, selection_stable): (bool, bool, bool) = env
        .eval(
            r#"
            local firstStarted = C_Reincarnation.StartReincarnation({ guid = "first-guid", name = "First" })
            local secondStarted = C_Reincarnation.StartReincarnation({ guid = "second-guid", name = "Second" })
            local current = C_Reincarnation.GetReincarnatingCharacter()
            return firstStarted, secondStarted, current and current.guid == "first-guid"
            "#,
        )
        .unwrap();

    assert!(first_started, "first start should succeed");
    assert!(!second_started, "second start should fail while active");
    assert!(
        selection_stable,
        "failed second start should not replace active character"
    );
}

#[test]
fn reincarnation_stop_clears_active_state() {
    let env = env();
    let (stop_when_active, active_after_stop, character_cleared, stop_when_inactive): (
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            C_Reincarnation.StartReincarnation({ guid = "stop-guid", name = "Stop" })
            local stopWhenActive = C_Reincarnation.StopReincarnation()
            local activeAfterStop = C_Reincarnation.IsReincarnating()
            local characterCleared = C_Reincarnation.GetReincarnatingCharacter() == nil
            local stopWhenInactive = C_Reincarnation.StopReincarnation()
            return stopWhenActive, activeAfterStop, characterCleared, stopWhenInactive
            "#,
        )
        .unwrap();

    assert!(
        stop_when_active,
        "StopReincarnation should report true when active"
    );
    assert!(!active_after_stop, "stop should clear active state");
    assert!(
        character_cleared,
        "stop should clear reincarnating character"
    );
    assert!(
        !stop_when_inactive,
        "stopping while already inactive should report false"
    );
}

#[test]
fn reincarnation_rejects_invalid_start_inputs() {
    let env = env();
    let (started, still_inactive): (bool, bool) = env
        .eval(
            r#"
            local started = C_Reincarnation.StartReincarnation(function() end)
            return started, not C_Reincarnation.IsReincarnating()
            "#,
        )
        .unwrap();

    assert!(!started, "invalid input should fail to start reincarnation");
    assert!(
        still_inactive,
        "invalid start input should not change active reincarnation state"
    );
}
