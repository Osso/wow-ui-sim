//! Modeled interaction queries consumed by Forever's unchanged gamepad icon resolver.
#![cfg(feature = "client-wowforever")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn unit_speed_stationary_and_translation_lifecycle() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local current, run, flight, swim = GetUnitSpeed("player")
        assert(current == 0 and run == 7 and flight == 7 and swim == 4.722222)
        MoveForwardStart()
        assert(IsPlayerMoving())
        current, run, flight, swim = GetUnitSpeed("self")
        assert(current == 7 and run == 7 and flight == 7 and swim == 4.722222)
        TargetUnit("player")
        assert(select(1, GetUnitSpeed("target")) == 7)
        MoveForwardStop()
        assert(not IsPlayerMoving())
        assert(select(1, GetUnitSpeed("player")) == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn unit_speed_prefers_swim_then_flight_and_only_moves_when_moving() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetFlying(true)
        assert(select(1, GetUnitSpeed("player")) == 0)
        A_Admin.SetMoving(true)
        assert(select(1, GetUnitSpeed("player")) == 7)
        A_Admin.SetSwimming(true)
        assert(select(1, GetUnitSpeed("player")) == 4.722222)
        A_Admin.SetSwimming(false)
        assert(select(1, GetUnitSpeed("player")) == 7)
        A_Admin.SetFlying(false)
        assert(select(1, GetUnitSpeed("player")) == 7)
        A_Admin.SetMoving(false)
        assert(select(1, GetUnitSpeed("player")) == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn unit_speed_reads_configured_capabilities() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state.player.movement_speeds.run = 9.0;
        state.player.movement_speeds.flight = 15.0;
        state.player.movement_speeds.swim = 5.0;
    }
    env.exec(
        r#"
        local current, run, flight, swim = GetUnitSpeed("player")
        assert(current == 0 and run == 9 and flight == 15 and swim == 5)
        A_Admin.SetMoving(true)
        assert(select(1, GetUnitSpeed("player")) == 9)
        A_Admin.SetFlying(true)
        assert(select(1, GetUnitSpeed("player")) == 15)
        A_Admin.SetSwimming(true)
        current, run, flight, swim = GetUnitSpeed("player")
        assert(current == 5 and run == 9 and flight == 15 and swim == 5)
    "#,
    )
    .unwrap();
}

#[test]
fn unit_speed_unknown_and_nonplayer_tokens_have_no_player_speed() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetMoving(true)
        A_Admin.SetTarget("Other", 70, 2, false)
        for _, token in ipairs({"unknown", "target", "pet", "party1"}) do
            local current, run, flight, swim = GetUnitSpeed(token)
            assert(current == 0 and run == 0 and flight == 0 and swim == 0, token)
        end
        assert(not pcall(GetUnitSpeed))
        assert(not pcall(GetUnitSpeed, {}))
    "#,
    )
    .unwrap();
}

#[test]
fn unit_speed_survives_post_cleanup_restoration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(select(2, GetUnitSpeed('player')) == 7)")
        .unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec("MoveForwardStart(); assert(select(1, GetUnitSpeed('player')) == 7)")
        .unwrap();
}

fn consumer() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_GamepadActionBars/MainActionBarFrame.lua");
    env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    env.exec(
        r#"
        local f = CreateFrame("Frame")
        icon = f:CreateTexture()
        local expectedIcon = f:CreateTexture()
        function AssertIcon(texture)
            expectedIcon:SetTexture(texture)
            assert(icon:GetTexture() == expectedIcon:GetTexture(),
                tostring(icon:GetTexture()) .. " ~= " .. tostring(expectedIcon:GetTexture()))
        end
        bar = { PageUnit = { actionBars = { topBar = { Right = {
            ActionButton1 = { SpecialActionIcon = icon }
        } } } } }
        Mixin(bar, GamepadMainActionBarFrameMixin)
        GamepadSharedUtility = { InputBindingManager = {
            IsOnlyCoreBindingSetActive = function() return true end
        } }
        EventRegistry = { TriggerEvent = function() end }
        SetUnitCursorTexture = function() return false end
        SetPreferredGamepadInteractTarget = function(unit) preferred = unit end
    "#,
    )
    .unwrap();
    env
}

