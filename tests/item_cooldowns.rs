use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{BagItem, SpellCooldownState};

fn seed_item_cooldown_env() -> WowLuaEnv {
    let env = crate::support::env();
    {
        let mut state = env.state().borrow_mut();
        // Item ID 13 deliberately collides with equipment slot 13.
        for (item_id, start, duration) in [(6948, 11.0, 31.0), (13, 22.0, 47.0)] {
            state
                .item_cooldowns
                .insert(item_id, SpellCooldownState { start, duration });
        }
        state.inventory_item_cooldowns.insert(
            13,
            SpellCooldownState {
                start: 33.0,
                duration: 59.0,
            },
        );
        for (slot, item_id) in [(8, 6948), (13, 13)] {
            state.bag_items.insert(
                (0, slot),
                BagItem {
                    item_id,
                    stack_count: 1,
                    hyperlink: None,
                    durability: None,
                    applied_transmog: None,
                    owned_identity: None,
                },
            );
        }
        state.item_cooldowns.remove(&999999);
        state.bag_items.remove(&(0, 15));
        state.inventory_item_cooldowns.remove(&15);
    }
    env.exec(
        r#"
        function assertItemCooldownTuple(expectedStart, expectedDuration, expectedEnable, ...)
            assert(select('#', ...) == 3, 'cooldown return count')
            local start, duration, enable = ...
            assert(type(start) == 'number', 'cooldown start type')
            assert(type(duration) == 'number', 'cooldown duration type')
            assert(type(enable) == type(expectedEnable), 'cooldown enable type')
            assert(start == expectedStart, 'cooldown start value')
            assert(duration == expectedDuration, 'cooldown duration value')
            assert(enable == expectedEnable, 'cooldown enable value')
        end
        "#,
    )
    .expect("install tuple assertions in isolated integration environment");
    env
}

#[test]
fn item_cooldown_item_info_id_and_hyperlink() {
    let env = seed_item_cooldown_env();
    env.exec(
        r#"
        local link = '|cnIQ1:|Hitem:6948::::::::80:70:::::::::|h[Hearthstone]|h|r'
        assert(C_Item.GetItemIDForItemInfo(6948) == 6948, 'numeric selector control')
        assert(C_Item.GetItemIDForItemInfo(link) == 6948, 'hyperlink selector control')
        assertItemCooldownTuple(11, 31, true, C_Item.GetItemCooldown(6948))
        assertItemCooldownTuple(11, 31, true, C_Item.GetItemCooldown(link))
        assertItemCooldownTuple(22, 47, true, C_Item.GetItemCooldown(13))
        assertItemCooldownTuple(0, 0, true, C_Item.GetItemCooldown(999999))
        "#,
    )
    .expect("C_Item ItemInfo selection and explicit missing-record policy");
}

#[test]
fn item_cooldown_container_item_id() {
    let env = seed_item_cooldown_env();
    env.exec(
        r#"
        assertItemCooldownTuple(11, 31, 1, C_Container.GetItemCooldown(6948))
        assertItemCooldownTuple(22, 47, 1, C_Container.GetItemCooldown(13))
        assertItemCooldownTuple(0, 0, 1, C_Container.GetItemCooldown(999999))
        "#,
    )
    .expect("C_Container numeric item-ID selection and explicit ready-shaped policy");
}

#[test]
fn item_cooldown_container_bag_slot() {
    let env = seed_item_cooldown_env();
    env.exec(
        r#"
        assert(C_Container.GetContainerItemID(0, 8) == 6948, 'bag occupant control')
        assert(C_Container.GetContainerItemID(0, 13) == 13, 'second occupant control')
        assert(C_Container.GetContainerItemID(0, 15) == nil, 'empty slot control')
        assertItemCooldownTuple(11, 31, 1, C_Container.GetContainerItemCooldown(0, 8))
        assertItemCooldownTuple(22, 47, 1, C_Container.GetContainerItemCooldown(0, 13))
        assertItemCooldownTuple(0, 0, 1, C_Container.GetContainerItemCooldown(0, 15))
        "#,
    )
    .expect("C_Container bag-slot selection remains a separate route");
}

#[test]
fn item_cooldown_legacy_equipment_slot_is_independent() {
    let env = seed_item_cooldown_env();
    env.exec(
        r#"
        assertItemCooldownTuple(33, 59, 1, GetInventoryItemCooldown('player', 13))
        assertItemCooldownTuple(0, 0, 1, GetInventoryItemCooldown('player', 15))
        "#,
    )
    .expect("legacy equipment-slot input is independent of colliding item-ID input");
}
