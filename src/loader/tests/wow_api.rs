//! Tests for WoW API coverage — verifying that the simulator implements
//! the globals, enums, and C_* namespaces that addons depend on.

use super::*;

#[path = "wow_api_equipment_set.rs"]
mod wow_api_equipment_set;

// ---------------------------------------------------------------------------
// Enum.BagIndex
// ---------------------------------------------------------------------------

#[test]
fn test_enum_bag_index_backpack_slots() {
    let env = WowLuaEnv::new().unwrap();
    let v: i32 = env.eval("return Enum.BagIndex.Backpack").unwrap();
    assert_eq!(v, 0, "Backpack should be 0");

    let _: i32 = env.eval("return Enum.BagIndex.Bag_1").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.Bag_2").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.Bag_3").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.Bag_4").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.ReagentBag").unwrap();
}

#[test]
fn test_enum_bag_index_retail_bank_slots() {
    let env = WowLuaEnv::new().unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.Characterbanktab").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.CharacterBankTab_1").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.CharacterBankTab_6").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.AccountBankTab_1").unwrap();
    let _: i32 = env.eval("return Enum.BagIndex.AccountBankTab_5").unwrap();
}

// Bank, BankBag_1–7, Reagentbank: Classic-only enum values, not in retail BagIndex.
// The sim runs as retail (WOW_PROJECT_MAINLINE). BetterBags only uses these
// in its `else` (non-retail) code path.

// ---------------------------------------------------------------------------
// Enum.ItemQuality
// ---------------------------------------------------------------------------

#[test]
fn test_enum_item_quality_values_and_ordering() {
    let env = WowLuaEnv::new().unwrap();
    let poor: i32 = env.eval("return Enum.ItemQuality.Poor").unwrap();
    let common: i32 = env.eval("return Enum.ItemQuality.Common").unwrap();
    let uncommon: i32 = env.eval("return Enum.ItemQuality.Uncommon").unwrap();
    let rare: i32 = env.eval("return Enum.ItemQuality.Rare").unwrap();
    let epic: i32 = env.eval("return Enum.ItemQuality.Epic").unwrap();
    let legendary: i32 = env.eval("return Enum.ItemQuality.Legendary").unwrap();
    let _: i32 = env.eval("return Enum.ItemQuality.Artifact").unwrap();
    let _: i32 = env.eval("return Enum.ItemQuality.Heirloom").unwrap();
    let _: i32 = env.eval("return Enum.ItemQuality.WoWToken").unwrap();

    assert!(poor < common);
    assert!(common < uncommon);
    assert!(uncommon < rare);
    assert!(rare < epic);
    assert!(epic < legendary);
}

// ---------------------------------------------------------------------------
// Enum.ItemClass
// ---------------------------------------------------------------------------

#[test]
fn test_enum_item_class_values() {
    let env = WowLuaEnv::new().unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Consumable").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Container").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Weapon").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Armor").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Reagent").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Tradegoods").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Questitem").unwrap();
    let _: i32 = env.eval("return Enum.ItemClass.Miscellaneous").unwrap();
}

// ---------------------------------------------------------------------------
// Enum.InventoryType
// ---------------------------------------------------------------------------

#[test]
fn test_enum_inventory_type_values() {
    let env = WowLuaEnv::new().unwrap();
    let _: i32 = env.eval("return Enum.InventoryType.IndexHeadType").unwrap();
    let _: i32 = env.eval("return Enum.InventoryType.IndexNeckType").unwrap();
    let _: i32 = env
        .eval("return Enum.InventoryType.IndexShoulderType")
        .unwrap();
    let _: i32 = env
        .eval("return Enum.InventoryType.IndexChestType")
        .unwrap();
    let _: i32 = env
        .eval("return Enum.InventoryType.IndexWeaponType")
        .unwrap();
    let _: i32 = env
        .eval("return Enum.InventoryType.IndexShieldType")
        .unwrap();
    let _: i32 = env
        .eval("return Enum.InventoryType.Index2HweaponType")
        .unwrap();
}

// ---------------------------------------------------------------------------
// Enum.BankType
// ---------------------------------------------------------------------------

