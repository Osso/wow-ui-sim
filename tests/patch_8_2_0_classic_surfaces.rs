//! Retail-only additions/retirements preserve Mists' existing surface.
#![cfg(feature = "client-mists")]

#[test]
fn patch_8_2_0_mists_widget_and_volume_preserved() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.state().borrow_mut().voice_chat.master_volume_scale = 0.41;
    env.exec(r#"
        assert(type(C_UIWidgetManager.GetTextureWithStateVisualizationInfo) == 'function')
        assert(C_VoiceChat.GetMasterVolumeScale() == 0.41)
    "#).unwrap();
}
