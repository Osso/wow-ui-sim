use std::fs;

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-wowforever")]
mod forever_shared_enums {
    use super::*;

    fn load_vendor(env: &WowLuaEnv, relative: &str) {
        let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join(relative);
        env.exec(&fs::read_to_string(&path).unwrap())
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    }

    #[test]
    fn forever_shared_enums_match_published_fields_and_metadata() {
        let env = env();
        env.exec(r#"
            checkedEnums = 0
            local wanted = { BattleNetFriendLevel = true, VisualAlertType = true, CooldownViewerSound = true, CombatAudioAlertPulsePercentValues = true }
            APIDocumentation = { AddDocumentationTable = function(_, doc)
                for _, enum in ipairs(doc.Tables or {}) do
                    if wanted[enum.Name] then
                        local actual = assert(Enum[enum.Name], enum.Name)
                        local meta = assert(Enum[enum.Name .. 'Meta'])
                        local count = 0
                        for _ in pairs(actual) do count = count + 1 end
                        assert(count == #enum.Fields)
                        for _, field in ipairs(enum.Fields) do
                            assert(actual[field.Name] == field.EnumValue, field.Name)
                        end
                        assert(meta.MinValue == enum.MinValue)
                        assert(meta.MaxValue == enum.MaxValue)
                        assert(meta.NumValues == #enum.Fields)
                        checkedEnums = checkedEnums + 1
                    end
                end
            end }
        "#).unwrap();
        for file in [
            "BattleNetSharedDocumentation.lua",
            "VisualAlertConstantsDocumentation.lua",
            "CooldownViewerConstantsDocumentation.lua",
            "CombatAudioAlertSharedDocumentation.lua",
        ] {
            load_vendor(&env, &format!("Blizzard_APIDocumentationGenerated/{file}"));
        }
        assert_eq!(env.eval::<i32>("return checkedEnums").unwrap(), 4);
    }

    #[test]
    fn forever_shared_enums_pulse_health_vendor_command_range() {
        let env = env();
        let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
            .unwrap()
            .join("Blizzard_ChatFrame/Shared/TextToSpeechCommands.lua");
        let source = fs::read_to_string(path).unwrap();
        // Execute the unmodified command block; the surrounding slash-command UI
        // is replaced by a recorder so its actual handler can be exercised.
        let block = source
            .split_once("-- Pulse Your Health")
            .unwrap()
            .1
            .split_once("-- Pulse Your Health Volume")
            .unwrap()
            .0;
        env.exec(
            r#"
            CAACommands = { AddCommand = function(_, _, handler, _, _, _, min, max)
                pulseHandler, pulseMin, pulseMax = handler, min, max
            end }
            CombatAudioAlertUtil = {
                EnumeratePulseHealthPercentInfo = function() return ipairs({}) end,
                GetPulseHealthPercentInfo = function(value) return { str = tostring(value) } end,
            }
            function GetInitialSuboptionFailureStrings() return 'invalid', 'invalid' end
            SLASH_CAA_CONFIRMATION = '%s: %s'
            CAA_PULSE_PLAYER_HEALTH_LABEL = 'Health'
            pulseCommand = { GetCommands = function() return {
                SpeakConfirmation = function(_, text) pulseConfirmation = text end,
            } end }
        "#,
        )
        .unwrap();
        env.exec(block).unwrap();
        env.exec(
            r#"
            assert(pulseMin == 0 and pulseMax == 9)
            for value = 0, 9 do
                assert(pulseHandler(pulseCommand, tostring(value)))
                assert(GetCVar('CAAPulsePlayerHealthPercent') == tostring(value))
                assert(pulseConfirmation == 'Health: ' .. value)
            end
            for _, value in ipairs({'-1', '10', 'invalid'}) do
                assert(not pulseHandler(pulseCommand, value))
                assert(GetCVar('CAAPulsePlayerHealthPercent') == '9')
            end
        "#,
        )
        .unwrap();
    }

    #[test]
    fn forever_shared_enums_support_account_friend_rank_queries() {
        let env = env();
        load_vendor(&env, "Blizzard_SharedXML/AccountUtil.lua");
        env.exec(r#"
            assert(BNet_GetFriendLevelRank(Enum.BattleNetFriendLevel.Title) == 1)
            assert(BNet_GetFriendLevelRank(Enum.BattleNetFriendLevel.BattleTag) == 2)
            assert(BNet_GetFriendLevelRank(Enum.BattleNetFriendLevel.RealID) == 3)
            assert(BNet_IsFriendLevelEqualOrHigher(Enum.BattleNetFriendLevel.RealID, Enum.BattleNetFriendLevel.Title))
            assert(not BNet_IsFriendLevelEqualOrHigher(Enum.BattleNetFriendLevel.Title, Enum.BattleNetFriendLevel.RealID))
        "#).unwrap();
    }

    #[test]
    fn forever_shared_enums_populate_sound_alert_data() {
        let env = env();
        load_vendor(
            &env,
            "Blizzard_CooldownViewer/CooldownViewerSettingsConstants.lua",
        );
        load_vendor(
            &env,
            "Blizzard_CooldownViewer/CooldownViewerSoundAlertData.lua",
        );
        env.exec(
            r#"
            local cat = CooldownViewerSoundData[Enum.CooldownViewerSoundCategory.Animals][1]
            assert(cat.soundEnum == 1)
            assert(cat.soundKitID == 316401)
            for _, category in pairs(CooldownViewerSoundData) do
                for _, sound in ipairs(category) do
                    assert(type(sound.soundEnum) == 'number')
                    assert(sound.soundEnum >= 0 and sound.soundEnum <= 93)
                end
            end
        "#,
        )
        .unwrap();
    }
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("failed to create Lua environment")
}

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
#[test]
fn c_string_util_escapes_bytes_and_wraps_nonempty_infixes() {
    let env = env();
    env.eval::<()>(
        r#"
        local util = C_StringUtil
        assert(util.EscapeLuaFormatString("100% ready %% %s") == "100%% ready %%%% %%s")
        assert(util.EscapeLuaFormatString("café\0%") == "café\0%%")
        assert(util.EscapeLuaFormatString("") == "")
        assert(util.EscapeLuaPatterns("^$()%.[]*+-?") == "%^%$%(%)%%%.%[%]%*%+%-%?")
        assert(util.EscapeLuaPatterns("plain text") == "plain text")
        assert(util.EscapeLuaPatterns("") == "")

        local raw = string.char(255, 0, 37, 94)
        assert(util.EscapeLuaFormatString(raw) == string.char(255, 0, 37, 37, 94))
        assert(util.EscapeLuaPatterns(raw) == string.char(255, 0, 37, 37, 37, 94))
        assert(util.WrapString(raw, "[", "]") == "[" .. raw .. "]")
        assert(util.WrapString("middle", raw, raw) == raw .. "middle" .. raw)
        assert(util.WrapString("middle") == "middle")
        assert(util.WrapString("middle", "[") == "[middle")
        assert(util.WrapString("middle", nil, "]") == "middle]")
        assert(util.WrapString("middle", "[", "]") == "[middle]")
        assert(util.WrapString("", "[", "]") == "")
        "#,
    )
    .expect("C_StringUtil must escape exact bytes and wrap only nonempty infixes");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_remove_contiguous_spaces_limits_runs() {
    env()
        .eval::<()>(
            r#"
            local text = "   alpha  beta gamma    "
            local cases = {
                {limit = 0, expected = "alphabetagamma"},
                {limit = 1, expected = " alpha beta gamma "},
                {limit = 2, expected = "  alpha  beta gamma  "},
            }
            for _, case in ipairs(cases) do
                assert(select('#', C_StringUtil.RemoveContiguousSpaces(text, case.limit)) == 1)
                local actual = C_StringUtil.RemoveContiguousSpaces(text, case.limit)
                assert(type(actual) == 'string' and actual == case.expected)
            end
            "#,
        )
        .expect("ASCII space runs at every position truncate to the requested limit");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_remove_contiguous_spaces_preserves_other_bytes() {
    env()
        .eval::<()>(
            r#"
            local rawByte = string.char(255)
            local unicodeText = "café" .. string.char(194, 160)
            local text = rawByte .. "  " .. unicodeText .. "\t  \n  \0  " .. rawByte
            local cases = {
                {limit = 0, expected = rawByte .. unicodeText .. "\t\n\0" .. rawByte},
                {limit = 1, expected = rawByte .. " " .. unicodeText .. "\t \n \0 " .. rawByte},
                {limit = 2, expected = text},
            }
            for _, case in ipairs(cases) do
                assert(select('#', C_StringUtil.RemoveContiguousSpaces(text, case.limit)) == 1)
                local actual = C_StringUtil.RemoveContiguousSpaces(text, case.limit)
                assert(type(actual) == 'string' and actual == case.expected)
            end
            "#,
        )
        .expect("only ASCII spaces change, preserving whitespace, NUL, UTF-8 and raw bytes");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_remove_contiguous_spaces_keeps_empty_and_space_free_strings() {
    env()
        .eval::<()>(
            r#"
            local inputs = {"", "plain", "\t\n\0café" .. string.char(255)}
            for _, text in ipairs(inputs) do
                for _, limit in ipairs({0, 1, 2}) do
                    assert(select('#', C_StringUtil.RemoveContiguousSpaces(text, limit)) == 1)
                    local actual = C_StringUtil.RemoveContiguousSpaces(text, limit)
                    assert(type(actual) == 'string' and actual == text)
                end
            end
            "#,
        )
        .expect("empty and ASCII-space-free strings remain unchanged at every tested limit");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_remove_contiguous_spaces_rejects_invalid_limits() {
    env()
        .eval::<()>(
            r#"
            -- Explicit simulator validation policy, not native coercion/range proof.
            local cases = {
                {name = "negative", value = -1},
                {name = "fractional", value = 0.5},
                {name = "NaN", value = 0 / 0},
                {name = "positive infinity", value = math.huge},
                {name = "negative infinity", value = -math.huge},
                {name = "numeric string", value = "1"},
                {name = "boolean", value = false},
                {name = "table", value = {}},
            }
            for _, case in ipairs(cases) do
                local success = pcall(C_StringUtil.RemoveContiguousSpaces, " a  b ", case.value)
                assert(not success, "invalid limit accepted: " .. case.name)
            end
            assert(not pcall(C_StringUtil.RemoveContiguousSpaces, " a  b "), "missing limit accepted")
            "#,
        )
        .expect("simulator policy rejects missing, nonnumeric, nonfinite and nonintegral limits");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_truncate_when_zero_formats_nonnegative_numbers() {
    env()
        .eval::<()>(
            r#"
            local cases = {
                {value = 0, expected = ""},
                {value = 0.1, expected = ""},
                {value = 0.99, expected = ""},
                {value = 1, expected = "1"},
                {value = 1.9, expected = "1"},
                {value = 2.1, expected = "2"},
                {value = 2.99, expected = "2"},
                {value = 10.75, expected = "10"},
            }
            for _, case in ipairs(cases) do
                assert(select('#', C_StringUtil.TruncateWhenZero(case.value)) == 1)
                local actual = C_StringUtil.TruncateWhenZero(case.value)
                assert(type(actual) == 'string' and actual == case.expected,
                    "unexpected text for " .. tostring(case.value))
            end
            "#,
        )
        .expect("nonnegative values round down, returning empty text for zero integers");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_truncate_when_zero_calls_are_independent() {
    env()
        .eval::<()>(
            r#"
            local inputs = {10.75, 0.99, 2.99, 0, 1.9, 0.1, 10.75}
            local expected = {"10", "", "2", "", "1", "", "10"}
            local results = {}
            for index, value in ipairs(inputs) do
                assert(select('#', C_StringUtil.TruncateWhenZero(value)) == 1)
                results[index] = C_StringUtil.TruncateWhenZero(value)
            end
            for index, text in ipairs(expected) do
                assert(type(results[index]) == 'string' and results[index] == text,
                    "result changed across calls at index " .. tostring(index))
            end
            "#,
        )
        .expect("alternating zero and populated results remain independent across calls");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn c_string_util_truncate_when_zero_rejects_invalid_numbers() {
    env()
        .eval::<()>(
            r#"
            -- Explicit simulator policy, not native validation or coercion proof.
            local cases = {
                {name = "numeric string", value = "1"},
                {name = "boolean", value = false},
                {name = "table", value = {}},
                {name = "NaN", value = 0 / 0},
                {name = "positive infinity", value = math.huge},
                {name = "negative infinity", value = -math.huge},
            }
            for _, case in ipairs(cases) do
                assert(not pcall(C_StringUtil.TruncateWhenZero, case.value),
                    "invalid number accepted: " .. case.name)
            end
            assert(not pcall(C_StringUtil.TruncateWhenZero), "missing number accepted")
            "#,
        )
        .expect("simulator policy rejects missing, nonnumeric and nonfinite numbers");
}

#[cfg(not(any(feature = "profile-retail", feature = "client-ptr")))]
#[test]
fn non_retail_profiles_do_not_publish_retail_string_helpers() {
    env()
        .eval::<()>(
            r#"
            assert(rawget(C_StringUtil, "EscapeLuaFormatString") == nil)
            assert(rawget(C_StringUtil, "EscapeLuaPatterns") == nil)
            assert(rawget(C_StringUtil, "WrapString") == nil)
            "#,
        )
        .expect("classic profiles must not publish retail string helpers");
}

#[cfg(any(feature = "profile-retail", feature = "client-ptr"))]
#[test]
fn c_loot_history_empty_state_has_retail_shapes() {
    let env = env();
    let result: (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local all = C_LootHistory.GetAllEncounterInfos()
            local drops = C_LootHistory.GetSortedDropsForEncounter(1)
            return type(C_LootHistory.GetAllEncounterInfos) == "function",
                   type(C_LootHistory.GetInfoForEncounter) == "function",
                   type(C_LootHistory.GetLootHistoryTime) == "function",
                   type(C_LootHistory.GetSortedDropsForEncounter) == "function",
                   type(C_LootHistory.GetSortedInfoForDrop) == "function",
                   type(all) == "table" and #all == 0,
                   C_LootHistory.GetInfoForEncounter(1) == nil,
                   type(drops) == "table" and #drops == 0,
                   C_LootHistory.GetSortedInfoForDrop(1, 1) == nil,
                   type(C_LootHistory.GetLootHistoryTime()) == "number"
                       and C_LootHistory.GetLootHistoryTime() == 0
            "#,
        )
        .expect("C_LootHistory empty-state probe should evaluate");

    assert!(result.0 && result.1 && result.2 && result.3 && result.4);
    assert!(result.5 && result.6 && result.7 && result.8 && result.9);
}

fn c_api_temporary_shims_source() -> String {
    fs::read_to_string("src/c_api/temporary_shims/mod.rs").unwrap_or_default()
}

#[test]
fn c_api_temporary_shims_module_is_removed() {
    assert!(
        !std::path::Path::new("src/c_api/temporary_shims/mod.rs").exists(),
        "temporary compatibility defaults should live under lua_api::workarounds, not c_api::temporary_shims"
    );
}

#[test]
fn c_api_reorg_keeps_core_namespaces_registered() {
    let env = env();
    let namespaces: (
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
        bool,
    ) = env
        .eval(
            r#"
            return type(C_AddOns) == "table",
                   type(C_Texture) == "table",
                   type(C_XMLUtil) == "table",
                   type(C_Item) == "table",
                   type(C_CurrencyInfo) == "table",
                   type(C_Container) == "table",
                   type(C_ItemUpgrade) == "table",
                   type(C_Spell) == "table",
                   type(C_SpellBook) == "table",
                   type(C_ModelInfo) == "table",
                   type(C_LFGInfo) == "table",
                   type(C_WowTokenSecure) == "table",
                   type(C_FogOfWar) == "table"
        "#,
        )
        .expect("failed to probe C_* namespace registration");

    assert!(namespaces.0, "C_AddOns should stay registered");
    assert!(namespaces.1, "C_Texture should stay registered");
    assert!(namespaces.2, "C_XMLUtil should stay registered");
    assert!(namespaces.3, "C_Item should stay registered");
    assert!(namespaces.4, "C_CurrencyInfo should stay registered");
    assert!(namespaces.5, "C_Container should stay registered");
    assert!(namespaces.6, "C_ItemUpgrade should stay registered");
    assert!(namespaces.7, "C_Spell should stay registered");
    assert!(namespaces.8, "C_SpellBook should stay registered");
    assert!(namespaces.9, "C_ModelInfo should stay registered");
    assert!(namespaces.10, "C_LFGInfo should stay registered");
    assert!(namespaces.11, "C_WowTokenSecure should stay registered");
    assert!(namespaces.12, "C_FogOfWar should stay registered");
}

#[test]
fn c_fog_of_war_unknown_id_keeps_default_shape() {
    let env = env();
    let info: (Option<String>, Option<String>, f64) = env
        .eval(
            r#"
            local info = C_FogOfWar.GetFogOfWarInfo(-1)
            return info.backgroundAtlas, info.maskAtlas, info.maskScalar
        "#,
        )
        .expect("failed to query C_FogOfWar");

    assert_eq!(info.0, None);
    assert_eq!(info.1, None);
    assert_eq!(info.2, 1.0);
}

#[test]
fn c_addons_scripts_disallowed_for_beta_defaults_false() {
    let env = env();
    let result: (String, bool) = env
        .eval(
            r#"
            return type(C_AddOns.GetScriptsDisallowedForBeta),
                   C_AddOns.GetScriptsDisallowedForBeta()
        "#,
        )
        .expect("failed to query C_AddOns.GetScriptsDisallowedForBeta");

    assert_eq!(result, ("function".to_string(), false));
}

#[test]
fn glue_login_static_defaults_are_permanent_shims() {
    let c_login = include_str!("../src/c_api/c_login.rs");
    let c_glue = include_str!("../src/c_api/c_glue.rs");
    let permanent_login = include_str!("../src/c_api/permanent_shims/c_login.rs");
    let permanent_glue = include_str!("../src/c_api/permanent_shims/c_glue.rs");

    for name in [
        "IsLauncherLogin",
        "IsReconnectLoginPossible",
        "GetLastError",
        "ClearLastError",
        "AttemptedLauncherLogin",
        "IsNewPlayer",
    ] {
        assert!(
            !c_login.contains(name),
            "{name} is a static glue default and should not live in the state-backed C_Login module"
        );
        assert!(
            permanent_login.contains(name),
            "{name} should be isolated under permanent C_Login shims"
        );
    }

    assert!(
        !c_glue.contains("IsFirstLoadThisSession"),
        "first-load session default should not live in the state-backed C_Glue module"
    );
    assert!(
        permanent_glue.contains("IsFirstLoadThisSession"),
        "first-load session default should be isolated under permanent C_Glue shims"
    );
}

#[test]
fn configuration_warnings_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_configuration_warnings"),
        "unmodeled C_ConfigurationWarnings defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_configuration_warnings"),
        "C_ConfigurationWarnings should not be wired through c_api registration"
    );
}

#[test]
fn item_targeting_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let item_spell = include_str!("../src/c_api/item_spell/mod.rs");

    assert!(
        !temporary_shims.contains("c_item_targeting"),
        "unmodeled C_Item targeting defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !item_spell.contains("c_item_targeting"),
        "C_Item IsHelpfulItem/IsHarmfulItem defaults should not be wired through c_api item registration"
    );
}

#[test]
fn spell_target_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_spell_target"),
        "unmodeled C_Spell target-spell metadata defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_spell_target"),
        "C_Spell TargetSpell* defaults should not be wired through c_api registration"
    );
}

#[test]
fn merchant_and_raid_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_merchant_raid_defaults"),
        "unmodeled C_MerchantFrame and C_RaidLocks defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_merchant_raid_defaults"),
        "C_MerchantFrame and C_RaidLocks no-state defaults should not be wired through c_api registration"
    );
}

#[test]
fn paper_doll_stagger_default_is_not_c_api_temporary_shim() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_paper_doll_stagger"),
        "unmodeled PaperDoll stagger defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_paper_doll_stagger"),
        "C_PaperDollInfo.GetStaggerPercentage should not be wired through c_api registration"
    );
}

