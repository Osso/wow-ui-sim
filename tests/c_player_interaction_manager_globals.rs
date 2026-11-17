//! Integration tests for the `C_PlayerInteractionManager` surface registered
//! in `src/c_api/c_player_interaction_manager.rs`. The surface drives
//! `AlliedRacesFrameMixin:OnHide` (`Blizzard_AlliedRacesFrameUI.lua:156`),
//! which calls `ClearInteraction(Enum.PlayerInteractionType.AlliedRaceDetailsGiver)`
//! and expects an `ALLIED_RACE_CLOSE` event for any open listener.

use wow_ui_sim::lua_api::WowLuaEnv;

const ALLIED_RACE_DETAILS_GIVER: i32 = 9;

#[test]
fn clear_interaction_namespace_and_method_exist() {
    let env = WowLuaEnv::new().expect("env");
    let kind: String = env
        .eval("return type(C_PlayerInteractionManager.ClearInteraction)")
        .unwrap();
    assert_eq!(kind, "function");
}

#[test]
fn allied_race_details_giver_enum_value_is_nine() {
    let env = WowLuaEnv::new().expect("env");
    let value: f64 = env
        .eval("return Enum.PlayerInteractionType.AlliedRaceDetailsGiver")
        .unwrap();
    assert_eq!(
        value, ALLIED_RACE_DETAILS_GIVER as f64,
        "AlliedRaceDetailsGiver must match the canonical PlayerInteractionType ordinal"
    );
}

#[test]
fn clear_interaction_removes_active_entry() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .active_player_interactions
        .insert(ALLIED_RACE_DETAILS_GIVER);

    env.exec(&format!(
        "C_PlayerInteractionManager.ClearInteraction({ALLIED_RACE_DETAILS_GIVER})"
    ))
    .unwrap();

    let still_active = env
        .state()
        .borrow()
        .active_player_interactions
        .contains(&ALLIED_RACE_DETAILS_GIVER);
    assert!(!still_active, "ClearInteraction should remove the entry");
}

fn install_close_event_probe(env: &WowLuaEnv) {
    env.exec(
        r#"
        ManagerCloseEvents = {}
        ManagerHideType = nil
        local listener = CreateFrame("Frame", "PlayerInteractionCloseListener")
        listener:RegisterEvent("ALLIED_RACE_CLOSE")
        listener:RegisterEvent("PLAYER_INTERACTION_MANAGER_FRAME_HIDE")
        listener:SetScript("OnEvent", function(_, event, interactionType)
            table.insert(ManagerCloseEvents, event)
            if event == "PLAYER_INTERACTION_MANAGER_FRAME_HIDE" then
                ManagerHideType = interactionType
            end
        end)
        "#,
    )
    .unwrap();
}

fn assert_manager_close_sequence(env: &WowLuaEnv, events: &str, hide_type: i32) {
    env.exec(&format!(
        "assert(table.concat(ManagerCloseEvents, ',') == '{events}'); assert(ManagerHideType == {hide_type})"
    ))
    .unwrap();
}

fn assert_no_manager_close_events(env: &WowLuaEnv) {
    env.exec("assert(#ManagerCloseEvents == 0)").unwrap();
}

#[test]
fn clear_interaction_fires_allied_race_close_before_hide_when_active() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .active_player_interactions
        .insert(ALLIED_RACE_DETAILS_GIVER);
    install_close_event_probe(&env);

    env.exec(&format!(
        "C_PlayerInteractionManager.ClearInteraction({ALLIED_RACE_DETAILS_GIVER})"
    ))
    .unwrap();

    assert_manager_close_sequence(
        &env,
        "ALLIED_RACE_CLOSE,PLAYER_INTERACTION_MANAGER_FRAME_HIDE",
        ALLIED_RACE_DETAILS_GIVER,
    );
}

#[test]
fn clear_interaction_does_not_fire_close_events_when_not_active() {
    let env = WowLuaEnv::new().expect("env");
    assert!(env.state().borrow().active_player_interactions.is_empty());
    install_close_event_probe(&env);

    env.exec(&format!(
        "C_PlayerInteractionManager.ClearInteraction({ALLIED_RACE_DETAILS_GIVER})"
    ))
    .unwrap();

    assert_no_manager_close_events(&env);
}

#[test]
fn clear_interaction_with_unmapped_type_fires_only_generic_hide() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .active_player_interactions
        .insert(1);
    install_close_event_probe(&env);

    env.exec("C_PlayerInteractionManager.ClearInteraction(1)")
        .unwrap();

    let still_active = env.state().borrow().active_player_interactions.contains(&1);
    assert!(
        !still_active,
        "ClearInteraction should remove the active entry"
    );
    assert_manager_close_sequence(&env, "PLAYER_INTERACTION_MANAGER_FRAME_HIDE", 1);
}

#[test]
fn clear_interaction_missing_arg_is_a_noop() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .active_player_interactions
        .insert(ALLIED_RACE_DETAILS_GIVER);

    env.exec("C_PlayerInteractionManager.ClearInteraction()")
        .unwrap();

    let still_active = env
        .state()
        .borrow()
        .active_player_interactions
        .contains(&ALLIED_RACE_DETAILS_GIVER);
    assert!(
        still_active,
        "Missing arg must not silently clear arbitrary entries"
    );
}
