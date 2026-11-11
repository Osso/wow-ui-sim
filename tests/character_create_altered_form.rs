#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn character_create_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("character-create environment");
    env.set_screen_mode(ScreenKind::Game);
    env.exec(
        r#"
        assert(rawget(C_CharacterCreation, 'SetViewingAlteredForm') == nil)
        assert(rawget(C_CharacterCreation, 'IsViewingAlteredForm') == nil)
        "#,
    )
    .expect("altered-form methods are not installed on Game");
    env.set_screen_mode(ScreenKind::CharacterCreate);
    env
}

#[test]
fn altered_form_methods_are_glue_only_and_use_lua_truthiness() {
    let env = character_create_env();
    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .insert(1);
    env.exec(
        r#"
        local set = rawget(C_CharacterCreation, 'SetViewingAlteredForm')
        local isViewing = rawget(C_CharacterCreation, 'IsViewingAlteredForm')
        assert(type(set) == 'function' and type(isViewing) == 'function')
        assert(select('#', isViewing()) == 1 and isViewing() == false)
        local usage = "bad argument #1 to 'SetViewingAlteredForm' (Usage: C_CharacterCreation.SetViewingAlteredForm(isViewingAlteredForm))"
        for _, call in ipairs({
            function() set() end,
            function() set(nil) end,
        }) do
            local ok, message = pcall(call)
            assert(not ok and message:find(usage, 1, true), tostring(message))
        end
        assert(set(0) == nil and isViewing() == true, 'zero is truthy')
        assert(select('#', set({})) == 0 and isViewing() == true, 'table is truthy')
        set(false)
        assert(isViewing() == false)
        "#,
    )
    .expect("altered-form Glue API registration, usage text, truthiness and return count");
}

#[test]
fn altered_form_requires_a_host_supplied_model() {
    let env = character_create_env();
    env.exec(
        r#"
        C_CharacterCreation.SetViewingAlteredForm(true)
        assert(C_CharacterCreation.IsViewingAlteredForm() == false)
        rawset(_G, '__wow_selected_race_id', nil)
        assert(C_CharacterCreation.IsViewingAlteredForm() == false)
        rawset(_G, '__wow_selected_race_id', 1)
        "#,
    )
    .expect("no model or no selected race reports false");

    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .insert(1);
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == false)")
        .expect("a missing-model set request stores false");

    env.exec("C_CharacterCreation.SetViewingAlteredForm(0)")
        .expect("set altered form for a supplied model");
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == true)")
        .expect("the host-supplied model exposes the stored choice");
}

#[test]
fn altered_form_state_tracks_character_create_screen_and_selection_setters() {
    let env = character_create_env();
    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .extend([1, 2]);
    env.exec(
        r#"
        C_CharacterCreation.SetViewingAlteredForm(true)
        C_CharacterCreation.SetSelectedRace(1)
        assert(C_CharacterCreation.IsViewingAlteredForm() == true, 'same race keeps value')
        C_CharacterCreation.SetSelectedSex(1)
        assert(C_CharacterCreation.IsViewingAlteredForm() == true, 'sex rebuild keeps altered-form value')
        C_CharacterCreation.SetSelectedClass(2)
        assert(C_CharacterCreation.IsViewingAlteredForm() == true, 'class rebuild keeps altered-form value')
        C_CharacterCreation.ResetCharCustomize()
        assert(C_CharacterCreation.IsViewingAlteredForm() == true, 'customization rebuild keeps altered-form value')
        C_CharacterCreation.SetSelectedRace(2)
        assert(C_CharacterCreation.IsViewingAlteredForm() == false, 'actual race change clears value')
        C_CharacterCreation.SetViewingAlteredForm(true)
        "#,
    )
    .expect("race, sex and class state transitions");

    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .remove(&2);
    env.exec(
        r#"
        C_CharacterCreation.SetSelectedSex(0)
        assert(C_CharacterCreation.IsViewingAlteredForm() == false)
        "#,
    )
    .expect("sex rebuild clears stored state when the race has no altered model");
    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .insert(2);
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == false)")
        .expect("the cleared value stays false if the host model returns");
    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .remove(&2);
    env.exec(
        "C_CharacterCreation.SetViewingAlteredForm(true); C_CharacterCreation.ResetCharCustomize()",
    )
    .expect("customization rebuild without an altered model");
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == false)")
        .expect("customization rebuild clears state without an altered model");

    env.exec("C_CharacterCreation.SetViewingAlteredForm(true)")
        .expect("restore altered-form choice");
    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .remove(&2);
    env.exec("C_CharacterCreation.SetSelectedClass(1)")
        .expect("class rebuild on race without altered model");
    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .insert(2);
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == false)")
        .expect("class rebuild clears the stored value without an altered model");

    env.state()
        .borrow_mut()
        .character_create_altered_form_races
        .insert(2);
    env.exec("C_CharacterCreation.SetViewingAlteredForm(true)")
        .expect("set state before screen initialization");
    env.set_screen_mode(ScreenKind::CharacterCreate);
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == false)")
        .expect("CharacterCreate screen initialization clears the value");
    env.exec("C_CharacterCreation.SetViewingAlteredForm(true)")
        .expect("set state before mode entry");
    env.exec("C_CharacterCreation.SetInCharacterCreate(true)")
        .expect("enter character-create mode");
    env.exec("assert(C_CharacterCreation.IsViewingAlteredForm() == false)")
        .expect("mode entry clears the value");
}

#[test]
fn set_blur_enabled_stores_required_lua_truthiness_without_a_getter() {
    let env = character_create_env();
    env.set_screen_mode(ScreenKind::Game);
    let game_api_absent: bool = env
        .eval("return rawget(C_CharacterCreation, 'SetBlurEnabled') == nil")
        .expect("Game screen API should be queryable");
    assert!(game_api_absent);

    env.set_screen_mode(ScreenKind::CharacterSelect);
    env.exec(
        r#"
        local set = rawget(C_CharacterCreation, 'SetBlurEnabled')
        assert(type(set) == 'function')
        assert(rawget(C_CharacterCreation, 'GetBlurEnabled') == nil)
        local usage = "Usage: C_CharacterCreation.SetBlurEnabled(enabled)"
        for _, call in ipairs({ function() set() end, function() set(nil) end }) do
            local ok, message = pcall(call)
            assert(not ok and message:find("bad argument #1", 1, true), tostring(message))
            assert(message:find(usage, 1, true), tostring(message))
        end
        "#,
    )
    .expect("SetBlurEnabled registration and required input errors");
    assert!(!env.state().borrow().character_create_blur_enabled);

    env.exec(
        r#"
        local set = C_CharacterCreation.SetBlurEnabled
        assert(select('#', set(0)) == 0)
        assert(select('#', set({})) == 0)
        "#,
    )
    .expect("zero and tables are truthy and the setter returns no values");
    assert!(env.state().borrow().character_create_blur_enabled);

    env.exec("assert(select('#', C_CharacterCreation.SetBlurEnabled('')) == 0)")
        .expect("empty strings are truthy and the setter returns no values");
    assert!(env.state().borrow().character_create_blur_enabled);

    env.exec("assert(select('#', C_CharacterCreation.SetBlurEnabled(false)) == 0)")
        .expect("false disables the selected camera profile");
    assert!(!env.state().borrow().character_create_blur_enabled);
}
