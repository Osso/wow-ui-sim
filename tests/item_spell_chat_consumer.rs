//! Original retail ChatFrameUtil.InsertLink classifies item links in macros.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn chat_link_inserts_use_for_item_spell_and_equip_for_armor(env: &WowLuaEnv) {
    env.exec(r#"
        local source = debug.getinfo(ChatFrameUtil.InsertLink, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_ChatFrameBase/Mainline/ChatFrameUtilOverrides.lua', 1, true), source)
        local inserted
        MacroFrameText = {
            HasFocus = function() return true end,
            GetCursorPosition = function() return 0 end,
            GetText = function() return '' end,
            Insert = function(_, text) inserted = text end,
        }
        local hearthstone = assert(C_Item.GetItemInfo(6948))
        assert(ChatFrameUtil.InsertLink('item:6948'))
        assert(inserted == SLASH_USE1 .. ' ' .. hearthstone .. '\n', tostring(inserted))
        local armor = assert(C_Item.GetItemInfo(211993))
        assert(ChatFrameUtil.InsertLink('item:211993'))
        assert(inserted == SLASH_EQUIP1 .. ' ' .. armor .. '\n', tostring(inserted))
        MacroFrameText = nil
    "#).unwrap();
}
}
