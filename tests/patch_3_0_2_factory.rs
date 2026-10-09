//! Bare current-retail publication measurement; no historical/native/model parity.
#![cfg(feature = "client-retail")]
#[path = "common/publication_sweep.rs"]
mod publication_sweep;

use publication_sweep::{SweepSpec, run_factory_publication_sweep, run_publication_sweep};
use wow_ui_sim::lua_api::WowLuaEnv;

const REGISTER: &str = include_str!("../data/patch-api/sources/3.0.2-wikitext-register.json");
const SPEC: SweepSpec = SweepSpec {
    register: REGISTER,
    known_gaps: include_str!("../data/patch-api/evidence/3.0.2-factory-2026-10-09/known-gaps.json"),
    row_count: 373,
    register_env: "WOW_SIM_P302_FACTORY_REGISTER",
    out_env: "WOW_SIM_P302_FACTORY_OUT",
    later_registers: &[
        include_str!("../data/patch-api/sources/3.0.3-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.0.8-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.3.3-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.3.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.3.4-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.0.4-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.8-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.2.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.2.4-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.0.3-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.3.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.3.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.2.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.1.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.6-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.1.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.2.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
    ],
};

#[test]
fn patch_3_0_2_exact_factory_publication_gaps() {
    let env = WowLuaEnv::new().expect("bare current-retail Lua environment");
    eprintln!(
        "current-retail interface: {}",
        wow_ui_sim::client_profile::ACTIVE_INTERFACE_VERSION
    );
    run_factory_publication_sweep(&env, &SPEC);
}

#[test]
fn literal_register_keeps_all_373_original_occurrences() {
    let current: serde_json::Value = serde_json::from_str(REGISTER).unwrap();
    let original: serde_json::Value = serde_json::from_str(include_str!(
        "../data/patch-api/evidence/3.0.2-session-2026-10-09/original/register.json"
    ))
    .unwrap();
    assert_eq!(current, original);
    assert_eq!(current["entries"].as_array().unwrap().len(), 373);
}

#[test]
fn bare_framexml_and_tooltip_handler_have_precise_limits() {
    let env = WowLuaEnv::new().unwrap();
    let observed: (String, String, bool, String, String) = env
        .eval(
            r#"
        local tooltip = CreateFrame('GameTooltip')
        return type(rawget(_G, 'InterfaceOptionsFrame_OpenToPage')),
            type(rawget(_G, 'InterfaceOptionsFrame_OpenToCategory')),
            tooltip:HasScript('OnTooltipSetAchievement'),
            tooltip:GetObjectType(), type(CreateFrame('Button').GetFont)
    "#,
        )
        .unwrap();
    eprintln!("FrameXML/handler/shared-method control: {observed:?}");
    assert_eq!(
        observed,
        (
            "nil".into(),
            "nil".into(),
            false,
            "GameTooltip".into(),
            "function".into()
        )
    );
}

#[test]
fn click_modifier_default_does_not_discriminate_unknown_actions() {
    let env = WowLuaEnv::new().unwrap();
    let observed: (String, String) = env
        .eval("return GetModifiedClick('FOCUSCAST'), GetModifiedClick('P302_UNKNOWN_MODIFIER')")
        .unwrap();
    eprintln!("click modifier control: {observed:?}");
    assert_eq!(observed, ("NONE".into(), "NONE".into()));
}

#[test]
fn retail_event_registration_rejects_unknown_but_not_source_events() {
    let env = WowLuaEnv::new().unwrap();
    let observed: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
        local f = CreateFrame('Frame')
        local called, message = pcall(f.RegisterEvent, f, 'P302_NOT_A_NATIVE_EVENT')
        f:RegisterEvent('COMPANION_UPDATE')
        f:RegisterEvent('UNIT_THREAT_LIST_UPDATE')
        f:RegisterEvent('UNIT_THREAT_SITUATION_UPDATE')
        return called, string.find(tostring(message), 'Attempt to register unknown event', 1, true) ~= nil,
            f:IsEventRegistered('COMPANION_UPDATE'),
            f:IsEventRegistered('UNIT_THREAT_LIST_UPDATE'),
            f:IsEventRegistered('UNIT_THREAT_SITUATION_UPDATE')
    "#,
        )
        .unwrap();
    eprintln!("retail event catalog control: {observed:?}");
    assert_eq!(observed, (false, true, true, true, true));
}

#[test]
fn console_catalog_distinguishes_cvar_records_from_commands() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("C_CVar.RegisterCVar('P302_FACTORY_CVAR_CONTROL', '0')")
        .unwrap();
    let observed: (bool, bool, i32) = env
        .eval(
            r#"
        local cvar, command, commands = false, false, 0
        for _, record in ipairs(C_Console.GetAllCommands()) do
            if record.commandType == Enum.ConsoleCommandType.Command then
                commands = commands + 1
                if record.command == 'P302_FACTORY_CVAR_CONTROL' then command = true end
            elseif record.command == 'P302_FACTORY_CVAR_CONTROL' then
                cvar = true
            end
        end
        return cvar, command, commands
    "#,
        )
        .unwrap();
    eprintln!("console catalog control: {observed:?}");
    assert_eq!(observed, (true, false, 14));
}

#[test]
#[should_panic(expected = "register client line does not match active profile")]
fn factory_rejects_classic_before_reading_cache() {
    let env = WowLuaEnv::new().unwrap();
    let foreign = SweepSpec {
        register: r#"{"client_line":"wrath-classic","entries":[]}"#,
        known_gaps: "[]",
        row_count: 0,
        register_env: "WOW_SIM_P302_FOREIGN_REGISTER",
        out_env: "WOW_SIM_P302_FOREIGN_OUT",
        later_registers: &[],
    };
    run_publication_sweep(&env, &foreign);
}
