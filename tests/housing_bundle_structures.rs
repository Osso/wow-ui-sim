use wow_ui_sim::c_api::c_housing_bundles::{HousingBundleDecorEntryInfo, HousingBundleInfo};
use wow_ui_sim::lua_api::WowLuaEnv;

fn housing_bundle_fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().housing_bundles.records.insert(
        801,
        HousingBundleInfo {
            product_id: 801.0,
            price: 19.25,
            original_price: Some(29.5),
            non_decor_products: vec![901.0, 902.5],
            decor_entries: vec![
                HousingBundleDecorEntryInfo {
                    decor_id: 101.0,
                    quantity: 3.0,
                },
                HousingBundleDecorEntryInfo {
                    decor_id: 102.0,
                    quantity: 0.0,
                },
            ],
            can_preview: false,
            was_viewed: false,
        },
    );
    env
}

#[test]
fn housing_bundle_seed_and_market_action_share_state_on_every_profile() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local saved = C_HousingCatalog.GetBundleInfo(5001)
        assert(saved.productID == 5001 and saved.price == 500)
        assert(saved.originalPrice == nil and saved.canPreview == true)
        assert(saved.wasViewed == false)
        assert(#saved.entryIDs == 2 and saved.entryIDs[1] == 1001 and saved.entryIDs[2] == 1002)
        assert(#saved.decorEntries == 2 and #saved.nonDecorProducts == 0)
        assert(saved.decorEntries[1].decorID == 1001 and saved.decorEntries[1].quantity == 1)
        assert(saved.decorEntries[2].decorID == 1002 and saved.decorEntries[2].quantity == 1)
        assert(#C_HousingCatalog.GetFeaturedBundles() == 1)
        assert(C_HousingCatalog.GetFeaturedBundles()[1].wasViewed == false)
        assert(select('#', C_HousingCatalog.HousingMarketActionViewBundle(5001)) == 1)
        assert(C_HousingCatalog.HousingMarketActionViewBundle(5001) == true)
        assert(C_HousingCatalog.GetBundleInfo(5001).wasViewed == true)
        assert(C_HousingCatalog.GetFeaturedBundles()[1].wasViewed == true)
        assert(saved.wasViewed == false)
        assert(C_HousingCatalog.HousingMarketActionViewBundle(5001) == true)
        assert(C_HousingCatalog.HousingMarketActionViewBundle(9999) == false)
        assert(C_HousingCatalog.GetBundleInfo(9999) == nil)
    "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    other
        .exec("assert(C_HousingCatalog.GetBundleInfo(5001).wasViewed == false)")
        .unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn housing_bundle_declared_shape_order_and_legacy_extensions() {
    let env = housing_bundle_fixture();
    env.exec(r#"
        local b = C_HousingCatalog.GetBundleInfo(801)
        local expected = {productID=true, price=true, originalPrice=true,
            nonDecorProducts=true, decorEntries=true, canPreview=true, entryIDs=true, wasViewed=true}
        local count = 0
        for k in pairs(b) do assert(expected[k]); count = count + 1 end
        assert(count == 8)
        assert(type(b.productID) == 'number' and b.productID == 801)
        assert(type(b.price) == 'number' and b.price == 19.25)
        assert(type(b.originalPrice) == 'number' and b.originalPrice == 29.5)
        assert(type(b.canPreview) == 'boolean' and b.canPreview == false)
        assert(type(b.wasViewed) == 'boolean' and b.wasViewed == false)
        assert(type(b.nonDecorProducts) == 'table' and #b.nonDecorProducts == 2)
        assert(type(b.nonDecorProducts[1]) == 'number' and b.nonDecorProducts[1] == 901)
        assert(type(b.nonDecorProducts[2]) == 'number' and b.nonDecorProducts[2] == 902.5)
        assert(type(b.decorEntries) == 'table' and #b.decorEntries == 2)
        for i, row in ipairs(b.decorEntries) do
            local fields = 0
            for k in pairs(row) do assert(k == 'decorID' or k == 'quantity'); fields = fields + 1 end
            assert(fields == 2)
            assert(type(row.decorID) == 'number' and row.decorID == 100 + i)
            assert(type(row.quantity) == 'number')
            assert(b.entryIDs[i] == row.decorID)
        end
        assert(b.decorEntries[1].quantity == 3 and b.decorEntries[2].quantity == 0)
    "#).unwrap();
}

#[test]
fn housing_bundle_empty_state_optional_price_and_zero_values() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.housing_bundles.records.clear();
        sim.housing_bundles.featured_ids.clear();
    }
    env.exec(
        r#"
        assert(select('#', C_HousingCatalog.GetBundleInfo(5001)) == 1)
        assert(C_HousingCatalog.GetBundleInfo(5001) == nil)
        assert(select('#', C_HousingCatalog.GetFeaturedBundles()) == 1)
        assert(#C_HousingCatalog.GetFeaturedBundles() == 0)
        assert(C_HousingCatalog.HousingMarketActionViewBundle(5001) == false)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().housing_bundles.records.insert(
        42,
        HousingBundleInfo {
            product_id: 42.0,
            price: 0.0,
            original_price: None,
            non_decor_products: vec![],
            decor_entries: vec![],
            can_preview: true,
            was_viewed: false,
        },
    );
    env.exec(
        r#"
        local b = C_HousingCatalog.GetBundleInfo(42)
        assert(b.price == 0 and b.productID == 42)
        assert(rawget(b, 'originalPrice') == nil)
        assert(type(b.nonDecorProducts) == 'table' and next(b.nonDecorProducts) == nil)
        assert(type(b.decorEntries) == 'table' and next(b.decorEntries) == nil)
        assert(type(b.entryIDs) == 'table' and next(b.entryIDs) == nil)
        assert(b.canPreview == true and b.wasViewed == false)
        local n = 0; for _ in pairs(b) do n = n + 1 end; assert(n == 7)
    "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    other
        .exec("assert(C_HousingCatalog.GetBundleInfo(42) == nil)")
        .unwrap();
}

#[test]
fn housing_bundle_featured_snapshots_follow_record_updates_and_removal() {
    let env = housing_bundle_fixture();
    env.state().borrow_mut().housing_bundles.featured_ids = vec![801, 5001];
    env.exec(
        r#"
        savedBundle = C_HousingCatalog.GetBundleInfo(801)
        local featured = C_HousingCatalog.GetFeaturedBundles()
        assert(#featured == 2 and featured[1].productID == 801 and featured[2].productID == 5001)
        assert(featured[1].nonDecorProducts[2] == 902.5)
        featured[1].decorEntries[1].quantity = 99
        featured[1].entryIDs[1] = -3
        featured[1].wasViewed = true
        savedBundle.nonDecorProducts[1] = -1
        savedBundle.decorEntries[1].quantity = -2
        collectgarbage('collect')
        assert(C_HousingCatalog.GetBundleInfo(801).nonDecorProducts[1] == 901)
        assert(C_HousingCatalog.GetBundleInfo(801).decorEntries[1].quantity == 3)
        assert(C_HousingCatalog.GetFeaturedBundles()[1].decorEntries[1].quantity == 3)
        assert(C_HousingCatalog.GetFeaturedBundles()[1].entryIDs[1] == 101)
        assert(C_HousingCatalog.GetBundleInfo(801).wasViewed == false)
        assert(C_HousingCatalog.HousingMarketActionViewBundle(801) == true)
        assert(C_HousingCatalog.GetFeaturedBundles()[1].wasViewed == true)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .housing_bundles
        .records
        .get_mut(&801)
        .unwrap()
        .price = 11.0;
    env.exec(
        r#"
        assert(C_HousingCatalog.GetBundleInfo(801).price == 11)
        assert(C_HousingCatalog.GetFeaturedBundles()[1].price == 11)
        assert(savedBundle.price == 19.25 and savedBundle.wasViewed == false)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .housing_bundles
        .records
        .remove(&801);
    env.exec(
        r#"
        assert(C_HousingCatalog.GetBundleInfo(801) == nil)
        assert(C_HousingCatalog.HousingMarketActionViewBundle(801) == false)
        local featured = C_HousingCatalog.GetFeaturedBundles()
        assert(#featured == 1 and featured[1].productID == 5001)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .housing_bundles
        .featured_ids
        .clear();
    env.exec("assert(#C_HousingCatalog.GetFeaturedBundles() == 0)")
        .unwrap();
}

#[test]
fn housing_bundle_non_decor_contents_disable_preview() {
    let env = housing_bundle_fixture();
    env.state()
        .borrow_mut()
        .housing_bundles
        .records
        .get_mut(&801)
        .unwrap()
        .can_preview = true;
    env.exec("assert(C_HousingCatalog.GetBundleInfo(801).canPreview == false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing_bundles
        .records
        .get_mut(&801)
        .unwrap()
        .non_decor_products
        .clear();
    env.exec("assert(C_HousingCatalog.GetBundleInfo(801).canPreview == true)")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing_bundles
        .records
        .get_mut(&801)
        .unwrap()
        .can_preview = false;
    env.exec("assert(C_HousingCatalog.GetBundleInfo(801).canPreview == false)")
        .unwrap();
}

#[test]
fn housing_bundle_invalid_selectors_do_not_mutate_records() {
    let env = housing_bundle_fixture();
    env.exec(
        r#"
        for _, v in ipairs({'801', false, {}, 801.25, math.huge, -math.huge, 0/0, 2147483648}) do
            assert(not pcall(C_HousingCatalog.GetBundleInfo, v))
            assert(not pcall(C_HousingCatalog.HousingMarketActionViewBundle, v))
        end
        assert(not pcall(C_HousingCatalog.GetBundleInfo))
        assert(not pcall(C_HousingCatalog.HousingMarketActionViewBundle))
        assert(C_HousingCatalog.GetBundleInfo(801).price == 19.25)
        assert(C_HousingCatalog.GetBundleInfo(801).wasViewed == false)
    "#,
    )
    .unwrap();
}
