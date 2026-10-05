#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_rest_policy_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_CombatLog.IsCombatLogRestricted() == false)
        assert(C_StableInfo.IsBonusPetSlotAvailable() == false)
        assert(C_HousingCustomizeMode.IsHouseExteriorDoorHovered() == false)
        for kind = 0, 3 do assert(C_LimitedInput.LimitedInputAllowed(kind) == false) end
        assert(not pcall(C_LimitedInput.LimitedInputAllowed, 4))
    "#,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.combat_log_restricted = true;
        state.pet_bonus_slot_available = true;
        state.housing.exterior.entry_door_hovered = true;
        state.limited_input_allowed = [true, false, true, false];
    }
    env.exec(r#"
        assert(C_CombatLog.IsCombatLogRestricted())
        assert(C_StableInfo.IsBonusPetSlotAvailable())
        assert(C_HousingCustomizeMode.IsHouseExteriorDoorHovered())
        assert(C_LimitedInput.LimitedInputAllowed(0) and C_LimitedInput.LimitedInputAllowed(2))
        assert(not C_LimitedInput.LimitedInputAllowed(1) and not C_LimitedInput.LimitedInputAllowed(3))
    "#).unwrap();
}

#[test]
fn p1200_rest_new_catalog_products_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(#C_CatalogShop.GetNewProducts() == 0)")
        .unwrap();
    env.state()
        .borrow_mut()
        .catalog_shop_products
        .new_product_ids = vec![44, 12];
    env.exec(
        r#"
        local ids = C_CatalogShop.GetNewProducts()
        assert(#ids == 2 and ids[1] == 44 and ids[2] == 12)
        ids[1] = 88
        assert(C_CatalogShop.GetNewProducts()[1] == 44)
    "#,
    )
    .unwrap();
}
