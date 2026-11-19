//! Behavior pin: action-button spell alerts transition from birth to loop anims.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ActionBar";

post_wrath_ui_fixture! {
#[test]
fn spell_alert_show_starts_birth_then_loop_and_hide_stops_both() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        create_spell_alert_button(env);
        assert_no_spell_alert(env);

        show_spell_alert(env);
        assert_birth_animation_playing(env);

        finish_birth_animation(env);
        assert_loop_animation_playing(env);

        hide_spell_alert(env);
        assert_alert_animations_stopped(env);
        assert_alert_frame_is_reused(env);
    });
    }
}
}

fn create_spell_alert_button(env: &WowLuaEnv) {
    env.exec(
        r#"
        SpellAlertAnimButton = CreateFrame("Button", "SpellAlertAnimButton", UIParent)
        SpellAlertAnimButton:SetSize(36, 36)
        SpellAlertAnimButton:SetPoint("CENTER")
        "#,
    )
    .expect("spell-alert button fixture must run cleanly");
}

fn assert_no_spell_alert(env: &WowLuaEnv) {
    let has_alert: bool = env
        .eval("return ActionButtonSpellAlertManager:HasAlert(SpellAlertAnimButton)")
        .expect("initial spell-alert state probe must run cleanly");
    assert!(!has_alert, "a new button must not start with an alert");
}

fn show_spell_alert(env: &WowLuaEnv) {
    env.eval::<()>("ActionButtonSpellAlertManager:ShowAlert(SpellAlertAnimButton)")
        .expect("ActionButtonSpellAlertManager:ShowAlert must run cleanly");
}

fn finish_birth_animation(env: &WowLuaEnv) {
    env.fire_on_update(0.8)
        .expect("spell-alert animation tick must run cleanly");
}

fn hide_spell_alert(env: &WowLuaEnv) {
    env.eval::<()>("ActionButtonSpellAlertManager:HideAlert(SpellAlertAnimButton)")
        .expect("ActionButtonSpellAlertManager:HideAlert must run cleanly");
}

fn assert_birth_animation_playing(env: &WowLuaEnv) {
    let (has_alert, is_default_type, start_playing, loop_playing): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local alert = SpellAlertAnimButton.SpellActivationAlert
            local hasAlert, alertType = ActionButtonSpellAlertManager:HasAlert(SpellAlertAnimButton)
            return hasAlert,
                alertType == ActionButtonSpellAlertManager.SpellAlertType.Default,
                alert.ProcStartAnim:IsPlaying(),
                alert.ProcLoop:IsPlaying()
            "#,
        )
        .expect("spell-alert birth probe must run cleanly");
    assert!(has_alert, "ShowAlert must mark the button active");
    assert!(is_default_type, "ShowAlert must use the original default alert type");
    assert!(start_playing, "ShowAlert must play ProcStartAnim");
    assert!(
        !loop_playing,
        "ProcLoop must wait for ProcStartAnim to finish"
    );
}

fn assert_loop_animation_playing(env: &WowLuaEnv) {
    let (start_playing, loop_playing): (bool, bool) = env
        .eval(
            r#"
            local alert = SpellAlertAnimButton.SpellActivationAlert
            return alert.ProcStartAnim:IsPlaying(), alert.ProcLoop:IsPlaying()
            "#,
        )
        .expect("spell-alert loop probe must run cleanly");
    assert!(
        !start_playing,
        "ProcStartAnim must stop after its birth animation finishes"
    );
    assert!(
        loop_playing,
        "ActionButtonSpellAlertMixin OnLoad must play ProcLoop after ProcStartAnim finishes"
    );
}

fn assert_alert_frame_is_reused(env: &WowLuaEnv) {
    let reused: bool = env
        .eval(
            r#"
            local alert = SpellAlertAnimButton.SpellActivationAlert
            ActionButtonSpellAlertManager:ShowAlert(SpellAlertAnimButton)
            return alert ~= nil
                and SpellAlertAnimButton.SpellActivationAlert == alert
                and alert:IsShown()
            "#,
        )
        .expect("spell-alert reuse probe must run cleanly");
    assert!(reused, "ShowAlert must reuse and show the original alert frame");
}

fn assert_alert_animations_stopped(env: &WowLuaEnv) {
    let (has_alert, alert_shown, start_playing, loop_playing): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local alert = SpellAlertAnimButton.SpellActivationAlert
            return ActionButtonSpellAlertManager:HasAlert(SpellAlertAnimButton),
                alert:IsShown(),
                alert.ProcStartAnim:IsPlaying(),
                alert.ProcLoop:IsPlaying()
            "#,
        )
        .expect("spell-alert hide probe must run cleanly");
    assert!(!has_alert, "HideAlert must clear the active alert");
    assert!(!alert_shown, "HideAlert must hide the alert frame");
    assert!(!start_playing, "HideAlert must stop ProcStartAnim");
    assert!(!loop_playing, "alert OnHide must stop ProcLoop");
}
