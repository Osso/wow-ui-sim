#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_housing::catalog::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogVariantRecord,
};
use wow_ui_sim::c_api::c_housing::exterior::{
    ExteriorCoreFixture, ExteriorCoreFixtureOption, ExteriorDecorPlacement,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::HousingState;

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

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let housing = &mut sim.housing;
        housing.inside_owned_plot = true;
        housing.active_house_editor_mode = 6;
        seed_core(housing);
        seed_catalog(housing);
        seed_placements(housing);
    }
    env.exec(
        r#"
        events = {}
        listener = CreateFrame('Frame')
        listener:RegisterEvent('HOUSING_STORAGE_ENTRY_UPDATED')
        listener:RegisterEvent('HOUSING_SET_FIXTURE_RESPONSE')
        listener:SetScript('OnEvent', function(_, event, ...)
            events[#events + 1] = {event, select('#', ...), ...}
        end)
    "#,
    )
    .unwrap();
    env
}

fn seed_core(housing: &mut HousingState) {
    housing.exterior.core_fixture = Some(ExteriorCoreFixture {
        selected_fixture_id: 301,
        options: vec![
            core_option(301, 11, Some(9)),
            core_option(302, 12, Some(9)),
            core_option(401, 21, Some(10)),
            core_option(501, 31, None),
        ],
    });
}

fn seed_catalog(housing: &mut HousingState) {
    housing.catalog.variants.insert(
        variant(),
        HousingCatalogVariantRecord {
            num_stored: 2,
            destroyable_instance_count: 1,
            dye_slots: vec![],
        },
    );
    housing.catalog.entries.insert(
        entry(),
        HousingCatalogEntryRecord {
            item_id: Some(7101),
            name: "Lamp".into(),
            is_unique_trophy: false,
            total_num_stored: Some(2),
            total_num_placed: Some(3),
        },
    );
}

fn seed_placements(housing: &mut HousingState) {
    for (id, owner) in [
        ("first", Some(11)),
        ("second", Some(11)),
        ("unrelated", Some(99)),
        ("floating", None),
    ] {
        housing.exterior.decor.insert(
            id.into(),
            ExteriorDecorPlacement {
                variant_id: variant(),
                position: [1.0, 2.0, 3.0],
                fixture_point_owner_hash: owner,
            },
        );
    }
}

fn core_option(id: u32, owner: u32, group: Option<u32>) -> ExteriorCoreFixtureOption {
    ExteriorCoreFixtureOption {
        fixture_id: id,
        owner_hash: owner,
        recolor_group: group,
        is_locked: false,
        is_invalid: false,
    }
}

fn assert_inventory(env: &WowLuaEnv, stored: i32, placed: u32) {
    let sim = env.state().borrow();
    let catalog = &sim.housing.catalog;
    assert_eq!(catalog.variants[&variant()].num_stored, stored);
    assert_eq!(catalog.variants[&variant()].destroyable_instance_count, 1);
    assert!(catalog.variants[&variant()].dye_slots.is_empty());
    assert_eq!(
        catalog.entries[&entry()].total_num_stored,
        Some(stored as u32)
    );
    assert_eq!(catalog.entries[&entry()].total_num_placed, Some(placed));
    assert_eq!(catalog.entries.len(), 1);
    assert_eq!(catalog.variants.len(), 1);
}

fn assert_selection(env: &WowLuaEnv, id: u32) {
    assert_eq!(
        env.state()
            .borrow()
            .housing
            .exterior
            .core_fixture
            .as_ref()
            .unwrap()
            .selected_fixture_id,
        id
    );
}

fn assert_unaffected(env: &WowLuaEnv) {
    let sim = env.state().borrow();
    let decor = &sim.housing.exterior.decor;
    assert_eq!(decor["unrelated"].fixture_point_owner_hash, Some(99));
    assert_eq!(decor["floating"].fixture_point_owner_hash, None);
    for placement in decor.values() {
        assert_eq!(placement.position, [1.0, 2.0, 3.0]);
        assert_eq!(placement.variant_id, variant());
    }
}

