use super::{load_full_game_ui_with_all_lod, missing_runtime_functions, present_runtime_symbols};

const EXPECTED_PTR_SOURCE_PUBLICATIONS: &[&str] = &[
    "CompactUnitFrameLayoutTemplates_LayoutFrameElement",
    "CompactUnitFrameUtil.ApplyConfig",
    "GameRulesUtil.IsPlayerAtEffectiveMaxLevel",
    "ManageFramePositions",
    "RaidWarningUtil.AddMessage",
    "RaidWarningUtil.ClearBossEmotes",
    "TextureUtil.AnimateTexCoords",
];

const ABSENT_NATIVE_MEMBERS: &[&str] = &[
    "C_Ping.GetContextualPingTypeForUnit",
    "C_RecruitAFriend.IsEnabled",
    "C_UnitAuras.TriggerPrivateAuraShowDispelType",
];

/// Global aliases published by the selected PTR deprecated RaidWarning addon.
const PTR_DEPRECATED_GLOBALS: &[&str] = &[
    "RaidNotice_AddMessage",
    "RaidNotice_Clear",
    "RaidNotice_FadeInit",
    "RaidNotice_UpdateSlot",
];

/// Verifies every final 12.1 row against its full-load PTR publication type.
#[test]
fn remaining_symbols_match_observed_publications() {
    let env = load_full_game_ui_with_all_lod();
    let fallbacks_enabled: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("deprecation fallback CVar query succeeds");
    assert!(
        fallbacks_enabled,
        "PTR source aliases require deprecation fallbacks"
    );

    let compatibility_mismatches: String = env
        .eval(
            r#"
            local function resolve(path)
                local value = _G
                for part in string.gmatch(path, "[^.]+") do
                    value = value and value[part]
                end
                return value
            end

            local function check_callable(path, ...)
                local value = resolve(path)
                if type(value) ~= "function" then
                    return path .. " expected=function observed=" .. type(value)
                end
                local ok, errorMessage = pcall(value, ...)
                if not ok then
                    return path .. " call failed: " .. tostring(errorMessage)
                end
            end

            local mismatches = {}
            for _, probe in ipairs({
                { "GetInventorySlotInfo", "MainHandSlot" },
                { "C_CVar.GetCVar", "UnitNameFocused" },
                { "C_Housing.IsInsideOwnHouse" },
                { "C_SuperTrack.GetNextWaypointForMap", 1 },
            }) do
                local path = probe[1]
                local errorMessage = check_callable(path, unpack(probe, 2))
                if errorMessage then
                    table.insert(mismatches, errorMessage)
                end
            end

            return table.concat(mismatches, ",")
            "#,
        )
        .expect("pre-removal compatibility observation succeeds");
    assert_eq!(
        compatibility_mismatches, "",
        "pinned Blizzard sources require compatibility symbols before strict removal"
    );

    let missing_source_publications =
        missing_runtime_functions(&env, EXPECTED_PTR_SOURCE_PUBLICATIONS);
    assert_eq!(
        missing_source_publications, "",
        "PTR source-declared publications must remain callable"
    );

    let missing_deprecated_globals = missing_runtime_functions(&env, PTR_DEPRECATED_GLOBALS);
    assert_eq!(
        missing_deprecated_globals, "",
        "the selected PTR deprecated RaidWarning aliases remain callable"
    );

    let present_native_members = present_runtime_symbols(&env, ABSENT_NATIVE_MEMBERS);
    assert_eq!(
        present_native_members, "",
        "native-absent members must stay absent"
    );
}
