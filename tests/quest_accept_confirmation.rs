//! Tests-first Retail 12.0.5 confirmation input; simulator policy, not native parity.
#![cfg(all(feature = "retail-12-0-5", feature = "profile-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const QUEST_ID: u32 = 80002;

fn create_confirmation_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create quest confirmation environment");
    env.exec(
        r#"
        promptCount = 0
        detailCount = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("QUEST_ACCEPT_CONFIRM")
        listener:RegisterEvent("QUEST_DETAIL")
        listener:SetScript("OnEvent", function(_, event, ...)
            if event == "QUEST_DETAIL" then
                detailCount = detailCount + 1
                return
            end
            assert(event == "QUEST_ACCEPT_CONFIRM")
            assert(select('#', ...) == 3, "confirmation needs exactly three arguments")
            local name, questTitle, questID = ...
            assert(type(name) == "string" and name == "Fixture quest sharer")
            assert(type(questTitle) == "string" and questTitle == "Explicit shared title")
            assert(type(questID) == "number" and questID == 80002)
            assert(C_QuestLog.GetSelectedQuest() == questID,
                "selected offer must be visible during callback")
            promptCount = promptCount + 1
            if acceptDuringPrompt then
                ConfirmAcceptQuest()
            end
        end)
        A_Admin.OpenQuestNpc(80002, "Different gossip title")
        "#,
    )
    .expect("seed actual gossip input and register real frame listeners");
    env
}

fn select_offer(env: &WowLuaEnv) {
    env.exec("C_GossipInfo.SelectAvailableQuest(80002)")
        .expect("select available quest through modeled API");
    assert_eq!(env.state().borrow().pending_quest_offer, Some(QUEST_ID));
}

fn request_confirmation(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(type(A_Admin.RequestQuestAcceptConfirmation) == "function",
            "explicit confirmation input must exist")
        A_Admin.RequestQuestAcceptConfirmation("Fixture quest sharer", "Explicit shared title")
        "#,
    )
    .expect("request confirmation using explicit name/title and existing pending ID");
}

fn queued_event_count(env: &WowLuaEnv, name: &str) -> usize {
    env.state()
        .borrow()
        .events
        .pending()
        .iter()
        .filter(|event| event.name == name)
        .count()
}

#[test]
fn selected_offer_without_confirmation_emits_detail_but_no_prompt() {
    let env = create_confirmation_env();
    let before_log = env.state().borrow().quest_log.clone();
    select_offer(&env);
    let counts: (i32, i32) = env.eval("return detailCount, promptCount").unwrap();
    assert_eq!(counts, (1, 0));
    assert_eq!(env.state().borrow().quest_log, before_log);
    assert_eq!(queued_event_count(&env, "QUEST_ACCEPT_CONFIRM"), 0);
}

#[test]
fn explicit_input_publishes_three_arguments_synchronously_without_accepting() {
    let env = create_confirmation_env();
    select_offer(&env);
    let before_log = env.state().borrow().quest_log.clone();
    let before_accepted = queued_event_count(&env, "QUEST_ACCEPTED");
    request_confirmation(&env);
    assert_eq!(env.eval::<i32>("return promptCount").unwrap(), 1);
    let state = env.state().borrow();
    assert_eq!(state.pending_quest_offer, Some(QUEST_ID));
    assert_eq!(state.quest_log, before_log);
    drop(state);
    assert_eq!(queued_event_count(&env, "QUEST_ACCEPTED"), before_accepted);
}

#[test]
fn accept_inside_real_prompt_listener_consumes_offer_without_reprompting() {
    let env = create_confirmation_env();
    select_offer(&env);
    let mut expected_log = env.state().borrow().quest_log.clone();
    assert!(
        !expected_log.contains(&QUEST_ID),
        "fixture quest must be new"
    );
    expected_log.push(QUEST_ID);
    let before_accepted = queued_event_count(&env, "QUEST_ACCEPTED");
    env.exec("acceptDuringPrompt = true").unwrap();
    request_confirmation(&env);
    let state = env.state().borrow();
    assert_eq!(state.pending_quest_offer, None);
    assert_eq!(state.quest_log, expected_log);
    drop(state);
    assert_eq!(
        queued_event_count(&env, "QUEST_ACCEPTED"),
        before_accepted + 1
    );
    assert_eq!(env.eval::<i32>("return promptCount").unwrap(), 1);

    env.exec("ConfirmAcceptQuest()")
        .expect("second acceptance with no pending offer is inert");
    assert_eq!(env.state().borrow().quest_log, expected_log);
    assert_eq!(
        queued_event_count(&env, "QUEST_ACCEPTED"),
        before_accepted + 1
    );
    assert_eq!(env.eval::<i32>("return promptCount").unwrap(), 1);
}

#[test]
fn closing_quest_frame_clears_offer_without_accepting_or_reprompting() {
    let env = create_confirmation_env();
    select_offer(&env);
    let before_log = env.state().borrow().quest_log.clone();
    let before_accepted = queued_event_count(&env, "QUEST_ACCEPTED");
    let before_finished = queued_event_count(&env, "QUEST_FINISHED");
    request_confirmation(&env);
    env.exec("CloseQuestFrame()")
        .expect("existing close path clears offer");
    assert_eq!(env.state().borrow().pending_quest_offer, None);
    assert_eq!(env.state().borrow().quest_log, before_log);
    assert_eq!(
        queued_event_count(&env, "QUEST_FINISHED"),
        before_finished + 1
    );
    assert_eq!(queued_event_count(&env, "QUEST_ACCEPTED"), before_accepted);
    assert_eq!(env.eval::<i32>("return promptCount").unwrap(), 1);
    request_confirmation(&env);
    assert_eq!(env.eval::<i32>("return promptCount").unwrap(), 1);
}

#[test]
fn confirmation_input_without_pending_offer_is_explicit_simulator_noop() {
    let env = create_confirmation_env();
    assert_eq!(env.state().borrow().pending_quest_offer, None);
    let before_log = env.state().borrow().quest_log.clone();
    let before_events = env.state().borrow().events.pending().len();
    request_confirmation(&env);
    assert_eq!(env.eval::<i32>("return promptCount").unwrap(), 0);
    assert_eq!(env.state().borrow().pending_quest_offer, None);
    assert_eq!(env.state().borrow().quest_log, before_log);
    assert_eq!(env.state().borrow().events.pending().len(), before_events);
}