#[test]
fn chat_info_no_state_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_chat_info"),
        "unmodeled C_ChatInfo emote/caution/chat-line defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_chat_info"),
        "C_ChatInfo no-state defaults should not be wired through c_api registration"
    );
}

#[test]
fn container_no_state_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let item_spell = include_str!("../src/c_api/item_spell/mod.rs");

    assert!(
        !temporary_shims.contains("c_container_defaults"),
        "unmodeled C_Container purchase/quest/filter/action defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !item_spell.contains("c_container_defaults"),
        "C_Container no-state defaults should not be wired through c_api item/spell registration"
    );
}

#[test]
fn pet_battle_static_fallbacks_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_pet_battles_static_fallbacks"),
        "unmodeled C_PetBattles journal/trap/select defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_pet_battles_static_fallbacks"),
        "C_PetBattles static fallbacks should not be wired through c_api registration"
    );
}

#[test]
fn party_info_instance_abandon_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_party_info_instance_abandon"),
        "unmodeled C_PartyInfo instance-abandon vote defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_party_info_instance_abandon"),
        "C_PartyInfo instance-abandon defaults should not be wired through c_api registration"
    );
}

#[test]
fn transmog_sets_empty_inventory_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_transmog_sets"),
        "unmodeled C_TransmogSets empty wardrobe-set defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_transmog_sets"),
        "C_TransmogSets empty inventory defaults should not be wired through c_api registration"
    );
}