#[test]
fn core_different_style_default_and_explicit_store_commit_inventory() {
    for args in ["401", "401, 0", "401, nil"] {
        let env = seeded_env();
        env.exec(&format!(
            "assert(select('#', C_HouseExterior.SelectCoreFixtureOption({args})) == 0)"
        ))
        .unwrap();
        assert_selection(&env, 401);
        assert_inventory(&env, 4, 1);
        assert_unaffected(&env);
        let sim = env.state().borrow();
        assert_eq!(sim.housing.exterior.decor.len(), 2);
        assert!(!sim.housing.exterior.decor.contains_key("first"));
        assert!(!sim.housing.exterior.decor.contains_key("second"));
        drop(sim);
        env.exec(
            r#"
            assert(#events == 2)
            assert(events[1][1] == 'HOUSING_STORAGE_ENTRY_UPDATED')
            assert(events[1][2] == 1 and events[1][3].recordID == 7101)
            assert(events[2][1] == 'HOUSING_SET_FIXTURE_RESPONSE')
            assert(events[2][2] == 1 and events[2][3] == 0)
        "#,
        )
        .unwrap();
    }
}

#[test]
fn core_detach_preserves_placements_and_inventory() {
    let env = seeded_env();
    env.exec("C_HouseExterior.SelectCoreFixtureOption(401, Enum.HousingFixtureDecorAction.Detach)")
        .unwrap();
    assert_selection(&env, 401);
    assert_inventory(&env, 2, 3);
    assert_unaffected(&env);
    let sim = env.state().borrow();
    assert_eq!(sim.housing.exterior.decor.len(), 4);
    assert_eq!(
        sim.housing.exterior.decor["first"].fixture_point_owner_hash,
        None
    );
    assert_eq!(
        sim.housing.exterior.decor["second"].fixture_point_owner_hash,
        None
    );
    drop(sim);
    env.exec("assert(#events == 1 and events[1][1] == 'HOUSING_SET_FIXTURE_RESPONSE')")
        .unwrap();
}

#[test]
fn core_recolor_reparents_even_when_store_is_requested() {
    for args in ["302", "302, 0", "302, 1"] {
        let env = seeded_env();
        env.exec(&format!("C_HouseExterior.SelectCoreFixtureOption({args})"))
            .unwrap();
        assert_selection(&env, 302);
        assert_inventory(&env, 2, 3);
        assert_unaffected(&env);
        let sim = env.state().borrow();
        assert_eq!(sim.housing.exterior.decor.len(), 4);
        assert_eq!(
            sim.housing.exterior.decor["first"].fixture_point_owner_hash,
            Some(12)
        );
        assert_eq!(
            sim.housing.exterior.decor["second"].fixture_point_owner_hash,
            Some(12)
        );
        drop(sim);
        env.exec("assert(#events == 1 and events[1][3] == 0)")
            .unwrap();
        // Follow-up proves the new parent is real ownership, not just an ID label.
        env.exec("C_HouseExterior.SelectCoreFixtureOption(401, 0)")
            .unwrap();
        assert_inventory(&env, 4, 1);
        assert_eq!(env.state().borrow().housing.exterior.decor.len(), 2);
    }
}

#[test]
fn core_authenticates_original_arguments_before_parsing_or_lookup() {
    let env = seeded_env();
    env.exec(
        r#"
        local api = C_HouseExterior.SelectCoreFixtureOption
        -- Wrap while secure: a tainted caller cannot create secrets, only pass them.
        local s302, s1, s0, snil = secretwrap(302), secretwrap(1), secretwrap(0), secretwrap(nil)
        local probes = {
            function() api(s302, 0) end,
            function() api(302, s1) end,
            function() api(false, s0) end,
            function() api(999, snil) end,
        }
        for _, probe in ipairs(probes) do
            debug.setobjecttaint(probe, 'CoreFixtureProbe')
            local ok, failure = pcall(probe)
            assert(not ok and failure:find('secret access denied', 1, true),
                'core secret probe must reach API authentication denial: ' .. tostring(failure))
        end
        assert(#events == 0, 'core secret rejection must not publish events')
    "#,
    )
    .unwrap();
    assert_selection(&env, 301);
    assert_inventory(&env, 2, 3);
    env.exec("C_HouseExterior.SelectCoreFixtureOption(secretwrap(302), secretwrap(0))")
        .unwrap();
    assert_selection(&env, 302);
    env.exec(
        r#"
        local function ordinary() C_HouseExterior.SelectCoreFixtureOption(401, 1) end
        debug.setobjecttaint(ordinary, 'CoreFixtureProbe')
        ordinary()
    "#,
    )
    .unwrap();
    assert_selection(&env, 401);
    assert_inventory(&env, 2, 3);
}

#[test]
fn core_rejections_are_atomic_and_do_not_publish_events() {
    for args in [
        "",
        "false, 0",
        "'401', 0",
        "401.5, 0",
        "0/0, 0",
        "math.huge, 0",
        "-1, 0",
        "401, 2",
        "401, false",
        "999, 0",
    ] {
        let env = seeded_env();
        let before = env.state().borrow().housing.exterior.clone();
        env.exec(&format!("assert(not pcall(function() C_HouseExterior.SelectCoreFixtureOption({args}) end)); assert(#events == 0)"))
            .unwrap();
        assert_eq!(env.state().borrow().housing.exterior, before);
        assert_inventory(&env, 2, 3);
    }
    for blocked in 0..5 {
        let env = seeded_env();
        {
            let mut sim = env.state().borrow_mut();
            match blocked {
                0 => sim.housing.inside_owned_plot = false,
                1 => sim.housing.active_house_editor_mode = 0,
                2 => {
                    sim.housing.exterior.core_fixture.as_mut().unwrap().options[2].is_locked = true
                }
                3 => {
                    sim.housing.exterior.core_fixture.as_mut().unwrap().options[2].is_invalid = true
                }
                _ => sim.housing.exterior.core_fixture = None,
            }
        }
        let before = env.state().borrow().housing.exterior.clone();
        env.exec("assert(not pcall(C_HouseExterior.SelectCoreFixtureOption, 401, 0)); assert(#events == 0)").unwrap();
        assert_eq!(env.state().borrow().housing.exterior, before);
        assert_inventory(&env, 2, 3);
    }
}

#[test]
fn core_store_validates_all_counts_before_mutating_any_placement() {
    for failure in 0..3 {
        let env = seeded_env();
        {
            let mut sim = env.state().borrow_mut();
            match failure {
                0 => {
                    sim.housing.catalog.variants.remove(&variant());
                }
                1 => {
                    sim.housing
                        .catalog
                        .variants
                        .get_mut(&variant())
                        .unwrap()
                        .num_stored = i32::MAX
                }
                _ => {
                    sim.housing
                        .catalog
                        .entries
                        .get_mut(&entry())
                        .unwrap()
                        .total_num_placed = Some(1)
                }
            }
        }
        let before = env.state().borrow().housing.exterior.clone();
        let counts = env
            .state()
            .borrow()
            .housing
            .catalog
            .variants
            .get(&variant())
            .map(|row| row.num_stored);
        env.exec("assert(not pcall(C_HouseExterior.SelectCoreFixtureOption, 401, 0)); assert(#events == 0)").unwrap();
        let sim = env.state().borrow();
        assert_eq!(sim.housing.exterior, before);
        assert_eq!(
            sim.housing
                .catalog
                .variants
                .get(&variant())
                .map(|row| row.num_stored),
            counts
        );
        assert_eq!(
            sim.housing.catalog.entries[&entry()].total_num_stored,
            Some(2)
        );
        assert_eq!(
            sim.housing.catalog.entries[&entry()].total_num_placed,
            Some(if failure == 2 { 1 } else { 3 })
        );
    }
}

#[test]
fn core_host_updates_empty_defaults_and_environment_isolation() {
    let env = seeded_env();
    let other = seeded_env();
    env.state()
        .borrow_mut()
        .housing
        .exterior
        .core_fixture
        .as_mut()
        .unwrap()
        .options[1]
        .recolor_group = Some(77);
    env.exec("C_HouseExterior.SelectCoreFixtureOption(302, 0)")
        .unwrap();
    assert_inventory(&env, 4, 1);
    assert_selection(&other, 301);
    assert_inventory(&other, 2, 3);
    // Unknown groups never imply equivalence merely because both are None.
    other
        .state()
        .borrow_mut()
        .housing
        .exterior
        .core_fixture
        .as_mut()
        .unwrap()
        .options[0]
        .recolor_group = None;
    other
        .exec("C_HouseExterior.SelectCoreFixtureOption(501, 0)")
        .unwrap();
    assert_inventory(&other, 4, 1);
    let empty = WowLuaEnv::new().unwrap();
    assert!(
        empty
            .state()
            .borrow()
            .housing
            .exterior
            .core_fixture
            .is_none()
    );
    empty
        .exec("assert(not pcall(C_HouseExterior.SelectCoreFixtureOption, 401))")
        .unwrap();
}

#[test]
fn core_mixed_variants_reject_before_storing_the_valid_stack() {
    let env = seeded_env();
    let absent = HousingCatalogEntryVariantID {
        record_id: 7200,
        entry_type: 1,
        variant_identifier: 17,
    };
    env.state()
        .borrow_mut()
        .housing
        .exterior
        .decor
        .get_mut("second")
        .unwrap()
        .variant_id = absent;
    let before = env.state().borrow().housing.exterior.clone();
    env.exec(
        "assert(not pcall(C_HouseExterior.SelectCoreFixtureOption, 401, 0)); assert(#events == 0)",
    )
    .unwrap();
    assert_eq!(env.state().borrow().housing.exterior, before);
    assert_inventory(&env, 2, 3);
}

#[test]
fn core_storage_callbacks_reenter_committed_selection_without_outer_overwrite() {
    let env = seeded_env();
    env.exec(
        r#"
        listener:SetScript('OnEvent', function(_, event, ...)
            events[#events + 1] = {event, select('#', ...), ...}
            if event == 'HOUSING_STORAGE_ENTRY_UPDATED' then
                C_HouseExterior.SelectCoreFixtureOption(302, 1)
            end
        end)
        C_HouseExterior.SelectCoreFixtureOption(401, 0)
        assert(#events == 3)
        assert(events[1][1] == 'HOUSING_STORAGE_ENTRY_UPDATED')
        assert(events[2][1] == 'HOUSING_SET_FIXTURE_RESPONSE' and events[2][3] == 0)
        assert(events[3][1] == 'HOUSING_SET_FIXTURE_RESPONSE' and events[3][3] == 0)
    "#,
    )
    .unwrap();
    assert_selection(&env, 302);
    assert_inventory(&env, 4, 1);
    assert_eq!(env.state().borrow().housing.exterior.decor.len(), 2);
}

#[test]
fn core_same_selection_does_not_store_or_detach_attachments() {
    let env = seeded_env();
    let before = env.state().borrow().housing.exterior.clone();
    env.exec("C_HouseExterior.SelectCoreFixtureOption(301, 0)")
        .unwrap();
    assert_eq!(env.state().borrow().housing.exterior, before);
    assert_inventory(&env, 2, 3);
    env.exec("assert(#events == 1 and events[1][3] == 0)")
        .unwrap();
}
