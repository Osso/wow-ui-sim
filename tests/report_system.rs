use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn report_system_player_report_flow_tracks_tokens_and_result_event() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if type(C_ReportSystem.InitiateReportPlayer) ~= "function" then
                return "missing_initiate_report_player"
            end
            if type(C_ReportSystem.SendReportPlayer) ~= "function" then
                return "missing_send_report_player"
            end

            C_PlayerInfo.GUIDIsPlayer = function(guid)
                return guid == "Player-3676-00000001"
            end
            C_AccountInfo.IsGUIDBattleNetAccountType = function()
                return false
            end

            local location = PlayerLocation:CreateFromGUID("Player-3676-00000001")
            if not location or not location:IsValid() then
                return "invalid_player_location"
            end

            local firstToken = C_ReportSystem.InitiateReportPlayer("cheater", location)
            local secondToken = C_ReportSystem.InitiateReportPlayer("badpetname")
            if type(firstToken) ~= "number" or firstToken <= 0 then
                return "first_report_token_should_be_positive_number"
            end
            if type(secondToken) ~= "number" or secondToken <= firstToken then
                return "report_tokens_should_increase"
            end

            C_ReportSystem.SendReportPlayer(firstToken, "cheating comment")
            C_ReportSystem.SendReportPlayer(secondToken)

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_ReportSystem should create report tokens and accept SendReportPlayer"
    );

    let events = env.state().borrow_mut().events.drain();
    assert_eq!(
        events.len(),
        2,
        "two report sends should queue two result events"
    );
    assert!(
        env.state().borrow().pending_player_reports.is_empty(),
        "sent report tokens should be removed from pending report state"
    );

    let first_event = &events[0];
    assert_eq!(first_event.name, "REPORT_PLAYER_RESULT");
    assert_eq!(first_event.args.len(), 2);
    assert!(
        matches!(first_event.args[0], EventArg::Number(result) if (result - 0.0).abs() < f64::EPSILON)
    );
    assert!(
        matches!(first_event.args[1], EventArg::String(ref report_type) if report_type == "cheater")
    );

    let second_event = &events[1];
    assert_eq!(second_event.name, "REPORT_PLAYER_RESULT");
    assert_eq!(second_event.args.len(), 2);
    assert!(
        matches!(second_event.args[0], EventArg::Number(result) if (result - 0.0).abs() < f64::EPSILON)
    );
    assert!(
        matches!(second_event.args[1], EventArg::String(ref report_type) if report_type == "badpetname")
    );
}

prefork_full_ui_case! {
fn report_button_respects_supplied_eligibility(env: &WowLuaEnv) {
    env.exec(r#"
        local context = { guid = UnitGUID('player') }
        local location = UnitPopupSharedUtil.TryCreatePlayerLocation(context)
        assert(location and UnitPopupSharedUtil.IsValidPlayerLocation(location), 'valid report location required')
        assert(UnitPopupReportButtonMixin:CanShow(context) == false)
    "#).expect("report button should use report eligibility");
}
}