#[test]
fn level_link_spell_lock_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_level_link_spell_lock"),
        "temporary C_LevelLink spell-lock state belongs in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_level_link_spell_lock"),
        "C_LevelLink spell-lock defaults should not be wired through c_api registration"
    );
}

#[test]
fn campaign_covenant_placeholder_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_campaign_covenant_defaults"),
        "unmodeled C_CampaignInfo and C_CovenantSanctumUI placeholder defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_campaign_covenant_defaults"),
        "campaign/covenant placeholder defaults should not be wired through c_api registration"
    );
}

#[test]
fn date_and_time_deterministic_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_date_and_time"),
        "unmodeled C_DateAndTime deterministic calendar defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_date_and_time"),
        "C_DateAndTime deterministic defaults should not be wired through c_api registration"
    );
}

#[test]
fn contribution_collector_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_contribution_collector"),
        "unmodeled C_ContributionCollector empty/default shapes belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_contribution_collector"),
        "C_ContributionCollector empty/default shapes should not be wired through c_api registration"
    );
}

#[test]
fn prototype_dialog_temporary_state_is_not_c_api_temporary_shim() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_prototype_dialog"),
        "temporary C_PrototypeDialog active/removed transition state belongs in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_prototype_dialog"),
        "C_PrototypeDialog temporary state should not be wired through c_api registration"
    );
}