#[test]
fn test_enum_bank_type_values() {
    let env = WowLuaEnv::new().unwrap();
    let _: i32 = env.eval("return Enum.BankType.Character").unwrap();
    let _: i32 = env.eval("return Enum.BankType.Account").unwrap();
}

// ---------------------------------------------------------------------------
// INVSLOT constants
// ---------------------------------------------------------------------------

#[test]
fn test_invslot_constants() {
    let env = WowLuaEnv::new().unwrap();
    let head: i32 = env.eval("return INVSLOT_HEAD").unwrap();
    assert!(head > 0);
    let _: i32 = env.eval("return INVSLOT_NECK").unwrap();
    let _: i32 = env.eval("return INVSLOT_SHOULDER").unwrap();
    let _: i32 = env.eval("return INVSLOT_CHEST").unwrap();
    let _: i32 = env.eval("return INVSLOT_MAINHAND").unwrap();
    let _: i32 = env.eval("return INVSLOT_OFFHAND").unwrap();
    let _: i32 = env.eval("return INVSLOT_BACK").unwrap();
    let _: i32 = env.eval("return INVSLOT_TABARD").unwrap();
    let _: i32 = env.eval("return INVSLOT_TRINKET1").unwrap();
    let _: i32 = env.eval("return INVSLOT_TRINKET2").unwrap();
    let _: i32 = env.eval("return INVSLOT_FINGER1").unwrap();
    let _: i32 = env.eval("return INVSLOT_FINGER2").unwrap();
}

// ---------------------------------------------------------------------------
// Color globals
// ---------------------------------------------------------------------------

#[test]
fn test_color_globals_expose_expected_rgb_values() {
    let env = WowLuaEnv::new().unwrap();

    let faction_ok: bool = env
        .eval(
            r#"
            local r, g, b = PLAYER_FACTION_COLOR_HORDE:GetRGB()
            return math.abs(r - 0.90196) < 0.0001
                and math.abs(g - 0.05098) < 0.0001
                and math.abs(b - 0.07059) < 0.0001
            "#,
        )
        .unwrap();
    assert!(faction_ok);

    let raid_ok: bool = env
        .eval(
            r#"
            local r, g, b = RAID_CLASS_COLORS.WARRIOR:GetRGB()
            return math.abs(r - 0.78) < 0.0001
                and math.abs(g - 0.61) < 0.0001
                and math.abs(b - 0.43) < 0.0001
            "#,
        )
        .unwrap();
    assert!(raid_ok);

    let debuff_ok: bool = env
        .eval(
            r#"
            local magic = DebuffTypeColor.Magic
            return type(DebuffTypeColor) == "table"
                and type(magic) == "table"
                and type(magic.r) == "number"
                and type(magic.g) == "number"
                and type(magic.b) == "number"
            "#,
        )
        .unwrap();
    assert!(debuff_ok);

    let legacy_debuff_color_ok: bool = env
        .eval(
            r#"
            local texture = UIParent:CreateTexture()
            texture:SetVertexColor(DEBUFF_TYPE_MAGIC_COLOR:GetRGBA())
            local r, g, b, a = texture:GetVertexColor()
            return math.abs(r - 0.2) < 0.0001
                and math.abs(g - 0.6) < 0.0001
                and math.abs(b - 1.0) < 0.0001
                and math.abs(a - 1.0) < 0.0001
            "#,
        )
        .unwrap();
    assert!(legacy_debuff_color_ok);
}

// ---------------------------------------------------------------------------
// Item quality description globals
// ---------------------------------------------------------------------------

#[test]
fn test_item_quality_desc_globals() {
    let env = WowLuaEnv::new().unwrap();
    for i in 0..=8 {
        let expr = format!("return type(ITEM_QUALITY{}_DESC)", i);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "string", "ITEM_QUALITY{}_DESC should be string", i);
    }
}

// ---------------------------------------------------------------------------
// Expansion globals
// ---------------------------------------------------------------------------

