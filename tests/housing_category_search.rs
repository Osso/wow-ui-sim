//! Batch26 input-only search expectations. Parent must observe RED before production changes.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_bool, wrap_host_secret_number};
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogCategoryRecord, HousingCatalogEntryVariantID, HousingCatalogSubcategoryRecord,
    HousingCatalogVariantRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn search_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        function assertIDs(actual, expected)
            assert(type(actual) == 'table' and #actual == #expected, 'unexpected result size')
            for index, id in ipairs(expected) do
                assert(actual[index] == id, 'unexpected ID or order at ' .. index)
            end
        end
        SearchQueries = {C_HousingCatalog.SearchCatalogCategories, C_HousingCatalog.SearchCatalogSubcategories}
    "#).unwrap();
    env
}

fn category(order_index: i32, stored: bool, contexts: Vec<i32>) -> HousingCatalogCategoryRecord {
    HousingCatalogCategoryRecord {
        order_index,
        name: Some("Unclassified host fixture".into()),
        icon: None,
        subcategory_ids: vec![],
        any_stored_entries: stored,
        editor_mode_contexts: contexts,
    }
}

fn subcategory(
    order_index: i32,
    parent_category_id: i32,
    stored: bool,
    contexts: Vec<i32>,
) -> HousingCatalogSubcategoryRecord {
    HousingCatalogSubcategoryRecord {
        order_index,
        parent_category_id,
        name: Some("Unclassified host fixture".into()),
        icon: None,
        any_stored_entries: stored,
        editor_mode_contexts: contexts,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = search_env();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let catalog = &mut state.housing.catalog;
        // Arbitrary explicit host modes, not native enum associations or label guesses.
        catalog.categories = [
            (403, category(9, false, vec![])),
            (401, category(9, true, vec![-37, 64])),
            (402, category(2, false, vec![64])),
        ]
        .into();
        // Parent 8888 is deliberately absent; no inferred graph constraint.
        catalog.subcategories = [
            (503, subcategory(9, 8888, false, vec![])),
            (501, subcategory(9, 8888, true, vec![-37, 64])),
            (502, subcategory(2, 401, false, vec![64])),
        ]
        .into();
    }
    env
}

#[test]
fn empty_maps_return_fresh_empty_arrays_without_seed_ids() {
    let env = search_env();
    env.exec(
        r#"
        for _, query in ipairs(SearchQueries) do
            local first = query({})
            local second = query({includeFeaturedCategory = true, withStoredEntriesOnly = true})
            assertIDs(first, {}); assertIDs(second, {})
            assert(first ~= second, 'empty arrays must be fresh')
            first[1] = 9999
            assertIDs(query({includeFeaturedCategory = true}), {})
        end
    "#,
    )
    .unwrap();
}

#[test]
fn stored_filter_uses_explicit_boolean_and_raw_api_ignores_removed_key() {
    let env = fixture_env();
    env.exec(r#"
        local expected = {{402, 401, 403}, {502, 501, 503}}
        local stored = {{401}, {501}}
        for index, query in ipairs(SearchQueries) do
            assertIDs(query({}), expected[index])
            assertIDs(query({withStoredEntriesOnly = false}), expected[index])
            assertIDs(query({withStoredEntriesOnly = true}), stored[index])
            assertIDs(query({withOwnedEntriesOnly = true}), expected[index])
            assertIDs(query({withStoredEntriesOnly = false, withOwnedEntriesOnly = true}), expected[index])
        end
    "#).unwrap();
}

#[test]
fn featured_filter_only_selects_existing_records_and_explicit_parent_ids() {
    let env = fixture_env();
    let featured: i32 = env
        .eval("return Constants.HousingCatalogConsts.HOUSING_CATALOG_FEATURED_CATEGORY_ID")
        .unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state
            .housing
            .catalog
            .subcategories
            .insert(504, subcategory(1, featured, true, vec![-37]));
    }
    // Featured subcategory policy uses its explicit parent ID, even without that parent record.
    env.exec(r#"
        assertIDs(C_HousingCatalog.SearchCatalogCategories({includeFeaturedCategory = true}), {402, 401, 403})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({}), {502, 501, 503})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({includeFeaturedCategory = false}), {502, 501, 503})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({includeFeaturedCategory = true}), {504, 502, 501, 503})
    "#).unwrap();
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .categories
        .insert(featured, category(1, true, vec![-37]));
    env.exec(r#"
        local featured = Constants.HousingCatalogConsts.HOUSING_CATALOG_FEATURED_CATEGORY_ID
        assertIDs(C_HousingCatalog.SearchCatalogCategories({}), {402, 401, 403})
        assertIDs(C_HousingCatalog.SearchCatalogCategories({includeFeaturedCategory = false}), {402, 401, 403})
        assertIDs(C_HousingCatalog.SearchCatalogCategories({includeFeaturedCategory = true}), {featured, 402, 401, 403})
        assertIDs(C_HousingCatalog.SearchCatalogCategories({includeFeaturedCategory = true,
            withStoredEntriesOnly = true, editorModeContext = -37}), {featured, 401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({includeFeaturedCategory = true,
            withStoredEntriesOnly = true, editorModeContext = -37}), {504, 501})
    "#).unwrap();
}

#[test]
fn context_filter_matches_exact_host_membership_without_parent_or_label_inference() {
    let env = fixture_env();
    env.exec(r#"
        local all = {{402, 401, 403}, {502, 501, 503}}
        local mode64 = {{402, 401}, {502, 501}}
        local modeNegative = {{401}, {501}}
        for index, query in ipairs(SearchQueries) do
            assertIDs(query({editorModeContext = nil}), all[index])
            assertIDs(query({editorModeContext = 64}), mode64[index])
            assertIDs(query({editorModeContext = -37}), modeNegative[index])
            assertIDs(query({editorModeContext = 65}), {})
            assertIDs(query({editorModeContext = 64, withStoredEntriesOnly = true}), modeNegative[index])
        end
    "#).unwrap();
}

#[test]
fn ordering_is_order_index_then_numeric_id_after_state_changes() {
    let env = fixture_env();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state
            .housing
            .catalog
            .categories
            .insert(49, category(9, false, vec![]));
        state
            .housing
            .catalog
            .subcategories
            .insert(59, subcategory(9, 8888, false, vec![]));
    }
    env.exec(
        r#"
        assertIDs(C_HousingCatalog.SearchCatalogCategories({}), {402, 49, 401, 403})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({}), {502, 59, 501, 503})
    "#,
    )
    .unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state
            .housing
            .catalog
            .categories
            .get_mut(&403)
            .unwrap()
            .order_index = -1;
        state
            .housing
            .catalog
            .subcategories
            .get_mut(&503)
            .unwrap()
            .order_index = -1;
    }
    env.exec(
        r#"
        assertIDs(C_HousingCatalog.SearchCatalogCategories({}), {403, 402, 49, 401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({}), {503, 502, 59, 501})
    "#,
    )
    .unwrap();
}

