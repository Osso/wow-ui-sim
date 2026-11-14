//! Native q3.1 propagation through the selected Retail UI; the host supplies
//! initial propagation states because loaded native owners are not assumed.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn host_propagation(env: &WowLuaEnv, name: &str, requested: bool) {
    let mut sim = env.state().borrow_mut();
    let id = sim.widgets.get_id_by_name(name).unwrap();
    sim.widgets.get_mut(id).unwrap().anchor_propagation = requested;
    sim.widgets.set_screen_clamp_suppressed(id, requested);
}

fn assert_clamp(env: &WowLuaEnv, target: &str, expected: bool, phase: &str) {
    env.exec(&format!(
        "assert({target}:IsClampedToScreen() == {expected}, '{phase}')"
    ))
    .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
prefork_full_ui_case! {
fn original_tooltip_inherits_distinct_owner_and_coordinate_root_states(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_SharedXML'))
        assert(C_AddOns.IsAddOnLoaded('Blizzard_GameTooltip'))
        local source = debug.getinfo(GameTooltip_ShowSimpleTooltip, 'S').source
        assert(source:gsub('\\', '/'):find('Blizzard_SharedXML/SharedTooltipTemplates.lua', 1, true))
        ClampQ3CursorOwner = CreateFrame('Frame', 'ClampQ3CursorOwner', UIParent)
        ClampQ3CursorOwner:SetSize(100, 50)
        ClampQ3CursorOwner:SetPoint('CENTER')
        ClampQ3RightOwner = CreateFrame('Frame', 'ClampQ3RightOwner', UIParent)
        ClampQ3RightOwner:SetSize(50, 20)
        ClampQ3RightOwner:SetPoint('CENTER')
        GameTooltip:ClearAllPoints()
        GameTooltip:SetClampedToScreen(true)
    "#).unwrap();
    host_propagation(env, "GameTooltip", false);
    host_propagation(env, "ClampQ3CursorOwner", true);
    host_propagation(env, "ClampQ3RightOwner", true);
    host_propagation(env, "UIParent", true);

    env.exec(r#"
        GameTooltip_ShowSimpleTooltip(GameTooltip, 'Cursor', nil, false,
            ClampQ3CursorOwner, 'ANCHOR_CURSOR')
    "#).unwrap();
    assert_clamp(env, "GameTooltip", true, "cursor must use guarded false coordinate root");

    env.exec(r#"
        GameTooltip_ShowSimpleTooltip(GameTooltip, 'Owner right', nil, false,
            ClampQ3RightOwner, 'ANCHOR_RIGHT')
    "#).unwrap();
    assert_clamp(env, "GameTooltip", false, "right must inherit true owner propagation");

    env.exec("GameTooltip:ClearAllPoints()").unwrap();
    assert_clamp(env, "GameTooltip", true, "clear must request false");
    env.exec("GameTooltip:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 8, -8)").unwrap();
    assert_clamp(env, "GameTooltip", false, "final point must inherit true UIParent propagation");

    // INFERRED conditional walkthrough: supplied false owners and UIParent
    // leave all four suppression results false. Not a native startup claim.
    host_propagation(env, "ClampQ3CursorOwner", false);
    host_propagation(env, "ClampQ3RightOwner", false);
    host_propagation(env, "UIParent", false);
    env.exec("GameTooltip:ClearAllPoints()").unwrap();
    for (call, phase) in [
        ("GameTooltip_ShowSimpleTooltip(GameTooltip, 'Cursor', nil, false, ClampQ3CursorOwner, 'ANCHOR_CURSOR')", "conditional cursor"),
        ("GameTooltip_ShowSimpleTooltip(GameTooltip, 'Right', nil, false, ClampQ3RightOwner, 'ANCHOR_RIGHT')", "conditional right"),
        ("GameTooltip:ClearAllPoints()", "conditional clear"),
        ("GameTooltip:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 8, -8)", "conditional final point"),
    ] {
        env.exec(call).unwrap();
        assert_clamp(env, "GameTooltip", true, phase);
    }
}
}

#[test]
fn ordinary_movement_uses_coordinate_root_instead_of_propagating_parent() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(400.0, 300.0);
    env.exec(
        r#"
        ClampQ3MovingParent = CreateFrame('Frame', 'ClampQ3MovingParent', UIParent)
        ClampQ3MovingParent:SetSize(150, 100)
        ClampQ3MovingParent:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 25, -15)
        ClampQ3Moving = CreateFrame('Frame', 'ClampQ3Moving', ClampQ3MovingParent)
        ClampQ3Moving:SetSize(20, 30)
        ClampQ3Moving:SetMovable(true)
        ClampQ3Moving:SetClampedToScreen(true)
    "#,
    )
    .unwrap();
    host_propagation(&env, "ClampQ3MovingParent", true);
    env.exec("ClampQ3Moving:SetPoint('TOPLEFT', ClampQ3MovingParent, 'TOPLEFT', 12, -2)")
        .unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        false,
        "initial point must inherit true propagation",
    );
    env.exec("ClampQ3Moving:StartMoving()").unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        true,
        "StartMoving must use coordinate root",
    );
    env.exec("ClampQ3Moving:SetPoint('TOPLEFT', ClampQ3MovingParent, 'TOPLEFT', 12, -2)")
        .unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        false,
        "new attached dependency is true",
    );
    env.exec("ClampQ3Moving:StopMovingOrSizing()").unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        true,
        "ordinary settlement must use coordinate root",
    );
}
