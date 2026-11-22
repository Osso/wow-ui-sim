use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

// VERIFIED: Retail getters share the client-local flag.
// INFERRED (R88co): Retail and Forever TradeFrame use C_SecureTransfer's shared warning source.
const WARNING_CONSUMER_CASE: &str = r#"
    A_Admin.SetSecureTransferTradeOfferWarning(true)
    local tradeTrue = C_TradeInfo.ShouldShowTradeOfferWarning()
    local secureTrue = C_SecureTransfer.ShouldShowTradeOfferWarning()
    TradeFrame_UpdateWarnings()
    local tradeShown = TradeFrameTradeButton.warningTooltip ~= nil

    A_Admin.SetSecureTransferTradeOfferWarning(false)
    local tradeFalse = C_TradeInfo.ShouldShowTradeOfferWarning()
    local secureFalse = C_SecureTransfer.ShouldShowTradeOfferWarning()
    TradeFrame_UpdateWarnings()
    local tradeHidden = TradeFrameTradeButton.warningTooltip == nil
    return tradeTrue, secureTrue, tradeShown, tradeFalse, secureFalse, tradeHidden
"#;

// VERIFIED: Restriction precedes parsing, valid money truncates, an over-offer skips
// removal, and the cursor limit is exclusive.
// INFERRED: The test supplies host restriction and cursor state.
// UNKNOWN: Packet, hook, flag, and full success/error outcomes are not modeled.
const MONEY_TRUNCATION_CASE: &str = r#"
    local pickup = GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')
    pickup({}, 25.9)
    local kind, copper = GetCursorInfo()
    return GetPlayerTradeMoney(), kind, copper
"#;

const MONEY_OVER_OFFER_CASE: &str = r#"
    ClearCursor()
    PickupPlayerMoney(7)
    local pickup = GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')
    pickup({}, 51)
    local kind, copper = GetCursorInfo()
    return GetPlayerTradeMoney(), kind, copper
"#;

const MONEY_CURSOR_LIMIT_CASE: &str = r#"
    ClearCursor()
    local pickup = GetMoneyTypeInfoField('PLAYER_TRADE', 'PickupFunc')
    pickup({}, 100000000000)
    return GetPlayerTradeMoney(), select('#', GetCursorInfo())
"#;

const MONEY_RESTRICTION_CASE: &str = r##"
    A_Admin.SetTradeMoneyPickupRestricted(true)
    local restrictedCall = pcall(function()
        return C_TradeInfo.PickupTradeMoney(nil)
    end)
    local restrictedReturns = select('#', C_TradeInfo.PickupTradeMoney(nil))
    A_Admin.SetTradeMoneyPickupRestricted(false)
    local ordinaryNilAccepted = pcall(function()
        C_TradeInfo.PickupTradeMoney(nil)
    end)
    return restrictedCall, restrictedReturns, ordinaryNilAccepted
"##;

fn supply_trade_offer(env: &WowLuaEnv, copper: u64) {
    let mut sim = env.state().borrow_mut();
    sim.active_trade = Some(Default::default());
    sim.active_trade
        .as_mut()
        .expect("trade state should be supplied")
        .player_money = copper;
}

fn assert_trade_money_pickup(env: &WowLuaEnv, script: &str, expected: (f64, String, f64)) {
    let actual: (f64, String, f64) = env
        .eval(script)
        .expect("PLAYER_TRADE pickup consumer should run");
    assert_eq!(actual, expected);
}

fn click_trade_accept(env: &WowLuaEnv, warning: bool) -> (bool, bool) {
    env.state().borrow_mut().active_trade = Some(Default::default());
    env.exec(&format!(
        "A_Admin.SetSecureTransferTradeOfferWarning({warning}); TradeFrame_UpdateWarnings(); TradeFrameTradeButton:Click()"
    ))
    .expect("loaded TradeFrame accept button should run");
    let warning_shown = env
        .eval("return TradeFrameTradeButton.warningTooltip ~= nil")
        .expect("TradeFrame warning state should be readable");
    let trade_accepted = env
        .state()
        .borrow()
        .active_trade
        .as_ref()
        .is_some_and(|trade| trade.player_accepted);
    (warning_shown, trade_accepted)
}

// INFERRED (R88ed, native evidence pending): the supplied warning input drives the selected TradeFrame before its original accept-button handler.
#[test]
fn r88ed_trade_frame_accept_button_uses_supplied_warning() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        assert_eq!(click_trade_accept(env, true), (true, true));
        assert_eq!(click_trade_accept(env, false), (false, true));
    });
}

#[test]
fn r88e5_trade_offer_warning_consumer_reads_shared_host_value() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        let result: (bool, bool, bool, bool, bool, bool) = env
            .eval(WARNING_CONSUMER_CASE)
            .expect("selected TradeFrame warning consumer should run");
        assert_eq!(result, (true, true, true, false, false, true));
    });
}

#[test]
fn r88e5_trade_money_consumer_uses_supplied_offer_and_cursor_state() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        assert_eq!(
            env.eval::<String>("return type(GetMoneyTypeInfoField)")
                .unwrap(),
            "function"
        );

        supply_trade_offer(env, 100);
        assert_trade_money_pickup(env, MONEY_TRUNCATION_CASE, (75.0, "money".into(), 25.0));

        supply_trade_offer(env, 50);
        env.state().borrow_mut().player.money = 1_000;
        assert_trade_money_pickup(env, MONEY_OVER_OFFER_CASE, (50.0, "money".into(), 7.0));

        supply_trade_offer(env, 100_000_000_001);
        let actual: (f64, i32) = env.eval(MONEY_CURSOR_LIMIT_CASE).unwrap();
        assert_eq!(actual, (1.0, 0));
    });
}

#[test]
fn r88e5_trade_money_restriction_precedes_argument_parsing() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let actual: (bool, i32, bool) = env
        .eval(MONEY_RESTRICTION_CASE)
        .expect("trade restriction and argument paths should be observable");
    assert_eq!(actual, (true, 0, false));
}
