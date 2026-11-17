use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const ENUM_ASSERTION: &str = r#"
local values = Enum.CombatAudioAlertSpecSetting
if type(values) ~= "table" then
    if Enum.CombatAudioAlertSpecSettingMeta ~= nil then return "orphan metadata" end
    return "absent"
end
local expected = {
    { "Resource1Percent", 0 }, { "Resource1Format", 1 },
    { "Resource1Voice", 2 }, { "Resource1Volume", 3 },
    { "Resource2Percent", 4 }, { "Resource2Format", 5 },
    { "Resource2Voice", 6 }, { "Resource2Volume", 7 },
    { "SayIfTargeted", 8 },
}
for _, member in ipairs(expected) do
    if values[member[1]] ~= member[2] then
        return member[1] .. ": expected " .. member[2] .. ", got " .. tostring(values[member[1]])
    end
end
local count = 0
for _ in pairs(values) do count = count + 1 end
local meta = Enum.CombatAudioAlertSpecSettingMeta
if count ~= 9 or type(meta) ~= "table" or meta.NumValues ~= 9
    or meta.MinValue ~= 0 or meta.MaxValue ~= 8 then
    return "metadata or member count mismatch"
end
return "ok"
"#;

#[test]
fn combat_audio_enum_publication_matches_profile_evidence() {
    let env = WowLuaEnv::new().expect("API enum environment should initialize");
    let actual: String = env.eval(ENUM_ASSERTION).expect("enum should be readable");
    let expected = if ACTIVE == ClientProfile::Wrath {
        "absent"
    } else {
        "ok"
    };
    assert_eq!(actual, expected);
}
