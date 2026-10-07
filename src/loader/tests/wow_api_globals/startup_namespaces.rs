//! Bootstrap-time `C_*` namespace surfaces (presence + return shape checks).

use super::super::*;

#[test]
fn test_bootstrap_fills_existing_namespace_defaults() {
    let env = WowLuaEnv::new().unwrap();
    let (
        trade_ty,
        trade_value,
        quest_ty,
        quest_value,
        color_ty,
        hex_ty,
        campaign_ty,
        campaign_name,
    ): (String, i64, String, bool, String, String, String, String) = env
        .eval(
            r#"
            local color = C_ColorOverrides.GetColorForQuality(0)
            local campaign = C_CampaignInfo.GetCampaignInfo(290)
            return type(C_TradeSkillUI.GetProfessionSkillLineID),
                C_TradeSkillUI.GetProfessionSkillLineID(7),
                type(C_QuestLog.ReadyForTurnIn),
                C_QuestLog.ReadyForTurnIn(42),
                type(C_ColorOverrides.GetColorForQuality),
                type(color and color.GenerateHexColorMarkup),
                type(C_CampaignInfo.GetCampaignInfo),
                campaign.name
            "#,
        )
        .unwrap();

    assert_eq!(trade_ty, "function");
    assert_eq!(trade_value, 7);
    assert_eq!(quest_ty, "function");
    assert!(!quest_value);
    assert_eq!(color_ty, "function");
    assert_eq!(hex_ty, "function");
    assert_eq!(campaign_ty, "function");
    assert_eq!(campaign_name, "Broken Shore");
}
#[test]
fn test_startup_bootstrap_namespaces_exist() {
    let env = WowLuaEnv::new().unwrap();
    let (
        chat_ty,
        replaced_message,
        chat_restricted,
        nav_ty,
        nav_distance,
        nav_frame_ty,
        token_ty,
        commerce_enabled,
        commerce_poll_seconds,
        commerce_balance_enabled,
    ): (
        String,
        String,
        bool,
        String,
        i64,
        String,
        String,
        bool,
        i64,
        bool,
    ) = env
        .eval(
            r#"
            local enabled, pollSeconds, balanceEnabled = C_WowTokenPublic.GetCommerceSystemStatus()
            return type(C_ChatInfo.PerformEmote),
                C_ChatInfo.ReplaceIconAndGroupExpressions("{rt1} hello"),
                C_ChatInfo.AreOutgoingAddonChatMessagesRestricted(),
                type(C_Navigation.GetDistance),
                C_Navigation.GetDistance(),
                type(C_Navigation.GetFrame()),
                type(C_WowTokenPublic.GetCommerceSystemStatus),
                enabled,
                pollSeconds,
                balanceEnabled
            "#,
        )
        .unwrap();

    assert_eq!(chat_ty, "function");
    let expected_message = if cfg!(feature = "retail-12-0-5") {
        "|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_1:0|t hello"
    } else {
        "{rt1} hello"
    };
    assert_eq!(replaced_message, expected_message);
    assert!(!chat_restricted);
    assert_eq!(nav_ty, "function");
    assert_eq!(nav_distance, 0);
    assert_eq!(nav_frame_ty, "nil");
    assert_eq!(token_ty, "function");
    assert!(commerce_enabled);
    assert_eq!(commerce_poll_seconds, 60);
    assert!(!commerce_balance_enabled);

    let (market_price_ty, market_price, guaranteed_price_ty, guaranteed_price): (
        String,
        i64,
        String,
        i64,
    ) = env
        .eval(
            r#"
            return type(C_WowTokenPublic.GetCurrentMarketPrice),
                C_WowTokenPublic.GetCurrentMarketPrice(),
                type(C_WowTokenPublic.GetGuaranteedPrice),
                C_WowTokenPublic.GetGuaranteedPrice()
            "#,
        )
        .unwrap();

    assert_eq!(market_price_ty, "function");
    assert_eq!(market_price, 0);
    assert_eq!(guaranteed_price_ty, "function");
    assert_eq!(guaranteed_price, 0);
}
#[test]
fn test_startup_service_namespaces_exist() {
    let env = WowLuaEnv::new().unwrap();
    let (
        voices_ty,
        voices_len,
        transcription_ty,
        transcription_allowed,
        calendar_ty,
        month_day,
        adjusted_day,
        reset_ty,
        reset_start,
    ): (String, i64, String, bool, String, i64, i64, String, i64) = env
        .eval(
            r#"
            local now = C_DateAndTime.GetCurrentCalendarTime()
            local tomorrow = C_DateAndTime.AdjustTimeByDays(now, 1)
            return type(C_VoiceChat.GetTtsVoices),
                #C_VoiceChat.GetTtsVoices(),
                type(C_VoiceChat.IsTranscriptionAllowed),
                C_VoiceChat.IsTranscriptionAllowed(),
                type(now),
                now.monthDay or 0,
                tomorrow.monthDay or 0,
                type(C_DateAndTime.GetWeeklyResetStartTime),
                C_DateAndTime.GetWeeklyResetStartTime()
            "#,
        )
        .unwrap();

    assert_eq!(voices_ty, "function");
    assert_eq!(voices_len, 0);
    assert_eq!(transcription_ty, "function");
    assert!(!transcription_allowed);
    assert_eq!(calendar_ty, "table");
    assert!(month_day > 0);
    assert_eq!(adjusted_day, month_day + 1);
    assert_eq!(reset_ty, "function");
    assert_eq!(reset_start, 0);

    let (
        shop_ty,
        shop_enabled,
        has_new_products_ty,
        has_new_products,
        token_secure_ty,
        price_lock_duration,
        token_count,
    ): (String, bool, String, bool, String, i64, i64) = env
        .eval(
            r#"
            return type(C_CatalogShop.IsShop2Enabled),
                C_CatalogShop.IsShop2Enabled(),
                type(C_CatalogShop.HasNewProducts),
                C_CatalogShop.HasNewProducts(),
                type(C_WowTokenSecure.GetPriceLockDuration),
                C_WowTokenSecure.GetPriceLockDuration(),
                C_WowTokenSecure.GetTokenCount()
            "#,
        )
        .unwrap();

    assert_eq!(shop_ty, "function");
    assert!(!shop_enabled);
    assert_eq!(has_new_products_ty, "function");
    assert!(!has_new_products);
    assert_eq!(token_secure_ty, "function");
    assert_eq!(price_lock_duration, 900);
    assert_eq!(token_count, 2);
}
#[test]
fn test_startup_runtime_method_and_namespace_gaps_exist() {
    let env = WowLuaEnv::new().unwrap();
    let (
        same_group,
        group_text,
        group_order,
        group_categories_ty,
        groups_ty,
        item_button_scale_ty,
        count_scale,
        calculate_action_ty,
        action_slot,
        force_update_calls,
        private_warning_ty,
        pet_effects_count,
    ): (
        bool,
        String,
        i64,
        String,
        String,
        String,
        f64,
        String,
        i64,
        i64,
        String,
        i64,
    ) = env
        .eval(
            r#"
            local panel = CreateFrame("Frame")
            local firstGroup = panel:GetOrCreateGroup("advanced", 7)
            local secondGroup = panel:GetOrCreateGroup("advanced", 3)

            local itemButton = CreateFrame("Button")
            local count = itemButton:CreateFontString(nil, "OVERLAY")
            itemButton.Count = count
            itemButton:SetItemButtonScale(1.25)

            local actionButton = CreateFrame("Button")
            actionButton:SetID(3)

            local timerContainer = CreateFrame("Frame")
            local calls = 0
            timerContainer.activeTimers = {
                one = { OnUpdate = function() calls = calls + 1 end },
                two = { OnUpdate = function() calls = calls + 1 end },
            }
            timerContainer:ForceUpdateTimers()

            return firstGroup == secondGroup,
                firstGroup.groupText,
                firstGroup.order,
                type(firstGroup.categories),
                type(panel.groups),
                type(itemButton.SetItemButtonScale),
                count:GetScale(),
                type(actionButton.CalculateAction),
                actionButton:CalculateAction(),
                calls,
                type(C_UnitAuras.SetPrivateWarningTextAnchor),
                #{C_PetBattles.GetAllEffectNames()}
            "#,
        )
        .unwrap();

    assert!(same_group);
    assert_eq!(group_text, "advanced");
    assert_eq!(group_order, 7);
    assert_eq!(group_categories_ty, "table");
    assert_eq!(groups_ty, "table");
    assert_eq!(item_button_scale_ty, "function");
    assert_eq!(count_scale, 1.25);
    assert_eq!(calculate_action_ty, "function");
    assert_eq!(action_slot, 3);
    assert_eq!(force_update_calls, 2);
    assert_eq!(private_warning_ty, "function");
    assert_eq!(pet_effects_count, 0);
}
