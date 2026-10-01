//! Bounded base selector behavior; parent owns compilation and acceptance gates.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogVariantRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let decor: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    let room: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Room")
        .unwrap();
    // Deliberately unrelated item/record IDs; type collision is explicit input.
    for (entry_type, item_id, name, trophy) in [
        (decor, Some(6948), "Catalog chair, not an item name", false),
        (room, Some(900002), "Catalog room", true),
    ] {
        env.state().borrow_mut().housing.catalog.entries.insert(
            HousingCatalogEntryID {
                record_id: 81001,
                entry_type,
            },
            HousingCatalogEntryRecord {
                item_id,
                name: name.into(),
                is_unique_trophy: trophy,
            },
        );
    }
    env.state().borrow_mut().housing.catalog.entries.insert(
        HousingCatalogEntryID {
            record_id: 81002,
            entry_type: decor,
        },
        HousingCatalogEntryRecord {
            item_id: None,
            name: "Record without item".into(),
            is_unique_trophy: true,
        },
    );
    for variant_identifier in [0, 1, 2] {
        env.state().borrow_mut().housing.catalog.variants.insert(
            HousingCatalogEntryVariantID {
                record_id: 81003,
                entry_type: decor,
                variant_identifier,
            },
            HousingCatalogVariantRecord {
                num_stored: 7,
                destroyable_instance_count: 3,
                dye_slots: vec![],
            },
        );
    }
    env
}

const ASSERT_CHAIR: &str = r#"
    function assertChair(info)
        assert(info, 'explicit base record missing')
        assert(info.recordID == 81001 and info.entryType == Enum.HousingCatalogEntryType.Decor)
        assert(info.itemID == 6948 and info.name == 'Catalog chair, not an item name')
        assert(info.isUniqueTrophy == false)
        assert(info.entryID == nil and info.entryVariantID == nil and info.variantIdentifier == nil)
        assert(info.numStored == nil and info.dyeSlots == nil, 'base lookup must not synthesize a variant')
    end
"#;

fn chair_env() -> WowLuaEnv {
    let env = fixture_env();
    env.exec(ASSERT_CHAIR).unwrap();
    env
}

#[test]
fn empty_item_lookup_ignores_legacy_seed() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_HousingCatalog.GetCatalogEntryInfoByItem(1001) == nil, 'empty item lookup read legacy seed')").unwrap();
}

#[test]
fn empty_record_lookup_ignores_legacy_seed() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Decor, 1001) == nil, 'empty record lookup read legacy seed')").unwrap();
}

#[test]
fn item_scalar_reads_explicit_base_not_record_id() {
    let env = chair_env();
    env.exec("assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem(6948)); assert(C_HousingCatalog.GetCatalogEntryInfoByItem(81001) == nil)").unwrap();
}

#[test]
fn item_links_read_explicit_base() {
    let env = chair_env();
    env.exec(
        r#"
        for _, item in ipairs({'item:6948', 'item:6948:::::::::::1:1:3524:',
            '|cff00ff00|Hitem:6948:::::::::|h[Not the catalog name]|h|r'}) do
            assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem(item))
        end
    "#,
    )
    .unwrap();
}

#[test]
fn item_names_do_not_use_catalog_display_names() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(C_HousingCatalog.GetCatalogEntryInfoByItem('Catalog chair, not an item name') == nil)
        assert(C_HousingCatalog.GetCatalogEntryInfoByItem('Hearthstone') == nil,
            'item-name resolution is not modeled')
    "#,
    )
    .unwrap();
}

#[test]
fn ambiguous_explicit_item_ids_reject_inferred_policy() {
    let env = chair_env();
    let room: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Room")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .entries
        .get_mut(&HousingCatalogEntryID {
            record_id: 81001,
            entry_type: room,
        })
        .unwrap()
        .item_id = Some(6948);
    // Inferred simulator policy, not evidence of a native duplicate-ID winner.
    env.exec(
        r#"
        for _, item in ipairs({6948, 'item:6948'}) do
            local ok, err = pcall(C_HousingCatalog.GetCatalogEntryInfoByItem, item)
            assert(not ok and string.find(tostring(err), 'ambiguous explicit housing catalog item ID', 1, true))
        end
        assertChair(C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Decor, 81001))
        local room = C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Room, 81001)
        assert(room.itemID == 6948 and room.name == 'Catalog room')
    "#,
    )
    .unwrap();
}

#[test]
fn legacy_item_tables_do_not_resolve() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, item in ipairs({{itemID = 6948}, {id = 6948}, {itemID = 1001}, {id = 1001}}) do
            local ok, info = pcall(C_HousingCatalog.GetCatalogEntryInfoByItem, item)
            assert(not ok or info == nil, 'undocumented table selector must not resolve')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn record_lookup_preserves_type_collision_and_optional_item() {
    let env = chair_env();
    env.exec(r#"
        local decor, room = Enum.HousingCatalogEntryType.Decor, Enum.HousingCatalogEntryType.Room
        assertChair(C_HousingCatalog.GetCatalogEntryInfoByRecordID(decor, 81001))
        local info = C_HousingCatalog.GetCatalogEntryInfoByRecordID(room, 81001)
        assert(info and info.recordID == 81001 and info.entryType == room)
        assert(info.itemID == 900002 and info.name == 'Catalog room' and info.isUniqueTrophy == true)
        local noItem = C_HousingCatalog.GetCatalogEntryInfoByRecordID(decor, 81002)
        assert(noItem and noItem.itemID == nil and noItem.name == 'Record without item')
        assert(noItem.recordID == 81002 and noItem.entryType == decor and noItem.isUniqueTrophy == true)
        assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(room, 81002) == nil)
        assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(decor, 6948) == nil)
    "#).unwrap();
}

