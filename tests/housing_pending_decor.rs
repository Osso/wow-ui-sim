//! Input-only 12.0.5 pending-request contract; compiled behavioral RED is still required.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryVariantID, HousingCatalogVariantRecord, HousingDecorDyeSlot,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn variant_id(env: &WowLuaEnv, entry_type: &str, variant: i32) -> HousingCatalogEntryVariantID {
    HousingCatalogEntryVariantID {
        record_id: 1001,
        entry_type: env
            .eval(&format!("return Enum.HousingCatalogEntryType.{entry_type}"))
            .unwrap(),
        variant_identifier: variant,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    for (entry_type, variant, stored, eligible, dye) in [
        ("Decor", 1, 3, 0, 701),
        ("Decor", 2, 5, 4, 702),
        ("Room", 1, 7, 2, 703),
        ("Decor", 3, 0, 1, 704),
    ] {
        let id = variant_id(&env, entry_type, variant);
        env.state().borrow_mut().housing.catalog.variants.insert(
            id,
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
            },
        );
    }
    env.exec(SELECTORS).unwrap();
    env
}

const SELECTORS: &str = r#"
    firstID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Decor, variantIdentifier = 1}
    secondID = {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 2}
    roomID = {recordID = 1001, entryType = Enum.HousingCatalogEntryType.Room, variantIdentifier = 1}
    zeroID = {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 3}
"#;

fn assert_pending(env: &WowLuaEnv, expected: Option<HousingCatalogEntryVariantID>) {
    assert_eq!(env.state().borrow().housing.pending_new_decor, expected);
    let placing: bool = env
        .eval("return C_HousingBasicMode.IsPlacingNewDecor()")
        .unwrap();
    assert_eq!(placing, expected.is_some());
}

fn start_first(env: &WowLuaEnv) {
    env.exec("C_HousingBasicMode.StartPlacingNewDecor(firstID)")
        .unwrap();
    assert_pending(env, Some(variant_id(env, "Decor", 1)));
}

fn assert_catalog_unchanged(env: &WowLuaEnv) {
    let decor = variant_id(env, "Decor", 1).entry_type;
    let room = variant_id(env, "Room", 1).entry_type;
    let state = env.state().borrow();
    let variants = &state.housing.catalog.variants;
    assert_eq!(variants.len(), 4);
    assert!(state.housing.catalog.entries.is_empty());
    for (entry_type, variant_identifier, stored, eligible, dye) in [
        (decor, 1, 3, 0, 701),
        (decor, 2, 5, 4, 702),
        (room, 1, 7, 2, 703),
        (decor, 3, 0, 1, 704),
    ] {
        let id = HousingCatalogEntryVariantID {
            record_id: 1001,
            entry_type,
            variant_identifier,
        };
        let record = &variants[&id];
        assert_eq!(
            (record.num_stored, record.destroyable_instance_count),
            (stored, eligible)
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
        assert_eq!(slot.dye_color_name, Some(format!("Fixture dye {dye}")));
    }
}

#[test]
fn plain_default_and_cancel_without_input_guards() {
    let env = WowLuaEnv::new().unwrap();
    assert_pending(&env, None);
    env.exec(
        r#"
        assert(select('#', C_HousingBasicMode.IsPlacingNewDecor()) == 1)
        assert(select('#', C_HousingBasicMode.CancelActiveEditing()) == 0)
        C_HousingBasicMode.CancelActiveEditing()
    "#,
    )
    .unwrap();
    assert_pending(&env, None);
}

#[test]
fn plain_missing_selector_errors_without_catalog_or_pending_setup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ok, message = pcall(C_HousingBasicMode.StartPlacingNewDecor)
        assert(not ok and type(message) == 'string' and #message > 0,
            'missing selector must error instead of silently accepting the no-op')
    "#,
    )
    .unwrap();
    assert_pending(&env, None);
}

#[test]
fn plain_invalid_selectors_error_without_input_guards() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, selector in ipairs({false, 1001, '1001', {}, function() end}) do
            local ok, message = pcall(C_HousingBasicMode.StartPlacingNewDecor, selector)
            assert(not ok and type(message) == 'string' and #message > 0)
        end
        assert(not pcall(C_HousingBasicMode.StartPlacingNewDecor, nil))
    "#,
    )
    .unwrap();
    assert_pending(&env, None);
}

