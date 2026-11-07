#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::state::OwnedKeystone;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn challenges_ui_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ChallengesUI/Blizzard_ChallengesUI_Mainline.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

prefork_full_ui_case! {
fn blizzard_challenges_ui_loads_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &challenges_ui_toc())
        .expect("Blizzard_ChallengesUI should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ChallengesUI emitted Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let frames_present: bool = env
        .eval(
            "return _G.ChallengesFrame ~= nil \
                and _G.ChallengesKeystoneFrame ~= nil \
                and _G.ChallengeModeCompleteBanner ~= nil",
        )
        .expect("frame query should succeed");
    assert!(
        frames_present,
        "ChallengesFrame, ChallengesKeystoneFrame, ChallengeModeCompleteBanner should be defined after load"
    );

    let mixins_present: bool = env
        .eval(
            "return type(ChallengesFrameMixin) == 'table' \
                and type(ChallengesKeystoneFrameMixin) == 'table' \
                and type(ChallengeModeCompleteBannerMixin) == 'table' \
                and type(ChallengesDungeonIconMixin) == 'table' \
                and type(ChallengesKeystoneSlotMixin) == 'table' \
                and type(ChallengesKeystoneFrameAffixMixin) == 'table' \
                and type(ChallengesFrameWeeklyInfoMixin) == 'table' \
                and type(ChallengeModeBannerPartyMemberMixin) == 'table' \
                and type(ChallengeModeLegacyWeeklyChestMixin) == 'table' \
                and type(DungeonScoreInfoMixin) == 'table'",
        )
        .expect("mixin query should succeed");
    assert!(
        mixins_present,
        "Blizzard_ChallengesUI mixins should be defined after load"
    );
}
}

