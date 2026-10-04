//! Behavior pin: ACTIONBAR_SHOW_BOTTOMLEFT waits for settings initialization.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBarController";

#[test]
fn show_bottomleft_defers_until_settings_loaded_then_sets_immediately() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        env.exec(
            r#"
            local originalContinueAfterAllEvents = EventUtil.ContinueAfterAllEvents

            _G.bottomLeftSettingExists = false
            _G.bottomLeftSettingValue = false
            _G.bottomLeftDeferredCalls = 0

            function Settings.GetSetting(name)
                if name == "PROXY_SHOW_ACTIONBAR_2" and _G.bottomLeftSettingExists then
                    return true
                end
                return nil
            end

            function Settings.SetValue(name, value)
                if name == "PROXY_SHOW_ACTIONBAR_2" then
                    _G.bottomLeftSettingValue = value
                end
            end

            function Settings.SetOnValueChangedCallback() end
            function MultiActionBar_Update() end
            StatusTrackingBarManager.UpdateBarTicks = function() end
            -- Silence UI callback events only: Blizzard's EventUtil routes
            -- frame events (SETTINGS_LOADED) through EventRegistry:TriggerEvent.
            local originalTriggerEvent = EventRegistry.TriggerEvent
            EventRegistry.TriggerEvent = function(registry, event, ...)
                if C_EventUtils.IsEventValid(event) then
                    return originalTriggerEvent(registry, event, ...)
                end
            end

            function EventUtil.ContinueAfterAllEvents(callback, ...)
                _G.bottomLeftDeferredCalls = _G.bottomLeftDeferredCalls + 1
                return originalContinueAfterAllEvents(callback, ...)
            end

            FireEvent("ACTIONBAR_SHOW_BOTTOMLEFT")
            "#,
        )
        .expect("pre-SETTINGS_LOADED ACTIONBAR_SHOW_BOTTOMLEFT dispatch must run cleanly");

        let (deferred_calls, value_before_settings): (i32, bool) = env
            .eval(
                r#"
                return _G.bottomLeftDeferredCalls,
                    _G.bottomLeftSettingValue
                "#,
            )
            .expect("pre-SETTINGS_LOADED probe must run cleanly");

        assert_eq!(
            deferred_calls, 1,
            "ACTIONBAR_SHOW_BOTTOMLEFT before SETTINGS_LOADED must register one deferred callback"
        );
        assert!(
            !value_before_settings,
            "ACTIONBAR_SHOW_BOTTOMLEFT before SETTINGS_LOADED must not set the bar setting yet"
        );

        env.exec(
            r#"
            _G.bottomLeftSettingExists = true
            FireEvent("SETTINGS_LOADED")
            "#,
        )
        .expect("SETTINGS_LOADED dispatch must run cleanly");

        let value_after_settings: bool = env
            .eval("return _G.bottomLeftSettingValue")
            .expect("post-SETTINGS_LOADED setting probe must run cleanly");

        assert!(
            value_after_settings,
            "SETTINGS_LOADED must run the deferred ShowBottomLeftBar callback"
        );

        env.exec(
            r#"
            _G.bottomLeftSettingValue = false
            _G.bottomLeftDeferredCalls = 0

            FireEvent("ACTIONBAR_SHOW_BOTTOMLEFT")
            "#,
        )
        .expect("post-SETTINGS_LOADED ACTIONBAR_SHOW_BOTTOMLEFT dispatch must run cleanly");

        let (immediate_deferred_calls, immediate_value): (i32, bool) = env
            .eval(
                r#"
                return _G.bottomLeftDeferredCalls,
                    _G.bottomLeftSettingValue
                "#,
            )
            .expect("post-SETTINGS_LOADED immediate probe must run cleanly");

        assert_eq!(
            immediate_deferred_calls, 0,
            "ACTIONBAR_SHOW_BOTTOMLEFT after SETTINGS_LOADED must not register another deferred callback"
        );
        assert!(
            immediate_value,
            "ACTIONBAR_SHOW_BOTTOMLEFT after SETTINGS_LOADED must set the bar setting immediately"
        );
    });
    }
}
