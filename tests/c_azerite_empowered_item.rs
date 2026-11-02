//! Integration tests for the `C_AzeriteEmpoweredItem` respec-flow and
//! panel-flow surface registered in `src/c_api/c_azerite_empowered_item.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{
    AzeriteEmpoweredPowerText, AzeriteEmpoweredSelectionKey, BagItem, EquippedItem,
};

#[test]
fn namespace_is_present() {
    let env = WowLuaEnv::new().expect("env");
    let ns_type: String = env.eval("return type(C_AzeriteEmpoweredItem)").unwrap();
    assert_eq!(ns_type, "table");
    let close_type: String = env
        .eval("return type(C_AzeriteEmpoweredItem.CloseAzeriteEmpoweredItemRespec)")
        .unwrap();
    assert_eq!(close_type, "function");
}

#[test]
fn get_respec_cost_defaults_to_zero() {
    let env = WowLuaEnv::new().expect("env");
    let cost: f64 = env
        .eval("return C_AzeriteEmpoweredItem.GetAzeriteEmpoweredItemRespecCost()")
        .unwrap();
    assert!(cost.abs() < 1e-6);
}

#[test]
fn get_respec_cost_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().azerite_empowered.respec_cost = 12_345_600;
    let cost: f64 = env
        .eval("return C_AzeriteEmpoweredItem.GetAzeriteEmpoweredItemRespecCost()")
        .unwrap();
    assert!((cost - 12_345_600.0).abs() < 1e-3);
}

#[test]
fn is_empowered_returns_false_without_state() {
    let env = WowLuaEnv::new().expect("env");
    let empowered: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItem({itemID = 158041})")
        .unwrap();
    assert!(!empowered);
}

#[test]
fn is_empowered_returns_false_for_nil_location() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .azerite_empowered
        .empowered_items
        .insert(158041);
    let empowered: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItem(nil)")
        .unwrap();
    assert!(!empowered);
}

#[test]
fn is_empowered_matches_item_id_table_shape() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .azerite_empowered
        .empowered_items
        .insert(158041);
    let empowered: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItem({itemID = 158041})")
        .unwrap();
    assert!(empowered);
    let other: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItem({itemID = 99999})")
        .unwrap();
    assert!(!other);
}

#[test]
fn is_empowered_resolves_bag_and_slot() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.azerite_empowered.empowered_items.insert(158041);
        state.bag_items.insert(
            (0, 3),
            BagItem {
                item_id: 158041,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
                owned_identity: None,
            },
        );
    }
    let empowered: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItem({bagID = 0, slotIndex = 3})")
        .unwrap();
    assert!(empowered);
}

#[test]
fn is_empowered_resolves_equipment_slot() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.azerite_empowered.empowered_items.insert(158041);
        state.player.equipped_items.insert(
            5,
            EquippedItem {
                item_id: 158041,
                enchant_id: 0,
                gem_ids: [0, 0, 0],
                applied_transmog: None,
                owned_identity: None,
            },
        );
    }
    let empowered: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItem({equipmentSlotIndex = 5})")
        .unwrap();
    assert!(empowered);
}

#[test]
fn confirm_respec_records_location() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        "C_AzeriteEmpoweredItem.ConfirmAzeriteEmpoweredItemRespec({bagID = 0, slotIndex = 4})",
    )
    .unwrap();
    let recorded = env
        .state()
        .borrow()
        .azerite_empowered
        .last_confirmed_respec
        .clone();
    let location = recorded.expect("confirm should record the location");
    assert_eq!(location.bag_id, Some(0));
    assert_eq!(location.slot_index, Some(4));
    assert_eq!(location.equipment_slot_index, None);
}

#[test]
fn confirm_respec_is_noop_with_nil_location() {
    let env = WowLuaEnv::new().expect("env");
    env.exec("C_AzeriteEmpoweredItem.ConfirmAzeriteEmpoweredItemRespec(nil)")
        .unwrap();
    let recorded = env
        .state()
        .borrow()
        .azerite_empowered
        .last_confirmed_respec
        .clone();
    assert!(recorded.is_none());
}