#[test]
fn test_le_expansion_constants() {
    let env = WowLuaEnv::new().unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_CLASSIC").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_BURNING_CRUSADE").unwrap();
    let _: i32 = env
        .eval("return LE_EXPANSION_WRATH_OF_THE_LICH_KING")
        .unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_CATACLYSM").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_MISTS_OF_PANDARIA").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_WARLORDS_OF_DRAENOR").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_LEGION").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_BATTLE_FOR_AZEROTH").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_SHADOWLANDS").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_DRAGONFLIGHT").unwrap();
    let _: i32 = env.eval("return LE_EXPANSION_WAR_WITHIN").unwrap();
}

#[test]
fn test_wrong_constant_snapshot_matches_expected_values() {
    let env = WowLuaEnv::new().unwrap();

    let expansion_level: i32 = env.eval("return LE_EXPANSION_LEVEL_CURRENT").unwrap();
    let expected_expansion = if cfg!(feature = "client-wowforever") {
        0
    } else {
        11
    };
    assert_eq!(expansion_level, expected_expansion);

    let autocomplete: (i32, i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            return LE_AUTOCOMPLETE_PRIORITY_OTHER,
                LE_AUTOCOMPLETE_PRIORITY_INTERACTED,
                LE_AUTOCOMPLETE_PRIORITY_IN_GROUP,
                LE_AUTOCOMPLETE_PRIORITY_GUILD,
                LE_AUTOCOMPLETE_PRIORITY_FRIEND,
                LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER,
                LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER_SAME_REALM
            "#,
        )
        .unwrap();
    assert_eq!(autocomplete, (1, 2, 3, 4, 5, 6, 7));

    let lfg_categories: (i32, i32, i32) = env
        .eval("return LE_LFG_CATEGORY_LFR, LE_LFG_CATEGORY_RF, LE_LFG_CATEGORY_SCENARIO")
        .unwrap();
    assert_eq!(lfg_categories, (2, 3, 4));

    let lfg_display_types: (i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            return LE_LFG_LIST_DISPLAY_TYPE_ROLE_COUNT,
                LE_LFG_LIST_DISPLAY_TYPE_ROLE_ENUMERATE,
                LE_LFG_LIST_DISPLAY_TYPE_CLASS_ENUMERATE,
                LE_LFG_LIST_DISPLAY_TYPE_HIDE_ALL,
                LE_LFG_LIST_DISPLAY_TYPE_PLAYER_COUNT
            "#,
        )
        .unwrap();
    assert_eq!(lfg_display_types, (1, 2, 3, 4, 5));

    let game_errors: (i32, i32, i32, i32) = env
        .eval(
            r#"
            return LE_GAME_ERR_SYSTEM,
                LE_GAME_ERR_BAG_FULL,
                LE_GAME_ERR_NOT_IN_GROUP,
                LE_GAME_ERR_WRONG_SLOT
            "#,
        )
        .unwrap();
    assert_eq!(game_errors, (1, 14, 106, 12));

    let max_lengths: (i32, i32, i32) = env
        .eval(
            "return MAX_CHARACTER_NAME_BYTES, MAX_COMMUNITY_NAME_LENGTH, MAX_COMMUNITY_NAME_LENGTH_NO_CHANNEL",
        )
        .unwrap();
    assert_eq!(max_lengths, (305, 12, 24));

    let strings: (String, String, String, String, String, String) = env
        .eval(
            r#"
            return ITEM_BNETACCOUNTBOUND,
                ITEM_COOLDOWN_TIME,
                ITEM_REQ_ALLIANCE,
                ITEM_REQ_HORDE,
                SPELL_FAILED_NOT_READY,
                RAID_BOSSES
            "#,
        )
        .unwrap();
    assert_eq!(
        strings,
        (
            "Warbound".into(),
            "Cooldown remaining: %s".into(),
            "Alliance Only".into(),
            "Horde Only".into(),
            "Not yet recovered".into(),
            "Bosses".into(),
        )
    );
}

