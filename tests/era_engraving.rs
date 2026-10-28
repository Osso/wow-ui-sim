//! Era/Anniversary supplied availability at the original XML-bound OnLoad.
#![cfg(any(feature = "client-era", feature = "client-anniversary"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_original_control(enabled: Option<bool>) -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.state().borrow_mut().engraving.enabled = enabled;
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_SharedXMLBase",
            "Blizzard_SharedXML",
            "Blizzard_SharedXMLGame",
            "Blizzard_LoadLocale",
            "Blizzard_CharacterFrame",
        ],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_CharacterFrame"));
    env.exec(
        r#"
        assert(RuneFrameControlButton, 'selected Vanilla XML creates original control')
        assert(RuneFrameControlButton:GetScript('OnLoad'), 'original XML binds OnLoad')
        local info = debug.getinfo(RuneFrameControlButton_OnLoad, 'S')
        local suffix = 'Blizzard_CharacterFrame/Vanilla/PaperDollFrame.lua'
        local source = info.source:gsub('\\', '/')
        assert(source:sub(-#suffix) == suffix and info.linedefined == 950, source)
        print('original engraving OnLoad', source, info.linedefined)
    "#,
    )
    .expect("selected original Vanilla control and callback");
    eprintln!(
        "original load diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env
}

fn assert_control_visibility(env: &WowLuaEnv, enabled: bool) {
    env.state().borrow_mut().engraving.enabled = Some(enabled);
    env.exec(
        r#"
        RuneFrameControlButton:Show()
        RuneFrameControlButton:GetScript('OnLoad')(RuneFrameControlButton)
    "#,
    )
    .expect("XML-bound original OnLoad consumes supplied availability");
    let shown: bool = env.eval("return RuneFrameControlButton:IsShown()").unwrap();
    assert_eq!(
        shown, enabled,
        "original OnLoad must not choose constant false"
    );
    let (arity, actual): (i32, bool) = env.eval(
        "local function pack(...) return select('#', ...), ... end; return pack(C_Engraving.IsEngravingEnabled())",
    ).unwrap();
    assert_eq!((arity, actual), (1, enabled));
}

fn assert_stock_default(env: &WowLuaEnv) {
    env.state().borrow_mut().engraving.enabled = None;
    env.exec("RuneFrameControlButton:Show(); RuneFrameControlButton:GetScript('OnLoad')(RuneFrameControlButton)")
        .expect("original OnLoad accepts stock availability");
    assert!(
        !env.eval::<bool>("return RuneFrameControlButton:IsShown()")
            .unwrap()
    );
    assert!(
        !env.eval::<bool>("return C_Engraving.IsEngravingEnabled()")
            .unwrap()
    );
}

fn exercise_original_onload(initial: bool) {
    let first = load_original_control(Some(initial));
    let initially_shown: bool = first
        .eval("return RuneFrameControlButton:IsShown()")
        .unwrap();
    assert_eq!(initially_shown, initial, "original XML load-time OnLoad");
    assert_control_visibility(&first, initial);
    let second = load_original_control(Some(!initial));
    assert_control_visibility(&second, !initial);
    assert_control_visibility(&first, !initial);
    assert_control_visibility(&second, initial);
    assert_stock_default(&first);
    assert_control_visibility(&first, true);
    assert_eq!(second.state().borrow().engraving.enabled, Some(initial));
    let unset = load_original_control(None);
    let recorded = unset.state().borrow().lua_errors.clone();
    assert!(
        !recorded
            .iter()
            .any(|error| error.contains("C_Engraving.IsEngravingEnabled")),
        "original load-time availability: {recorded:?}"
    );
    assert_stock_default(&unset);
    assert_control_visibility(&unset, false);
}

#[test]
fn original_xml_onload_enabled_first_transitions_isolation_and_stock_default() {
    exercise_original_onload(true);
}

#[test]
fn original_xml_onload_disabled_first_transitions_isolation_and_stock_default() {
    exercise_original_onload(false);
}

#[test]
fn original_settings_closure_preservation_boundary() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.state().borrow_mut().engraving.enabled = Some(false);
    load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_SharedXMLBase",
            "Blizzard_SharedXML",
            "Blizzard_SharedXMLGame",
            "Blizzard_LoadLocale",
            "Blizzard_SettingsDefinitions_Frame",
        ],
        &[],
    );
    eprintln!(
        "settings closure diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(
        r#"
        local info = debug.getinfo(InterfaceOverrides.AdjustDisplaySettings, 'S')
        local source = info.source:gsub('\\', '/')
        local suffix = 'Blizzard_SettingsDefinitions_Frame/Classic/InterfaceOverrides.lua'
        assert(source:sub(-#suffix) == suffix and info.linedefined == 3, source)
        local category = Settings.RegisterVerticalLayoutCategory('Engraving preservation')
        local ok, boundary = pcall(InterfaceOverrides.AdjustDisplaySettings, category)
        print('settings preservation outcome', ok, tostring(boundary))
        assert(ok == true, tostring(boundary))
    "#,
    )
    .expect("attempt complete original selected settings consumer without substitutes");
}
