//! Original BossBanner loot frame consumes item set IDs and names.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn boss_banner_displays_set_names_from_item_records(env: &WowLuaEnv) {
    env.exec(r#"
        local source = debug.getinfo(BossBanner_ConfigureLootFrame, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_FrameXML/BossBannerToast.lua', 1, true), source)
        local frame = CreateFrame('Frame', nil, UIParent, 'BossBannerLootFrameTemplate')
        local function configure(item)
            BossBanner_ConfigureLootFrame(frame, {itemLink = 'item:' .. item, quantity = 1, playerName = 'Tester', className = 'PALADIN'})
            return frame.SetName:IsShown(), frame.SetName:GetText()
        end
        local shown, text = configure(211993)
        assert(shown and text == BOSS_BANNER_LOOT_SET:format("Entombed Seraph's Radiance"), tostring(text))
        shown, text = configure(122245)
        assert(shown and text == BOSS_BANNER_LOOT_SET:format('Regalia of Fabled Adventurers'), tostring(text))
        assert(not configure(6948))
        assert(C_Item.GetItemSetInfo('1455') == 'Regalia of Fabled Adventurers')
        assert(select('#', C_Item.GetItemSetInfo(999999)) == 1 and C_Item.GetItemSetInfo(999999) == nil)
        assert(not pcall(C_Item.GetItemSetInfo))
    "#).unwrap();
}
}
