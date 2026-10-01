//! Test-only inferred destruction contract; parent must observe RED before production.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_bool, wrap_host_secret_number};
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogVariantRecord, HousingDecorDyeSlot,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn variant_record(stored: i32, eligible: i32, dye: i32) -> HousingCatalogVariantRecord {
    HousingCatalogVariantRecord {
        num_stored: stored,
        destroyable_instance_count: eligible,
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

fn inject_catalog(env: &WowLuaEnv) {
    let (decor, room): (i32, i32) = env
        .eval("return Enum.HousingCatalogEntryType.Decor, Enum.HousingCatalogEntryType.Room")
        .unwrap();
    let mut state = env.state().borrow_mut();
    for (record_id, entry_type) in [(1001, decor), (1001, room), (1002, decor)] {
        state.housing.catalog.entries.insert(
            HousingCatalogEntryID {
                record_id,
                entry_type,
            },
            HousingCatalogEntryRecord {
                item_id: Some(record_id),
                name: format!("Fixture entry {record_id}"),
                is_unique_trophy: false,
                total_num_stored: None,
                total_num_placed: None,
            },
        );
    }
    for (record_id, entry_type, variant_identifier, stored, eligible, dye) in [
        (1001, decor, 1, 5, 3, 701),
        (1001, decor, 2, 7, 2, 702),
        (1001, room, 1, 6, 1, 703),
        (1002, decor, 1, 8, 4, 704),
    ] {
        state.housing.catalog.variants.insert(
            HousingCatalogEntryVariantID {
                record_id,
                entry_type,
                variant_identifier,
            },
            variant_record(stored, eligible, dye),
        );
    }
}

const LISTENER: &str = r#"
    assert(type(C_HousingCatalog.DestroyEntry) == 'function', 'DestroyEntry must be callable')
    firstID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
    secondID = {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 2}
    roomID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Room, variantIdentifier = 1}
    otherID = {recordID = 1002, entryType = firstID.entryType, variantIdentifier = 1}
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
            eligible = C_HousingCatalog.GetDestroyableInstanceCount(id),
            dye = info.dyeSlots[1].dyeColorID,
        }
    end)
    function assertCounts(id, stored, eligible)
        local info = C_HousingCatalog.GetCatalogEntryVariantInfo(id)
        assert(info and info.numStored == stored, 'stored count must match explicit transition')
        assert(C_HousingCatalog.GetDestroyableInstanceCount(id) == eligible)
    end
    function assertEvent(index, id, stored, eligible, dye)
        local row = storageEvents[index]
        assert(row and row.event == 'HOUSING_STORAGE_ENTRY_UPDATED' and row.arity == 1)
        assert(type(row.id) == 'table')
        assert(row.id.recordID == id.recordID and row.id.entryType == id.entryType)
        assert(row.id.variantIdentifier == id.variantIdentifier)
        assert(row.stored == stored and row.eligible == eligible and row.dye == dye)
    end
    function assertRejected(...)
        local before = storageDispatchCount
        local ok, message = pcall(C_HousingCatalog.DestroyEntry, ...)
        assert(not ok and type(message) == 'string' and #message > 0,
            'invalid destruction input must fail explicitly')
        assert(storageDispatchCount == before, 'rejection must not dispatch an event')
        return message
    end
"#;

fn listen_for_destruction() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    inject_catalog(&env);
    env.exec(LISTENER).unwrap();
    env
}

fn assert_dye(record: &HousingCatalogVariantRecord, dye: i32) {
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
    assert_eq!(slot.dye_color_name, Some(format!("Fixture dye {dye}")));
}

