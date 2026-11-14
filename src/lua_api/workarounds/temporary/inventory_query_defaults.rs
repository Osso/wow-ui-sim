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

const INVENTORY_SLOT_DEFAULT_LUA: &str = r#"
if GetInventoryItemsForSlot == nil then
    function GetInventoryItemsForSlot()
    end
end
"#;

// Retail 12.1.5: an empty inventory slot returns zero values (VERIFIED).
// PTR may share this native path (INFERRED); keep this change Retail-only.
const RETAIL_INVENTORY_QUERY_DEFAULTS_LUA: &str = r#"
if IsInventoryItemProfessionBag == nil then
    function IsInventoryItemProfessionBag(unit, slot)
        if unit == "player" and type(slot) == "number" and slot % 1 == 0
                and GetInventoryItemID(unit, slot) == nil then
            return
        end
        return false
    end
end

"#;

const INVENTORY_QUERY_DEFAULTS_LUA: &str = r#"
if IsInventoryItemProfessionBag == nil then
    function IsInventoryItemProfessionBag(_unit, _slot)
        return false
    end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
        lua.exec(INVENTORY_SLOT_DEFAULT_LUA)?;
    }
    #[cfg(not(feature = "retail-12-0-0"))]
    lua.exec(LEGACY_RELIC_QUERY_LUA)?;
    let defaults = if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail
    {
        RETAIL_INVENTORY_QUERY_DEFAULTS_LUA
    } else {
        INVENTORY_QUERY_DEFAULTS_LUA
    };
    lua.exec(defaults)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_unmodeled_inventory_items_for_slot_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let function_type: String = env
            .eval("return type(GetInventoryItemsForSlot)")
            .expect("inventory slot lookup type probe should run");
        assert_eq!(function_type, "function");

        if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
            let is_nil: bool = env
                .eval("return GetInventoryItemsForSlot(1) == nil")
                .expect("inventory slot lookup fallback probe should run");
            assert!(is_nil);
        }
    }

    #[test]
    fn installs_unmodeled_profession_bag_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, i64) = env
            .eval(
                r##"
                return type(IsInventoryItemProfessionBag),
                       select("#", IsInventoryItemProfessionBag("player", 20))
                "##,
            )
            .expect("profession bag fallback probe should run");

        let expected_returns =
            if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
                0
            } else {
                1
            };
        assert_eq!(result, ("function".to_string(), expected_returns));
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