#[test]
fn test_legacy_item_quality_constants_match_enum_item_quality() {
    let env = WowLuaEnv::new().unwrap();

    let legacy_aliases: (i32, i32, i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            return LE_ITEM_QUALITY_COMMON,
                LE_ITEM_QUALITY_UNCOMMON,
                LE_ITEM_QUALITY_RARE,
                LE_ITEM_QUALITY_EPIC,
                LE_ITEM_QUALITY_LEGENDARY,
                LE_ITEM_QUALITY_ARTIFACT,
                LE_ITEM_QUALITY_HEIRLOOM,
                LE_ITEM_QUALITY_WOW_TOKEN
            "#,
        )
        .unwrap();

    let enum_values: (i32, i32, i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            return Enum.ItemQuality.Common,
                Enum.ItemQuality.Uncommon,
                Enum.ItemQuality.Rare,
                Enum.ItemQuality.Epic,
                Enum.ItemQuality.Legendary,
                Enum.ItemQuality.Artifact,
                Enum.ItemQuality.Heirloom,
                Enum.ItemQuality.WoWToken
            "#,
        )
        .unwrap();

    assert_eq!(legacy_aliases, enum_values);
}

#[test]
fn test_legacy_wow_token_constants_match_expected_values() {
    let env = WowLuaEnv::new().unwrap();

    let token_constants: (i32, i32, i32) = env
        .eval(
            r#"
            return LE_TOKEN_REDEEM_TYPE_GAME_TIME,
                LE_TOKEN_REDEEM_TYPE_BALANCE,
                LE_TOKEN_RESULT_ERROR_BALANCE_NEAR_CAP
            "#,
        )
        .unwrap();

    assert_eq!(token_constants, (1, 2, 10));
}

#[test]
fn test_legacy_garrison_constants_match_expected_values() {
    let env = WowLuaEnv::new().unwrap();

    let garrison_constants: (i32, i32, i32) = env
        .eval(
            r#"
            return LE_FOLLOWER_MISSION_COMPLETE_STATE_ALIVE,
                LE_FOLLOWER_MISSION_COMPLETE_STATE_SAVED,
                LE_FOLLOWER_TYPE_GARRISON_7_0
            "#,
        )
        .unwrap();

    assert_eq!(garrison_constants, (1, 3, 4));
}

#[test]
fn test_expansion_name_globals() {
    let env = WowLuaEnv::new().unwrap();
    for i in 0..=10 {
        let expr = format!("return type(EXPANSION_NAME{})", i);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "string", "EXPANSION_NAME{} should be string", i);
    }
}

#[test]
fn test_character_select_map_scene_highlight_globals_exist() {
    let env = WowLuaEnv::new().unwrap();

    let start_ty: String = env
        .eval("return type(MapSceneCharacterHighlightStart)")
        .unwrap();
    assert_eq!(start_ty, "function");

    let end_ty: String = env
        .eval("return type(MapSceneCharacterHighlightEnd)")
        .unwrap();
    assert_eq!(end_ty, "function");

    env.exec(
        r#"
        MapSceneCharacterHighlightStart("Player-1-00000001")
        MapSceneCharacterHighlightEnd("Player-1-00000001")
        "#,
    )
    .unwrap();
}

// WOW_PROJECT_ID, WOW_PROJECT_MAINLINE: from Blizzard_SharedXML/ProjectConstants.lua
// Tested via Lua addon tests (run-tests), not available in bare WowLuaEnv.

// ---------------------------------------------------------------------------
// C_Container API
// ---------------------------------------------------------------------------

#[test]
fn test_c_container_get_num_slots() {
    let env = WowLuaEnv::new().unwrap();
    let slots: i32 = env
        .eval("return C_Container.GetContainerNumSlots(0)")
        .unwrap();
    assert!(slots >= 0);
}

#[test]
fn test_c_container_get_num_free_slots() {
    let env = WowLuaEnv::new().unwrap();
    let free: i32 = env
        .eval("return C_Container.GetContainerNumFreeSlots(0)")
        .unwrap();
    assert!(free >= 0);
}

#[test]
fn test_c_container_functions_exist() {
    let env = WowLuaEnv::new().unwrap();
    for f in &[
        "ContainerIDToInventoryID",
        "GetBagName",
        "GetContainerItemPurchaseInfo",
        "IsBattlePayItem",
        "UseContainerItem",
        "PickupContainerItem",
        "SplitContainerItem",
        "GetContainerItemInfo",
        "GetContainerItemQuestInfo",
        "GetContainerItemID",
        "GetContainerItemLink",
    ] {
        let expr = format!("return type(C_Container.{})", f);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "function", "C_Container.{} should be function", f);
    }
}

