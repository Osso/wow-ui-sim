//! Temporary inventory query defaults not backed by equipment state yet.
//!
//! `GetInventoryItemID` and related equipped-item probes are SimState-backed in
//! `globals::inventory_probes`. Keep only still-unmodeled inventory lookup
//! fallbacks here instead of in generic global stub tables.

#[cfg(not(feature = "retail-12-0-0"))]
const LEGACY_RELIC_QUERY_LUA: &str = r#"
if IsArtifactRelicItem == nil then
    function IsArtifactRelicItem()
        return false
    end
end
"#;

const INVENTORY_QUERY_DEFAULTS_LUA: &str = r#"
if GetInventoryItemsForSlot == nil then
    function GetInventoryItemsForSlot()
    end
end

if IsInventoryItemProfessionBag == nil then
    function IsInventoryItemProfessionBag(_unit, _slot)
        return false
    end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(feature = "retail-12-0-0"))]
    lua.exec(LEGACY_RELIC_QUERY_LUA)?;
    lua.exec(INVENTORY_QUERY_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_unmodeled_inventory_items_for_slot_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, bool) = env
            .eval(
                r#"
                return type(GetInventoryItemsForSlot),
                       GetInventoryItemsForSlot(1) == nil
                "#,
            )
            .expect("inventory slot lookup fallback probe should run");

        assert_eq!(result, ("function".to_string(), true));
    }

    #[test]
    fn installs_unmodeled_profession_bag_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, bool) = env
            .eval(
                r#"
                return type(IsInventoryItemProfessionBag),
                       IsInventoryItemProfessionBag("player", 20)
                "#,
            )
            .expect("profession bag fallback probe should run");

        assert_eq!(result, ("function".to_string(), false));
    }

    #[test]
    fn preserves_existing_inventory_items_for_slot_function() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("function GetInventoryItemsForSlot() return 'existing' end")
            .expect("fixture should install existing inventory slot lookup");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("inventory query defaults should apply");
        }

        let value: String = env
            .eval("return GetInventoryItemsForSlot(1)")
            .expect("inventory slot lookup preservation probe should run");

        assert_eq!(value, "existing");
    }

    #[test]
    fn preserves_existing_profession_bag_function() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("function IsInventoryItemProfessionBag() return true end")
            .expect("fixture should install existing profession bag probe");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("inventory query defaults should apply");
        }

        let value: bool = env
            .eval(r#"return IsInventoryItemProfessionBag("player", 20)"#)
            .expect("profession bag preservation probe should run");

        assert!(value);
    }
}