#[test]
fn pvp_talent_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_specialization_pvp_talents"),
        "unmodeled PvP talent placeholder defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_specialization_pvp_talents"),
        "PvP talent placeholder defaults should not be wired through c_api registration"
    );
}

#[test]
fn spell_metadata_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    for module in [
        "c_spell_classification",
        "c_spell_counts",
        "c_spell_priority_aura",
    ] {
        assert!(
            !temporary_shims.contains(module),
            "unmodeled C_Spell metadata/count defaults belong in lua_api::workarounds::temporary"
        );
        assert!(
            !registration.contains(module),
            "C_Spell metadata/count defaults should not be wired through c_api registration"
        );
    }
}

#[test]
fn minimap_tracking_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_minimap"),
        "unmodeled C_Minimap tracking/radius defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_minimap"),
        "C_Minimap tracking/radius defaults should not be wired through c_api registration"
    );
}

#[test]
fn super_track_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_super_track"),
        "unmodeled C_SuperTrack no-active-target defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_super_track"),
        "C_SuperTrack no-active-target defaults should not be wired through c_api registration"
    );
}

#[test]
fn trade_info_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_trade_info"),
        "unmodeled C_TradeInfo warning/no-op defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_trade_info"),
        "C_TradeInfo warning/no-op defaults should not be wired through c_api registration"
    );
}

