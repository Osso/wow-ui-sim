use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn secure_transfer_namespace_has_expected_shape() {
    let env = env();
    let (
        is_table,
        has_accept_trade,
        has_cancel,
        has_complete_housing_purchase,
        has_complete_housing_vc_purchase,
        has_get_housing_purchase_cost,
        has_get_housing_purchase_quantity,
        has_get_housing_vc_product_id,
        has_get_mail_info,
        has_get_trade_partner,
        has_send_mail,
        has_should_show_trade_warning,
        mail_target,
        mail_send_money,
        trade_warning,
        trade_partner_is_nil,
    ): (
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        String,
        i64,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local ns = C_SecureTransfer
            local mailInfo = ns.GetMailInfo()
            return type(ns) == "table",
                type(ns.AcceptTrade) == "function",
                type(ns.Cancel) == "function",
                type(ns.CompleteHousingPurchase) == "function",
                type(ns.CompleteHousingVCPurchase) == "function",
                type(ns.GetHousingPurchaseCost) == "function",
                type(ns.GetHousingPurchaseQuantity) == "function",
                type(ns.GetHousingVCPurchaseProductID) == "function",
                type(ns.GetMailInfo) == "function",
                type(ns.GetTradePartner) == "function",
                type(ns.SendMail) == "function",
                type(ns.ShouldShowTradeOfferWarning) == "function",
                mailInfo.target,
                mailInfo.sendMoney,
                ns.ShouldShowTradeOfferWarning(),
                ns.GetTradePartner() == nil
            "#,
        )
        .unwrap();

    assert!(is_table, "C_SecureTransfer should be a table namespace");
    assert!(has_accept_trade, "AcceptTrade should exist");
    assert!(has_cancel, "Cancel should exist");
    assert!(
        has_complete_housing_purchase,
        "CompleteHousingPurchase should exist"
    );
    assert!(
        has_complete_housing_vc_purchase,
        "CompleteHousingVCPurchase should exist"
    );
    assert!(
        has_get_housing_purchase_cost,
        "GetHousingPurchaseCost should exist"
    );
    assert!(
        has_get_housing_purchase_quantity,
        "GetHousingPurchaseQuantity should exist"
    );
    assert!(
        has_get_housing_vc_product_id,
        "GetHousingVCPurchaseProductID should exist"
    );
    assert!(has_get_mail_info, "GetMailInfo should exist");
    assert!(has_get_trade_partner, "GetTradePartner should exist");
    assert!(has_send_mail, "SendMail should exist");
    assert!(
        has_should_show_trade_warning,
        "ShouldShowTradeOfferWarning should exist"
    );
    assert_eq!(
        mail_target, "",
        "default mail target should be empty string"
    );
    assert_eq!(mail_send_money, 0, "default sendMoney should be zero");
    assert!(!trade_warning, "trade warning should default to false");
    assert!(trade_partner_is_nil, "trade partner should default to nil");
}

#[test]
fn secure_transfer_queries_use_state() {
    let env = env();
    let (
        trade_warning,
        trade_partner,
        mail_target,
        mail_send_money,
        housing_cost,
        housing_quantity,
        housing_vc_product_id,
    ): (bool, String, String, i64, i64, i64, i64) = env
        .eval(
            r#"
            C_TradeInfo._state = C_TradeInfo._state or {}
            C_TradeInfo._state.shouldShowTradeOfferWarning = true
            C_SecureTransfer._state.tradePartner = "Merchant-MoonGuard"
            C_SecureTransfer._state.mailInfo = {
                target = "Alt-Illidan",
                sendMoney = "12345",
            }
            C_SecureTransfer._state.housingPurchaseCost = "4500"
            C_SecureTransfer._state.housingPurchaseQuantity = "3"
            C_SecureTransfer._state.housingVCPurchaseProductID = "77"

            local mailInfo = C_SecureTransfer.GetMailInfo()
            return C_SecureTransfer.ShouldShowTradeOfferWarning(),
                C_SecureTransfer.GetTradePartner(),
                mailInfo.target,
                mailInfo.sendMoney,
                C_SecureTransfer.GetHousingPurchaseCost(),
                C_SecureTransfer.GetHousingPurchaseQuantity(),
                C_SecureTransfer.GetHousingVCPurchaseProductID()
            "#,
        )
        .unwrap();

    assert!(trade_warning, "warning flag should read from state");
    assert_eq!(
        trade_partner, "Merchant-MoonGuard",
        "trade partner should read from state"
    );
    assert_eq!(
        mail_target, "Alt-Illidan",
        "mail target should read from state"
    );
    assert_eq!(
        mail_send_money, 12345,
        "mail money should normalize to integer"
    );
    assert_eq!(
        housing_cost, 4500,
        "housing cost should normalize to integer"
    );
    assert_eq!(housing_quantity, 3, "quantity should normalize to integer");
    assert_eq!(
        housing_vc_product_id, 77,
        "product id should normalize to integer"
    );
}

#[test]
fn secure_transfer_actions_update_state() {
    let env = env();
    let (
        accept_trade_count,
        send_mail_count,
        complete_housing_purchase_count,
        complete_housing_vc_purchase_count,
        cancel_count,
        last_action,
    ): (i64, i64, i64, i64, i64, String) = env
        .eval(
            r#"
            C_SecureTransfer._state.acceptTradeCount = 0
            C_SecureTransfer._state.sendMailCount = 0
            C_SecureTransfer._state.completeHousingPurchaseCount = 0
            C_SecureTransfer._state.completeHousingVCPurchaseCount = 0
            C_SecureTransfer._state.cancelCount = 0
            C_SecureTransfer._state.lastAction = nil

            C_SecureTransfer.AcceptTrade()
            C_SecureTransfer.SendMail()
            C_SecureTransfer.CompleteHousingPurchase()
            C_SecureTransfer.CompleteHousingVCPurchase()
            C_SecureTransfer.Cancel()

            return C_SecureTransfer._state.acceptTradeCount,
                C_SecureTransfer._state.sendMailCount,
                C_SecureTransfer._state.completeHousingPurchaseCount,
                C_SecureTransfer._state.completeHousingVCPurchaseCount,
                C_SecureTransfer._state.cancelCount,
                C_SecureTransfer._state.lastAction
            "#,
        )
        .unwrap();

    assert_eq!(accept_trade_count, 1, "AcceptTrade should increment count");
    assert_eq!(send_mail_count, 1, "SendMail should increment count");
    assert_eq!(
        complete_housing_purchase_count, 1,
        "CompleteHousingPurchase should increment count"
    );
    assert_eq!(
        complete_housing_vc_purchase_count, 1,
        "CompleteHousingVCPurchase should increment count"
    );
    assert_eq!(cancel_count, 1, "Cancel should increment count");
    assert_eq!(
        last_action, "Cancel",
        "lastAction should reflect final call"
    );
}
