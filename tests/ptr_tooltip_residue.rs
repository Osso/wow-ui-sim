//! WM92: bounded PTR 70668 evidence applied to the pinned 70493 UI profile.

#[cfg(feature = "client-ptr")]
use wow_ui_sim::c_api::action_button_hover::ActionVisualizationRecord;
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
mod tooltip_full_env_helpers;

#[cfg(feature = "client-ptr")]
fn supplied_visualization(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.action_bars.insert(5, 45524);
    sim.action_bars.insert(6, 49998);
    let state = &mut sim.spell_range_visualization;
    state.installed_object_identity = Some("installed-player".into());
    for (spell, value) in [(45524, 71), (49998, 92)] {
        state.records.insert(
            spell,
            ActionVisualizationRecord {
                identity: format!("action-{spell}"),
                visualization_value: Some(value),
            },
        );
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_bank_reagent_decision_matrix() {
    let env = WowLuaEnv::new().unwrap();
    for (object, restricted, expected) in [
        (None, false, false),
        (None, true, false),
        (Some(false), false, true),
        (Some(true), false, true),
        (Some(false), true, false),
        (Some(true), true, true),
    ] {
        env.state().borrow_mut().bank_reagent_object = object;
        env.exec(&format!(
            "A_Admin.SetGameRule('CraftFromBankRestricted', {restricted})"
        ))
        .unwrap();
        env.exec(&format!(
            r#"
            assert(select('#', ReagentsFromBankAllowed()) == 1)
            assert(ReagentsFromBankAllowed() == {expected})
            assert(ReagentsFromBankAllowed('ignored', nil, {{}}) == {expected})
        "#
        ))
        .unwrap();
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_item_util_uses_supplied_bank_object() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec("A_Admin.SetGameRule('CraftFromBankRestricted', true)")
        .unwrap();
    for flag in [false, true] {
        env.state().borrow_mut().bank_reagent_object = Some(flag);
        env.exec(&format!(
            r#"
            assert(ReagentsFromBankAllowed() == {flag})
            assert(type(ItemUtil.GetCraftingReagentCount(100001, false)) == 'number')
            assert(type(ItemUtil.GetCraftingReagentCount(100001, true)) == 'number')
        "#
        ))
        .unwrap();
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_action_hover_parser_and_zero_return_arity() {
    let env = WowLuaEnv::new().unwrap();
    supplied_visualization(&env);
    env.exec(r#"
        for _, name in ipairs({'EnterActionButton', 'LeaveActionButton'}) do
            local query = rawget(C_ActionBar, name)
            assert(type(query) == 'function', name)
            local usage = 'Usage: C_ActionBar.' .. name .. '(actionID)'
            local ok, err = pcall(query)
            assert(not ok and err:find(usage, 1, true), 'missing usage')
            ok, err = pcall(query, nil)
            assert(not ok and err:find(usage, 1, true), 'nil usage')
            for _, value in ipairs({-1, -0.1, 4294967296, math.huge, -math.huge, 0/0,
                                    'not a slot', false, {}, function() end}) do
                ok, err = pcall(query, value)
                assert(not ok and err:find('outside of expected range 0 to 4294967295', 1, true), tostring(value))
                assert(err:find(usage, 1, true) and err:find('bad argument #1', 1, true))
            end
            for _, value in ipairs({0, 409, 4294967295, '409', '0', 2147483649,
                                    0.25, 1, 1.75, '5', ' 6.75 ', 408, 408.9}) do
                assert(select('#', query(value)) == 0, tostring(value))
            end
        end
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_action_hover_secret_errors_are_distinct_emulated_policy() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'EnterActionButton', 'LeaveActionButton'}) do
            local query = C_ActionBar[name]
            local slot = secretwrap(5)
            assert(select('#', query(slot)) == 0, 'EMULATED AllowedWhenUntainted')
            local function probe()
                local ok, err = pcall(query, slot)
                assert(not ok and err:find('Secret values', 1, true), 'secret rejection')
                assert(not err:find('outside of expected range', 1, true), 'not a parser error')
                assert(debug.getstacktaint() == 'WM92Probe')
            end
            debug.setobjecttaint(probe, 'WM92Probe')
            probe()
        end
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_action_hover_rule_and_lookup_gates_preserve_state() {
    let env = WowLuaEnv::new().unwrap();
    supplied_visualization(&env);
    env.exec("C_ActionBar.EnterActionButton(5)").unwrap();
    assert_eq!(
        env.state().borrow().spell_range_visualization.stored_slot,
        None
    );
    env.exec("A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)")
        .unwrap();
    for (input, slot, value) in [("'5.75'", 4, 71), ("'6'", 5, 92)] {
        env.exec(&format!("C_ActionBar.EnterActionButton({input})"))
            .unwrap();
        let sim = env.state().borrow();
        assert_eq!(sim.spell_range_visualization.stored_slot, Some(slot));
        assert_eq!(sim.spell_range_visualization.value, value);
        assert!(sim.spell_range_visualization.enabled);
    }
    let saved = env.state().borrow().spell_range_visualization.clone();
    for call in [
        "C_ActionBar.EnterActionButton(7)",
        "C_ActionBar.EnterActionButton(409)",
        "C_ActionBar.LeaveActionButton(5)",
        "C_ActionBar.LeaveActionButton(0)",
    ] {
        env.exec(call).unwrap();
        assert_eq!(env.state().borrow().spell_range_visualization, saved);
    }
    // No action record, installed object, differing identity, or lookup means no update.
    for gate in 0..4 {
        env.state().borrow_mut().spell_range_visualization = saved.clone();
        {
            let mut sim = env.state().borrow_mut();
            let state = &mut sim.spell_range_visualization;
            match gate {
                0 => {
                    state.records.remove(&45524);
                }
                1 => state.installed_object_identity = None,
                2 => state.records.get_mut(&45524).unwrap().identity = "installed-player".into(),
                _ => state.records.get_mut(&45524).unwrap().visualization_value = None,
            }
        }
        env.exec("C_ActionBar.EnterActionButton(5)").unwrap();
        let sim = env.state().borrow();
        assert_eq!(sim.spell_range_visualization.stored_slot, Some(5));
        assert_eq!(sim.spell_range_visualization.value, 92);
        assert!(sim.spell_range_visualization.enabled);
    }
    env.exec("A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', false); C_ActionBar.LeaveActionButton('6')").unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.spell_range_visualization.stored_slot, Some(5));
    assert_eq!(sim.spell_range_visualization.value, 0);
    assert!(!sim.spell_range_visualization.enabled);
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_original_action_button_enter_and_leave_drive_visualization() {
    let env = tooltip_full_env_helpers::setup_full_env();
    supplied_visualization(&env);
    env.exec(
        r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)
        ActionButton5.action = 5
        ActionButton6.action = 6
    "#,
    )
    .unwrap();
    for (name, slot, value) in [("ActionButton5", 4, 71), ("ActionButton6", 5, 92)] {
        let id = env.state().borrow().widgets.get_id_by_name(name).unwrap();
        env.state().borrow_mut().hovered_frame = Some(id);
        env.state().borrow_mut().lua_errors.clear();
        env.fire_script_handler(id, "OnEnter", vec![]).unwrap();
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "original OnEnter failed: {:?}",
            env.state().borrow().lua_errors
        );
        assert_eq!(env.state().borrow().spell_range_visualization.value, value);
        assert!(env.state().borrow().spell_range_visualization.enabled);
        env.fire_script_handler(id, "OnLeave", vec![]).unwrap();
        let sim = env.state().borrow();
        assert!(
            sim.lua_errors.is_empty(),
            "original OnLeave failed: {:?}",
            sim.lua_errors
        );
        assert_eq!(sim.spell_range_visualization.stored_slot, Some(slot));
        assert_eq!(sim.spell_range_visualization.value, 0);
        assert!(!sim.spell_range_visualization.enabled);
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn tooltip_ptr_reagent_and_action_hover_apis_are_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(rawget(_G, 'ReagentsFromBankAllowed') == nil)
        assert(ReagentsFromBankAllowed == nil)
        for _, name in ipairs({'EnterActionButton', 'LeaveActionButton'}) do
            assert(rawget(C_ActionBar, name) == nil)
            assert(C_ActionBar[name] == nil)
        end
    "#,
    )
    .unwrap();
}
