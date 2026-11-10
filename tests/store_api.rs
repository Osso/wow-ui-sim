use wow_ui_sim::lua_api::WowLuaEnv;

const STORE_SECURE_VAS_SCRIPT: &str = r#"
    local events = {}
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

    local localGuid = C_StoreSecure.GetWoWAccountGUIDFromName("WoW1", true)
    if localGuid ~= 1001 then
        return "wrong_local_guid:" .. tostring(localGuid)
    end

    local remoteGuid = C_StoreSecure.GetWoWAccountGUIDFromName("WoW2", false)
    if remoteGuid ~= 2002 then
        return "wrong_remote_guid:" .. tostring(remoteGuid)
    end

    C_StoreSecure.ValidateBnetTransfer("transfer@example.com")

    if events[1] ~= "VAS_TRANSFER_VALIDATION_UPDATE:false" then
        return "wrong_transfer_event:" .. tostring(events[1])
    end

    local bnetGuid, gameAccounts = C_StoreSecure.GetBnetTransferInfo()
    if bnetGuid ~= 3001 then
        return "wrong_bnet_guid:" .. tostring(bnetGuid)
    end

    if #gameAccounts ~= 2 or gameAccounts[1] ~= "WoW2" or gameAccounts[2] ~= "WoW3" then
        return "wrong_game_accounts"
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
    if #guildCharacters ~= 1 or guildCharacters[1].guid ~= 501001 then
        return "wrong_guild_character_filter"
    end

    local character = C_StoreSecure.GetCharacterInfoByGUID(characters[1].guid)
    if not character or character.name ~= "Simhero" or character.realmName ~= "Azeroth" then
        return "wrong_character_info"
    end

    local races = C_StoreSecure.GetEligibleRacesForVASService(character.guid, Enum.VasServiceType.NameChange)
    if #races ~= 2 or races[1].raceName ~= "Human" or races[2].raceName ~= "Void Elf" or races[2].isAlliedRace ~= true then
        return "wrong_eligible_races"
    end

    local guildInfo = C_StoreSecure.GetVASGuildMasterInfoForCharacterByGUID(character.guid)
    if not guildInfo or guildInfo.guildName ~= "Simulator Guild" or guildInfo.guildMasterName ~= "Simleader" then
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

    if events[2] ~= "STORE_GUILD_MASTER_INFO_RECEIVED:101" then
        return "wrong_guild_master_event:" .. tostring(events[2])
    end

    if events[3] ~= "STORE_GUILD_FOLLOW_INFO_RECEIVED:501001:Kalimdor" then
        return "wrong_guild_follow_event:" .. tostring(events[3])
    end

    C_StoreSecure.OpenNydusLink(1003)
    local confirmationProductID, walletName, _, _, currentDollars, currentCents = C_StoreSecure.GetConfirmationInfo()
    if confirmationProductID ~= 2003 or walletName ~= "Blizzard Balance" or currentDollars ~= 10 or currentCents ~= 0 then
        return "wrong_confirmation_info"
    end

    local boostProduct, boostCharacter, boostRealm = C_StoreSecure.GetUnrevokedBoostInfo()
    if boostProduct ~= "Level 70 Character Boost" or boostCharacter ~= "Simhero" or boostRealm ~= "Azeroth" then
        return "wrong_boost_info"
    end

    local purchaseOk = C_StoreSecure.PurchaseVASProduct(
        2003,
        character.guid,
        "Renamedhero",
        nil,
        nil,
        realms[2].virtualRealmAddress,
        remoteGuid,
        bnetGuid,
        false,
        false
    )
    if purchaseOk ~= true then
        return "initial_purchase_failed"
    end

    if C_StoreSecure.HasPurchaseInProgress() ~= true then
        return "purchase_state_not_marked_in_progress"
    end

    local purchaseConfirmationProductID, purchaseWalletName, _, _, purchaseDollars, purchaseCents = C_StoreSecure.GetConfirmationInfo()
    if purchaseConfirmationProductID ~= 2003 or purchaseWalletName ~= "Blizzard Balance" or purchaseDollars ~= 10 or purchaseCents ~= 0 then
        return "wrong_purchase_confirmation_info:" .. tostring(purchaseConfirmationProductID) .. ":" .. tostring(purchaseWalletName) .. ":" .. tostring(purchaseDollars) .. ":" .. tostring(purchaseCents)
    end

    local completionProductID, completionGuid, completionRealm, shouldHandle = C_StoreSecure.GetVASCompletionInfo()
    if completionProductID ~= 2003 or completionGuid ~= character.guid or completionRealm ~= "Kalimdor" or shouldHandle ~= false then
        return "wrong_completion_info_before_disconnect:" .. tostring(completionProductID) .. ":" .. tostring(completionGuid) .. ":" .. tostring(completionRealm) .. ":" .. tostring(shouldHandle)
    end

    local duplicatePurchase = C_StoreSecure.PurchaseVASProduct(
        2003,
        character.guid,
        "Renamedhero",
        nil,
        nil,
        realms[2].virtualRealmAddress,
        remoteGuid,
        bnetGuid,
        false,
        false
    )
    if duplicatePurchase ~= false then
        return "duplicate_purchase_should_fail"
    end

    local failureCode, failureReason = C_StoreSecure.GetFailureInfo()
    if failureCode ~= Enum.StoreError.Other or failureReason ~= "DuplicateVASPurchase" then
        return "wrong_failure_info:" .. tostring(failureCode) .. ":" .. tostring(failureReason)
    end

    C_StoreSecure.AckFailure()
    if select(1, C_StoreSecure.GetFailureInfo()) ~= nil then
        return "failure_not_cleared"
    end

    C_StoreSecure.ClearPreGeneratedExternalTransactionID()

    local retriedPurchase = C_StoreSecure.PurchaseVASProduct(
        2003,
        character.guid,
        "Renamedhero",
        nil,
        nil,
        realms[2].virtualRealmAddress,
        remoteGuid,
        bnetGuid,
        false,
        false
    )
    if retriedPurchase ~= true then
        return "retry_purchase_failed"
    end

    C_StoreSecure.SetDisconnectOnLogout(true)
    local _, _, _, shouldHandleAfterDisconnect = C_StoreSecure.GetVASCompletionInfo()
    if shouldHandleAfterDisconnect ~= true then
        return "disconnect_flag_not_applied"
    end

    C_StoreSecure.SetVASProductReady(true)
    if events[4] ~= "STORE_VAS_PURCHASE_COMPLETE" then
        return "wrong_vas_complete_event:" .. tostring(events[4])
    end

    return "ok"
