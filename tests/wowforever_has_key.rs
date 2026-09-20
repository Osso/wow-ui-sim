#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};

#[test]
fn forever_has_key_tracks_keyring_inventory() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().bag_items.clear();
    assert!(!env.eval::<bool>("return HasKey()").unwrap());
    let key = BagItem {
        item_id: 7146,
        stack_count: 1,
        hyperlink: None,
    };
    env.state()
        .borrow_mut()
        .bag_items
        .insert((0, 1), key.clone());
    assert!(!env.eval::<bool>("return HasKey()").unwrap());
    env.state().borrow_mut().bag_items.insert((-1, 1), key);
    assert!(env.eval::<bool>("return HasKey()").unwrap());
    assert_eq!(env.eval::<i32>("return select('#', HasKey())").unwrap(), 1);
    env.state()
        .borrow_mut()
        .bag_items
        .get_mut(&(-1, 1))
        .unwrap()
        .stack_count = 0;
    assert!(!env.eval::<bool>("return HasKey()").unwrap());
    env.state().borrow_mut().bag_items.remove(&(-1, 1));
    assert!(!env.eval::<bool>("return HasKey()").unwrap());
    assert!(
        !WowLuaEnv::new()
            .unwrap()
            .eval::<bool>("return HasKey()")
            .unwrap()
    );
}

#[test]
fn forever_keyring_tutorial_consumer_checks_inventory() {
    let env = WowLuaEnv::new().unwrap();
    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_MainMenuBarBagButtons/Camelot/MainMenuBarBagButtons.lua");
    let shared = path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("Shared/MainMenuBarBagButtons.lua");
    env.exec(&std::fs::read_to_string(shared).unwrap()).unwrap();
    env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    env.exec("SetCVar('showKeyring', 1); ring = CreateFrame('Button'); KeyRingMixin.TriggerTutorial(ring)").unwrap();
    env.state().borrow_mut().bag_items.insert(
        (-1, 1),
        BagItem {
            item_id: 7146,
            stack_count: 1,
            hyperlink: None,
        },
    );
    env.exec("KeyRingMixin.TriggerTutorial(ring); assert(GetCVarBool('showKeyring'))")
        .unwrap();
}
