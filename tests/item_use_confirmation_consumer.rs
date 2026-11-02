//! Original StaticPopup callbacks for pending item-use confirmation.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn item_confirmation_popups_clear_the_pending_identity(env: &WowLuaEnv) {
    env.state().borrow_mut().pending_item_use_identity = Some([11, 22]);
    env.exec(r#"
        local callback = assert(StaticPopupDialogs.CONFIM_BEFORE_USE.OnAccept)
        local source = debug.getinfo(callback, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_StaticPopup_Game/GameDialogDefs.lua', 1, true), source)
        callback(nil, nil)
    "#).unwrap();
    assert!(env.state().borrow().pending_item_use_identity.is_none());
    env.state().borrow_mut().pending_item_use_identity = Some([33, 44]);
    env.exec(r#"
        local callback = assert(StaticPopupDialogs.USE_NO_REFUND_CONFIRM.OnAccept)
        local source = debug.getinfo(callback, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_StaticPopup_Game/GameDialogDefs.lua', 1, true), source)
        callback(nil, nil)
    "#).unwrap();
    assert!(env.state().borrow().pending_item_use_identity.is_none());
}
}
