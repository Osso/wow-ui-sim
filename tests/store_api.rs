#[cfg(feature = "client-retail")]
use wow_ui_sim::c_api::c_store_glue;
#[cfg(feature = "client-retail")]
use wow_ui_sim::c_api::c_store_glue::CompletionInput;
#[cfg(feature = "client-retail")]
use wow_ui_sim::c_api::store_catalog_model::StoreCurrencyInfo;
#[cfg(feature = "client-retail")]
use wow_ui_sim::c_api::store_vas_model::{
    StoreVasGuildFollowInfoInput, StoreVasGuildMasterInfoInput, StoreVasGuildMemberInput,
    receive_guild_follow_info, receive_guild_master_info,
};
#[cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
const STORE_SECURE_VAS_SCRIPT: &str = r#"
    local events = {}
    STORE_SECURE_VAS_EVENTS = events
    local listener = CreateFrame("Frame")
    listener:SetScript("OnEvent", function(_, event, ...)
        if event == "VAS_TRANSFER_VALIDATION_UPDATE" then
            table.insert(events, event .. ":" .. tostring(...))
            return
        end

        if event == "STORE_GUILD_MASTER_INFO_RECEIVED" then
            local realmAddress = ...
            table.insert(events, event .. ":" .. tostring(realmAddress))
            return
        end

        if event == "STORE_GUILD_FOLLOW_INFO_RECEIVED" then
            local guid, info = ...
            table.insert(events, event .. ":" .. tostring(guid) .. ":" .. tostring(info and info.transferredRealm))
            return
        end

        if event == "STORE_VAS_PURCHASE_COMPLETE" then
            table.insert(events, event .. ":" .. tostring(select('#', ...)))
            return
        end

        table.insert(events, event)
    end)

    for _, event in ipairs({
        "VAS_TRANSFER_VALIDATION_UPDATE",
        "STORE_GUILD_MASTER_INFO_RECEIVED",
        "STORE_GUILD_FOLLOW_INFO_RECEIVED",
        "STORE_VAS_PURCHASE_COMPLETE",
    }) do
        listener:RegisterEvent(event)
    end

    local localGuid = C_StoreSecure.GetWoWAccountGUIDFromName("ignored-local-name", true)
    if localGuid ~= "WowAccount-1001-1" then
        return "wrong_local_guid:" .. tostring(localGuid)
    end

    local remoteGuid = C_StoreSecure.GetWoWAccountGUIDFromName("WoW2", false)
    if remoteGuid ~= "WowAccount-2002-2" then
        return "wrong_remote_guid:" .. tostring(remoteGuid)
    end

    C_StoreSecure.ValidateBnetTransfer("transfer@example.com")
    if #events ~= 0 then
        return "validation_request_fired_synchronously"
    end

    local realms = C_StoreSecure.GetRealmList()
    local vasRealms = C_StoreSecure.GetVASRealmList()
    if #realms ~= 2 or #vasRealms ~= 2 then
        return "wrong_realm_count"
    end

    if realms[1].realmName ~= "Azeroth" or realms[2].realmName ~= "Kalimdor" then
        return "wrong_realm_names"
    end

    local characters = C_StoreSecure.GetCharactersForRealm(realms[1].virtualRealmAddress, false)
    if #characters ~= 2 then
        return "wrong_character_count:" .. tostring(#characters)
    end

    local guildCharacters = C_StoreSecure.GetCharactersForRealm(realms[1].virtualRealmAddress, true)
    if #guildCharacters ~= 1 or guildCharacters[1].guid ~= "Player-101-1-11" then
        return "wrong_guild_character_filter"
    end

    local character = C_StoreSecure.GetCharacterInfoByGUID(characters[1].guid)
    if not character or character.name ~= "Simhero" or character.guid ~= "Player-101-1-11" then
        return "wrong_character_info"
    end

    local races = C_StoreSecure.GetEligibleRacesForVASService(character.guid, Enum.VasServiceType.NameChange)
    if #races ~= 2 or races[1].raceName ~= "Human" or races[2].raceName ~= "Void Elf" or races[2].isAlliedRace ~= true then
        return "wrong_eligible_races"
    end

    local guildInfo = C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID(character.guid)
    if not guildInfo or guildInfo.guildName ~= "Simulator Guild" or not guildInfo.guildMemberInfos[1] or guildInfo.guildMemberInfos[1].memberName ~= "Simalt" then
        return "wrong_guild_info"
    end

    local vasServiceType = C_StoreSecure.GetVasServiceType(2003)
    if vasServiceType ~= Enum.VasServiceType.NameChange then
        return "wrong_vas_service_type:" .. tostring(vasServiceType)
    end

    if C_StoreSecure.IsRegionLocked() then
        return "region_locked"
    end

    if C_StoreSecure.GetLastProductListResponseError() ~= 0 then
        return "wrong_product_list_error"
    end

    if #C_StoreSecure.GetVASErrors() ~= 0 then
        return "unexpected_vas_errors"
    end

    C_StoreSecure.RequestRealmGuildMasterInfo(realms[1].virtualRealmAddress)
    C_StoreSecure.RequestCharacterGuildFollowInfo(character.guid, realms[1].virtualRealmAddress)
    if #events ~= 0 then
        return "guild_requests_fired_synchronously"
    end

    C_StoreSecure.OpenNydusLink(1003)
    local confirmationProductID, walletName, originalDollars, originalCents, currentDollars, currentCents = C_StoreSecure.GetConfirmationInfo()
    if confirmationProductID ~= 2003 or walletName ~= "Balance" or originalDollars ~= 10 or originalCents ~= 0 or currentDollars ~= 5 or currentCents ~= 1 then
        return "wrong_confirmation_info"
    end

    local boostProduct, boostCharacter, boostRealm = C_StoreSecure.GetUnrevokedBoostInfo()
    if boostProduct ~= "Apprentice Rider Bundle" or boostCharacter ~= "Simhero" or boostRealm ~= "Azeroth" then
        return "wrong_boost_info"
    end

    C_StoreSecure.PurchaseVASProduct(
        2003,
        character.guid,
        "Renamedhero",
        nil,
        nil,
        realms[2].virtualRealmAddress,
        remoteGuid,
        nil,
        false,
        false
    )

    local purchaseConfirmationProductID, purchaseWalletName, originalDollars, originalCents, purchaseDollars, purchaseCents = C_StoreSecure.GetConfirmationInfo()
    if purchaseConfirmationProductID ~= 2003 or purchaseWalletName ~= "Balance" or originalDollars ~= 10 or originalCents ~= 0 or purchaseDollars ~= 5 or purchaseCents ~= 1 then
        return "wrong_purchase_confirmation_info:" .. tostring(purchaseConfirmationProductID) .. ":" .. tostring(purchaseWalletName) .. ":" .. tostring(purchaseDollars) .. ":" .. tostring(purchaseCents)
    end

    local completionProductID, completionGuid, completionRealm, shouldHandle = C_StoreSecure.GetVASCompletionInfo()
    if completionProductID ~= 2003 or completionGuid ~= character.guid or completionRealm ~= "Azeroth" or shouldHandle ~= true then
        return "wrong_completion_info_before_disconnect:" .. tostring(completionProductID) .. ":" .. tostring(completionGuid) .. ":" .. tostring(completionRealm) .. ":" .. tostring(shouldHandle)
    end

    C_StoreSecure.PurchaseVASProduct(99999, character.guid, "Unused", nil, nil, nil, nil, nil, false, false)

    local failureCode, failureReason = C_StoreSecure.GetFailureInfo()
    if failureCode ~= 9 or failureReason ~= nil then
        return "wrong_failure_info:" .. tostring(failureCode) .. ":" .. tostring(failureReason)
    end

    C_StoreSecure.AckFailure()
    C_StoreSecure.PurchaseProduct(2003)
    if select(1, C_StoreSecure.GetFailureInfo()) ~= nil then
        return "failure_not_cleared"
    end

    C_StoreSecure.ClearPreGeneratedExternalTransactionID()

    if #events ~= 0 then
        return "store_request_emitted_an_event"
    end

    local eventCountBeforeSetters = #events
    C_StoreSecure.SetDisconnectOnLogout(true)
    C_StoreSecure.SetVASProductReady(true)
    if STORE_SECURE_VAS_RETAIL_CONTRACT then
        if not C_StoreGlue.GetDisconnectOnLogout() then
            return "disconnect_flag_not_applied"
        end
        if not C_StoreGlue.GetVASProductReady() then
            return "ready_flag_not_applied"
        end
        if #events ~= eventCountBeforeSetters then
            return "setter_emitted_completion_event"
        end
    end

    return "ok"
"#;

#[cfg(feature = "client-retail")]
fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(feature = "client-retail")]
#[test]
fn store_apis_report_enabled_and_available() {
    let env = env();
    crate::common::store_catalog_fixture::publish_featured_catalog(&env);
    env.set_server_mirrored_cvar("bpayStoreEnable", "1")
        .expect("store mirror should accept the enabled input");
    let values: (bool, bool, bool, bool, bool) = env
        .eval(
            "return C_StorePublic.IsEnabled(), C_StoreSecure.IsAvailable(), C_StoreSecure.HasPurchaseList(), C_StoreSecure.HasProductList(), C_StoreSecure.HasDistributionList()",
        )
        .expect("store API flags should be queryable");

    // R38 (VERIFIED; docs/specs/store-api-availability.md) covers IsEnabled.
    assert_eq!(values, (true, true, true, true, true));
}

