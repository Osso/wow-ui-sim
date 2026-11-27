//! PvpInfoDocumentation.lua `GetArenaCrowdControlInfo` / `RequestCrowdControlSpell`
//! (Retail 12.1.5 :1194, Forever 1.60.1 :169) over the supplied arena
//! crowd-control spell, observed through the documented synchronous
//! `ARENA_CROWD_CONTROL_SPELL_UPDATE` payload.
#![cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const CROWD_CONTROL: &str = r##"
    local seen = {}
    RegisterEventCallback("ARENA_CROWD_CONTROL_SPELL_UPDATE", function(_, unit, spellID)
        seen[#seen + 1] = unit .. ":" .. spellID
    end)
    assert(select("#", C_PvP.GetArenaCrowdControlInfo("player")) == 0)
    C_PvP.RequestCrowdControlSpell("player")
    assert(#seen == 0)
    local now = GetTime()
    for _, case in ipairs({ { 336126, 120 }, { 42292, 90 } }) do
        A_Admin.SetArenaCrowdControl("player", case[1], now, case[2])
        local spellID, startMs, durationMs = C_PvP.GetArenaCrowdControlInfo("player")
        assert(spellID == case[1] and startMs == now * 1000 and durationMs == case[2] * 1000)
        C_PvP.RequestCrowdControlSpell("player")
        assert(seen[#seen] == "player:" .. case[1], seen[#seen])
    end
    A_Admin.SetArenaCrowdControl("player", 42292)
    assert(select(3, C_PvP.GetArenaCrowdControlInfo("player")) == 0)
    A_Admin.SetArenaCrowdControl("player", 0)
    C_PvP.RequestCrowdControlSpell("player")
    assert(#seen == 2 and select("#", C_PvP.GetArenaCrowdControlInfo("player")) == 0)
"##;

#[test]
fn arena_crowd_control_spell_follows_the_supplied_spell() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(CROWD_CONTROL).expect("crowd-control spell follows the input");
}
