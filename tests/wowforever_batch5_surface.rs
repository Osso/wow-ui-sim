#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_documented_events_register_and_reject_unknown_names() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local frame = CreateFrame("Frame")
        for _, event in ipairs({"DISCORD_SERVER_LIST_UPDATE", "LEGACY_FRIEND_SYSTEM_STATUS_UPDATED"}) do
            frame:RegisterEvent(event)
            assert(frame:IsEventRegistered(event))
            frame:UnregisterEvent(event)
            assert(not frame:IsEventRegistered(event))
        end
        assert(not pcall(frame.RegisterEvent, frame, "FOREVER_INVENTED_EVENT"))
    "#).unwrap();
}

#[test]
fn forever_guild_and_friends_documented_events_register() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local rejected = {}
        for _, event in ipairs({
            "DISCORD_GUILD_LOBBY_UPDATE",
            "DISCORD_GUILD_SETTINGS_UPDATE",
            "SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED",
            "NEW_MATCHMAKING_PARTY_INVITE",
        }) do
            local ok = pcall(frame.RegisterEvent, frame, event)
            if not ok then
                rejected[#rejected + 1] = event
            else
                assert(frame:IsEventRegistered(event))
                frame:UnregisterEvent(event)
                assert(not frame:IsEventRegistered(event))
            end
        end
        assert(not pcall(frame.RegisterEvent, frame, "FOREVER_INVENTED_EVENT"))
        assert(#rejected == 0, table.concat(rejected, ", "))
    "#,
    )
    .unwrap();
}

#[test]
fn forever_aura_styles_support_deprecated_vendor_aliases() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local styles = Enum.CustomAuraButtonDispelTypeTextureStyle
        for index, name in ipairs({"Border", "BorderWithIcon", "Icon", "PreserveAsset", "CustomAsset"}) do
            assert(styles[name] == index - 1)
        end
        local meta = Enum.CustomAuraButtonDispelTypeTextureStyleMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 4 and meta.NumValues == 5)
    "#).unwrap();
    let source = std::fs::read_to_string(
        wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join("Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua"),
    )
    .unwrap();
    env.exec(&source).unwrap();
    env.exec("assert(AuraButtonBorderStyle.Atlas == 1 and AuraButtonBorderStyle.Color == 3)")
        .unwrap();
}

#[test]
fn forever_curio_rarity_supports_epic_tier_two_keys() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local rarity = Enum.CurioRarity
        assert(rarity.Common == 1 and rarity.Uncommon == 2 and rarity.Rare == 3)
        assert(rarity.Epic == 4 and rarity.EpicTier2 == 5)
        local colors = {[rarity.EpicTier2] = "epic-tier-two"}
        assert(colors[5] == "epic-tier-two")
        local meta = Enum.CurioRarityMeta
        assert(meta.MinValue == 1 and meta.MaxValue == 5 and meta.NumValues == 5)
    "#,
    )
    .unwrap();
}
