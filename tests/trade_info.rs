use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn r88e5_trade_offer_warning_consumer_reads_shared_host_value(env: &WowLuaEnv) {
    env.exec("C_AddOns.LoadAddOn('Blizzard_UIPanels_Game')")
        .expect("Retail trade panel should load");

    let results: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            C_SecureTransfer._state.shouldShowTradeOfferWarning = true
            TradeFrame_UpdateWarnings()
            local tradeShown = TradeFrameTradeButton.warningTooltip ~= nil
            local tradeValue = C_TradeInfo.ShouldShowTradeOfferWarning()
            local secureValue = C_SecureTransfer.ShouldShowTradeOfferWarning()

            C_SecureTransfer._state.shouldShowTradeOfferWarning = false
            TradeFrame_UpdateWarnings()
            local tradeHidden = TradeFrameTradeButton.warningTooltip == nil
            local tradeFalse = C_TradeInfo.ShouldShowTradeOfferWarning()
            local secureFalse = C_SecureTransfer.ShouldShowTradeOfferWarning()
            return tradeValue, secureValue, tradeShown, tradeFalse, secureFalse, tradeHidden
            "#,
        )
        .expect("Retail TradeFrame warning consumer should run");

    assert_eq!(results, (true, true, true, false, false, true));
}
}

#[cfg(feature = "optional-tests")]
prefork_full_ui_case! {
fn r88e5_trade_money_consumer_uses_supplied_offer_and_cursor_state(env: &WowLuaEnv) {
    env.exec("C_AddOns.LoadAddOn('Blizzard_MoneyFrame')")
        .expect("Retail money frame should load");
    env.state().borrow_mut().player.money = 1_000;

    env.exec(
        "InitiateTrade('Partner'); C_TradeInfo.SetTradeMoney(100); MoneyTypeInfo.PLAYER_TRADE.PickupFunc(MoneyTypeInfo.PLAYER_TRADE, 25.9)",
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
    let cursor: (String, f64) = env
        .eval("local kind, amount = GetCursorInfo(); return kind, amount")
        .expect("money cursor should be readable");
    assert_eq!(cursor, ("money".to_string(), 25.0));

    env.exec(
        "ClearCursor(); C_TradeInfo.SetTradeMoney(50); PickupPlayerMoney(7); MoneyTypeInfo.PLAYER_TRADE.PickupFunc(MoneyTypeInfo.PLAYER_TRADE, 51)",
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
        "ClearCursor(); C_TradeInfo.SetTradeMoney(100000000001); MoneyTypeInfo.PLAYER_TRADE.PickupFunc(MoneyTypeInfo.PLAYER_TRADE, 100000000000)",
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
}
}