#[test]
fn scenario_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_scenario"),
        "unmodeled C_Scenario not-in-scenario defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_scenario"),
        "C_Scenario not-in-scenario defaults should not be wired through c_api registration"
    );
}

#[test]
fn gossip_poi_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");
    let missing_surface = include_str!("../src/lua_api/globals/missing_surface.rs");

    assert!(
        !temporary_shims.contains("c_gossip_info"),
        "unmodeled C_GossipInfo POI lookup defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_gossip_info"),
        "C_GossipInfo POI lookup defaults should not be wired through c_api registration"
    );
    assert!(
        !registration.contains("register_gossip_info_tables")
            && !missing_surface.contains("register_gossip_info_tables"),
        "dead C_GossipInfo fallback registration hooks should stay removed from the C API boundary"
    );
}

#[test]
fn mythic_plus_cache_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_mythic_plus"),
        "unmodeled C_MythicPlus weekly-chest/request defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_mythic_plus"),
        "C_MythicPlus weekly-chest/request defaults should not be wired through c_api registration"
    );
}

#[test]
fn shared_character_services_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_shared_character_services"),
        "unmodeled C_SharedCharacterServices upgrade-distribution defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_shared_character_services"),
        "C_SharedCharacterServices upgrade-distribution defaults should not be wired through c_api registration"
    );
}

