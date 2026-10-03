#![cfg(feature = "retail-12-0-5")]

use std::collections::{HashMap, HashSet};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{EquipmentSet, EquippedItem};

fn equipset_command_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.equipped_items.clear();
        sim.player.equipped_items.insert(
            1,
            EquippedItem {
                item_id: 211993,
                enchant_id: 0,
                gem_ids: [0; 3],
            },
        );
        sim.player.equipped_items.insert(
            2,
            EquippedItem {
                item_id: 211994,
                enchant_id: 0,
                gem_ids: [0; 3],
            },
        );
        sim.equipment_manager.sets = vec![EquipmentSet {
            id: 71,
            name: "Raid Gear".into(),
            icon: String::new(),
            spec_index: None,
            ignored_slots: HashSet::from([2]),
            item_locations: HashMap::new(),
            item_ids: HashMap::from([(1, 211995), (3, 211996)]),
        }];
        sim.equipment_manager.last_used_set_id = None;
    }
    env
}

#[test]
fn equipset_invalid_names_do_not_error_or_mutate_and_valid_command_recovers() {
    let env = equipset_command_env();
    env.exec(
        r#"
        C_Macro.RunMacroText('/equipset Unknown Set\n/equipset\n/equipset    \n/equipset raid gear')
        assert(GetInventoryItemID('player', 1) == 211993)
        assert(GetInventoryItemID('player', 2) == 211994)
        assert(GetInventoryItemID('player', 3) == nil)
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().equipment_manager.last_used_set_id,
        None
    );
    env.exec(
        r#"
        C_Macro.RunMacroText('/equipset Raid Gear')
        assert(GetInventoryItemID('player', 1) == 211995)
        assert(GetInventoryItemID('player', 2) == 211994)
        assert(GetInventoryItemID('player', 3) == 211996)
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().equipment_manager.last_used_set_id,
        Some(71)
    );
    env.exec("C_Macro.RunMacroText('/equipset Unknown Set')")
        .unwrap();
    assert_eq!(
        env.state().borrow().equipment_manager.last_used_set_id,
        Some(71)
    );
    assert_eq!(
        env.state().borrow().player.equipped_items[&1].item_id,
        211995
    );
}

#[test]
fn equipset_command_uses_live_catalog_and_existing_swap_events() {
    let env = equipset_command_env();
    env.exec(
        r#"
        swapEvents = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('EQUIPMENT_SWAP_PENDING')
        frame:RegisterEvent('EQUIPMENT_SWAP_FINISHED')
        frame:RegisterEvent('EQUIPMENT_SETS_CHANGED')
        frame:SetScript('OnEvent', function(_, event, success, id)
            swapEvents[#swapEvents + 1] = {event, success, id}
        end)
        C_Macro.RunMacroText('/equipset Unknown')
        assert(#swapEvents == 0)
        C_Macro.RunMacroText('/equipset Raid Gear')
        assert(#swapEvents == 3)
        assert(swapEvents[1][1] == 'EQUIPMENT_SWAP_PENDING')
        assert(swapEvents[2][1] == 'EQUIPMENT_SWAP_FINISHED')
        assert(swapEvents[2][2] == true and swapEvents[2][3] == 71)
        assert(swapEvents[3][1] == 'EQUIPMENT_SETS_CHANGED')
        C_EquipmentSet.ModifyEquipmentSet(71, 'New Gear')
        C_Macro.RunMacroText('/equipset Raid Gear')
        assert(#swapEvents == 4)
        C_Macro.RunMacroText('/equipset New Gear')
        assert(#swapEvents == 7)
        C_EquipmentSet.DeleteEquipmentSet(71)
        C_Macro.RunMacroText('/equipset New Gear')
        assert(#swapEvents == 8)
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().equipment_manager.last_used_set_id,
        None
    );
}
