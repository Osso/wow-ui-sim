//! Input-only storage-event contract; parent must observe actual RED before production.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogVariantRecord, HousingDecorDyeSlot,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn variant_record(stored: i32, destroyable: i32, dye: i32) -> HousingCatalogVariantRecord {
    HousingCatalogVariantRecord {
        num_stored: stored,
        destroyable_instance_count: destroyable,
        dye_slots: vec![HousingDecorDyeSlot {
            id: 11,
            dye_color_category_id: 12,
            order_index: 1,
            channel: 0,
            dye_color_id: Some(dye),
            dye_color_name: Some(format!("Fixture dye {dye}")),
        }],
    }
}

fn inject_variants(env: &WowLuaEnv) {
    let (decor, room): (i32, i32) = env
        .eval("return Enum.HousingCatalogEntryType.Decor, Enum.HousingCatalogEntryType.Room")
        .unwrap();
    let mut state = env.state().borrow_mut();
    for entry_type in [decor, room] {
        state.housing.catalog.entries.insert(
            HousingCatalogEntryID {
                record_id: 1001,
                entry_type,
            },
            HousingCatalogEntryRecord {
                item_id: Some(1001),
                name: "Fixture entry".into(),
                is_unique_trophy: false,
            },
        );
    }
    for (entry_type, variant_identifier, stored, destroyable, dye) in [
        (decor, 1, 3, 1, 701),
        (decor, 2, 5, 4, 702),
        (room, 1, 7, 2, 703),
    ] {
        state.housing.catalog.variants.insert(
            HousingCatalogEntryVariantID {
                record_id: 1001,
                entry_type,
                variant_identifier,
            },
            variant_record(stored, destroyable, dye),
        );
    }
}

fn listen_for_storage() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create housing storage environment");
    inject_variants(&env);
    env.exec(r#"
        assert(type(A_Admin.SetHousingCatalogVariantStoredCount) == 'function',
            'explicit housing storage input must be implemented')
        firstID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
        secondID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 2}
        roomID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Room, variantIdentifier = 1}
        storageEvents = {}
        storageDispatchCount = 0
        storageListener = CreateFrame('Frame')
        storageListener:RegisterEvent('HOUSING_STORAGE_ENTRY_UPDATED')
        storageListener:SetScript('OnEvent', function(_, event, ...)
            storageDispatchCount = storageDispatchCount + 1
            local id = ...
            local info = C_HousingCatalog.GetCatalogEntryVariantInfo(id)
            storageEvents[#storageEvents + 1] = {
                event = event, arity = select('#', ...), id = id,
                stored = info.numStored,
                destroyable = C_HousingCatalog.GetDestroyableInstanceCount(id),
                dye = info.dyeSlots[1].dyeColorID,
            }
        end)
        function assertStored(first, second, room)
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(firstID).numStored == first)
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(secondID).numStored == second)
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(roomID).numStored == room)
        end
        function assertRejected(id, count)
            local before = storageDispatchCount
            local ok, message = pcall(A_Admin.SetHousingCatalogVariantStoredCount, id, count)
            assert(not ok and type(message) == 'string' and #message > 0,
                'invalid input must fail explicitly, not silently succeed')
            assert(storageDispatchCount == before, 'rejected input must not emit')
            assertStored(3, 5, 7)
        end
    "#).expect("register real storage listener and require callable input before rejection tests");
    env
}

fn assert_model(env: &WowLuaEnv, counts: [i32; 3]) {
    let (decor, room): (i32, i32) = env
        .eval("return Enum.HousingCatalogEntryType.Decor, Enum.HousingCatalogEntryType.Room")
        .unwrap();
    let state = env.state().borrow();
    let catalog = &state.housing.catalog;
    assert_eq!(catalog.entries.len(), 2);
    assert_eq!(catalog.variants.len(), 3);
    for (entry_type, variant_identifier, stored, destroyable, dye) in [
        (decor, 1, counts[0], 1, 701),
        (decor, 2, counts[1], 4, 702),
        (room, 1, counts[2], 2, 703),
    ] {
        let key = HousingCatalogEntryVariantID {
            record_id: 1001,
            entry_type,
            variant_identifier,
        };
        let record = &catalog.variants[&key];
        assert_eq!(
            (record.num_stored, record.destroyable_instance_count),
            (stored, destroyable)
        );
        assert_eq!(record.dye_slots.len(), 1);
        let slot = &record.dye_slots[0];
        assert_eq!(
            (
                slot.id,
                slot.dye_color_category_id,
                slot.order_index,
                slot.channel
            ),
            (11, 12, 1, 0)
        );
        assert_eq!(slot.dye_color_id, Some(dye));
        assert_eq!(
            slot.dye_color_name.as_deref(),
            Some(format!("Fixture dye {dye}").as_str())
        );
    }
    for entry in catalog.entries.values() {
        assert_eq!(entry.item_id, Some(1001));
        assert_eq!(entry.name, "Fixture entry");
        assert!(!entry.is_unique_trophy);
    }
    assert!(
        !state
            .events
            .pending()
            .iter()
            .any(|event| event.name == "HOUSING_STORAGE_ENTRY_UPDATED")
    );
}