#[test]
fn click_bindings_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api_registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_click_bindings"),
        "unmodeled C_ClickBindings profile defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api_registration.contains("c_click_bindings"),
        "C_ClickBindings profile defaults should not be wired through c_api registration"
    );
}

#[test]
fn spell_book_static_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let item_spell = include_str!("../src/c_api/item_spell/mod.rs");

    assert!(
        !temporary_shims.contains("c_spell_book_call_pet"),
        "unmodeled C_SpellBook call-pet and deprecated spellbook defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !item_spell.contains("c_spell_book_call_pet"),
        "C_SpellBook static defaults should not be wired through c_api item/spell registration"
    );
}

#[test]
fn spell_static_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let item_spell = include_str!("../src/c_api/item_spell/mod.rs");
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_spell_static_fallbacks"),
        "unmodeled C_Spell charges/override/visibility/Maw defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !item_spell.contains("c_spell_static_fallbacks"),
        "C_Spell static defaults should not be wired through item/spell C API registration"
    );
    assert!(
        !registration.contains("c_spell_static_fallbacks"),
        "C_Spell static defaults should not be wired through shared C API registration"
    );
}

#[test]
fn map_group_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_map_groups"),
        "unmodeled C_Map group defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_map_groups"),
        "C_Map group defaults should not be wired through C API registration"
    );
}