#[test]
fn test_c_container_empty_slot_returns_nil() {
    let env = WowLuaEnv::new().unwrap();
    let is_nil: bool = env
        .eval("return C_Container.GetContainerItemInfo(0, 999) == nil")
        .unwrap();
    assert!(is_nil);
}

// ---------------------------------------------------------------------------
// C_NewItems API
// ---------------------------------------------------------------------------

#[test]
fn test_c_new_items_api() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env.eval("return C_NewItems.IsNewItem(0, 1)").unwrap();
    assert!(!result);

    for f in &["RemoveNewItem", "ClearAll"] {
        let expr = format!("return type(C_NewItems.{})", f);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "function");
    }
}

// ---------------------------------------------------------------------------
// C_CurrencyInfo API
// ---------------------------------------------------------------------------

#[test]
fn test_c_currency_info_api() {
    let env = WowLuaEnv::new().unwrap();
    let size: i32 = env
        .eval("return C_CurrencyInfo.GetCurrencyListSize()")
        .unwrap();
    assert!(size >= 0);

    for f in &[
        "GetCurrencyListInfo",
        "GetCoinIcon",
        "GetCoinText",
        "GetCoinTextureString",
        "GetAzeriteCurrencyID",
        "GetWarResourcesCurrencyID",
    ] {
        let expr = format!("return type(C_CurrencyInfo.{})", f);
        let ty: String = env.eval(&expr).unwrap();
        assert_eq!(ty, "function");
    }

    let (azerite_id, war_resources_id): (i32, i32) = env
        .eval("return C_CurrencyInfo.GetAzeriteCurrencyID(), C_CurrencyInfo.GetWarResourcesCurrencyID()")
        .unwrap();
    assert_eq!(azerite_id, 1553);
    assert_eq!(war_resources_id, 1560);
}

// ---------------------------------------------------------------------------
// C_Bank API
// ---------------------------------------------------------------------------

#[test]
fn test_c_bank_api() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(C_Bank)").unwrap();
    assert_eq!(ty, "table");
    let fn_ty: String = env.eval("return type(C_Bank.FetchDepositedMoney)").unwrap();
    assert_eq!(fn_ty, "function");
}

// ---------------------------------------------------------------------------
// C_Timer API
// ---------------------------------------------------------------------------

#[test]
fn test_c_timer_functions_exist() {
    let env = WowLuaEnv::new().unwrap();
    let after_ty: String = env.eval("return type(C_Timer.After)").unwrap();
    assert_eq!(after_ty, "function");
    let new_timer_id_ty: String = env.eval("return type(C_Timer.NewTimerID)").unwrap();
    assert_eq!(new_timer_id_ty, "function");
    let new_ty: String = env.eval("return type(C_Timer.NewTimer)").unwrap();
    assert_eq!(new_ty, "function");
}

#[test]
fn test_c_timer_new_timer_id_shares_timer_id_space() {
    let env = WowLuaEnv::new().unwrap();
    let ids: (f64, f64, f64) = env
        .eval(
            r#"
            local id1 = C_Timer.NewTimerID()
            local handle = C_Timer.NewTimer(0, function() end)
            local id2 = C_Timer.NewTimerID()
            return id1, handle.__id, id2
            "#,
        )
        .unwrap();

    assert!(
        ids.0 < ids.1,
        "first timer id should precede the timer handle id"
    );
    assert!(
        ids.1 < ids.2,
        "timer handle id should precede the second timer id"
    );
    assert_eq!(ids.1 - ids.0, 1.0, "timer ids should advance by one");
    assert_eq!(ids.2 - ids.1, 1.0, "timer ids should advance by one");
}

#[test]
fn test_c_timer_new_timer_callback_receives_handle() {
    let env = WowLuaEnv::new().unwrap();
    let expected_id: f64 = env
        .eval(
            r#"
            _timer_callback_id = nil
            local handle = C_Timer.NewTimer(0, function(timerObject)
                _timer_callback_id = timerObject.__id
            end)
            return handle.__id
            "#,
        )
        .unwrap();

    env.process_timers().unwrap();

    let callback_id: f64 = env.eval("return _timer_callback_id or 0").unwrap();
    assert_eq!(callback_id, expected_id);
}