#[test]
fn fresh_snapshots_follow_current_inputs_without_aliasing_prior_results() {
    let env = fixture_env();
    env.exec(r#"
        OldCategories = C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true, editorModeContext = -37})
        OldSubcategories = C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true, editorModeContext = -37})
        local categories = C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true, editorModeContext = -37})
        local subs = C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true, editorModeContext = -37})
        assert(categories ~= OldCategories and subs ~= OldSubcategories)
        categories[1] = 9999; subs[1] = 9999
        assertIDs(OldCategories, {401}); assertIDs(OldSubcategories, {501})
        assertIDs(C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true, editorModeContext = -37}), {401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true, editorModeContext = -37}), {501})
    "#).unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let catalog = &mut state.housing.catalog;
        catalog.categories.get_mut(&401).unwrap().any_stored_entries = false;
        let category = catalog.categories.get_mut(&402).unwrap();
        category.any_stored_entries = true;
        category.editor_mode_contexts = vec![-37];
        catalog.subcategories.remove(&501);
        let subcategory = catalog.subcategories.get_mut(&502).unwrap();
        subcategory.any_stored_entries = true;
        subcategory.editor_mode_contexts = vec![-37];
    }
    env.exec(r#"
        assertIDs(OldCategories, {401}); assertIDs(OldSubcategories, {501})
        assertIDs(C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true, editorModeContext = -37}), {402})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true, editorModeContext = -37}), {502})
    "#).unwrap();
}

#[test]
fn queries_do_not_mutate_records_or_derive_stored_predicates_from_variants() {
    let env = fixture_env();
    let entry_type = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    let variant_id = HousingCatalogEntryVariantID {
        record_id: 402,
        entry_type,
        variant_identifier: 1,
    };
    env.state().borrow_mut().housing.catalog.variants.insert(
        variant_id,
        HousingCatalogVariantRecord {
            num_stored: 25,
            destroyable_instance_count: 7,
            dye_slots: vec![],
        },
    );
    let before = format!("{:?}", env.state().borrow().housing.catalog);
    env.exec(r#"
        assertIDs(C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true}), {401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true}), {501})
        assertIDs(C_HousingCatalog.SearchCatalogCategories({editorModeContext = 64}), {402, 401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({editorModeContext = 64}), {502, 501})
    "#).unwrap();
    assert_eq!(
        format!("{:?}", env.state().borrow().housing.catalog),
        before
    );
}

#[test]
fn required_public_table_and_strict_supplied_field_types_reject_invalid_inputs() {
    let env = fixture_env();
    env.exec(r#"
        for _, query in ipairs(SearchQueries) do
            assert(not pcall(query), 'missing required table must reject')
            for _, value in ipairs({false, 1, 'table'}) do
                assert(not pcall(query, value), 'non-table must reject')
            end
            for _, params in ipairs({
                {withStoredEntriesOnly = 1}, {withStoredEntriesOnly = 'true'},
                {includeFeaturedCategory = 0}, {includeFeaturedCategory = 'false'},
                {editorModeContext = '64'}, {editorModeContext = 64.5}, {editorModeContext = false},
                {editorModeContext = 2147483648},
            }) do
                local ok, message = pcall(query, params)
                assert(not ok and type(message) == 'string' and #message > 0, 'invalid public field must reject')
            end
        end
    "#).unwrap();
}

#[test]
fn public_addon_queries_preserve_caller_taint() {
    let env = fixture_env();
    env.exec(r#"
        local function addon()
            assert(not issecure())
            assertIDs(C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true, editorModeContext = -37}), {401})
            assertIDs(C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true, editorModeContext = -37}), {501})
            assert(not issecure(), 'public search must not clear addon taint')
        end
        debug.setobjecttaint(addon, 'HousingCategorySearchFixture')
        assert(issecure()); addon(); assert(issecure())
    "#).unwrap();
}

