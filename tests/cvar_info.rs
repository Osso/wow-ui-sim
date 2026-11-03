#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "profile-retail")]
#[test]
fn console_autocomplete_displays_cvar_values_and_scopes() {
    let env = WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .unwrap()
        .join("Blizzard_Console/Blizzard_Console_AutoComplete.lua");
    env.loader_env()
        .exec(&std::fs::read_to_string(source).unwrap())
        .unwrap();
    env.exec(r#"
        local function color(name, r, g, b)
            return { GetRGB = function() return r, g, b end,
                WrapTextInColorCode = function(_, text) return name .. ':' .. text end }
        end
        GREEN_FONT_COLOR = color('green', 0, 1, 0)
        RED_FONT_COLOR = color('red', 1, 0, 0)
        BATTLENET_FONT_COLOR = color('blue', 0, 0, 1)
        YELLOW_FONT_COLOR = color('yellow', 1, 1, 0)
        CreateColor = function(r, g, b) return color('type', r, g, b) end
        Enum.ConsoleCommandType = { Cvar = 1, Macro = 2 }
        Enum.ConsoleCategory = { Debug = 1, Graphics = 2, Console = 3 }
        local function widget()
            return { SetText = function(self, text) self.text = text end,
                SetTextColor = function(self, r, g, b) self.rgb = {r, g, b} end,
                SetPoint = function() end, Show = function() end, Hide = function() end,
                SetShown = function() end, SetWidth = function() end,
                SetHeight = function() end, GetStringWidth = function() return 10 end,
                GetWrappedWidth = function() return 10 end,
                GetStringHeight = function() return 10 end }
        end
        local entry = widget()
        entry.Text, entry.Type, entry.Help, entry.Value, entry.Selected = widget(), widget(), widget(), widget(), widget()
        local tooltip = widget()
        tooltip.Text = widget()
        local console = setmetatable({
            bestResults = {}, BackgroundElements = {}, Background = widget(), Tooltip = tooltip,
            entryPool = { ReleaseAll = function() end, Acquire = function() return entry end },
            CalculateMaxEntriesToDisplay = function() return 10 end,
            GetEntryIndex = function() return 1 end,
        }, { __index = DeveloperConsoleAutoCompleteMixin })
        local function check(name, changed, expectedServer, expectedCharacter)
            local default = GetCVarDefault(name)
            assert(C_CVar.SetCVar(name, default))
            console.bestResults = {{ command = name, commandType = Enum.ConsoleCommandType.Cvar }}
            console:DisplayResults()
            assert(entry.Value.text == default and entry.Value.rgb[2] == 1)
            assert(C_CVar.SetCVar(name, changed))
            console:DisplayResults()
            assert(entry.Value.text == changed and entry.Value.rgb[1] == 1 and entry.Value.rgb[2] == 0)
            console:OnEntryEnter(entry)
            assert(tooltip.Text.text:find('Current: "' .. changed .. '"', 1, true))
            assert(tooltip.Text.text:find('Default: "' .. default .. '"', 1, true))
            assert(tooltip.Text.text:find('Is Server Stored: ' .. expectedServer, 1, true))
            assert(tooltip.Text.text:find('Is Per Character: ' .. expectedCharacter, 1, true))
        end
        check('showTimestamps', 'changed', 'green:true', 'red:false')
        check('nameplateMaxDistance', '61', 'red:false', 'green:true')
    "#).unwrap();
}

#[test]
fn cvar_info_returns_live_values_and_native_account_character_flags() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env.eval(r##"
        local info = rawget(C_CVar, "GetCVarInfo")
        assert(type(info) == "function")
        assert(select("#", info("__missing_cvar_info_probe")) == 0)
        local value, default, account, character, locked, secure, readonly = info("SHOWTIMESTAMPS")
        assert(select("#", info("showTimestamps")) == 7)
        assert(value == GetCVar("showTimestamps") and default == GetCVarDefault("showTimestamps"))
        assert(account == true and character == false)
        assert(type(locked) == "boolean" and type(secure) == "boolean" and type(readonly) == "boolean")
        local beforeDefault = GetCVarDefault("nameplateMaxDistance")
        assert(C_CVar.SetCVar("nameplateMaxDistance", "61"))
        value, default, account, character = info("nameplateMaxDistance")
        assert(value == "61" and default == beforeDefault)
        assert(account == false and character == true)
        return "61:character"
    "##).unwrap();
    assert_eq!(result, "61:character");
}

#[test]
fn cvar_info_observes_runtime_registration_and_rejects_invalid_arguments() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env.eval(r#"
        local info = C_CVar.GetCVarInfo
        assert(not pcall(info) and not pcall(info, {}) and not pcall(info, true))
        C_CVar.RegisterCVar("__cvar_info_registered", "initial")
        C_CVar.SetCVar("__cvar_info_registered", "changed")
        local value, default, account, character, locked, secure, readonly = info("__CVAR_INFO_REGISTERED")
        assert(value == "changed" and default == "initial")
        assert(account == false and character == false and locked == false and secure == false and readonly == false)
        -- This native account CVar is absent from the simulator's default-value
        -- catalog. Registering a value must preserve its known native scope.
        C_CVar.RegisterCVar("coordsByTenths", "0")
        local _, _, nativeAccount, nativeCharacter = info("coordsByTenths")
        assert(nativeAccount and not nativeCharacter)
        return value .. ":" .. default
    "#).unwrap();
    assert_eq!(result, "changed:initial");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn cvar_info_reports_effective_temporary_values_without_changing_defaults() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
        local name = "showTimestamps"
        local value, default = C_CVar.GetCVarInfo(name)
        C_CVar.SetTempCVar(name, "temporary")
        local current, currentDefault, account, character = C_CVar.GetCVarInfo(name)
        assert(current == "temporary" and currentDefault == default)
        assert(account and not character)
        C_CVar.RemoveTempCVar(name)
        assert(C_CVar.GetCVarInfo(name) == value)
        return current
    "#,
        )
        .unwrap();
    assert_eq!(result, "temporary");
}

#[test]
fn cvar_info_preserves_native_locked_secure_and_readonly_flags() {
    let env = WowLuaEnv::new().unwrap();
    let result: String = env.eval(r#"
        local _, _, account, character, locked, secure, readonly = C_CVar.GetCVarInfo("accountNeedsTurnStrafeDialog")
        assert(account and not character and locked and not secure and not readonly)
        _, _, account, character, locked, secure, readonly = C_CVar.GetCVarInfo("ActionButtonUseKeyDown")
        assert(account and not character and not locked and secure and not readonly)
        _, _, account, character, locked, secure, readonly = C_CVar.GetCVarInfo("portal")
        assert(not account and not character and not locked and not secure and readonly)
        return "locked:secure:readonly"
    "#).unwrap();
    assert_eq!(result, "locked:secure:readonly");
}
