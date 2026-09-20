use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn gamepad_stick_lua_aliases_share_binding_and_dispatch_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        StickFrame = CreateFrame('Frame', 'StickFrame')
        local function original(self, stick, x, y)
            assert(self == StickFrame and stick == 2 and x == 0.25 and y == -0.75)
            StickCalls = (StickCalls or '') .. 'original;'
        end
        StickFrame:SetScript('OnGamepadStick', original)
        assert(StickFrame:GetScript('OnGamePadStick') == original)
        assert(StickFrame:HasScript('OnGamePadStick'))
        assert(StickFrame:HasScript('OnGamepadStick'))
        StickFrame:HookScript('OnGamePadStick', function(self, stick, x, y)
            assert(self == StickFrame and stick == 2 and x == 0.25 and y == -0.75)
            StickCalls = StickCalls .. 'hook;'
        end)
        assert(StickFrame:GetScript('OnGamepadStick') == StickFrame:GetScript('OnGamePadStick'))
        assert(not pcall(function() StickFrame:SetScript('ongamepadstick', original) end))
    "#,
    )
    .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("StickFrame")
        .unwrap();
    for name in ["OnGamePadStick", "OnGamepadStick"] {
        env.fire_script_handler(
            id,
            name,
            vec![
                rilua::Val::Num(2.0),
                rilua::Val::Num(0.25),
                rilua::Val::Num(-0.75),
            ],
        )
        .unwrap();
    }
    assert_eq!(
        env.eval::<String>("return StickCalls").unwrap(),
        "original;hook;original;hook;"
    );
    env.exec("StickFrame:SetScript('OnGamePadStick', nil); assert(StickFrame:GetScript('OnGamepadStick') == nil)").unwrap();
}

#[test]
fn gamepad_stick_xml_binding_is_visible_through_lua_alias() {
    let env = WowLuaEnv::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Stick.toc"), "Stick.xml\n").unwrap();
    std::fs::write(dir.path().join("Stick.xml"), r#"<Ui><Frame name="XmlStick"><Scripts><OnGamePadStick>XmlStickArgs = {self, ...}</OnGamePadStick></Scripts></Frame></Ui>"#).unwrap();
    load_addon(&env.loader_env(), &dir.path().join("Stick.toc")).unwrap();
    env.exec("assert(type(XmlStick:GetScript('OnGamepadStick')) == 'function')")
        .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("XmlStick")
        .unwrap();
    env.fire_script_handler(
        id,
        "OnGamepadStick",
        vec![
            rilua::Val::Num(1.0),
            rilua::Val::Num(-0.5),
            rilua::Val::Num(0.125),
        ],
    )
    .unwrap();
    env.exec("assert(XmlStickArgs[1] == XmlStick and XmlStickArgs[2] == 1 and XmlStickArgs[3] == -0.5 and XmlStickArgs[4] == 0.125)").unwrap();
}