#[test]
fn missing_base_never_falls_back_to_variants_or_seeds() {
    let env = fixture_env();
    env.exec(
        r#"
        local decor = Enum.HousingCatalogEntryType.Decor
        for _, recordID in ipairs({1001, 81003, 998877}) do
            assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(decor, recordID) == nil,
                'missing base must not resolve seed or variant zero/one')
            assert(C_HousingCatalog.GetCatalogEntryInfoByItem(recordID) == nil)
        end
        assert(C_HousingCatalog.GetCatalogEntryVariantInfo({recordID = 81003,
            entryType = decor, variantIdentifier = 0}).numStored == 7)
    "#,
    )
    .unwrap();
}

#[test]
fn snapshots_are_independent_across_queries_and_model_mutation() {
    let env = chair_env();
    env.exec(
        r#"
        local decor = Enum.HousingCatalogEntryType.Decor
        SavedItem = C_HousingCatalog.GetCatalogEntryInfoByItem(6948)
        SavedRecord = C_HousingCatalog.GetCatalogEntryInfoByRecordID(decor, 81001)
        assertChair(SavedItem); assertChair(SavedRecord)
        SavedItem.name = 'Lua mutation'; SavedItem.itemID = 99; SavedItem.recordID = 99
        SavedItem.entryType = 99; SavedItem.isUniqueTrophy = true
        assertChair(SavedRecord)
        assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem(6948))
        assertChair(C_HousingCatalog.GetCatalogEntryInfoByRecordID(decor, 81001))
    "#,
    )
    .unwrap();
    let decor: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .entries
        .get_mut(&HousingCatalogEntryID {
            record_id: 81001,
            entry_type: decor,
        })
        .unwrap()
        .name = "Host mutation".into();
    env.exec(
        r#"
        collectgarbage('collect')
        assertChair(SavedRecord)
        assert(C_HousingCatalog.GetCatalogEntryInfoByItem(6948).name == 'Host mutation')
        assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(Enum.HousingCatalogEntryType.Decor,
            81001).name == 'Host mutation')
    "#,
    )
    .unwrap();
}

#[test]
fn removed_trailing_arguments_do_not_change_base_selection() {
    let env = chair_env();
    env.exec(
        r#"
        for _, oldOwned in ipairs({false, true}) do
            assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem(6948, oldOwned))
            assertChair(C_HousingCatalog.GetCatalogEntryInfoByRecordID(
                Enum.HousingCatalogEntryType.Decor, 81001, oldOwned))
        end
    "#,
    )
    .unwrap();
}

#[test]
fn cached_deprecated_wrapper_ignores_removed_args_for_missing_results() {
    let env = WowLuaEnv::new().unwrap();
    let path = std::path::PathBuf::from(std::env::var_os("HOME").expect("HOME"))
        .join(".cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read cached wrapper {}: {error}", path.display()));
    // Execute the actual file, not a copied bridge or a fake full DTO.
    env.exec(&source).unwrap();
    env.exec(
        r#"
        for _, oldOwned in ipairs({false, true}) do
            assert(C_HousingCatalog.GetCatalogEntryInfoByItem(1001, oldOwned) == nil)
            assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(
                Enum.HousingCatalogEntryType.Decor, 1001, oldOwned) == nil)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn ordinary_public_selectors_preserve_addon_taint() {
    let env = chair_env();
    env.exec(
        r#"
        local function addon()
            assert(not issecure())
            assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem(6948))
            assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem('item:6948'))
            assertChair(C_HousingCatalog.GetCatalogEntryInfoByRecordID(
                Enum.HousingCatalogEntryType.Decor, 81001))
            assert(not issecure(), 'queries must not clear caller taint')
        end
        debug.setobjecttaint(addon, 'HousingBaseLookupFixture')
        addon()
        assert(issecure(), 'addon return must leave secure caller unchanged')
    "#,
    )
    .unwrap();
}

#[test]
fn secret_scalar_selectors_reject_without_unwrapping_or_taint_changes() {
    let env = chair_env();
    let decor: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        for (name, number) in [
            ("SecretItem", 6948.0),
            ("SecretRecord", 81001.0),
            ("SecretType", f64::from(decor)),
        ] {
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
            local decor = Enum.HousingCatalogEntryType.Decor
            assert(not pcall(C_HousingCatalog.GetCatalogEntryInfoByItem, SecretItem), 'secret item must reject')
            assert(not pcall(C_HousingCatalog.GetCatalogEntryInfoByRecordID, decor, SecretRecord), 'secret record must reject')
            assert(not pcall(C_HousingCatalog.GetCatalogEntryInfoByRecordID, SecretType, 81001), 'secret type must reject')
            assert(issecretvalue(SecretItem) and issecretvalue(SecretRecord) and issecretvalue(SecretType))
        end
        assert(issecure()); reject(); assert(issecure())
        local function addon()
            assert(not issecure()); reject()
            assert(not issecure(), 'rejection must not clear caller taint')
        end
        debug.setobjecttaint(addon, 'HousingBaseLookupFixture')
        addon()
        assert(issecure()); reject()
        assertChair(C_HousingCatalog.GetCatalogEntryInfoByItem(6948))
    "#).unwrap();
}
