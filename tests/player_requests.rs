//! InitiateRolePoll and BeginTrade driven through their original popups
//! after full game startup.
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

const ROLE_POLL_CASES: &str = r#"
A_Admin.SetPartySize(2)
A_Admin.SetPartyMember(1, "Thrynn", 2, 80)
A_Admin.SetPartyMember(2, "Kazzara", 1, 80)
A_Admin.SetPartyLeader(1)
assert(InitiateRolePoll() == false and not RolePollPopup:IsShown(), "not the leader")
A_Admin.SetPartyLeader(0)
assert(InitiateRolePoll() == true and RolePollPopup:IsShown(), "ROLE_POLL_BEGIN opens the poll")
StaticPopupSpecial_Hide(RolePollPopup)
A_Admin.SetPartySize(0)
assert(InitiateRolePoll() == false, "solo players cannot poll")

"#;

#[test]
fn role_poll_opens_original_popup_for_group_leader() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(ROLE_POLL_CASES).expect("role poll consumer");
    }
}

const TRADE_CASES: &str = r#"
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
fn trade_request_drives_original_popup() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(TRADE_CASES).expect("trade request consumer");
    }
}

/// Retail has no ranged slot: main-hand bows, guns, crossbows and wands are
/// ranged. PaperDollFrame_SetAttackPower picks its label from IsRangedWeapon.
#[cfg(feature = "client-retail")]
const WEAPON_CASES: &str = r#"
local statFrame = CharacterStatsPane.statsFramePool:Acquire()
local function attackPowerTag()
    PAPERDOLL_STATINFO.ATTACK_AP.updateFunc(statFrame, "player")
    return statFrame.tooltip
end
local function hasTag(tag) return attackPowerTag():find(tag, 1, true) ~= nil end
A_Admin.UnequipItem(16)
A_Admin.UnequipItem(17)
assert(hasTag(MELEE_ATTACK_POWER) and not IsDualWielding(), "unarmed")
A_Admin.EquipItem(16, 273784) -- Ancestral Amani Recurve (bow)
assert(hasTag(RANGED_ATTACK_POWER), attackPowerTag())
assert(not IsDualWielding(), "a bow alone")
A_Admin.EquipItem(16, 212405) -- one-hand sword
assert(hasTag(MELEE_ATTACK_POWER), attackPowerTag())
A_Admin.EquipItem(17, 212405)
assert(IsDualWielding(), "two one-hand weapons")
A_Admin.UnequipItem(17)
assert(not IsDualWielding(), "empty off hand")
CharacterStatsPane.statsFramePool:Release(statFrame)
"#;

#[cfg(feature = "client-retail")]
#[test]
fn equipped_weapon_queries_drive_paper_doll_attack_power_label() {
    test_timeout! {
        let env = load_game_ui();
        env.exec(WEAPON_CASES).expect("equipped weapon queries");
    }
}
