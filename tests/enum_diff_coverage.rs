use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn parse_enum_names(file_name: &str) -> BTreeSet<String> {
    let path = manifest_dir().join("docs/wow-client-diff").join(file_name);
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn diff_enums_missing_matches_live_runtime_gaps() {
    let env = WowLuaEnv::new().unwrap();
    let missing = parse_enum_names("diff_enums_missing.txt");

    let lua_list = missing
        .iter()
        .map(|name| format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        r#"
        local names = {{ {lua_list} }}
        local missing = {{}}
        for _, name in ipairs(names) do
            if type(Enum[name]) ~= "table" then
                table.insert(missing, name)
            end
        end
        return table.concat(missing, "\n")
        "#
    );

    let runtime_missing = env.eval::<String>(&script).unwrap();
    let runtime_missing: BTreeSet<String> = runtime_missing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    assert_eq!(missing, runtime_missing);
}

#[test]
fn representative_missing_enums_are_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let breakpoint = if cfg!(feature = "client-retail") {
        1
    } else {
        0
    };
    env.exec(&format!(
        "assert(Enum.AbbreviationDataError.InvalidBreakpoint == {breakpoint})"
    ))
    .unwrap();
    let result: String = env
        .eval(
            r#"
            local checks = {
                { "AccountData", "Config", 0 },
                { "AccountStoreItemStatus", "Owned", 3 },
                { "ClientDebugAISpellReadyStatus", "Ready", 0 },
                { "ClientDebugAISpellReadyStatusMeta", "NumValues", 32 },
            }

            for _, check in ipairs(checks) do
                local enumTable = Enum[check[1]]
                if type(enumTable) ~= "table" then
                    return "missing_enum:" .. check[1]
                end

                if enumTable[check[2]] ~= check[3] then
                    return "wrong_value:" .. check[1] .. "." .. check[2] .. "=" .. tostring(enumTable[check[2]])
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn sparse_large_max_value_meta_enums_are_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local checks = {
                { "AccountStateLoadedFlagsMeta", "MaxValue", -2147483648 },
                { "BagFlagMeta", "MaxValue", 134217728 },
                { "BattlePetSpeciesFlagsMeta", "MaxValue", 65536 },
                { "BnetAccountFlagMeta", "MaxValue", 524288 },
            }

            for _, check in ipairs(checks) do
                local enumTable = Enum[check[1]]
                if type(enumTable) ~= "table" then
                    return "missing_enum:" .. check[1]
                end

                if enumTable[check[2]] ~= check[3] then
                    return "wrong_value:" .. check[1] .. "." .. check[2] .. "=" .. tostring(enumTable[check[2]])
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn seconds_formatter_enums_are_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    #[cfg(feature = "native-duration-formatting")]
    env.exec(
        r#"
        local abbreviation = Enum.SecondsFormatterAbbreviation
        assert(abbreviation.None == 0 and abbreviation.Truncate == 1)
        assert(abbreviation.OneLetter == 2)
        assert(abbreviation.TwoLetters == nil and abbreviation.Full == nil)
        local metadata = Enum.SecondsFormatterAbbreviationMeta
        assert(metadata.MinValue == 0 and metadata.MaxValue == 2 and metadata.NumValues == 3)
        "#,
    )
    .unwrap();
    #[cfg(not(feature = "native-duration-formatting"))]
    env.exec(
        r#"
        local abbreviation = Enum.SecondsFormatterAbbreviation
        assert(abbreviation.None == 0 and abbreviation.OneLetter == 1)
        assert(abbreviation.TwoLetters == 2 and abbreviation.Full == 3)
        "#,
    )
    .unwrap();
    let result: String = env
        .eval(
            r#"
            local interval = Enum.SecondsFormatterInterval
            if type(interval) ~= "table" then
                return "missing_interval"
            end
            if interval.Seconds ~= 0 or interval.Minutes ~= 1 or interval.Hours ~= 2 or interval.Days ~= 3 then
                return "wrong_interval"
            end

            local abbreviation = Enum.SecondsFormatterAbbreviation
            if type(abbreviation) ~= "table" then
                return "missing_abbreviation"
            end
            local rounding = Enum.SecondsFormatterRounding
            if type(rounding) ~= "table" then
                return "missing_rounding"
            end
            if rounding.RoundUp ~= 0 or rounding.Truncate ~= 1 then
                return "wrong_rounding"
            end

            local roundingMeta = Enum.SecondsFormatterRoundingMeta
            if type(roundingMeta) ~= "table" then
                return "missing_rounding_meta"
            end
            if roundingMeta.MinValue ~= 0 or roundingMeta.MaxValue ~= 1 or roundingMeta.NumValues ~= 2 then
                return "wrong_rounding_meta"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn script_object_access_restriction_enum_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local restriction = Enum.ScriptObjectAccessRestriction
            local meta = Enum.ScriptObjectAccessRestrictionMeta
            if type(restriction) ~= "table" or type(meta) ~= "table" then return "tables" end
            if restriction.DenyTaintedAccessWhenAurasAreSecret ~= 1 then return "value" end
            if meta.MinValue ~= 1 or meta.MaxValue ~= 1 or meta.NumValues ~= 1 then return "metadata" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn cooldown_layout_enums_are_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local checks = {
                { "CooldownLayoutStatus", "Success", 0 },
                { "CooldownLayoutStatus", "NoValidAlerts", 6 },
                { "CDMLayoutMode", "AccessOnly", false },
                { "CDMLayoutMode", "AllowCreate", true },
                { "CooldownLayoutAction", "ChangeOrder", 0 },
                { "CooldownLayoutAction", "AddAlert", 3 },
                { "CooldownLayoutType", "Character", 1 },
                { "CooldownLayoutType", "Account", 2 },
            }

            for _, check in ipairs(checks) do
                local enumTable = Enum[check[1]]
                if type(enumTable) ~= "table" then
                    return "missing_enum:" .. check[1]
                end

                if enumTable[check[2]] ~= check[3] then
                    return "wrong_value:" .. check[1] .. "." .. check[2] .. "=" .. tostring(enumTable[check[2]])
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn edit_mode_chat_frame_display_only_setting_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local enumTable = Enum.EditModeChatFrameDisplayOnlySetting
            if type(enumTable) ~= "table" then
                return "missing_enum"
            end

            if enumTable.Width ~= 4 then
                return "wrong_width:" .. tostring(enumTable.Width)
            end

            if enumTable.Height ~= 5 then
                return "wrong_height:" .. tostring(enumTable.Height)
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn housing_fixture_decor_action_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local enumTable = Enum.HousingFixtureDecorAction
            if type(enumTable) ~= "table" then
                return "missing_enum"
            end

            if enumTable.Store ~= 0 then
                return "wrong_store:" .. tostring(enumTable.Store)
            end

            if enumTable.Detach ~= 1 then
                return "wrong_detach:" .. tostring(enumTable.Detach)
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn guide_frame_state_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local enumTable = Enum.GuideFrameState
            if type(enumTable) ~= "table" then
                return "missing_enum"
            end

            if enumTable.StartGuiding ~= 1 then
                return "wrong_start:" .. tostring(enumTable.StartGuiding)
            end

            if enumTable.StopGuiding ~= 2 then
                return "wrong_stop:" .. tostring(enumTable.StopGuiding)
            end

            if enumTable.CannotGuide ~= 3 then
                return "wrong_cannot:" .. tostring(enumTable.CannotGuide)
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn warband_scene_animation_event_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local enumTable = Enum.WarbandSceneAnimationEvent
            if type(enumTable) ~= "table" then
                return "missing_enum"
            end

            local checks = {
                { "StartingPose", 0 },
                { "Idle", 1 },
                { "Select", 3 },
                { "EnterWorld", 6 },
                { "Ffx", 9 },
            }

            for _, check in ipairs(checks) do
                if enumTable[check[1]] ~= check[2] then
                    return "wrong_value:" .. check[1] .. "=" .. tostring(enumTable[check[1]])
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn tiered_entrance_reward_type_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local enumTable = Enum.TieredEntranceRewardType
            if type(enumTable) ~= "table" then
                return "missing_enum"
            end

            if enumTable.Item ~= 0 then
                return "wrong_item:" .. tostring(enumTable.Item)
            end

            if enumTable.Currency ~= 1 then
                return "wrong_currency:" .. tostring(enumTable.Currency)
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn low_usage_character_create_and_transmog_enums_are_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local raceMode = Enum.CharacterCreateRaceMode
            if type(raceMode) ~= "table" then
                return "missing_character_create_race_mode"
            end

            if raceMode.Normal ~= 0 then
                return "wrong_character_create_race_mode_normal:" .. tostring(raceMode.Normal)
            end

            if raceMode.Allied ~= 1 then
                return "wrong_character_create_race_mode_allied:" .. tostring(raceMode.Allied)
            end

            local sheatheCategory = Enum.TransmogOutfitSlotOptionSheatheCategory
            if type(sheatheCategory) ~= "table" then
                return "missing_transmog_outfit_slot_option_sheathe_category"
            end

            if sheatheCategory.Default ~= 0 then
                return "wrong_sheathe_category_default:" .. tostring(sheatheCategory.Default)
            end

            if sheatheCategory.Back ~= 1 then
                return "wrong_sheathe_category_back:" .. tostring(sheatheCategory.Back)
            end

            if sheatheCategory.Side ~= 2 then
                return "wrong_sheathe_category_side:" .. tostring(sheatheCategory.Side)
            end

            if sheatheCategory.Hide ~= 3 then
                return "wrong_sheathe_category_hide:" .. tostring(sheatheCategory.Hide)
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn unit_sex_is_available_with_expected_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local enumTable = Enum.UnitSex
            if type(enumTable) ~= "table" then
                return "missing_enum"
            end

            local checks = {
                { "Male", 0 },
                { "Female", 1 },
                { "None", 2 },
                { "Both", 3 },
                { "Neutral", 4 },
            }

            for _, check in ipairs(checks) do
                if enumTable[check[1]] ~= check[2] then
                    return "wrong_value:" .. check[1] .. "=" .. tostring(enumTable[check[1]])
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn diff_enums_extra_is_empty_and_removed_runtime_enums_stay_absent() {
    let extra = parse_enum_names("diff_enums_extra.txt");
    assert!(
        extra.is_empty(),
        "expected no extra enums, found: {extra:?}"
    );

    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local removed = {
                "TransmogOutfitFlags",
            }

            for _, name in ipairs(removed) do
                if type(Enum[name]) == "table" then
                    return "unexpected_enum:" .. name
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}
