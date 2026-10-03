//! Batch70 controls plus Batch73 removal inputs. Main owns compiled RED/GREEN gates.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod modern {
    use super::*;
    use rilua::LuaApiMut;
    use rilua::table_security::{
        wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string,
    };
    use wow_ui_sim::c_api::c_housing::catalog::{
        HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
        HousingCatalogVariantRecord, HousingDecorDyeSlot,
    };
    use wow_ui_sim::c_api::c_housing::exterior::{
        ExteriorDecorPlacement, ExteriorFixtureOption, ExteriorFixturePoint, ExteriorSizeOption,
        ExteriorTypeOption, HouseExteriorState,
    };

    const CALLS: [(&str, i32, &str); 3] = [
        ("SelectFixtureOption", 302, "HOUSING_SET_FIXTURE_RESPONSE"),
        (
            "SetHouseExteriorSize",
            4,
            "HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE",
        ),
        (
            "SetHouseExteriorType",
            102,
            "HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE",
        ),
    ];

    fn variant() -> HousingCatalogEntryVariantID {
        HousingCatalogEntryVariantID {
            record_id: 7101,
            entry_type: 1,
            variant_identifier: 0,
        }
    }

    fn entry() -> HousingCatalogEntryID {
        HousingCatalogEntryID {
            record_id: 7101,
            entry_type: 1,
        }
    }

    fn fixture_env() -> WowLuaEnv {
        let env = WowLuaEnv::new().unwrap();
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            housing.inside_owned_plot = true;
            housing.active_house_editor_mode = 6;
            housing.catalog.variants.insert(
                variant(),
                HousingCatalogVariantRecord {
                    num_stored: 2,
                    destroyable_instance_count: 1,
                    dye_slots: vec![HousingDecorDyeSlot {
                        id: 7,
                        dye_color_category_id: 8,
                        order_index: 1,
                        channel: 0,
                        dye_color_id: Some(701),
                        dye_color_name: Some("Blue".into()),
                    }],
                },
            );
            housing.catalog.entries.insert(
                entry(),
                HousingCatalogEntryRecord {
                    item_id: Some(7101),
                    name: "Exterior lamp".into(),
                    is_unique_trophy: false,
                    total_num_stored: Some(2),
                    total_num_placed: Some(3),
                },
            );
            housing.exterior.selected_size = Some(3);
            housing.exterior.selected_type_id = Some(101);
            housing.exterior.size_options = [3, 4]
                .into_iter()
                .map(|size| ExteriorSizeOption {
                    size,
                    name: format!("Size {size}"),
                    is_locked: false,
                })
                .collect();
            housing.exterior.type_options = [101, 102]
                .into_iter()
                .map(|id| ExteriorTypeOption {
                    id,
                    name: format!("Type {id}"),
                    is_locked: false,
                    is_invalid: false,
                    reason: "".into(),
                })
                .collect();
            housing.exterior.selected_fixture_point = Some(ExteriorFixturePoint {
                owner_hash: 11,
                selected_fixture_id: Some(301),
                can_remove: true,
                options: [301, 302]
                    .into_iter()
                    .map(|id| ExteriorFixtureOption {
                        id,
                        name: format!("Fixture {id}"),
                        type_id: 9,
                        type_name: "Window".into(),
                        is_locked: false,
                        is_invalid: false,
                        reason: "".into(),
                        color_id: 5,
                    })
                    .collect(),
            });
            for (id, parent, position) in placements() {
                housing.exterior.decor.insert(
                    id.into(),
                    ExteriorDecorPlacement {
                        variant_id: variant(),
                        position,
                        fixture_point_owner_hash: parent,
                    },
                );
            }
        }
        env.exec(r#"
            events = {}
            listener = CreateFrame('Frame')
            for _, event in ipairs({'HOUSING_STORAGE_ENTRY_UPDATED', 'HOUSING_SET_FIXTURE_RESPONSE',
                'HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE', 'HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE'}) do
                listener:RegisterEvent(event)
            end
            listener:SetScript('OnEvent', function(_, event, ...)
                events[#events + 1] = {name = event, arity = select('#', ...), payload = ...}
            end)
        "#).unwrap();
        env
    }

    fn placements() -> [(&'static str, Option<u32>, [f64; 3]); 4] {
        [
            ("point11a", Some(11), [1.0, 2.0, 3.0]),
            ("point11b", Some(11), [4.0, 5.0, 6.0]),
            ("point22", Some(22), [-1.0, -2.0, 7.0]),
            ("floating", None, [8.0, 9.0, 10.0]),
        ]
    }

    fn invoke(env: &WowLuaEnv, method: usize, action: &str) {
        let (name, target, _) = CALLS[method];
        env.exec(&format!(
            "assert(select('#', C_HouseExterior.{name}({target}{action})) == 0)"
        ))
        .unwrap();
    }

    fn assert_effect(env: &WowLuaEnv, method: usize, store: bool) {
        let state = env.state().borrow();
        let housing = &state.housing;
        let exterior = &housing.exterior;
        assert_eq!(
            exterior.selected_size,
            Some(if method == 1 { 4 } else { 3 })
        );
        assert_eq!(
            exterior.selected_type_id,
            Some(if method == 2 { 102 } else { 101 })
        );
        let point = exterior.selected_fixture_point.as_ref().unwrap();
        assert_eq!(
            point.selected_fixture_id,
            Some(if method == 0 { 302 } else { 301 })
        );
        assert_eq!(point.owner_hash, 11);
        let count = if store {
            if method == 0 { 2 } else { 3 }
        } else {
            0
        };
        assert_eq!(exterior.decor.len(), 4 - count);
        for (id, parent, position) in placements() {
            let affected = parent.is_some() && (method != 0 || parent == Some(11));
            if affected && store {
                assert!(!exterior.decor.contains_key(id));
            } else {
                let decor = &exterior.decor[id];
                assert_eq!(decor.variant_id, variant());
                assert_eq!(decor.position, position);
                assert_eq!(
                    decor.fixture_point_owner_hash,
                    if affected { None } else { parent }
                );
            }
        }
        let record = &housing.catalog.variants[&variant()];
        assert_eq!(record.num_stored, 2 + count as i32);
        assert_eq!(record.destroyable_instance_count, 1);
        assert_eq!(record.dye_slots.len(), 1);
        let dye = &record.dye_slots[0];
        assert_eq!(
            (
                dye.id,
                dye.dye_color_category_id,
                dye.order_index,
                dye.channel
            ),
            (7, 8, 1, 0)
        );
        assert_eq!(dye.dye_color_id, Some(701));
        assert_eq!(dye.dye_color_name.as_deref(), Some("Blue"));
        let base = &housing.catalog.entries[&entry()];
        assert_eq!(base.total_num_stored, Some(2 + count as u32));
        assert_eq!(base.total_num_placed, Some(3 - count as u32));
        assert_eq!(base.item_id, Some(7101));
        assert_eq!(base.name, "Exterior lamp");
        assert!(!base.is_unique_trophy);
        assert_eq!(
            (
                housing.catalog.entries.len(),
                housing.catalog.variants.len()
            ),
            (1, 1)
        );
    }

    fn assert_unchanged(env: &WowLuaEnv, before: &HouseExteriorState) {
        let state = env.state().borrow();
        assert_eq!(&state.housing.exterior, before);
        assert_eq!(state.housing.catalog.variants[&variant()].num_stored, 2);
        assert_eq!(
            state.housing.catalog.entries[&entry()].total_num_stored,
            Some(2)
        );
        assert_eq!(
            state.housing.catalog.entries[&entry()].total_num_placed,
            Some(3)
        );
        drop(state);
    }

    fn reject(env: &WowLuaEnv, method: usize, arguments: &str) {
        let name = CALLS[method].0;
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(&format!(
            r#"
            local ok, message = pcall(C_HouseExterior.{name}, {arguments})
            assert(not ok and type(message) == 'string' and #message > 0)
            assert(string.find(message, '{name}', 1, true), 'error must name API')
        "#
        ))
        .unwrap();
        assert_unchanged(env, &before);
        env.exec("assert(#events == 0)").unwrap();
    }

    // Native error versus failure-response mapping is unproved; require observable rejection.
    fn reject_model(env: &WowLuaEnv, method: usize, target: i32) {
        let (name, _, response) = CALLS[method];
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(&format!(r#"
            events = {{}}
            local ok, message = pcall(C_HouseExterior.{name}, {target}, 0)
            if ok then
                assert(#events == 1 and events[1].name == '{response}')
                assert(events[1].arity == 1 and type(events[1].payload) == 'number' and events[1].payload ~= 0)
            else
                assert(type(message) == 'string' and #message > 0 and #events == 0)
            end
        "#)).unwrap();
        assert_unchanged(env, &before);
    }

    fn install_secrets(env: &WowLuaEnv, method: usize) {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        for (name, number) in [
            ("SecretTarget", f64::from(CALLS[method].1)),
            ("SecretAction", 1.0),
        ] {
            let secret = wrap_host_secret_number(lua.state_mut(), number);
            lua.state_mut().push(secret);
            let result = lua.set_global_val(name, secret);
            lua.state_mut().pop();
            result.unwrap();
        }
    }

    // B73 inputs: all removal policy beyond declarations is inferred, not native proof.
    fn remove_fixture(env: &WowLuaEnv, action: &str) {
        env.exec(&format!(
            "assert(select('#', C_HouseExterior.RemoveFixtureFromSelectedPoint({action})) == 0)"
        ))
        .unwrap();
        assert!(env.state().borrow().events.pending().iter().all(|event| {
            event.name != "HOUSING_STORAGE_ENTRY_UPDATED"
                && event.name != "HOUSING_SET_FIXTURE_RESPONSE"
        }));
    }

    fn assert_removal_state(env: &WowLuaEnv, expected: &wow_ui_sim::lua_api::state::HousingState) {
        let state = env.state().borrow();
        let housing = &state.housing;
        assert_eq!(housing.exterior, expected.exterior);
        assert_eq!(housing.pending_new_decor, expected.pending_new_decor);
        assert_eq!(housing.inside_owned_plot, expected.inside_owned_plot);
        assert_eq!(
            housing.active_house_editor_mode,
            expected.active_house_editor_mode
        );
        assert_eq!(
            housing.catalog.variants.len(),
            expected.catalog.variants.len()
        );
        assert_eq!(
            housing.catalog.entries.len(),
            expected.catalog.entries.len()
        );
        for (id, record) in &expected.catalog.variants {
            // Compare every record field, including dye metadata and destroyability.
            assert_eq!(
                format!("{:?}", housing.catalog.variants[id]),
                format!("{record:?}")
            );
        }
        for (id, record) in &expected.catalog.entries {
            assert_eq!(
                format!("{:?}", housing.catalog.entries[id]),
                format!("{record:?}")
            );
        }
    }

    fn assert_removed_fixture(
        env: &WowLuaEnv,
        before: &wow_ui_sim::lua_api::state::HousingState,
        store: bool,
    ) {
        let mut expected = before.clone();
        expected
            .exterior
            .selected_fixture_point
            .as_mut()
            .unwrap()
            .selected_fixture_id = None;
        for (id, owner, _) in placements() {
            if owner != Some(11) {
                continue;
            }
            if store {
                expected.exterior.decor.remove(id);
            } else {
                expected
                    .exterior
                    .decor
                    .get_mut(id)
                    .unwrap()
                    .fixture_point_owner_hash = None;
            }
        }
        if store {
            expected
                .catalog
                .variants
                .get_mut(&variant())
                .unwrap()
                .num_stored += 2;
            let base = expected.catalog.entries.get_mut(&entry()).unwrap();
            base.total_num_stored = base.total_num_stored.map(|count| count + 2);
            base.total_num_placed = base.total_num_placed.map(|count| count - 2);
        }
        assert_removal_state(env, &expected);
        env.exec(
            r#"
            local point = C_HouseExterior.GetSelectedFixturePointInfo()
            assert(point.selectedFixtureID == nil and point.ownerHash == 11)
            assert(point.canSelectionBeRemoved == false)
            assert(#point.fixtureOptions == 2 and point.fixtureOptions[2].fixtureID == 302)
            assert(C_HouseExterior.HasSelectedFixturePoint())
            assert(not C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint())
            assert(C_HouseExterior.IsAnyDecorAttachedToHouseExterior())
            assert(C_HouseExterior.GetCurrentHouseExteriorSize() == 3)
            assert(C_HouseExterior.GetCurrentHouseExteriorType() == 101)
        "#,
        )
        .unwrap();
    }

    fn reject_remove_fixture(env: &WowLuaEnv, action: &str) {
        let before = env.state().borrow().housing.clone();
        let pending = env.state().borrow().events.pending().len();
        env.exec(&format!(
            r#"
            local ok, message = pcall(C_HouseExterior.RemoveFixtureFromSelectedPoint, {action})
            assert(not ok and type(message) == 'string' and #message > 0)
            assert(string.find(message, 'RemoveFixtureFromSelectedPoint', 1, true))
            assert(#events == 0)
        "#
        ))
        .unwrap();
        assert_removal_state(env, &before);
        assert_eq!(env.state().borrow().events.pending().len(), pending);
    }

    fn install_remove_secret(env: &WowLuaEnv, number: f64) {
        let loader = env.loader();
        let mut lua = loader.rilua_mut();
        let secret = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(secret);
        let result = lua.set_global_val("RemoveSecret", secret);
        lua.state_mut().pop();
        result.unwrap();
    }

    #[test]
    fn remove_store_clears_only_fixture_and_returns_selected_owner_attachments() {
        let env = fixture_env();
        let before = env.state().borrow().housing.clone();
        remove_fixture(&env, "0");
        assert_removed_fixture(&env, &before, true);
    }

    #[test]
    fn remove_detach_preserves_full_identity_position_and_inventory() {
        let env = fixture_env();
        let before = env.state().borrow().housing.clone();
        remove_fixture(&env, "1");
        assert_removed_fixture(&env, &before, false);
        env.exec("assert(#events == 1 and events[1].name == 'HOUSING_SET_FIXTURE_RESPONSE' and events[1].arity == 1 and events[1].payload == 0)").unwrap();
    }

    #[test]
    fn remove_omitted_action_defaults_to_store() {
        let env = fixture_env();
        let before = env.state().borrow().housing.clone();
        remove_fixture(&env, "");
        assert_removed_fixture(&env, &before, true);
    }

    #[test]
    fn remove_nil_action_is_inferred_store() {
        let env = fixture_env();
        let before = env.state().borrow().housing.clone();
        remove_fixture(&env, "nil");
        assert_removed_fixture(&env, &before, true);
    }

    #[test]
    fn remove_secure_actual_secret_store_and_detach_are_accepted() {
        for action in [0, 1] {
            let env = fixture_env();
            install_remove_secret(&env, f64::from(action));
            let before = env.state().borrow().housing.clone();
            env.exec("collectgarbage('collect'); assert(issecure())")
                .unwrap();
            remove_fixture(&env, "RemoveSecret");
            assert_removed_fixture(&env, &before, action == 0);
        }
    }

    #[test]
    fn remove_public_addon_actions_retain_taint() {
        for action in [0, 1] {
            let env = fixture_env();
            let before = env.state().borrow().housing.clone();
            env.exec(&format!(r#"
                local function addon()
                    assert(select('#', C_HouseExterior.RemoveFixtureFromSelectedPoint({action})) == 0)
                    assert(not issecure())
                end
                debug.setobjecttaint(addon, 'RemoveInputs')
                addon()
            "#)).unwrap();
            assert_removed_fixture(&env, &before, action == 0);
        }
    }

    #[test]
    fn remove_addon_secret_authentication_precedes_domain_and_model() {
        for secret in [0.0, 1.0, 872364.0, f64::NAN] {
            let env = fixture_env();
            install_remove_secret(&env, secret);
            let before = env.state().borrow().housing.clone();
            env.exec(r#"
                local function addon()
                    local ok, message = pcall(C_HouseExterior.RemoveFixtureFromSelectedPoint, RemoveSecret)
                    assert(not ok and type(message) == 'string')
                    assert(string.find(string.lower(message), 'secret', 1, true))
                    assert(not string.find(message, '872364', 1, true))
                    assert(not issecure())
                    denial = message
                end
                debug.setobjecttaint(addon, 'RemoveInputs')
                removeAddon = addon
                addon()
                assert(#events == 0)
            "#).unwrap();
            assert_removal_state(&env, &before);
            env.state()
                .borrow_mut()
                .housing
                .exterior
                .selected_fixture_point = None;
            let absent = env.state().borrow().housing.clone();
            env.exec(
                "local original=denial; removeAddon(); assert(denial==original and #events==0)",
            )
            .unwrap();
            assert_removal_state(&env, &absent);
        }
    }

    #[test]
    fn remove_actual_secret_wrong_types_authenticate_before_type_validation() {
        for text_secret in [false, true] {
            let env = fixture_env();
            {
                let loader = env.loader();
                let mut lua = loader.rilua_mut();
                let secret = if text_secret {
                    wrap_host_secret_string(lua.state_mut(), "private-remove-action-78423")
                } else {
                    wrap_host_secret_bool(lua.state_mut(), false)
                };
                lua.state_mut().push(secret);
                let result = lua.set_global_val("RemoveSecret", secret);
                lua.state_mut().pop();
                result.unwrap();
            }
            let before = env.state().borrow().housing.clone();
            env.exec(r#"
                collectgarbage('collect')
                local ok, message = pcall(C_HouseExterior.RemoveFixtureFromSelectedPoint, RemoveSecret)
                assert(not ok and type(message) == 'string')
                assert(not string.find(message, 'private-remove-action-78423', 1, true))
                assert(not string.find(string.lower(message), 'secret access denied', 1, true))
                local function addon()
                    local accepted, denial = pcall(C_HouseExterior.RemoveFixtureFromSelectedPoint, RemoveSecret)
                    assert(not accepted and type(denial) == 'string')
                    assert(string.find(string.lower(denial), 'secret', 1, true))
                    assert(not string.find(denial, 'private-remove-action-78423', 1, true))
                    assert(not issecure())
                    secretTypeDenial = denial
                end
                debug.setobjecttaint(addon, 'RemoveInputs')
                removeTypeAddon = addon
                addon()
                assert(#events == 0)
            "#).unwrap();
            assert_removal_state(&env, &before);
            env.state()
                .borrow_mut()
                .housing
                .exterior
                .selected_fixture_point = None;
            let absent = env.state().borrow().housing.clone();
            env.exec("local original=secretTypeDenial; removeTypeAddon(); assert(secretTypeDenial==original and #events==0)").unwrap();
            assert_removal_state(&env, &absent);
        }
    }

    #[test]
    fn remove_wrong_types_and_malformed_enum_reject_atomically() {
        let env = fixture_env();
        for action in [
            "false",
            "true",
            "'0'",
            "'1'",
            "{}",
            "function() end",
            "-1",
            "2",
            "0.5",
            "0/0",
            "math.huge",
            "-math.huge",
            "4294967296",
        ] {
            reject_remove_fixture(&env, action);
        }
        install_remove_secret(&env, 872364.0);
        reject_remove_fixture(&env, "RemoveSecret");
    }

    #[test]
    fn remove_missing_fixture_point_and_ineligible_host_fail_without_events() {
        for failure in 0..5 {
            for action in ["0", "1"] {
                let env = fixture_env();
                {
                    let mut state = env.state().borrow_mut();
                    let housing = &mut state.housing;
                    match failure {
                        0 => housing.exterior.selected_fixture_point = None,
                        1 => {
                            housing
                                .exterior
                                .selected_fixture_point
                                .as_mut()
                                .unwrap()
                                .selected_fixture_id = None
                        }
                        2 => {
                            housing
                                .exterior
                                .selected_fixture_point
                                .as_mut()
                                .unwrap()
                                .can_remove = false
                        }
                        3 => housing.inside_owned_plot = false,
                        _ => housing.active_house_editor_mode = 0,
                    }
                }
                reject_remove_fixture(&env, action);
            }
        }
    }

    #[test]
    fn remove_no_attachments_clears_fixture_with_one_success_response() {
        for action in ["0", "1"] {
            let env = fixture_env();
            env.state()
                .borrow_mut()
                .housing
                .exterior
                .decor
                .retain(|_, decor| decor.fixture_point_owner_hash != Some(11));
            let mut expected = env.state().borrow().housing.clone();
            expected
                .exterior
                .selected_fixture_point
                .as_mut()
                .unwrap()
                .selected_fixture_id = None;
            remove_fixture(&env, action);
            assert_removal_state(&env, &expected);
            env.exec(
                r#"
                assert(#events == 1 and events[1].name == 'HOUSING_SET_FIXTURE_RESPONSE')
                assert(events[1].arity == 1 and events[1].payload == 0)
                assert(C_HouseExterior.HasSelectedFixturePoint())
                assert(C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID == nil)
                assert(not C_HouseExterior.GetSelectedFixturePointInfo().canSelectionBeRemoved)
            "#,
            )
            .unwrap();
        }
    }

    #[test]
    fn remove_detach_needs_no_catalog_and_never_reattaches_after_selection() {
        let env = fixture_env();
        env.state().borrow_mut().housing.catalog.variants.clear();
        env.state().borrow_mut().housing.catalog.entries.clear();
        let before = env.state().borrow().housing.clone();
        remove_fixture(&env, "1");
        assert_removed_fixture(&env, &before, false);
        invoke(&env, 0, ", 1");
        let mut expected = before;
        expected
            .exterior
            .selected_fixture_point
            .as_mut()
            .unwrap()
            .selected_fixture_id = Some(302);
        for id in ["point11a", "point11b"] {
            expected
                .exterior
                .decor
                .get_mut(id)
                .unwrap()
                .fixture_point_owner_hash = None;
        }
        assert_removal_state(&env, &expected);
        env.exec("assert(not C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint())")
            .unwrap();
    }

    #[test]
    fn remove_live_eligibility_restores_on_selection_and_repeat_remove_errors() {
        let env = fixture_env();
        remove_fixture(&env, "1");
        env.exec("assert(not C_HouseExterior.GetSelectedFixturePointInfo().canSelectionBeRemoved); events={}").unwrap();
        reject_remove_fixture(&env, "0");
        assert!(
            env.state()
                .borrow()
                .housing
                .exterior
                .selected_fixture_point
                .as_ref()
                .unwrap()
                .can_remove
        );
        invoke(&env, 0, ", 1");
        env.exec("assert(C_HouseExterior.GetSelectedFixturePointInfo().canSelectionBeRemoved)")
            .unwrap();
        remove_fixture(&env, "0");
        env.exec("assert(not C_HouseExterior.GetSelectedFixturePointInfo().canSelectionBeRemoved)")
            .unwrap();
    }

    #[test]
    fn remove_store_unknown_aggregate_totals_remain_unknown() {
        let env = fixture_env();
        {
            let mut state = env.state().borrow_mut();
            let base = state.housing.catalog.entries.get_mut(&entry()).unwrap();
            base.total_num_stored = None;
            base.total_num_placed = None;
        }
        let before = env.state().borrow().housing.clone();
        remove_fixture(&env, "0");
        assert_removed_fixture(&env, &before, true);
    }

    #[test]
    fn remove_store_mixed_valid_invalid_variants_and_aggregates_are_atomic() {
        for failure in 0..5 {
            let env = fixture_env();
            let second = HousingCatalogEntryVariantID {
                record_id: 7102,
                entry_type: 1,
                variant_identifier: 9,
            };
            let second_entry = HousingCatalogEntryID {
                record_id: 7102,
                entry_type: 1,
            };
            {
                let mut state = env.state().borrow_mut();
                let housing = &mut state.housing;
                let mut record = housing.catalog.variants[&variant()].clone();
                record.num_stored = 6;
                housing.catalog.variants.insert(second, record);
                let mut base = housing.catalog.entries[&entry()].clone();
                base.total_num_stored = Some(6);
                base.total_num_placed = Some(1);
                housing.catalog.entries.insert(second_entry, base);
                housing
                    .exterior
                    .decor
                    .get_mut("point11b")
                    .unwrap()
                    .variant_id = second;
                match failure {
                    0 => {
                        housing.catalog.variants.remove(&second);
                    }
                    1 => {
                        housing
                            .catalog
                            .variants
                            .get_mut(&second)
                            .unwrap()
                            .num_stored = -1
                    }
                    2 => {
                        housing
                            .catalog
                            .variants
                            .get_mut(&second)
                            .unwrap()
                            .num_stored = i32::MAX
                    }
                    3 => {
                        housing
                            .catalog
                            .entries
                            .get_mut(&second_entry)
                            .unwrap()
                            .total_num_stored = Some(u32::MAX)
                    }
                    _ => {
                        housing
                            .catalog
                            .entries
                            .get_mut(&second_entry)
                            .unwrap()
                            .total_num_placed = Some(0)
                    }
                }
            }
            reject_remove_fixture(&env, "0");
        }
    }

    #[test]
    fn remove_store_combined_known_aggregate_limits_reject_before_any_commit() {
        for (stored, placed) in [(u32::MAX - 1, 3), (2, 1)] {
            let env = fixture_env();
            {
                let mut state = env.state().borrow_mut();
                let base = state.housing.catalog.entries.get_mut(&entry()).unwrap();
                base.total_num_stored = Some(stored);
                base.total_num_placed = Some(placed);
            }
            reject_remove_fixture(&env, "0");
        }
    }

    #[test]
    fn remove_store_full_variant_events_follow_complete_commit_and_survive_gc() {
        let env = fixture_env();
        let second = HousingCatalogEntryVariantID {
            variant_identifier: 1,
            ..variant()
        };
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            let mut record = housing.catalog.variants[&variant()].clone();
            record.num_stored = 6;
            housing.catalog.variants.insert(second, record);
            housing
                .catalog
                .entries
                .get_mut(&entry())
                .unwrap()
                .total_num_stored = Some(8);
            housing
                .exterior
                .decor
                .get_mut("point11b")
                .unwrap()
                .variant_id = second;
        }
        let mut expected = env.state().borrow().housing.clone();
        expected
            .exterior
            .selected_fixture_point
            .as_mut()
            .unwrap()
            .selected_fixture_id = None;
        expected.exterior.decor.remove("point11a");
        expected.exterior.decor.remove("point11b");
        expected
            .catalog
            .variants
            .get_mut(&variant())
            .unwrap()
            .num_stored = 3;
        expected
            .catalog
            .variants
            .get_mut(&second)
            .unwrap()
            .num_stored = 7;
        expected
            .catalog
            .entries
            .get_mut(&entry())
            .unwrap()
            .total_num_stored = Some(10);
        expected
            .catalog
            .entries
            .get_mut(&entry())
            .unwrap()
            .total_num_placed = Some(1);
        env.exec(r#"
            listener:SetScript('OnEvent', function(_, event, ...)
                local payload = ...
                collectgarbage('collect')
                local point = C_HouseExterior.GetSelectedFixturePointInfo()
                local base = C_HousingCatalog.GetCatalogEntryInfo({recordID=7101,entryType=1})
                events[#events+1] = {name=event, payload=payload, arity=select('#', ...),
                    baseStored=base.totalNumStored, basePlaced=base.totalNumPlaced,
                    first=C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101,entryType=1,variantIdentifier=0}).numStored,
                    second=C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101,entryType=1,variantIdentifier=1}).numStored,
                    fixture=point.selectedFixtureID, removable=point.canSelectionBeRemoved,
                    hasPoint=C_HouseExterior.HasSelectedFixturePoint(),
                    selected=C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint()}
            end)
        "#).unwrap();
        remove_fixture(&env, "0");
        assert_removal_state(&env, &expected);
        env.exec(r#"
            collectgarbage('collect')
            assert(#events == 3 and events[3].name == 'HOUSING_SET_FIXTURE_RESPONSE' and events[3].payload == 0)
            local seen = {}
            for index,row in ipairs(events) do
                assert(row.arity == 1 and row.first == 3 and row.second == 7)
                assert(row.baseStored == 10 and row.basePlaced == 1)
                assert(row.fixture == nil and row.hasPoint and not row.removable and not row.selected)
                if index < 3 then
                    assert(row.name == 'HOUSING_STORAGE_ENTRY_UPDATED')
                    local id = row.payload
                    assert(id.recordID == 7101 and id.entryType == 1)
                    seen[id.variantIdentifier] = (seen[id.variantIdentifier] or 0) + 1
                    assert(C_HousingCatalog.GetCatalogEntryVariantInfo(id).numStored == (id.variantIdentifier == 0 and 3 or 7))
                end
            end
            assert(seen[0] == 1 and seen[1] == 1)
        "#).unwrap();
    }

    #[test]
    fn remove_storage_and_response_gc_reentry_keep_new_fixture_and_original_payload() {
        for reenter_on in [
            "HOUSING_STORAGE_ENTRY_UPDATED",
            "HOUSING_SET_FIXTURE_RESPONSE",
        ] {
            let env = fixture_env();
            let before = env.state().borrow().housing.clone();
            env.exec(&format!(r#"
                reentered = false
                listener:SetScript('OnEvent', function(_, event, ...)
                    local payload = ...
                    collectgarbage('collect')
                    local point = C_HouseExterior.GetSelectedFixturePointInfo()
                    events[#events+1] = {{name=event, payload=payload, arity=select('#', ...),
                        fixture=point.selectedFixtureID, removable=point.canSelectionBeRemoved,
                        stored=C_HousingCatalog.GetCatalogEntryVariantInfo({{recordID=7101,entryType=1,variantIdentifier=0}}).numStored}}
                    if event == '{reenter_on}' and not reentered then
                        reentered = true
                        originalPayload = payload
                        C_HouseExterior.SelectFixtureOption(302, 1)
                        collectgarbage('collect')
                        afterFixture = C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID
                        afterRemovable = C_HouseExterior.GetSelectedFixturePointInfo().canSelectionBeRemoved
                    end
                end)
            "#)).unwrap();
            remove_fixture(&env, "0");
            let mut expected = before;
            expected
                .exterior
                .selected_fixture_point
                .as_mut()
                .unwrap()
                .selected_fixture_id = Some(302);
            expected.exterior.decor.remove("point11a");
            expected.exterior.decor.remove("point11b");
            expected
                .catalog
                .variants
                .get_mut(&variant())
                .unwrap()
                .num_stored = 4;
            expected
                .catalog
                .entries
                .get_mut(&entry())
                .unwrap()
                .total_num_stored = Some(4);
            expected
                .catalog
                .entries
                .get_mut(&entry())
                .unwrap()
                .total_num_placed = Some(1);
            assert_removal_state(&env, &expected);
            env.exec(&format!(r#"
                collectgarbage('collect')
                assert(reentered and afterFixture == 302 and afterRemovable and #events == 3)
                assert(events[1].name == 'HOUSING_STORAGE_ENTRY_UPDATED')
                assert(events[1].fixture == nil and not events[1].removable)
                assert(events[1].payload.recordID == 7101 and events[1].payload.entryType == 1 and events[1].payload.variantIdentifier == 0)
                for _,row in ipairs(events) do assert(row.arity == 1 and row.stored == 4) end
                assert(events[2].name == 'HOUSING_SET_FIXTURE_RESPONSE' and events[2].payload == 0)
                assert(events[3].name == 'HOUSING_SET_FIXTURE_RESPONSE' and events[3].payload == 0)
                if '{reenter_on}' == 'HOUSING_STORAGE_ENTRY_UPDATED' then
                    assert(originalPayload == events[1].payload)
                    assert(C_HousingCatalog.GetCatalogEntryVariantInfo(originalPayload).numStored == 4)
                    assert(events[2].fixture == 302 and events[3].fixture == 302)
                else
                    assert(originalPayload == 0 and events[2].fixture == nil and events[3].fixture == 302)
                end
                assert(C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID == 302)
                assert(not C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint())
            "#)).unwrap();
        }
    }

    #[test]
    fn remove_preserves_pending_request_and_environment_isolation() {
        let first = fixture_env();
        let second = fixture_env();
        first.state().borrow_mut().housing.pending_new_decor = Some(variant());
        let before_first = first.state().borrow().housing.clone();
        let before_second = second.state().borrow().housing.clone();
        remove_fixture(&first, "0");
        assert_removed_fixture(&first, &before_first, true);
        assert_removal_state(&second, &before_second);
        second.exec("assert(#events == 0)").unwrap();
        remove_fixture(&second, "1");
        assert_removed_fixture(&second, &before_second, false);
        assert_removed_fixture(&first, &before_first, true);
    }

    #[test]
    fn fixture_store_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 0, ", 0");
        assert_effect(&env, 0, true);
    }

    #[test]
    fn fixture_detach_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 0, ", 1");
        assert_effect(&env, 0, false);
    }

    #[test]
    fn fixture_omitted_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 0, "");
        assert_effect(&env, 0, true);
    }

    #[test]
    fn size_store_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 1, ", 0");
        assert_effect(&env, 1, true);
    }

    #[test]
    fn size_detach_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 1, ", 1");
        assert_effect(&env, 1, false);
    }

    #[test]
    fn size_omitted_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 1, "");
        assert_effect(&env, 1, true);
    }

    #[test]
    fn type_store_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 2, ", 0");
        assert_effect(&env, 2, true);
    }

    #[test]
    fn type_detach_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 2, ", 1");
        assert_effect(&env, 2, false);
    }

    #[test]
    fn type_omitted_changes_only_affected_placements() {
        let env = fixture_env();
        invoke(&env, 2, "");
        assert_effect(&env, 2, true);
    }

    #[test]
    fn nil_action_is_inferred_store_for_each_mutator() {
        for method in 0..3 {
            let env = fixture_env();
            invoke(&env, method, ", nil");
            assert_effect(&env, method, true);
        }
    }

    #[test]
    fn fixture_secure_actual_secret_numbers_are_accepted() {
        let env = fixture_env();
        install_secrets(&env, 0);
        env.exec("collectgarbage('collect'); assert(issecure()); assert(select('#', C_HouseExterior.SelectFixtureOption(SecretTarget, SecretAction)) == 0)").unwrap();
        assert_effect(&env, 0, false);
    }

    #[test]
    fn fixture_public_addon_call_preserves_taint() {
        let env = fixture_env();
        env.exec(
            r#"
            local function addon()
                C_HouseExterior.SelectFixtureOption(302, 1)
                assert(not issecure())
            end
            debug.setobjecttaint(addon, 'ExteriorInputs')
            addon()
        "#,
        )
        .unwrap();
        assert_effect(&env, 0, false);
    }

    #[test]
    fn fixture_both_original_selectors_authenticate_before_types_or_model() {
        let env = fixture_env();
        install_secrets(&env, 0);
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(r#"
            local function addon()
                local api = C_HouseExterior.SelectFixtureOption
                for _, args in ipairs({ {SecretTarget, 1}, {302, SecretAction},
                    {false, SecretAction}, {999999, SecretAction}, {SecretTarget, false} }) do
                    local ok, message = pcall(api, unpack(args))
                    assert(not ok and type(message) == 'string')
                    assert(string.find(string.lower(message), 'secret', 1, true))
                    assert(not string.find(message, '302', 1, true), 'private target payload must not leak')
                    assert(not issecure())
                end
                local _, second = pcall(api, 302, SecretAction)
                local _, badType = pcall(api, false, SecretAction)
                local _, badModel = pcall(api, 999999, SecretAction)
                assert(second == badType and second == badModel, 'arg2 authentication precedes arg1 validation')
            end
            debug.setobjecttaint(addon, 'ExteriorInputs')
            addon()
            assert(#events == 0)
        "#).unwrap();
        assert_unchanged(&env, &before);
    }

    #[test]
    fn fixture_invalid_types_and_domains_reject_atomically() {
        let env = fixture_env();
        for args in [
            "nil, 0",
            "false, 0",
            "{}, 0",
            "'4', 0",
            "0, 0",
            "-1, 0",
            "1.5, 0",
            "0/0, 0",
            "math.huge, 0",
            "4294967296, 0",
            "302, false",
            "302, '1'",
            "302, {}",
            "302, -1",
            "302, 2",
            "302, 0.5",
            "302, 0/0",
            "302, math.huge",
        ] {
            reject(&env, 0, args);
        }
    }

    #[test]
    fn fixture_unknown_locked_or_invalid_option_has_no_inventory_effect() {
        let env = fixture_env();
        reject_model(&env, 0, 999999);
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            housing
                .exterior
                .selected_fixture_point
                .as_mut()
                .unwrap()
                .options[1]
                .is_locked = true;
        }
        reject_model(&env, 0, 302);
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            housing
                .exterior
                .selected_fixture_point
                .as_mut()
                .unwrap()
                .options[1]
                .is_locked = false;
            housing
                .exterior
                .selected_fixture_point
                .as_mut()
                .unwrap()
                .options[1]
                .is_invalid = true;
        }
        reject_model(&env, 0, 302);
    }

    #[test]
    fn fixture_synchronous_storage_then_response_observe_committed_state() {
        let env = fixture_env();
        env.exec(r#"
            listener:SetScript('OnEvent', function(_, event, ...)
                local payload = ...
                local stored = C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101, entryType=1, variantIdentifier=0}).numStored
                events[#events+1] = {name=event, payload=payload, arity=select('#', ...), stored=stored,
                    size=C_HouseExterior.GetCurrentHouseExteriorSize(),
                    exteriorType=C_HouseExterior.GetCurrentHouseExteriorType(),
                    fixture=C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID,
                    all=C_HouseExterior.IsAnyDecorAttachedToHouseExterior(),
                    selected=C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint()}
            end)
        "#).unwrap();
        invoke(&env, 0, ", 0");
        assert_effect(&env, 0, true);
        env.exec(r#"
            assert(#events == 2, 'one storage event per affected variant, then synchronous response')
            assert(events[1].name == 'HOUSING_STORAGE_ENTRY_UPDATED')
            local id = events[1].payload
            assert(id.recordID == 7101 and id.entryType == 1 and id.variantIdentifier == 0)
            assert(events[2].name == 'HOUSING_SET_FIXTURE_RESPONSE')
            assert(events[2].payload == 0)
            for _, row in ipairs(events) do
                assert(row.arity == 1 and row.stored == 4)
                assert(row.size == 3 and row.exteriorType == 101 and row.fixture == 302)
                assert(row.all == true and not row.selected)
            end
        "#).unwrap();
        assert!(
            env.state()
                .borrow()
                .events
                .pending()
                .iter()
                .all(|event| !event.name.starts_with("HOUSING_SET_")
                    && event.name != "HOUSING_STORAGE_ENTRY_UPDATED")
        );
    }

    #[test]
    fn size_secure_actual_secret_numbers_are_accepted() {
        let env = fixture_env();
        install_secrets(&env, 1);
        env.exec("collectgarbage('collect'); assert(issecure()); assert(select('#', C_HouseExterior.SetHouseExteriorSize(SecretTarget, SecretAction)) == 0)").unwrap();
        assert_effect(&env, 1, false);
    }

    #[test]
    fn size_public_addon_call_preserves_taint() {
        let env = fixture_env();
        env.exec(
            r#"
            local function addon()
                C_HouseExterior.SetHouseExteriorSize(4, 1)
                assert(not issecure())
            end
            debug.setobjecttaint(addon, 'ExteriorInputs')
            addon()
        "#,
        )
        .unwrap();
        assert_effect(&env, 1, false);
    }

    #[test]
    fn size_both_original_selectors_authenticate_before_types_or_model() {
        let env = fixture_env();
        install_secrets(&env, 1);
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(r#"
            local function addon()
                local api = C_HouseExterior.SetHouseExteriorSize
                for _, args in ipairs({ {SecretTarget, 1}, {4, SecretAction},
                    {false, SecretAction}, {999999, SecretAction}, {SecretTarget, false} }) do
                    local ok, message = pcall(api, unpack(args))
                    assert(not ok and type(message) == 'string')
                    assert(string.find(string.lower(message), 'secret', 1, true))
                    assert(not string.find(message, '4', 1, true), 'private target payload must not leak')
                    assert(not issecure())
                end
                local _, second = pcall(api, 4, SecretAction)
                local _, badType = pcall(api, false, SecretAction)
                local _, badModel = pcall(api, 999999, SecretAction)
                assert(second == badType and second == badModel, 'arg2 authentication precedes arg1 validation')
            end
            debug.setobjecttaint(addon, 'ExteriorInputs')
            addon()
            assert(#events == 0)
        "#).unwrap();
        assert_unchanged(&env, &before);
    }

    #[test]
    fn size_invalid_types_and_domains_reject_atomically() {
        let env = fixture_env();
        for args in [
            "nil, 0",
            "false, 0",
            "{}, 0",
            "'4', 0",
            "0, 0",
            "-1, 0",
            "1.5, 0",
            "0/0, 0",
            "math.huge, 0",
            "4294967296, 0",
            "4, false",
            "4, '1'",
            "4, {}",
            "4, -1",
            "4, 2",
            "4, 0.5",
            "4, 0/0",
            "4, math.huge",
        ] {
            reject(&env, 1, args);
        }
    }

    #[test]
    fn size_unknown_locked_or_invalid_option_has_no_inventory_effect() {
        let env = fixture_env();
        reject_model(&env, 1, 999999);
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            housing.exterior.size_options[1].is_locked = true;
        }
        reject_model(&env, 1, 4);
    }

    #[test]
    fn size_synchronous_storage_then_response_observe_committed_state() {
        let env = fixture_env();
        env.exec(r#"
            listener:SetScript('OnEvent', function(_, event, ...)
                local payload = ...
                local stored = C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101, entryType=1, variantIdentifier=0}).numStored
                events[#events+1] = {name=event, payload=payload, arity=select('#', ...), stored=stored,
                    size=C_HouseExterior.GetCurrentHouseExteriorSize(),
                    exteriorType=C_HouseExterior.GetCurrentHouseExteriorType(),
                    fixture=C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID,
                    all=C_HouseExterior.IsAnyDecorAttachedToHouseExterior(),
                    selected=C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint()}
            end)
        "#).unwrap();
        invoke(&env, 1, ", 0");
        assert_effect(&env, 1, true);
        env.exec(r#"
            assert(#events == 2, 'one storage event per affected variant, then synchronous response')
            assert(events[1].name == 'HOUSING_STORAGE_ENTRY_UPDATED')
            local id = events[1].payload
            assert(id.recordID == 7101 and id.entryType == 1 and id.variantIdentifier == 0)
            assert(events[2].name == 'HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE')
            assert(events[2].payload == 0)
            for _, row in ipairs(events) do
                assert(row.arity == 1 and row.stored == 5)
                assert(row.size == 4 and row.exteriorType == 101 and row.fixture == 301)
                assert(row.all == false and not row.selected)
            end
        "#).unwrap();
        assert!(
            env.state()
                .borrow()
                .events
                .pending()
                .iter()
                .all(|event| !event.name.starts_with("HOUSING_SET_")
                    && event.name != "HOUSING_STORAGE_ENTRY_UPDATED")
        );
    }

    #[test]
    fn type_secure_actual_secret_numbers_are_accepted() {
        let env = fixture_env();
        install_secrets(&env, 2);
        env.exec("collectgarbage('collect'); assert(issecure()); assert(select('#', C_HouseExterior.SetHouseExteriorType(SecretTarget, SecretAction)) == 0)").unwrap();
        assert_effect(&env, 2, false);
    }

    #[test]
    fn type_public_addon_call_preserves_taint() {
        let env = fixture_env();
        env.exec(
            r#"
            local function addon()
                C_HouseExterior.SetHouseExteriorType(102, 1)
                assert(not issecure())
            end
            debug.setobjecttaint(addon, 'ExteriorInputs')
            addon()
        "#,
        )
        .unwrap();
        assert_effect(&env, 2, false);
    }

    #[test]
    fn type_both_original_selectors_authenticate_before_types_or_model() {
        let env = fixture_env();
        install_secrets(&env, 2);
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(r#"
            local function addon()
                local api = C_HouseExterior.SetHouseExteriorType
                for _, args in ipairs({ {SecretTarget, 1}, {102, SecretAction},
                    {false, SecretAction}, {999999, SecretAction}, {SecretTarget, false} }) do
                    local ok, message = pcall(api, unpack(args))
                    assert(not ok and type(message) == 'string')
                    assert(string.find(string.lower(message), 'secret', 1, true))
                    assert(not string.find(message, '102', 1, true), 'private target payload must not leak')
                    assert(not issecure())
                end
                local _, second = pcall(api, 102, SecretAction)
                local _, badType = pcall(api, false, SecretAction)
                local _, badModel = pcall(api, 999999, SecretAction)
                assert(second == badType and second == badModel, 'arg2 authentication precedes arg1 validation')
            end
            debug.setobjecttaint(addon, 'ExteriorInputs')
            addon()
            assert(#events == 0)
        "#).unwrap();
        assert_unchanged(&env, &before);
    }

    #[test]
    fn type_invalid_types_and_domains_reject_atomically() {
        let env = fixture_env();
        for args in [
            "nil, 0",
            "false, 0",
            "{}, 0",
            "'4', 0",
            "0, 0",
            "-1, 0",
            "1.5, 0",
            "0/0, 0",
            "math.huge, 0",
            "4294967296, 0",
            "102, false",
            "102, '1'",
            "102, {}",
            "102, -1",
            "102, 2",
            "102, 0.5",
            "102, 0/0",
            "102, math.huge",
        ] {
            reject(&env, 2, args);
        }
    }

    #[test]
    fn type_unknown_locked_or_invalid_option_has_no_inventory_effect() {
        let env = fixture_env();
        reject_model(&env, 2, 999999);
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            housing.exterior.type_options[1].is_locked = true;
        }
        reject_model(&env, 2, 102);
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            housing.exterior.type_options[1].is_locked = false;
            housing.exterior.type_options[1].is_invalid = true;
        }
        reject_model(&env, 2, 102);
    }

    #[test]
    fn type_synchronous_storage_then_response_observe_committed_state() {
        let env = fixture_env();
        env.exec(r#"
            listener:SetScript('OnEvent', function(_, event, ...)
                local payload = ...
                local stored = C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101, entryType=1, variantIdentifier=0}).numStored
                events[#events+1] = {name=event, payload=payload, arity=select('#', ...), stored=stored,
                    size=C_HouseExterior.GetCurrentHouseExteriorSize(),
                    exteriorType=C_HouseExterior.GetCurrentHouseExteriorType(),
                    fixture=C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID,
                    all=C_HouseExterior.IsAnyDecorAttachedToHouseExterior(),
                    selected=C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint()}
            end)
        "#).unwrap();
        invoke(&env, 2, ", 0");
        assert_effect(&env, 2, true);
        env.exec(r#"
            assert(#events == 2, 'one storage event per affected variant, then synchronous response')
            assert(events[1].name == 'HOUSING_STORAGE_ENTRY_UPDATED')
            local id = events[1].payload
            assert(id.recordID == 7101 and id.entryType == 1 and id.variantIdentifier == 0)
            assert(events[2].name == 'HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE')
            assert(events[2].payload == 0)
            for _, row in ipairs(events) do
                assert(row.arity == 1 and row.stored == 5)
                assert(row.size == 3 and row.exteriorType == 102 and row.fixture == 301)
                assert(row.all == false and not row.selected)
            end
        "#).unwrap();
        assert!(
            env.state()
                .borrow()
                .events
                .pending()
                .iter()
                .all(|event| !event.name.starts_with("HOUSING_SET_")
                    && event.name != "HOUSING_STORAGE_ENTRY_UPDATED")
        );
    }

    #[test]
    fn decor_action_enum_matches_primary_store_and_detach_values() {
        let env = WowLuaEnv::new().unwrap();
        env.exec("assert(Enum.HousingFixtureDecorAction.Store == 0 and Enum.HousingFixtureDecorAction.Detach == 1)").unwrap();
    }

    #[test]
    fn store_emits_once_per_affected_full_variant_after_all_counts_commit() {
        let env = fixture_env();
        let second = HousingCatalogEntryVariantID {
            variant_identifier: 1,
            ..variant()
        };
        {
            let mut state = env.state().borrow_mut();
            let housing = &mut state.housing;
            let mut record = housing.catalog.variants[&variant()].clone();
            record.num_stored = 6;
            housing.catalog.variants.insert(second, record);
            housing
                .catalog
                .entries
                .get_mut(&entry())
                .unwrap()
                .total_num_stored = Some(8);
            housing
                .exterior
                .decor
                .get_mut("point11b")
                .unwrap()
                .variant_id = second;
        }
        env.exec(r#"
            listener:SetScript('OnEvent', function(_, event, ...)
                local id = ...
                local first = C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101,entryType=1,variantIdentifier=0}).numStored
                local second = C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101,entryType=1,variantIdentifier=1}).numStored
                events[#events+1] = {name=event, payload=id, first=first, second=second, arity=select('#',...)}
            end)
        "#).unwrap();
        invoke(&env, 0, ", 0");
        env.exec(r#"
            assert(#events==3 and events[3].name=='HOUSING_SET_FIXTURE_RESPONSE' and events[3].payload==0)
            local variants={}
            for index,row in ipairs(events) do
                assert(row.arity==1 and row.first==3 and row.second==7)
                if index<3 then
                    assert(row.name=='HOUSING_STORAGE_ENTRY_UPDATED')
                    assert(row.payload.recordID==7101 and row.payload.entryType==1)
                    variants[row.payload.variantIdentifier]=(variants[row.payload.variantIdentifier] or 0)+1
                end
            end
            assert(variants[0]==1 and variants[1]==1)
        "#).unwrap();
        let state = env.state().borrow();
        let housing = &state.housing;
        assert_eq!(housing.catalog.variants[&variant()].num_stored, 3);
        assert_eq!(housing.catalog.variants[&second].num_stored, 7);
        assert_eq!(
            housing.catalog.variants[&second].destroyable_instance_count,
            1
        );
        assert_eq!(
            housing.catalog.variants[&second].dye_slots[0].dye_color_id,
            Some(701)
        );
        assert_eq!(housing.catalog.entries[&entry()].total_num_stored, Some(10));
        assert_eq!(housing.catalog.entries[&entry()].total_num_placed, Some(1));
        assert_eq!(housing.exterior.decor.len(), 2);
        assert_eq!(
            housing.exterior.decor["point22"].fixture_point_owner_hash,
            Some(22)
        );
        assert_eq!(
            housing.exterior.decor["floating"].position,
            [8.0, 9.0, 10.0]
        );
    }

    #[test]
    fn current_selection_and_primary_dtos_are_live_read_only_snapshots() {
        let env = fixture_env();
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(r#"
            assert(C_HouseExterior.GetCurrentHouseExteriorSize() == 3)
            local id, name = C_HouseExterior.GetCurrentHouseExteriorType()
            assert(id == 101 and name == 'Type 101')
            local sizes = C_HouseExterior.GetHouseExteriorSizeOptions()
            assert(sizes.selectedSize == 3 and #sizes.options == 2)
            assert(sizes.options[1].size == 3 and sizes.options[1].name == 'Size 3' and sizes.options[1].isLocked == false)
            local types = C_HouseExterior.GetHouseExteriorTypeOptions()
            assert(types.selectedExteriorType == 101 and #types.options == 2)
            local option = types.options[2]
            assert(option.houseExteriorTypeID == 102 and option.name == 'Type 102')
            assert(option.isLocked == false and option.isInvalid == false and option.reasonString == '')
            local point = C_HouseExterior.GetSelectedFixturePointInfo()
            assert(point.ownerHash == 11 and point.selectedFixtureID == 301 and point.canSelectionBeRemoved == true)
            assert(#point.fixtureOptions == 2)
            local fixture = point.fixtureOptions[2]
            assert(fixture.fixtureID == 302 and fixture.name == 'Fixture 302')
            assert(fixture.typeID == 9 and fixture.typeName == 'Window' and fixture.colorID == 5)
            assert(fixture.isLocked == false and fixture.isInvalid == false and fixture.reasonString == '')
            sizes.options[1].size = 99; types.options[2].name = 'Changed'; point.ownerHash = 99
            point.fixtureOptions[2].fixtureID = 99
            collectgarbage('collect')
            assert(C_HouseExterior.GetHouseExteriorSizeOptions().options[1].size == 3)
            assert(C_HouseExterior.GetHouseExteriorTypeOptions().options[2].name == 'Type 102')
            assert(C_HouseExterior.GetSelectedFixturePointInfo().fixtureOptions[2].fixtureID == 302)
            assert(C_HouseExterior.HasSelectedFixturePoint())
            assert(C_HouseExterior.IsAnyDecorAttachedToHouseExterior())
            assert(C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint())
            assert(#events == 0)
        "#).unwrap();
        assert_unchanged(&env, &before);
        env.state().borrow_mut().housing.exterior.selected_type_id = Some(102);
        env.exec("local id,name=C_HouseExterior.GetCurrentHouseExteriorType(); assert(id==102 and name=='Type 102')").unwrap();
    }

    #[test]
    fn selected_point_and_availability_queries_use_explicit_state() {
        let env = fixture_env();
        env.state()
            .borrow_mut()
            .housing
            .exterior
            .selected_fixture_point = None;
        env.exec(
            r#"
            assert(not C_HouseExterior.HasSelectedFixturePoint())
            assert(C_HouseExterior.GetSelectedFixturePointInfo() == nil)
            assert(not C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint())
            assert(C_HouseExterior.IsAnyDecorAttachedToHouseExterior())
        "#,
        )
        .unwrap();
        env.state().borrow_mut().housing.inside_owned_plot = false;
        env.exec("assert(not C_HouseExterior.IsAnyDecorAttachedToHouseExterior())")
            .unwrap();
        {
            let mut state = env.state().borrow_mut();
            state.housing.inside_owned_plot = true;
            state.housing.active_house_editor_mode = 0;
        }
        env.exec("assert(not C_HouseExterior.IsAnyDecorAttachedToHouseExterior())")
            .unwrap();
    }

    #[test]
    fn default_has_no_exterior_records_and_no_fabricated_query_selection() {
        let env = WowLuaEnv::new().unwrap();
        assert_eq!(
            env.state().borrow().housing.exterior,
            HouseExteriorState::default()
        );
        env.exec(r#"
            assert(C_HouseExterior.GetCurrentHouseExteriorSize() == nil)
            local id,name=C_HouseExterior.GetCurrentHouseExteriorType(); assert(id==nil and name==nil)
            assert(C_HouseExterior.GetHouseExteriorSizeOptions() == nil)
            assert(C_HouseExterior.GetHouseExteriorTypeOptions() == nil)
            assert(C_HouseExterior.GetSelectedFixturePointInfo() == nil)
            assert(not C_HouseExterior.HasSelectedFixturePoint())
            assert(not C_HouseExterior.IsAnyDecorAttachedToHouseExterior())
            assert(not C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint())
        "#).unwrap();
        for (name, target, _) in CALLS {
            env.exec(&format!("pcall(C_HouseExterior.{name}, {target}, 0)"))
                .unwrap();
            let state = env.state().borrow();
            assert_eq!(state.housing.exterior, HouseExteriorState::default());
            assert!(state.housing.catalog.variants.is_empty());
            assert!(state.housing.catalog.entries.is_empty());
        }
    }

    #[test]
    fn same_size_type_and_fixture_never_touch_existing_attachments() {
        for (method, current, response_code) in [(0, 301, 0), (1, 3, 42), (2, 101, 43)] {
            for action in [0, 1] {
                let env = fixture_env();
                let before = env.state().borrow().housing.exterior.clone();
                let (name, _, response) = CALLS[method];
                env.exec(&format!(
                    "assert(select('#', C_HouseExterior.{name}({current}, {action})) == 0)"
                ))
                .unwrap();
                assert_unchanged(&env, &before);
                env.exec(&format!("assert(#events==1 and events[1].name=='{response}' and events[1].payload=={response_code} and events[1].arity==1)")).unwrap();
            }
        }
    }

    #[test]
    fn response_listener_can_query_and_reenter_without_outer_overwrite() {
        let env = fixture_env();
        env.exec(r#"
            nested = false
            listener:SetScript('OnEvent', function(_, event, result)
                if event == 'HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE' then
                    assert(result == 0 and C_HouseExterior.GetCurrentHouseExteriorType() == 102)
                    C_HouseExterior.SetHouseExteriorSize(4, 0)
                    nested = C_HouseExterior.GetCurrentHouseExteriorSize() == 4
                elseif event == 'HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE' then
                    events[#events+1] = {result=result, attached=C_HouseExterior.IsAnyDecorAttachedToHouseExterior(),
                        stored=C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101,entryType=1,variantIdentifier=0}).numStored}
                end
            end)
            C_HouseExterior.SetHouseExteriorType(102, 1)
            assert(nested and #events == 1)
            assert(events[1].result == 0 and not events[1].attached and events[1].stored == 2)
        "#).unwrap();
        let state = env.state().borrow();
        assert_eq!(state.housing.exterior.selected_size, Some(4));
        assert_eq!(state.housing.exterior.selected_type_id, Some(102));
        for (id, _, position) in placements() {
            let decor = &state.housing.exterior.decor[id];
            assert_eq!(decor.position, position);
            assert_eq!(decor.fixture_point_owner_hash, None);
        }
        assert_eq!(state.housing.catalog.variants[&variant()].num_stored, 2);
    }

    #[test]
    fn storage_listener_gc_and_reentry_preserve_committed_state_and_original_payload() {
        let env = fixture_env();
        // Storage-before-response is inferred; the outer Store is complete before reentry.
        env.exec(r#"
            reentered = false
            listener:SetScript('OnEvent', function(_, event, ...)
                local payload = ...
                collectgarbage('collect')
                local row = {name=event, payload=payload, arity=select('#', ...),
                    stored=C_HousingCatalog.GetCatalogEntryVariantInfo({recordID=7101,entryType=1,variantIdentifier=0}).numStored,
                    size=C_HouseExterior.GetCurrentHouseExteriorSize(),
                    exteriorType=C_HouseExterior.GetCurrentHouseExteriorType(),
                    fixture=C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID,
                    all=C_HouseExterior.IsAnyDecorAttachedToHouseExterior(),
                    selected=C_HouseExterior.IsAnyDecorAttachedToSelectedFixturePoint()}
                events[#events+1] = row
                if event == 'HOUSING_STORAGE_ENTRY_UPDATED' and not reentered then
                    reentered = true
                    originalPayload = payload
                    C_HouseExterior.SetHouseExteriorSize(4, 0)
                    collectgarbage('collect')
                    afterReentry = {stored=C_HousingCatalog.GetCatalogEntryVariantInfo(originalPayload).numStored,
                        size=C_HouseExterior.GetCurrentHouseExteriorSize(),
                        fixture=C_HouseExterior.GetSelectedFixturePointInfo().selectedFixtureID,
                        all=C_HouseExterior.IsAnyDecorAttachedToHouseExterior()}
                end
            end)
        "#).unwrap();
        invoke(&env, 0, ", 0");
        env.exec(r#"
            collectgarbage('collect')
            assert(reentered and #events == 4)
            assert(events[1].name == 'HOUSING_STORAGE_ENTRY_UPDATED')
            assert(events[2].name == 'HOUSING_STORAGE_ENTRY_UPDATED')
            assert(events[3].name == 'HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE')
            assert(events[4].name == 'HOUSING_SET_FIXTURE_RESPONSE')
            assert(events[3].payload == 0 and events[4].payload == 0)
            assert(originalPayload.recordID == 7101 and originalPayload.entryType == 1 and originalPayload.variantIdentifier == 0)
            assert(events[1].payload == originalPayload)
            assert(events[2].payload.recordID == 7101 and events[2].payload.entryType == 1 and events[2].payload.variantIdentifier == 0)
            local outer = events[1]
            assert(outer.stored == 4 and outer.size == 3 and outer.fixture == 302)
            assert(outer.all and not outer.selected)
            for index, row in ipairs(events) do
                assert(row.arity == 1 and row.exteriorType == 101 and row.fixture == 302)
                if index > 1 then
                    assert(row.stored == 5 and row.size == 4 and not row.all and not row.selected)
                end
            end
            assert(afterReentry.stored == 5 and afterReentry.size == 4 and afterReentry.fixture == 302 and not afterReentry.all)
            assert(C_HousingCatalog.GetCatalogEntryVariantInfo(originalPayload).numStored == 5)
        "#).unwrap();
        let state = env.state().borrow();
        let housing = &state.housing;
        assert_eq!(housing.exterior.selected_size, Some(4));
        assert_eq!(housing.exterior.selected_type_id, Some(101));
        assert_eq!(
            housing
                .exterior
                .selected_fixture_point
                .as_ref()
                .unwrap()
                .selected_fixture_id,
            Some(302)
        );
        assert_eq!(housing.exterior.decor.len(), 1);
        let floating = &housing.exterior.decor["floating"];
        assert_eq!(floating.variant_id, variant());
        assert_eq!(floating.position, [8.0, 9.0, 10.0]);
        assert_eq!(floating.fixture_point_owner_hash, None);
        assert_eq!(housing.catalog.variants[&variant()].num_stored, 5);
        assert_eq!(housing.catalog.entries[&entry()].total_num_stored, Some(5));
        assert_eq!(housing.catalog.entries[&entry()].total_num_placed, Some(0));
        assert!(
            state
                .events
                .pending()
                .iter()
                .all(|event| !event.name.starts_with("HOUSING_SET_")
                    && event.name != "HOUSING_STORAGE_ENTRY_UPDATED")
        );
    }

    #[test]
    fn mixed_valid_and_invalid_affected_variants_reject_store_atomically() {
        for method in [0, 1] {
            for missing in [true, false] {
                let env = fixture_env();
                let second = HousingCatalogEntryVariantID {
                    record_id: 7102,
                    entry_type: 1,
                    variant_identifier: 9,
                };
                let second_entry = HousingCatalogEntryID {
                    record_id: 7102,
                    entry_type: 1,
                };
                {
                    let mut state = env.state().borrow_mut();
                    let housing = &mut state.housing;
                    let mut record = housing.catalog.variants[&variant()].clone();
                    record.num_stored = i32::MAX;
                    housing.catalog.variants.insert(second.clone(), record);
                    let mut base = housing.catalog.entries[&entry()].clone();
                    base.item_id = Some(7102);
                    base.name = "Overflow exterior lamp".into();
                    base.total_num_stored = Some(i32::MAX as u32);
                    base.total_num_placed = Some(1);
                    housing.catalog.entries.insert(second_entry.clone(), base);
                    housing
                        .exterior
                        .decor
                        .get_mut("point11b")
                        .unwrap()
                        .variant_id = second.clone();
                    housing
                        .catalog
                        .entries
                        .get_mut(&entry())
                        .unwrap()
                        .total_num_placed = Some(2);
                    if missing {
                        housing.catalog.variants.remove(&second);
                    }
                }
                let snapshot = |env: &WowLuaEnv| {
                    let state = env.state().borrow();
                    let housing = &state.housing;
                    let variants = [variant(), second.clone()].map(|id| {
                        housing.catalog.variants.get(&id).map(|record| {
                            let dyes: Vec<_> = record
                                .dye_slots
                                .iter()
                                .map(|dye| {
                                    (
                                        dye.id,
                                        dye.dye_color_category_id,
                                        dye.order_index,
                                        dye.channel,
                                        dye.dye_color_id,
                                        dye.dye_color_name.clone(),
                                    )
                                })
                                .collect();
                            (record.num_stored, record.destroyable_instance_count, dyes)
                        })
                    });
                    let bases = [entry(), second_entry.clone()].map(|id| {
                        let base = &housing.catalog.entries[&id];
                        (
                            base.item_id,
                            base.name.clone(),
                            base.is_unique_trophy,
                            base.total_num_stored,
                            base.total_num_placed,
                        )
                    });
                    (
                        housing.exterior.clone(),
                        variants,
                        bases,
                        housing.catalog.variants.len(),
                        housing.catalog.entries.len(),
                    )
                };
                let before = snapshot(&env);
                let pending_before = env.state().borrow().events.pending().len();
                let (name, target, _) = CALLS[method];
                env.exec(&format!(r#"
                    local ok, message = pcall(C_HouseExterior.{name}, {target}, 0)
                    assert(not ok and type(message) == 'string' and #message > 0)
                    assert(#events == 0, 'invalid affected variant must reject before emitting events')
                "#)).unwrap();
                assert_eq!(snapshot(&env), before, "method={name}, missing={missing}");
                assert_eq!(env.state().borrow().events.pending().len(), pending_before);
            }
        }
    }

    #[test]
    fn environments_do_not_share_exterior_placements_or_storage() {
        let first = fixture_env();
        let second = fixture_env();
        let before = second.state().borrow().housing.exterior.clone();
        invoke(&first, 0, ", 0");
        assert_effect(&first, 0, true);
        assert_unchanged(&second, &before);
        second.exec("assert(#events==0)").unwrap();
        invoke(&second, 2, ", 1");
        assert_effect(&second, 2, false);
        assert_effect(&first, 0, true);
    }

    #[test]
    fn missing_selection_options_and_unavailable_host_reject_without_inventory_effect() {
        for method in 0..3 {
            for gap in 0..4 {
                let env = fixture_env();
                {
                    let mut state = env.state().borrow_mut();
                    let housing = &mut state.housing;
                    match gap {
                        0 => housing.inside_owned_plot = false,
                        1 => housing.active_house_editor_mode = 0,
                        2 => match method {
                            0 => housing.exterior.selected_fixture_point = None,
                            1 => housing.exterior.selected_size = None,
                            _ => housing.exterior.selected_type_id = None,
                        },
                        _ => match method {
                            0 => housing
                                .exterior
                                .selected_fixture_point
                                .as_mut()
                                .unwrap()
                                .options
                                .clear(),
                            1 => housing.exterior.size_options.clear(),
                            _ => housing.exterior.type_options.clear(),
                        },
                    }
                }
                reject_model(&env, method, CALLS[method].1);
            }
        }
    }

    #[test]
    fn store_preserves_unknown_base_totals_without_synthesizing_aggregates() {
        for method in 0..3 {
            let env = fixture_env();
            {
                let mut state = env.state().borrow_mut();
                let base = state.housing.catalog.entries.get_mut(&entry()).unwrap();
                base.total_num_stored = None;
                base.total_num_placed = None;
            }
            invoke(&env, method, ", 0");
            let state = env.state().borrow();
            let base = &state.housing.catalog.entries[&entry()];
            assert_eq!((base.total_num_stored, base.total_num_placed), (None, None));
            assert_eq!(
                state.housing.catalog.variants[&variant()].num_stored,
                if method == 0 { 4 } else { 5 }
            );
            assert_eq!(
                state.housing.exterior.decor.len(),
                if method == 0 { 2 } else { 1 }
            );
        }
    }

    #[test]
    fn invalid_inventory_host_state_rejects_store_atomically() {
        for method in 0..3 {
            for invalid in 0..5 {
                let env = fixture_env();
                {
                    let mut state = env.state().borrow_mut();
                    let housing = &mut state.housing;
                    match invalid {
                        0 => {
                            housing.catalog.variants.remove(&variant());
                        }
                        1 => {
                            housing
                                .catalog
                                .variants
                                .get_mut(&variant())
                                .unwrap()
                                .num_stored = -1
                        }
                        2 => {
                            housing
                                .catalog
                                .variants
                                .get_mut(&variant())
                                .unwrap()
                                .num_stored = i32::MAX
                        }
                        3 => {
                            housing
                                .catalog
                                .entries
                                .get_mut(&entry())
                                .unwrap()
                                .total_num_stored = Some(u32::MAX)
                        }
                        _ => {
                            housing
                                .catalog
                                .entries
                                .get_mut(&entry())
                                .unwrap()
                                .total_num_placed = Some(0)
                        }
                    }
                }
                let before = env.state().borrow().housing.exterior.clone();
                let count = env
                    .state()
                    .borrow()
                    .housing
                    .catalog
                    .variants
                    .get(&variant())
                    .map(|v| v.num_stored);
                let totals = {
                    let state = env.state().borrow();
                    let base = &state.housing.catalog.entries[&entry()];
                    (base.total_num_stored, base.total_num_placed)
                };
                let (name, target, response) = CALLS[method];
                env.exec(&format!(
                    r#"
                    local ok,message=pcall(C_HouseExterior.{name},{target},0)
                    if ok then
                        assert(#events==1 and events[1].name=='{response}' and events[1].payload~=0)
                    else
                        assert(type(message)=='string' and #message>0 and #events==0)
                    end
                "#
                ))
                .unwrap();
                let state = env.state().borrow();
                assert_eq!(state.housing.exterior, before);
                assert_eq!(
                    state
                        .housing
                        .catalog
                        .variants
                        .get(&variant())
                        .map(|v| v.num_stored),
                    count
                );
                let base = &state.housing.catalog.entries[&entry()];
                assert_eq!((base.total_num_stored, base.total_num_placed), totals);
            }
        }
    }
}

#[cfg(not(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
)))]
#[test]
fn legacy_inverse_control_preserves_seeded_queries_and_noop_mutators() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local oldSize=C_HouseExterior.GetCurrentHouseExteriorSize()
        local oldType,oldName=C_HouseExterior.GetCurrentHouseExteriorType()
        local oldPoint=C_HouseExterior.GetSelectedFixturePointInfo()
        local oldSizes=C_HouseExterior.GetHouseExteriorSizeOptions()
        local oldTypes=C_HouseExterior.GetHouseExteriorTypeOptions()
        assert(oldSize==(Enum.HousingFixtureSize.Medium or 3))
        assert(oldType==1 and oldName=='Sunspire Cottage')
        assert(oldPoint.fixtureID==1 and oldPoint.pointID==1 and oldPoint.name=='Front Door')
        assert(oldSizes.selectedSize==oldSize and #oldSizes.options==2)
        assert(oldSizes.options[1].size==oldSize and oldSizes.options[1].name=='Medium')
        assert(oldSizes.options[2].size==(Enum.HousingFixtureSize.Large or 4) and oldSizes.options[2].name=='Large')
        assert(oldTypes.selectedExteriorType==1 and #oldTypes.options==2)
        assert(oldTypes.options[1].houseExteriorTypeID==1 and oldTypes.options[1].name=='Sunspire Cottage')
        assert(oldTypes.options[2].houseExteriorTypeID==2 and oldTypes.options[2].name=='Sunspire Manor')
        assert(select('#',C_HouseExterior.SelectFixtureOption(302,0))==0)
        assert(select('#',C_HouseExterior.SetHouseExteriorSize(4,1))==0)
        assert(select('#',C_HouseExterior.SetHouseExteriorType(102))==0)
        assert(select('#',C_HouseExterior.RemoveFixtureFromSelectedPoint())==0)
        assert(select('#',C_HouseExterior.RemoveFixtureFromSelectedPoint(0))==0)
        assert(select('#',C_HouseExterior.RemoveFixtureFromSelectedPoint(1))==0)
        assert(C_HouseExterior.GetCurrentHouseExteriorSize()==oldSize)
        local id,name=C_HouseExterior.GetCurrentHouseExteriorType()
        assert(id==oldType and name==oldName)
        local point=C_HouseExterior.GetSelectedFixturePointInfo()
        assert(point.fixtureID==oldPoint.fixtureID and point.pointID==oldPoint.pointID and point.name==oldPoint.name)
        local sizes=C_HouseExterior.GetHouseExteriorSizeOptions()
        local types=C_HouseExterior.GetHouseExteriorTypeOptions()
        assert(sizes.selectedSize==oldSizes.selectedSize and #sizes.options==#oldSizes.options)
        assert(types.selectedExteriorType==oldTypes.selectedExteriorType and #types.options==#oldTypes.options)
        for index,option in ipairs(oldSizes.options) do
            assert(sizes.options[index].size==option.size and sizes.options[index].name==option.name)
        end
        for index,option in ipairs(oldTypes.options) do
            assert(types.options[index].houseExteriorTypeID==option.houseExteriorTypeID and types.options[index].name==option.name)
        end
    "#).unwrap();
}