fn assert_model(env: &WowLuaEnv, counts: [(i32, i32); 4]) {
    let (decor, room): (i32, i32) = env
        .eval("return Enum.HousingCatalogEntryType.Decor, Enum.HousingCatalogEntryType.Room")
        .unwrap();
    let state = env.state().borrow();
    let catalog = &state.housing.catalog;
    assert_eq!(catalog.entries.len(), 3);
    assert_eq!(
        catalog.variants.len(),
        4,
        "zero-count variant must remain present"
    );
    for ((record_id, entry_type, variant_identifier, dye), (stored, eligible)) in [
        (1001, decor, 1, 701),
        (1001, decor, 2, 702),
        (1001, room, 1, 703),
        (1002, decor, 1, 704),
    ]
    .into_iter()
    .zip(counts)
    {
        let key = HousingCatalogEntryVariantID {
            record_id,
            entry_type,
            variant_identifier,
        };
        let record = &catalog.variants[&key];
        assert_eq!(
            (record.num_stored, record.destroyable_instance_count),
            (stored, eligible)
        );
        assert_dye(record, dye);
    }
    assert_base_entries(env, decor, room);
    assert!(
        !state
            .events
            .pending()
            .iter()
            .any(|event| event.name == "HOUSING_STORAGE_ENTRY_UPDATED")
    );
}

fn assert_base_entries(env: &WowLuaEnv, decor: i32, room: i32) {
    let state = env.state().borrow();
    for (record_id, entry_type) in [(1001, decor), (1001, room), (1002, decor)] {
        let record = &state.housing.catalog.entries[&HousingCatalogEntryID {
            record_id,
            entry_type,
        }];
        assert_eq!(record.item_id, Some(record_id));
        assert_eq!(record.name, format!("Fixture entry {record_id}"));
        assert!(!record.is_unique_trophy);
    }
}

const INITIAL_COUNTS: [(i32, i32); 4] = [(5, 3), (7, 2), (6, 1), (8, 4)];

