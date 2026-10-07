//! Grouped coverage for explicit aggregates and current raw catalog snapshots.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogVariantRecord,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn entry_id(env: &WowLuaEnv) -> HousingCatalogEntryID {
    HousingCatalogEntryID {
        record_id: 82001,
        entry_type: env
            .eval("return Enum.HousingCatalogEntryType.Decor")
            .unwrap(),
    }
}

fn fixture_env(stored: Option<u32>, placed: Option<u32>) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let id = entry_id(&env);
    let mut state = env.state().borrow_mut();
    state.housing.catalog.entries.insert(
        id,
        HousingCatalogEntryRecord {
            item_id: Some(6948),
            name: "Aggregate fixture chair".into(),
            is_unique_trophy: false,
            total_num_stored: stored,
            remaining_redeemable: None,
            total_num_placed: placed,
        },
    );
    // Incomplete variant map: its sum is deliberately not the supplied aggregate.
    for (variant_identifier, num_stored) in [(1, 3), (2, 5)] {
        state.housing.catalog.variants.insert(
            HousingCatalogEntryVariantID {
                record_id: id.record_id,
                entry_type: id.entry_type,
                variant_identifier,
            },
            HousingCatalogVariantRecord {
                num_stored,
                destroyable_instance_count: 1,
                dye_slots: vec![],
            },
        );
    }
    drop(state);
    env.exec(
        r#"
        aggregateQueries = {
            function() return C_HousingCatalog.GetCatalogEntryInfo({
                recordID = 82001, entryType = Enum.HousingCatalogEntryType.Decor}) end,
            function() return C_HousingCatalog.GetCatalogEntryInfoByItem(6948) end,
            function() return C_HousingCatalog.GetCatalogEntryInfoByRecordID(
                Enum.HousingCatalogEntryType.Decor, 82001) end,
        }
        function assertAggregates(info, stored, placed)
            assert(info and info.recordID == 82001 and info.itemID == 6948)
            assert(info.totalNumStored == stored, 'explicit totalNumStored mismatch')
            assert(info.totalNumPlaced == placed, 'explicit totalNumPlaced mismatch')
            assert(info.numStored == nil and info.numPlaced == nil)
            assert(info.entryVariantID == nil and info.variantIdentifier == nil)
            assert(info.remainingRedeemable == nil, 'must not invent redeemable data')
        end
        function assertAllAggregates(stored, placed)
            for _, query in ipairs(aggregateQueries) do
                assertAggregates(query(), stored, placed)
            end
        end
        function assertVariantCounts(first, second)
            for variant, count in ipairs({first, second}) do
                local info = C_HousingCatalog.GetCatalogEntryVariantInfo({
                    recordID = 82001, entryType = Enum.HousingCatalogEntryType.Decor,
                    variantIdentifier = variant})
                assert(info.entryVariantID.variantIdentifier == variant)
                assert(info.numStored == count, 'variant count coupled to aggregate')
                assert(info.totalNumStored == nil and info.totalNumPlaced == nil)
            end
        end
    "#,
    )
    .unwrap();
    env
}

fn assert_query_publishes(index: usize) {
    let env = fixture_env(Some(37), Some(11));
    env.exec(&format!(
        "assertAggregates(aggregateQueries[{index}](), 37, 11); assertVariantCounts(3, 5)"
    ))
    .unwrap();
}

