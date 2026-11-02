//! WM48 GetItemSpecInfo: supplied and derived records, API-level boundaries.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_spell::item_spec::{ItemSpecRecord, level_band};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_spec_info_filters_wm48_examples_and_returns_fresh_tables() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.level = 80;
        sim.player.class_index = 2;
    }
    env.exec(
        r#"
        local a = C_Item.GetItemSpecInfo(80773)
        assert(#a == 1 and a[1] == 65)
        local b = C_Item.GetItemSpecInfo(80773)
        assert(a ~= b)
        a[1] = 999
        assert(b[1] == 65)
        local gloves = C_Item.GetItemSpecInfo(892)
        local expected = {62,63,64,256,257,258,265,266,267}
        assert(#gloves == #expected)
        for i = 1, #expected do assert(gloves[i] == expected[i], tostring(i)) end
        assert(select('#', C_Item.GetItemSpecInfo(99999999)) == 0)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().player.class_index = 5;
    env.exec(
        r#"
        local specs = C_Item.GetItemSpecInfo(80773)
        assert(#specs == 3 and specs[1] == 256 and specs[2] == 257 and specs[3] == 258)
        A_Admin.SetGameRule('CharacterlessLogin', 1)
        assert(#C_Item.GetItemSpecInfo(80773) == 0)
        assert(#C_Item.GetItemSpecInfo(892) == 9)
    "#,
    )
    .unwrap();
}

#[test]
fn item_spec_info_rejects_markers_and_preserves_override_order() {
    let env = WowLuaEnv::new().unwrap();
    let key = (80773, level_band(80));
    {
        let mut sim = env.state().borrow_mut();
        sim.player.level = 80;
        sim.player.class_index = 2;
        sim.item_spec_records.insert(
            key,
            ItemSpecRecord {
                no_traits_and_specializations: false,
                no_stat_traits: true,
                reject_unmatched: false,
                item_flag_27: false,
                specialization_ids: vec![65, 1449, 65, 256],
            },
        );
    }
    env.exec(
        "local s = C_Item.GetItemSpecInfo(80773); assert(#s == 2 and s[1] == 65 and s[2] == 65)",
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .item_spec_records
        .get_mut(&key)
        .unwrap()
        .item_flag_27 = true;
    env.exec("assert(select('#', C_Item.GetItemSpecInfo(80773)) == 0)")
        .unwrap();
    let mut sim = env.state().borrow_mut();
    let record = sim.item_spec_records.get_mut(&key).unwrap();
    record.item_flag_27 = false;
    record.no_traits_and_specializations = true;
    drop(sim);
    env.exec("assert(select('#', C_Item.GetItemSpecInfo(80773)) == 0)")
        .unwrap();
}
