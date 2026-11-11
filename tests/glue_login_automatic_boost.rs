#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

#[test]
fn glue_automatic_boost_preserves_independent_session_values_and_clearers() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.exec(r#"
        local boost = C_CharacterServices.GetAutomaticBoost()
        assert(boost == nil) -- Absence return count is only inferred.
        assert(C_CharacterServices.GetAutomaticBoostCharacter() == nil)
        assert(select('#', C_CharacterServices.GetAutomaticBoostCharacter()) == 1)
        boostEvents = 0
        local f = CreateFrame('Frame')
        f:RegisterEvent('CHARACTER_LIST_UPDATE')
        f:RegisterEvent('LOGIN_STATE_CHANGED')
        f:SetScript('OnEvent', function() boostEvents = boostEvents + 1 end)
        C_CharacterServices.SetAutomaticBoost(0)
        assert(C_CharacterServices.GetAutomaticBoost('ignored') == 0)
        assert(select('#', C_CharacterServices.GetAutomaticBoost()) == 1)
        C_CharacterServices.SetAutomaticBoostCharacter('Player-1-00000001')
        assert(C_CharacterServices.GetAutomaticBoostCharacter('ignored') == 'Player-1-00000001')
        C_CharacterServices.SetAutomaticBoost(nil)
        boost = C_CharacterServices.GetAutomaticBoost()
        assert(boost == nil and C_CharacterServices.GetAutomaticBoostCharacter() == 'Player-1-00000001')
        C_CharacterServices.SetAutomaticBoost(5)
        C_CharacterServices.SetAutomaticBoostCharacter(nil)
        assert(C_CharacterServices.GetAutomaticBoost() == 5 and C_CharacterServices.GetAutomaticBoostCharacter() == nil)
        C_CharacterServices.SetAutomaticBoostCharacter('Player-1-00000001')
        Logout() -- Shipped class-trial/store consumers require persistence here.
        assert(C_CharacterServices.GetAutomaticBoost() == 5)
        CancelLogout()
        boost = C_CharacterServices.GetAutomaticBoost()
        assert(boost == nil and C_CharacterServices.GetAutomaticBoostCharacter() == 'Player-1-00000001')
        assert(boostEvents == 0)
        C_CharacterServices.SetAutomaticBoost(6)
    "#).unwrap();
    env.set_screen_mode(ScreenKind::Login);
    env.exec("assert(C_CharacterServices.GetAutomaticBoost() == 6 and C_CharacterServices.GetAutomaticBoostCharacter() == 'Player-1-00000001')").unwrap();
    env.state()
        .borrow_mut()
        .character_services
        .on_realm_disconnect();
    env.exec("local boost = C_CharacterServices.GetAutomaticBoost(); assert(boost == nil); assert(C_CharacterServices.GetAutomaticBoostCharacter() == 'Player-1-00000001')").unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn glue_automatic_boost_keeps_existing_protected_refusals_outside_and_inside_combat() {
    let env = common::lock_env(|| WowLuaEnv::new().unwrap());
    env.exec(
        r#"
        C_CharacterServices.SetAutomaticBoost(5)
        C_CharacterServices.SetAutomaticBoostCharacter('Player-1-00000001')
        refusedBoostCalls = {}
        local f = CreateFrame('Frame')
        f:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        f:SetScript('OnEvent', function(_, event, source, name)
            assert(source == 'BoostRefusalProbe')
            table.insert(refusedBoostCalls, name)
        end)
    "#,
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let addon = dir.path().join("BoostRefusalProbe");
    std::fs::create_dir(&addon).unwrap();
    let toc = addon.join("BoostRefusalProbe.toc");
    std::fs::write(&toc, "## Title: BoostRefusalProbe\nprobe.lua\n").unwrap();
    std::fs::write(addon.join("probe.lua"), r#"
        function BoostReadType() return C_CharacterServices.GetAutomaticBoost() end
        function BoostReadGUID() return C_CharacterServices.GetAutomaticBoostCharacter() end
        function BoostWriteType() return C_CharacterServices.SetAutomaticBoost(99) end
        function BoostWriteGUID() return C_CharacterServices.SetAutomaticBoostCharacter('Player-1-00000002') end
    "#).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    for combat in [false, true] {
        env.state().borrow_mut().player.in_combat = combat;
        env.exec(
            r#"
            for index, call in ipairs({BoostReadType, BoostReadGUID, BoostWriteType, BoostWriteGUID}) do
                local count = select('#', call())
                assert(count == 0, 'protected call ' .. index .. ' returned ' .. count .. ' values')
            end
        "#,
        )
        .unwrap();
        // Do not query protected getters in the now-tainted caller chunk.
        let sim = env.state();
        let sim = sim.borrow();
        assert_eq!(sim.character_services.automatic_boost_type, Some(5));
        assert_eq!(
            sim.character_services
                .automatic_boost_character_guid
                .as_deref(),
            Some("Player-1-00000001")
        );
    }
    env.exec(
        r#"
        assert(#refusedBoostCalls == 8)
        assert(refusedBoostCalls[1] == 'C_CharacterServices.GetAutomaticBoost()')
        assert(refusedBoostCalls[2] == 'C_CharacterServices.GetAutomaticBoostCharacter()')
        assert(refusedBoostCalls[3] == 'C_CharacterServices.SetAutomaticBoost()')
        assert(refusedBoostCalls[4] == 'C_CharacterServices.SetAutomaticBoostCharacter()')
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