#[test]
fn start_and_cancel_record_only_pending_request() {
    let env = fixture_env();
    env.exec("assert(select('#', C_HousingBasicMode.StartPlacingNewDecor(firstID)) == 0)")
        .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 1)));
    assert_catalog_unchanged(&env);
    env.exec("C_HousingBasicMode.CancelActiveEditing()")
        .unwrap();
    assert_pending(&env, None);
    assert_catalog_unchanged(&env);
    start_first(&env);
}

#[test]
fn repeated_and_replacement_requests_preserve_full_identity() {
    let env = fixture_env();
    start_first(&env);
    for (selector, entry_type, variant) in [
        ("firstID", "Decor", 1),
        ("secondID", "Decor", 2),
        ("roomID", "Room", 1),
        ("firstID", "Decor", 1),
    ] {
        env.exec(&format!(
            "C_HousingBasicMode.StartPlacingNewDecor({selector})"
        ))
        .unwrap();
        assert_pending(&env, Some(variant_id(&env, entry_type, variant)));
    }
    env.exec(
        "firstID.recordID = 998877; firstID.variantIdentifier = 99; collectgarbage('collect')",
    )
    .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 1)));
    assert_catalog_unchanged(&env);
}

#[test]
fn replacement_request_changes_record_identity() {
    let env = fixture_env();
    let mut other = variant_id(&env, "Decor", 1);
    other.record_id = 1002;
    let record =
        env.state().borrow().housing.catalog.variants[&variant_id(&env, "Decor", 1)].clone();
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .variants
        .insert(other, record);
    start_first(&env);
    env.exec("C_HousingBasicMode.StartPlacingNewDecor({recordID = 1002, entryType = firstID.entryType, variantIdentifier = 1})").unwrap();
    assert_pending(&env, Some(other));
    let state = env.state().borrow();
    assert_eq!(state.housing.catalog.variants.len(), 5);
    for id in [
        other,
        HousingCatalogEntryVariantID {
            record_id: 1001,
            ..other
        },
    ] {
        let record = &state.housing.catalog.variants[&id];
        assert_eq!(
            (record.num_stored, record.destroyable_instance_count),
            (3, 0)
        );
        assert_eq!(record.dye_slots[0].dye_color_id, Some(701));
    }
}

#[test]
fn nonpositive_stock_cannot_replace_pending_request() {
    let env = fixture_env();
    start_first(&env);
    let zero = variant_id(&env, "Decor", 3);
    env.state()
        .borrow_mut()
        .housing
        .catalog
        .variants
        .get_mut(&zero)
        .unwrap()
        .num_stored = -1;
    env.exec("C_HousingBasicMode.StartPlacingNewDecor(zeroID)")
        .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 1)));
    assert_eq!(
        env.state().borrow().housing.catalog.variants[&zero].num_stored,
        -1
    );
}

#[test]
fn unknown_and_zero_stock_noop_preserve_empty_or_existing_pending() {
    let env = fixture_env();
    for pending in [false, true] {
        if pending {
            start_first(&env);
        }
        env.exec(
            r#"
            for _, id in ipairs({zeroID,
                {recordID = 998877, entryType = firstID.entryType, variantIdentifier = 1},
                {recordID = 1001, entryType = -91, variantIdentifier = 1},
                {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 99},
                {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 0},
            }) do
                assert(select('#', C_HousingBasicMode.StartPlacingNewDecor(id)) == 0)
            end
        "#,
        )
        .unwrap();
        assert_pending(&env, pending.then(|| variant_id(&env, "Decor", 1)));
        assert_catalog_unchanged(&env);
    }
}

