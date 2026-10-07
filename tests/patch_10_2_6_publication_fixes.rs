//! Bounded current-retail retirement, not historical/native parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
    for _, path in ipairs({
        {'C_CameraDefaults', 'GetCameraFOVDefaults'},
        {'C_TaskQuest', 'GetUIWidgetSetIDFromQuestID'},
    }) do
        local namespace = _G[path[1]]
        for i = 1, 2 do
            assert(rawget(namespace, path[2]) == nil, path[1] .. '.' .. path[2])
            assert(namespace[path[2]] == nil, path[1] .. '.' .. path[2])
        end
    end
    for i = 1, 2 do
        assert(rawget(_G, 'CanSummonFriend') == nil)
        assert(CanSummonFriend == nil)
    end
    assert(type(GetCameraFOVDefaults) == 'function')
    assert(type(rawget(C_RecruitAFriend, 'CanSummonFriend')) == 'function')
"#;

#[test]
fn patch_10_2_6_unused_retirements_preserve_successors() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
