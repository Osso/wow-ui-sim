#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

const MODELED_STATE_GLOBALS_LUA: &str = r#"
local baseFirst = GetBaseDifficultyID(233)
local baseSecond = GetBaseDifficultyID(250)
if baseFirst ~= 16 or baseSecond ~= 17 then return "base-difficulty" end
A_Admin.SetInstanceInfo("First", "party", 233, 5)
local dungeonFirst = GetDungeonDifficultyID()
A_Admin.SetInstanceInfo("Second", "party", 250, 5)
local dungeonSecond = GetDungeonDifficultyID()
if dungeonFirst ~= 233 or dungeonSecond ~= 250 then return "dungeon-difficulty" end
A_Admin.SetGameRule(93, 1)
local roleButtonFirst = CanShowSetRoleButton()
A_Admin.SetGameRule(93, 0)
local roleButtonSecond = CanShowSetRoleButton()
if roleButtonFirst ~= true or roleButtonSecond ~= false then return "role-button" end
A_Admin.SetPlayerRace(0)
A_Admin.SetGuildFactionGroupFlags(1, 4)
local factionFirst = GetGuildFactionGroup()
A_Admin.SetPlayerRace(1)
A_Admin.SetGuildFactionGroupFlags(2, 2)
local factionSecond = GetGuildFactionGroup()
if factionFirst ~= 0 or factionSecond ~= 1 then return "guild-faction" end
A_Admin.QueueSpellConfirmationPrompt({spellID=401, confirmType=1, text="First", duration=3,
    currencyID=0, currencyCost=0, difficultyID=0, displayItemID=0, itemContext=0,
    treasureContextLevel=0})
A_Admin.QueueSpellConfirmationPrompt({spellID=402, confirmType=2, text="Second", duration=4,
    currencyID=7, currencyCost=8, difficultyID=9, displayItemID=10, itemContext=11,
    treasureContextLevel=12})
local prompts = GetSpellConfirmationPromptsInfo()
local promptText = {}
for _, prompt in ipairs(prompts) do promptText[prompt.spellID] = prompt.text end
if #prompts ~= 2 or promptText[401] ~= "First" or promptText[402] ~= "Second" then
    return "spell-prompts"
end
return "ok"
"#;

const EVENT_CALLBACKS_LUA: &str = r#"
local observed = { first = 0, second = 0, unit = 0 }
local function firstCallback(_, _, value) observed.first = value end
local function secondCallback(_, _, value) observed.second = value end
local function unitCallback(_, _, value) observed.unit = value end
local event = "UNIT_HEALTH"
RegisterEventCallback(event, firstCallback)
RegisterEventCallback(event, secondCallback)
RegisterUnitEventCallback(event, unitCallback, "player")
A_Admin.FireEvent(event, "player", 31)
local initial = observed.first == 31 and observed.second == 31 and observed.unit == 31
UnregisterEventCallback(event, firstCallback)
UnregisterUnitEventCallback(event, unitCallback, "player")
A_Admin.FireEvent(event, "player", 47)
return initial and observed.first == 31 and observed.second == 47 and observed.unit == 31
"#;

fn addon_env(root: &str) -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &[root], &[]);
    assert!(loaded.iter().any(|name| name == root));
    env
}

fn chat_type_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let constants = ui.join("Blizzard_ChatFrameBase/Shared/ChatTypeInfoConstants.lua");
    let overrides = ui.join("Blizzard_ChatFrameBase/Mainline/ChatTypeInfoConstantsOverride.lua");
    let colors = ui.join("Blizzard_ChatFrameBase/Shared/ChatTypeInfoColors.lua");
    for path in [constants, overrides, colors] {
        let source = std::fs::read_to_string(path).unwrap();
        env.exec(&source).unwrap();
    }
    env
}

#[test]
fn specialization_consumer_resolves_two_catalog_rows() {
    let env = addon_env("Blizzard_FrameXMLUtil");
    let lines: (String, String) = env
        .eval(
            "return CommunitiesUtil.GetRoleSpecClassLine(2, 65), \
                CommunitiesUtil.GetRoleSpecClassLine(2, 66)",
        )
        .unwrap();
    assert!(lines.0.contains("Holy"), "{}", lines.0);
    assert!(lines.1.contains("Protection"), "{}", lines.1);
    assert_ne!(lines.0, lines.1);
}

#[test]
fn chat_color_consumer_binds_two_distinct_type_indices() {
    let env = chat_type_env();
    let indices: (i32, i32, i32, i32, i32) = env
        .eval("return ChatTypeInfo.SAY.id, ChatTypeInfo.PARTY.id, ChatTypeInfo.GUILD.id, ChatTypeInfo.GUILD_DISCORD.id, ChatTypeInfo.PING.id")
        .unwrap();
    assert_eq!((indices.0, indices.1), (2, 3));
    assert_ne!(indices.0, indices.1);
    assert_ne!(indices.2, indices.3);
    assert!(indices.4 > 0);
}

#[test]
fn global_text_lookup_uses_two_loaded_strings() {
    let env = addon_env("Blizzard_FrameXML");
    let values: (String, String, String, String) = env
        .eval("return GetText('READY'), READY, GetText('NOT_READY'), NOT_READY")
        .unwrap();
    assert_eq!(values.0, values.1);
    assert_eq!(values.2, values.3);
    assert_ne!(values.0, values.2);
}

#[test]
fn state_backed_legacy_globals_follow_distinct_inputs() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env.eval(MODELED_STATE_GLOBALS_LUA).unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn global_event_callbacks_register_remove_and_dispatch() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env.eval(EVENT_CALLBACKS_LUA).unwrap();
    assert!(result);
}

#[test]
fn channel_display_queries_follow_two_joined_channels() {
    let env = WowLuaEnv::new().unwrap();
    let channels: (i32, String, String) = env
        .eval(
            r#"
            JoinChannelByName("Trade")
            JoinChannelByName("Guild")
            return GetNumDisplayChannels(), (GetChannelDisplayInfo(1)),
                (GetChannelDisplayInfo(2))
            "#,
        )
        .unwrap();
    assert_eq!(channels, (2, "Trade".to_string(), "Guild".to_string()));
}