prefork_full_ui_case! {
fn challenges_keystone_filter_tracks_item_and_map_state(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_challenge_mode::KeystoneLocation;

    load_addon(&env.loader_env(), &challenges_ui_toc()).expect("ChallengesUI should load");
    env.exec(r#"
        A_Admin.ClearBags()
        A_Admin.AddBagItem(0, 1, 158923, 1)
        A_Admin.AddBagItem(0, 2, 6948, 1)
        keystoneLocation = ItemLocation:CreateFromBagAndSlot(0, 1)
        ordinaryLocation = ItemLocation:CreateFromBagAndSlot(0, 2)
        ChallengesKeystoneFrame:Show()
    "#).expect("keystone bag fixture should load");
    {
        let mut sim = env.state().borrow_mut();
        sim.keystone.item_modifiers.insert(
            KeystoneLocation::Bag(0, 1),
            (158923, vec![(17, 2), (17, 56)]),
        );
        sim.keystone.current_map_selector = Some(960);
    }
    let (keystone_matches, ordinary_matches): (bool, bool) = env.eval(r#"
        local expected = ItemButtonUtil.ItemContextMatchResult.Match
        return ItemButtonUtil.GetItemContextMatchResultForItem(keystoneLocation) == expected,
            ItemButtonUtil.GetItemContextMatchResultForItem(ordinaryLocation) == expected
    "#).expect("original ItemUtil keystone filter should run");
    assert_eq!((keystone_matches, ordinary_matches), (true, false));
    env.exec(r#"
        assert(C_Item.IsItemKeystoneByID(158923))
        assert(C_Item.IsItemKeystoneByID('item:158923'))
        assert(C_Item.IsItemKeystoneByID('Mythic Keystone'))
        assert(not C_Item.IsItemKeystoneByID(6948))
        assert(not C_Item.IsItemKeystoneByID(999999999))
        assert(not pcall(C_Item.IsItemKeystoneByID, nil))
        assert(not pcall(C_ChallengeMode.CanUseKeystoneInCurrentMap, 158923))
        assert(not pcall(C_ChallengeMode.CanUseKeystoneInCurrentMap, ItemLocation:CreateFromBagAndSlot(0, 99)))
    "#).expect("item-info and location contracts should hold");
    env.state().borrow_mut().keystone.current_map_selector = Some(961);
    let matches: bool = env.eval(r#"
        return ItemButtonUtil.GetItemContextMatchResultForItem(keystoneLocation)
            == ItemButtonUtil.ItemContextMatchResult.Match
    "#).expect("original ItemUtil filter should reflect the supplied map");
    assert!(!matches);
}
}

prefork_full_ui_case! {
fn challenges_keystone_close_and_clear_keep_independent_state(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_challenge_mode::KeystoneLocation;

    load_addon(&env.loader_env(), &challenges_ui_toc()).expect("ChallengesUI should load");
    let slot = KeystoneLocation::Bag(0, 1);
    env.exec("A_Admin.ClearBags(); A_Admin.AddBagItem(0, 1, 158923, 1)")
        .expect("selected item fixture should exist");
    {
        let mut sim = env.state().borrow_mut();
        sim.keystone.receptacle_guid = Some("GameObject-0-0-0-0-42".into());
        sim.keystone.selected_guid = Some("Item-0-0-42".into());
        sim.keystone.selected_location = Some(slot);
        sim.keystone.selected_mode = Some(7);
        sim.keystone.reason_one_locks.insert(slot);
        sim.keystone.other_locks.insert(slot);
    }
    let locked: bool = env.eval("return C_Container.GetContainerItemInfo(0, 1).isLocked")
        .expect("selected item lock should be observable");
    assert!(locked);
    env.exec("C_ChallengeMode.CloseKeystoneFrame()")
        .expect("receptacle close should execute");
    {
        let sim = env.state().borrow();
        assert!(sim.keystone.receptacle_guid.is_none());
        assert!(sim.keystone.selected_guid.is_some());
        assert!(sim.keystone.reason_one_locks.contains(&slot));
    }
    env.exec("C_ChallengeMode.ClearKeystone()")
        .expect("selection clear should execute");
    {
        let sim = env.state().borrow();
        assert!(sim.keystone.selected_guid.is_none());
        assert!(sim.keystone.selected_location.is_none());
        assert!(sim.keystone.selected_mode.is_none());
        assert!(!sim.keystone.reason_one_locks.contains(&slot));
        assert!(sim.keystone.other_locks.contains(&slot));
    }
    env.state().borrow_mut().keystone.selected_mode = Some(9);
    env.exec("C_ChallengeMode.ClearKeystone()")
        .expect("clearing an empty selection should execute");
    assert_eq!(env.state().borrow().keystone.selected_mode, Some(9));
    env.exec(r#"
        keystoneLockEvents = {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('ITEM_LOCK_CHANGED')
        observer:RegisterEvent('ITEM_UNLOCKED')
        observer:SetScript('OnEvent', function(_, event, bag, slot)
            table.insert(keystoneLockEvents, event .. ':' .. bag .. ':' .. tostring(slot))
        end)
    "#).expect("lock event observer should register");
    {
        let mut sim = env.state().borrow_mut();
        sim.keystone.selected_guid = Some("Item-0-0-42".into());
        sim.keystone.selected_location = Some(slot);
        sim.keystone.reason_one_locks.insert(slot);
        sim.keystone.other_locks.remove(&slot);
    }
    env.exec("C_ChallengeMode.ClearKeystone()")
        .expect("clearing the last lock should execute");
    let locked: bool = env.eval("return C_Container.GetContainerItemInfo(0, 1).isLocked")
        .expect("unlocked item should be observable");
    assert!(!locked);
    let events: String = env.eval("return table.concat(keystoneLockEvents, ',')")
        .expect("lock events should be observable");
    assert_eq!(events, "ITEM_LOCK_CHANGED:0:1,ITEM_UNLOCKED:0:1");
    {
        let mut sim = env.state().borrow_mut();
        let bag = sim.bag_info.get_mut(&1).expect("bag slot fixture");
        bag.inventory_slot = Some(20);
        bag.item_id = Some(158923);
        let equipment = KeystoneLocation::Equipment(20);
        sim.keystone.selected_guid = Some("Item-0-0-43".into());
        sim.keystone.selected_location = Some(equipment);
        sim.keystone.reason_one_locks.insert(equipment);
    }
    env.exec("assert(IsInventoryItemLocked(20)); C_ChallengeMode.ClearKeystone(); assert(not IsInventoryItemLocked(20))")
        .expect("equipped bag lock should clear");
    let events: String = env.eval("return table.concat(keystoneLockEvents, ',')")
        .expect("equipment lock events should be observable");
    assert!(events.ends_with("ITEM_LOCK_CHANGED:20:nil,ITEM_UNLOCKED:20:nil"));
}
}

prefork_full_ui_case! {
fn challenges_keystone_frame_show_and_hide_run_without_errors(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &challenges_ui_toc()).expect("Blizzard_ChallengesUI should load");

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("ChallengesKeystoneFrame:Show(); ChallengesKeystoneFrame:Hide()")
        .expect("ChallengesKeystoneFrame Show/Hide should run");

    let unexpected_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        unexpected_errors.is_empty(),
        "ChallengesKeystoneFrame Show/Hide emitted Lua errors:\n  {}",
        unexpected_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn challenges_weekly_chest_reads_the_owned_keystone_context(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &challenges_ui_toc()).expect("Blizzard_ChallengesUI should load");

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    // Original consumer: ChallengeModeWeeklyChestMixin:Update
    // (Blizzard_ChallengesUI/Mainline/Blizzard_ChallengesUI.lua:295) tests
    // C_MythicPlus.GetOwnedKeystoneLevel() for truthiness at :312. A Lua 0 is
    // truthy, so publishing 0 instead of nothing changes the chest state from
    // WALL_OF_TEXT (1) to INCOMPLETE (2) even when the player owns no key.
    let chest_state_without_a_key: i32 = env
        .eval("return ChallengesFrame.WeeklyInfo.Child.WeeklyChest:Update(0, nil)")
        .expect("the weekly chest consumer should run");

    let unexpected_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        unexpected_errors.is_empty(),
        "ChallengeModeWeeklyChestMixin:Update emitted Lua errors:\n  {}",
        unexpected_errors.join("\n  ")
    );
    assert_eq!(
        chest_state_without_a_key, 1,
        "no owned key means no truthy level, so the chest stays wall-of-text"
    );

    // With a keystone context present the same consumer takes the INCOMPLETE
    // branch (:312) because the published level is truthy.
    {
        let mut state = env.state().borrow_mut();
        state.mythic_plus.owned_keystone = Some(OwnedKeystone {
            level: 10,
            challenge_map_id: 0,
        });
    }
    let chest_state_with_a_key: i32 = env
        .eval("return ChallengesFrame.WeeklyInfo.Child.WeeklyChest:Update(0, nil)")
        .expect("the weekly chest consumer should run with an owned key");
    assert_eq!(chest_state_with_a_key, 2, "an owned key selects the incomplete chest");
}
}
