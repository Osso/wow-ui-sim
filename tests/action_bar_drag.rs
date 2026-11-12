use crate::common;

use wow_ui_sim::lua_api::WowLuaEnv;

fn seed_action_slot(env: &WowLuaEnv, slot: u32, spell_id: u32) {
    env.state().borrow_mut().action_bars.insert(slot, spell_id);
}

fn load_action_bar_addons(env: &WowLuaEnv) {
    common::panel_fixtures::seed_addon_search_paths(env);
    common::panel_fixtures::load_panel_addons(env);
}

fn fire_action_bar_startup(env: &WowLuaEnv) {
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("WoWUISim")])
        .unwrap();
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        env.fire_event(event).unwrap();
    }
    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
    )
    .unwrap();
    env.fire_edit_mode_layouts_updated().unwrap();
    env.fire_event("ACTIONBAR_SHOWGRID").unwrap();
}

fn env_with_action_bar() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_size(1024.0, 768.0);
        load_action_bar_addons(&env);
        env.apply_post_load_workarounds();
        fire_action_bar_startup(&env);
        env
    })
}

fn assert_action_button_template_has_receive_drag() {
    let chain = wow_ui_sim::xml::get_template_chain("ActionBarButtonTemplate");
    let code_template = chain
        .iter()
        .find(|entry| entry.name == "ActionBarButtonCodeTemplate")
        .unwrap();
    assert!(
        code_template
            .frame
            .scripts()
            .is_some_and(|scripts| !scripts.on_receive_drag.is_empty()),
        "template chain should include OnReceiveDrag on ActionBarButtonCodeTemplate"
    );
}

#[test]
fn pickup_action_accepts_ignore_removal_arg_and_place_restores_slot() {
    let env = WowLuaEnv::new().unwrap();
    seed_action_slot(&env, 1, 853);

    let ok: bool = env
        .eval(
            r#"
            local ok, err = pcall(function()
                PickupAction(1, false)
            end)
            TEST_PICKUP_ACTION_ERR = err
            return ok
            "#,
        )
        .unwrap();
    assert!(
        ok,
        "PickupAction(slot, ignoreRemoval) errored: {}",
        env.eval::<String>("return tostring(TEST_PICKUP_ACTION_ERR)")
            .unwrap()
    );
    assert!(!env.eval::<bool>("return HasAction(1)").unwrap());

    // Retail and PTR project mode 3 as ("spell", book slot, book type,
    // spell ID, ...) (WM46, WM56, R72); other profiles keep ("spell", spell ID).
    let projection = if cfg!(feature = "client-retail") || cfg!(feature = "client-ptr") {
        "local kind, _, _, spellID = GetCursorInfo(); return kind, spellID"
    } else {
        "return GetCursorInfo()"
    };
    let (cursor_type, cursor_spell_id): (String, i32) = env.eval(projection).unwrap();
    assert_eq!(cursor_type, "spell");
    assert_eq!(cursor_spell_id, 853);

    env.exec("PlaceAction(1)").unwrap();

    assert!(env.eval::<bool>("return HasAction(1)").unwrap());
    assert!(env.eval::<bool>("return GetCursorInfo() == nil").unwrap());
    assert!(
        env.eval::<bool>("return type(C_ActionBar.GetActionTexture(1)) == 'string'")
            .unwrap()
    );
}

#[test]
fn pickup_action_updates_action_button_icon_immediately() {
    wrath_action_case!(pickup_icon);

    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(1.0)])
            .unwrap();

        let before_drag: bool = env
            .eval("return ActionButton1.icon:IsShown() and HasAction(1)")
            .unwrap();
        assert!(
            before_drag,
            "action button should show its icon before PickupAction"
        );

        env.exec("PickupAction(1, false)").unwrap();

        let after_pickup: bool = env
            .eval("return (not ActionButton1.icon:IsShown()) and not HasAction(1)")
            .unwrap();
        assert!(
            after_pickup,
            "PickupAction should fire ACTIONBAR_SLOT_CHANGED so the source icon hides immediately"
        );

        env.exec("PlaceAction(1)").unwrap();

        let after_place: bool = env
            .eval("return ActionButton1.icon:IsShown() and HasAction(1)")
            .unwrap();
        assert!(
            after_place,
            "PlaceAction should fire ACTIONBAR_SLOT_CHANGED so the icon restores immediately"
        );
    });
}

