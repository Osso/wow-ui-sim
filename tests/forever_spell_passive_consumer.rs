#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_action_button_spell_status_uses_passive_and_bar_inputs() {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();

    assert_eq!(
        wow_ui_sim::spells::get_spell(131).map(|spell| spell.name),
        Some("Water Breathing")
    );
    assert_eq!(
        wow_ui_sim::spells::get_spell(467).map(|spell| spell.name),
        Some("Thorns")
    );
    assert_eq!(
        wow_ui_sim::spells::get_spell(782).map(|spell| spell.name),
        Some("Thorns")
    );

    {
        let mut state = env.state().borrow_mut();
        state.set_input_interface_style(
            wow_ui_sim::c_api::c_input_interface_style::InputInterfaceStyle::Mkb,
        );
        state.spell_passive_attribute_masks.clear();
        state.spell_passive_attribute_masks.insert(131, 64);
        state.action_bars.clear();
        state.action_bars.insert(13, 467);
        state.ranged_auto_attack_spell_id = 467;
    }

    let action_bar_chain: String = env
        .eval(
            r#"
            local slots = C_ActionBar.FindSpellActionButtons(467, Enum.ActionBarSet.Mkb)
            local page = ActionButtonUtil.GetPageForSlot(13)
            local function show(value) return tostring(value) end
            return table.concat({
                'style=' .. show(C_InputInterfaceStyle.GetCurrentStyle()),
                'mkb=' .. show(InputUtil.IsMKBUIEnabled()),
                'slots=' .. show(slots and table.concat(slots, ',') or nil),
                'page=' .. show(page),
                'multi=' .. show(MultiActionBar_GetBarForPage(page)),
                'vehicle=' .. show(C_ActionBar.GetVehicleBarIndex()),
                'override=' .. show(C_ActionBar.GetOverrideBarIndex()),
                'temp=' .. show(C_ActionBar.GetTempShapeshiftBarIndex()),
                'bonus=' .. show(C_ActionBar.GetBonusBarIndexForSlot(13)),
                'viewable=' .. show(VIEWABLE_ACTION_BAR_PAGES[page]),
                'actionState=' .. show(ActionBarController_GetCurrentActionBarState()),
                'status=' .. show(ActionButtonUtil.GetActionBarStatusForSpell(467)),
            }, ';')
            "#,
        )
        .expect("inspect each input in the original action-bar status path");
    eprintln!("ActionButtonUtil status inputs: {action_bar_chain}");

    env.exec(
        r#"
        local status = ActionButtonUtil.GetActionBarStatusForSpell
        local actionStatus = ActionButtonUtil.ActionBarActionStatus
        assert(status(131) == actionStatus.NotMissing,
            'seeded passive spell must not be missing')
        assert(status(467) == actionStatus.NotMissing,
            'spell on an action bar must not be missing')
        assert(status(782) == actionStatus.MissingFromAllBars,
            'known non-passive spell off all bars must be missing')
        assert(C_Spell.IsSpellPassive(131), 'seeded passive mask is honored')
        assert(not C_Spell.IsSpellPassive(0), 'ID 0 is not passive')
        assert(C_Spell.IsRangedAutoAttackSpell(467))
        assert(not C_Spell.IsRangedAutoAttackSpell(782))
        "#,
    )
    .expect("original Forever ActionButtonUtil consumer returns spell status");
}