#[test]
fn perks_program_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_perks_program"),
        "unmodeled C_PerksProgram empty-catalog defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_perks_program"),
        "C_PerksProgram empty-catalog defaults should not be wired through c_api registration"
    );
}

#[test]
fn party_info_static_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");
    let missing_surface = include_str!("../src/lua_api/globals/missing_surface.rs");

    assert!(
        !temporary_shims.contains("c_party_info_static_fallbacks"),
        "unmodeled C_PartyInfo invite/Torghast/walk-in defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_party_info_static_fallbacks"),
        "C_PartyInfo static defaults should not be wired through C API registration"
    );
    assert!(
        !registration.contains("register_party_info_fallback_tables")
            && !missing_surface.contains("register_party_info_fallback_tables"),
        "dead C_PartyInfo fallback registration hooks should stay removed from the C API boundary"
    );
}

#[test]
fn pet_battle_static_defaults_have_no_c_api_fallback_registration_hook() {
    let registration = include_str!("../src/c_api/registration.rs");
    let missing_surface = include_str!("../src/lua_api/globals/missing_surface.rs");

    assert!(
        !registration.contains("register_pet_battle_fallback_tables")
            && !missing_surface.contains("register_pet_battle_fallback_tables"),
        "dead C_PetBattles fallback registration hooks should stay removed from the C API boundary"
    );
}

