//! Concrete modeled volume state and retail retirement boundaries.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
local ns = C_UIWidgetManager
assert(rawget(ns, 'GetTextureWithStateVisualizationInfo') == nil)
assert(ns.GetTextureWithStateVisualizationInfo == nil, 'removed widget lookup')
assert(ns.GetTextureWithStateVisualizationInfo == nil, 'removed widget repeat')
"#;

pub(crate) const VOLUME_ASSERTIONS: &str = r#"
for _, scale in ipairs({0, 0.37, 1, 0.62}) do
    assert(select('#', C_VoiceChat.SetMasterVolumeScale(scale)) == 0)
    assert(C_VoiceChat.GetMasterVolumeScale() == scale, 'volume round trip')
end
for _, invalid in ipairs({-0.01, 1.01, math.huge, -math.huge, 0/0, {}}) do
    assert(not pcall(C_VoiceChat.SetMasterVolumeScale, invalid), 'invalid scale accepted')
    assert(C_VoiceChat.GetMasterVolumeScale() == 0.62, 'invalid scale mutated state')
end
assert(not pcall(C_VoiceChat.SetMasterVolumeScale), 'missing scale accepted')
"#;

#[test]
fn patch_8_2_0_master_volume_round_trip() {
    let env = WowLuaEnv::new().unwrap();
    let output_before = env.state().borrow().voice_chat.output_volume;
    env.exec(VOLUME_ASSERTIONS).unwrap();
    assert_eq!(env.state().borrow().voice_chat.master_volume_scale, 0.62);
    assert_eq!(env.state().borrow().voice_chat.output_volume, output_before);
}

#[test]
fn patch_8_2_0_colon_handler_probe_uses_declared_owner() {
    let env = WowLuaEnv::new().unwrap();
    let entry = sweep::Entry {
        id: "Frame:OnShow".into(),
        section: "widgets".into(),
        direction: "added".into(),
        symbol: "Frame:OnShow".into(),
        page_default: None,
        kind: Some("widget-script".into()),
    };
    let result = sweep::probe_entry(&env, &entry, false, &std::collections::BTreeMap::new());
    assert_eq!(result.0, "widget-script");
    assert_eq!(result.1, "Frame HasScript=true");
    assert!(result.2);
}

#[test]
fn patch_8_2_0_unused_widget_member_stays_absent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
