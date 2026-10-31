#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_insecure_addon(env: &WowLuaEnv) {
    let dir = tempfile::tempdir().unwrap();
    let addon = dir.path().join("ProtectedGlobalsProbe");
    std::fs::create_dir(&addon).unwrap();
    let toc = addon.join("ProtectedGlobalsProbe.toc");
    std::fs::write(&toc, "## Title: ProtectedGlobalsProbe\nprobe.lua\n").unwrap();
    std::fs::write(
        addon.join("probe.lua"),
        r#"
        function ProtectedGlobalsTarget() return TargetUnit('player') end
        function ProtectedGlobalsSecurecall() return securecall(TargetUnit, 'player') end
        function ProtectedGlobalsBind(action) return SetBinding('CTRL-F12', action) end
    "#,
    )
    .unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).expect("load tainted addon");
}

#[test]
fn protected_globals_consumer() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_Game"],
        &[],
        |env, _| {
            env.exec(
                r#"
                ClearTarget()
                protectedEvents = {}
                local f = CreateFrame('Frame')
                f:RegisterEvent('ADDON_ACTION_FORBIDDEN')
                f:RegisterEvent('ADDON_ACTION_BLOCKED')
                f:SetScript('OnEvent', function(_, event, origin, name)
                    table.insert(protectedEvents, {event, origin, name})
                end)
            "#,
            )
            .unwrap();
            load_insecure_addon(env);
            env.exec(
                r#"
                assert(select('#', ProtectedGlobalsTarget()) == 0)
                assert(not UnitExists('target'), 'insecure TargetUnit changed target')
                assert(protectedEvents[1][1] == 'ADDON_ACTION_FORBIDDEN')
                assert(protectedEvents[1][2] == 'ProtectedGlobalsProbe')
                assert(protectedEvents[1][3] == 'TargetUnit()')
                assert(StaticPopup_Visible('ADDON_ACTION_FORBIDDEN'), 'Blizzard popup missing')
            "#,
            )
            .expect("insecure targeting and popup");
            env.exec(
                r#"
                TargetUnit('player')
                assert(UnitIsUnit('target', 'player'), 'secure targeting failed')
                ClearTarget()
                ProtectedGlobalsSecurecall()
                assert(not UnitExists('target'), 'securecall laundered taint')
                assert(protectedEvents[2][1] == 'ADDON_ACTION_FORBIDDEN')
                assert(protectedEvents[2][2] == 'ProtectedGlobalsProbe')
                assert(protectedEvents[2][3] == 'TargetUnit()')
                ProtectedGlobalsBind('TOGGLEGAMEMENU')
                assert(GetBindingAction('CTRL-F12') == 'TOGGLEGAMEMENU')
            "#,
            )
            .expect("protected and out-of-combat behavior");
            env.state().borrow_mut().player.in_combat = true;
            env.exec(
                r#"
                assert(select('#', ProtectedGlobalsBind('TOGGLEBACKPACK')) == 0)
                assert(GetBindingAction('CTRL-F12') == 'TOGGLEGAMEMENU')
                assert(protectedEvents[3][1] == 'ADDON_ACTION_BLOCKED')
                assert(protectedEvents[3][2] == 'ProtectedGlobalsProbe')
                assert(protectedEvents[3][3] == 'SetBinding()')
            "#,
            )
            .expect("in-combat refusal");
        },
    );
}
