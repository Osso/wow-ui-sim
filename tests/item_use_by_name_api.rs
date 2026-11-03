//! WM68 `C_Item.UseItemByName` search order and prompt selection.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::container_actions::ContainerUseRequest;
use wow_ui_sim::c_api::item_spell::item_equip::{OwnedItemEquipInputs, UsePromptInputs};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, EquippedItem};

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

fn prompted(prompts: UsePromptInputs) -> OwnedItemEquipInputs {
    OwnedItemEquipInputs {
        use_prompts: prompts,
        ..Default::default()
    }
}

fn env_with_hearthstone() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_items.insert((0, 3), owned(6948, 81));
        sim.container_actions.usable_items.insert(6948);
        sim.container_actions.use_requests.clear();
    }
    env.exec(
        r#"
        promptEvents = {}
        local listener = CreateFrame('Frame')
        for _, name in ipairs({'USE_BIND_CONFIRM', 'CONFIRM_BEFORE_USE',
                'USE_NO_REFUND_CONFIRM', 'END_BOUND_TRADEABLE'}) do
            listener:RegisterEvent(name)
        end
        listener:SetScript('OnEvent', function(_, name, ...)
            promptEvents[#promptEvents + 1] = table.concat({name, select('#', ...), tostring((...))}, ',')
        end)
    "#,
    )
    .unwrap();
    env
}

fn prompt_events(env: &WowLuaEnv) -> String {
    env.eval("return table.concat(promptEvents, ';')").unwrap()
}

#[test]
fn ordinary_use_submits_the_first_carried_match() {
    let env = env_with_hearthstone();
    env.state()
        .borrow_mut()
        .bag_items
        .insert((0, 5), owned(6948, 82));
    env.exec("C_Item.UseItemByName('Hearthstone')").unwrap();
    let sim = env.state().borrow();
    let used = ContainerUseRequest {
        bag: 0,
        slot: 3,
        item_id: 6948,
    };
    assert_eq!(sim.container_actions.use_requests, vec![used]);
    assert!(sim.pending_item_use.is_none());
}

#[test]
fn zero_or_unknown_items_skip_the_action() {
    let env = env_with_hearthstone();
    env.exec("C_Item.UseItemByName('No Such Item'); C_Item.UseItemByName(0)")
        .unwrap();
    assert!(
        env.state()
            .borrow()
            .container_actions
            .use_requests
            .is_empty()
    );
}

#[test]
fn prompts_follow_the_verified_order_and_save_the_item() {
    let env = env_with_hearthstone();
    let all = UsePromptInputs {
        bind: true,
        before_use: true,
        no_refund: true,
        end_bound_tradeable: true,
    };
    env.state()
        .borrow_mut()
        .item_equip
        .inputs
        .insert(81, prompted(all));
    env.exec("C_Item.UseItemByName(6948)").unwrap();
    assert_eq!(prompt_events(&env), "USE_BIND_CONFIRM,0,nil");
    let sim = env.state().borrow();
    assert_eq!(sim.pending_item_use, Some(81));
    assert!(sim.container_actions.use_requests.is_empty());
    drop(sim);
    for (prompts, expected) in [
        (
            UsePromptInputs {
                before_use: true,
                no_refund: true,
                ..all
            },
            "CONFIRM_BEFORE_USE,0,nil",
        ),
        (
            UsePromptInputs {
                no_refund: true,
                end_bound_tradeable: true,
                ..Default::default()
            },
            "USE_NO_REFUND_CONFIRM,0,nil",
        ),
        (
            UsePromptInputs {
                end_bound_tradeable: true,
                ..Default::default()
            },
            "END_BOUND_TRADEABLE,1,itemconsume",
        ),
    ] {
        let prompts = UsePromptInputs {
            bind: false,
            ..prompts
        };
        env.state()
            .borrow_mut()
            .item_equip
            .inputs
            .insert(81, prompted(prompts));
        env.exec("promptEvents = {}; C_Item.UseItemByName(6948)")
            .unwrap();
        assert_eq!(prompt_events(&env), expected);
    }
}

#[test]
fn player_root_slots_are_searched_before_carried_bags() {
    let env = env_with_hearthstone();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.equipped_items.insert(
            16,
            EquippedItem {
                item_id: 6948,
                enchant_id: 0,
                gem_ids: [0; 3],
                applied_transmog: None,
                owned_identity: Some(91),
            },
        );
        let prompts = UsePromptInputs {
            before_use: true,
            ..Default::default()
        };
        sim.item_equip.inputs.insert(91, prompted(prompts));
        sim.item_equip.inputs.insert(81, prompted(prompts));
    }
    env.exec("C_Item.UseItemByName(6948)").unwrap();
    assert_eq!(env.state().borrow().pending_item_use, Some(91));
}
