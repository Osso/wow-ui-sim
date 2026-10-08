//! Bounded current-retail tree context; no reconstructed Legion catalog claim.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::c_garrison_trees::GarrisonTalentTree;
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_7_2_5_garrison_empty_context(env: &WowLuaEnv) {
    let (tree, faction, lists): (bool, bool, i32) = env.eval(r#"
        return C_Garrison.GetCurrentGarrTalentTreeID() == nil,
               C_Garrison.GetCurrentGarrTalentTreeFriendshipFactionID() == nil,
               select('#', C_Garrison.GetTalentTreeIDsByClassID(3, 2))
    "#).expect("empty tree context queries");
    assert!(tree);
    assert!(faction);
    assert_eq!(lists, 0);
}
}

fn seed_trees(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let trees = &mut state.garrison_trees;
    trees.catalog.insert(461, GarrisonTalentTree {
        garrison_type: 3, class_id: 2, friendship_faction_id: Some(2164),
    });
    trees.catalog.insert(281, GarrisonTalentTree {
        garrison_type: 3, class_id: 2, friendship_faction_id: None,
    });
    trees.catalog.insert(900, GarrisonTalentTree {
        garrison_type: 9, class_id: 2, friendship_faction_id: Some(2507),
    });
    trees.catalog.insert(283, GarrisonTalentTree {
        garrison_type: 3, class_id: 1, friendship_faction_id: None,
    });
    trees.current_tree_id = Some(461);
}

fn assert_context_transitions(env: &WowLuaEnv) {
    seed_trees(env);
    let selected: (i64, i64) = env.eval(r#"
        return C_Garrison.GetCurrentGarrTalentTreeID(),
               C_Garrison.GetCurrentGarrTalentTreeFriendshipFactionID()
    "#).unwrap();
    assert_eq!(selected, (461, 2164));
    env.state().borrow_mut().garrison_trees.current_tree_id = Some(281);
    let (selected, no_faction): (i64, bool) = env.eval(r#"
        return C_Garrison.GetCurrentGarrTalentTreeID(),
               C_Garrison.GetCurrentGarrTalentTreeFriendshipFactionID() == nil
    "#).unwrap();
    assert_eq!(selected, 281);
    assert!(no_faction);
    env.state().borrow_mut().garrison_trees.current_tree_id = None;
    let cleared: bool = env.eval(r#"
        return C_Garrison.GetCurrentGarrTalentTreeID() == nil and
               C_Garrison.GetCurrentGarrTalentTreeFriendshipFactionID() == nil
    "#).unwrap();
    assert!(cleared);
}

fn assert_filtered_catalog(env: &WowLuaEnv) {
    seed_trees(env);
    let result: String = env.eval(r#"
        local first = C_Garrison.GetTalentTreeIDsByClassID(3, 2)
        assert(#first == 2 and first[1] == 281 and first[2] == 461)
        first[1] = 999
        local second = C_Garrison.GetTalentTreeIDsByClassID(3, 2)
        assert(second[1] == 281 and second[2] == 461)
        assert(C_Garrison.GetTalentTreeIDsByClassID(9, 2)[1] == 900)
        assert(C_Garrison.GetTalentTreeIDsByClassID(3, 1)[1] == 283)
        assert(select('#', C_Garrison.GetTalentTreeIDsByClassID(3, 13)) == 0)
        return table.concat(second, ',')
    "#).unwrap();
    assert_eq!(result, "281,461");
    env.state().borrow_mut().garrison_trees.catalog.remove(&281);
    let remaining: String = env.eval("return table.concat(C_Garrison.GetTalentTreeIDsByClassID(3, 2), ',')").unwrap();
    assert_eq!(remaining, "461");
}

prefork_full_ui_case! {
fn patch_7_2_5_garrison_selected_context(env: &WowLuaEnv) {
    assert_context_transitions(env);
}
}

prefork_full_ui_case! {
fn patch_7_2_5_garrison_catalog_filters_and_isolation(env: &WowLuaEnv) {
    assert_filtered_catalog(env);
}
}

prefork_full_ui_case! {
fn patch_7_2_5_garrison_required_arguments(env: &WowLuaEnv) {
    let valid: bool = env.eval(r#"
        local missing = pcall(C_Garrison.GetTalentTreeIDsByClassID)
        local badClass = pcall(C_Garrison.GetTalentTreeIDsByClassID, 3, {})
        local badType = pcall(C_Garrison.GetTalentTreeIDsByClassID, {}, 2)
        return not missing and not badClass and not badType
    "#).unwrap();
    assert!(valid);
}
}

#[test]
fn patch_7_2_5_garrison_bare_context_transitions() {
    let env = WowLuaEnv::new().unwrap();
    assert_context_transitions(&env);
}

#[test]
fn patch_7_2_5_garrison_bare_catalog_isolation() {
    let env = WowLuaEnv::new().unwrap();
    assert_filtered_catalog(&env);
}
