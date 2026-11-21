//! Temporary loot-journal defaults.
//!
//! Loot journal item-set data is not modeled yet. Startup consumers only need
//! empty result lists, so keep that inert compatibility behavior outside the C
//! API implementation until item-set state exists.

const LOOT_JOURNAL_DEFAULTS_LUA: &str = r#"
C_LootJournal = C_LootJournal or __wow_namespace()
if rawget(C_LootJournal, "GetItemSets") == nil then
    function C_LootJournal.GetItemSets(_classID, _specID)
        return {}
    end
end
if rawget(C_LootJournal, "GetItemSetItems") == nil then
    function C_LootJournal.GetItemSetItems(_itemSetID)
        return {}
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(LOOT_JOURNAL_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_loot_journal_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if #C_LootJournal.GetItemSets(1, 1) ~= 0 then return "sets" end
                if #C_LootJournal.GetItemSetItems(1) ~= 0 then return "items" end
                return "ok"
                "#,
            )
            .expect("loot-journal defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_loot_journal_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_LootJournal.GetItemSets()
                return { "modeled-set" }
            end
            function C_LootJournal.GetItemSetItems()
                return { "modeled-item" }
            end
            "#,
        )
        .expect("fixture should install existing functions");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                return C_LootJournal.GetItemSets()[1] .. ":" .. C_LootJournal.GetItemSetItems()[1]
                "#,
            )
            .expect("existing loot-journal functions should remain callable");

        assert_eq!(result, "modeled-set:modeled-item");
    }
}
