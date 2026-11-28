//! FollowUnit, Dismount, InitiateRolePoll and BeginTrade driven through their
//! original slash commands, popups and status frames after full game startup.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

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

const PLAYER_REQUEST_CASES: &str = r#"
local function chat(text)
    ChatFrame1EditBox:SetText(text)
    ChatEdit_SendText(ChatFrame1EditBox, false)
end
local function status() return AutoFollowStatusText:GetText() end
A_Admin.SetTarget("Hogger", 11, 1, true)
chat("/follow")
assert(AutoFollowStatus:IsShown() and status() == AUTOFOLLOWSTART:format("Hogger"), tostring(status()))
A_Admin.StopAutoFollow()
assert(status() == AUTOFOLLOWSTOP:format("Hogger"), "AUTOFOLLOW_END")
A_Admin.SetPartySize(2)
A_Admin.SetPartyMember(1, "Thrynn", 2, 80)
A_Admin.SetPartyMember(2, "Kazzara", 1, 80)
FollowUnit("party2")
assert(status() == AUTOFOLLOWSTART:format("Kazzara"), tostring(status()))
FollowUnit("thrynn")
assert(status() == AUTOFOLLOWSTART:format("Thrynn"), "name match")
A_Admin.StopAutoFollow()
FollowUnit("Nobody")
assert(status() == AUTOFOLLOWSTOP:format("Thrynn"), "an unknown name starts nothing")

local displayChanges = 0
EventRegistry:RegisterFrameEventAndCallback("PLAYER_MOUNT_DISPLAY_CHANGED", function()
    displayChanges = displayChanges + 1
end)
-- /dismount calls Dismount only when SecureCmdOptionParse("") is truthy; the
-- simulator parser returns nil for empty text (R-ag10-12), so call it directly.
A_Admin.SetMounted(true)
Dismount()
assert(not IsMounted() and displayChanges == 1, "dismounts " .. displayChanges)
Dismount()
assert(displayChanges == 1, "already on foot")

A_Admin.SetPartyLeader(1)
assert(InitiateRolePoll() == false and not RolePollPopup:IsShown(), "not the leader")
A_Admin.SetPartyLeader(0)
assert(InitiateRolePoll() == true and RolePollPopup:IsShown(), "ROLE_POLL_BEGIN opens the poll")
StaticPopupSpecial_Hide(RolePollPopup)
A_Admin.SetPartySize(0)
assert(InitiateRolePoll() == false, "solo players cannot poll")

assert(not pcall(A_Admin.RequestTrade, ""), "requester name")
A_Admin.RequestTrade("Thrall")
local dialog = StaticPopup_FindVisible("TRADE")
assert(dialog and dialog:GetTextFontString():GetText():find("Thrall", 1, true), "TRADE popup")
dialog.ButtonContainer.Button1:Click()
assert(TradeFrame:IsShown(), "BeginTrade opens the trade window")
HideUIPanel(TradeFrame)
A_Admin.RequestTrade("Jaina")
StaticPopup_FindVisible("TRADE").ButtonContainer.Button2:Click()
assert(not TradeFrame:IsShown(), "declined")
BeginTrade()
assert(not TradeFrame:IsShown(), "a declined request cannot be begun")
"#;

#[test]
fn player_requests_drive_follow_mount_role_poll_and_trade_consumers() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(PLAYER_REQUEST_CASES).expect("original player-request consumers");
    }
}

