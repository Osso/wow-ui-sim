//! Temporary black-market defaults.
//!
//! Black market auction state is not modeled yet. The Black Market UI can load
//! against an empty auction list, so expose that inert compatibility surface as
//! a temporary workaround until real bidding/listing state exists.

const BLACK_MARKET_DEFAULTS_LUA: &str = r#"
C_BlackMarket = C_BlackMarket or __wow_namespace()

if rawget(C_BlackMarket, "Close") == nil then
    function C_BlackMarket.Close()
    end
end
if rawget(C_BlackMarket, "RequestItems") == nil then
    function C_BlackMarket.RequestItems()
    end
end
if rawget(C_BlackMarket, "ItemPlaceBid") == nil then
    function C_BlackMarket.ItemPlaceBid(_marketID, _bidAmount)
    end
end
if rawget(C_BlackMarket, "IsViewOnly") == nil then
    function C_BlackMarket.IsViewOnly()
        return false
    end
end
if rawget(C_BlackMarket, "GetNumItems") == nil then
    function C_BlackMarket.GetNumItems()
        return 0
    end
end
if rawget(C_BlackMarket, "GetHotItem") == nil then
    function C_BlackMarket.GetHotItem()
    end
end
if rawget(C_BlackMarket, "GetItemInfoByID") == nil then
    function C_BlackMarket.GetItemInfoByID(_marketID)
    end
end
if rawget(C_BlackMarket, "GetItemInfoByIndex") == nil then
    function C_BlackMarket.GetItemInfoByIndex(_index)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(BLACK_MARKET_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_black_market_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if C_BlackMarket.IsViewOnly() ~= false then return "view-only" end
                if C_BlackMarket.GetNumItems() ~= 0 then return "num-items" end
                if C_BlackMarket.GetHotItem() ~= nil then return "hot-item" end
                if C_BlackMarket.GetItemInfoByID(1) ~= nil then return "by-id" end
                if C_BlackMarket.GetItemInfoByIndex(1) ~= nil then return "by-index" end
                C_BlackMarket.Close()
                C_BlackMarket.RequestItems()
                C_BlackMarket.ItemPlaceBid(1, 100)
                return "ok"
                "#,
            )
            .expect("black-market defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_black_market_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_BlackMarket.GetNumItems()
                return 7
            end
            function C_BlackMarket.IsViewOnly()
                return true
            end
            "#,
        )
        .expect("fixture should install existing functions");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                return tostring(C_BlackMarket.IsViewOnly()) .. ":" .. C_BlackMarket.GetNumItems()
                "#,
            )
            .expect("existing black-market functions should remain callable");

        assert_eq!(result, "true:7");
    }
}
