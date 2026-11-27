//! Temporary `C_ConfigurationWarnings` defaults.
//!
//! Configuration warning state is not modeled yet. Keep the empty warning list
//! and local seen cache explicit here until a real settings/system-warning
//! backend owns this namespace.

const CONFIGURATION_WARNINGS_DEFAULTS_LUA: &str = r#"
C_ConfigurationWarnings = C_ConfigurationWarnings or __wow_namespace()
if type(rawget(C_ConfigurationWarnings, "__wow_seen_warnings")) ~= "table" then
    C_ConfigurationWarnings.__wow_seen_warnings = {}
end

local function installConfigurationWarningsDefault(name, fn)
    if rawget(C_ConfigurationWarnings, name) == nil then
        C_ConfigurationWarnings[name] = fn
    end
end

local function warningKey(warning)
    if warning == nil then
        return nil
    end
    return tostring(warning)
end

installConfigurationWarningsDefault("GetConfigurationWarningSeen", function(warning)
    local key = warningKey(warning)
    local seenWarnings = rawget(C_ConfigurationWarnings, "__wow_seen_warnings")
    return key ~= nil and seenWarnings[key] == true
end)

installConfigurationWarningsDefault("GetConfigurationWarningString", function()
    return nil
end)

installConfigurationWarningsDefault("GetConfigurationWarnings", function()
    return {}
end)

installConfigurationWarningsDefault("SetConfigurationWarningSeen", function(warning)
    local key = warningKey(warning)
    if key ~= nil then
        local seenWarnings = rawget(C_ConfigurationWarnings, "__wow_seen_warnings")
        seenWarnings[key] = true
    end
end)

if HasCheckedSystemRequirements == nil
    or SetCheckedSystemRequirements == nil
    or CheckSystemRequirements == nil
then
    local checkedSystemRequirements = false

    if HasCheckedSystemRequirements == nil then
        function HasCheckedSystemRequirements()
            return checkedSystemRequirements
        end
    end

    if SetCheckedSystemRequirements == nil then
        function SetCheckedSystemRequirements(checked)
            checkedSystemRequirements = not not checked
        end
    end

end
"#;

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const CHECK_SYSTEM_REQUIREMENTS_FALLBACK_LUA: &str = r#"
if CheckSystemRequirements == nil then
    function CheckSystemRequirements(includeSeenWarnings)
        if type(C_ConfigurationWarnings.GetConfigurationWarnings) ~= "function" then
            return
        end

        local warnings = C_ConfigurationWarnings.GetConfigurationWarnings(includeSeenWarnings)
        if type(warnings) ~= "table" then
            return
        end

        if type(StaticPopup_Queue) ~= "function" then
            return
        end

        for _, warning in ipairs(warnings) do
            local text = nil
            if type(C_ConfigurationWarnings.GetConfigurationWarningString) == "function" then
                text = C_ConfigurationWarnings.GetConfigurationWarningString(warning)
            end
            if text then
                StaticPopup_Queue("CONFIGURATION_WARNING", text, nil, { configurationWarning = warning })
            end
        end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CONFIGURATION_WARNINGS_DEFAULTS_LUA)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(CHECK_SYSTEM_REQUIREMENTS_FALLBACK_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    fn installs_empty_configuration_warning_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, Option<String>, bool) = env
            .eval(
                r#"
                local warnings = C_ConfigurationWarnings.GetConfigurationWarnings(false)
                return type(warnings),
                    C_ConfigurationWarnings.GetConfigurationWarningString("warning"),
                    C_ConfigurationWarnings.GetConfigurationWarningSeen("warning")
                "#,
            )
            .expect("configuration warning defaults should be callable");

        assert_eq!(result, ("table".to_string(), None, false));
    }

    // The Rust provider (14a0d85002) shadows this fallback on every profile;
    // seen state belongs to supplied warnings with numeric IDs.
    #[test]
    fn marks_only_the_selected_supplied_warning_seen() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let seen: (bool, bool, bool) = env
            .eval(
                r#"
                A_Admin.SetConfigurationWarnings({
                    { warning = 3, text = "Driver" },
                    { warning = 5, text = "Memory" },
                })
                local before = C_ConfigurationWarnings.GetConfigurationWarningSeen(3)
                C_ConfigurationWarnings.SetConfigurationWarningSeen(3)
                return before,
                    C_ConfigurationWarnings.GetConfigurationWarningSeen(3),
                    C_ConfigurationWarnings.GetConfigurationWarningSeen(5)
                "#,
            )
            .expect("configuration warning seen state should be callable");

        assert_eq!(seen, (false, true, false));
    }

    #[test]
    fn preserves_existing_configuration_warning_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_ConfigurationWarnings.GetConfigurationWarnings()
                return { "existing" }
            end
            "#,
        )
        .expect("fixture should install existing configuration warning provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let warning: String = env
            .eval("return C_ConfigurationWarnings.GetConfigurationWarnings()[1]")
            .expect("existing configuration warning provider should remain callable");

        assert_eq!(warning, "existing");
    }

    #[test]
    fn stores_checked_system_requirements_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let checked: (bool, bool, bool) = env
            .eval(
                r#"
                local before = HasCheckedSystemRequirements()
                SetCheckedSystemRequirements(true)
                local afterTrue = HasCheckedSystemRequirements()
                SetCheckedSystemRequirements(false)
                return before, afterTrue, HasCheckedSystemRequirements()
                "#,
            )
            .expect("system requirements checked state should be callable");

        assert_eq!(checked, (false, true, false));
    }

    #[test]
    fn check_system_requirements_queues_warnings_with_text() {
        if cfg!(any(
            feature = "client-retail",
            feature = "client-wowforever"
        )) {
            return;
        }
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            queued = {}
            C_ConfigurationWarnings.GetConfigurationWarnings = function(includeSeenWarnings)
                includeSeenArg = includeSeenWarnings
                return { "missing-driver", "silent-warning" }
            end
            C_ConfigurationWarnings.GetConfigurationWarningString = function(warning)
                if warning == "missing-driver" then
                    return "Driver missing"
                end
                return nil
            end
            StaticPopup_Queue = function(which, text, data, info)
                queued[#queued + 1] = {
                    which = which,
                    text = text,
                    data = data,
                    warning = info and info.configurationWarning,
                }
            end
            "#,
        )
        .expect("configuration warning queue fixture should install");

        let result: (bool, i64, String, String, Option<String>) = env
            .eval(
                r#"
                CheckSystemRequirements(true)
                return includeSeenArg,
                    #queued,
                    queued[1].which,
                    queued[1].text,
                    queued[2] and queued[2].text
                "#,
            )
            .expect("system requirements should queue warnings with text");

        assert_eq!(
            result,
            (
                true,
                1,
                "CONFIGURATION_WARNING".to_string(),
                "Driver missing".to_string(),
                None
            )
        );
    }
}
