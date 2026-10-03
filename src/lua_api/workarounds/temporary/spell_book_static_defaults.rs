//! Temporary `C_SpellBook` static/default fallbacks.
//!
//! The state-backed `C_SpellBook` surface owns real spellbook rows. These
//! defaults cover unmodeled pet-call, override, flyout, base-spell, autocast,
//! and loss-of-control cooldown gaps until those domains are modeled.

const SPELL_BOOK_STATIC_DEFAULTS_LUA: &str = r#"
C_SpellBook = C_SpellBook or __wow_namespace()

if rawget(C_SpellBook, "GetOverrideSpell") == nil then
    function C_SpellBook.GetOverrideSpell(spellID)
        return spellID
    end
end

if rawget(C_SpellBook, "FindSpellOverrideByID") == nil then
    function C_SpellBook.FindSpellOverrideByID(spellID)
        return spellID
    end
end

if rawget(C_SpellBook, "FindFlyoutSlotBySpellID") == nil then
    function C_SpellBook.FindFlyoutSlotBySpellID(_spellID)
    end
end

if rawget(C_SpellBook, "FindBaseSpellByID") == nil then
    function C_SpellBook.FindBaseSpellByID(_spellID)
    end
end

if rawget(C_SpellBook, "GetSpellBookItemAutoCast") == nil then
    function C_SpellBook.GetSpellBookItemAutoCast(_slot, _spellBank)
        return false, false
    end
end

if rawget(C_SpellBook, "GetSpellBookItemLossOfControlCooldownInfo") == nil then
    function C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(slot, spellBank)
        if type(C_SpellBook.GetSpellBookItemInfo) == "function"
            and C_SpellBook.GetSpellBookItemInfo(slot, spellBank) == nil then
            return nil
        end
        return {
            isActive = false,
            startTime = 0,
            duration = 0,
            modRate = 1,
            shouldReplaceNormalCooldown = false,
        }
    end
end

if rawget(_G, "GetCallPetSpellInfo") == nil then
    function GetCallPetSpellInfo(_spellID)
        return nil, nil
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SPELL_BOOK_STATIC_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_spell_book_static_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, i32, i32, bool, bool, bool, bool) = env
            .eval(
                r##"
                local autocast, autoable = C_SpellBook.GetSpellBookItemAutoCast(1)
                local spellID, texture = GetCallPetSpellInfo(1)
                return C_SpellBook.FindSpellOverrideByID(116),
                       select("#", C_SpellBook.FindFlyoutSlotBySpellID(116)),
                       select("#", C_SpellBook.FindBaseSpellByID(116)),
                       autocast,
                       autoable,
                       spellID == nil,
                       texture == nil
                "##,
            )
            .expect("spellbook static defaults should be callable");

        assert_eq!(result, (116, 0, 0, false, false, true, true));
    }

    #[test]
    fn preserves_existing_spell_book_static_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_SpellBook = C_SpellBook or __wow_namespace()

            function C_SpellBook.FindSpellOverrideByID(_spellID)
                return 999
            end

            function GetCallPetSpellInfo(_spellID)
                return 123, "pet-icon"
            end
            "#,
        )
        .expect("fixture should install existing spellbook defaults provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32, String) = env
            .eval(
                r#"
                local spellID, texture = GetCallPetSpellInfo(1)
                return C_SpellBook.FindSpellOverrideByID(116), spellID, texture
                "#,
            )
            .expect("existing spellbook defaults provider should remain callable");

        assert_eq!(result, (999, 123, "pet-icon".to_string()));
    }

    #[test]
    fn loss_of_control_info_preserves_absent_slot_nil_shape() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let spell_id: i32 = env
            .eval("return C_SpellBook.GetSpellBookItemInfo(5, 0).spellID")
            .expect("actual player-bank slot 5 should have a spell");
        assert_eq!(spell_id, 19750);
        env.state().borrow_mut().spell_loss_of_control.insert(
            spell_id.try_into().expect("positive fixture spell ID"),
            crate::lua_api::LossOfControlInfo {
                start_time: 0.0,
                duration: 0.0,
                mod_rate: 1.0,
                is_active: false,
                should_replace_normal_cooldown: false,
            },
        );

        let result: (bool, String) = env
            .eval(
                r#"
                local missing = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(99999, Enum.SpellBookSpellBank.Player)
                local present = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(5, Enum.SpellBookSpellBank.Player)
                return missing == nil, type(present)
                "#,
            )
            .expect("loss-of-control cooldown defaults should be callable");

        assert_eq!(result, (true, "table".to_string()));
    }
}
