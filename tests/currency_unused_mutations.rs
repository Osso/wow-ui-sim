#![cfg(feature = "client-retail")]

use crate::currency_displayed_indices::{with_token_ui, with_token_ui_env};

#[test]
fn currency_unused_mutations_original_checkbox_updates_selected_currency_and_provider() {
    with_token_ui(
        r#"
        -- Resolve from current displayed indices after a preceding category collapses.
        B5Header(1):ToggleCollapsed()
        local watched = C_CurrencyInfo.GetBackpackCurrencyInfo(3)
        assert(watched.currencyTypesID == 1792)
        local selected = assert(C_CurrencyInfo.GetCurrencyListInfo(3))
        assert(selected.currencyTypesID == 1792)
        TokenFrame.selectedID = 3
        TokenFrame.selectedToken = selected.name
        local checkbox = TokenFramePopup.InactiveCheckbox
        checkbox:SetChecked(true)
        checkbox:OnClick()
        local selectedAfterUpdate = assert(C_CurrencyInfo.GetCurrencyListInfo(TokenFrame.selectedID))
        assert(selectedAfterUpdate.currencyTypesID == 1792, "original checkbox must reselect its currency")
        assert(selectedAfterUpdate.isTypeUnused == true, "B5 unused: original checkbox must mark selected currency unused")
        assert(C_CurrencyInfo.GetCurrencyInfo(1792).isTypeUnused == true)
        assert(C_CurrencyInfo.GetCurrencyInfo(1792).quantity == 4350)
        assert(selectedAfterUpdate.isShowInBackpack == true)
        assert(C_CurrencyInfo.GetCurrencyInfo(1792).isShowInBackpack == true)
        assert(C_CurrencyInfo.GetBackpackCurrencyInfo(3).currencyTypesID == 1792)
        local provider = TokenFrame.ScrollBox:GetDataProvider()
        assert(provider:Find(TokenFrame.selectedID).isTypeUnused == true,
            "original TokenFrame must display the unused flag")
        B5AssertProvider()
        checkbox:SetChecked(false)
        checkbox:OnClick()
        assert(C_CurrencyInfo.GetCurrencyListInfo(TokenFrame.selectedID).currencyTypesID == 1792)
        assert(C_CurrencyInfo.GetCurrencyListInfo(TokenFrame.selectedID).isTypeUnused == false)
        assert(C_CurrencyInfo.GetCurrencyInfo(1792).isTypeUnused == false)
        assert(TokenFrame.ScrollBox:GetDataProvider():Find(TokenFrame.selectedID).isTypeUnused == false)
        assert(C_CurrencyInfo.GetBackpackCurrencyInfo(3).currencyTypesID == 1792)
        B5AssertProvider()
        "#,
    );
}

#[test]
fn currency_unused_mutations_original_popup_shows_discovered_currency_checkbox() {
    with_token_ui(
        r#"
        local row = C_CurrencyInfo.GetCurrencyListInfo(3)
        row.currencyIndex = 3
        local entry = CreateFrame("Button", nil, UIParent, "TokenEntryTemplate")
        entry:Initialize(row)
        TokenFrame:UpdatePopup(entry)
        assert(TokenFramePopup.InactiveCheckbox:IsShown(),
            "B5 unused: original popup must expose a discovered currency checkbox")
        assert(TokenFramePopup.InactiveCheckbox:GetChecked() == false)
        C_CurrencyInfo.SetCurrencyUnused(3, true)
        entry:Initialize(C_CurrencyInfo.GetCurrencyListInfo(3))
        TokenFrame:UpdatePopup(entry)
        assert(TokenFramePopup.InactiveCheckbox:IsShown())
        assert(TokenFramePopup.InactiveCheckbox:GetChecked() == true)
        "#,
    );
}

#[test]
fn currency_popup_propagates_list_currency_id_to_transfer_button() {
    with_token_ui_env(|env| {
        env.exec(r#"
            local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166)
            assert(not ok and reason == 10)
            C_CurrencyInfo.RequestCurrencyDataForAccountCharacters()
            assert(C_CurrencyInfo.IsAccountCharacterCurrencyDataReady())
            assert(type(C_CurrencyInfo.GetCurrencyListInfo(1).currencyID) == "number")
            local row = assert(C_CurrencyInfo.GetCurrencyListInfo(3))
            row.currencyIndex = 3
            local entry = CreateFrame("Button", nil, UIParent, "TokenEntryTemplate")
            entry:Initialize(row)
            TokenFrame:UpdatePopup(entry)
            assert(TokenFramePopup.CurrencyTransferToggleButton.currencyID == 2806)
            assert(not TokenFramePopup.CurrencyTransferToggleButton:IsEnabled())
            assert(TokenFramePopup.CurrencyTransferToggleButton:GetDisabledTooltip() ~= nil)
        "#).unwrap();
    });
}

