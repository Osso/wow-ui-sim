#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const ARENA_CASES: &str = r##"
local frame = CompactArenaFrame_Generate()
local member = frame.memberUnitFrames[1]
local cc = member.CcRemoverFrame
assert(cc and cc.Icon and cc.Cooldown)

local function set_case(guid, spell, duration)
    A_Admin.SetArenaOpponent("arena1", guid, 8, 62)
    A_Admin.SetArenaCrowdControl("arena1", spell, GetTime(), duration)
    assert(cc.Icon:GetTexture() == C_Spell.GetSpellTexture(spell))
    local start, length = cc.Cooldown:GetCooldownTimes()
    assert(math.abs(start / 1000 - GetTime()) < 0.1)
    assert(math.abs(length / 1000 - duration) < 0.1)
end

set_case("Player-0000-00000081", 336126, 120)
set_case("Player-0000-00000082", 42292, 90)
"##;

#[test]
fn arena_frame_renders_supplied_opponent_control_spell_and_cooldown() {
    with_blizzard_addon_closure(
        &["Blizzard_UnitFrame", "Blizzard_CompactRaidFrames"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|addon| addon == "Blizzard_UnitFrame"));
            env.exec(ARENA_CASES).expect(
                "original CompactArenaFrame should render each supplied crowd-control state",
            );
        },
    );
}