#[test]
fn false_destroys_one_eligible_instance_before_synchronous_event() {
    let env = listen_for_destruction();
    env.exec(
        r#"
        assert(select('#', C_HousingCatalog.DestroyEntry(firstID, false)) == 0)
        assert(#storageEvents == 1 and storageDispatchCount == 1, 'must dispatch before return')
        assertEvent(1, firstID, 4, 2, 701)
        assertCounts(firstID, 4, 2)
    "#,
    )
    .unwrap();
    assert_model(&env, [(4, 2), (7, 2), (6, 1), (8, 4)]);
}

#[test]
fn true_destroys_all_eligible_not_all_stored_in_mixed_stack() {
    let env = listen_for_destruction();
    env.exec(
        r#"
        assert(select('#', C_HousingCatalog.DestroyEntry(firstID, true)) == 0)
        assert(#storageEvents == 1 and storageDispatchCount == 1)
        assertEvent(1, firstID, 2, 0, 701)
        assertCounts(firstID, 2, 0)
        assert(select('#', C_HousingCatalog.DestroyEntry(firstID, true)) == 0)
        assert(select('#', C_HousingCatalog.DestroyEntry(firstID, false)) == 0)
        assert(#storageEvents == 1 and storageDispatchCount == 1, 'no eligibility means no event')
    "#,
    )
    .unwrap();
    assert_model(&env, [(2, 0), (7, 2), (6, 1), (8, 4)]);
}

#[test]
fn one_then_all_reduces_both_counts_and_preserves_exempt_instances() {
    let env = listen_for_destruction();
    env.exec(
        r#"
        C_HousingCatalog.DestroyEntry(firstID, false)
        C_HousingCatalog.DestroyEntry(firstID, true)
        assert(#storageEvents == 2 and storageDispatchCount == 2)
        assertEvent(1, firstID, 4, 2, 701)
        assertEvent(2, firstID, 2, 0, 701)
    "#,
    )
    .unwrap();
    assert_model(&env, [(2, 0), (7, 2), (6, 1), (8, 4)]);
}

fn replace_first_counts(env: &WowLuaEnv, stored: i32, eligible: i32) {
    let decor: i32 = env
        .eval("return Enum.HousingCatalogEntryType.Decor")
        .unwrap();
    let mut state = env.state().borrow_mut();
    let key = HousingCatalogEntryVariantID {
        record_id: 1001,
        entry_type: decor,
        variant_identifier: 1,
    };
    let record = state.housing.catalog.variants.get_mut(&key).unwrap();
    record.num_stored = stored;
    record.destroyable_instance_count = eligible;
}

#[test]
fn fully_eligible_stack_retains_zero_record_after_one_or_all() {
    for destroy_all in [false, true] {
        let env = listen_for_destruction();
        replace_first_counts(&env, 1, 1);
        env.exec(&format!(
            r#"
            assert(select('#', C_HousingCatalog.DestroyEntry(firstID, {destroy_all})) == 0)
            assert(#storageEvents == 1 and storageDispatchCount == 1)
            assertEvent(1, firstID, 0, 0, 701)
            assertCounts(firstID, 0, 0)
            C_HousingCatalog.DestroyEntry(firstID, {destroy_all})
            assert(#storageEvents == 1 and storageDispatchCount == 1)
        "#
        ))
        .unwrap();
        assert_model(&env, [(0, 0), (7, 2), (6, 1), (8, 4)]);
    }
}

#[test]
fn full_key_isolates_variant_type_and_record() {
    let env = listen_for_destruction();
    env.exec(
        r#"
        C_HousingCatalog.DestroyEntry(secondID, false)
        C_HousingCatalog.DestroyEntry(roomID, true)
        C_HousingCatalog.DestroyEntry(otherID, true)
        assert(#storageEvents == 3 and storageDispatchCount == 3)
        assertEvent(1, secondID, 6, 1, 702)
        assertEvent(2, roomID, 5, 0, 703)
        assertEvent(3, otherID, 4, 0, 704)
        assertCounts(firstID, 5, 3)
    "#,
    )
    .unwrap();
    assert_model(&env, [(5, 3), (6, 1), (5, 0), (4, 0)]);
}

#[test]
fn missing_keys_and_explicit_zero_eligibility_are_noops() {
    let env = listen_for_destruction();
    replace_first_counts(&env, 5, 0);
    env.exec(
        r#"
        for _, id in ipairs({firstID,
            {recordID = 998877, entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 99},
            {recordID = 1001, entryType = 998877, variantIdentifier = 1},
        }) do
            for _, destroyAll in ipairs({false, true}) do
                assert(select('#', C_HousingCatalog.DestroyEntry(id, destroyAll)) == 0)
            end
        end
        assert(storageDispatchCount == 0 and #storageEvents == 0)
        assertCounts(firstID, 5, 0)
    "#,
    )
    .unwrap();
    assert_model(&env, [(5, 0), (7, 2), (6, 1), (8, 4)]);
}

#[test]
fn empty_catalog_stays_empty_without_invented_permission() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(LISTENER).unwrap();
    env.exec(
        r#"
        for _, destroyAll in ipairs({false, true}) do
            assert(select('#', C_HousingCatalog.DestroyEntry(firstID, destroyAll)) == 0)
        end
        assert(storageDispatchCount == 0 and #storageEvents == 0)
        assert(C_HousingCatalog.GetCatalogEntryVariantInfo(firstID) == nil)
        assert(C_HousingCatalog.GetDestroyableInstanceCount(firstID) == 0)
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert!(state.housing.catalog.variants.is_empty());
    assert!(state.housing.catalog.entries.is_empty());
    assert!(
        !state
            .events
            .pending()
            .iter()
            .any(|event| event.name == "HOUSING_STORAGE_ENTRY_UPDATED")
    );
}

#[test]
fn malformed_public_selectors_fail_atomically() {
    let env = listen_for_destruction();
    env.exec(r#"
        assertRejected(nil, false)
        for _, id in ipairs({false, 1001, '1001', {},
            {recordID = 1001, entryType = firstID.entryType},
            {entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001, variantIdentifier = 1},
        }) do
            assertRejected(id, false)
            assertRejected(id, true)
        end
        for _, field in ipairs({'recordID', 'entryType', 'variantIdentifier'}) do
            for _, value in ipairs({'1', false, {}, -1, 1.5, 0/0, math.huge, -math.huge, 2147483648}) do
                local id = {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 1}
                id[field] = value
                assertRejected(id, false)
                assertRejected(id, true)
            end
        end
        assert(storageDispatchCount == 0 and #storageEvents == 0)
    "#).unwrap();
    assert_model(&env, INITIAL_COUNTS);
}

#[test]
fn destroy_all_is_required_public_boolean_even_for_missing_keys() {
    let env = listen_for_destruction();
    env.exec(
        r#"
        assertRejected()
        for _, id in ipairs({firstID,
            {recordID = 998877, entryType = firstID.entryType, variantIdentifier = 1},
        }) do
            assertRejected(id)
            assertRejected(id, nil)
            for _, value in ipairs({0, 1, 'false', 'true', {}, function() end}) do
                assertRejected(id, value)
            end
        end
        assert(storageDispatchCount == 0 and #storageEvents == 0)
    "#,
    )
    .unwrap();
    assert_model(&env, INITIAL_COUNTS);
}

#[test]
fn inconsistent_counts_fail_explicitly_without_clamping_mutating_or_emitting() {
    for (stored, eligible, field) in [
        (-1, 0, "numStored"),
        (5, -1, "destroyable"),
        (2, 3, "destroyable"),
        (0, 1, "destroyable"),
    ] {
        let env = listen_for_destruction();
        replace_first_counts(&env, stored, eligible);
        env.exec(&format!(
            r#"
            local one = assertRejected(firstID, false)
            local all = assertRejected(firstID, true)
            assert(one == all, 'same invalid model input must fail consistently for one/all')
            assert(string.find(one, '{field}', 1, true), 'error must name inconsistent count')
            assert(storageDispatchCount == 0 and #storageEvents == 0)
        "#
        ))
        .unwrap();
        assert_model(&env, [(stored, eligible), (7, 2), (6, 1), (8, 4)]);
    }
}

#[test]
fn listener_reads_and_reenters_same_and_other_variant_before_outer_return() {
    let env = listen_for_destruction();
    env.exec(r#"
        local trace = {}
        storageListener:SetScript('OnEvent', function(_, event, ...)
            local id = ...
            trace[#trace + 1] = {
                event = event, arity = select('#', ...), record = id.recordID,
                entryType = id.entryType, variant = id.variantIdentifier,
                stored = C_HousingCatalog.GetCatalogEntryVariantInfo(id).numStored,
                eligible = C_HousingCatalog.GetDestroyableInstanceCount(id),
            }
            if #trace == 1 then
                C_HousingCatalog.DestroyEntry(id, true)
                C_HousingCatalog.DestroyEntry(secondID, false)
                nestedReturned = true
            end
        end)
        C_HousingCatalog.DestroyEntry(firstID, false)
        assert(nestedReturned == true, 'nested mutations must finish inside listener')
        assert(#trace == 3, 'nested transitions must dispatch before outer return')
        for index, expected in ipairs({{1, 4, 2}, {1, 2, 0}, {2, 6, 1}}) do
            local row = trace[index]
            assert(row.event == 'HOUSING_STORAGE_ENTRY_UPDATED' and row.arity == 1)
            assert(row.record == 1001 and row.entryType == firstID.entryType)
            assert(row.variant == expected[1] and row.stored == expected[2] and row.eligible == expected[3])
        end
        assertCounts(firstID, 2, 0)
        assertCounts(secondID, 6, 1)
    "#).unwrap();
    assert_model(&env, [(2, 0), (6, 1), (6, 1), (8, 4)]);
}

#[test]
fn public_addon_call_preserves_taint() {
    let env = listen_for_destruction();
    env.exec(r#"
        local function addon()
            assert(not issecure())
            local id = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
            assert(select('#', C_HousingCatalog.DestroyEntry(id, false)) == 0)
            assert(not issecure(), 'destruction must not clear caller taint')
            assertCounts(id, 4, 2)
        end
        debug.setobjecttaint(addon, 'HousingDestroyFixture')
        local ok, message = pcall(addon)
        assert(ok, message)
        assert(#storageEvents == 1 and storageDispatchCount == 1)
        assertEvent(1, firstID, 4, 2, 701)
    "#).unwrap();
    assert_model(&env, [(4, 2), (7, 2), (6, 1), (8, 4)]);
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
    ] {
        let secret = wrap_host_secret_number(lua.state_mut(), value);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val(name, secret);
        lua.state_mut().pop();
        inserted.unwrap();
    }
    let secret = wrap_host_secret_bool(lua.state_mut(), true);
    lua.state_mut().push(secret);
    let inserted = lua.set_global_val("SecretAll", secret);
    lua.state_mut().pop();
    inserted.unwrap();
}

#[test]
fn secret_selector_fields_and_boolean_reject_secure_and_tainted_callers() {
    let env = listen_for_destruction();
    install_secret_inputs(&env);
    env.exec(
        r#"
        collectgarbage('collect')
        local function rejectSecrets()
            assertRejected(SecretRecord, false)
            for _, id in ipairs({
                {recordID = SecretRecord, entryType = firstID.entryType, variantIdentifier = 1},
                {recordID = 1001, entryType = SecretType, variantIdentifier = 1},
                {recordID = 1001, entryType = firstID.entryType, variantIdentifier = SecretVariant},
            }) do
                assertRejected(id, false)
                assertRejected(id, true)
            end
            assertRejected(firstID, SecretAll)
        end
        assert(issecure())
        rejectSecrets()
        local function addon()
            rejectSecrets()
            assert(not issecure(), 'secret rejection must preserve caller taint')
        end
        debug.setobjecttaint(addon, 'HousingDestroyFixture')
        addon()
        assert(storageDispatchCount == 0 and #storageEvents == 0)
        assert(issecure(), 'secure fixture caller must remain untainted')
    "#,
    )
    .unwrap();
    assert_model(&env, INITIAL_COUNTS);
}

#[test]
fn guarded_selector_retains_vm_access_policy_without_mutation_on_rejection() {
    let env = listen_for_destruction();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
    }
    env.exec(r#"
        assert(issecure())
        settablesecurity(firstID, 0) -- host-installed VM DisallowTaintedAccess
        local function addon()
            assert(not issecure())
            local sourceOK, sourceError = pcall(rawget, firstID, 'recordID')
            assert(not sourceOK and string.find(sourceError, 'tainted access to secured table', 1, true))
            for _, destroyAll in ipairs({false, true}) do
                local message = assertRejected(firstID, destroyAll)
                assert(string.find(message, 'tainted access to secured table', 1, true))
                assert(not issecure())
            end
        end
        debug.setobjecttaint(addon, 'HousingDestroyFixture')
        addon()
        assert(issecure() and firstID.recordID == 1001)
        assertCounts(firstID, 5, 3)
        assert(storageDispatchCount == 0 and #storageEvents == 0)
    "#).unwrap();
    assert_model(&env, INITIAL_COUNTS);
    env.exec(
        r#"
        C_HousingCatalog.DestroyEntry(firstID, false)
        assert(#storageEvents == 1 and storageDispatchCount == 1)
        assertEvent(1, firstID, 4, 2, 701)
    "#,
    )
    .unwrap();
    assert_model(&env, [(4, 2), (7, 2), (6, 1), (8, 4)]);
}

#[test]
fn destruction_and_events_are_environment_local() {
    let first = listen_for_destruction();
    let second = listen_for_destruction();
    first
        .exec("C_HousingCatalog.DestroyEntry(firstID, true); assert(#storageEvents == 1)")
        .unwrap();
    second.exec("assertCounts(firstID, 5, 3); assert(#storageEvents == 0 and storageDispatchCount == 0)").unwrap();
    assert_model(&first, [(2, 0), (7, 2), (6, 1), (8, 4)]);
    assert_model(&second, INITIAL_COUNTS);
}
