//! Integration tests for `src/lua_api/globals/set_cvar_verb.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(not(feature = "retail-12-0-7"))]
#[test]
fn patch_12_0_7_named_defaults_are_absent_before_publication() {
    env()
        .exec(
            r#"
            -- Control: public CVar state remains usable before 12.0.7.
            assert(GetCVarDefault('nameplateShowEnemies') == '1')
            assert(C_CVar.SetCVar('nameplateShowEnemies', '0') == true)
            assert(GetCVar('nameplateShowEnemies') == '0')
            assert(C_CVar.GetCVarBool('nameplateShowEnemies') == false)
            assert(SetCVar('nameplateShowEnemies', GetCVarDefault('nameplateShowEnemies')) == true)
            assert(C_CVar.GetCVar('nameplateShowEnemies') == '1')
            assert(GetCVarBool('nameplateShowEnemies') == true)
            local function checkAbsent(name)
                local function checkNil(...)
                    assert(select('#', ...) == 1, name .. ': one nil result')
                    assert((...) == nil, name .. ': built-in not published')
                end
                checkNil(GetCVar(name))
                checkNil(C_CVar.GetCVar(name))
                checkNil(GetCVarDefault(name))
                checkNil(C_CVar.GetCVarDefault(name))
                assert(GetCVarBool(name) == false, name .. ': global bool')
                assert(C_CVar.GetCVarBool(name) == false, name .. ': namespace bool')
            end
            for _, name in ipairs({
                'assistedCombatReduceHighlights',
                'developerLogFilterDebug',
                'developerLogFilterError',
                'developerLogFilterFatal',
                'developerLogFilterNormal',
                'developerLogFilterSpam',
            }) do
                checkAbsent(name)
                checkAbsent(string.upper(name))
            end
        "#,
        )
        .expect("six built-in defaults absent before their publication epoch");
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn set_cvar_stores_string_value() {
    let env = env();
    let ok: bool = env
        .eval(r#"return SetCVar("nameplateShowAll", "1")"#)
        .unwrap();
    assert!(ok);
    let value = env.state().borrow().cvars.get("nameplateShowAll");
    assert_eq!(value.as_deref(), Some("1"));
}

#[test]
fn set_cvar_accepts_numeric_value() {
    let env = env();
    env.exec(r#"SetCVar("nameplateMaxDistance", 60)"#).unwrap();
    let value = env.state().borrow().cvars.get("nameplateMaxDistance");
    assert_eq!(value.as_deref(), Some("60"));
}

#[test]
fn set_cvar_accepts_fractional_numeric_value() {
    let env = env();
    env.exec(r#"SetCVar("uiScale", 0.75)"#).unwrap();
    let value = env.state().borrow().cvars.get("uiScale");
    assert_eq!(value.as_deref(), Some("0.75"));
}

#[test]
fn set_cvar_accepts_boolean_as_one_zero() {
    let env = env();
    env.exec(r#"SetCVar("nameplateShowEnemies", true)"#)
        .unwrap();
    let value = env.state().borrow().cvars.get("nameplateShowEnemies");
    assert_eq!(value.as_deref(), Some("1"));
    env.exec(r#"SetCVar("nameplateShowEnemies", false)"#)
        .unwrap();
    let value = env.state().borrow().cvars.get("nameplateShowEnemies");
    assert_eq!(value.as_deref(), Some("0"));
}

#[test]
fn set_cvar_fires_cvar_update_after_storing_value() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            RegisterCVar("runtimeEventProbe", "old")
            local observed
            local frame = CreateFrame("Frame")
            frame:RegisterEvent("CVAR_UPDATE")
            frame:SetScript("OnEvent", function(_, event, name, value)
                if name == "runtimeEventProbe" then
                    observed = table.concat({ event, name, tostring(value), GetCVar(name) }, ":")
                end
            end)
            SetCVar("runtimeEventProbe", "new")
            return observed
            "#,
        )
        .unwrap();

    assert_eq!(result, "CVAR_UPDATE:runtimeEventProbe:new:new");
}

#[test]
fn set_cvar_empty_name_returns_false() {
    let env = env();
    let ok: bool = env.eval(r#"return SetCVar("", "1")"#).unwrap();
    assert!(!ok);
}

#[test]
fn c_cvar_get_reads_back_the_value_set_via_global() {
    let env = env();
    env.exec(r#"SetCVar("showTimestamps", "chat")"#).unwrap();
    let value: String = env
        .eval(r#"return C_CVar.GetCVar("showTimestamps")"#)
        .unwrap();
    assert_eq!(value, "chat");
}

#[test]
fn register_cvar_exposes_runtime_default_through_global_and_namespace() {
    let env = env();
    let (global_value, namespace_value, default_value): (String, String, String) = env
        .eval(
            r#"
            RegisterCVar("__codex_register_default", "1")
            return GetCVar("__codex_register_default"),
                   C_CVar.GetCVar("__codex_register_default"),
                   GetCVarDefault("__codex_register_default")
            "#,
        )
        .unwrap();
    assert_eq!(global_value, "1");
    assert_eq!(namespace_value, "1");
    assert_eq!(default_value, "1");
}

#[test]
fn register_cvar_accepts_finite_numeric_defaults_from_both_surfaces() {
    env()
        .exec(
            r#"
            for index, register in ipairs({ RegisterCVar, C_CVar.RegisterCVar }) do
                for _, entry in ipairs({
                    { 'zero', 0, '0' },
                    { 'positive', 17, '17' },
                    { 'fraction', 0.75, '0.75' },
                    { 'negative', -2.5, '-2.5' },
                }) do
                    local name = '__numeric_default_' .. index .. '_' .. entry[1]
                    register(name, entry[2])
                    assert(GetCVar(name) == entry[3], name .. ' current')
                    assert(C_CVar.GetCVar(name) == entry[3], name .. ' namespace current')
                    assert(GetCVarDefault(name) == entry[3], name .. ' default')
                    assert(C_CVar.GetCVarDefault(name) == entry[3], name .. ' namespace default')
                end
            end
            -- Datamine SettingsCore's actual call must register a readable string value.
            C_CVar.RegisterCVar('debugTargetInfo', 0)
            assert(C_CVar.GetCVar('debugTargetInfo') == '0')
            "#,
        )
        .unwrap();
}

#[test]
fn register_cvar_numeric_default_preserves_existing_override_and_first_default() {
    env()
        .exec(
            r#"
            for index, register in ipairs({ RegisterCVar, C_CVar.RegisterCVar }) do
                local name = '__numeric_override_' .. index
                SetCVar(name, '1.25')
                register(name, 0)
                assert(GetCVar(name) == '1.25')
                assert(GetCVarDefault(name) == '0')
                register(name, 6)
                assert(GetCVar(name) == '1.25')
                assert(GetCVarDefault(name) == '0')
            end
            "#,
        )
        .unwrap();
}

#[test]
fn register_cvar_rejects_nonfinite_and_unsupported_defaults() {
    env()
        .exec(
            r#"
            for index, register in ipairs({ RegisterCVar, C_CVar.RegisterCVar }) do
                for suffix, value in pairs({ bool = true, table = {}, infinite = math.huge }) do
                    local name = '__invalid_register_' .. index .. '_' .. suffix
                    local ok = pcall(register, name, value)
                    assert(not ok, name .. ' should reject unsupported default')
                    assert(GetCVar(name) == nil, name .. ' should remain unknown')
                end
            end
            "#,
        )
        .unwrap();
}

#[test]
fn register_cvar_makes_unknown_cvar_visible_with_zero_default() {
    let env = env();
    let (value, default, enabled): (String, String, bool) = env
        .eval(
            r#"
            RegisterCVar("__codex_register_zero_default")
            return GetCVar("__codex_register_zero_default"), GetCVarDefault("__codex_register_zero_default"), GetCVarBool("__codex_register_zero_default")
            "#,
        )
        .unwrap();
    assert_eq!(value, "0");
    assert_eq!(default, "0");
    assert!(!enabled);
}

#[test]
fn register_cvar_preserves_explicit_empty_default_lifecycle() {
    env()
        .exec(
            r#"
        for index, register in ipairs({RegisterCVar, C_CVar.RegisterCVar}) do
            local name = '__empty_default_lifecycle_' .. index
            assert(GetCVar(name) == nil)
            register(name, '')
            assert(GetCVar(name) == '', 'explicit empty default must stay empty')
            assert(C_CVar.GetCVar(name) == '')
            assert(GetCVarDefault(name) == '')
            assert(C_CVar.GetCVarDefault(name) == '')
            register(name, 'replacement')
            assert(GetCVar(name) == '', 'registration must preserve the first default')
            SetCVar(name, '1.25')
            register(name, '')
            assert(GetCVar(name) == '1.25', 'registration must preserve overrides')
            assert(C_CVar.GetCVarDefault(name) == '', 'override must not replace default')
        end
    "#,
        )
        .unwrap();
}

#[test]
fn register_cvar_nil_default_retains_zero_policy() {
    env().exec(r#"
        for index, register in ipairs({RegisterCVar, C_CVar.RegisterCVar}) do
            local name = '__nil_default_' .. index
            assert(GetCVar(name) == nil)
            register(name, nil)
            assert(GetCVar(name) == '0')
            assert(C_CVar.GetCVarDefault(name) == '0')
            register(name, '')
            assert(GetCVarDefault(name) == '0', 'empty registration cannot replace existing default')
        end
    "#).unwrap();
}

#[test]
fn empty_cvar_registration_preserves_classic_castbar_scale_default() {
    env()
        .exec(
            r#"
        -- ClassicCastBarForever's fresh-profile ReadCVar/GetCVarValue path.
        local function ReadCVar(name)
            if C_CVar and C_CVar.GetCVar then
                local ok, value = pcall(C_CVar.GetCVar, name)
                if ok and type(value) == 'string' and value ~= '' then return value end
            end
            if GetCVar then
                local ok, value = pcall(GetCVar, name)
                if ok and type(value) == 'string' and value ~= '' then return value end
            end
        end
        local function GetCVarValue(name)
            local value = ReadCVar(name)
            if value then return value end
            if C_CVar and C_CVar.RegisterCVar then
                pcall(C_CVar.RegisterCVar, name, '')
                return ReadCVar(name)
            end
        end
        local name = 'ClassicCastBarForever_scale'
        assert(GetCVar(name) == nil, 'fixture must start with an unknown CVar')
        local db = {scale = 1}
        local scale = GetCVarValue(name)
        if scale then db.scale = tonumber(scale) or db.scale end
        assert(db.scale == 1, 'fresh empty registration must retain addon scale default')
        local frame = CreateFrame('Frame')
        frame:SetScale(db.scale)
        assert(frame:GetScale() == 1)
        SetCVar(name, '1.25')
        scale = GetCVarValue(name)
        if scale then db.scale = tonumber(scale) or db.scale end
        frame:SetScale(db.scale)
        assert(frame:GetScale() == 1.25, 'persisted nonempty scale remains usable')
    "#,
        )
        .unwrap();
}

#[test]
fn c_cvar_register_cvar_sets_default_without_overwriting_existing_value() {
    let env = env();
    let (before, after): (String, String) = env
        .eval(
            r#"
            SetCVar("__codex_register_preserve_override", "1")
            C_CVar.RegisterCVar("__codex_register_preserve_override", "0")
            return GetCVar("__codex_register_preserve_override"), GetCVarDefault("__codex_register_preserve_override")
            "#,
        )
        .unwrap();
    assert_eq!(before, "1");
    assert_eq!(after, "0");
}

#[test]
fn legacy_cvar_compat_globals_exist_before_framexml_loads() {
    let env = env();
    let (register_cvar, get_bitfield, set_bitfield, reset_test): (String, String, String, String) =
        env.eval(
            r#"
            return type(RegisterCVar),
                   type(GetCVarBitfield),
                   type(SetCVarBitfield),
                   type(ResetTestCvars)
            "#,
        )
        .unwrap();
    assert_eq!(register_cvar, "function");
    assert_eq!(get_bitfield, "function");
    assert_eq!(set_bitfield, "function");
    assert_eq!(reset_test, "function");
}

#[test]
fn clamp_exists_during_bootstrap() {
    let env = env();
    let (high, low, mid, saturate): (i32, i32, i32, f64) = env
        .eval(
            r#"
            return Clamp(8, 1, 5), Clamp(-2, 1, 5), Clamp(3, 1, 5), Saturate(1.5)
            "#,
        )
        .unwrap();
    assert_eq!(high, 5);
    assert_eq!(low, 1);
    assert_eq!(mid, 3);
    assert_eq!(saturate, 1.0);
}
