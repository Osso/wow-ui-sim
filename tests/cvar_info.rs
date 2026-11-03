#![cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

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

#[cfg(feature = "client-retail")]
#[test]
fn cvar_info_installed_uninstall_records_follow_native_scope_flags() {
    use std::path::{Path, PathBuf};
    use wow_ui_sim::toc::TocFile;

    let Some(addon_dir) = std::env::var_os("ELLESMERE_UI_PATH").map(PathBuf::from) else {
        eprintln!(
            "skipping installed CVar consumer: set ELLESMERE_UI_PATH to the EllesmereUI addon directory"
        );
        return;
    };
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon_dir)
        .expect("installed EllesmereUI must have a profile-selected TOC");
    let toc = TocFile::from_file(&toc_path).expect("parse installed EllesmereUI TOC");
    for name in ["EllesmereUI_ClientGate.lua", "EllesmereUI_Uninstall.lua"] {
        assert!(
            toc.files.iter().any(|path| path == Path::new(name)),
            "installed EllesmereUI TOC must select {name}"
        );
    }

    let env = WowLuaEnv::new().unwrap();
    let gate = std::fs::read_to_string(addon_dir.join("EllesmereUI_ClientGate.lua"))
        .expect("read unchanged installed client gate");
    env.exec(&gate)
        .expect("load unchanged installed client gate");
    env.exec(
        r#"
        assert(not EUI_CLIENT_BLOCKED, 'installed EllesmereUI must allow this client')
        EllesmereUIDB = {}
        -- Supply only the outer framework lifecycle; API calls stay real.
        EllesmereUI = { Lite = {
            StoreFromBeforeLoad = function() return false end,
            OnSavedVariablesLoaded = function(callback) callback() end,
        } }
        "#,
    )
    .unwrap();

    let source = std::fs::read_to_string(addon_dir.join("EllesmereUI_Uninstall.lua"))
        .expect("read unchanged installed uninstall consumer");
    let mut separator = String::from("=");
    while source.contains(&format!("]{separator}]")) {
        separator.push('=');
    }
    env.exec(&format!(
        "assert(loadstring([{separator}[{source}]{separator}], '@EllesmereUI_Uninstall.lua'))('EllesmereUI')"
    ))
        .expect("load entire unchanged uninstall file with its addon-name argument");

    let observed: String = env
        .eval(
            r#"
            local accountBefore = C_CVar.GetCVar('showTimestamps')
            local characterBefore = C_CVar.GetCVar('nameplateMaxDistance')
            assert(type(accountBefore) == 'string' and tonumber(characterBefore))
            local accountAfter = accountBefore .. ':scope-probe'
            local characterAfter = tostring(tonumber(characterBefore) + 1)
            assert(EllesmereUI.SetCVar('showTimestamps', accountAfter, 'EllesmereUIChat'))
            assert(EllesmereUI.SetCVar('nameplateMaxDistance', characterAfter, 'EllesmereUINameplates'))

            local record = EllesmereUIDB.restoreOnUninstall
            local character = assert(record.chars[UnitGUID('player')])
            assert(record.fresh == true)
            assert(record.cvar.showtimestamps.b == accountBefore)
            assert(record.cvar.showtimestamps.a == accountAfter)
            assert(record.cvar.showtimestamps.o == 'EllesmereUIChat')
            assert(character.cvar.nameplatemaxdistance.b == characterBefore)
            assert(character.cvar.nameplatemaxdistance.a == characterAfter)
            assert(character.cvar.nameplatemaxdistance.o == 'EllesmereUINameplates')
            assert(record.cvar.nameplatemaxdistance == nil, 'character CVar leaked into account record')
            assert(character.cvar.showtimestamps == nil, 'account CVar leaked into character record')
            return 'account=showtimestamps;character=nameplatemaxdistance'
            "#,
        )
        .expect("original consumer must preserve native account and character scopes");
    assert_eq!(
        observed,
        "account=showtimestamps;character=nameplatemaxdistance"
    );
}
