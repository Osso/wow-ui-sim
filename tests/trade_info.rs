use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const WARNING_CONSUMER_CASE: &str = r#"
    A_Admin.SetSecureTransferTradeOfferWarning(true)
    assert(C_TradeInfo.ShouldShowTradeOfferWarning() == true)
    TradeFrame_UpdateWarnings()
    assert(TradeFrameTradeButton.warningTooltip ~= nil)

    A_Admin.SetSecureTransferTradeOfferWarning(false)
    assert(C_TradeInfo.ShouldShowTradeOfferWarning() == false)
    TradeFrame_UpdateWarnings()
    assert(TradeFrameTradeButton.warningTooltip == nil)
"#;

const MONEY_CONSUMER_CASE: &str = r#"
    A_Admin.SetMoney(1000)
    InitiateTrade('Partner')
    C_TradeInfo.SetTradeMoney(100)
    local pickup = GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')
    pickup({}, 25)
    assert(GetPlayerTradeMoney() == 75)
    local kind, amount = GetCursorInfo()
    assert(kind == 'money' and amount == 25)

    ClearCursor()
    C_TradeInfo.SetTradeMoney(50)
    PickupPlayerMoney(7)
    pickup({}, 51)
    assert(GetPlayerTradeMoney() == 50)
    kind, amount = GetCursorInfo()
    assert(kind == 'money' and amount == 7)
"#;

#[test]
fn r88co_trade_offer_warning_updates_trade_frame_for_both_supplied_values() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        env.exec(WARNING_CONSUMER_CASE)
            .expect("selected TradeFrame warning consumer should reflect supplied state");
    });
}

#[test]
fn r88co_trade_money_pickup_uses_the_trade_money_frame_consumer() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        env.exec(MONEY_CONSUMER_CASE)
            .expect("selected trade money-frame consumer should move supplied money");
    });
}
