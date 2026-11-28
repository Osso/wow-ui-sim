//! GetLootThreshold/SetLootThreshold for the group leader,
//! GetOptOutOfLoot/SetOptOutOfLoot, and LoggingChat/LoggingCombat through the
//! original `/chatlog` and `/combatlog` slash commands.
#![cfg(feature = "client-retail")]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_game_ui() -> WowLuaEnv {
    let ui = common::blizzard_ui_dir();
    let env = common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

/// The original consumer is the player unit popup (`UnitPopupMenuSelf`), but
/// that menu cannot open on Retail while `HasLootSpecializations` stays
/// unavailable (docs/specs/loot-specialization.md), so the globals are called
/// directly.
const LOOT_THRESHOLD_CASES: &str = r#"
local changes = 0
EventRegistry:RegisterFrameEventAndCallback("PARTY_LOOT_METHOD_CHANGED", function() changes = changes + 1 end)

assert(GetLootThreshold() == 2, "default Uncommon threshold")
assert(ITEM_QUALITY_COLORS[GetLootThreshold()], "a quality index")
A_Admin.SetPartySize(0)
SetLootThreshold(4)
assert(GetLootThreshold() == 2 and changes == 0, "ungrouped players cannot set the threshold")

A_Admin.SetPartySize(1)
A_Admin.SetPartyMember(1, "Thrynn", 2, 80)
SetLootThreshold(4)
assert(GetLootThreshold() == 4 and changes == 1, "the leader sets Epic")
SetLootThreshold(4)
assert(changes == 1, "an unchanged threshold fires nothing")
SetLootThreshold(6)
assert(GetLootThreshold() == 4, "Artifact is not a group-loot threshold")
A_Admin.SetPartyLeader(1)
SetLootThreshold(3)
assert(GetLootThreshold() == 4 and changes == 1, "only the leader sets the threshold")
A_Admin.SetPartyLeader(0)
SetLootThreshold(3)
assert(GetLootThreshold() == 3 and changes == 2, "the leader again")

assert(GetOptOutOfLoot() == false, "not opted out by default")
SetOptOutOfLoot(1)
assert(GetOptOutOfLoot() == true, "opted out")
SetOptOutOfLoot(nil)
assert(GetOptOutOfLoot() == false, "opted back in")
"#;

#[test]
fn loot_threshold_is_set_by_the_group_leader_and_opt_out_is_stored() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(LOOT_THRESHOLD_CASES).expect("loot threshold cases");
    }
}

const LOGGING_CASES: &str = r#"
local function chat(text)
    ChatFrame1EditBox:SetText(text)
    ChatEdit_SendText(ChatFrame1EditBox, false)
end
local function lastMessage()
    return (DEFAULT_CHAT_FRAME:GetMessageInfo(DEFAULT_CHAT_FRAME:GetNumMessages()))
end
assert(LoggingChat() == false and LoggingCombat() == false, "both logs start off")
chat("/chatlog")
assert(LoggingChat() == true and lastMessage() == CHATLOGENABLED, tostring(lastMessage()))
chat("/chatlog")
assert(LoggingChat() == false and lastMessage() == CHATLOGDISABLED, tostring(lastMessage()))
chat("/combatlog")
assert(LoggingCombat() == true and lastMessage() == COMBATLOGENABLED, tostring(lastMessage()))
assert(LoggingChat() == false, "the chat log is independent")
assert(LoggingCombat(false) == false and LoggingCombat() == false, "a set returns the new state")
"#;

#[test]
fn chatlog_and_combatlog_slash_commands_toggle_session_logging() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(LOGGING_CASES).expect("logging cases");
    }
}
