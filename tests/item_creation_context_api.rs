//! API-level tests for item queries without a call site in the pinned Forever UI.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_creation_context_returns_id_and_empty_for_unrecognized_contexts() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function link(id, context)
            return 'item:' .. table.concat({id, 0, 0, 0, 0, 0, 0, 0, 80, 70, 0, context}, ':')
        end
        local id1, context1 = C_Item.GetItemCreationContext(link(6948, 0))
        assert(id1 == 6948 and context1 == '')
        local id2, context2 = C_Item.GetItemCreationContext(link(211993, 999))
        assert(id2 == 211993 and context2 == '')
        assert(select('#', C_Item.GetItemCreationContext(link(6948, 0))) == 2)
        local _, normal = C_Item.GetItemCreationContext(link(6948, 1))
        local _, bonus = C_Item.GetItemCreationContext(link(6948, 191))
        local _, empty = C_Item.GetItemCreationContext(link(6948, 192))
        assert(normal == 'dungeon-normal')
        assert(bonus == 'bonus-roll-5')
        assert(empty == '')
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_item_gems_read_two_link_sockets() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local source = 'item:211993:7:22459:23094:0:0:0:0:80:70:0:12:0'
        local first, firstLink = C_Item.GetItemGem(source, 1)
        local second, secondLink = C_Item.GetItemGem(source, 2)
        assert(first == 'Void Sphere' and firstLink:find('|Hitem:22459:', 1, true))
        assert(second == 'Brilliant Blood Garnet' and secondLink:find('|Hitem:23094:', 1, true))
        assert(select('#', C_Item.GetItemGem(source, 3)) == 0)
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_item_children_select_catalog_rows() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local first = C_Item.GetItemChildInfo(127417, 16)
        assert(#first == 2 and first[1] == 1728 and first[2] == 15)
        local second = C_Item.GetItemChildInfo(127829, 17)
        assert(#second == 2 and second[1] == 127830 and second[2] == 16)
        assert(#C_Item.GetItemChildInfo(127829, 16) == 0)
        assert(#C_Item.GetItemChildInfo(6948) == 0)
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_item_range_uses_item_spell_range() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_Item.ItemHasRange(1434))
        assert(not C_Item.ItemHasRange(6948))
        assert(not C_Item.ItemHasRange(833))
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_item_type_reads_equipped_items() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        A_Admin.EquipItem(1, 211993)
        assert(C_Item.IsEquippedItemType('INVTYPE_HEAD'))
        A_Admin.EquipItem(16, 128872)
        assert(C_Item.IsEquippedItemType('INVTYPE_WEAPONMAINHAND'))
        assert(not C_Item.IsEquippedItemType('not an item type'))
    "#).unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_item_specialization_reads_catalog_membership() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        A_Admin.SetPlayerLevel(80)
        assert(C_Item.DoesItemContainSpec(80773, 2, 65))
        assert(not C_Item.DoesItemContainSpec(80773, 2, 66))
        assert(C_Item.DoesItemContainSpec(63684, 1, 71))
        assert(not C_Item.DoesItemContainSpec(63684, 10, 268))
    "#).unwrap();
}
