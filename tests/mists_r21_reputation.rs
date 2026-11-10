#![cfg(feature = "client-mists")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

fn supply_rows(env: &WowLuaEnv) {
    env.exec(r#"
        local state = __wow_reputation_state
        state.factions = {
            { name = 'Pandaria', factionID = 0, reaction = 4, isHeader = true },
            { name = 'Golden Lotus', factionID = 1269, reaction = 5, bottom = 0, top = 6000, standing = 1250 },
            { name = 'Shado-Pan', factionID = 1270, reaction = 6, bottom = 0, top = 12000, standing = 3300 },
            { name = 'Alliance', factionID = 0, reaction = 4, isHeader = true },
            { name = 'Stormwind', factionID = 72, reaction = 7, bottom = 0, top = 21000, standing = 9400 },
        }
    "#).unwrap();
}

#[test]
fn mists_r21_reputation_original_frame_tracks_header_visibility() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, _| {
        supply_rows(env);
        env.exec(r#"
            local source = debug.getinfo(ReputationFrame_Update, 'S').source:gsub('\\', '/')
            assert(source:find('Blizzard_UIPanels_Game/Classic/ReputationFrame.lua', 1, true), source)
            ReputationFrame_Update()
            assert(GetNumFactions() == 5, 'supplied full list')
            assert(ReputationBar2FactionName:GetText() == 'Golden Lotus', 'first supplied faction')
            CollapseFactionHeader(1)
            ReputationFrame_Update()
            assert(GetNumFactions() == 3, 'collapsed visible count')
            assert(ReputationBar2FactionName:GetText() == 'Alliance', 'second visible header')
            assert(ReputationBar3FactionName:GetText() == 'Stormwind', 'remaining visible faction')
            assert(not ReputationBar4:IsShown(), 'hidden rows must not remain in the frame')
            assert(GetFactionInfo(4) ~= nil, 'getter accepts backing rows above visible count')
            ExpandFactionHeader(999)
            ReputationFrame_Update()
            assert(GetNumFactions() == 5 and ReputationBar5:IsShown(), 'invalid expand means all')
            CollapseFactionHeader(2)
            ReputationFrame_Update()
            assert(GetNumFactions() == 2, 'non-header collapse means all')
            assert(ReputationBar2FactionName:GetText() == 'Alliance')
            ExpandFactionHeader(0)
            ReputationFrame_Update()
            assert(GetNumFactions() == 5, 'zero expands all')
        "#).unwrap();
    });
}

#[test]
fn mists_r21_reputation_tuple_counts_fallback_and_selection() {
    let env = WowLuaEnv::new().unwrap();
    supply_rows(&env);
    env.exec(
        r#"
        assert(select('#', GetFactionInfo(2)) == 16, 'normal row has 16 results')
        assert(select('#', GetFactionInfo(1)) == 14, 'header has 14 results')
        for _, index in ipairs({0, -1, 6, 999}) do
            assert(select('#', GetFactionInfo(index)) == 13, 'fallback has 13 results')
            local row = {GetFactionInfo(index)}
            assert(row[1] == nil and row[2] == nil and row[3] == 1)
            assert(row[4] == 0 and row[5] == 0 and row[6] == 0)
            for i = 7, 13 do assert(row[i] == nil, 'fallback tail is nil') end
        end
        SetSelectedFaction(5)
        assert(GetSelectedFaction() == 5)
        CollapseFactionHeader(1)
        assert(GetSelectedFaction() == 3, 'selection follows faction identity')
        ExpandAllFactionHeaders()
        assert(GetSelectedFaction() == 5)
        CollapseAllFactionHeaders()
        assert(GetNumFactions() == 2)
        ExpandAllFactionHeaders()
        assert(GetNumFactions() == 5)
        __wow_reputation_state.factions = {}
        assert(GetNumFactions() == 0 and GetSelectedFaction() == 0)
    "#,
    )
    .unwrap();
}
