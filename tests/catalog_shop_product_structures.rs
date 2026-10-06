#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_catalog_shop_products::{
    CatalogShopProductDisplayInfo, CatalogShopProductInfo, CatalogShopSubItemInfo,
    CatalogShopVirtualCurrency, DecorQuantity,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn product_record() -> CatalogShopProductInfo {
    CatalogShopProductInfo {
        catalog_shop_product_id: 81001.0,
        name: "fixture://name".into(),
        r#type: Some("fixture://type".into()),
        description: "fixture://description".into(),
        icon_texture: "fixture://iconTexture".into(),
        is_fully_owned: true,
        is_purchase_pending: false,
        refundable: true,
        price: "fixture://price".into(),
        original_price: "fixture://originalPrice".into(),
        discount_percentage: 110.25,
        item_id: 111.25,
        mount_id: 112.25,
        mount_type_name: "fixture://mountTypeName".into(),
        species_id: 114.25,
        transmog_set_id: 115.25,
        item_modified_appearance_id: 116.25,
        sub_items: vec![CatalogShopSubItemInfo {
        name: "fixture://name".into(),
        item_id: 101.25,
        item_appearance_id: 102.25,
        inv_type: "fixture://invType".into(),
        quality: 3,
    }, CatalogShopSubItemInfo {
        name: "fixture://name".into(),
        item_id: 101.25,
        item_appearance_id: 102.25,
        inv_type: "fixture://invType".into(),
        quality: 3,
    }],
        sub_items_loaded: false,
        background_texture: "fixture://backgroundTexture".into(),
        foreground_texture: Some("fixture://foregroundTexture".into()),
        small_card_bg_texture: Some("fixture://smallCardBGTexture".into()),
        small_card_fg_texture: Some("fixture://smallCardFGTexture".into()),
        wide_card_bg_texture: Some("fixture://wideCardBGTexture".into()),
        wide_card_fg_texture: Some("fixture://wideCardFGTexture".into()),
        preview_icon_texture: Some("fixture://previewIconTexture".into()),
        optional_wide_card_background_texture: Some("fixture://optionalWideCardBackgroundTexture".into()),
        is_bundle: true,
        bundle_children_size: 128.25,
        license_term_type: 129.25,
        license_term_duration: 130.25,
        virtual_currencies: vec![CatalogShopVirtualCurrency {
        amount: 100.25,
        currency_code: "fixture://currencyCode".into(),
    }, CatalogShopVirtualCurrency {
        amount: 100.25,
        currency_code: "fixture://currencyCode".into(),
    }],
        is_hidden: false,
        is_mystery: true,
        has_pending_orders: false,
        num_bundle_detail_cards: 135.25,
        is_dynamically_discounted: false,
        should_show_original_price: true,
        wide_card_bg_override_product_url: Some("fixture://wideCardBGOverrideProductURL".into()),
        preview_bg_override_product_url: Some("fixture://previewBGOverrideProductURL".into()),
        preview_small_bg_override_product_url: Some("fixture://previewSmallBGOverrideProductURL".into()),
        decor_quantity: Some(DecorQuantity { placed_quantity: 3.0, stored_quantity: 7.0 }),
        is_vc_product: false,
        contains_housing_item: true,
        deferred_game_time_days: 144.25,
    }
}