#[test]
fn redeemable_count_is_an_independent_host_snapshot() {
    let env = fixture_env(Some(0), Some(11));
    let id = entry_id(&env);
    env.state().borrow_mut().housing.catalog.entries.get_mut(&id).unwrap().remaining_redeemable = Some(13);
    env.exec(r#"
        for _, query in ipairs(aggregateQueries) do
            local info = query()
            assert(info.totalNumStored == 0 and info.totalNumPlaced == 11)
            assert(info.remainingRedeemable == 13, 'redeemable count must not be derived from stored variants')
            info.remainingRedeemable = 999
            assert(query().remainingRedeemable == 13, 'redeemable snapshot aliases host data')
        end
        assertVariantCounts(3, 5)
    "#).unwrap();
    env.state().borrow_mut().housing.catalog.entries.get_mut(&id).unwrap().remaining_redeemable = Some(0);
    env.exec(r#"
        for _, query in ipairs(aggregateQueries) do
            assert(query().remainingRedeemable == 0, 'explicit zero must not be omitted')
        end
    "#).unwrap();
}

#[test]
fn entry_info_publishes_explicit_aggregates() {
    assert_query_publishes(1);
}

#[test]
fn by_item_publishes_explicit_aggregates() {
    assert_query_publishes(2);
}

#[test]
fn by_record_id_publishes_explicit_aggregates() {
    assert_query_publishes(3);
}

#[test]
fn explicit_zero_is_not_missing_inferred_policy() {
    // Simulator inference, not native default evidence; nonzero variants remain distinct.
    let env = fixture_env(Some(0), Some(0));
    env.exec("assertAllAggregates(0, 0); assertVariantCounts(3, 5)")
        .unwrap();
}

#[test]
fn missing_aggregates_are_nil_not_variant_sums_inferred_gap() {
    // Native declarations require numbers. Nil marks absent simulator input, not parity.
    let env = fixture_env(None, None);
    env.exec("assertAllAggregates(nil, nil); assertVariantCounts(3, 5)")
        .unwrap();
}

#[test]
fn each_aggregate_can_be_missing_independently_inferred_gap() {
    let stored_only = fixture_env(Some(37), None);
    stored_only.exec("assertAllAggregates(37, nil)").unwrap();
    let placed_only = fixture_env(None, Some(11));
    placed_only.exec("assertAllAggregates(nil, 11)").unwrap();
}

#[test]
fn nonnegative_counts_preserve_values_above_signed_range() {
    let env = fixture_env(Some(u32::MAX), Some(2_147_483_648));
    env.exec("assertAllAggregates(4294967295, 2147483648)")
        .unwrap();
}

#[test]
fn variant_and_unrelated_housing_mutation_do_not_change_aggregates() {
    let env = fixture_env(Some(37), Some(11));
    env.exec("assertAllAggregates(37, 11)").unwrap();
    let id = entry_id(&env);
    let mut state = env.state().borrow_mut();
    let variant_id = HousingCatalogEntryVariantID {
        record_id: id.record_id,
        entry_type: id.entry_type,
        variant_identifier: 1,
    };
    state
        .housing
        .catalog
        .variants
        .get_mut(&variant_id)
        .unwrap()
        .num_stored = 9;
    // Pending selection, room membership and spent budget are not placed-instance totals.
    state.housing.pending_new_decor = Some(variant_id);
    state.housing.rooms_with_decor = vec![101, 102, 103];
    state.housing.spent_indoor_placement_budget = Some(77);
    state.housing.spent_outdoor_placement_budget = Some(88);
    drop(state);
    env.exec("assertAllAggregates(37, 11); assertVariantCounts(9, 5)")
        .unwrap();
    env.state().borrow_mut().housing.catalog.variants.clear();
    env.exec("assertAllAggregates(37, 11)").unwrap();
}

#[test]
fn aggregate_input_mutation_does_not_change_variants_or_pending_state() {
    let env = fixture_env(Some(37), Some(11));
    let id = entry_id(&env);
    let pending = HousingCatalogEntryVariantID {
        record_id: id.record_id,
        entry_type: id.entry_type,
        variant_identifier: 2,
    };
    let mut state = env.state().borrow_mut();
    state.housing.pending_new_decor = Some(pending);
    let record = state.housing.catalog.entries.get_mut(&id).unwrap();
    record.total_num_stored = Some(41);
    record.total_num_placed = Some(0);
    drop(state);
    env.exec("assertAllAggregates(41, 0); assertVariantCounts(3, 5)")
        .unwrap();
    assert_eq!(
        env.state().borrow().housing.pending_new_decor,
        Some(pending)
    );
}

#[test]
fn snapshots_are_independent_across_lua_and_input_mutation() {
    let env = fixture_env(Some(37), Some(11));
    env.exec(
        r#"
        savedAggregates = {}
        for index, query in ipairs(aggregateQueries) do
            savedAggregates[index] = query()
            assertAggregates(savedAggregates[index], 37, 11)
        end
        for index, info in ipairs(savedAggregates) do
            info.totalNumStored = 100 + index
            info.totalNumPlaced = 200 + index
        end
        assertAllAggregates(37, 11)
        for index, info in ipairs(savedAggregates) do
            assertAggregates(info, 100 + index, 200 + index)
        end
        pristineAggregates = {}
        for index, query in ipairs(aggregateQueries) do pristineAggregates[index] = query() end
    "#,
    )
    .unwrap();
    let id = entry_id(&env);
    let mut state = env.state().borrow_mut();
    let record = state.housing.catalog.entries.get_mut(&id).unwrap();
    record.total_num_stored = Some(0);
    record.total_num_placed = None;
    drop(state);
    env.exec(
        r#"
        collectgarbage('collect')
        assertAllAggregates(0, nil)
        for index, info in ipairs(savedAggregates) do
            assertAggregates(info, 100 + index, 200 + index)
            assertAggregates(pristineAggregates[index], 37, 11)
        end
        assertVariantCounts(3, 5)
    "#,
    )
    .unwrap();
}

#[test]
fn missing_base_record_is_nil_even_with_surviving_variants() {
    let env = fixture_env(Some(37), Some(11));
    let id = entry_id(&env);
    env.state().borrow_mut().housing.catalog.entries.remove(&id);
    env.exec(
        r#"
        for _, query in ipairs(aggregateQueries) do assert(query() == nil) end
        assertVariantCounts(3, 5)
        assert(C_HousingCatalog.GetCatalogEntryInfo({recordID = 998877,
            entryType = Enum.HousingCatalogEntryType.Decor}) == nil)
        assert(C_HousingCatalog.GetCatalogEntryInfoByItem(998877) == nil)
        assert(C_HousingCatalog.GetCatalogEntryInfoByRecordID(
            Enum.HousingCatalogEntryType.Decor, 998877) == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn raw_entry_snapshots_omit_removed_fields_after_legacy_injection() {
    let env = fixture_env(Some(37), Some(11));
    env.exec(
        r#"
        local removedFields = {
            'showQuantity', 'quantity', 'numPlaced', 'customizations', 'dyeIDs',
            'entryID',
        }
        local function assertCurrentSnapshot(info)
            assertAggregates(info, 37, 11)
            assert(rawget(info, 'recordID') == 82001)
            assert(rawget(info, 'entryType') == Enum.HousingCatalogEntryType.Decor)
            assert(rawget(info, 'itemID') == 6948)
            assert(rawget(info, 'name') == 'Aggregate fixture chair')
            assert(rawget(info, 'isUniqueTrophy') == false)
            assert(rawget(info, 'totalNumStored') == 37)
            assert(rawget(info, 'totalNumPlaced') == 11)
            for _, field in ipairs(removedFields) do
                assert(rawget(info, field) == nil, 'raw snapshot retains ' .. field)
            end
        end
        local snapshots = {}
        for index, query in ipairs(aggregateQueries) do
            local info = query()
            assertCurrentSnapshot(info)
            snapshots[index] = info
            for _, field in ipairs(removedFields) do
                rawset(info, field, {snapshot = index, field = field})
            end
        end
        for _, query in ipairs(aggregateQueries) do
            assertCurrentSnapshot(query())
        end
        for index, info in ipairs(snapshots) do
            for _, field in ipairs(removedFields) do
                local injected = rawget(info, field)
                assert(injected.snapshot == index and injected.field == field)
            end
        end
    "#,
    )
    .unwrap();
}

#[test]
fn aggregates_are_environment_local() {
    let supplied = fixture_env(Some(37), Some(11));
    let missing = fixture_env(None, None);
    supplied.exec("assertAllAggregates(37, 11)").unwrap();
    missing.exec("assertAllAggregates(nil, nil)").unwrap();
}