#[test]
fn unknown_interactions_and_vendor_no_target() {
    let env = consumer();
    env.exec(
        r#"
        for _, unit in ipairs({"target", "softinteract", "missing"}) do
            assert(UnitIsGameObject(unit) == false)
            assert(UnitHasLootInteraction(unit) == false)
            assert(UnitIsInInteractRange(unit) == false)
            assert(UnitIsInteractable(unit) == false)
        end
        bar:UpdateInteractIcons()
        AssertIcon(C_Spell.GetSpellTexture(6603))
        assert(preferred == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn resolved_interaction_state_drives_vendor_icon_branches() {
    let env = consumer();
    env.exec("TargetUnit('enemy1')").unwrap();
    env.exec("bar:UpdateInteractIcons(); AssertIcon(C_Spell.GetSpellTexture(6603)); assert(preferred == nil)").unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let target = state.current_target.as_mut().unwrap();
        target.interaction.has_loot = true;
        target.interaction.in_range = true;
    }
    env.exec(
        r#"
        bar:UpdateInteractIcons()
        AssertIcon("Interface\\Cursor\\LootAll")
        assert(preferred == "target")
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .interaction
        .in_range = false;
    env.exec(
        r#"
        bar:UpdateInteractIcons()
        AssertIcon("Interface\\Cursor\\UnableLootAll")
    "#,
    )
    .unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let target = state.current_target.as_mut().unwrap();
        target.interaction.has_loot = false;
        target.interaction.interactable = true;
        target.interaction.in_range = true;
    }
    env.exec(
        r#"
        bar:UpdateInteractIcons()
        AssertIcon("Interface\\Cursor\\Interact")
        assert(preferred == "target")
    "#,
    )
    .unwrap();
    env.state().borrow_mut().soft_interact_target = Some("target".into());
    env.exec("assert(UnitExists('softinteract')); assert(UnitIsInteractable('softinteract'))")
        .unwrap();
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .interaction
        .is_game_object = true;
    env.exec(
        r#"
        assert(not UnitExists("softinteract"))
        assert(UnitIsGameObject("softinteract"))
        bar:UpdateInteractIcons()
        AssertIcon("Interface\\Cursor\\Interact")
        assert(preferred == "softinteract")
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .current_target
        .as_mut()
        .unwrap()
        .interaction
        .has_loot = true;
    env.exec(
        r#"
        bar:UpdateInteractIcons()
        AssertIcon("Interface\\Cursor\\LootAll")
        assert(preferred == "softinteract")
    "#,
    )
    .unwrap();
    env.exec("ClearTarget(); assert(not UnitIsGameObject('softinteract')); assert(not UnitHasLootInteraction('softinteract')); bar:UpdateInteractIcons(); assert(preferred == nil)").unwrap();
}

#[test]
fn focus_snapshot_queries_are_read_only_and_aliases_follow_selection() {
    let env = consumer();
    env.exec("TargetUnit('enemy1')").unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let interaction = &mut state.current_target.as_mut().unwrap().interaction;
        interaction.is_game_object = true;
        interaction.has_loot = true;
        interaction.interactable = true;
        interaction.in_range = true;
        state.current_focus = state.current_target.clone();
        state.soft_interact_target = Some("focus".into());
        state.current_target = None;
    }
    env.exec(
        r#"
        for _, token in ipairs({"focus", "softinteract"}) do
            assert(UnitIsGameObject(token) and UnitHasLootInteraction(token))
            assert(UnitIsInteractable(token) and UnitIsInInteractRange(token))
        end
        assert(not UnitIsGameObject(nil) and not UnitIsInteractable(nil))
    "#,
    )
    .unwrap();
    assert!(
        env.state()
            .borrow()
            .current_focus
            .as_ref()
            .unwrap()
            .interaction
            .has_loot
    );
    env.state().borrow_mut().current_focus = None;
    env.exec("assert(not UnitIsGameObject('softinteract')); assert(not UnitIsInInteractRange('softinteract'))").unwrap();
}
