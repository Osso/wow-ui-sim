//! Temporary macro UI fixture defaults.
//!
//! Macro execution/edit verbs are Rust-backed in `spell_macro_verbs`; these
//! startup defaults seed the small macro list and icon lists expected by
//! Blizzard_MacroUI until macro storage is fully modeled.

const MACRO_DEFAULTS_LUA: &str = r#"
if GetNumMacros == nil then
    function GetNumMacros() return 2, 1 end
end
if GetMacroInfo == nil then
    function GetMacroInfo(index)
        if index == 1 then
            return "Raid Beacon", "Interface\\Icons\\INV_Misc_QuestionMark", "/rw Stack on star"
        end
        if index == 121 then
            return "Crusader", "Interface\\Icons\\Spell_Holy_CrusaderAura", "/cast Crusader Aura"
        end
        return nil
    end
end
if GetMacroBody == nil then
    function GetMacroBody(index)
        local _, _, body = GetMacroInfo(index)
        return body
    end
end
if CursorHasMacro == nil then
    function CursorHasMacro() return false end
end

__wow_loose_macro_icons = {
    "INV_Misc_QuestionMark",
}
__wow_macro_icons = {
    "Spell_Holy_CrusaderAura",
}
__wow_loose_macro_item_icons = {
    "INV_Misc_Bag_08",
}
__wow_macro_item_icons = {
    "INV_Sword_04",
}
function __wow_append_icons(iconTable, icons)
    if type(iconTable) ~= "table" then
        iconTable = {}
    end
    for _, icon in ipairs(icons) do
        table.insert(iconTable, icon)
    end
    return iconTable
end
if GetLooseMacroIcons == nil then
    function GetLooseMacroIcons(iconTable)
        __wow_append_icons(iconTable, __wow_loose_macro_icons)
    end
end
if GetLooseMacroItemIcons == nil then
    function GetLooseMacroItemIcons(iconTable)
        __wow_append_icons(iconTable, __wow_loose_macro_item_icons)
    end
end
if GetMacroIcons == nil then
    function GetMacroIcons(iconTable)
        return __wow_append_icons(iconTable, __wow_macro_icons)
    end
end
if GetMacroItemIcons == nil then
    function GetMacroItemIcons(iconTable)
        return __wow_append_icons(iconTable, __wow_macro_item_icons)
    end
end

C_Macro = C_Macro or __wow_namespace()
if rawget(C_Macro, "GetNumMacros") == nil then
    function C_Macro.GetNumMacros() return 2, 1 end
end
if rawget(C_Macro, "GetMacroName") == nil then
    function C_Macro.GetMacroName(index)
        if index == 1 then
            return "Raid Beacon"
        end
        if index == 121 then
            return "Crusader"
        end
        return nil
    end
end
if rawget(C_Macro, "GetSelectedMacroIcon") == nil then
    function C_Macro.GetSelectedMacroIcon(index)
        if index == 121 then
            return "Interface\\Icons\\Spell_Holy_CrusaderAura"
        end
        return nil
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MACRO_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_macro_ui_fixture_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local accountCount, characterCount = GetNumMacros()
                if accountCount ~= 2 or characterCount ~= 1 then return "counts" end

                local accountName, accountIcon, accountBody = GetMacroInfo(1)
                if accountName ~= "Raid Beacon" then return "account_name" end
                if accountIcon ~= "Interface\\Icons\\INV_Misc_QuestionMark" then return "account_icon" end
                if accountBody ~= "/rw Stack on star" then return "account_body" end
                if GetMacroBody(121) ~= "/cast Crusader Aura" then return "character_body" end
                if CreateMacro("A", "B", "C", true) ~= 121 then return "create_character" end
                if CursorHasMacro() ~= false then return "cursor" end

                local spellIcons = {}
                if GetMacroIcons(spellIcons) ~= spellIcons then return "macro_icons_return" end
                if spellIcons[1] ~= "Spell_Holy_CrusaderAura" then return "macro_icons_value" end

                local looseIcons = {}
                GetLooseMacroIcons(looseIcons)
                if looseIcons[1] ~= "INV_Misc_QuestionMark" then return "loose_icons" end

                local cAccountCount, cCharacterCount = C_Macro.GetNumMacros()
                if cAccountCount ~= 2 or cCharacterCount ~= 1 then return "c_counts" end
                if C_Macro.GetMacroName(1) ~= "Raid Beacon" then return "c_name" end
                if C_Macro.GetSelectedMacroIcon(121) ~= "Interface\\Icons\\Spell_Holy_CrusaderAura" then return "c_icon" end
                return "ok"
                "#,
            )
            .expect("macro defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_macro_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function GetNumMacros() return 9, 8 end
            C_Macro.GetMacroName = function() return "Existing" end
            "#,
        )
        .expect("fixture should install existing macro members");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("macro defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                local accountCount, characterCount = GetNumMacros()
                if accountCount ~= 9 or characterCount ~= 8 then return "overwrote_counts" end
                if C_Macro.GetMacroName(1) ~= "Existing" then return "overwrote_c_macro" end
                if type(GetMacroInfo) ~= "function" then return "missing_global_default" end
                if type(C_Macro.GetSelectedMacroIcon) ~= "function" then return "missing_c_default" end
                return "ok"
                "#,
            )
            .expect("macro preservation probe should run");

        assert_eq!(result, "ok");
    }
}
