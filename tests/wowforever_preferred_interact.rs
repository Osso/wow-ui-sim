//! Preferred interaction selection driven by unchanged Forever vendor code.
#![cfg(feature = "client-wowforever")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn vendor_selects_interaction_identity_and_clears_preference() {
    let env = WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_GamepadActionBars/MainActionBarFrame.lua");
    env.exec(&std::fs::read_to_string(source).unwrap()).unwrap();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        bar = { PageUnit = { actionBars = { topBar = { Right = {
            ActionButton1 = { SpecialActionIcon = frame:CreateTexture() }
        } } } } }
        Mixin(bar, GamepadMainActionBarFrameMixin)
        coreBindings = true
        GamepadSharedUtility = { InputBindingManager = {
            IsOnlyCoreBindingSetActive = function() return coreBindings end
        } }
        EventRegistry = { TriggerEvent = function() end }
        SetUnitCursorTexture = function() return false end
        TargetUnit("enemy1")
    "#,
    )
    .unwrap();
    let identity = {
        let state = env.state();
        let mut state = state.borrow_mut();
        let target = state.current_target.as_mut().unwrap();
        target.interaction.has_loot = true;
        target.interaction.in_range = true;
        target.guid.clone()
    };
    env.exec("bar:UpdateInteractIcons()").unwrap();
    // An explicit host read observes the setter without inventing a Lua getter.
    assert_eq!(
        env.state()
            .borrow()
            .preferred_gamepad_interact_guid
            .as_deref(),
        Some(identity.as_str())
    );
    env.exec("coreBindings = false; bar:UpdateInteractIcons()")
        .unwrap();
    assert_eq!(env.state().borrow().preferred_gamepad_interact_guid, None);
    env.exec("coreBindings = true; bar:UpdateInteractIcons()")
        .unwrap();
    env.exec("ClearTarget()").unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .preferred_gamepad_interact_guid
            .as_deref(),
        Some(identity.as_str())
    );
    env.exec("bar:UpdateInteractIcons()").unwrap();
    assert_eq!(env.state().borrow().preferred_gamepad_interact_guid, None);
}

#[test]
fn preferred_selection_resolves_alias_without_retargeting() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("TargetUnit('enemy1'); SetPreferredGamepadInteractTarget('target')")
        .unwrap();
    let selected = env
        .state()
        .borrow()
        .current_target
        .as_ref()
        .unwrap()
        .guid
        .clone();
    assert_eq!(
        env.state()
            .borrow()
            .preferred_gamepad_interact_guid
            .as_deref(),
        Some(selected.as_str())
    );
    env.state().borrow_mut().soft_interact_target = Some("target".into());
    env.exec("SetPreferredGamepadInteractTarget('softinteract')")
        .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .preferred_gamepad_interact_guid
            .as_deref(),
        Some(selected.as_str())
    );
    env.exec("SetPreferredGamepadInteractTarget(nil)").unwrap();
    assert_eq!(env.state().borrow().preferred_gamepad_interact_guid, None);
    assert_eq!(
        env.state().borrow().current_target.as_ref().unwrap().guid,
        selected
    );
    env.exec("SetPreferredGamepadInteractTarget('missing')")
        .unwrap();
    assert_eq!(env.state().borrow().preferred_gamepad_interact_guid, None);
}
