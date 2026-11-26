#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn spell_targeting_admin_input_updates_existing_classification_sets() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        A_Admin.SetSpellTargeting(123456, "helpful")
        A_Admin.SetSpellTargeting(234567, "harmful")
        assert(IsHelpfulSpell(123456) and not IsHarmfulSpell(123456))
        assert(IsHarmfulSpell(234567) and not IsHelpfulSpell(234567))
        A_Admin.SetSpellTargeting(123456, nil)
        assert(not IsHelpfulSpell(123456) and not IsHarmfulSpell(123456))
        "#,
    )
    .expect("spell targeting input should update the existing spell sets");
}

#[test]
fn item_targeting_uses_item_use_spells_and_returns_false_without_one() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        local _, helpfulSpell = C_Item.GetItemSpell(5512)
        local _, harmfulSpell = C_Item.GetItemSpell(6948)
        assert(helpfulSpell and helpfulSpell > 0, "Healthstone use spell must resolve")
        assert(harmfulSpell and harmfulSpell > 0, "Hearthstone use spell must resolve")
        assert(helpfulSpell ~= harmfulSpell, "test items must have distinct use spells")
        A_Admin.SetSpellTargeting(helpfulSpell, "helpful")
        A_Admin.SetSpellTargeting(harmfulSpell, "harmful")
        assert(C_Item.IsHelpfulItem(5512) == true)
        assert(C_Item.IsHarmfulItem(6948) == true)
        assert(C_Item.IsHarmfulItem(5512) == false)
        assert(C_Item.IsHelpfulItem(6948) == false)
        assert(C_Item.IsHelpfulItem(999999) == false)
        assert(C_Item.IsHarmfulItem(999999) == false)
        "#,
    )
    .expect("item targeting should classify supplied use spells");
}

