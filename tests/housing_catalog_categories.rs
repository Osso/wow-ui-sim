//! Category DTO expectations only; no search/filter or derived ownership policy.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogCategoryRecord, HousingCatalogEntryVariantID, HousingCatalogSubcategoryRecord,
    HousingCatalogVariantRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn inject_categories(env: &WowLuaEnv) {
    let state = env.state();
    let mut state = state.borrow_mut();
    let catalog = &mut state.housing.catalog;
    catalog.categories = [
        (
            101,
            HousingCatalogCategoryRecord {
                order_index: 7,
                name: Some("Fixture category".into()),
                icon: Some("fixture-category-atlas".into()),
                subcategory_ids: vec![1002, 1001],
                any_stored_entries: true,
                editor_mode_contexts: vec![],
            },
        ),
        (
            102,
            HousingCatalogCategoryRecord {
                order_index: 7,
                name: Some("Fixture category".into()),
                icon: Some("fixture-category-atlas".into()),
                subcategory_ids: vec![1002, 1001],
                any_stored_entries: false,
                editor_mode_contexts: vec![],
            },
        ),
    ]
    .into();
    catalog.subcategories = [
        (
            1001,
            HousingCatalogSubcategoryRecord {
                order_index: 9,
                parent_category_id: 102,
                name: Some("Fixture subcategory".into()),
                icon: Some("fixture-subcategory-atlas".into()),
                any_stored_entries: true,
                editor_mode_contexts: vec![],
            },
        ),
        (
            1002,
            HousingCatalogSubcategoryRecord {
                order_index: 9,
                parent_category_id: 102,
                name: Some("Fixture subcategory".into()),
                icon: Some("fixture-subcategory-atlas".into()),
                any_stored_entries: false,
                editor_mode_contexts: vec![],
            },
        ),
    ]
    .into();
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    inject_categories(&env);
    env
}

#[test]
fn empty_category_ids_do_not_return_legacy_seeds() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, id in ipairs({101, 102, 1001}) do
            assert(C_HousingCatalog.GetCatalogCategoryInfo(id) == nil,
                'empty category map must not expose legacy seeds')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn empty_subcategory_ids_do_not_return_legacy_seeds() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, id in ipairs({101, 102, 1001, 1002}) do
            assert(C_HousingCatalog.GetCatalogSubcategoryInfo(id) == nil,
                'empty subcategory map must not expose legacy seeds')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn category_fields_preserve_explicit_true_and_false_under_new_name() {
    let env = fixture_env();
    env.exec(r#"
        for _, expected in ipairs({{101, true}, {102, false}}) do
            local info = C_HousingCatalog.GetCatalogCategoryInfo(expected[1])
            assert(info and info.ID == expected[1] and info.orderIndex == 7)
            assert(info.name == 'Fixture category' and info.icon == 'fixture-category-atlas')
            assert(type(info.subcategoryIDs) == 'table' and #info.subcategoryIDs == 2)
            assert(info.subcategoryIDs[1] == 1002 and info.subcategoryIDs[2] == 1001)
            assert(type(info.anyStoredEntries) == 'boolean' and info.anyStoredEntries == expected[2])
            assert(info.anyOwnedEntries == nil, 'retired field must be absent')
        end
    "#).unwrap();
}

#[test]
fn subcategory_fields_preserve_explicit_true_and_false_under_new_name() {
    let env = fixture_env();
    env.exec(r#"
        for _, expected in ipairs({{1001, true}, {1002, false}}) do
            local info = C_HousingCatalog.GetCatalogSubcategoryInfo(expected[1])
            assert(info and info.ID == expected[1] and info.orderIndex == 9)
            assert(info.parentCategoryID == 102)
            assert(info.name == 'Fixture subcategory' and info.icon == 'fixture-subcategory-atlas')
            assert(type(info.anyStoredEntries) == 'boolean' and info.anyStoredEntries == expected[2])
            assert(info.anyOwnedEntries == nil, 'retired field must be absent')
        end
    "#).unwrap();
}

#[test]
fn optional_fields_are_nullable_without_losing_required_fields() {
    let env = fixture_env();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let category = state.housing.catalog.categories.get_mut(&101).unwrap();
        category.name = None;
        category.icon = None;
        category.subcategory_ids.clear();
        let subcategory = state.housing.catalog.subcategories.get_mut(&1001).unwrap();
        subcategory.name = None;
        subcategory.icon = None;
    }
    env.exec(r#"
        local category = C_HousingCatalog.GetCatalogCategoryInfo(101)
        assert(category.ID == 101 and category.orderIndex == 7 and category.anyStoredEntries == true)
        assert(category.name == nil and category.icon == nil)
        assert(type(category.subcategoryIDs) == 'table' and #category.subcategoryIDs == 0)
        local subcategory = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
        assert(subcategory.ID == 1001 and subcategory.orderIndex == 9 and subcategory.parentCategoryID == 102)
        assert(subcategory.anyStoredEntries == true and subcategory.name == nil and subcategory.icon == nil)
    "#).unwrap();
}

#[test]
fn selectors_use_exact_id_and_never_cross_category_maps() {
    let env = fixture_env();
    env.exec(r#"
        for _, id in ipairs({1001, 1002, 9999}) do
            assert(C_HousingCatalog.GetCatalogCategoryInfo(id) == nil, 'category key must match exactly')
        end
        for _, id in ipairs({101, 102, 9999}) do
            assert(C_HousingCatalog.GetCatalogSubcategoryInfo(id) == nil, 'subcategory key must match exactly')
        end
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001).parentCategoryID == 102)
    "#).unwrap();
}

#[test]
fn stored_predicates_are_not_computed_from_partial_variant_storage() {
    let env = fixture_env();
    let entry_type = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    env.state().borrow_mut().housing.catalog.variants.insert(
        HousingCatalogEntryVariantID {
            record_id: 1002,
            entry_type,
            variant_identifier: 1,
        },
        HousingCatalogVariantRecord {
            num_stored: 25,
            destroyable_instance_count: 0,
            dye_slots: vec![],
        },
    );
    env.exec(
        r#"
        assert(C_HousingCatalog.GetCatalogCategoryInfo(101).anyStoredEntries == true)
        assert(C_HousingCatalog.GetCatalogCategoryInfo(102).anyStoredEntries == false)
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001).anyStoredEntries == true)
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1002).anyStoredEntries == false)
    "#,
    )
    .unwrap();
}

