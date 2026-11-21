use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

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

const PICKUP_MONEY_CASE: &str = r#"
    assert(type(GetMoneyTypeInfoField) == 'function')
    local pickup = GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')
    pickup({}, 25)
    assert(GetPlayerTradeMoney() == 75)
    local kind, amount = GetCursorInfo()
    assert(kind == 'money' and amount == 25)
"#;

const REJECT_EXCESS_MONEY_CASE: &str = r#"
    local pickup = GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')
    pickup({}, 51)
    assert(GetPlayerTradeMoney() == 50)
    assert(select('#', GetCursorInfo()) == 0)
"#;

fn supply_trade_offer(env: &WowLuaEnv, copper: u64) {
    let mut sim = env.state().borrow_mut();
    sim.active_trade = Some(Default::default());
    sim.active_trade
        .as_mut()
        .expect("trade state should be supplied")
        .player_money = copper;
}

#[test]
fn r88co_trade_offer_warning_updates_trade_frame_for_both_supplied_values() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        env.exec(WARNING_CONSUMER_CASE)
            .expect("selected TradeFrame warning consumer should reflect supplied state");
    });
}

#[test]
fn r88co_trade_money_pickup_moves_supplied_offer_amount_to_cursor() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        supply_trade_offer(env, 100);
        env.exec(PICKUP_MONEY_CASE)
            .expect("selected trade money-frame consumer should move supplied money");
    });
}

#[test]
fn r88co_trade_money_pickup_preserves_offer_when_amount_is_too_large() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        supply_trade_offer(env, 50);
        env.exec(REJECT_EXCESS_MONEY_CASE)
            .expect("selected trade money-frame consumer should preserve an oversized offer");
    });
}