#[test]
fn changed_storage_publishes_one_full_id_after_state_before_return() {
    let env = listen_for_storage();
    env.exec(
        r#"
        assert(select('#', A_Admin.SetHousingCatalogVariantStoredCount(firstID, 9)) == 0)
        assert(#storageEvents == 1, 'event must dispatch before input returns')
        local row = storageEvents[1]
        assert(row.event == 'HOUSING_STORAGE_ENTRY_UPDATED' and row.arity == 1)
        assert(type(row.id) == 'table')
        assert(row.id.recordID == 1001 and row.id.entryType == firstID.entryType)
        assert(row.id.variantIdentifier == 1)
        assert(row.stored == 9 and row.destroyable == 1 and row.dye == 701)
        assertStored(9, 5, 7)
        A_Admin.SetHousingCatalogVariantStoredCount(firstID, 0)
        assert(#storageEvents == 2 and storageEvents[2].stored == 0)
        assert(storageEvents[2].arity == 1 and storageEvents[2].id.variantIdentifier == 1)
    "#,
    )
    .unwrap();
    assert_model(&env, [0, 5, 7]);
}

#[test]
fn full_key_distinguishes_variant_and_entry_type() {
    let env = listen_for_storage();
    env.exec(
        r#"
        A_Admin.SetHousingCatalogVariantStoredCount(secondID, 11)
        A_Admin.SetHousingCatalogVariantStoredCount(roomID, 13)
        assertStored(3, 11, 13)
        assert(#storageEvents == 2)
        local decor, room = storageEvents[1], storageEvents[2]
        assert(decor.arity == 1 and decor.id.recordID == 1001)
        assert(decor.id.entryType == firstID.entryType and decor.id.variantIdentifier == 2)
        assert(decor.stored == 11 and decor.destroyable == 4 and decor.dye == 702)
        assert(room.arity == 1 and room.id.recordID == 1001)
        assert(room.id.entryType == roomID.entryType and room.id.variantIdentifier == 1)
        assert(room.stored == 13 and room.destroyable == 2 and room.dye == 703)
    "#,
    )
    .unwrap();
    assert_model(&env, [3, 11, 13]);
}

#[test]
fn identical_count_is_inferred_noop_with_zero_returns() {
    let env = listen_for_storage();
    env.exec(
        r#"
        assert(select('#', A_Admin.SetHousingCatalogVariantStoredCount(firstID, 3)) == 0)
        assert(#storageEvents == 0 and storageDispatchCount == 0)
        A_Admin.SetHousingCatalogVariantStoredCount(firstID, 8)
        assert(select('#', A_Admin.SetHousingCatalogVariantStoredCount(firstID, 8)) == 0)
        assert(#storageEvents == 1 and storageDispatchCount == 1 and storageEvents[1].stored == 8)
    "#,
    )
    .unwrap();
    assert_model(&env, [8, 5, 7]);
}

#[test]
fn unknown_full_keys_error_without_inserting_or_emitting() {
    let env = listen_for_storage();
    env.exec(
        r#"
        for _, id in ipairs({
            {recordID = 998877, entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 99},
            {recordID = 1001, entryType = -91, variantIdentifier = 1},
        }) do
            assertRejected(id, 6)
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id) == nil)
        end
    "#,
    )
    .unwrap();
    assert_model(&env, [3, 5, 7]);
}

#[test]
fn malformed_selectors_error_without_mutation_or_event() {
    let env = listen_for_storage();
    env.exec(
        r#"
        assertRejected(nil, 6)
        for _, id in ipairs({false, 1001, '1001', {},
            {recordID = 1001, entryType = firstID.entryType},
            {entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001, variantIdentifier = 1},
            {recordID = '1001', entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001.5, entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001, entryType = {}, variantIdentifier = 1},
            {recordID = 1001, entryType = 1.5, variantIdentifier = 1},
            {recordID = 1001, entryType = firstID.entryType, variantIdentifier = '1'},
            {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 1.5},
        }) do
            assertRejected(id, 6)
        end
    "#,
    )
    .unwrap();
    assert_model(&env, [3, 5, 7]);
}

#[test]
fn malformed_negative_fractional_and_nonfinite_counts_error_atomically() {
    let env = listen_for_storage();
    env.exec(
        r#"
        assertRejected(firstID, nil)
        for _, count in ipairs({false, {}, '6', -1, 1.5, math.huge, -math.huge, 0/0, 2147483648}) do
            assertRejected(firstID, count)
        end
    "#,
    )
    .unwrap();
    assert_model(&env, [3, 5, 7]);
}

#[test]
fn listener_can_read_and_reenter_input_before_outer_return() {
    let env = listen_for_storage();
    env.exec(
        r#"
        local trace = {}
        storageListener:SetScript('OnEvent', function(_, event, ...)
            local id = ...
            trace[#trace + 1] = {
                event = event, arity = select('#', ...), record = id.recordID,
                entryType = id.entryType, variant = id.variantIdentifier,
                stored = C_HousingCatalog.GetCatalogEntryVariantInfo(id).numStored,
            }
            if #trace == 1 then
                assertStored(9, 5, 7)
                A_Admin.SetHousingCatalogVariantStoredCount(id, 12)
                A_Admin.SetHousingCatalogVariantStoredCount(secondID, 14)
                nestedReturned = true
            end
        end)
        A_Admin.SetHousingCatalogVariantStoredCount(firstID, 9)
        assert(nestedReturned == true, 'nested calls must finish inside listener')
        assert(#trace == 3, 'both nested transitions must dispatch synchronously')
        for index, expected in ipairs({{1, 9}, {1, 12}, {2, 14}}) do
            local row = trace[index]
            assert(row.event == 'HOUSING_STORAGE_ENTRY_UPDATED' and row.arity == 1)
            assert(row.record == 1001 and row.entryType == firstID.entryType)
            assert(row.variant == expected[1] and row.stored == expected[2])
        end
        assertStored(12, 14, 7)
    "#,
    )
    .unwrap();
    assert_model(&env, [12, 14, 7]);
}

#[test]
fn public_addon_input_preserves_caller_taint() {
    let env = listen_for_storage();
    env.exec(r#"
        local function addon()
            assert(not issecure())
            local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 2}
            assert(select('#', A_Admin.SetHousingCatalogVariantStoredCount(id, 10)) == 0)
            assert(not issecure(), 'admin input must not clear addon taint')
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id).numStored == 10)
        end
        debug.setobjecttaint(addon, 'HousingStorageFixture')
        local ok, message = pcall(addon)
        assert(ok, message)
        assert(#storageEvents == 1 and storageEvents[1].stored == 10)
        assert(storageEvents[1].arity == 1 and storageEvents[1].id.variantIdentifier == 2)
    "#).unwrap();
    assert_model(&env, [3, 10, 7]);
}

fn install_secret_inputs(env: &WowLuaEnv) {
    let decor: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, value) in [
        ("SecretRecord", 1001.0),
        ("SecretType", f64::from(decor)),
        ("SecretVariant", 1.0),
        ("SecretStored", 9.0),
    ] {
        let secret = wrap_host_secret_number(lua.state_mut(), value);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val(name, secret);
        lua.state_mut().pop();
        inserted.unwrap();
    }
}

#[test]
fn secret_selectors_and_counts_reject_secure_and_tainted_callers() {
    let env = listen_for_storage();
    install_secret_inputs(&env);
    env.exec(r#"
        collectgarbage('collect')
        local function rejectSecrets()
            assertRejected(SecretRecord, 9)
            assertRejected({recordID = SecretRecord, entryType = firstID.entryType, variantIdentifier = 1}, 9)
            assertRejected({recordID = 1001, entryType = SecretType, variantIdentifier = 1}, 9)
            assertRejected({recordID = 1001, entryType = firstID.entryType, variantIdentifier = SecretVariant}, 9)
            assertRejected(firstID, SecretStored)
        end
        rejectSecrets()
        local function addon()
            rejectSecrets()
            assert(not issecure(), 'rejection must preserve caller taint')
        end
        debug.setobjecttaint(addon, 'HousingStorageFixture')
        addon()
        assertStored(3, 5, 7)
        assert(#storageEvents == 0)
    "#).unwrap();
    assert_model(&env, [3, 5, 7]);
}

#[test]
fn empty_catalog_rejects_unknown_key_without_fake_records() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(type(A_Admin.SetHousingCatalogVariantStoredCount) == 'function',
            'explicit housing storage input must be implemented')
        local events = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('HOUSING_STORAGE_ENTRY_UPDATED')
        listener:SetScript('OnEvent', function() events = events + 1 end)
        local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
        local ok, message = pcall(A_Admin.SetHousingCatalogVariantStoredCount, id, 6)
        assert(not ok and type(message) == 'string' and #message > 0)
        assert(events == 0)
        assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id) == nil)
    "#).unwrap();
    let state = env.state().borrow();
    assert!(state.housing.catalog.entries.is_empty());
    assert!(state.housing.catalog.variants.is_empty());
    assert!(
        !state
            .events
            .pending()
            .iter()
            .any(|event| event.name == "HOUSING_STORAGE_ENTRY_UPDATED")
    );
}

#[test]
fn storage_input_and_event_are_environment_local() {
    let first = listen_for_storage();
    let second = listen_for_storage();
    first
        .exec(
            "A_Admin.SetHousingCatalogVariantStoredCount(firstID, 9); assert(#storageEvents == 1)",
        )
        .unwrap();
    second
        .exec("assertStored(3, 5, 7); assert(#storageEvents == 0)")
        .unwrap();
    assert_model(&first, [9, 5, 7]);
    assert_model(&second, [3, 5, 7]);
}
