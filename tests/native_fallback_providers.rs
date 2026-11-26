use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Lua environment should initialize")
}

#[test]
fn tutorials_enabled_reads_supplied_cvar_values() {
    let env = env();
    let values: (bool, bool) = env
        .eval(
            r#"
            C_CVar.SetCVar("showTutorials", "0")
            local disabled = GetTutorialsEnabled()
            C_CVar.SetCVar("showTutorials", "1")
            return disabled, GetTutorialsEnabled()
            "#,
        )
        .expect("tutorial setting should follow the CVar");

    assert_eq!(values, (false, true));
}

#[test]
fn account_type_queries_follow_two_supplied_records() {
    let env = env();
    let values: (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            A_Admin.SetAccountType({accountType=4, restricted=false, expansionTrial=true,
                expansionTrialExpiry=10000, restrictedLevel=20, restrictedMoney=0})
            local veteran, trial, restricted = IsVeteranTrialAccount(), IsTrialAccount(),
                IsRestrictedAccount()
            local expansion = IsExpansionTrial()
            local infoEnabled, remaining = GetExpansionTrialInfo()
            A_Admin.SetAccountType({accountType=5, restricted=true, expansionTrial=false,
                expansionTrialExpiry=0, restrictedLevel=10, restrictedMoney=500})
            local disabledInfo = not select(1, GetExpansionTrialInfo())
            return veteran, trial, restricted, expansion, infoEnabled and remaining > 0,
                IsVeteranTrialAccount(), IsTrialAccount(), IsRestrictedAccount(),
                IsExpansionTrial(), disabledInfo
            "#,
        )
        .expect("account queries should follow supplied account records");

    assert_eq!(values, (true, false, false, true, true, false, true, true, false, true));
}

#[test]
fn player_language_queries_follow_two_supplied_races() {
    let env = env();
    let values: (String, i32, i32, bool, String, i32, i32, String, i32, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerRace(0)
            local humanName, humanID = GetDefaultLanguage()
            local humanCount = GetNumLanguages()
            local humanIndexedName, humanIndexedID = GetLanguageByIndex(1)
            local humanSecond = GetLanguageByIndex(2) == nil
            A_Admin.SetPlayerRace(2)
            local dwarfName, dwarfID = GetDefaultLanguage()
            local alternativeName, alternativeID = GetAlternativeDefaultLanguage()
            local dwarfCount = GetNumLanguages()
            local indexedName, indexedID = GetLanguageByIndex(2)
            return humanName, humanID, humanCount, humanSecond,
                dwarfName, dwarfID, dwarfCount, alternativeName, alternativeID, indexedID
            "#,
        )
        .expect("language queries should follow supplied races");

    assert_eq!(values, ("Common".to_string(), 7, 1, true, "Common".to_string(), 7, 2,
        "Dwarvish".to_string(), 6, 6));
}
