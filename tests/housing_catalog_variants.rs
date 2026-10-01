//! Empty-backed catalog fixtures; no Blizzard addons or production seed data.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogState, HousingCatalogVariantRecord, HousingDecorDyeSlot,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn inject_catalog(env: &WowLuaEnv) {
    let entry_type: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    let entry_id = HousingCatalogEntryID {
        record_id: 1001,
        entry_type,
    };
    let entries = [(
        entry_id,
        HousingCatalogEntryRecord {
            item_id: Some(1001),
            name: "Fixture chair".into(),
            is_unique_trophy: false,
        },
    )]
    .into();
    let variants = [(1, 3, 701, "Fixture amber"), (2, 5, 702, "Fixture blue")]
        .into_iter()
        .map(
            |(variant_identifier, num_stored, dye_color_id, dye_color_name)| {
                let id = HousingCatalogEntryVariantID {
                    record_id: 1001,
                    entry_type,
                    variant_identifier,
                };
                let variant = HousingCatalogVariantRecord {
                    num_stored,
                    dye_slots: vec![HousingDecorDyeSlot {
                        id: 11,
                        dye_color_category_id: 12,
                        order_index: 1,
                        channel: 0,
                        dye_color_id: Some(dye_color_id),
                        dye_color_name: Some(dye_color_name.into()),
                    }],
                };
                (id, variant)
            },
        )
        .collect();
    env.state().borrow_mut().housing.catalog = HousingCatalogState { entries, variants };
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    inject_catalog(&env);
    env
}

const ASSERT_VARIANT_IDS: &str = r#"
    function assertFixtureIDs(ids)
        assert(type(ids) == 'table' and #ids == 2, 'expected exactly two fixture variants')
        local seen = {}
        for _, id in ipairs(ids) do
            assert(id.recordID == 1001 and id.entryType == Enum.HousingCatalogEntryType.Decor)
            assert(id.variantIdentifier == 1 or id.variantIdentifier == 2)
            assert(not seen[id.variantIdentifier], 'variant identity collapsed')
            seen[id.variantIdentifier] = true
        end
        assert(seen[1] and seen[2])
    end
"#;

fn assert_search_ids(method: &str) {
    let env = fixture_env();
    env.exec(ASSERT_VARIANT_IDS).unwrap();
    // Fresh independent searchers; no filters, callbacks, release or timing policy.
    env.exec(&format!(
        "local first = C_HousingCatalog.CreateCatalogSearcher(); \
         local second = C_HousingCatalog.CreateCatalogSearcher(); \
         first:RunSearch(); second:RunSearch(); \
         assertFixtureIDs(first:{method}()); assertFixtureIDs(second:{method}())"
    ))
    .unwrap();
}

#[test]
fn housing_variant_default_search_source_is_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "local s = C_HousingCatalog.CreateCatalogSearcher(); s:RunSearch(); \
         local ids = s:GetAllSearchItems(); \
         assert(type(ids) == 'table' and #ids == 0, 'default source must be empty')",
    )
    .unwrap();
}

#[test]
fn housing_variant_default_search_results_are_empty() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "local s = C_HousingCatalog.CreateCatalogSearcher(); s:RunSearch(); \
         local ids = s:GetCatalogSearchResults(); \
         assert(type(ids) == 'table' and #ids == 0, 'default results must be empty')",
    )
    .unwrap();
}

#[test]
fn housing_variant_search_source_preserves_compound_ids() {
    assert_search_ids("GetAllSearchItems");
}

#[test]
fn housing_variant_search_results_preserve_compound_ids() {
    assert_search_ids("GetCatalogSearchResults");
}

