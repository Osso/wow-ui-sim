//! Temporary UIParent OnUpdate worklist guard.
//!
//! Some global OnUpdate handlers assume their worklists are populated. Keep the
//! empty-worklist guards isolated until the corresponding startup state is
//! modeled well enough that Blizzard's handlers can run unwrapped.

use crate::lua_api::WowLuaEnv;

const FCF_ONUPDATE_DEFAULT_LUA: &str = r#"
if FCF_OnUpdate == nil then
    function FCF_OnUpdate()
    end
end
"#;

const UIPARENT_ONUPDATE_WORKLISTS_WORKAROUND_LUA: &str = r#"
if type(FCF_OnUpdate) == "function" and rawget(_G, "__wow_fcf_onupdate_wrapper") ~= FCF_OnUpdate then
    local original_fcf_onupdate = FCF_OnUpdate
    local wrapper = function(elapsed)
        if type(CHAT_FRAMES) == "table" and next(CHAT_FRAMES) == nil then
            return
        end
        return original_fcf_onupdate(elapsed)
    end
    FCF_OnUpdate = wrapper
    rawset(_G, "__wow_fcf_onupdate_wrapper", wrapper)
end

if type(ButtonPulse_OnUpdate) == "function" and rawget(_G, "__wow_button_pulse_onupdate_wrapper") ~= ButtonPulse_OnUpdate then
    local original_button_pulse_onupdate = ButtonPulse_OnUpdate
    local wrapper = function(elapsed)
        if type(PULSEBUTTONS) == "table" and next(PULSEBUTTONS) == nil then
            return
        end
        return original_button_pulse_onupdate(elapsed)
    end
    ButtonPulse_OnUpdate = wrapper
    rawset(_G, "__wow_button_pulse_onupdate_wrapper", wrapper)
end

if type(AnimatedShine_OnUpdate) == "function" and rawget(_G, "__wow_animated_shine_onupdate_wrapper") ~= AnimatedShine_OnUpdate then
    local original_animated_shine_onupdate = AnimatedShine_OnUpdate
    local wrapper = function(elapsed)
        if type(SHINES_TO_ANIMATE) == "table" and next(SHINES_TO_ANIMATE) == nil then
            return
        end
        return original_animated_shine_onupdate(elapsed)
    end
    AnimatedShine_OnUpdate = wrapper
    rawset(_G, "__wow_animated_shine_onupdate_wrapper", wrapper)
end