#[test]
fn malformed_selector_errors_atomically_with_existing_pending() {
    let env = fixture_env();
    start_first(&env);
    env.exec(
        r#"
        local function reject(id)
            local ok, message = pcall(C_HousingBasicMode.StartPlacingNewDecor, id)
            assert(not ok and type(message) == 'string' and #message > 0)
        end
        for _, id in ipairs({{}, {recordID = 1001},
            {recordID = 1001, entryType = firstID.entryType},
            {entryType = firstID.entryType, variantIdentifier = 1},
            {recordID = 1001, variantIdentifier = 1},
        }) do reject(id) end
        for _, field in ipairs({'recordID', 'entryType', 'variantIdentifier'}) do
            for _, value in ipairs({false, {}, '1', 1.5, 0/0, math.huge, -math.huge, 2147483648}) do
                local id = {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 1}
                id[field] = value
                reject(id)
            end
        end
    "#,
    )
    .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 1)));
    assert_catalog_unchanged(&env);
}

#[test]
fn public_addon_requests_and_cancel_preserve_taint() {
    let env = fixture_env();
    env.exec(
        r#"
        local function addon()
            assert(not issecure())
            C_HousingBasicMode.StartPlacingNewDecor(secondID)
            assert(C_HousingBasicMode.IsPlacingNewDecor())
            assert(not issecure(), 'start must not clear addon taint')
        end
        debug.setobjecttaint(addon, 'HousingPendingFixture')
        addon()
    "#,
    )
    .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 2)));
    env.exec(
        r#"
        local function addon()
            C_HousingBasicMode.CancelActiveEditing()
            assert(not C_HousingBasicMode.IsPlacingNewDecor())
            assert(not issecure(), 'cancel must not clear addon taint')
        end
        debug.setobjecttaint(addon, 'HousingPendingFixture')
        addon()
    "#,
    )
    .unwrap();
    assert_pending(&env, None);
    assert_catalog_unchanged(&env);
}

fn install_secret_selectors(env: &WowLuaEnv) {
    let id = variant_id(env, "Decor", 1);
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, number) in [
        ("SecretRecord", id.record_id),
        ("SecretType", id.entry_type),
        ("SecretVariant", id.variant_identifier),
    ] {
        let secret = wrap_host_secret_number(lua.state_mut(), f64::from(number));
        lua.state_mut().push(secret);
        let result = lua.set_global_val(name, secret);
        lua.state_mut().pop();
        result.unwrap();
    }
}

#[test]
fn secret_selectors_reject_atomically_in_secure_and_tainted_callers() {
    let env = fixture_env();
    start_first(&env);
    install_secret_selectors(&env);
    env.exec(
        r#"
        collectgarbage('collect')
        local function rejectSecrets()
            for _, id in ipairs({SecretRecord,
                {recordID = SecretRecord, entryType = firstID.entryType, variantIdentifier = 2},
                {recordID = 1001, entryType = SecretType, variantIdentifier = 2},
                {recordID = 1001, entryType = firstID.entryType, variantIdentifier = SecretVariant},
            }) do
                local ok, message = pcall(C_HousingBasicMode.StartPlacingNewDecor, id)
                assert(not ok and type(message) == 'string' and #message > 0)
            end
        end
        assert(issecure())
        rejectSecrets()
        local function addon()
            rejectSecrets()
            assert(not issecure(), 'secret rejection must not clear caller taint')
        end
        debug.setobjecttaint(addon, 'HousingPendingFixture')
        addon()
    "#,
    )
    .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 1)));
    assert_catalog_unchanged(&env);
}

#[test]
fn guarded_selector_rejects_tainted_access_without_replacing_pending() {
    let env = fixture_env();
    start_first(&env);
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
    }
    env.exec(r#"
        local guarded = {recordID = 1001, entryType = firstID.entryType, variantIdentifier = 2}
        settablesecurity(guarded, 0) -- Explicit host-installed VM policy, not native API parity.
        local function addon()
            local sourceOK, sourceError = pcall(rawget, guarded, 'recordID')
            assert(not sourceOK and string.find(sourceError, 'tainted access to secured table', 1, true))
            local ok, message = pcall(C_HousingBasicMode.StartPlacingNewDecor, guarded)
            assert(not ok and string.find(message, 'tainted access to secured table', 1, true))
            assert(not issecure())
        end
        debug.setobjecttaint(addon, 'HousingPendingFixture')
        addon()
    "#).unwrap();
    assert_pending(&env, Some(variant_id(&env, "Decor", 1)));
    assert_catalog_unchanged(&env);
}

