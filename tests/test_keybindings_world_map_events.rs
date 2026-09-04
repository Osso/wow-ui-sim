//! Integration tests for world-map event tabs and Escape keybinding panels.

use crate::common;
#[path = "common/keybindings_panels_detail.rs"]
mod keybindings_panels_detail;
#[path = "common/token_ui_fixtures.rs"]
mod token_ui_fixtures;

use keybindings_panels_detail::{drain_test_errors, frame_is_shown, install_test_error_handler};
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn world_map_events_tab_click_and_zone_switch_without_errors(env: &WowLuaEnv) {
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let events_tab_id = {
            let state = env.state().borrow();
            let quest_map_id = state
                .widgets
                .get_id_by_name("QuestMapFrame")
                .expect("QuestMapFrame should exist after opening the world map");
            state
                .widgets
                .get(quest_map_id)
                .and_then(|frame| frame.children_keys.get("EventsTab").copied())
                .expect("QuestMapFrame.EventsTab should exist after opening the world map")
        };

        env.send_click(events_tab_id)
            .expect("clicking QuestMapFrame.EventsTab failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                if not (QuestMapFrame and QuestMapFrame.EventsTab and QuestMapFrame.EventsTab:IsShown()) then
                    return "events_tab_not_shown"
                end

                if QuestMapFrame.displayMode ~= QuestLogDisplayMode.Events then
                    return "events_tab_not_selected"
                end

                C_Map.SetMapForQuestLog(1)

                if WorldMapFrame:GetMapID() ~= 1 then
                    return "quest_log_map_not_switched"
                end

                ToggleWorldMap()

                if WorldMapFrame:IsShown() then
                    return "world_map_not_closed"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "World map events tab flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "World map events tab flow should open, switch to events, change zone, and close: {result}"
        );
}
}

prefork_full_ui_case! {
fn quest_log_validate_tabs_shows_events_tab_when_scheduler_can_show_events(env: &WowLuaEnv) {
        install_test_error_handler(&env);

        env.send_key_press("M", None).expect("M keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (WorldMapFrame and WorldMapFrame:IsShown()) then
                    return "world_map_not_open"
                end

                if not (QuestMapFrame and QuestMapFrame.EventsTab) then
                    return "events_tab_missing"
                end

                C_EventScheduler._state.canShowEvents = true
                QuestMapFrame.EventsTab:Hide()
                QuestMapFrame:ValidateTabs()

                if not C_EventScheduler.CanShowEvents() then
                    return "scheduler_cannot_show_events"
                end

                if not QuestMapFrame.EventsTab:IsShown() then
                    return "events_tab_not_shown"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Quest log ValidateTabs flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Quest log ValidateTabs should show the Events tab when C_EventScheduler.CanShowEvents() is true: {result}"
        );
}
}

// ── ESCAPE → toggle GameMenuFrame ───────────────────────────────────────

prefork_full_ui_case! {
fn keybind_escape_opens_game_menu(env: &WowLuaEnv) {
        env.send_key_press("ESCAPE", None).expect("ESCAPE keybind failed");
        assert!(
            frame_is_shown(&env, "GameMenuFrame"),
            "GameMenuFrame should be shown after pressing ESCAPE"
        );
}
}

prefork_full_ui_case! {
fn keybind_escape_closes_game_menu(env: &WowLuaEnv) {
        env.send_key_press("ESCAPE", None).expect("first ESCAPE failed");
        assert!(frame_is_shown(&env, "GameMenuFrame"));
        env.send_key_press("ESCAPE", None).expect("second ESCAPE failed");
        assert!(
            !frame_is_shown(&env, "GameMenuFrame"),
            "GameMenuFrame should be hidden after second ESCAPE"
        );
}
}
