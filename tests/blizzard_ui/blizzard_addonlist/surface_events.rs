//! EventRegistry surface for `Blizzard_AddOnList`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_AddOnList";
const FRAME_HIDDEN_EVENT: &str = "AddonList.FrameHidden";

// Wrath has no AddonList vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn addon_list_on_hide_triggers_frame_hidden_callback() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        register_frame_hidden_observer(env);
        trigger_addon_list_on_hide(env);

        let call_count = frame_hidden_callback_count(env);

        assert_eq!(
            call_count, 1,
            "`AddonList:OnHide()` must trigger one `{FRAME_HIDDEN_EVENT}` EventRegistry callback"
        );
    });
}
}

fn register_frame_hidden_observer(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.eval::<()>(
        r#"
        _G.__addon_list_frame_hidden_call_count = 0
        EventRegistry:RegisterCallback(
            "AddonList.FrameHidden",
            function()
                _G.__addon_list_frame_hidden_call_count =
                    _G.__addon_list_frame_hidden_call_count + 1
            end,
            "addon_list_frame_hidden_observer"
        )
        return
        "#,
    )
    .expect("AddonList.FrameHidden callback registration must run cleanly");
}

fn trigger_addon_list_on_hide(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.eval::<()>("AddonList:OnHide(); return")
        .expect("AddonList:OnHide() must run cleanly");
}

fn frame_hidden_callback_count(env: &wow_ui_sim::lua_api::WowLuaEnv) -> i64 {
    env.eval("return _G.__addon_list_frame_hidden_call_count")
        .expect("AddonList.FrameHidden callback count probe must run cleanly")
}