#[test]
fn close_respec_fires_event_and_records_location() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
        fired = 0
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("AZERITE_EMPOWERED_ITEM_RESPEC_CLOSE")
        frame:SetScript("OnEvent", function() fired = fired + 1 end)
        C_AzeriteEmpoweredItem.CloseAzeriteEmpoweredItemRespec({equipmentSlotIndex = 7})
    "#,
    )
    .unwrap();
    let fired: f64 = env.eval("return fired").unwrap();
    assert!((fired - 1.0).abs() < 1e-6);
    let recorded = env
        .state()
        .borrow()
        .azerite_empowered
        .last_close_request
        .clone();
    let location = recorded.expect("close should record the location");
    assert_eq!(location.equipment_slot_index, Some(7));
}

#[test]
fn get_power_text_returns_nil_when_unseeded() {
    let env = WowLuaEnv::new().expect("env");
    let kind: String = env
        .eval("return type(C_AzeriteEmpoweredItem.GetPowerText({itemID = 158041}, 263, 0))")
        .unwrap();
    assert_eq!(kind, "nil");
}

#[test]
fn get_power_text_returns_seeded_struct() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .azerite_empowered
        .power_text
        .insert(
            (158041, 263, 0),
            AzeriteEmpoweredPowerText {
                name: "Wracking Brilliance".to_string(),
                description: "Increases your Critical Strike.".to_string(),
            },
        );
    let name: String = env
        .eval("return C_AzeriteEmpoweredItem.GetPowerText({itemID = 158041}, 263, 0).name")
        .unwrap();
    assert_eq!(name, "Wracking Brilliance");
    let description: String = env
        .eval("return C_AzeriteEmpoweredItem.GetPowerText({itemID = 158041}, 263, 0).description")
        .unwrap();
    assert_eq!(description, "Increases your Critical Strike.");
}

#[test]
fn get_power_text_keys_on_level() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.azerite_empowered.power_text.insert(
            (158041, 263, 0),
            AzeriteEmpoweredPowerText {
                name: "Base".to_string(),
                description: "base text".to_string(),
            },
        );
        state.azerite_empowered.power_text.insert(
            (158041, 263, 1),
            AzeriteEmpoweredPowerText {
                name: "Upgraded".to_string(),
                description: "upgraded text".to_string(),
            },
        );
    }
    let base_name: String = env
        .eval("return C_AzeriteEmpoweredItem.GetPowerText({itemID = 158041}, 263, 0).name")
        .unwrap();
    let upgraded_name: String = env
        .eval("return C_AzeriteEmpoweredItem.GetPowerText({itemID = 158041}, 263, 1).name")
        .unwrap();
    assert_eq!(base_name, "Base");
    assert_eq!(upgraded_name, "Upgraded");
}

#[test]
fn is_heart_of_azeroth_equipped_defaults_false() {
    let env = WowLuaEnv::new().expect("env");
    let equipped: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsHeartOfAzerothEquipped()")
        .unwrap();
    assert!(!equipped);
}

#[test]
fn is_heart_of_azeroth_equipped_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().azerite_empowered.heart_equipped = true;
    let equipped: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsHeartOfAzerothEquipped()")
        .unwrap();
    assert!(equipped);
}

#[test]
fn is_power_available_for_spec_defaults_true() {
    let env = WowLuaEnv::new().expect("env");
    let available: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsPowerAvailableForSpec(263, 70)")
        .unwrap();
    assert!(available);
}

#[test]
fn is_power_available_for_spec_reads_seeded_false() {
    let env = WowLuaEnv::new().expect("env");
    env.state()
        .borrow_mut()
        .azerite_empowered
        .spec_available
        .insert((263, 70), false);
    let available: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsPowerAvailableForSpec(263, 70)")
        .unwrap();
    assert!(!available);
    let other: bool = env
        .eval("return C_AzeriteEmpoweredItem.IsPowerAvailableForSpec(263, 71)")
        .unwrap();
    assert!(other);
}