#[test]
fn pending_requests_do_not_leak_between_environments() {
    let first = fixture_env();
    let second = fixture_env();
    let empty = WowLuaEnv::new().unwrap();
    start_first(&first);
    assert_pending(&second, None);
    assert_pending(&empty, None);
    second
        .exec("C_HousingBasicMode.StartPlacingNewDecor(secondID)")
        .unwrap();
    assert_pending(&second, Some(variant_id(&second, "Decor", 2)));
    first
        .exec("C_HousingBasicMode.CancelActiveEditing()")
        .unwrap();
    assert_pending(&first, None);
    assert_pending(&second, Some(variant_id(&second, "Decor", 2)));
    empty.exec(SELECTORS).unwrap();
    empty
        .exec("C_HousingBasicMode.StartPlacingNewDecor(firstID)")
        .unwrap();
    assert_pending(&empty, None);
}

#[test]
fn pending_lifecycle_preserves_instances_preview_and_emits_no_success() {
    let env = fixture_env();
    env.exec(PRESERVE_EXISTING_OUTPUTS).unwrap();
    let events_before: Vec<String> = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .map(|event| event.name.clone())
        .collect();
    for selector in ["firstID", "firstID", "secondID", "zeroID", "roomID"] {
        env.exec(&format!(
            "C_HousingBasicMode.StartPlacingNewDecor({selector}); assertExistingOutputs()"
        ))
        .unwrap();
    }
    env.exec("C_HousingBasicMode.FinishPlacingNewDecor(); assertExistingOutputs()")
        .unwrap();
    assert_pending(&env, Some(variant_id(&env, "Room", 1)));
    env.exec("C_HousingBasicMode.CancelActiveEditing(); assertExistingOutputs(); assert(#pendingEvents == 0)").unwrap();
    assert_pending(&env, None);
    assert_catalog_unchanged(&env);
    let events_after: Vec<String> = env
        .state()
        .borrow()
        .events
        .pending()
        .iter()
        .map(|event| event.name.clone())
        .collect();
    assert_eq!(
        events_after, events_before,
        "request lifecycle must not queue events"
    );
}

const PRESERVE_EXISTING_OUTPUTS: &str = r#"
    local function equal(left, right)
        if type(left) ~= type(right) then return false end
        if type(left) ~= 'table' then return left == right end
        for key, value in pairs(left) do if not equal(value, right[key]) then return false end end
        for key in pairs(right) do if left[key] == nil then return false end end
        return true
    end
    local function snapshot()
        return {
            basic = C_HousingBasicMode.GetSelectedDecorInfo(),
            decor = C_HousingDecor.GetSelectedDecorInfo(),
            customize = C_HousingCustomizeMode.GetSelectedDecorInfo(),
            placed = C_HousingDecor.GetAllPlacedDecor(),
            basicSelected = C_HousingBasicMode.IsDecorSelected(),
            decorSelected = C_HousingDecor.IsDecorSelected(),
            customizeSelected = C_HousingCustomizeMode.IsDecorSelected(),
            preview = C_HousingDecor.IsPreviewState(),
            previewCount = C_HousingDecor.GetNumPreviewDecor(),
        }
    end
    C_HousingDecor.EnterPreviewState()
    local before = snapshot()
    function assertExistingOutputs()
        assert(equal(before, snapshot()), 'pending request must not synthesize or alter instance/preview info')
    end
    pendingEvents = {}
    pendingListener = CreateFrame('Frame')
    for _, event in ipairs({'HOUSING_DECOR_PLACE_SUCCESS', 'HOUSING_DECOR_PLACE_FAILURE',
        'HOUSING_STORAGE_ENTRY_UPDATED', 'HOUSING_BASIC_MODE_SELECTED_TARGET_CHANGED'}) do
        pendingListener:RegisterEvent(event)
    end
    pendingListener:SetScript('OnEvent', function(_, event) pendingEvents[#pendingEvents + 1] = event end)
"#;