#[test]
fn action_button_drag_round_trip_keeps_spell_visible() {
    wrath_action_case!(drag_round_trip);

    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        // The round-trip asserts only the verified empty-map branch.
        {
            let mut sim = env.state().borrow_mut();
            sim.action_bar_spell_type_restrictions.clear();
            sim.spell_type_labels.clear();
        }
        assert_action_button_template_has_receive_drag();
        let button_id = env
            .state()
            .borrow()
            .widgets
            .get_id_by_name("ActionButton1")
            .unwrap();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec("if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end")
            .unwrap();

        let before_drag: bool = env
            .eval(
                "return ActionButton1.icon:IsShown() and ActionButton1.icon:GetTexture() == 135963",
            )
            .unwrap();
        assert!(
            before_drag,
            "action button should show spell texture 135963 before drag"
        );
        let has_receive_drag: bool = env
            .eval("return ActionButton1:GetScript('OnReceiveDrag') ~= nil")
            .unwrap();
        assert!(
            has_receive_drag,
            "action button should have an OnReceiveDrag handler"
        );

        env.fire_script_handler(button_id, "OnDragStart", vec![])
            .unwrap();
        if cfg!(feature = "client-ptr") {
            env.exec(
                r#"
                assert(select('#', GetCursorInfo()) == 5)
                local kind, slot, book, spellID, relatedID = GetCursorInfo()
                assert(kind == "spell")
                assert(slot == (C_SpellBook.FindSpellBookSlotForSpell(853) or 0))
                assert(book == "spell")
                assert(spellID == 853)
                assert(relatedID == nil)
                "#,
            )
            .unwrap();
        }
        env.fire_script_handler(button_id, "OnReceiveDrag", vec![])
            .unwrap();

        let (texture, action_texture, icon_shown, has_action): (String, String, bool, bool) = env
            .eval(
                "return tostring(ActionButton1.icon:GetTexture()), \
                 tostring(GetActionTexture(1)), ActionButton1.icon:IsShown(), HasAction(1)",
            )
            .unwrap();
        assert_eq!(
            (
                texture.as_str(),
                action_texture.as_str(),
                icon_shown,
                has_action
            ),
            ("135963", "135963", true, true),
            "dragging off and back onto the same button should preserve the spell texture and action"
        );
    });
}

#[test]
fn action_button_1_texture_path_resolves_to_icon_fdid() {
    wrath_action_case!(icon_identity);

    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec("if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end")
            .unwrap();

        let result: String = env
            .eval(
                r#"
                if not ActionButton1 then
                    return "missing_action_button_1"
                end
                if not ActionButton1.icon then
                    return "missing_action_button_1_icon"
                end

                local actionTexture = GetActionTexture(1)
                if actionTexture ~= "ICONS/Spell_Holy_SealOfMight" then
                    return string.format(
                        "action_texture_mismatch_expected_%s_actual_%s",
                        "ICONS/Spell_Holy_SealOfMight",
                        tostring(actionTexture)
                    )
                end

                local iconTexture = ActionButton1.icon:GetTexture()
                if iconTexture ~= 135963 then
                    return string.format(
                        "icon_fdid_mismatch_expected_%s_actual_%s",
                        tostring(135963),
                        tostring(iconTexture)
                    )
                end

                return "ok"
            "#,
            )
            .unwrap();

        assert_eq!(
            result, "ok",
            "ActionButton1 texture path should resolve to the expected icon FDID: {result}"
        );
    });
}
