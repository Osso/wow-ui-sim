//! Focused 12.1 service-payload compatibility contracts.

#[cfg(feature = "retail-12-1-0")]
use super::super::*;

#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::state::{
    PlayerChoiceInfo, PlayerChoiceOptionButtonInfo, PlayerChoiceOptionInfo,
    PlayerChoiceOptionRewardInfo, PlayerChoiceRewardCurrencyInfo, PlayerChoiceRewardItemInfo,
    PlayerChoiceRewardReputationInfo,
};

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_battle_net_friend_level_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local level = Enum.BattleNetFriendLevel
            local meta = Enum.BattleNetFriendLevelMeta
            if type(level) ~= "table" or type(meta) ~= "table" then return "tables" end
            if level.BattleTag ~= 1 or level.RealID ~= 2 or level.Title ~= 3 then return "values" end
            if meta.MinValue ~= 1 or meta.MaxValue ~= 3 or meta.NumValues ~= 3 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_tiered_entrance_type_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local entrance = Enum.TieredEntranceType
            if type(entrance) ~= "table" then return "table" end
            if entrance.Invalid ~= 0 or entrance.Delve ~= 1 then return "first" end
            if entrance.Sites ~= 2 or entrance.WorldTier ~= 3 or entrance.Lairs ~= 4 then return "rest" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_battle_net_friend_tag_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local tag = Enum.BattleNetFriendTag
            local meta = Enum.BattleNetFriendTagMeta
            if type(tag) ~= "table" or type(meta) ~= "table" then return "tables" end
            if tag.Professions ~= 0 or tag.PvP ~= 1 or tag.Raiding ~= 2 then return "interests-1" end
            if tag.Dungeons ~= 3 or tag.Delves ~= 4 or tag.Questing ~= 5 or tag.Roleplaying ~= 6 then return "interests-2" end
            if tag.DamagerRole ~= 7 or tag.HealerRole ~= 8 or tag.TankRole ~= 9 then return "roles" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 9 or meta.NumValues ~= 10 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_custom_aura_button_dispel_type_texture_style_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local style = Enum.CustomAuraButtonDispelTypeTextureStyle
            local meta = Enum.CustomAuraButtonDispelTypeTextureStyleMeta
            if type(style) ~= "table" or type(meta) ~= "table" then return "tables" end
            if style.Border ~= 0 or style.BorderWithIcon ~= 1 or style.Icon ~= 2 then return "values-1" end
            if style.PreserveAsset ~= 3 or style.CustomAsset ~= 4 then return "values-2" end
            if table.count(style) ~= 5 then return "count" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 4 or meta.NumValues ~= 5 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_recent_allies_friend_tag_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local tag = Enum.RecentAlliesFriendTag
            local meta = Enum.RecentAlliesFriendTagMeta
            if type(tag) ~= "table" or type(meta) ~= "table" then return "tables" end
            if tag.Professions ~= 0 or tag.PvP ~= 1 or tag.Raiding ~= 2 then return "first" end
            if tag.Dungeons ~= 3 or tag.Delves ~= 4 or tag.Questing ~= 5 then return "last" end
            if table.count(tag) ~= 6 or tag.DamagerRole ~= nil then return "distinct" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 5 or meta.NumValues ~= 6 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_rolodex_legacy_friend_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local rolodex = Enum.RolodexType
            local meta = Enum.RolodexTypeMeta
            if type(rolodex) ~= "table" or type(meta) ~= "table" then return "tables" end
            if rolodex.PvPKill ~= 20 or rolodex.LegacyFriend ~= 23 then return "values" end
            for _, value in pairs(rolodex) do
                if value == 21 or value == 22 then return "gaps" end
            end
            if table.count(rolodex) ~= 22 then return "count" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 23 or meta.NumValues ~= 22 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_social_ui_shared_preload_globals() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local presence = Enum.SocialUIPresenceType
            local meta = Enum.SocialUIPresenceTypeMeta
            if type(presence) ~= "table" or type(meta) ~= "table" then return "tables" end
            if presence.Unknown ~= 0 or presence.Online ~= 1 or presence.Offline ~= 2 then return "presence-1" end
            if presence.Away ~= 3 or presence.Busy ~= 4 or presence.AppearOffline ~= 5 then return "presence-2" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 5 or meta.NumValues ~= 6 then return "metadata" end

            local system = Enum.SocialSystemType
            local systemMeta = Enum.SocialSystemTypeMeta
            if type(system) ~= "table" or type(systemMeta) ~= "table" then return "system-tables" end
            if system.Friends ~= 0 or system.QuickJoin ~= 1 or system.RaidList ~= 2 then return "system-first" end
            if system.RecruitAFriend ~= 3 or system.RecentAllies ~= 4 then return "system-last" end
            if systemMeta.MinValue ~= 0 or systemMeta.MaxValue ~= 4 or systemMeta.NumValues ~= 5 then return "system-metadata" end

            local labelNames = {
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_PROFESSIONS",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_PVP",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_RAIDING",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DUNGEONS",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DELVE",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_QUESTING",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_ROLEPLAYING",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_DPS",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_HEALER",
                "SOCIAL_UI_BATTLE_NET_FRIEND_TAG_LABEL_TANK",
                "SOCIAL_UI_PRESENCE_TYPE_LABEL_UNKNOWN",
                "SOCIAL_UI_PRESENCE_TYPE_LABEL_ONLINE",
                "SOCIAL_UI_PRESENCE_TYPE_LABEL_OFFLINE",
                "SOCIAL_UI_PRESENCE_TYPE_LABEL_AWAY",
                "SOCIAL_UI_PRESENCE_TYPE_LABEL_BUSY",
                "SOCIAL_UI_PRESENCE_TYPE_LABEL_APPEAR_OFFLINE",
            }
            for index, name in ipairs(labelNames) do
                local label = _G[name]
                if type(label) ~= "string" or label == "" then return "label-" .. index end
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_social_ui_block_type_preload_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local block = Enum.SocialUIBlockType
            local meta = Enum.SocialUIBlockTypeMeta
            if type(block) ~= "table" or type(meta) ~= "table" then return "tables" end
            if block.None ~= 0 or block.Ignore ~= 1 or block.BattleNetInviteBlock ~= 2 then return "values" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 2 or meta.NumValues ~= 3 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_cooldown_viewer_sound_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local sound = Enum.CooldownViewerSound
            local meta = Enum.CooldownViewerSoundMeta
            if type(sound) ~= "table" or type(meta) ~= "table" then return "tables" end

            local names = {
                "TextToSpeech",
                "AnimalsCat",
                "AnimalsChicken",
                "AnimalsCow",
                "AnimalsGnoll",
                "AnimalsGoat",
                "AnimalsLion",
                "AnimalsPanther",
                "AnimalsRattlesnake",
                "AnimalsSheep",
                "AnimalsWolf",
                "DevicesBoatHorn",
                "DevicesAirHorn",
                "DevicesBikeHorn",
                "DevicesCashRegister",
                "DevicesJackpotBell",
                "DevicesJackpotCoins",
                "DevicesJackpotFail",
                "DevicesRotaryPhoneDial",
                "DevicesRotaryPhoneRing",
                "DevicesStovePipe",
                "DevicesTrashcanLid",
                "ImpactsAnvilStrike",
                "ImpactsBubbleSmash",
                "ImpactsLowThud",
                "ImpactsMetalClanks",
                "ImpactsMetalRattle",
                "ImpactsMetalScrape",
                "ImpactsMetalWarble",
                "ImpactsPopClick",
                "ImpactsStrangeClang",
                "ImpactsSwordScrape",
                "InstrumentsBellRing",
                "InstrumentsBellTrill",
                "InstrumentsBrass",
                "InstrumentsChimeAscending",
                "InstrumentsGuitarChug",
                "InstrumentsGuitarPinch",
                "InstrumentsPitchPipeDistressed",
                "InstrumentsPitchPipeNote",
                "InstrumentsSynthBig",
                "InstrumentsSynthBuzz",
                "InstrumentsSynthHigh",
                "InstrumentsWarhorn",
                "War2AbstractWhoosh",
                "War2Choir",
                "War2Construction",
                "War2MagicChimes",
                "War2PigSqueal",
                "War2Saws",
                "War2Seal",
                "War2Slow",
                "War2Smith",
                "War2SynthStinger",
                "War2TrumpetRally",
                "War2ZippyMagic",
                "War3Bell",
                "War3CrunchyBell",
                "War3DrumSplash",
                "War3Error",
                "War3Fanfare",
                "War3GateOpen",
                "War3Gold",
                "War3MagicShimmer",
                "War3Ringout",
                "War3Rooster",
                "War3ShimmerBell",
                "War3WolfHowl",
                "ShortBellStrike",
                "ShortBellTree",
                "ShortBigPot",
                "ShortBlades",
                "ShortCoffeeMug",
                "ShortCowBell",
                "ShortFingerSnap",
                "ShortGuitar",
                "ShortKalimba",
                "ShortMetalBladeDrop",
                "ShortMetalBladeOnRod",
                "ShortMetalImpact",
                "ShortMiniWoodXylophone",
                "ShortPaperCup",
                "ShortSheetMetal",
                "ShortStovePipe",
                "ShortStovePipeBlade",
                "ShortSwordShing",
                "ShortSynthBleep",
                "ShortSynthBlurp",
                "ShortSynthError",
                "ShortSynthHigh",
                "ShortTriangle",
                "ShortWaterDrop",
                "ShortWineBottle",
                "ShortWoodXylophone",
            }
            for index, name in ipairs(names) do
                if sound[name] ~= index - 1 then return "sound-" .. index end
            end
            if table.count(sound) ~= #names then return "count" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 93 or meta.NumValues ~= 94 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn test_patch_12_1_5_curio_rarity_includes_epic_tier_2() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local rarity = Enum.CurioRarity
            local metadata = Enum.CurioRarityMeta
            if rarity.Common ~= 1 or rarity.Uncommon ~= 2 or rarity.Rare ~= 3 then return "base-values" end
            if rarity.Epic ~= 4 or rarity.EpicTier2 ~= 5 then return "epic-tier-2" end
            if metadata.MinValue ~= 1 or metadata.MaxValue ~= 5 or metadata.NumValues ~= 5 then return "metadata" end
            local qualityByRarity = {
                [rarity.Common] = Enum.ItemQuality.Common,
                [rarity.Uncommon] = Enum.ItemQuality.Uncommon,
                [rarity.Rare] = Enum.ItemQuality.Rare,
                [rarity.Epic] = Enum.ItemQuality.Epic,
                [rarity.EpicTier2] = Enum.ItemQuality.Epic,
            }
            return qualityByRarity[5] == Enum.ItemQuality.Epic and "ok" or "vendor-map"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_chat_frame_sound_help_strings() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local names = {
                "SLASH_CAA_HELP_SAY_COMBAT_START_SOUND",
                "SLASH_CAA_HELP_SAY_COMBAT_END_SOUND",
                "SLASH_CAA_HELP_SAY_TARGET_CASTS_INTERRUPT_SOUND",
                "SLASH_CAA_HELP_SAY_TARGET_CASTS_INTERRUPT_SUCCESS_SOUND",
                "SLASH_CAA_HELP_WHEN_TARGET_DIES_SOUND",
                "SLASH_CAA_HELP_SAY_IF_TARGETED_SOUND",
                "SLASH_CAA_HELP_DEBUFF_SELF_ALERT_SOUND",
            }
            for index, name in ipairs(names) do
                local formatString = _G[name]
                if type(formatString) ~= "string" or formatString == "" then return "missing-" .. index end
                local formatted = formatString:format(1, 93)
                if not formatted:find("1", 1, true) or not formatted:find("93", 1, true) then
                    return "format-" .. index
                end
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_chat_frame_command_names() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local names = {
                "SLASH_CAA_WHEN_TARGET_DIES",
                "SLASH_CAA_PLAY_SOUND",
            }
            for index, name in ipairs(names) do
                local command = _G[name]
                if type(command) ~= "string" or command == "" then return "missing-" .. index end
                if command ~= string.lower(command) then return "uppercase-" .. index end
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_raid_dispel_overlay_type_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local overlay = Enum.RaidDispelOverlayType
            local meta = Enum.RaidDispelOverlayTypeMeta
            if type(overlay) ~= "table" or type(meta) ~= "table" then return "tables" end
            if overlay.Disabled ~= 0 or overlay.UseDebuffColor ~= 1 or overlay.UseBlack ~= 2 then return "values" end
            if table.count(overlay) ~= 3 then return "extras" end
            if meta.MinValue ~= 0 or meta.MaxValue ~= 2 or meta.NumValues ~= 3 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_lfg_lair_category_constant() {
    let env = WowLuaEnv::new().unwrap();
    let category: i32 = env.eval("return LE_LFG_CATEGORY_LAIR").unwrap();

    assert_eq!(category, 8);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_edit_mode_loss_of_control_enums() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local system = Enum.EditModeSystem
            if system.RaidWarning ~= 24 or system.TotemActionBar ~= 25 or system.LossOfControl ~= 26 then
                return "system"
            end
            local systemMeta = Enum.EditModeSystemMeta
            if systemMeta.MinValue ~= 0 or systemMeta.MaxValue ~= 26 or systemMeta.NumValues ~= 27 then
                return "system-meta"
            end

            local account = Enum.EditModeAccountSetting
            if account.ShowRaidWarning ~= 33 or account.ShowTotemActionBar ~= 34 or account.ShowLossOfControl ~= 35 then
                return "account"
            end
            local accountMeta = Enum.EditModeAccountSettingMeta
            if accountMeta.MinValue ~= 0 or accountMeta.MaxValue ~= 35 or accountMeta.NumValues ~= 36 then
                return "account-meta"
            end

            local lossOfControl = Enum.EditModeLossOfControlSetting
            local lossOfControlMeta = Enum.EditModeLossOfControlSettingMeta
            if type(lossOfControl) ~= "table" or lossOfControl.Size ~= 0 then return "loss-of-control" end
            if lossOfControlMeta.MinValue ~= 0 or lossOfControlMeta.MaxValue ~= 0 or lossOfControlMeta.NumValues ~= 1 then
                return "loss-of-control-meta"
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_visual_alert_type_enum() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local alert = Enum.VisualAlertType
            local meta = Enum.VisualAlertTypeMeta
            if type(alert) ~= "table" or type(meta) ~= "table" then return "tables" end
            if alert.MarchingAnts ~= 1 or alert.MarchingAntsCyan ~= 2 or alert.MarchingAntsRed ~= 3 then return "ants-1" end
            if alert.MarchingAntsGreen ~= 4 or alert.MarchingAntsBlue ~= 5 then return "ants-2" end
            if alert.Flash ~= 6 or alert.FlashCyan ~= 7 or alert.FlashRed ~= 8 then return "flash-1" end
            if alert.FlashGreen ~= 9 or alert.FlashBlue ~= 10 then return "flash-2" end
            if meta.MinValue ~= 1 or meta.MaxValue ~= 10 or meta.NumValues ~= 10 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_pet_and_lfg_payloads() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local pet = C_PetJournal.GetPetInfoTableBySpeciesID(39)
            if type(pet) ~= "table" then return "pet-type" end
            if pet.name ~= "Mechanical Squirrel" then return "pet-name" end
            if pet.icon ~= 132932 or pet.petType ~= 9 or pet.speciesID ~= 39 then return "pet-identity" end
            if pet.isWild ~= false or pet.canBattle ~= true then return "pet-battle" end
            if pet.isTradeable ~= false or pet.isUnique ~= false or pet.obtainable ~= true then return "pet-flags" end
            if pet.canAttachToDecor ~= false or pet.creatureModelScale ~= 1 then return "pet-12-1" end
            if C_PetJournal.GetPetInfoTableBySpeciesID(999999) ~= nil then return "pet-unknown" end

            local listing = C_LFGList.GetSearchResultInfo(7)
            if type(listing) ~= "table" then return "lfg-type" end
            if listing.searchResultID ~= 7 or listing.name ~= "RBG yolo" then return "lfg-identity" end
            if listing.activityID ~= 493 or listing.activityIDs[1] ~= 493 then return "lfg-activity" end
            if listing.numMembers ~= 7 or listing.maxMembers ~= 10 then return "lfg-size" end
            if listing.partyGUID ~= "Party-3-0000-1234-00000007" then return "lfg-guid" end
            if listing.censored ~= false then return "lfg-censored" end
            if C_LFGList.GetSearchResultInfo(999999) ~= nil then return "lfg-unknown" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_player_choice_payload_and_mutator_intent() {
    let env = WowLuaEnv::new().unwrap();
    let default_result: String = env
        .eval(
            r#"
            if type(C_PlayerChoice) ~= "table" then return "namespace" end
            if select('#', C_PlayerChoice.GetCurrentPlayerChoiceInfo()) ~= 0 then return "default-info-count" end
            if C_PlayerChoice.GetCurrentPlayerChoiceInfo() ~= nil then return "default-info" end
            if C_PlayerChoice.GetNumRerolls() ~= 0 then return "default-rerolls" end
            if C_PlayerChoice.GetRemainingTime() ~= nil then return "default-time" end
            if C_PlayerChoice.IsWaitingForPlayerChoiceResponse() ~= false then return "default-waiting" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(default_result, "ok");

    env.state().borrow_mut().player_choice.current = Some(PlayerChoiceInfo {
        object_guid: "Creature-0-0000-0000-00000-12345-0000000000".into(),
        choice_id: 42,
        question_text: "Choose your path".into(),
        pending_choice_text: "Waiting".into(),
        ui_texture_kit: "playerchoice-test".into(),
        hide_warboard_header: false,
        keep_open_after_choice: true,
        show_choices_as_list: true,
        requires_selection: true,
        show_choices_as_grid: false,
        options: vec![PlayerChoiceOptionInfo {
            id: 7,
            description: "Take the reward".into(),
            header: "Reward".into(),
            choice_art_id: 99,
            desaturated_art: false,
            disabled_option: false,
            has_rewards: true,
            reward_info: PlayerChoiceOptionRewardInfo {
                currency_rewards: vec![PlayerChoiceRewardCurrencyInfo {
                    currency_id: 2003,
                    name: "Dragon Isles Supplies".into(),
                    currency_texture: 463446,
                    quantity: 25,
                    is_currency_container: false,
                }],
                item_rewards: vec![PlayerChoiceRewardItemInfo {
                    item_id: 19019,
                    name: "Thunderfury".into(),
                    quantity: 1,
                }],
                reputation_rewards: vec![PlayerChoiceRewardReputationInfo {
                    faction_id: 72,
                    quantity: 100,
                }],
            },
            ui_texture_kit: "playerchoice-option".into(),
            max_stacks: 1,
            buttons: vec![PlayerChoiceOptionButtonInfo {
                id: 7,
                text: "Select".into(),
                disabled: false,
                show_checkmark: true,
                hide_button_show_text: false,
                selected: true,
                confirmation: Some("Confirm".into()),
                tooltip: Some("Choose this reward".into()),
                reward_quest_id: Some(70000),
                sound_kit_id: Some(12867),
                list_text: Some("Reward list entry".into()),
            }],
            widget_set_id: Some(55),
            spell_id: Some(642),
            rarity: Some(3),
            type_art_id: Some(12),
            header_icon_atlas_element: Some("playerchoice-icon".into()),
            sub_header: Some("Epic".into()),
            consolidate_widgets: true,
        }],
        sound_kit_id: Some(100),
        close_ui_sound_kit_id: Some(101),
    });
    {
        let mut state = env.state().borrow_mut();
        state.player_choice.num_rerolls = 2;
        state.player_choice.remaining_time = Some(30.5);
        state.player_choice.waiting_for_response = true;
    }

    let result: String = env
        .eval(
            r#"
            local info = C_PlayerChoice.GetCurrentPlayerChoiceInfo()
            if type(info) ~= "table" or info.choiceID ~= 42 then return "info" end
            if info.objectGUID ~= "Creature-0-0000-0000-00000-12345-0000000000" then return "info-guid" end
            if info.questionText ~= "Choose your path" or info.pendingChoiceText ~= "Waiting" then return "info-text" end
            if info.uiTextureKit ~= "playerchoice-test" or info.hideWarboardHeader ~= false then return "info-display" end
            if info.keepOpenAfterChoice ~= true or info.showChoicesAsList ~= true then return "info-list" end
            if info.requiresSelection ~= true or info.showChoicesAsGrid ~= false then return "info-layout" end
            if info.soundKitID ~= 100 or info.closeUISoundKitID ~= 101 then return "info-sounds" end

            local option = info.options[1]
            if option.id ~= 7 or option.description ~= "Take the reward" or option.header ~= "Reward" then return "option-identity" end
            if option.choiceArtID ~= 99 or option.desaturatedArt ~= false or option.disabledOption ~= false then return "option-art" end
            if option.hasRewards ~= true or option.uiTextureKit ~= "playerchoice-option" then return "option-display" end
            if option.maxStacks ~= 1 or option.widgetSetID ~= 55 or option.spellID ~= 642 then return "option-ids" end
            if option.rarity ~= 3 or option.typeArtID ~= 12 then return "option-types" end
            if option.headerIconAtlasElement ~= "playerchoice-icon" or option.subHeader ~= "Epic" then return "option-header" end
            if option.consolidateWidgets ~= true then return "option-widgets" end

            local button = option.buttons[1]
            if button.id ~= 7 or button.text ~= "Select" or button.disabled ~= false then return "button-identity" end
            if button.showCheckmark ~= true or button.hideButtonShowText ~= false or button.selected ~= true then return "button-flags" end
            if button.confirmation ~= "Confirm" or button.tooltip ~= "Choose this reward" then return "button-text" end
            if button.rewardQuestID ~= 70000 or button.soundKitID ~= 12867 or button.listText ~= "Reward list entry" then return "button-optionals" end

            local currency = option.rewardInfo.currencyRewards[1]
            if currency.currencyId ~= 2003 or currency.name ~= "Dragon Isles Supplies" then return "currency-identity" end
            if currency.currencyTexture ~= 463446 or currency.quantity ~= 25 or currency.isCurrencyContainer ~= false then return "currency-values" end
            local item = option.rewardInfo.itemRewards[1]
            if item.itemId ~= 19019 or item.name ~= "Thunderfury" or item.quantity ~= 1 then return "item" end
            local reputation = option.rewardInfo.repRewards[1]
            if reputation.factionId ~= 72 or reputation.quantity ~= 100 then return "reputation" end
            if C_PlayerChoice.GetNumRerolls() ~= 2 then return "rerolls" end
            if C_PlayerChoice.GetRemainingTime() ~= 30.5 then return "time" end
            if C_PlayerChoice.IsWaitingForPlayerChoiceResponse() ~= true then return "waiting" end
            C_PlayerChoice.SendPlayerChoiceResponse(7)
            C_PlayerChoice.RequestRerollPlayerChoice()
            C_PlayerChoice.OnUIClosed()
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");

    let state = env.state().borrow();
    assert_eq!(state.player_choice.last_response_id, Some(7));
    assert!(state.player_choice.reroll_requested);
    assert!(state.player_choice.ui_closed);
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_tiered_entrance_payloads() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local active = C_DelvesUI.GetActiveDelveTier()
            if active.tier ~= 4 or active.suggestedILvl ~= 610 or active.unlocked ~= true then return "active-scalars" end
            if active.tierDescription ~= "Tier 4" or active.modifierUIWidgetSetID ~= 4404 then return "active-display" end
            if active.lockedReason ~= nil or type(active.rewards) ~= "table" then return "active-optional" end
            local itemReward = active.rewards[1]
            if itemReward.id ~= 228361 or itemReward.quantity ~= 1 then return "item-reward" end
            if itemReward.rewardType ~= Enum.TieredEntranceRewardType.Item or itemReward.context ~= 0 then return "item-reward-type" end
            local currencyReward = active.rewards[2]
            if currencyReward.id ~= 2815 or currencyReward.quantity ~= 25 then return "currency-reward" end
            if currencyReward.rewardType ~= Enum.TieredEntranceRewardType.Currency or currencyReward.context ~= 0 then return "currency-reward-type" end

            local tiers = C_DelvesUI.GetDelveEntranceTiers()
            if #tiers ~= 5 or tiers[1].tier ~= 1 or tiers[5].tier ~= 5 then return "tier-order" end
            if tiers[5].unlocked ~= false or type(tiers[5].lockedReason) ~= "string" then return "locked-tier" end
            if type(tiers[1].rewards) ~= "table" or #tiers[1].rewards ~= 2 then return "tier-rewards" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_spell_cooldown_payload() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let start = state.start_time.elapsed().as_secs_f64();
        state.spell_cooldowns.insert(
            12345,
            crate::lua_api::state::SpellCooldownState {
                start,
                duration: 12.5,
            },
        );
    }

    let result: String = env
        .eval(
            r#"
            local active = C_Spell.GetSpellCooldown(12345)
            if type(active) ~= "table" then return "active-type" end
            if active.startTime < 0 or active.duration ~= 12.5 then return "active-time" end
            if active.isEnabled ~= true or active.isActive ~= true or active.modRate ~= 1 then return "active-flags" end

            local inactive = C_Spell.GetSpellCooldown(999999)
            if type(inactive) ~= "table" then return "inactive-type" end
            if inactive.startTime ~= 0 or inactive.duration ~= 0 then return "inactive-time" end
            if inactive.isEnabled ~= true or inactive.isActive ~= false or inactive.modRate ~= 1 then return "inactive-flags" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn test_patch_12_1_housing_and_forbidden_enums() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local pet = Enum.HousingPetBehaviorType
            local petMeta = Enum.HousingPetBehaviorTypeMeta
            if type(pet) ~= "table" or type(petMeta) ~= "table" then return "pet-tables" end
            if pet.Stationary ~= 0 or pet.Wander ~= 1 then return "pet-values" end
            if petMeta.MinValue ~= 0 or petMeta.MaxValue ~= 1 or petMeta.NumValues ~= 2 then return "pet-metadata" end

            local player = Enum.HouseEditorPlayerType
            local playerMeta = Enum.HouseEditorPlayerTypeMeta
            if type(player) ~= "table" or type(playerMeta) ~= "table" then return "player-tables" end
            if player.None ~= 0 or player.Owner ~= 1 or player.Visitor ~= 2 then return "player-values" end
            if playerMeta.MinValue ~= 0 or playerMeta.MaxValue ~= 2 or playerMeta.NumValues ~= 3 then return "player-metadata" end

            local budget = Enum.HousingBudgetType
            local budgetMeta = Enum.HousingBudgetTypeMeta
            if type(budget) ~= "table" or type(budgetMeta) ~= "table" then return "budget-tables" end
            if budget.RoomPlacement ~= 0 or budget.DecorPlacement ~= 1 or budget.PetDecor ~= 2 then return "budget-values" end
            if budgetMeta.MinValue ~= 0 or budgetMeta.MaxValue ~= 2 or budgetMeta.NumValues ~= 3 then return "budget-metadata" end

            local scope = Enum.HousingHouseScope
            local scopeMeta = Enum.HousingHouseScopeMeta
            if type(scope) ~= "table" or type(scopeMeta) ~= "table" then return "scope-tables" end
            if scope.None ~= 0 or scope.Interior ~= 1 or scope.Exterior ~= 2 then return "scope-values" end
            if scopeMeta.MinValue ~= 0 or scopeMeta.MaxValue ~= 2 or scopeMeta.NumValues ~= 3 then return "scope-metadata" end

            local aspect = Enum.ForbiddenAspect
            local aspectMeta = Enum.ForbiddenAspectMeta
            if type(aspect) ~= "table" or type(aspectMeta) ~= "table" then return "aspect-tables" end
            if aspect.SetToDefaults ~= 1 or aspect.ScriptBindings ~= 2 or aspect.UntrustedScriptExecution ~= 4 then return "aspect-first" end
            if aspect.UntrustedLayoutScriptExecution ~= 8 or aspect.EventRegistrations ~= 16 or aspect.AlwaysPropagateInput ~= 32 then return "aspect-middle-1" end
            if aspect.ScriptedInput ~= 64 or aspect.QueryFocus ~= 128 or aspect.ChangeAnimationTarget ~= 256 then return "aspect-middle-2" end
            if aspect.RemoveSecretAspects ~= 512 or aspect.ChangeParent ~= 1024 then return "aspect-last" end
            if aspectMeta.MinValue ~= 1 or aspectMeta.MaxValue ~= 1024 or aspectMeta.NumValues ~= 11 then return "aspect-metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}
