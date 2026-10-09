//! Startup publication and values for the 12.0.0 TransmogSituation enum.

// Later cumulative epochs publish additional members and different metadata.
#![cfg(all(feature = "retail-12-0-0", not(feature = "retail-12-0-5")))]

use super::super::*;

const TRANSMOG_SITUATION_VALUES: &[(&str, i64)] = &[
    ("AllEquipmentSets", 17),
    ("AllLocations", 2),
    ("AllMovement", 12),
    ("AllRacialForms", 19),
    ("AllSpecs", 0),
    ("EquipmentSets", 18),
    ("FormNative", 20),
    ("FormNonNative", 21),
    ("LocationArenas", 10),
    ("LocationBattlegrounds", 11),
    ("LocationCharacterSelect", 5),
    ("LocationDelves", 7),
    ("LocationDungeons", 8),
    ("LocationHouse", 4),
    ("LocationRaids", 9),
    ("LocationRested", 3),
    ("LocationWorld", 6),
    ("MovementFlyingMount", 16),
    ("MovementGroundMount", 15),
    ("MovementSwimming", 14),
    ("MovementUnmounted", 13),
    ("Spec", 1),
];

#[test]
fn test_patch_12_0_0_transmog_situation_enum_values() {
    let env = WowLuaEnv::new().unwrap();
    let expected_lua = TRANSMOG_SITUATION_VALUES
        .iter()
        .map(|(name, value)| format!("[{name:?}] = {value}"))
        .collect::<Vec<_>>()
        .join(",\n                ");
    let script = format!(
        r#"
            local namespace = Enum.TransmogSituation
            if type(namespace) ~= "table" then
                return "namespace:" .. type(namespace)
            end
            local expected = {{
                {expected_lua}
            }}
            for name, value in pairs(expected) do
                local actual = namespace[name]
                if type(actual) ~= "number" then
                    return name .. ":type=" .. type(actual)
                end
                if actual ~= value then
                    return name .. ":value=" .. tostring(actual)
                end
            end

            local metadata = Enum.TransmogSituationMeta
            local expected_metadata = {{
                MaxValue = 21,
                MinValue = 0,
                NumValues = 22,
            }}
            for name, value in pairs(expected_metadata) do
                local actual = metadata[name]
                if type(actual) ~= "number" or actual ~= value then
                    return "metadata." .. name .. "=" .. tostring(actual)
                end
            end
            return "ok"
        "#,
        expected_lua = expected_lua,
    );
    let result: String = env.eval(&script).unwrap();
    assert_eq!(
        result, "ok",
        "TransmogSituation did not match the 12.0.0 source register"
    );
}