#[cfg(feature = "client-retail")]
#[test]
fn store_currency_id_zero_does_not_resolve_host_metadata() {
    let env = env();
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .currencies
        .insert(
            0,
            StoreCurrencyInfo {
                region_id: 1,
                format_short: "%s".into(),
                format_long: "%s".into(),
                license_accept_text: String::new(),
                require_license_accept: false,
                browse_has_star: false,
                hide_browse_notice: false,
                hide_confirmation_browse_notice: false,
            },
        );
    let result: (i32, bool) = env
        .eval("return C_StoreSecure.GetCurrencyID(), C_StoreSecure.GetCurrencyInfo() == nil")
        .expect("zero currency ID should not publish metadata");
    assert_eq!(result, (0, true));
}

#[cfg(feature = "client-retail")]
#[test]
fn store_disconnect_resets_verified_availability_and_list_readiness() {
    let env = env();
    crate::common::store_catalog_fixture::publish_featured_catalog(&env);
    wow_ui_sim::c_api::store_catalog_model::receive_dynamic_price(&env, 2003, vec![1])
        .expect("dynamic-price response should be accepted");
    let before: (bool, bool, bool, bool, i32, bool) = env
        .eval("return C_StoreSecure.IsAvailable(), C_StoreSecure.HasProductList(), C_StoreSecure.HasPurchaseList(), C_StoreSecure.HasDistributionList(), C_StoreSecure.GetCurrencyID(), C_StoreSecure.HasDynamicPriceData(2003)")
        .expect("host responses should populate Store state");
    assert_eq!(before, (true, true, true, true, 61, true));

    wow_ui_sim::c_api::store_catalog_model::receive_disconnect(&env);

    let after: (bool, bool, bool, bool, bool, bool, bool, i32, bool) = env
        .eval("return C_StoreSecure.IsAvailable(), C_StoreSecure.HasProductList(), C_StoreSecure.HasPurchaseList(), C_StoreSecure.HasDistributionList(), C_StoreSecure.IsRegionLocked(), C_StoreSecure.GetLastProductListResponseError() == 0, C_StoreSecure.GetCurrencyInfo() == nil, C_StoreSecure.GetCurrencyID(), C_StoreSecure.HasDynamicPriceData(2003)")
        .expect("disconnect should reset the verified Store fields");
    assert_eq!(
        after,
        (false, false, false, false, false, true, true, 0, false)
    );
}