"#;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn store_apis_report_enabled_and_available() {
    let env = env();
    env.set_server_mirrored_cvar("bpayStoreEnable", "1")
        .expect("store mirror should accept the enabled input");
    let (store_public_enabled, store_secure_available, has_purchase_list, has_product_list, has_distribution_list): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            "return C_StorePublic.IsEnabled(), C_StoreSecure.IsAvailable(), C_StoreSecure.HasPurchaseList(), C_StoreSecure.HasProductList(), C_StoreSecure.HasDistributionList()",
        )
        .expect("store API flags should be queryable");

    assert!(
        store_public_enabled,
        "C_StorePublic.IsEnabled() should be true"
    );
    assert!(
        store_secure_available,
        "C_StoreSecure.IsAvailable() should be true"
    );
    assert!(
        has_purchase_list,
        "C_StoreSecure.HasPurchaseList() should be true"
    );
    assert!(
        has_product_list,
        "C_StoreSecure.HasProductList() should be true"
    );
    assert!(
        has_distribution_list,
        "C_StoreSecure.HasDistributionList() should be true"
    );
}

#[test]
fn store_secure_vas_queries_and_stateful_actions_use_seeded_data() {
    let env = env();
    let result: String = env
        .eval(STORE_SECURE_VAS_SCRIPT)
        .expect("seeded C_StoreSecure VAS flow should be queryable");

    assert_eq!(result, "ok");
}
