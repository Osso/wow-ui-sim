//! Temporary housing compatibility surface.
//!
//! Catalog variants/search are state-backed and start empty. Remaining legacy
//! decor/neighborhood fixtures stay isolated here until their backing models exist.

const HOUSING_CATALOG_STATE_LUA: &str = include_str!("housing_catalog_state.lua");

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    // Only inverse profiles publish the original seeded exterior members.
    // Modern profiles install real state-backed APIs; no missing-method fallback.
    let install_legacy = if crate::c_api::c_housing::exterior::MODELED {
        ""
    } else {
        "install()"
    };
    let source = format!(
        "local install = (function()\n{HOUSING_CATALOG_STATE_LUA}\nend)()\n{install_legacy}"
    );
    lua.exec(&source)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_housing_catalog_surface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if C_CatalogShop.IsShop2Enabled() ~= false then
                    return "bad_shop_flag"
                end
                local productIDs = C_CatalogShop.GetProductIDsForCategory(
                    Constants.HousingCatalogConsts.HOUSING_CATALOG_ALL_CATEGORY_ID)
                if type(productIDs) ~= "table" or #productIDs == 0 then
                    return "bad_catalog_products"
                end
                local decorInfo = C_HousingDecor.GetSelectedDecorInfo()
                if type(decorInfo) ~= "table" or decorInfo.decorID == nil then
                    return "bad_decor"
                end
                local featured = C_HousingCatalog.GetFeaturedSmallProducts()
                if type(featured) ~= "table" or #featured == 0 then
                    return "bad_featured"
                end
                local searcher = C_HousingCatalog.CreateCatalogSearcher()
                if type(searcher) ~= "table" then
                    return "bad_searcher_type"
                end
                if searcher:GetSearchCount() ~= 0 then
                    return "searcher_count_not_empty"
                end
                return "ok"
                "#,
            )
            .expect("housing catalog probe should run");

        assert_eq!(result, "ok");
    }
}
