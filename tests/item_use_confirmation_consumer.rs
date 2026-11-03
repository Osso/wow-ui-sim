//! Original StaticPopup callbacks for pending item-use confirmation (WM44-C/D).
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::container_actions::ContainerUseRequest;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

fn owned(item_id: u32, identity: u64) -> BagItem {
    BagItem {
        owned_identity: Some(identity),
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

fn accept_original(env: &WowLuaEnv, dialog: &str, data: &str) {
    env.exec(&format!(
        r#"
        local callback = assert(StaticPopupDialogs.{dialog}.OnAccept)
        local source = debug.getinfo(callback, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_StaticPopup_Game/GameDialogDefs.lua', 1, true), source)
        callback(nil, {data})
    "#
    ))
    .unwrap();
}

fn requests(env: &WowLuaEnv) -> Vec<ContainerUseRequest> {
    env.state().borrow().container_actions.use_requests.clone()
}

prefork_full_ui_case! {
fn item_confirmation_popups_use_and_clear_the_pending_item(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 2), owned(6948, 61));
        sim.bag_items.insert((0, 4), owned(7005, 62));
        sim.container_actions.usable_items.insert(6948);
        sim.container_actions.use_requests.clear();
        sim.pending_item_use = Some(61);
    }
    accept_original(env, "CONFIM_BEFORE_USE", "nil");
    let used = ContainerUseRequest { bag: 0, slot: 2, item_id: 6948 };
    assert_eq!(requests(env), vec![used.clone()]);
    assert!(env.state().borrow().pending_item_use.is_none());
    // A refused use still clears the pending item.
    env.state().borrow_mut().pending_item_use = Some(62);
    accept_original(env, "USE_NO_REFUND_CONFIRM", "nil");
    assert_eq!(requests(env), vec![used.clone()]);
    assert!(env.state().borrow().pending_item_use.is_none());
    // So does a pending identity that no longer resolves.
    env.state().borrow_mut().pending_item_use = Some(99);
    accept_original(env, "CONFIM_BEFORE_USE", "nil");
    assert_eq!(requests(env), vec![used.clone()]);
    assert!(env.state().borrow().pending_item_use.is_none());
}
}

prefork_full_ui_case! {
fn use_bind_popup_uses_and_clears_the_pending_item(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 2), owned(6948, 61));
        sim.bag_items.insert((0, 4), owned(7005, 62));
        sim.container_actions.usable_items.insert(6948);
        sim.container_actions.use_requests.clear();
        sim.pending_item_use = Some(61);
    }
    accept_original(env, "USE_BIND", "nil");
    let used = ContainerUseRequest { bag: 0, slot: 2, item_id: 6948 };
    assert_eq!(requests(env), vec![used.clone()]);
    assert!(env.state().borrow().pending_item_use.is_none());
    env.state().borrow_mut().pending_item_use = Some(62);
    accept_original(env, "USE_BIND", "nil");
    assert_eq!(requests(env), vec![used]);
    assert!(env.state().borrow().pending_item_use.is_none());
}
}

prefork_full_ui_case! {
fn secure_use_command_searches_and_uses_the_named_item(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_items.insert((0, 5), owned(6948, 63));
        sim.container_actions.usable_items.insert(6948);
        sim.container_actions.use_requests.clear();
    }
    env.exec(r#"
        local source = debug.getinfo(SecureCmdUseItem, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua', 1, true), source)
        SecureCmdUseItem('Hearthstone', nil, nil, nil)
    "#).unwrap();
    let used = ContainerUseRequest { bag: 0, slot: 5, item_id: 6948 };
    assert_eq!(requests(env), vec![used]);
}
}

prefork_full_ui_case! {
fn end_bound_tradeable_consumes_only_for_item_consume(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 3), owned(6948, 71));
        sim.container_actions.usable_items.insert(6948);
        sim.container_actions.use_requests.clear();
        sim.pending_item_use = Some(71);
    }
    for branch in ["'gem'", "'itemenchant'", "'spellenchant'"] {
        accept_original(env, "END_BOUND_TRADEABLE", branch);
        assert!(requests(env).is_empty(), "{branch}");
        assert_eq!(env.state().borrow().pending_item_use, Some(71), "{branch}");
    }
    accept_original(env, "END_BOUND_TRADEABLE", "'itemconsume'");
    let used = ContainerUseRequest { bag: 0, slot: 3, item_id: 6948 };
    assert_eq!(requests(env), vec![used]);
    assert!(env.state().borrow().pending_item_use.is_none());
}
}
