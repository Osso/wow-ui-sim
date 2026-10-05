//! Temporary legacy SpellBook / Spell global wrappers.
//!
//! The state-backed `C_Spell` and `C_SpellBook` surfaces are registered in Rust.
//! These legacy globals remain here for older Blizzard/addon callers until the
//! compatibility boundary is narrowed further.

#[cfg(not(feature = "retail-12-0-0"))]
const PRE_12_0_0_SPELL_GLOBALS_LUA: &str = r#"
if GetSpellBookItemName == nil and C_SpellBook ~= nil then
    function GetSpellBookItemName(...)
        return C_SpellBook.GetSpellBookItemName(...)
    end
end
if GetSpellInfo == nil and C_Spell ~= nil then
    function GetSpellInfo(spellID)
        local info = C_Spell.GetSpellInfo(spellID)
        if info == nil then
            return nil
        end
        return info.name, nil, info.iconID, info.castTime, info.minRange, info.maxRange, info.spellID
    end
end
if GetSpellTexture == nil and C_Spell ~= nil then
    function GetSpellTexture(spellID)
        return C_Spell.GetSpellTexture(spellID)
    end
end
"#;

const LEGACY_SPELL_GLOBALS_LUA: &str = r#"
if GetSpellBookItemInfo == nil and C_SpellBook ~= nil then
    function GetSpellBookItemInfo(...)
        return C_SpellBook.GetSpellBookItemInfo(...)
    end
end
if GetSpellBookItemTexture == nil and C_SpellBook ~= nil then
    function GetSpellBookItemTexture(...)
        return C_SpellBook.GetSpellBookItemTexture(...)
    end
end
if IsPlayerSpell == nil and C_SpellBook ~= nil then
    function IsPlayerSpell(spellID)
        return C_SpellBook.IsSpellKnown(spellID, Enum.SpellBookSpellBank.Player)
    end
end
if IsSpellKnownOrOverridesKnown == nil and C_SpellBook ~= nil then
    function IsSpellKnownOrOverridesKnown(spellID, isPet)
        local spellBank = isPet and Enum.SpellBookSpellBank.Pet or Enum.SpellBookSpellBank.Player
        return C_SpellBook.IsSpellInSpellBook(spellID, spellBank, true)
    end
end
if FindFlyoutSlotBySpellID == nil and C_SpellBook ~= nil then
    function FindFlyoutSlotBySpellID(spellID)
        return C_SpellBook.FindFlyoutSlotBySpellID(spellID)
    end
end
if FindSpellOverrideByID == nil and C_SpellBook ~= nil then
    function FindSpellOverrideByID(spellID)
        return C_SpellBook.FindSpellOverrideByID(spellID)
    end
end
if FindBaseSpellByID == nil and C_SpellBook ~= nil then
    function FindBaseSpellByID(spellID)
        return C_SpellBook.FindBaseSpellByID(spellID)
    end
end
if IsPassiveSpell == nil then
    function IsPassiveSpell(_spellID)
        return false
    end
end
if IsPressHoldReleaseSpell == nil and C_Spell ~= nil then
    function IsPressHoldReleaseSpell(...)
        return C_Spell.IsPressHoldReleaseSpell(...)
    end
end
if IsSelectedSpellBookItem == nil then
    function IsSelectedSpellBookItem(_slotIndex, _unit)
        return false
    end
end
if SpellBook_GetSpellBookSlot == nil then
    function SpellBook_GetSpellBookSlot(slot, _offset)
        return slot
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(feature = "retail-12-0-0"))]
    lua.exec(PRE_12_0_0_SPELL_GLOBALS_LUA)?;
    lua.exec(LEGACY_SPELL_GLOBALS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(feature = "retail-12-0-0"))]
    #[test]
    fn keeps_state_backed_legacy_spell_globals_callable() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy spell globals should apply");
        }

        let result: String = env
            .eval(
                r##"
                if type(GetSpellBookItemName) ~= "function" then return "book_name" end
                if type(GetSpellBookItemInfo) ~= "function" then return "book_info" end
                if type(GetSpellBookItemTexture) ~= "function" then return "book_texture" end
                if type(IsPlayerSpell) ~= "function" then return "is_player_spell" end
                if type(IsSpellKnownOrOverridesKnown) ~= "function" then return "known_or_override" end
                if type(FindFlyoutSlotBySpellID) ~= "function" then return "flyout_slot" end
                if type(FindSpellOverrideByID) ~= "function" then return "spell_override" end
                if type(FindBaseSpellByID) ~= "function" then return "base_spell" end
                if type(GetSpellInfo) ~= "function" then return "spell_info" end
                if type(GetSpellTexture) ~= "function" then return "spell_texture" end
                if IsPassiveSpell(116) ~= false then return "passive" end
                if type(IsPressHoldReleaseSpell) ~= "function" then return "press_hold_type" end
                if IsPressHoldReleaseSpell(116) ~= false then return "press_hold_value" end
                if IsSelectedSpellBookItem(1, "player") ~= false then return "selected_spellbook" end
                if SpellBook_GetSpellBookSlot(3, 20) ~= 3 then return "slot" end
                if FindSpellOverrideByID(116) ~= 116 then return "override_value" end
                if select("#", FindFlyoutSlotBySpellID(116)) ~= 0 then return "flyout_value" end
                if select("#", FindBaseSpellByID(116)) ~= 0 then return "base_value" end
                return "ok"
                "##,
            )
            .expect("legacy spell global probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_legacy_spell_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            GetSpellInfo = function() return "existing" end
            IsPassiveSpell = function() return true end
            IsPressHoldReleaseSpell = function() return true end
            IsSelectedSpellBookItem = function() return true end
            C_Spell = {
                GetSpellInfo = function()
                    return { name = "new" }
                end,
                IsPressHoldReleaseSpell = function()
                    return false
                end,
            }
            "#,
        )
        .expect("fixture should install existing legacy spell globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("legacy spell globals should apply");
        }

        let (spell_name, passive, press_hold, selected): (String, bool, bool, bool) = env
            .eval(
                "return GetSpellInfo(1), IsPassiveSpell(1), IsPressHoldReleaseSpell(1), IsSelectedSpellBookItem(1)",
            )
            .expect("legacy spell preservation probe should run");

        assert_eq!(spell_name, "existing");
        assert!(passive);
        assert!(press_hold);
        assert!(selected);
    }
}
