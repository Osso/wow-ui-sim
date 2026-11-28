//! C_Item pending-use confirmations on Retail and Forever (WM44-C/D).
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

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

fn pending_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 2), owned(6948, 61));
        sim.bag_items.insert((0, 4), owned(7005, 62));
        sim.container_actions.usable_items.insert(6948);
        sim.container_actions.use_requests.clear();
    }
    env
}

fn confirm(env: &WowLuaEnv, pending: u64, call: &str) -> (Vec<ContainerUseRequest>, Option<u64>) {
    env.state().borrow_mut().pending_item_use = Some(pending);
    env.exec(call).unwrap();
    let sim = env.state().borrow();
    (sim.container_actions.use_requests.clone(), sim.pending_item_use)
}

#[test]
fn confirmations_use_and_clear_the_pending_item() {
    let env = pending_env();
    let used = ContainerUseRequest { bag: 0, slot: 2, item_id: 6948 };
    let (requests, pending) = confirm(&env, 61, "C_Item.ConfirmOnUse()");
    assert_eq!((requests, pending), (vec![used.clone()], None));
    // A refused use (unusable item) and an unresolved identity still clear it.
    let (requests, pending) = confirm(&env, 62, "C_Item.ConfirmNoRefundOnUse()");
    assert_eq!((requests, pending), (vec![used.clone()], None));
    let (requests, pending) = confirm(&env, 99, "C_Item.ConfirmBindOnUse()");
    assert_eq!((requests, pending), (vec![used], None));
}

#[test]
fn end_bound_tradeable_consumes_only_for_item_consume() {
    let env = pending_env();
    for branch in ["gem", "itemenchant", "spellenchant"] {
        let call = format!("C_Item.EndBoundTradeable('{branch}')");
        let (requests, pending) = confirm(&env, 61, &call);
        assert!(requests.is_empty(), "{branch}");
        assert_eq!(pending, Some(61), "{branch}");
    }
    let (requests, pending) = confirm(&env, 61, "C_Item.EndBoundTradeable('itemconsume')");
    let used = ContainerUseRequest { bag: 0, slot: 2, item_id: 6948 };
    assert_eq!((requests, pending), (vec![used], None));
}
