#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const ARENA_CASES: &str = r##"
assert(GetNumArenaOpponents() == 0 and GetNumArenaOpponentSpecs() == 0, "empty arena defaults")
assert(GetArenaOpponentSpec(1) == 0, "unknown opponent spec default")
assert(not pcall(A_Admin.SetArenaOpponent, "arena6", true, "bad", 8, 62), "reject invalid arena token")
assert(not pcall(A_Admin.SetArenaOpponent, "arena1", true, "", 8, 62), "reject empty guid")
assert(not pcall(A_Admin.SetArenaOpponent, "arena1", true, "guid", 0, 62), "reject invalid class")
assert(not pcall(A_Admin.SetArenaOpponent, "arena1", false, "guid", nil, nil), "reject absent details")
assert(not pcall(A_Admin.SetArenaCrowdControl, "player", -1), "reject negative spell")

local frame = CompactArenaFrame_Generate()
local member = frame.memberUnitFrames[1]
local cc = member.CcRemoverFrame
assert(cc and cc.Icon and cc.Cooldown, "arena frame has CC controls")

local function set_case(guid, spell, duration)
    A_Admin.SetArenaOpponent("arena1", true, guid, 8, 62)
    assert(UnitExists("arena1") and select(3, UnitClass("arena1")) == 8, "arena unit queries")
    assert(UnitGUID("arena1") == guid and GetNumArenaOpponents() == 1, "arena GUID and count")
    assert(GetNumArenaOpponentSpecs() == 1 and GetArenaOpponentSpec(1) == 62, "arena spec queries")
    A_Admin.SetArenaCrowdControl("arena1", spell, GetTime(), duration)
    assert(cc:IsShown() and cc.Icon:GetTexture() == C_Spell.GetSpellTexture(spell), "CC icon for " .. spell)
    local start, length = cc.Cooldown:GetCooldownTimes()
    assert(math.abs(start / 1000 - GetTime()) < 0.1, "CC cooldown start for " .. spell)
    assert(math.abs(length / 1000 - duration) < 0.1, "CC cooldown length for " .. spell)
end

set_case("Player-0000-00000081", 336126, 120)
set_case("Player-0000-00000082", 42292, 90)
"##;

#[test]
fn arena_frame_renders_supplied_opponent_control_spell_and_cooldown() {
    with_blizzard_addon_closure(
        &[
            "Blizzard_AuraContainer",
            "Blizzard_UnitFrame",
            "Blizzard_CompactRaidFrames",
        ],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitFrame"));
            env.exec(ARENA_CASES).expect(
                "original CompactArenaFrame should render each supplied crowd-control state",
            );
        },
    );
}