#[test]
fn currency_transfer_button_refreshes_original_consumer_for_nontransferable_currency() {
    with_token_ui_env(|env| {
        {
            let mut state = env.state().borrow_mut();
            let list = &mut state.currency_list;
            list.transfer_server_enabled = Some(true);
            list.current_character_guid = Some("player-guid".into());
            list.transfer_rules.insert(1166, wow_ui_sim::c_api::currency_list::CurrencyTransferRules {
                award_condition_met: Some(true), quantity_cap_reached: Some(false),
                weekly_cap_reached: Some(false), ..Default::default()
            });
        }
        env.exec(r#"
        local button = TokenFramePopup.CurrencyTransferToggleButton
        local currency = assert(C_CurrencyInfo.GetCurrencyInfo(2806))
        assert(currency.currencyID == 2806)
        button:Refresh(currency)
        assert(not button:IsEnabled())
        assert(not button:IsShown())
        button:Refresh(assert(C_CurrencyInfo.GetCurrencyInfo(1166)))
        assert(not button:IsEnabled())
        assert(button:IsShown())
        "#).unwrap();
    });
}

#[test]
fn currency_transfer_eligibility_uses_supplied_policy_and_account_sources() {
    use wow_ui_sim::c_api::currency_list::CurrencyTransferRules;
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let list = &mut state.currency_list;
        list.transfer_server_enabled = Some(false);
        list.current_character_guid = Some("self".into());
        list.transfer_rules.insert(1166, CurrencyTransferRules {
            award_condition_met: Some(true), quantity_cap_reached: Some(false),
            weekly_cap_reached: Some(false), ..Default::default()
        });
    }
    env.exec(r#"
        assert(C_CurrencyInfo.IsAccountTransferableCurrency(1166))
        assert(not C_CurrencyInfo.IsAccountTransferableCurrency(2806))
        assert(not C_CurrencyInfo.IsAccountTransferableCurrency(999999))
        local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166)
        assert(not ok and reason == 10)
    "#).unwrap();
    env.state().borrow_mut().currency_list.transfer_server_enabled = Some(true);
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166); assert(not ok and reason == 6)").unwrap();
    env.state().borrow_mut().currency_list.account_currency_sources.insert(1166, vec![("self".into(), 99), ("other".into(), 7)]);
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166); assert(ok and reason == nil)").unwrap();
    env.state().borrow_mut().currency_list.transfer_rules.get_mut(&1166).unwrap().quantity_cap_reached = Some(true);
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166); assert(not ok and reason == 4)").unwrap();
    {
        let mut state = env.state().borrow_mut();
        let rule = state.currency_list.transfer_rules.get_mut(&1166).unwrap();
        rule.quantity_cap_reached = Some(false);
        rule.award_condition_met = Some(false);
    }
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166); assert(not ok and reason == 8)").unwrap();
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(3008); assert(not ok and reason == 5)").unwrap();
    env.state().borrow_mut().currency_list.transfer_rules.insert(1716, CurrencyTransferRules::default());
    env.exec("assert(not pcall(C_CurrencyInfo.CanTransferCurrency, 1716))").unwrap();
    env.state().borrow_mut().currency_list.transfer_rules.get_mut(&1716).unwrap().faction_key_0 = Some(false);
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1716); assert(not ok and reason == 5)").unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.currency_transfer_in_progress = true;
        state.currency_list.transfer_server_enabled = Some(false);
    }
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166); assert(not ok and reason == 10)").unwrap();
    env.state().borrow_mut().currency_list.transfer_server_enabled = Some(true);
    env.exec("local ok, reason = C_CurrencyInfo.CanTransferCurrency(1166); assert(not ok and reason == 9)").unwrap();
}

#[test]
fn currency_unused_mutations_ignore_non_currency_indices_and_missing_supplied_rows() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, index in ipairs({0, -1, 20, 2245, 1, 7, 14}) do
            C_CurrencyInfo.SetCurrencyUnused(index, true)
        end
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
        for _, id in ipairs({2245, 2806, 1792, 1901, 1191}) do
            assert(C_CurrencyInfo.GetCurrencyInfo(id).isTypeUnused == false)
        end
        "#,
    )
    .unwrap();
    env.state().borrow_mut().currency_info.remove(&2245);
    env.exec(
        r#"
        C_CurrencyInfo.SetCurrencyUnused(2, true)
        assert(C_CurrencyInfo.GetCurrencyListInfo(2) == nil)
        assert(C_CurrencyInfo.GetCurrencyInfo(2245) == nil)
        assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
        "#,
    )
    .unwrap();
}

#[test]
fn currency_unused_mutations_project_supplied_unused_and_discovery_flags() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    for (unused, discovered) in [(true, false), (false, true)] {
        {
            let mut state = env.state().borrow_mut();
            let info = state.currency_info.get_mut(&2806).unwrap();
            info.is_type_unused = unused;
            info.discovered = discovered;
        }
        let actual: (bool, bool) = env
            .eval(
                "local row = C_CurrencyInfo.GetCurrencyListInfo(3); return row.isTypeUnused, row.discovered",
            )
            .unwrap();
        assert_eq!(actual, (unused, discovered));
    }
}

#[test]
fn currency_unused_mutations_repeated_requests_and_fresh_environments() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _ = 1, 2 do
            C_CurrencyInfo.SetCurrencyUnused(3, true)
            assert(C_CurrencyInfo.GetCurrencyInfo(2806).isTypeUnused == true,
                "B5 unused: repeated request must keep the supplied flag")
        end
        "#,
    )
    .unwrap();
    let fresh = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    fresh
        .exec(
            r#"
            assert(C_CurrencyInfo.GetCurrencyInfo(2806).isTypeUnused == false)
            assert(C_CurrencyInfo.GetCurrencyListSize() == 19)
            "#,
        )
        .unwrap();
}
