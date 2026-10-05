use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn legacy_macro_and_spellbook_globals_exist_for_addons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if ACCOUNT_BINDINGS ~= 1 then
            return "account_bindings=" .. tostring(ACCOUNT_BINDINGS)
        end
        if CHARACTER_BINDINGS ~= 2 or CHARACTERBINDINGS ~= 2 then
            return "character_bindings=" .. tostring(CHARACTER_BINDINGS) .. "," .. tostring(CHARACTERBINDINGS)
        end
        if type(CreateMacro) ~= "function" then
            return "missing_CreateMacro"
        end
        if type(DeleteMacro) ~= "function" then
            return "missing_DeleteMacro"
        end
        if type(GetMacroBody) ~= "function" then
            return "missing_GetMacroBody"
        end
        if type(CursorHasMacro) ~= "function" then
            return "missing_CursorHasMacro"
        end
        if type(GetContainerItemInfo) ~= "function" then
            return "missing_GetContainerItemInfo"
        end
        if type(GetSpellBookItemInfo) ~= "function" then
            return "missing_GetSpellBookItemInfo"
        end
        if type(GetSpellBookItemName) ~= "function" then
            return "missing_GetSpellBookItemName"
        end

        if GetMacroBody(1) ~= "/rw Stack on star" then
            return "macro_body=" .. tostring(GetMacroBody(1))
        end
        local created = CreateMacro("Compat", "INV_Misc_QuestionMark", "/say compat")
        if type(created) ~= "number" then
            return "created_type=" .. type(created)
        end
        DeleteMacro(created)

        local spellName = GetSpellBookItemName(1)
        if spellName == nil then
            return "missing_spell_name"
        end
        local itemInfo = GetContainerItemInfo(0, 1)
        if type(itemInfo) ~= "table" then
            return "container_info_type=" .. type(itemInfo)
        end

        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "legacy macro and spellbook globals should exist for addon compatibility: {result}"
    );
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn legacy_spell_globals_exist_for_addons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if GAME_LOCALE ~= GetLocale() then
            return "game_locale=" .. tostring(GAME_LOCALE)
        end
        if type(GetSpellInfo) ~= "function" then
            return "missing_GetSpellInfo"
        end
        if type(GetSpellTexture) ~= "function" then
            return "missing_GetSpellTexture"
        end
        if type(IsPassiveSpell) ~= "function" then
            return "missing_IsPassiveSpell"
        end
        if type(GetSpellBookItemTexture) ~= "function" then
            return "missing_GetSpellBookItemTexture"
        end
        if type(SpellBook_GetSpellBookSlot) ~= "function" then
            return "missing_SpellBook_GetSpellBookSlot"
        end

        local spellName = GetSpellInfo(19750)
        if spellName == nil then
            return "missing_spell_info"
        end
        if GetSpellTexture(19750) == nil then
            return "missing_spell_texture"
        end
        if IsPassiveSpell(19750) ~= false then
            return "passive=" .. tostring(IsPassiveSpell(19750))
        end
        if GetSpellBookItemTexture(1) == nil then
            return "missing_spellbook_texture"
        end
        if SpellBook_GetSpellBookSlot(4, 7) ~= 4 then
            return "spellbook_slot=" .. tostring(SpellBook_GetSpellBookSlot(4, 7))
        end

        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "legacy spell globals should exist for addon compatibility: {result}"
    );
}

#[test]
fn class_talents_initialize_view_loadout_exists_for_addons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if type(C_ClassTalents.InitializeViewLoadout) ~= "function" then
            return "missing_InitializeViewLoadout"
        end
        if C_ClassTalents.InitializeViewLoadout(70, 80) ~= true then
            return "initialize_result=" .. tostring(C_ClassTalents.InitializeViewLoadout(70, 80))
        end
        return "ok"
    "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_ClassTalents.InitializeViewLoadout should exist for talent-tree addons: {result}"
    );
}

#[test]
fn player_login_legacy_globals_exist_for_addons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if type(IsOnTournamentRealm) ~= "function" then
            return "missing_IsOnTournamentRealm"
        end
        if IsOnTournamentRealm() ~= false then
            return "tournament_realm=" .. tostring(IsOnTournamentRealm())
        end
        if type(GetNumDisplayChannels) ~= "function" then
            return "missing_GetNumDisplayChannels"
        end
        if GetNumDisplayChannels() ~= 0 then
            return "display_channels=" .. tostring(GetNumDisplayChannels())
        end
        if type(GetChannelDisplayInfo) ~= "function" then
            return "missing_GetChannelDisplayInfo"
        end
        if GetChannelDisplayInfo(1) ~= nil then
            return "channel_info=" .. tostring(GetChannelDisplayInfo(1))
        end
        return "ok"
    "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "PLAYER_LOGIN legacy globals should exist for addon compatibility: {result}"
    );
}

#[test]
fn restricted_actions_state_is_numeric_for_addons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        if type(C_RestrictedActions.GetAddOnRestrictionState) ~= "function" then
            return "missing_GetAddOnRestrictionState"
        end
        local combatState = C_RestrictedActions.GetAddOnRestrictionState(Enum.AddOnRestrictionType.Combat)
        if combatState ~= 0 then
            return "combat_state=" .. tostring(combatState)
        end
        return "ok"
    "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "restricted action state should be numeric so addons can compare it: {result}"
    );
}
