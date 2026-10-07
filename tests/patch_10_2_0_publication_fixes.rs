//! Bounded parent-key state and publication classifier proof, not native parity.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

pub(crate) const PARENT_KEY_ASSERTIONS: &str = r#"
local parent = CreateFrame('Frame')
local child = CreateFrame('Frame', nil, parent)
child:SetParentKey('First')
child:SetParentKey('Second', true)
assert(parent.First == nil and parent.Second == child)
assert(child:GetParentKey() == 'Second')
child:ClearParentKey()
assert(parent.Second == nil and child:GetParentKey() == nil)
child:ClearParentKey()
local other = CreateFrame('Frame', nil, parent)
other:SetParentKey('Second')
child:ClearParentKey()
assert(parent.Second == other)
local texture = parent:CreateTexture()
texture:SetParentKey('Texture')
texture:ClearParentKey()
assert(parent.Texture == nil and texture:GetParentKey() == nil)
"#;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
for _, name in ipairs({'GetFontHeight', 'PrintAllMatchingCommands', 'SetFontHeight'}) do
    assert(rawget(C_Console, name) == nil)
    assert(C_Console[name] == nil)
    assert(C_Console[name] == nil)
end
assert(type(ConsoleGetFontHeight) == 'function')
assert(type(ConsoleSetFontHeight) == 'function')
assert(type(ConsoleGetAllCommands) == 'function')
"#;

#[test]
fn patch_10_2_0_unused_console_members_are_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}

#[test]
fn patch_10_2_0_clear_parent_key_updates_lua_and_widget_state() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(PARENT_KEY_ASSERTIONS).unwrap();
}

#[test]
fn patch_10_2_0_interface_and_script_probes_do_not_use_invalid_frame_types() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let entry = |symbol: &str, kind: Option<&str>| sweep::Entry {
        id: symbol.into(),
        section: "widgets".into(),
        direction: "added".into(),
        symbol: symbol.into(),
        page_default: None,
        kind: kind.map(str::to_owned),
    };
    let aliases = std::collections::BTreeMap::new();
    let object = entry("Object:SetParentKey", None);
    assert!(sweep::probe_entry(&env, &object, false, &aliases).2);
    let script = entry("OnMovieHideSubtitle", Some("widget-script"));
    let result = sweep::probe_entry(&env, &script, true, &aliases);
    assert_eq!(result.0, "widget-script");
    assert_eq!(result.1, "MovieFrame HasScript=false");
    assert!(result.2);
}