#[test]
fn returned_tables_and_nested_lists_are_fresh_snapshots() {
    let env = fixture_env();
    env.exec(r#"
        local first = C_HousingCatalog.GetCatalogCategoryInfo(101)
        local second = C_HousingCatalog.GetCatalogCategoryInfo(101)
        assert(first ~= second and first.subcategoryIDs ~= second.subcategoryIDs)
        first.name = 'Changed locally'; first.anyStoredEntries = false
        first.subcategoryIDs[1] = 9999; first.subcategoryIDs[3] = 8888
        local third = C_HousingCatalog.GetCatalogCategoryInfo(101)
        for _, info in ipairs({second, third}) do
            assert(info.name == 'Fixture category' and info.anyStoredEntries == true)
            assert(#info.subcategoryIDs == 2 and info.subcategoryIDs[1] == 1002 and info.subcategoryIDs[2] == 1001)
        end
        local subfirst = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
        local subsecond = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
        assert(subfirst ~= subsecond)
        subfirst.parentCategoryID = 9999; subfirst.name = 'Changed locally'; subfirst.anyStoredEntries = false
        local subthird = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
        for _, info in ipairs({subsecond, subthird}) do
            assert(info.parentCategoryID == 102 and info.name == 'Fixture subcategory' and info.anyStoredEntries == true)
        end
    "#).unwrap();
}

#[test]
fn record_mutation_changes_new_snapshots_only() {
    let env = fixture_env();
    env.exec(
        r#"
        OldCategory = C_HousingCatalog.GetCatalogCategoryInfo(101)
        OldSubcategory = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
    "#,
    )
    .unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let category = state.housing.catalog.categories.get_mut(&101).unwrap();
        category.order_index = 13;
        category.name = Some("Updated category".into());
        category.icon = None;
        category.subcategory_ids = vec![2001];
        category.any_stored_entries = false;
        let subcategory = state.housing.catalog.subcategories.get_mut(&1001).unwrap();
        subcategory.order_index = 14;
        subcategory.parent_category_id = 101;
        subcategory.name = None;
        subcategory.icon = Some("updated-subcategory-atlas".into());
        subcategory.any_stored_entries = false;
    }
    env.exec(r#"
        assert(OldCategory.ID == 101 and OldCategory.orderIndex == 7 and OldCategory.name == 'Fixture category')
        assert(OldCategory.icon == 'fixture-category-atlas' and OldCategory.anyStoredEntries == true)
        assert(#OldCategory.subcategoryIDs == 2 and OldCategory.subcategoryIDs[1] == 1002)
        assert(OldSubcategory.ID == 1001 and OldSubcategory.orderIndex == 9 and OldSubcategory.parentCategoryID == 102)
        assert(OldSubcategory.name == 'Fixture subcategory' and OldSubcategory.icon == 'fixture-subcategory-atlas')
        assert(OldSubcategory.anyStoredEntries == true)
        local category = C_HousingCatalog.GetCatalogCategoryInfo(101)
        assert(category.ID == 101 and category.orderIndex == 13 and category.name == 'Updated category')
        assert(category.icon == nil and category.anyStoredEntries == false)
        assert(#category.subcategoryIDs == 1 and category.subcategoryIDs[1] == 2001)
        local subcategory = C_HousingCatalog.GetCatalogSubcategoryInfo(1001)
        assert(subcategory.ID == 1001 and subcategory.orderIndex == 14 and subcategory.parentCategoryID == 101)
        assert(subcategory.name == nil and subcategory.icon == 'updated-subcategory-atlas' and subcategory.anyStoredEntries == false)
    "#).unwrap();
}

#[test]
fn records_are_isolated_between_environments() {
    let populated = fixture_env();
    let empty = WowLuaEnv::new().unwrap();
    populated
        .exec(
            r#"
        assert(C_HousingCatalog.GetCatalogCategoryInfo(101).name == 'Fixture category')
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001).name == 'Fixture subcategory')
    "#,
        )
        .unwrap();
    empty
        .exec(
            r#"
        assert(C_HousingCatalog.GetCatalogCategoryInfo(101) == nil)
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001) == nil)
    "#,
        )
        .unwrap();
    populated
        .exec(
            r#"
        assert(C_HousingCatalog.GetCatalogCategoryInfo(102).anyStoredEntries == false)
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1002).anyStoredEntries == false)
    "#,
        )
        .unwrap();
}

