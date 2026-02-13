//! Tests for hero talent spec resolution.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_protection_hero_subtrees() {
    let env = env();
    // Protection (specID=66) should get Templar (48) and Lightsmith (49)
    let result: String = env
        .eval(
            r#"
            local ids, level = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 66)
            assert(ids, "subtree IDs should not be nil")
            assert(level == 71, "unlock level should be 71")
            table.sort(ids)
            return table.concat(ids, ",")
            "#,
        )
        .unwrap();
    assert_eq!(result, "48,49", "Protection should have Templar(48) + Lightsmith(49)");
}

#[test]
fn test_retribution_hero_subtrees() {
    let env = env();
    // Retribution (specID=70) should get Templar (48) and Herald of the Sun (50)
    let result: String = env
        .eval(
            r#"
            local ids, level = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 70)
            assert(ids, "subtree IDs should not be nil")
            table.sort(ids)
            return table.concat(ids, ",")
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "48,50",
        "Retribution should have Templar(48) + Herald of the Sun(50)"
    );
}

#[test]
fn test_holy_hero_subtrees() {
    let env = env();
    // Holy (specID=65) should get Lightsmith (49) and Herald of the Sun (50)
    let result: String = env
        .eval(
            r#"
            local ids, level = C_ClassTalents.GetHeroTalentSpecsForClassSpec(1, 65)
            assert(ids, "subtree IDs should not be nil")
            table.sort(ids)
            return table.concat(ids, ",")
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "49,50",
        "Holy should have Lightsmith(49) + Herald of the Sun(50)"
    );
}

#[test]
fn test_subtree_info_has_selection_node_ids() {
    let env = env();
    // Templar (subtree 48) should have subTreeSelectionNodeIDs
    let result: String = env
        .eval(
            r#"
            local info = C_Traits.GetSubTreeInfo(1, 48)
            assert(info, "subtree info should not be nil")
            assert(info.name == "Templar", "name should be Templar, got: " .. tostring(info.name))
            assert(info.subTreeSelectionNodeIDs, "should have subTreeSelectionNodeIDs")
            assert(#info.subTreeSelectionNodeIDs > 0, "should have at least one selection node")
            return tostring(#info.subTreeSelectionNodeIDs)
            "#,
        )
        .unwrap();
    assert!(
        result.parse::<i32>().unwrap() > 0,
        "Templar should have selection nodes"
    );
}

#[test]
fn test_tcontains_nil_safe() {
    let env = env();
    // tContains should return false when passed nil table
    let result: bool = env.eval("return tContains(nil, 42)").unwrap();
    assert!(!result, "tContains(nil, x) should return false");
}
