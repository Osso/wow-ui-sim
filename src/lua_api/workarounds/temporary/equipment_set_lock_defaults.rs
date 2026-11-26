//! Legacy equipment-set lock default for excluded Wrath/Era/Anniversary.
//!
//! Retail/PTR/Forever/Mists use the supplied model in c_api::equipment_set_locks;
//! this workaround must not provide a fallback for those profiles.

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const EQUIPMENT_SET_LOCK_DEFAULTS_LUA: &str = r#"
C_EquipmentSet = C_EquipmentSet or __wow_namespace()
if rawget(C_EquipmentSet, "EquipmentSetContainsLockedItems") == nil then
    function C_EquipmentSet.EquipmentSetContainsLockedItems(_setID)
        return false
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    {
        if crate::c_api::equipment_set_locks::supports_item_lock_observations() {
            return Ok(());
        }
        lua.exec(EQUIPMENT_SET_LOCK_DEFAULTS_LUA)?;
    }
    Ok(())
}

#[cfg(all(
    test,
    any(
        feature = "client-wrath",
        feature = "client-era",
        feature = "client-anniversary"
    )
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_unlocked_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let locked: bool = env
            .eval("return C_EquipmentSet.EquipmentSetContainsLockedItems(1)")
            .expect("equipment set lock default should be callable");

        assert!(!locked);
    }

    #[test]
    fn preserves_existing_lock_state_function() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_EquipmentSet.EquipmentSetContainsLockedItems()
                return true
            end
            "#,
        )
        .expect("fixture should install existing function");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let locked: bool = env
            .eval("return C_EquipmentSet.EquipmentSetContainsLockedItems(1)")
            .expect("existing equipment set lock function should remain callable");

        assert!(locked);
    }
}