if type(UIParent) == "table"
    and type(UIParent.GetScript) == "function"
    and type(UIParent.SetScript) == "function" then
    local wrapper = rawget(_G, "__wow_ui_parent_onupdate_worklist_wrapper")
    if wrapper == nil or UIParent:GetScript("OnUpdate") ~= wrapper then
        wrapper = function(self, elapsed)
            if type(FCF_OnUpdate) == "function"
                and (type(CHAT_FRAMES) ~= "table" or next(CHAT_FRAMES) ~= nil) then
                FCF_OnUpdate(elapsed)
            end
            if type(ButtonPulse_OnUpdate) == "function"
                and (type(PULSEBUTTONS) ~= "table" or next(PULSEBUTTONS) ~= nil) then
                ButtonPulse_OnUpdate(elapsed)
            end
            if type(AnimatedShine_OnUpdate) == "function"
                and (type(SHINES_TO_ANIMATE) ~= "table" or next(SHINES_TO_ANIMATE) ~= nil) then
                AnimatedShine_OnUpdate(elapsed)
            end
            if type(HelpOpenWebTicketButton_OnUpdate) == "function" then
                HelpOpenWebTicketButton_OnUpdate(HelpOpenWebTicketButton, elapsed)
            end
        end
        UIParent:SetScript("OnUpdate", wrapper)
        rawset(_G, "__wow_ui_parent_onupdate_worklist_wrapper", wrapper)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    // VERIFIED: selected ChatFrameBase Mainline Lua owns this update function.
    // Its registered callback executes after the provider finishes loading.
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }
    // INFERRED: retain compatibility on profiles without audited provider timing.
    lua.exec(FCF_ONUPDATE_DEFAULT_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(UIPARENT_ONUPDATE_WORKLISTS_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_fcf_onupdate_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(FCF_OnUpdate) ~= "function" then return "missing" end
                if FCF_OnUpdate(0.1) ~= nil then return "return" end
                return "ok"
                "#,
            )
            .expect("FCF_OnUpdate default should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn global_onupdate_wrappers_skip_empty_worklists() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_onupdate_globals(&env);

        patch(&env);

        let (empty_count, populated_count): (i64, i64) = env
            .eval(
                r#"
                FCF_OnUpdate(0.1)
                ButtonPulse_OnUpdate(0.1)
                AnimatedShine_OnUpdate(0.1)
                local emptyCount = #calls

                CHAT_FRAMES = { "chat" }
                PULSEBUTTONS = { "pulse" }
                SHINES_TO_ANIMATE = { "shine" }
                FCF_OnUpdate(0.1)
                ButtonPulse_OnUpdate(0.1)
                AnimatedShine_OnUpdate(0.1)
                return emptyCount, #calls
                "#,
            )
            .expect("wrapped global OnUpdate calls should be readable");

        assert_eq!(empty_count, 0);
        assert_eq!(populated_count, 3);
    }

    #[test]
    fn uiparent_wrapper_fans_out_only_non_empty_worklists() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_onupdate_globals(&env);
        env.exec(
            r#"
            HelpOpenWebTicketButton = {}
            HelpOpenWebTicketButton_OnUpdate = function(button, elapsed)
                table.insert(calls, "help")
            end
            UIParent = {
                scripts = {},
                GetScript = function(self, event)
                    return self.scripts[event]
                end,
                SetScript = function(self, event, script)
                    self.scripts[event] = script
                end,
            }
            "#,
        )
        .expect("UIParent OnUpdate surface should install");

        patch(&env);

        let (empty_count, populated_count): (i64, i64) = env
            .eval(
                r#"
                UIParent.scripts.OnUpdate(UIParent, 0.1)
                local emptyCount = #calls

                CHAT_FRAMES = { "chat" }
                PULSEBUTTONS = { "pulse" }
                SHINES_TO_ANIMATE = { "shine" }
                UIParent.scripts.OnUpdate(UIParent, 0.1)
                return emptyCount, #calls
                "#,
            )
            .expect("UIParent OnUpdate wrapper calls should be readable");

        assert_eq!(empty_count, 1);
        assert_eq!(populated_count, 5);
    }

    #[test]
    fn uiparent_wrapper_skips_missing_global_handlers() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_onupdate_globals(&env);
        env.exec(
            r#"
            ButtonPulse_OnUpdate = nil
            AnimatedShine_OnUpdate = nil
            UIParent = {
                scripts = {},
                GetScript = function(self, event)
                    return self.scripts[event]
                end,
                SetScript = function(self, event, script)
                    self.scripts[event] = script
                end,
            }
            "#,
        )
        .expect("UIParent OnUpdate missing-handler surface should install");

        patch(&env);

        let call_count: i64 = env
            .eval(
                r#"
                CHAT_FRAMES = { "chat" }
                PULSEBUTTONS = { "pulse" }
                SHINES_TO_ANIMATE = { "shine" }
                UIParent.scripts.OnUpdate(UIParent, 0.1)
                return #calls
                "#,
            )
            .expect("UIParent OnUpdate wrapper should tolerate missing handlers");

        assert_eq!(call_count, 1);
    }

    fn install_onupdate_globals(env: &WowLuaEnv) {
        env.exec(
            r#"
            calls = {}
            CHAT_FRAMES = {}
            PULSEBUTTONS = {}
            SHINES_TO_ANIMATE = {}
            FCF_OnUpdate = function(elapsed)
                table.insert(calls, "chat")
            end
            ButtonPulse_OnUpdate = function(elapsed)
                table.insert(calls, "pulse")
            end
            AnimatedShine_OnUpdate = function(elapsed)
                table.insert(calls, "shine")
            end
            "#,
        )
        .expect("OnUpdate globals should install");
    }
}