#[test]
fn character_services_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");
    let missing_surface = include_str!("../src/lua_api/globals/missing_surface.rs");

    assert!(
        !temporary_shims.contains("c_character_services"),
        "unmodeled C_CharacterServices service/display/assignment defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_character_services"),
        "C_CharacterServices service/display/assignment defaults should not be wired through C API registration"
    );
    assert!(
        !registration.contains("register_character_services_tables")
            && !missing_surface.contains("register_character_services_tables"),
        "dead C_CharacterServices fallback registration hooks should stay removed from the C API boundary"
    );
}

#[test]
fn world_activity_defaults_have_no_empty_c_api_registration_hook() {
    let registration = include_str!("../src/c_api/registration.rs");
    let missing_surface = include_str!("../src/lua_api/globals/missing_surface.rs");

    assert!(
        !registration.contains("register_world_activity_tables")
            && !missing_surface.contains("register_world_activity_tables"),
        "dead world-activity fallback registration hooks should stay removed from the C API boundary"
    );
}

#[test]
fn display_safe_area_defaults_are_permanent_shims() {
    let c_api = include_str!("../src/c_api/mod.rs");
    let permanent_shims = include_str!("../src/c_api/permanent_shims/mod.rs");

    assert!(
        !c_api.contains("pub mod c_ui;"),
        "static C_UI display-safe-area defaults should not live as a root state-backed C API module"
    );
    assert!(
        permanent_shims.contains("pub mod c_ui;"),
        "static C_UI display-safe-area defaults should stay in permanent_shims"
    );
}

#[test]
fn fog_of_war_lookup_surface_is_permanent_shim() {
    let c_api = include_str!("../src/c_api/mod.rs");
    let permanent_shims = include_str!("../src/c_api/permanent_shims/mod.rs");

    assert!(
        !c_api.contains("pub mod c_fog_of_war;"),
        "static C_FogOfWar lookup defaults should not live as a root state-backed C API module"
    );
    assert!(
        permanent_shims.contains("pub mod c_fog_of_war;"),
        "static C_FogOfWar lookup defaults should stay in permanent_shims"
    );
}

#[test]
fn major_faction_display_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let registration = include_str!("../src/c_api/registration.rs");

    assert!(
        !temporary_shims.contains("c_major_faction_display"),
        "unmodeled C_MajorFactions display-policy defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !registration.contains("c_major_faction_display"),
        "C_MajorFactions display-policy defaults should not be wired through C API registration"
    );
}

#[test]
fn reincarnation_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_reincarnation"),
        "unmodeled C_Reincarnation mutable defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_reincarnation"),
        "C_Reincarnation mutable defaults should not be wired through C API registration"
    );
}

#[test]
fn transmog_outfit_slot_defaults_are_not_c_api_temporary_shims() {
    let temporary_shims = c_api_temporary_shims_source();
    let c_api = include_str!("../src/c_api/mod.rs");

    assert!(
        !temporary_shims.contains("c_transmog_outfit_slots"),
        "unmodeled C_TransmogOutfitInfo slot/outfit defaults belong in lua_api::workarounds::temporary"
    );
    assert!(
        !c_api.contains("c_transmog_outfit_slots"),
        "C_TransmogOutfitInfo slot/outfit defaults should not be wired through C API registration"
    );
}
