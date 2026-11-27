//! Temporary catalog-shop inbound globals.
//!
//! Blizzard catalog-shop and checkout addons publish these inbound tables from
//! secure/global environment swaps. Retail and Forever selected addons publish the
//! interfaces before consumers run; legacy profiles keep bootstrap fallbacks.

const CATALOG_SHOP_INBOUND_GLOBALS_LUA: &str = r#"
local function ensure_inbound_interface(name)
    if rawget(_G, name) ~= nil then
        return
    end

    local inbound = {}

    function inbound.IsShown()
        return false
    end

    function inbound.SetShown(_shown, _contextKey)
    end

    function inbound.EscapePressed()
        return false
    end

    function inbound.SelectSubscriptionProduct()
    end

    function inbound.SetTokenCategory()
    end

    function inbound.CheckForFree(_event)
    end

    function inbound.OpenGamesCategory()
    end

    function inbound.SetGamesCategory()
    end

    function inbound.SetServicesCategory()
    end

    function inbound.SelectBoost(_boostType, _reason, _guid)
    end

    function inbound.SelectGameTimeProduct()
    end

    function inbound.SelectSpecificProduct(_productID)
    end

    rawset(_G, name, inbound)
end

ensure_inbound_interface("CatalogShopInboundInterface")
ensure_inbound_interface("CatalogShopTopUpFlowInboundInterface")
ensure_inbound_interface("CatalogShopRefundFlowInboundInterface")
ensure_inbound_interface("SimpleCheckoutInboundInterface")
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    lua.exec(CATALOG_SHOP_INBOUND_GLOBALS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_catalog_shop_inbound_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local names = {
                    "CatalogShopTopUpFlowInboundInterface",
                    "CatalogShopRefundFlowInboundInterface",
                    "SimpleCheckoutInboundInterface",
                }
                for _, name in ipairs(names) do
                    local inbound = rawget(_G, name)
                    if type(inbound) ~= "table" then return name .. ":table" end
                    if inbound:IsShown() ~= false then return name .. ":shown" end
                    if inbound:EscapePressed() ~= false then return name .. ":escape" end
                    if type(inbound.SetShown) ~= "function" then return name .. ":set_shown" end
                    if type(inbound.CheckForFree) ~= "function" then return name .. ":free" end
                    if type(inbound.SelectSpecificProduct) ~= "function" then return name .. ":product" end
                end
                return "ok"
                "#,
            )
            .expect("catalog shop inbound defaults probe should run");

        assert_eq!(result, "ok");

        let main_interface_present: bool = env
            .eval("return type(CatalogShopInboundInterface) == 'table'")
            .expect("main Catalog Shop inbound fallback should be queryable");
        assert_eq!(
            main_interface_present,
            crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail
        );
    }

    #[test]
    fn preserves_existing_inbound_interface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        env.exec(
            r#"
                local original = {
                    IsShown = function() return true end,
                    EscapePressed = function() return true end,
                }
                rawset(_G, "CatalogShopInboundInterface", original)
                "#,
        )
        .expect("catalog shop inbound fixture should install");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("catalog shop inbound defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                return CatalogShopInboundInterface:IsShown()
                    and CatalogShopInboundInterface:EscapePressed()
                    and "ok"
                    or "replaced"
                "#,
            )
            .expect("catalog shop inbound preservation probe should run");

        assert_eq!(result, "ok");
    }
}