fn display_record() -> CatalogShopProductDisplayInfo {
    CatalogShopProductDisplayInfo {
        default_preview_model_scene_id: 100.25,
        default_card_model_scene_id: 101.25,
        default_wide_card_model_scene_id: 102.25,
        item_id: 103.25,
        override_preview_model_scene_id: Some(104.25),
        override_card_model_scene_id: Some(105.25),
        override_wide_card_model_scene_id: Some(106.25),
        creature_display_info_ids: vec![41.5, 42.5],
        spell_visual_ids: vec![41.5, 42.5],
        main_hand_item_modified_appearance_id: Some(109.25),
        off_hand_item_modified_appearance_id: Some(110.25),
        item_modified_appearance_ids: vec![41.5, 42.5],
        icon_file_data_id: Some(112.25),
        icon_texture_kit: Some("fixture://iconTextureKit".into()),
        product_type: Some("fixture://productType".into()),
        item_description: Some("fixture://itemDescription".into()),
        has_unknown_license: false,
        product_pmt_url: Some("fixture://productPMTURL".into()),
        additional_product_pmt_urls: vec!["fixture://first".into(), "fixture://second".into()],
        other_product_image_atlas_name: Some("fixture://otherProductImageAtlasName".into()),
        other_product_game_title_base_tag: Some("fixture://otherProductGameTitleBaseTag".into()),
        other_product_game_type: Some("fixture://otherProductGameType".into()),
        custom_looping_sound_start: Some(122.25),
        custom_looping_sound_middle: Some(123.25),
        custom_looping_sound_end: Some(124.25),
        special_actor_id_1: Some("fixture://specialActorID_1".into()),
        special_actor_id_2: Some("fixture://specialActorID_2".into()),
        special_actor_id_3: Some("fixture://specialActorID_3".into()),
        special_actor_id_4: Some("fixture://specialActorID_4".into()),
        special_actor_id_5: Some("fixture://specialActorID_5".into()),
        game_flavor_id: Some(130.25),
        decor_file_data_id: Some(131.25),
        quantity: Some(132.25),
        house_texture_atlas: Some("fixture://houseTextureAtlas".into()),
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().catalog_shop_products.products.insert(81001, product_record());
    env.state().borrow_mut().catalog_shop_products.displays.insert(81001, display_record());
    env
}

const EXACT: &str = r#"
    function exact(actual, expected)
        assert(type(actual) == type(expected), 'wrong Lua type')
        if type(expected) ~= 'table' then
            assert(actual == expected, 'wrong value')
            return
        end
        for key, value in pairs(expected) do exact(actual[key], value) end
        for key in pairs(actual) do
            assert(rawget(expected, key) ~= nil, 'unexpected key: '..tostring(key))
        end
    end
"#;

#[test]
fn catalog_shop_patch_12_0_1_empty_bundle_and_section_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(next(C_CatalogShop.GetProductIDsForBundle(2003)) == nil, 'no fabricated bundle children')
        local ok, err = pcall(C_CatalogShop.GetCategorySectionInfo, 101, 1)
        assert(not ok and type(err) == 'string', 'missing nonnullable section must report missing host input')
    "#).unwrap();
}

#[test]
fn catalog_shop_patch_12_0_1_bundle_section_snapshots() {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.catalog_shop_products.bundle_children.insert(81001, vec![
            CatalogShopBundleChildInfo { child_product_id: 81002.0, display_order: 7.0, quantity_in_bundle: 3.0 },
            CatalogShopBundleChildInfo { child_product_id: 81003.0, display_order: 2.0, quantity_in_bundle: 9.0 },
        ]);
        state.catalog_shop_products.sections.insert((101, 8), CatalogShopSectionInfo {
            id: 8.0, display_name: "Recommended decor".into(),
            parent_catalog_shop_category_info_id: Some(101.0), card_type: Some("Wide".into()),
            scroll_grid_size: Some(4.0), should_show_recommendation_opt_out_disclaimer: true,
        });
    }
    env.exec(EXACT).unwrap();
    env.exec(r#"
        ExpectedChildren = {
            {childProductID=81002, displayOrder=7, quantityInBundle=3},
            {childProductID=81003, displayOrder=2, quantityInBundle=9},
        }
        ExpectedSection = {ID=8, displayName='Recommended decor', parentCatalogShopCategoryInfoID=101,
            cardType='Wide', scrollGridSize=4, shouldShowRecommendationOptOutDisclaimer=true}
        Bundle = C_CatalogShop.GetProductIDsForBundle(81001)
        Section = C_CatalogShop.GetCategorySectionInfo(101, 8)
        exact(Bundle, ExpectedChildren); exact(Section, ExpectedSection)
        local edited = C_CatalogShop.GetProductIDsForBundle(81001)
        edited[1].quantityInBundle = -1; table.remove(edited, 2)
        local editedSection = C_CatalogShop.GetCategorySectionInfo(101, 8)
        editedSection.shouldShowRecommendationOptOutDisclaimer = false
        exact(C_CatalogShop.GetProductIDsForBundle(81001), ExpectedChildren)
        exact(C_CatalogShop.GetCategorySectionInfo(101, 8), ExpectedSection)
        local d = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)
        assert(rawget(d, 'otherProductPMTURL') == nil and d.otherProductPMTURL == nil)
        assert(d.productPMTURL == 'fixture://productPMTURL' and not issecretvalue(d.productPMTURL))
    "#).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.catalog_shop_products.bundle_children.get_mut(&81001).unwrap()[0].quantity_in_bundle = 5.0;
        let section = state.catalog_shop_products.sections.get_mut(&(101, 8)).unwrap();
        section.should_show_recommendation_opt_out_disclaimer = false;
        section.parent_catalog_shop_category_info_id = None;
        section.card_type = None;
        section.scroll_grid_size = None;
    }
    env.exec(r#"
        assert(Bundle[1].quantityInBundle == 3 and Section.shouldShowRecommendationOptOutDisclaimer)
        ExpectedChildren[1].quantityInBundle = 5
        exact(C_CatalogShop.GetProductIDsForBundle(81001), ExpectedChildren)
        exact(C_CatalogShop.GetCategorySectionInfo(101, 8),
            {ID=8, displayName='Recommended decor', shouldShowRecommendationOptOutDisclaimer=false})
        assert(next(C_CatalogShop.GetProductIDsForBundle(81002)) == nil)
        assert(not pcall(C_CatalogShop.GetCategorySectionInfo, 102, 8))
    "#).unwrap();
}