#[test]
fn housing_variant_query_preserves_distinct_stack_fields() {
    let env = fixture_env();
    env.exec(
        r#"
        local entryType = Enum.HousingCatalogEntryType.Decor
        for _, expected in ipairs({{1, 3, 701, 'Fixture amber'}, {2, 5, 702, 'Fixture blue'}}) do
            local info = C_HousingCatalog.GetCatalogEntryVariantInfo({
                recordID = 1001, entryType = entryType, variantIdentifier = expected[1],
            })
            assert(info and info.numStored == expected[2], 'explicit variant stack not read')
            assert(info.variantID == nil and info.productID == nil and info.name == nil,
                'variant stack must not synthesize storefront or base metadata')
            local id = info.entryVariantID
            assert(id.recordID == 1001 and id.entryType == entryType and id.variantIdentifier == expected[1])
            assert(#info.dyeSlots == 1)
            local dye = info.dyeSlots[1]
            assert(dye.ID == 11 and dye.dyeColorCategoryID == 12 and dye.orderIndex == 1 and dye.channel == 0)
            assert(dye.dyeColorID == expected[3] and dye.dyeColorName == expected[4])
        end
        "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_list_uses_explicit_fixture_records() {
    let env = fixture_env();
    env.exec(ASSERT_VARIANT_IDS).unwrap();
    env.exec(
        r#"
        local infos = C_HousingCatalog.GetAllVariantInfosForEntry({
            recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor,
        })
        assert(#infos == 2)
        local ids = {}
        for _, info in ipairs(infos) do
            ids[#ids + 1] = info.entryVariantID
            local expected = info.entryVariantID.variantIdentifier == 1 and 3 or 5
            assert(info.numStored == expected and #info.dyeSlots == 1)
        end
        assertFixtureIDs(ids)
        "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_entry_info_remains_base_info() {
    let env = fixture_env();
    env.exec(
        r#"
        local info = C_HousingCatalog.GetCatalogEntryInfo({
            recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor,
        })
        assert(info and info.recordID == 1001 and info.entryType == Enum.HousingCatalogEntryType.Decor)
        assert(info.itemID == 1001 and info.name == 'Fixture chair' and info.isUniqueTrophy == false)
        assert(info.entryID == nil and info.entryVariantID == nil and info.variantIdentifier == nil)
        assert(info.numStored == nil, 'variant storage must not leak into base info')
        "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_default_queries_have_no_seed_records() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor}
        assert(C_HousingCatalog.GetCatalogEntryInfo(id) == nil)
        local infos = C_HousingCatalog.GetAllVariantInfosForEntry(id)
        assert(type(infos) == 'table' and #infos == 0)
        id.variantIdentifier = 1
        assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_public_selectors_allow_tainted_callers() {
    let env = fixture_env();
    env.exec(
        r#"
        local function addon()
            local info = C_HousingCatalog.GetCatalogEntryVariantInfo({
                recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 2,
            })
            assert(info and info.numStored == 5)
            local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor}
            assert(C_HousingCatalog.GetCatalogEntryInfo(id).name == 'Fixture chair')
            assert(#C_HousingCatalog.GetAllVariantInfosForEntry(id) == 2)
            assert(not issecure(), 'query must preserve caller taint')
        end
        debug.setobjecttaint(addon, 'HousingCatalogFixture')
        local ok, err = pcall(addon)
        assert(ok, err)
        "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_missing_selectors_do_not_use_legacy_seeds() {
    let env = fixture_env();
    env.exec(
        r#"
        local decor = Enum.HousingCatalogEntryType.Decor
        for _, id in ipairs({
            {recordID = 1001, entryType = decor, variantIdentifier = 99},
            {recordID = 1002, entryType = decor, variantIdentifier = 1},
            {recordID = 1001, entryType = -91, variantIdentifier = 1},
        }) do
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id) == nil,
                'missing full key must not resolve to a different variant')
            if id.recordID ~= 1001 or id.entryType ~= decor then
                assert(C_HousingCatalog.GetCatalogEntryInfo(id) == nil)
                assert(#C_HousingCatalog.GetAllVariantInfosForEntry(id) == 0)
            end
        end
        "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_inputs_do_not_seed_another_environment() {
    let populated = fixture_env();
    let empty = WowLuaEnv::new().unwrap();
    populated.exec(ASSERT_VARIANT_IDS).unwrap();
    populated
        .exec("local s = C_HousingCatalog.CreateCatalogSearcher(); s:RunSearch(); assertFixtureIDs(s:GetAllSearchItems())")
        .unwrap();
    empty
        .exec(
            "local s = C_HousingCatalog.CreateCatalogSearcher(); s:RunSearch(); \
             assert(#s:GetAllSearchItems() == 0 and #s:GetCatalogSearchResults() == 0); \
             assert(C_HousingCatalog.GetCatalogEntryVariantInfo({recordID = 1001, \
             entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}) == nil)",
        )
        .unwrap();
}

#[test]
fn housing_variant_snapshots_do_not_alias_model_or_search_containers() {
    let env = fixture_env();
    env.exec(ASSERT_VARIANT_IDS).unwrap();
    env.exec(r#"
        local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
        local info = C_HousingCatalog.GetCatalogEntryVariantInfo(id)
        info.entryVariantID.variantIdentifier = 99
        info.dyeSlots[1].dyeColorName = 'changed'
        info.numStored = 99
        local again = C_HousingCatalog.GetCatalogEntryVariantInfo(id)
        assert(again.numStored == 3 and again.entryVariantID.variantIdentifier == 1)
        assert(again.dyeSlots[1].dyeColorName == 'Fixture amber')
        local list = C_HousingCatalog.GetAllVariantInfosForEntry(id)
        list[1].dyeSlots[1].ID = 99
        for _, row in ipairs(C_HousingCatalog.GetAllVariantInfosForEntry(id)) do
            assert(row.dyeSlots[1].ID == 11)
        end
        C_HousingCatalog.GetCatalogEntryInfo(id).name = 'changed'
        assert(C_HousingCatalog.GetCatalogEntryInfo(id).name == 'Fixture chair')

        local first = C_HousingCatalog.CreateCatalogSearcher()
        local second = C_HousingCatalog.CreateCatalogSearcher()
        first:RunSearch(); second:RunSearch()
        local source, results = first:GetAllSearchItems(), first:GetCatalogSearchResults()
        assert(source ~= results, 'source and results are distinct containers')
        source[1].variantIdentifier = 99
        source[2] = nil
        assertFixtureIDs(results)
        assertFixtureIDs(second:GetAllSearchItems())
        assertFixtureIDs(second:GetCatalogSearchResults())
        results[1].recordID = 99
        collectgarbage('collect')
        first:RunSearch()
        assertFixtureIDs(first:GetAllSearchItems())
        assertFixtureIDs(first:GetCatalogSearchResults())
    "#).unwrap();
}

#[test]
fn housing_variant_nested_secrets_reject_without_unwrapping() {
    let env = fixture_env();
    let entry_type: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, value) in [
        ("SecretRecord", 1001.0),
        ("SecretType", f64::from(entry_type)),
        ("SecretVariant", 1.0),
    ] {
        let secret = wrap_host_secret_number(lua.state_mut(), value);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val(name, secret);
        lua.state_mut().pop();
        inserted.unwrap();
    }
    drop(lua);
    // Conservative simulator rejection, not native AllowedWhenUntainted parity.
    env.exec(
        r#"
        collectgarbage('collect')
        local decor = Enum.HousingCatalogEntryType.Decor
        local function queries()
            for _, id in ipairs({
                {recordID = SecretRecord, entryType = decor, variantIdentifier = 1},
                {recordID = 1001, entryType = SecretType, variantIdentifier = 1},
            }) do
                for _, query in ipairs({C_HousingCatalog.GetCatalogEntryInfo,
                    C_HousingCatalog.GetAllVariantInfosForEntry,
                    C_HousingCatalog.GetCatalogEntryVariantInfo}) do
                    assert(not pcall(query, id), 'secret selector must not resolve')
                end
            end
            assert(not pcall(C_HousingCatalog.GetCatalogEntryVariantInfo,
                {recordID = 1001, entryType = decor, variantIdentifier = SecretVariant}))
        end
        queries()
        local function addon()
            queries()
            assert(not issecure(), 'rejection must not clear taint')
        end
        debug.setobjecttaint(addon, 'HousingCatalogFixture')
        addon()
    "#,
    )
    .unwrap();
}

#[test]
fn housing_variant_secured_selector_preserves_access_guard() {
    let env = fixture_env();
    // Retail settablesecurity is a compatibility no-op. Install the pinned VM
    // policy explicitly for this host-owned fixture, not the runtime surface.
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
    }
    env.exec(r#"
        assert(issecure(), 'selector fixture must be created by a secure caller')
        local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
        settablesecurity(id, 0) -- VM DisallowTaintedAccess policy, not a catalog inference.
        assert(id.recordID == 1001, 'secure source indexing must remain allowed')
        assert(rawget(id, 'recordID') == 1001, 'secure source rawget must remain allowed')
        local queries = {
            {'GetCatalogEntryInfo', C_HousingCatalog.GetCatalogEntryInfo},
            {'GetAllVariantInfosForEntry', C_HousingCatalog.GetAllVariantInfosForEntry},
            {'GetCatalogEntryVariantInfo', C_HousingCatalog.GetCatalogEntryVariantInfo},
        }
        local function assert_secure_queries()
            assert(C_HousingCatalog.GetCatalogEntryInfo(id).name == 'Fixture chair',
                'secure base query must resolve guarded selector')
            assert(#C_HousingCatalog.GetAllVariantInfosForEntry(id) == 2,
                'secure list query must resolve guarded selector')
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id).numStored == 3,
                'secure variant query must resolve guarded selector')
        end
        assert_secure_queries()
        local function addon()
            assert(not issecure(), 'guard probe must run as a tainted caller')
            local source_ok, source_error = pcall(function() return id.recordID end)
            assert(not source_ok, 'tainted source indexing must reject guarded selector')
            assert(string.find(source_error, 'tainted access to secured table', 1, true),
                'source indexing must fail at table access guard')
            local raw_ok, raw_error = pcall(rawget, id, 'recordID')
            assert(not raw_ok, 'tainted source rawget must reject guarded selector')
            assert(string.find(raw_error, 'tainted access to secured table', 1, true),
                'source rawget must fail at table access guard')
            for _, query in ipairs(queries) do
                local ok, message = pcall(query[2], id)
                assert(not ok, query[1] .. ' must preserve table access guard')
                assert(string.find(message, 'tainted access to secured table', 1, true),
                    query[1] .. ' must fail at table access guard')
                assert(not issecure(), query[1] .. ' rejection must not clear caller taint')
            end
        end
        debug.setobjecttaint(addon, 'HousingCatalogFixture')
        addon()
        assert(issecure(), 'secure fixture caller must remain untainted after addon returns')
        assert(id.recordID == 1001, 'rejected accesses must leave source unchanged')
        assert_secure_queries()
    "#).unwrap();
}
