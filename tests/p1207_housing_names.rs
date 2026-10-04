#![cfg(feature = "retail-12-0-7")]
use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogCategoryRecord, HousingCatalogSubcategoryRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.housing.catalog.categories = [
            (
                401,
                HousingCatalogCategoryRecord {
                    order_index: 1,
                    name: Some("Furniture".into()),
                    icon: None,
                    subcategory_ids: vec![501],
                    any_stored_entries: false,
                    editor_mode_contexts: vec![],
                },
            ),
            (
                402,
                HousingCatalogCategoryRecord {
                    order_index: 2,
                    name: Some("Lighting".into()),
                    icon: None,
                    subcategory_ids: vec![502],
                    any_stored_entries: false,
                    editor_mode_contexts: vec![],
                },
            ),
        ]
        .into();
        sim.housing.catalog.subcategories = [
            (
                501,
                HousingCatalogSubcategoryRecord {
                    order_index: 1,
                    parent_category_id: 401,
                    name: Some("Chairs".into()),
                    icon: None,
                    any_stored_entries: false,
                    editor_mode_contexts: vec![],
                },
            ),
            (
                502,
                HousingCatalogSubcategoryRecord {
                    order_index: 2,
                    parent_category_id: 402,
                    name: Some("Lamps".into()),
                    icon: None,
                    any_stored_entries: false,
                    editor_mode_contexts: vec![],
                },
            ),
        ]
        .into();
    }
    env.exec(
        r#"
        function Names(id, category, subcategory)
            local function check(...)
                if category == nil then assert(select('#', ...) == 0); return end
                assert(select('#', ...) == 2)
                local a,b = ...
                assert(a == category and b == subcategory)
                assert(not issecretvalue(a) and not issecretvalue(b))
            end
            check(C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames(id))
        end
    "#,
    )
    .unwrap();
    env
}

#[test]
fn names_read_parent_relation_and_live_labels_without_seed_fallback() {
    let env = fixture();
    env.exec("Names(501, 'Furniture', 'Chairs'); Names(502, 'Lighting', 'Lamps'); Names(999, nil)")
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.housing.catalog.categories.get_mut(&402).unwrap().name = Some("Lumieres".into());
        let sub = sim.housing.catalog.subcategories.get_mut(&501).unwrap();
        sub.parent_category_id = 402;
        sub.name = Some("Reading".into());
    }
    env.exec("Names(501, 'Lumieres', 'Reading'); Names(502, 'Lumieres', 'Lamps')")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .categories
        .remove(&402);
    env.exec("Names(501, nil); Names(502, nil)").unwrap();
}

#[test]
fn names_misses_missing_labels_invalid_selectors_and_isolation() {
    let env = fixture();
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .subcategories
        .get_mut(&501)
        .unwrap()
        .name = None;
    env.exec(
        r#"
        Names(501, nil); Names(nil, nil); Names(false, nil); Names('502', nil)
        Names({}, nil); Names(1.5, nil); Names(1/0, nil)
        Names(502, 'Lighting', 'Lamps')
        local a,b = C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames(502)
        a = 'changed'; b = 'changed'
        Names(502, 'Lighting', 'Lamps')
    "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    let count: f64 = other
        .eval("return select('#', C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames(502))")
        .unwrap();
    assert_eq!(count, 0.0);
    assert_eq!(
        env.state().borrow().housing.catalog.subcategories[&502].parent_category_id,
        402
    );
}

#[test]
fn names_authenticate_secrets_before_miss_or_extra_validation() {
    let env = fixture();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let value = wrap_host_secret_number(lua.state_mut(), 502.0);
        lua.state_mut().push(value);
        lua.set_global_val("NamesSecret", value).unwrap();
        lua.state_mut().pop();
    }
    env.exec(r#"
        Names(NamesSecret, 'Lighting', 'Lamps')
        collectgarbage('collect')
        Names(NamesSecret, 'Lighting', 'Lamps')
        local function probe()
            Names(502, 'Lighting', 'Lamps')
            for _, call in ipairs({
                function() return C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames(NamesSecret) end,
                function() return C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames(false, NamesSecret) end,
                function() return C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames(999, NamesSecret) end,
            }) do
                local ok, err = pcall(call)
                assert(not ok and string.find(err, 'untainted caller', 1, true))
            end
            assert(debug.getstacktaint() == 'NamesProbe')
        end
        debug.setobjecttaint(probe, 'NamesProbe'); probe()
        assert(issecure() and issecretvalue(NamesSecret))
        Names(NamesSecret, 'Lighting', 'Lamps')
    "#).unwrap();
}