#[test]
fn test_runscript_executes_lua_string() {
    let env = WowLuaEnv::new().unwrap();
    let value: i32 = env
        .eval("RunScript('_runscript_probe = 42'); return _runscript_probe")
        .unwrap();
    assert_eq!(value, 42);
}

// ---------------------------------------------------------------------------
// C_CVar, C_AddOns
// ---------------------------------------------------------------------------

#[test]
fn test_c_cvar_set_cvar_exists() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(C_CVar.SetCVar)").unwrap();
    assert_eq!(ty, "function");
}

#[test]
fn test_c_addons_api() {
    let env = WowLuaEnv::new().unwrap();
    let ty: String = env.eval("return type(C_AddOns)").unwrap();
    assert_eq!(ty, "table");
    let fn_ty: String = env.eval("return type(C_AddOns.IsAddOnLoaded)").unwrap();
    assert_eq!(fn_ty, "function");
}

#[test]
fn test_c_traits_get_node_info_exposes_non_nil_rank_delta_fields() {
    let env = WowLuaEnv::new().unwrap();
    let all_nodes_have_fields: bool = env
        .eval(
            r#"
            local nodeIDs = C_Traits.GetTreeNodes(790)
            for _, nodeID in ipairs(nodeIDs) do
                local info = C_Traits.GetNodeInfo(1, nodeID)
                if not info
                    or info.ranksIncreased == nil
                    or info.entryIDToRanksIncreased == nil
                    or info.totalMaxRanks == nil
                then
                    return false
                end
            end

            local missingNode = C_Traits.GetNodeInfo(1, -1)
            return missingNode
                and missingNode.ranksIncreased ~= nil
                and missingNode.entryIDToRanksIncreased ~= nil
                and missingNode.totalMaxRanks ~= nil
            "#,
        )
        .unwrap();
    assert!(
        all_nodes_have_fields,
        "C_Traits.GetNodeInfo should always provide non-nil rank delta fields",
    );
}

#[test]
fn test_c_traits_get_entry_info_exposes_entry_cost_table() {
    let env = WowLuaEnv::new().unwrap();
    let entries_have_costs: bool = env
        .eval(
            r#"
            local nodeIDs = C_Traits.GetTreeNodes(790)
            for _, nodeID in ipairs(nodeIDs) do
                local nodeInfo = C_Traits.GetNodeInfo(1, nodeID)
                for _, entryID in ipairs(nodeInfo.entryIDs) do
                    local entryInfo = C_Traits.GetEntryInfo(1, entryID)
                    if not entryInfo or type(entryInfo.entryCost) ~= "table" then
                        return false
                    end
                end
            end
            return true
            "#,
        )
        .unwrap();
    assert!(
        entries_have_costs,
        "C_Traits.GetEntryInfo should always provide an entryCost array",
    );
}

#[test]
fn test_c_spell_get_spell_description_returns_compact_trait_text() {
    let env = WowLuaEnv::new().unwrap();
    let has_description: bool = env
        .eval(
            r#"
            local desc = C_Spell.GetSpellDescription(116)
            return type(desc) == "string" and desc ~= ""
            "#,
        )
        .unwrap();
    assert!(
        has_description,
        "C_Spell.GetSpellDescription should return compact spell text for trait-linked spells",
    );
}

#[test]
fn test_c_spell_get_spell_texture_covers_high_id_paladin_talent_spells() {
    let env = WowLuaEnv::new().unwrap();
    let no_fallback_icons: bool = env
        .eval(
            r#"
            local fallback = 136243
            local ids = {
                1272143,
                1232418,
                1279510,
                1242031,
                1247534,
                1230084,
                1232421,
                1234430,
            }
            for _, spellID in ipairs(ids) do
                local fileDataID = select(2, C_Spell.GetSpellTexture(spellID))
                if fileDataID == fallback then
                    return false
                end
            end
            return true
            "#,
        )
        .unwrap();
    assert!(
        no_fallback_icons,
        "high-id paladin talent spells should not fall back to Trade_Engineering icons",
    );
}
