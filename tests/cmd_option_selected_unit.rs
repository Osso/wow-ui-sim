#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn cmd_option_returns_only_explicit_unit_from_the_selected_clause() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function check(text, expectedValue, expectedUnit)
            local function inspect(...)
                assert(select('#', ...) == (expectedValue == nil and 1 or 2))
                local value, unit = ...
                assert(value == expectedValue)
                assert(unit == expectedUnit)
            end
            inspect(SecureCmdOptionParse(text))
        end
        check('plain', 'plain', nil)
        check('[nocombat] 2', '2', nil)
        check('[] 3', '3', nil)
        check('[@focus] 4', '4', 'focus')
        check('[target=focus] 5', '5', 'focus')
        check('[unit=focus] 6', '6', 'focus')
        check('[@target] 7', '7', 'target')
        check('[@focus,combat] 1; fallback', 'fallback', nil)
        check('[combat] 1; [@player,exists] 8', '8', 'player')
        check('[@player,@focus] 1', '1', 'focus')
        check('[unknown] 1', nil, nil)
        check('', nil, nil)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().player.in_combat = true;
    env.exec(
        r#"
        local value, unit = SecureCmdOptionParse('[@focus,combat] 1; fallback')
        assert(value == '1' and unit == 'focus')
        "#,
    )
    .unwrap();
}

#[test]
fn cached_vendor_target_marker_handler_uses_selected_unit_and_default_target() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("TargetUnit('player')").unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let mut focus = sim.current_target.clone().unwrap();
        focus.guid = "Creature-0-1-2-3-448-000002".into();
        focus.name = "CmdOption focus fixture".into();
        sim.current_focus = Some(focus);
        // Enum.GameMode.Standard = 1; default mode 0 registers no vendor commands.
        sim.game_rules.active_game_mode = 1;
    }
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source =
        std::fs::read_to_string(ui.join("Blizzard_ChatFrameBase/Shared/SlashCommands.lua"))
            .expect("profile-scoped cached vendor SlashCommands.lua required");
    // Supply the addon's normal chunk varargs; do not replace registration or handlers.
    env.exec(&format!(
        "local function loadVendor(...)\n{source}\nend\nloadVendor('Blizzard_ChatFrameBase', {{ SecureCmdList = {{}} }})"
    ))
    .expect("load entire unchanged cached SlashCommands.lua");
    env.exec(
        r#"
        local handler = SlashCmdList[SLASH_COMMAND.TARGET_MARKER]
        assert(type(handler) == 'function')
        markerEvents = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('RAID_TARGET_UPDATE')
        listener:SetScript('OnEvent', function(_, event)
            assert(event == 'RAID_TARGET_UPDATE')
            markerEvents = markerEvents + 1
        end)
        handler('[@focus,exists] 5')
        assert(GetRaidTargetIndex('focus') == 5)
        assert(GetRaidTargetIndex('target') == nil)
        assert(markerEvents == 1)
        handler('[target=focus,exists] 6')
        assert(GetRaidTargetIndex('focus') == 6)
        assert(GetRaidTargetIndex('target') == nil)
        handler('[nocombat] 2')
        assert(GetRaidTargetIndex('target') == 2)
        assert(GetRaidTargetIndex('focus') == 6)
        assert(markerEvents == 3)
        handler('[@focus,combat] 7')
        assert(GetRaidTargetIndex('focus') == 6 and markerEvents == 3)
        handler('[@focus] 0')
        assert(GetRaidTargetIndex('focus') == nil)
        assert(GetRaidTargetIndex('target') == 2 and markerEvents == 4)
        "#,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