#[test]
fn additional_product_pmt_urls_preserve_order_and_string_type() {
    let env = fixture_env();
    env.exec(r#"
        local d = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)
        assert(type(d.additionalProductPMTURLs) == 'table')
        assert(#d.additionalProductPMTURLs == 2)
        assert(type(d.additionalProductPMTURLs[1]) == 'string' and d.additionalProductPMTURLs[1] == 'fixture://first')
        assert(type(d.additionalProductPMTURLs[2]) == 'string' and d.additionalProductPMTURLs[2] == 'fixture://second')
    "#).unwrap();
}

#[test]
fn house_texture_atlas_round_trips() {
    let env = fixture_env();
    env.exec(r#"
        local d = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)
        assert(type(d.houseTextureAtlas) == 'string' and d.houseTextureAtlas == 'fixture://houseTextureAtlas')
    "#).unwrap();
}

#[test]
fn preview_bg_override_product_url_round_trips() {
    let env = fixture_env();
    env.exec(r#"
        local p = C_CatalogShop.GetProductInfo(81001)
        assert(type(p.previewBGOverrideProductURL) == 'string' and p.previewBGOverrideProductURL == 'fixture://previewBGOverrideProductURL')
    "#).unwrap();
}

#[test]
fn preview_small_bg_override_product_url_round_trips() {
    let env = fixture_env();
    env.exec(r#"
        local p = C_CatalogShop.GetProductInfo(81001)
        assert(type(p.previewSmallBGOverrideProductURL) == 'string' and p.previewSmallBGOverrideProductURL == 'fixture://previewSmallBGOverrideProductURL')
    "#).unwrap();
}

#[test]
fn decor_quantity_round_trips_nested_numbers() {
    let env = fixture_env();
    env.exec(r#"
        local p = C_CatalogShop.GetProductInfo(81001)
        assert(type(p.decorQuantity) == 'table')
        assert(type(p.decorQuantity.placedQuantity) == 'number' and p.decorQuantity.placedQuantity == 3)
        assert(type(p.decorQuantity.storedQuantity) == 'number' and p.decorQuantity.storedQuantity == 7)
    "#).unwrap();
}

#[test]
fn consumable_quantity_is_absent_from_populated_product() {
    let env = fixture_env();
    env.exec(r#"
        assert(rawget(C_CatalogShop.GetProductInfo(81001), 'consumableQuantity') == nil)
    "#).unwrap();
}

#[test]
fn fully_populated_product_has_exact_declared_shape() {
    let env = fixture_env();
    env.exec(EXACT).unwrap();
    env.exec(r#"
    local expected = {
        catalogShopProductID = 81001,
        name = 'fixture://name',
        ['type'] = 'fixture://type',
        description = 'fixture://description',
        iconTexture = 'fixture://iconTexture',
        isFullyOwned = true,
        isPurchasePending = false,
        refundable = true,
        price = 'fixture://price',
        originalPrice = 'fixture://originalPrice',
        discountPercentage = 110.25,
        itemID = 111.25,
        mountID = 112.25,
        mountTypeName = 'fixture://mountTypeName',
        speciesID = 114.25,
        transmogSetID = 115.25,
        itemModifiedAppearanceID = 116.25,
        subItems = {{
        name = 'fixture://name',
        itemID = 101.25,
        itemAppearanceID = 102.25,
        invType = 'fixture://invType',
        quality = 3,
    }, {
        name = 'fixture://name',
        itemID = 101.25,
        itemAppearanceID = 102.25,
        invType = 'fixture://invType',
        quality = 3,
    }},
        subItemsLoaded = false,
        backgroundTexture = 'fixture://backgroundTexture',
        foregroundTexture = 'fixture://foregroundTexture',
        smallCardBGTexture = 'fixture://smallCardBGTexture',
        smallCardFGTexture = 'fixture://smallCardFGTexture',
        wideCardBGTexture = 'fixture://wideCardBGTexture',
        wideCardFGTexture = 'fixture://wideCardFGTexture',
        previewIconTexture = 'fixture://previewIconTexture',
        optionalWideCardBackgroundTexture = 'fixture://optionalWideCardBackgroundTexture',
        isBundle = true,
        bundleChildrenSize = 128.25,
        licenseTermType = 129.25,
        licenseTermDuration = 130.25,
        virtualCurrencies = {{
        amount = 100.25,
        currencyCode = 'fixture://currencyCode',
    }, {
        amount = 100.25,
        currencyCode = 'fixture://currencyCode',
    }},
        isHidden = false,
        isMystery = true,
        hasPendingOrders = false,
        numBundleDetailCards = 135.25,
        isDynamicallyDiscounted = false,
        shouldShowOriginalPrice = true,
        wideCardBGOverrideProductURL = 'fixture://wideCardBGOverrideProductURL',
        previewBGOverrideProductURL = 'fixture://previewBGOverrideProductURL',
        previewSmallBGOverrideProductURL = 'fixture://previewSmallBGOverrideProductURL',
        decorQuantity = {placedQuantity = 3, storedQuantity = 7},
        isVCProduct = false,
        containsHousingItem = true,
        deferredGameTimeDays = 144.25,
    }
    exact(C_CatalogShop.GetProductInfo(81001), expected)
    "#).unwrap();
}

#[test]
fn fully_populated_display_has_exact_declared_shape() {
    let env = fixture_env();
    env.exec(EXACT).unwrap();
    env.exec(r#"
    local expected = {
        defaultPreviewModelSceneID = 100.25,
        defaultCardModelSceneID = 101.25,
        defaultWideCardModelSceneID = 102.25,
        itemID = 103.25,
        overridePreviewModelSceneID = 104.25,
        overrideCardModelSceneID = 105.25,
        overrideWideCardModelSceneID = 106.25,
        creatureDisplayInfoIDs = {41.5, 42.5},
        spellVisualIDs = {41.5, 42.5},
        mainHandItemModifiedAppearanceID = 109.25,
        offHandItemModifiedAppearanceID = 110.25,
        itemModifiedAppearanceIDs = {41.5, 42.5},
        iconFileDataID = 112.25,
        iconTextureKit = 'fixture://iconTextureKit',
        productType = 'fixture://productType',
        itemDescription = 'fixture://itemDescription',
        hasUnknownLicense = false,
        productPMTURL = 'fixture://productPMTURL',
        additionalProductPMTURLs = {'fixture://first', 'fixture://second'},
        otherProductImageAtlasName = 'fixture://otherProductImageAtlasName',
        otherProductGameTitleBaseTag = 'fixture://otherProductGameTitleBaseTag',
        otherProductGameType = 'fixture://otherProductGameType',
        customLoopingSoundStart = 122.25,
        customLoopingSoundMiddle = 123.25,
        customLoopingSoundEnd = 124.25,
        specialActorID_1 = 'fixture://specialActorID_1',
        specialActorID_2 = 'fixture://specialActorID_2',
        specialActorID_3 = 'fixture://specialActorID_3',
        specialActorID_4 = 'fixture://specialActorID_4',
        specialActorID_5 = 'fixture://specialActorID_5',
        gameFlavorID = 130.25,
        decorFileDataID = 131.25,
        quantity = 132.25,
        houseTextureAtlas = 'fixture://houseTextureAtlas',
    }
    exact(C_CatalogShop.GetCatalogShopProductDisplayInfo(81001), expected)
    "#).unwrap();
}

#[test]
fn all_nullable_fields_are_absent_and_required_arrays_exist() {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        let record = state.catalog_shop_products.products.get_mut(&81001).unwrap();
        record.r#type = None;
        record.sub_items.clear();
        record.foreground_texture = None;
        record.small_card_bg_texture = None;
        record.small_card_fg_texture = None;
        record.wide_card_bg_texture = None;
        record.wide_card_fg_texture = None;
        record.preview_icon_texture = None;
        record.optional_wide_card_background_texture = None;
        record.virtual_currencies.clear();
        record.wide_card_bg_override_product_url = None;
        record.preview_bg_override_product_url = None;
        record.preview_small_bg_override_product_url = None;
        record.decor_quantity = None;
    }
    {
        let mut state = env.state().borrow_mut();
        let record = state.catalog_shop_products.displays.get_mut(&81001).unwrap();
        record.override_preview_model_scene_id = None;
        record.override_card_model_scene_id = None;
        record.override_wide_card_model_scene_id = None;
        record.creature_display_info_ids.clear();
        record.spell_visual_ids.clear();
        record.main_hand_item_modified_appearance_id = None;
        record.off_hand_item_modified_appearance_id = None;
        record.item_modified_appearance_ids.clear();
        record.icon_file_data_id = None;
        record.icon_texture_kit = None;
        record.product_type = None;
        record.item_description = None;
        record.product_pmt_url = None;
        record.additional_product_pmt_urls.clear();
        record.other_product_image_atlas_name = None;
        record.other_product_game_title_base_tag = None;
        record.other_product_game_type = None;
        record.custom_looping_sound_start = None;
        record.custom_looping_sound_middle = None;
        record.custom_looping_sound_end = None;
        record.special_actor_id_1 = None;
        record.special_actor_id_2 = None;
        record.special_actor_id_3 = None;
        record.special_actor_id_4 = None;
        record.special_actor_id_5 = None;
        record.game_flavor_id = None;
        record.decor_file_data_id = None;
        record.quantity = None;
        record.house_texture_atlas = None;
    }
    env.exec(r#"
        local p = C_CatalogShop.GetProductInfo(81001)
        local d = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)
        assert(rawget(p, 'type') == nil)
        assert(type(p.subItems) == 'table' and next(p.subItems) == nil)
        assert(rawget(p, 'foregroundTexture') == nil)
        assert(rawget(p, 'smallCardBGTexture') == nil)
        assert(rawget(p, 'smallCardFGTexture') == nil)
        assert(rawget(p, 'wideCardBGTexture') == nil)
        assert(rawget(p, 'wideCardFGTexture') == nil)
        assert(rawget(p, 'previewIconTexture') == nil)
        assert(rawget(p, 'optionalWideCardBackgroundTexture') == nil)
        assert(type(p.virtualCurrencies) == 'table' and next(p.virtualCurrencies) == nil)
        assert(rawget(p, 'wideCardBGOverrideProductURL') == nil)
        assert(rawget(p, 'previewBGOverrideProductURL') == nil)
        assert(rawget(p, 'previewSmallBGOverrideProductURL') == nil)
        assert(rawget(p, 'decorQuantity') == nil)
        assert(rawget(d, 'overridePreviewModelSceneID') == nil)
        assert(rawget(d, 'overrideCardModelSceneID') == nil)
        assert(rawget(d, 'overrideWideCardModelSceneID') == nil)
        assert(type(d.creatureDisplayInfoIDs) == 'table' and next(d.creatureDisplayInfoIDs) == nil)
        assert(type(d.spellVisualIDs) == 'table' and next(d.spellVisualIDs) == nil)
        assert(rawget(d, 'mainHandItemModifiedAppearanceID') == nil)
        assert(rawget(d, 'offHandItemModifiedAppearanceID') == nil)
        assert(type(d.itemModifiedAppearanceIDs) == 'table' and next(d.itemModifiedAppearanceIDs) == nil)
        assert(rawget(d, 'iconFileDataID') == nil)
        assert(rawget(d, 'iconTextureKit') == nil)
        assert(rawget(d, 'productType') == nil)
        assert(rawget(d, 'itemDescription') == nil)
        assert(rawget(d, 'productPMTURL') == nil)
        assert(type(d.additionalProductPMTURLs) == 'table' and next(d.additionalProductPMTURLs) == nil)
        assert(rawget(d, 'otherProductImageAtlasName') == nil)
        assert(rawget(d, 'otherProductGameTitleBaseTag') == nil)
        assert(rawget(d, 'otherProductGameType') == nil)
        assert(rawget(d, 'customLoopingSoundStart') == nil)
        assert(rawget(d, 'customLoopingSoundMiddle') == nil)
        assert(rawget(d, 'customLoopingSoundEnd') == nil)
        assert(rawget(d, 'specialActorID_1') == nil)
        assert(rawget(d, 'specialActorID_2') == nil)
        assert(rawget(d, 'specialActorID_3') == nil)
        assert(rawget(d, 'specialActorID_4') == nil)
        assert(rawget(d, 'specialActorID_5') == nil)
        assert(rawget(d, 'gameFlavorID') == nil)
        assert(rawget(d, 'decorFileDataID') == nil)
        assert(rawget(d, 'quantity') == nil)
        assert(rawget(d, 'houseTextureAtlas') == nil)
    "#).unwrap();
}

#[test]
fn unknown_ids_and_legacy_seed_ids_return_nil() {
    let env = WowLuaEnv::new().unwrap();
    // INFERRED: display misses return nil despite the declared nonnullable result.
    env.exec(r#"
        for _, id in ipairs({2003, 20031, 20032, 81001, -17}) do
            assert(select('#', C_CatalogShop.GetProductInfo(id)) == 1)
            assert(C_CatalogShop.GetProductInfo(id) == nil)
            assert(select('#', C_CatalogShop.GetCatalogShopProductDisplayInfo(id)) == 1)
            assert(C_CatalogShop.GetCatalogShopProductDisplayInfo(id) == nil)
        end
    "#).unwrap();
}

#[test]
fn returned_tables_and_every_nested_array_are_independent() {
    let env = fixture_env();
    env.exec(EXACT).unwrap();
    // INFERRED: fresh nested snapshots, not native-verified aliasing semantics.
    env.exec(r#"
        local p = C_CatalogShop.GetProductInfo(81001)
        local q = C_CatalogShop.GetProductInfo(81001)
        local d = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)
        local e = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)
        assert(p ~= q and d ~= e)
        assert(p.decorQuantity ~= q.decorQuantity)
        assert(p.subItems ~= q.subItems and p.subItems[1] ~= q.subItems[1])
        assert(p.virtualCurrencies ~= q.virtualCurrencies)
        assert(p.virtualCurrencies[1] ~= q.virtualCurrencies[1])
        p.name = 'changed'
        p.decorQuantity.storedQuantity = 999
        p.subItems[1].name = 'changed child'
        p.virtualCurrencies[1].amount = 999
        d.houseTextureAtlas = 'changed atlas'
        for _, field in ipairs({'creatureDisplayInfoIDs', 'spellVisualIDs',
            'itemModifiedAppearanceIDs', 'additionalProductPMTURLs'}) do
            assert(d[field] ~= e[field])
            d[field][1] = 'changed element'
        end
        collectgarbage('collect')
        exact(q, C_CatalogShop.GetProductInfo(81001))
        exact(e, C_CatalogShop.GetCatalogShopProductDisplayInfo(81001))
    "#).unwrap();
    let state = env.state().borrow();
    assert_eq!(state.catalog_shop_products.products[&81001].decor_quantity.as_ref().unwrap().stored_quantity, 7.0);
    assert_eq!(state.catalog_shop_products.products[&81001].sub_items[0].name, "fixture://name");
    assert_eq!(state.catalog_shop_products.displays[&81001].additional_product_pmt_urls[0], "fixture://first");
}

#[test]
fn host_replacements_affect_new_snapshots_only() {
    let env = fixture_env();
    env.exec("oldProduct = C_CatalogShop.GetProductInfo(81001); oldDisplay = C_CatalogShop.GetCatalogShopProductDisplayInfo(81001)").unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.catalog_shop_products.products.get_mut(&81001).unwrap().preview_bg_override_product_url = None;
        state.catalog_shop_products.displays.get_mut(&81001).unwrap().additional_product_pmt_urls = vec!["fixture://replacement".into()];
    }
    env.exec(r#"
        assert(oldProduct.previewBGOverrideProductURL == 'fixture://previewBGOverrideProductURL')
        assert(rawget(C_CatalogShop.GetProductInfo(81001), 'previewBGOverrideProductURL') == nil)
        assert(oldDisplay.additionalProductPMTURLs[1] == 'fixture://first')
        assert(C_CatalogShop.GetCatalogShopProductDisplayInfo(81001).additionalProductPMTURLs[1] == 'fixture://replacement')
    "#).unwrap();
    env.state().borrow_mut().catalog_shop_products.products.remove(&81001);
    env.state().borrow_mut().catalog_shop_products.displays.remove(&81001);
    env.exec("assert(C_CatalogShop.GetProductInfo(81001) == nil and C_CatalogShop.GetCatalogShopProductDisplayInfo(81001) == nil)").unwrap();
}

#[test]
fn maps_use_exact_keys_without_product_display_cross_lookup() {
    let env = fixture_env();
    let mut other = product_record();
    other.name = "different product".into();
    env.state().borrow_mut().catalog_shop_products.products.insert(81002, other);
    env.state().borrow_mut().catalog_shop_products.displays.insert(81003, display_record());
    env.exec(r#"
        assert(C_CatalogShop.GetProductInfo(81002).name == 'different product')
        assert(C_CatalogShop.GetProductInfo(81001).name == 'fixture://name')
        assert(C_CatalogShop.GetCatalogShopProductDisplayInfo(81002) == nil)
        assert(C_CatalogShop.GetProductInfo(81003) == nil)
        assert(type(C_CatalogShop.GetCatalogShopProductDisplayInfo(81003)) == 'table')
    "#).unwrap();
}

#[test]
fn environments_are_isolated() {
    let first = fixture_env();
    let second = WowLuaEnv::new().unwrap();
    second.exec("assert(C_CatalogShop.GetProductInfo(81001) == nil and C_CatalogShop.GetCatalogShopProductDisplayInfo(81001) == nil)").unwrap();
    let mut product = product_record();
    product.name = "second environment".into();
    let mut display = display_record();
    display.house_texture_atlas = Some("second-atlas".into());
    second.state().borrow_mut().catalog_shop_products.products.insert(81001, product);
    second.state().borrow_mut().catalog_shop_products.displays.insert(81001, display);
    first.exec("assert(C_CatalogShop.GetProductInfo(81001).name == 'fixture://name'); assert(C_CatalogShop.GetCatalogShopProductDisplayInfo(81001).houseTextureAtlas == 'fixture://houseTextureAtlas')").unwrap();
    second.exec("assert(C_CatalogShop.GetProductInfo(81001).name == 'second environment'); assert(C_CatalogShop.GetCatalogShopProductDisplayInfo(81001).houseTextureAtlas == 'second-atlas')").unwrap();
}

#[test]
fn zero_decor_counts_are_present_not_nil() {
    let env = fixture_env();
    env.state().borrow_mut().catalog_shop_products.products.get_mut(&81001).unwrap().decor_quantity = Some(DecorQuantity::default());
    env.exec("local q = C_CatalogShop.GetProductInfo(81001).decorQuantity; assert(type(q) == 'table' and q.placedQuantity == 0 and q.storedQuantity == 0)").unwrap();
}

#[test]
fn malformed_public_selectors_do_not_alias_stored_products() {
    let env = fixture_env();
    // INFERRED: exact public i32 selectors, not native argument validation proof.
    env.exec(r#"
        for _, getter in ipairs({C_CatalogShop.GetProductInfo,
            C_CatalogShop.GetCatalogShopProductDisplayInfo}) do
            for _, id in ipairs({'81001', 81001.5, 2147483648, math.huge, {}}) do
                local ok = pcall(getter, id)
                assert(not ok, 'malformed selector must not resolve a stored record')
            end
            assert(not pcall(getter, nil))
            assert(not pcall(getter, 0 / 0))
        end
    "#).unwrap();
}