fn install_host_security_fixtures(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    let secret = wrap_host_secret_bool(lua.state_mut(), true);
    lua.state_mut().push(secret);
    let inserted = lua.set_global_val("SecretSearchBool", secret);
    lua.state_mut().pop();
    inserted.unwrap();
    let secret = wrap_host_secret_number(lua.state_mut(), -37.0);
    lua.state_mut().push(secret);
    let inserted = lua.set_global_val("SecretSearchMode", secret);
    lua.state_mut().pop();
    inserted.unwrap();
}

#[test]
fn secret_table_and_fields_reject_without_unwrapping_or_clearing_taint() {
    let env = fixture_env();
    install_host_security_fixtures(&env);
    env.exec(r#"
        SecretSearchTable = secretwrap({withStoredEntriesOnly = true})
        collectgarbage('collect')
        local function reject()
            for _, query in ipairs(SearchQueries) do
                for _, params in ipairs({SecretSearchTable,
                    {withStoredEntriesOnly = SecretSearchBool},
                    {includeFeaturedCategory = SecretSearchBool},
                    {editorModeContext = SecretSearchMode},
                }) do
                    local ok, message = pcall(query, params)
                    assert(not ok and type(message) == 'string' and #message > 0, 'secret must reject')
                end
            end
            assert(issecretvalue(SecretSearchTable) and issecretvalue(SecretSearchBool) and issecretvalue(SecretSearchMode))
        end
        assert(issecure()); reject(); assert(issecure())
        local function addon()
            assert(not issecure()); reject(); assert(not issecure(), 'rejection must preserve taint')
        end
        debug.setobjecttaint(addon, 'HousingCategorySearchFixture')
        addon(); assert(issecure()); reject(); assert(issecure())
        assertIDs(C_HousingCatalog.SearchCatalogCategories({withStoredEntriesOnly = true}), {401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories({withStoredEntriesOnly = true}), {501})
    "#).unwrap();
}

#[test]
fn guarded_search_table_preserves_real_vm_access_policy() {
    let env = fixture_env();
    install_host_security_fixtures(&env);
    env.exec(r#"
        local guarded = {withStoredEntriesOnly = true, editorModeContext = -37}
        settablesecurity(guarded, 0) -- Host-installed VM policy, not the retail no-op global.
        assertIDs(C_HousingCatalog.SearchCatalogCategories(guarded), {401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories(guarded), {501})
        local function addon()
            assert(not issecure())
            local sourceOK, sourceError = pcall(rawget, guarded, 'withStoredEntriesOnly')
            assert(not sourceOK and string.find(sourceError, 'tainted access to secured table', 1, true))
            for _, query in ipairs(SearchQueries) do
                local ok, message = pcall(query, guarded)
                assert(not ok and string.find(message, 'tainted access to secured table', 1, true),
                    'query must respect table access guard')
                assert(not issecure(), 'guard failure must preserve caller taint')
            end
        end
        debug.setobjecttaint(addon, 'HousingCategorySearchFixture')
        addon(); assert(issecure())
        assert(guarded.withStoredEntriesOnly == true and guarded.editorModeContext == -37)
        assertIDs(C_HousingCatalog.SearchCatalogCategories(guarded), {401})
        assertIDs(C_HousingCatalog.SearchCatalogSubcategories(guarded), {501})
    "#).unwrap();
}

#[test]
fn cached_deprecated_wrapper_maps_old_only_when_new_field_is_missing() {
    let env = fixture_env();
    let path = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .unwrap()
        .join("Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read cached bridge {}: {error}", path.display()));
    env.exec("assert(GetCVarBool('loadDeprecationFallbacks'), 'bridge must be enabled')")
        .unwrap();
    env.exec(&source).unwrap();
    env.exec(r#"
        local queries = {C_HousingCatalog.SearchCatalogCategories, C_HousingCatalog.SearchCatalogSubcategories}
        local stored = {{401}, {501}}
        local all = {{402, 401, 403}, {502, 501, 503}}
        for index, query in ipairs(queries) do
            local old = {withOwnedEntriesOnly = true}
            assertIDs(query(old), stored[index])
            assert(old.withStoredEntriesOnly == true, 'real bridge converts missing new field')
            local both = {withStoredEntriesOnly = false, withOwnedEntriesOnly = true}
            assertIDs(query(both), all[index])
            assert(both.withStoredEntriesOnly == false, 'explicit new false must win')
        end
    "#).unwrap();
}
