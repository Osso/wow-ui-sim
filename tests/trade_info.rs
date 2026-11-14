use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn r88e5_trade_offer_warning_consumer_reads_shared_host_value(env: &WowLuaEnv) {
    let loaded: (bool, bool) = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_UIPanels_Game'), C_AddOns.IsAddOnLoaded('Blizzard_MoneyFrame')")
        .expect("Retail trade and money addons should have load state");
    assert_eq!(loaded, (true, true));

    let results: (bool, bool, bool, bool) = env
        .eval(
            r#"
            C_SecureTransfer = C_SecureTransfer or {}
            C_SecureTransfer._state = C_SecureTransfer._state or {}
            C_SecureTransfer._state.shouldShowTradeOfferWarning = true
            TradeFrame_UpdateWarnings()
            local tradeShown = TradeFrameTradeButton.warningTooltip ~= nil
            local tradeValue = C_TradeInfo.ShouldShowTradeOfferWarning()
            C_SecureTransfer._state.shouldShowTradeOfferWarning = false
            TradeFrame_UpdateWarnings()
            local tradeHidden = TradeFrameTradeButton.warningTooltip == nil
            local tradeFalse = C_TradeInfo.ShouldShowTradeOfferWarning()
            return tradeValue, tradeShown, tradeFalse, tradeHidden
            "#,
        )
        .expect("Retail TradeFrame warning consumer should run");

    assert_eq!(results, (true, true, false, true));

    let api_env = WowLuaEnv::new().expect("standalone API environment should initialize");
    let getters: (bool, bool, bool, bool) = api_env
        .eval(
            r#"
            C_SecureTransfer._state.shouldShowTradeOfferWarning = true
            local tradeTrue = C_TradeInfo.ShouldShowTradeOfferWarning()
            local secureTrue = C_SecureTransfer.ShouldShowTradeOfferWarning()
            C_SecureTransfer._state.shouldShowTradeOfferWarning = false
            local tradeFalse = C_TradeInfo.ShouldShowTradeOfferWarning()
            local secureFalse = C_SecureTransfer.ShouldShowTradeOfferWarning()
            return tradeTrue, secureTrue, tradeFalse, secureFalse
            "#,
        )
        .expect("trade warning getters should share their supplied value");
    assert_eq!(getters, (true, true, false, false));
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn r88e5_trade_money_consumer_uses_supplied_offer_and_cursor_state(env: &WowLuaEnv) {
    let money_frame_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MoneyFrame')")
        .expect("Retail money frame load state should be queryable");
    assert!(money_frame_loaded, "Retail MoneyFrame is selected at startup");
    let money_picker_type: String = env
        .eval("return type(GetMoneyTypeInfoField)")
        .expect("MoneyFrame consumer accessor should be queryable");
    assert_eq!(money_picker_type, "function");
    env.state().borrow_mut().player.money = 1_000;

    env.exec(
        "InitiateTrade('Partner'); C_TradeInfo.SetTradeMoney(100); GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')({}, 25.9)",
    )
    .expect("MoneyFrame trade pickup consumer should run");
    assert_eq!(
        env.state()
            .borrow()
            .active_trade
            .as_ref()
            .expect("trade remains open")
            .player_money,
        75
    );
    env.exec(
        "ClearCursor(); C_TradeInfo.SetTradeMoney(50); PickupPlayerMoney(7); GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')({}, 51)",
    )
    .expect("over-offer pickup should return through the original consumer");
    let state = env.state().borrow();
    assert_eq!(
        state
            .active_trade
            .as_ref()
            .expect("trade remains open")
            .player_money,
        50
    );
    drop(state);
    let cursor: (String, f64) = env
        .eval("local kind, amount = GetCursorInfo(); return kind, amount")
        .expect("unchanged money cursor should be readable");
    assert_eq!(cursor, ("money".to_string(), 7.0));

    env.exec(
        "ClearCursor(); C_TradeInfo.SetTradeMoney(100000000001); GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')({}, 100000000000)",
    )
    .expect("cursor-limit pickup should return through the original consumer");
    assert_eq!(
        env.state()
            .borrow()
            .active_trade
            .as_ref()
            .expect("trade remains open")
            .player_money,
        1
    );
    let cursor_type: Option<String> = env
        .eval("return (GetCursorInfo())")
        .expect("cursor state should be readable");
    assert_eq!(cursor_type, None);

    let restriction: (bool, i32, bool) = env
        .eval(
            r#"
            C_TradeInfo._state = { pickupTradeMoneyRestricted = true }
            local restrictedCall = pcall(function()
                return C_TradeInfo.PickupTradeMoney(nil)
            end)
            local restrictedReturns = select("#", C_TradeInfo.PickupTradeMoney(nil))
            C_TradeInfo._state.pickupTradeMoneyRestricted = false
            local ordinaryNilAccepted = pcall(function()
                C_TradeInfo.PickupTradeMoney(nil)
            end)
            return restrictedCall, restrictedReturns, ordinaryNilAccepted
            "#,
        )
        .expect("trade restriction and argument paths should be observable");
    assert_eq!(restriction, (true, 0, false));
}
}
