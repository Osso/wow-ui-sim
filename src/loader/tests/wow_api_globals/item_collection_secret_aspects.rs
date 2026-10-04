//! Startup publication and values for the 12.0.0 collection and secret-aspect enums.

#![cfg(feature = "retail-12-0-0")]

use super::super::*;

const ITEM_COLLECTION_TYPE_VALUES: &[(&str, i64)] = &[
    ("ExteriorFixture", 9),
    ("Heirloom", 2),
    ("HouseType", 13),
    ("None", 0),
    ("Room", 8),
    ("RoomMaterial", 11),
    ("RoomTheme", 10),
    ("RuneforgeLegendaryAbility", 5),
    ("Toy", 1),
    ("Transmog", 3),
    ("TransmogIllusion", 6),
    ("TransmogOutfit", 12),
    ("TransmogSetFavorite", 4),
    ("WarbandScene", 7),
];

const ITEM_COLLECTION_TYPE_META_VALUES: &[(&str, i64)] = &[("MaxValue", 13), ("NumValues", 14)];

#[cfg(not(feature = "retail-12-0-5"))]
const SECRET_ASPECT_META_12_0_0_VALUES: &[(&str, i64)] =
    &[("MaxValue", 262_144), ("MinValue", 1), ("NumValues", 24)];

const SECRET_ASPECT_VALUES: &[(&str, i64)] = &[
    ("Alpha", 128),
    ("BarValue", 16384),
    ("Cooldown", 32768),
    ("Cursor", 1024),
    ("Desaturation", 4096),
    ("FrameLevel", 256),
    ("Hierarchy", 1),
    ("ID", 2),
    ("MinimumWidth", 131072),
    ("ObjectDebug", 1),
    ("ObjectName", 1),
    ("ObjectSecrets", 1),
    ("ObjectSecurity", 1),
    ("ObjectType", 1),
    ("Padding", 262144),
    ("Rotation", 65536),
    ("Scale", 64),
    ("ScrollRange", 512),
    ("SecureText", 16),
    ("Shown", 32),
    ("TexCoords", 8192),
    ("Text", 8),
    ("Toplevel", 4),
    ("VertexColor", 2048),
];

fn assert_numeric_namespace(env: &WowLuaEnv, namespace_name: &str, expected: &[(&str, i64)]) {
    let expected_lua = expected
        .iter()
        .map(|(name, value)| format!("[{name:?}] = {value}"))
        .collect::<Vec<_>>()
        .join(",\n                ");
    let script = format!(
        r#"
            local namespace = {namespace_name}
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
            return "ok"
        "#,
        namespace_name = namespace_name,
        expected_lua = expected_lua,
    );
    let result: String = env.eval(&script).unwrap();
    assert_eq!(
        result, "ok",
        "{namespace_name} did not match the 12.0.0 source register"
    );
}

#[test]
fn test_patch_12_0_0_item_collection_and_secret_aspect_values() {
    let env = WowLuaEnv::new().unwrap();
    assert_numeric_namespace(&env, "Enum.ItemCollectionType", ITEM_COLLECTION_TYPE_VALUES);
    assert_numeric_namespace(
        &env,
        "Enum.ItemCollectionTypeMeta",
        ITEM_COLLECTION_TYPE_META_VALUES,
    );
    assert_numeric_namespace(&env, "Enum.SecretAspect", SECRET_ASPECT_VALUES);
}

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn test_patch_12_0_0_secret_aspect_epoch_values() {
    let env = WowLuaEnv::new().unwrap();
    assert_numeric_namespace(
        &env,
        "Enum.SecretAspectMeta",
        SECRET_ASPECT_META_12_0_0_VALUES,
    );
    let result: String = env
        .eval(
            r#"
                local count = 0
                for _ in pairs(Enum.SecretAspect) do
                    count = count + 1
                end
                if count ~= 24 then
                    return "count=" .. tostring(count)
                end
                if rawget(Enum.SecretAspect, "Attributes") ~= nil then
                    return "Attributes=" .. tostring(Enum.SecretAspect.Attributes)
                end
                if rawget(Enum.SecretAspect, "CooldownStyle") ~= nil then
                    return "CooldownStyle=" .. tostring(Enum.SecretAspect.CooldownStyle)
                end
                return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "SecretAspect did not match the 12.0.0 epoch");
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn test_later_retail_secret_aspect_values() {
    let env = WowLuaEnv::new().unwrap();
    // 12.1.0 adds TooltipTexture..RadialProgress (SecretAspectConstantsDocumentation.lua:6–44).
    let (max_value, num_values) = if cfg!(feature = "retail-12-1-0") {
        (8_388_608, 30)
    } else {
        (524_288, 26)
    };
    let result: String = env
        .eval(&format!(
            r#"
                local aspects = Enum.SecretAspect
                local metadata = Enum.SecretAspectMeta
                if aspects.Attributes ~= 1 then
                    return "Attributes=" .. tostring(aspects.Attributes)
                end
                if aspects.CooldownStyle ~= 524288 then
                    return "CooldownStyle=" .. tostring(aspects.CooldownStyle)
                end
                if metadata.MaxValue ~= {max_value} or metadata.MinValue ~= 1 or metadata.NumValues ~= {num_values} then
                    return "metadata=" .. tostring(metadata.MaxValue) .. "/" .. tostring(metadata.MinValue) .. "/" .. tostring(metadata.NumValues)
                end
                return "ok"
            "#,
        ))
        .unwrap();
    assert_eq!(result, "ok", "later retail SecretAspect values changed");
}