#[test]
fn select_power_returns_false_for_unresolvable_location() {
    let env = WowLuaEnv::new().expect("env");
    let success: bool = env
        .eval("return C_AzeriteEmpoweredItem.SelectPower({bagID = 0, slotIndex = 99}, 263)")
        .unwrap();
    assert!(!success);
}

#[test]
fn select_power_appends_and_fires_event() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
        fired = 0
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED")
        frame:SetScript("OnEvent", function() fired = fired + 1 end)
    "#,
    )
    .unwrap();
    let success: bool = env
        .eval("return C_AzeriteEmpoweredItem.SelectPower({itemID = 158041, bagID = 0, slotIndex = 4}, 263)")
        .unwrap();
    assert!(success);
    let fired: f64 = env.eval("return fired").unwrap();
    assert!((fired - 1.0).abs() < 1e-6);
    let key = AzeriteEmpoweredSelectionKey {
        item_id: 158041,
        bag_id: Some(0),
        slot_index: Some(4),
        equipment_slot_index: None,
    };
    let powers = env
        .state()
        .borrow()
        .azerite_empowered
        .selections
        .get(&key)
        .cloned()
        .expect("selection should be recorded under the resolved key");
    assert_eq!(powers, vec![263]);
}

#[test]
fn select_power_keeps_locations_independent() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        "C_AzeriteEmpoweredItem.SelectPower({itemID = 158041, bagID = 0, slotIndex = 4}, 263)",
    )
    .unwrap();
    env.exec(
        "C_AzeriteEmpoweredItem.SelectPower({itemID = 158041, bagID = 1, slotIndex = 9}, 264)",
    )
    .unwrap();
    let bag0 = env
        .state()
        .borrow()
        .azerite_empowered
        .selections
        .get(&AzeriteEmpoweredSelectionKey {
            item_id: 158041,
            bag_id: Some(0),
            slot_index: Some(4),
            equipment_slot_index: None,
        })
        .cloned()
        .expect("first location entry");
    let bag1 = env
        .state()
        .borrow()
        .azerite_empowered
        .selections
        .get(&AzeriteEmpoweredSelectionKey {
            item_id: 158041,
            bag_id: Some(1),
            slot_index: Some(9),
            equipment_slot_index: None,
        })
        .cloned()
        .expect("second location entry");
    assert_eq!(bag0, vec![263]);
    assert_eq!(bag1, vec![264]);
}

#[test]
fn confirm_respec_clears_selections_for_resolved_item() {
    let env = WowLuaEnv::new().expect("env");
    let key = AzeriteEmpoweredSelectionKey {
        item_id: 158041,
        bag_id: Some(0),
        slot_index: Some(4),
        equipment_slot_index: None,
    };
    env.state()
        .borrow_mut()
        .azerite_empowered
        .selections
        .insert(key.clone(), vec![263, 264]);
    env.exec(
        "C_AzeriteEmpoweredItem.ConfirmAzeriteEmpoweredItemRespec({itemID = 158041, bagID = 0, slotIndex = 4})",
    )
    .unwrap();
    let cleared = env
        .state()
        .borrow()
        .azerite_empowered
        .selections
        .contains_key(&key);
    assert!(!cleared);
}

#[test]
fn close_respec_with_no_arg_still_fires_event() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
        fired = 0
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("AZERITE_EMPOWERED_ITEM_RESPEC_CLOSE")
        frame:SetScript("OnEvent", function() fired = fired + 1 end)
        C_AzeriteEmpoweredItem.CloseAzeriteEmpoweredItemRespec()
    "#,
    )
    .unwrap();
    let fired: f64 = env.eval("return fired").unwrap();
    assert!((fired - 1.0).abs() < 1e-6);
    let recorded = env
        .state()
        .borrow()
        .azerite_empowered
        .last_close_request
        .clone();
    assert!(recorded.is_none());
}
