#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::trait_group_display::{GroupDisplayRow, SkillLineDisplay};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn trait_display_rows_preserve_provider_order_and_skill_line_joins() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let model = &mut sim.talents.forever.group_display;
        model.rows_by_tree.insert(7, vec![
            GroupDisplayRow { group_id: 33, skill_line_id: 50, order_index: 30 },
            GroupDisplayRow { group_id: 11, skill_line_id: 51, order_index: 1 },
            GroupDisplayRow { group_id: 22, skill_line_id: 52, order_index: 2 },
        ]);
        model.skill_lines.insert(50, SkillLineDisplay { name: "First supplied".into(), icon: 135920 });
        model.skill_lines.insert(51, SkillLineDisplay { name: "Second supplied".into(), icon: 135873 });
    }
    env.exec(r#"
        local rows = C_Traits.GetGroupDisplayInfoByTreeID(7)
        assert(#rows == 3 and rows[1].groupID == 33 and rows[2].groupID == 11)
        assert(rows[1].treeID == 7 and rows[1].skillLineID == 50 and rows[1].orderIndex == 30)
        assert(rows[1].displayName == 'First supplied' and rows[1].icon == 135920)
        assert(rows[2].displayName == 'Second supplied' and rows[2].icon == 135873)
        assert(rows[3].groupID == 22 and rows[3].displayName == '' and rows[3].icon == 0)
        rows[1].displayName = 'mutated'
        assert(C_Traits.GetGroupDisplayInfoByTreeID(7)[1].displayName == 'First supplied')
        assert(C_Traits.GetGroupDisplayInfoByTreeID(7.9)[1].treeID == 7)
        assert(#C_Traits.GetGroupDisplayInfoByTreeID(8) == 0)
        assert(#C_Traits.GetGroupDisplayInfoByTreeID(0) == 0)
        assert(select('#', C_Traits.GetGroupDisplayInfoByTreeID(8)) == 1)
    "#).unwrap();
    env.state().borrow_mut().talents.forever.group_display.rows_by_tree.remove(&7);
    env.exec("assert(#C_Traits.GetGroupDisplayInfoByTreeID(7) == 0)").unwrap();
}

#[test]
fn trait_display_tree_id_uses_required_signed_numeric_reader() {
    let env = WowLuaEnv::new().unwrap();
    for call in ["C_Traits.GetGroupDisplayInfoByTreeID()", "C_Traits.GetGroupDisplayInfoByTreeID(nil)", "C_Traits.GetGroupDisplayInfoByTreeID('7')"] {
        let error = env.exec(call).expect_err("required numeric tree ID").to_string();
        assert!(error.contains("Usage: local displayInfos = C_Traits.GetGroupDisplayInfoByTreeID(treeID)"), "{error}");
    }
    for number in [-2147483649.0, 2147483648.0] {
        assert!(env.exec(&format!("C_Traits.GetGroupDisplayInfoByTreeID({number})")).is_err());
    }
    env.exec("assert(#C_Traits.GetGroupDisplayInfoByTreeID(-2147483648) == 0); assert(#C_Traits.GetGroupDisplayInfoByTreeID(2147483647) == 0)").unwrap();
}
