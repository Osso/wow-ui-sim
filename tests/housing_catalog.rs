use wow_ui_sim::lua_api::WowLuaEnv;

const HOUSING_CART_SIZE_LIMIT_SCRIPT: &str = r#"
    local getCartSizeLimit = C_HousingCatalog.GetCartSizeLimit
    if type(getCartSizeLimit) ~= "function" then
        return "not_callable"
    end

    local returnCount, cartSizeLimit = select('#', getCartSizeLimit()), getCartSizeLimit()
    if returnCount ~= 1 or type(cartSizeLimit) ~= "number" then
        return "wrong_return_shape"
    end

    if cartSizeLimit ~= 20 then
        return "wrong_value:" .. tostring(cartSizeLimit)
    end

    return "ok"
"#;

const HOUSING_PREVIEW_CART_SCRIPT: &str = r#"
    local previewGUID = "Preview-Decor-1001"

    if C_HousingCatalog.IsPreviewCartItemShown(previewGUID) ~= false then
        return "unknown_preview_should_be_hidden"
    end

    C_HousingCatalog.SetPreviewCartItemShown(previewGUID, true)
    if C_HousingCatalog.IsPreviewCartItemShown(previewGUID) ~= true then
        return "preview_true_not_reflected"
    end

    C_HousingCatalog.SetPreviewCartItemShown(previewGUID, false)
    if C_HousingCatalog.IsPreviewCartItemShown(previewGUID) ~= false then
        return "preview_false_not_reflected"
    end

    if C_HousingCatalog.PromotePreviewDecor(1001, previewGUID) ~= true then
        return "preview_promotion_failed"
    end

    if C_HousingCatalog.IsPreviewCartItemShown(previewGUID) ~= true then
        return "preview_promotion_not_reflected"
    end

    return "ok"
"#;

const HOUSING_CATALOG_SCRIPT: &str = r#"
    local featured = C_HousingCatalog.GetFeaturedSmallProducts()
    if #featured ~= 2 then
        return "wrong_featured_count:" .. tostring(#featured)
    end

    if featured[1].entryID ~= 1001 or featured[1].productID ~= 91001 then
        return "wrong_first_featured_product"
    end

    local marketInfo = C_HousingCatalog.GetMarketInfoForDecor(1001)
    if marketInfo.isInCart ~= false or marketInfo.cartCount ~= 0 or marketInfo.wasViewedInStore ~= false then
        return "wrong_initial_market_info"
    end

    if C_HousingCatalog.HousingMarketActionAddToCart(1001) ~= true then
        return "add_to_cart_failed"
    end

    marketInfo = C_HousingCatalog.GetMarketInfoForDecor(1001)
    if marketInfo.isInCart ~= true or marketInfo.cartCount ~= 1 then
        return "add_to_cart_not_reflected"
    end

    if C_HousingCatalog.HousingMarketActionViewInStore(1001) ~= true then
        return "view_in_store_failed"
    end

    marketInfo = C_HousingCatalog.GetMarketInfoForDecor(1001)
    if marketInfo.wasViewedInStore ~= true then
        return "view_in_store_not_reflected"
    end

    if C_HousingCatalog.HousingMarketActionRemoveFromCart(1001) ~= true then
        return "remove_from_cart_failed"
    end

    marketInfo = C_HousingCatalog.GetMarketInfoForDecor(1001)
    if marketInfo.isInCart ~= false or marketInfo.cartCount ~= 0 then
        return "remove_from_cart_not_reflected"
    end

    C_HousingCatalog.HousingMarketActionAddToCart(1001)
    C_HousingCatalog.HousingMarketActionAddToCart(1002)
    C_HousingCatalog.HousingMarketActionClearCart()

    local firstAfterClear = C_HousingCatalog.GetMarketInfoForDecor(1001)
    local secondAfterClear = C_HousingCatalog.GetMarketInfoForDecor(1002)
    if firstAfterClear.isInCart ~= false or secondAfterClear.isInCart ~= false then
        return "clear_cart_not_reflected"
    end

    local bundle = C_HousingCatalog.GetBundleInfo(5001)
    if not bundle or bundle.wasViewed ~= false or #bundle.entryIDs ~= 2 then
        return "wrong_initial_bundle_info"
    end

    if type(bundle.canPreview) ~= "boolean" or bundle.canPreview ~= true then
        return "wrong_bundle_can_preview"
    end

    if C_HousingCatalog.HousingMarketActionViewBundle(5001) ~= true then
        return "view_bundle_failed"
    end

    bundle = C_HousingCatalog.GetBundleInfo(5001)
    if bundle.wasViewed ~= true then
        return "view_bundle_not_reflected"
    end

    return "ok"
"#;

// This surface was unconditional before native ownership; no profile removal.
#[test]
fn housing_catalog_empty_surface_remains_registered() {
    let env = env();
    env.exec(
        r#"
        local id = {recordID = 1001, entryType = 1, variantIdentifier = 1}
        assert(C_HousingCatalog.GetCatalogEntryInfo(id) == nil)
        assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id) == nil)
        assert(#C_HousingCatalog.GetAllVariantInfosForEntry(id) == 0)
        local searcher = C_HousingCatalog.CreateCatalogSearcher()
        searcher:SetStoredOnly(true)
        assert(searcher:IsStoredOnlyActive())
        searcher:ToggleStoredOnly()
        assert(searcher:IsStoredOnlyActive() == false)
        searcher:ToggleBaseVariantOnly()
        assert(searcher:IsBaseVariantOnlyActive() == true)
        searcher:SetSearchText('retained parameter')
        assert(searcher:GetSearchText() == 'retained parameter')
        searcher:SetAutoUpdateOnParamChanges(false)
        local callbacks = 0
        searcher:SetResultsUpdatedCallback(function()
            callbacks = callbacks + 1
            assert(not searcher:IsSearchInProgress())
            assert(#searcher:GetAllSearchItems() == 0)
            assert(#searcher:GetCatalogSearchResults() == 0)
        end)
        searcher:RunSearch()
        assert(callbacks == 1)
        assert(#searcher:GetAllSearchItems() == 0)
        assert(#searcher:GetCatalogSearchResults() == 0)
    "#,
    )
    .unwrap();
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn housing_catalog_cart_size_limit_returns_seeded_limit() {
    let env = env();
    let result: String = env
        .eval(HOUSING_CART_SIZE_LIMIT_SCRIPT)
        .expect("seeded C_HousingCatalog cart size limit should be queryable");
    assert_eq!(result, "ok");
}

#[test]
fn housing_preview_cart_state_round_trips_and_promotes() {
    let env = env();
    let result: String = env
        .eval(HOUSING_PREVIEW_CART_SCRIPT)
        .expect("seeded C_HousingCatalog preview cart state should be queryable");
    assert_eq!(result, "ok");
}

#[test]
fn housing_catalog_storefront_and_market_methods_use_seeded_state() {
    let env = env();
    let result: String = env
        .eval(HOUSING_CATALOG_SCRIPT)
        .expect("seeded C_HousingCatalog methods should be queryable");
    assert_eq!(result, "ok");
}