#[test]
fn public_numeric_selectors_preserve_caller_taint() {
    let env = fixture_env();
    env.exec(
        r#"
        local function addon()
            assert(not issecure())
            assert(C_HousingCatalog.GetCatalogCategoryInfo(101).ID == 101)
            assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001).ID == 1001)
            assert(not issecure(), 'queries must not clear caller taint')
        end
        debug.setobjecttaint(addon, 'HousingCategoryFixture')
        assert(issecure()); addon(); assert(issecure())
    "#,
    )
    .unwrap();
}

#[test]
fn secret_numeric_selectors_reject_without_unwrapping_or_taint_changes() {
    let env = fixture_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        for (name, number) in [("SecretCategory", 101.0), ("SecretSubcategory", 1001.0)] {
            let secret = wrap_host_secret_number(lua.state_mut(), number);
            lua.state_mut().push(secret);
            let inserted = lua.set_global_val(name, secret);
            lua.state_mut().pop();
            inserted.unwrap();
        }
    }
    env.exec(r#"
        collectgarbage('collect')
        local function reject()
            assert(not pcall(C_HousingCatalog.GetCatalogCategoryInfo, SecretCategory), 'secret category must reject')
            assert(not pcall(C_HousingCatalog.GetCatalogSubcategoryInfo, SecretSubcategory), 'secret subcategory must reject')
            assert(issecretvalue(SecretCategory) and issecretvalue(SecretSubcategory))
        end
        assert(issecure()); reject(); assert(issecure())
        local function addon()
            assert(not issecure()); reject()
            assert(not issecure(), 'rejection must not clear caller taint')
        end
        debug.setobjecttaint(addon, 'HousingCategoryFixture')
        addon()
        assert(issecure()); reject(); assert(issecure())
        assert(C_HousingCatalog.GetCatalogCategoryInfo(101).ID == 101)
        assert(C_HousingCatalog.GetCatalogSubcategoryInfo(1001).ID == 1001)
    "#).unwrap();
}