#[cfg(feature = "client-retail")]
#[test]
fn store_secure_vas_queries_and_stateful_actions_use_seeded_data() {
    let env = env();
    crate::common::store_catalog_fixture::seed_vas_catalog(&env);
    crate::common::store_catalog_fixture::seed_store_vas_state(&env);
    env.exec("STORE_SECURE_VAS_RETAIL_CONTRACT = true")
        .expect("Retail Store model should be available to the probe");
    let result: String = env
        .eval(STORE_SECURE_VAS_SCRIPT)
        .expect("seeded C_StoreSecure VAS flow should be queryable");

    assert_eq!(result, "ok");
    assert_store_vas_response_events(&env);
    assert_retail_completion_callback_owns_the_event(&env);
}

#[cfg(feature = "client-retail")]
fn assert_store_vas_response_events(env: &WowLuaEnv) {
    receive_guild_master_info(env, 101, vec![store_api_guild_master_info()])
        .expect("guild-master response should dispatch its event");
    receive_guild_follow_info(env, store_api_guild_follow_info())
        .expect("guild-follow response should dispatch its event");
    let events: (String, String) = env
        .eval("return STORE_SECURE_VAS_EVENTS[1], STORE_SECURE_VAS_EVENTS[2]")
        .expect("response listener should retain both asynchronous events");
    assert_eq!(events.0, "STORE_GUILD_MASTER_INFO_RECEIVED:101");
    assert_eq!(
        events.1,
        "STORE_GUILD_FOLLOW_INFO_RECEIVED:Player-101-1-11:Kalimdor"
    );
}

#[cfg(feature = "client-retail")]
fn store_api_guild_master_info() -> StoreVasGuildMasterInfoInput {
    StoreVasGuildMasterInfoInput {
        guid: "Player-101-1-11".into(),
        guild_name: "Simulator Guild".into(),
        members: vec![StoreVasGuildMemberInput {
            guid: Some("Player-101-1-12".into()),
            member_name: "Simalt".into(),
        }],
    }
}

#[cfg(feature = "client-retail")]
fn store_api_guild_follow_info() -> StoreVasGuildFollowInfoInput {
    StoreVasGuildFollowInfoInput {
        guid: "Player-101-1-11".into(),
        transferred_realm: Some("Kalimdor".into()),
        faction_changed: false,
    }
}

#[cfg(feature = "client-retail")]
fn assert_retail_completion_callback_owns_the_event(env: &WowLuaEnv) {
    env.state()
        .borrow_mut()
        .store_glue
        .callbacks
        .purchase_complete = true;
    c_store_glue::receive_completion(
        env,
        CompletionInput {
            first: 0,
            second: 0,
            first_flag: false,
            second_flag: false,
            data: Vec::new(),
        },
    )
    .expect("completion callback should dispatch its event");
    let event: String = env
        .eval("return STORE_SECURE_VAS_EVENTS[3]")
        .expect("completion listener should retain the callback event");

    assert_eq!(event, "STORE_VAS_PURCHASE_COMPLETE:0");
}
