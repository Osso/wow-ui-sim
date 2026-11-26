//! Temporary `C_Item` targeting defaults.
//!
//! Helpful/harmful item targeting depends on cursor and targeting state that is
//! not modeled yet. Keep the inert false answers explicit here while the
//! state-backed item metadata surface remains in the C API owner.

const ITEM_TARGETING_DEFAULTS_LUA: &str = r#"
C_Item = C_Item or __wow_namespace()

local function installItemTargetingDefault(name, fn)
    if rawget(C_Item, name) == nil then
        C_Item[name] = fn
    end
end

installItemTargetingDefault("IsHelpfulItem", function()
    return false
end)

installItemTargetingDefault("IsHarmfulItem", function()
    return false
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(ITEM_TARGETING_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_false_item_targeting_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, bool) = env
            .eval("return C_Item.IsHelpfulItem(6948), C_Item.IsHarmfulItem(6948)")
            .expect("item targeting defaults should be callable");

        assert_eq!(result, (false, false));
    }

    #[test]
    fn preserves_existing_item_targeting_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_Item.IsHelpfulItem()
                return true
            end
            "#,
        )
        .expect("fixture should install existing item targeting provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let helpful: bool = env
            .eval("return C_Item.IsHelpfulItem(6948)")
            .expect("existing item targeting provider should remain callable");

        assert!(helpful);
    }
}
