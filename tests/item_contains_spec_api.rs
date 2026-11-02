//! API-level test: no pinned retail Blizzard consumer calls DoesItemContainSpec.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_spell::item_spec::{ItemSpecRecord, level_band};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn item_spec_query_uses_supplied_derived_record_and_player_presence() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.present = false;
    env.exec("assert(C_Item.DoesItemContainSpec(6948, 0, 65)); assert(not C_Item.DoesItemContainSpec(6948, 2, 65))").unwrap();
    env.state().borrow_mut().player.present = true;
    env.exec("assert(C_Item.DoesItemContainSpec(6948, 2, 65))")
        .unwrap();
    let key = (6948, level_band(env.state().borrow().player.level));
    env.state().borrow_mut().item_spec_records.insert(
        key,
        ItemSpecRecord {
            no_traits_and_specializations: false,
            no_stat_traits: false,
            reject_unmatched: true,
            item_flag_27: false,
            specialization_ids: vec![65, 66],
        },
    );
    env.exec(
        r#"
        assert(C_Item.DoesItemContainSpec(6948, 2, 65))
        assert(not C_Item.DoesItemContainSpec(6948, 2, 70))
        assert(C_Item.DoesItemContainSpec(6948, 2, 0))
        assert(not C_Item.DoesItemContainSpec(6948, 1, 0))
        assert(select('#', C_Item.DoesItemContainSpec(6948, 2, 65)) == 1)
        assert(not pcall(C_Item.DoesItemContainSpec, 6948))
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .item_spec_records
        .get_mut(&key)
        .unwrap()
        .reject_unmatched = false;
    env.exec("assert(C_Item.DoesItemContainSpec(6948, 2, 70))")
        .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let record = sim.item_spec_records.get_mut(&key).unwrap();
        record.reject_unmatched = true;
        record.no_traits_and_specializations = true;
    }
    env.exec("assert(C_Item.DoesItemContainSpec(6948, 2, 70))")
        .unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let record = sim.item_spec_records.get_mut(&key).unwrap();
        record.no_traits_and_specializations = false;
        record.no_stat_traits = true;
    }
    env.exec("assert(C_Item.DoesItemContainSpec(6948, 2, 70))")
        .unwrap();
}
