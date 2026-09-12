//! Startup publication and values for small 12.0.0 enum families.

#![cfg(feature = "retail-12-0-0")]

use super::super::*;

#[cfg(not(feature = "retail-12-0-5"))]
const ENUM_VALUES: &[(&str, &[(&str, i64)])] = &[
    (
        "AccountTransType",
        &[
            ("HouseInitiativeFavor", 66),
            ("TransmogOutfitCollection", 65),
        ],
    ),
    (
        "AccountTransTypeMeta",
        &[("MaxValue", 66), ("NumValues", 67)],
    ),
    (
        "CraftingReagentItemFlag",
        &[("TooltipShowsAsStatModifications", 1)],
    ),
    (
        "CraftingReagentItemFlagMeta",
        &[("MaxValue", 1), ("MinValue", 1)],
    ),
    ("CurrencyDestroyReason", &[("CraftingOrderReagent", 16)]),
    (
        "CurrencyDestroyReasonMeta",
        &[("MaxValue", 16), ("NumValues", 17)],
    ),
    ("CurrencySource", &[("InitiativeReward", 67)]),
    ("CurrencySourceMeta", &[("MaxValue", 67), ("NumValues", 68)]),
    (
        "EditModeAuraFrameSystemIndices",
        &[("ExternalDefensivesFrame", 3)],
    ),
    (
        "EditModeAuraFrameSystemIndicesMeta",
        &[("MaxValue", 3), ("NumValues", 3)],
    ),
    ("EditModeCooldownViewerSetting", &[("BarWidthScale", 11)]),
    (
        "EditModeCooldownViewerSettingMeta",
        &[("MaxValue", 11), ("NumValues", 12)],
    ),
    (
        "GameRule",
        &[
            ("EjJourneysDisabled", 156),
            ("PvPInitialRatingOverride", 190),
        ],
    ),
    ("GameRuleMeta", &[("MaxValue", 190), ("NumValues", 140)]),
    ("HousingDecorActionFlags", &[("PreviewDecor", 2_048)]),
    (
        "HousingDecorActionFlagsMeta",
        &[("MaxValue", 2_048), ("NumValues", 13)],
    ),
    ("HousingItemToastType", &[("House", 4)]),
    (
        "HousingItemToastTypeMeta",
        &[("MaxValue", 4), ("NumValues", 5)],
    ),
    ("MapIconUIWidgetSetType", &[("AdventureMapDetails", 2)]),
    (
        "MapIconUIWidgetSetTypeMeta",
        &[("MaxValue", 2), ("NumValues", 3)],
    ),
    ("SurveyDeliveryMoment", &[("MythicPlusCompleted", 4)]),
    (
        "SurveyDeliveryMomentMeta",
        &[("MaxValue", 4), ("NumValues", 5)],
    ),
    ("TooltipDataType", &[("Outfit", 27)]),
    (
        "TooltipDataTypeMeta",
        &[("MaxValue", 27), ("NumValues", 28)],
    ),
    ("TraitNodeFlag", &[("ShowTierTrack", 256)]),
    ("TraitNodeFlagMeta", &[("MaxValue", 256), ("NumValues", 9)]),
    ("UICursorType", &[("Outfit", 21)]),
    ("UICursorTypeMeta", &[("MaxValue", 21), ("NumValues", 22)]),
    ("UIWidgetVisualizationType", &[("PreyHuntProgress", 31)]),
    (
        "UIWidgetVisualizationTypeMeta",
        &[("MaxValue", 31), ("NumValues", 32)],
    ),
];

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn test_patch_12_0_0_enum_current_keys_and_old_keys_absent() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
                local expected = {
                    {"ItemRecraftFlags", "Invalid", 1, "ItemRecraftFlagInvalid"},
                    {"PerksVendorCategoryType", "RefundUnused", 24, "UnusedPerksVendorCategoryRefundUnused"},
                    {"PlayerInteractionType", "TieredEntrance", 79, "PlaceholderType79"},
                    {"GossipNpcOption", "TieredEntrance", 66, "Placeholder_6"},
                    {"QuestTagType", "Prey", 19, "Placeholder_1"},
                }
                for _, check in ipairs(expected) do
                    local namespace_name = check[1]
                    local namespace = Enum[namespace_name]
                    if type(namespace) ~= "table" then
                        return namespace_name .. ":namespace=" .. type(namespace)
                    end
                    local current_value = namespace[check[2]]
                    if type(current_value) ~= "number" then
                        return namespace_name .. "." .. check[2] .. ":type=" .. type(current_value)
                    end
                    if current_value ~= check[3] then
                        return namespace_name .. "." .. check[2] .. ":value=" .. tostring(current_value)
                    end
                    if rawget(namespace, check[4]) ~= nil then
                        return namespace_name .. "." .. check[4] .. ":present"
                    end
                end
                return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "12.0.0 enum publications did not match the expected current keys"
    );
}

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn test_patch_12_0_0_small_enum_values() {
    let env = WowLuaEnv::new().unwrap();
    let expected_namespaces = ENUM_VALUES
        .iter()
        .map(|(namespace, values)| {
            let expected_values = values
                .iter()
                .map(|(name, value)| format!("[{name:?}] = {value}"))
                .collect::<Vec<_>>()
                .join(",\n                    ");
            format!(
                "[{namespace:?}] = {{\n                    {expected_values}\n                }}"
            )
        })
        .collect::<Vec<_>>()
        .join(",\n                ");
    let script = format!(
        r#"
            local expected = {{
                {expected_namespaces}
            }}
            for namespace_name, expected_values in pairs(expected) do
                local namespace = Enum[namespace_name]
                if type(namespace) ~= "table" then
                    return namespace_name .. ":namespace=" .. type(namespace)
                end
                for name, value in pairs(expected_values) do
                    local actual = namespace[name]
                    if type(actual) ~= "number" then
                        return namespace_name .. "." .. name .. ":type=" .. type(actual)
                    end
                    if actual ~= value then
                        return namespace_name .. "." .. name .. ":value=" .. tostring(actual)
                    end
                end
            end
            return "ok"
        "#,
        expected_namespaces = expected_namespaces,
    );
    let result: String = env.eval(&script).unwrap();
    assert_eq!(
        result, "ok",
        "small 12.0.0 enum namespaces did not match the source register"
    );
}

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn test_patch_12_0_0_encounter_event_flags_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
                local flags = Enum.EncounterEventFlags
                if type(flags) ~= "table" then
                    return "flags:namespace=" .. type(flags)
                end
                if type(flags.Disabled) ~= "number" or flags.Disabled ~= 1 then
                    return "flags.Disabled=" .. tostring(flags.Disabled)
                end
                if rawget(flags, "IgnoreCastConsume") ~= nil then
                    return "flags.IgnoreCastConsume=" .. tostring(flags.IgnoreCastConsume)
                end

                local metadata = Enum.EncounterEventFlagsMeta
                if type(metadata) ~= "table" then
                    return "metadata:namespace=" .. type(metadata)
                end
                local expected = {
                    MaxValue = 1,
                    MinValue = 1,
                    NumValues = 1,
                }
                for name, value in pairs(expected) do
                    local actual = metadata[name]
                    if type(actual) ~= "number" or actual ~= value then
                        return "metadata." .. name .. "=" .. tostring(actual)
                    end
                end
                return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "EncounterEventFlags did not match the retail 12.0.0 epoch"
    );
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn test_later_retail_encounter_event_flags_values() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
                local flags = Enum.EncounterEventFlags
                local metadata = Enum.EncounterEventFlagsMeta
                if type(flags.IgnoreCastConsume) ~= "number" or flags.IgnoreCastConsume ~= 2 then
                    return "flags.IgnoreCastConsume=" .. tostring(flags.IgnoreCastConsume)
                end
                if type(metadata.MaxValue) ~= "number" or metadata.MaxValue ~= 2 then
                    return "metadata.MaxValue=" .. tostring(metadata.MaxValue)
                end
                if type(metadata.NumValues) ~= "number" or metadata.NumValues ~= 2 then
                    return "metadata.NumValues=" .. tostring(metadata.NumValues)
                end
                return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "later retail EncounterEventFlags values were changed by the 12.0.0 override"
    );
}
