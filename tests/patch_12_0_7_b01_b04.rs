#![cfg(feature = "retail-12-0-7")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::c_api::bag_info::BagInfo;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

const ASSERTIONS: &str = r#"
        function CheckAsset(asset, expected)
            local function check(...)
                assert(select('#', ...) == 1)
                local value = ...
                assert(value == expected and not issecretvalue(value))
            end
            check(C_UIFileAsset.GetFileID(asset))
        end
        function RejectCall(query, message, ...)
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string')
            assert(string.find(err, message, 1, true), err)
        end
        function CheckTitle(expected, ...)
            local function check(...)
                assert(select('#', ...) == 1)
                local value = ...
                assert(value == expected and not issecretvalue(value))
            end
            check(C_DelvesUI.GetDelveEntranceTitleString(...))
        end
        "#;

fn new_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create bounded batch environment");
    env.exec(ASSERTIONS).unwrap();
    env
}

fn install_secrets(env: &WowLuaEnv) {
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
    }
    install_number_secrets(env);
    install_string_secrets(env);
    install_other_secrets(env);
}

fn install_number_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, number) in [("SecretAssetID", 136243.0), ("SecretAccountID", 64217.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.unwrap();
    }
}

fn install_string_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, text) in [
        ("SecretAssetPath", "Interface/Icons/Trade_Engineering.blp"),
        ("SecretBadAccount", "not-an-account"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.unwrap();
    }
}

fn install_other_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, payload) in [
        ("SecretNil", rilua::Val::Nil),
        ("SecretFalse", rilua::Val::Bool(false)),
    ] {
        let value = wrap_secret(lua.state_mut(), payload).unwrap();
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.unwrap();
    }
}

#[test]
fn b01_asset_ids_and_normalized_catalog_paths_are_concrete_not_constants() {
    new_env()
        .exec(
            r#"
        CheckAsset(123, 123)
        CheckAsset(136243, 136243)
        CheckAsset(4294967295, 4294967295)
        CheckAsset('Interface\\Icons\\Trade_Engineering.blp', 136243)
        CheckAsset('iNtErFaCe/iCoNs/TrAdE_EnGiNeErInG.bLp', 136243)
        CheckAsset('Interface/Icons/Unseeded-B01.blp', nil)
        CheckAsset('', nil)
        CheckAsset('136243', nil)
        "#,
        )
        .unwrap();
}

#[test]
fn b01_inferred_numeric_boundaries_and_mandatory_asset_type() {
    new_env()
        .exec(
            r#"
        for _, value in ipairs({0, -1, 0.5, 123.75, 4294967296, 0/0, math.huge, -math.huge}) do
            CheckAsset(value, nil)
        end
        RejectCall(C_UIFileAsset.GetFileID, 'asset must', nil)
        RejectCall(C_UIFileAsset.GetFileID, 'asset must')
        for _, value in ipairs({false, true, {}, function() end, CreateFrame('Frame')}) do
            RejectCall(C_UIFileAsset.GetFileID, 'asset must', value)
        end
        CheckAsset(123, 123)
        "#,
        )
        .unwrap();
}

#[test]
fn b01_untainted_secrets_work_and_remain_rooted_across_gc() {
    let env = new_env();
    install_secrets(&env);
    env.exec(
        r#"
        local id, path = SecretAssetID, SecretAssetPath
        for iteration = 1, 3 do
            collectgarbage('collect')
            CheckAsset(SecretAssetID, 136243)
            CheckAsset(SecretAssetPath, 136243)
            assert(C_UIFileAsset.GetFileID(123, SecretNil, SecretFalse) == 123)
            assert(issecretvalue(id) and issecretvalue(path))
            assert(rawequal(id, SecretAssetID) and rawequal(path, SecretAssetPath))
            assert(issecure())
        end
        RejectCall(C_UIFileAsset.GetFileID, 'asset must', SecretFalse)
        CheckAsset(123, 123)
        "#,
    )
    .unwrap();
}

#[test]
fn b01_tainted_secret_extras_are_denied_before_invalid_first_argument() {
    let env = new_env();
    install_secrets(&env);
    env.exec(
        r#"
        local function probe()
            assert(debug.getstacktaint() == 'B01Probe')
            CheckAsset(123, 123)
            CheckAsset('Interface/Icons/Trade_Engineering.blp', 136243)
            RejectCall(C_UIFileAsset.GetFileID, 'untainted caller', SecretAssetID)
            RejectCall(C_UIFileAsset.GetFileID, 'untainted caller', SecretAssetPath)
            RejectCall(C_UIFileAsset.GetFileID, 'untainted caller', SecretFalse)
            RejectCall(C_UIFileAsset.GetFileID, 'untainted caller', false, SecretNil)
            RejectCall(C_UIFileAsset.GetFileID, 'untainted caller', false, 0, SecretFalse)
            assert(C_UIFileAsset.GetFileID(123, false, {}) == 123)
            assert(issecretvalue(SecretAssetID) and issecretvalue(SecretFalse))
            assert(debug.getstacktaint() == 'B01Probe')
        end
        debug.setobjecttaint(probe, 'B01Probe')
        probe()
        assert(issecure())
        CheckAsset(SecretAssetID, 136243)
        "#,
    )
    .unwrap();
}

#[test]
fn b02_numeric_invitation_requests_replace_live_input_without_fake_friendship() {
    let env = new_env();
    let before = env.state().borrow().bnet_friends.len();
    assert_eq!(env.state().borrow().last_bnet_invite_game_account_id, None);
    env.exec("assert(select('#', C_BattleNet.InviteFriend(64217)) == 0)")
        .unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(64217)
    );
    env.exec("C_BattleNet.InviteFriend(91023); C_BattleNet.InviteFriend(91023)")
        .unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(91023)
    );
    assert_eq!(env.state().borrow().bnet_friends.len(), before);
    let result: f64 = env.eval("return C_BattleNet.GetNumFriends()").unwrap();
    assert_eq!(result, before as f64);
    let other = new_env();
    assert_eq!(
        other.state().borrow().last_bnet_invite_game_account_id,
        None
    );
    other.exec("C_BattleNet.InviteFriend(17)").unwrap();
    assert_eq!(
        other.state().borrow().last_bnet_invite_game_account_id,
        Some(17)
    );
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(91023)
    );
}

#[test]
fn b02_cached_deprecated_dispatch_forwards_numeric_account_and_zero_results() {
    let env = new_env();
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    let source = std::fs::read_to_string(addons.join("Blizzard_DeprecatedBattleNet/Deprecated_BattleNet.lua")).unwrap();
    env.exec("SetCVar('loadDeprecationFallbacks', '1')").unwrap();
    env.exec(&source).unwrap();
    env.exec("assert(select('#', BNInviteFriend(80231)) == 0)").unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(80231)
    );
}

#[test]
fn b02_invalid_ids_do_not_mutate_the_last_request() {
    let env = new_env();
    env.exec(
        r#"
        C_BattleNet.InviteFriend(64217)
        RejectCall(C_BattleNet.InviteFriend, 'gameAccountID', nil)
        RejectCall(C_BattleNet.InviteFriend, 'gameAccountID')
        for _, value in ipairs({'Jaina#3000', '64217', '', false, {}, function() end,
            CreateFrame('Frame'), 0, -1, 0.5, 2147483648, 0/0, math.huge, -math.huge}) do
            RejectCall(C_BattleNet.InviteFriend, 'gameAccountID', value)
        end
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(64217)
    );
    env.exec("C_BattleNet.InviteFriend(2147483647)").unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(i32::MAX)
    );
}

#[test]
fn b02_untainted_secrets_and_every_extra_are_authenticated() {
    let env = new_env();
    install_secrets(&env);
    env.exec(
        r#"
        local id = SecretAccountID
        for iteration = 1, 3 do
            collectgarbage('collect')
            assert(select('#', C_BattleNet.InviteFriend(SecretAccountID, SecretNil, SecretFalse)) == 0)
            assert(rawequal(id, SecretAccountID) and issecretvalue(id))
            assert(issecure())
        end
        RejectCall(C_BattleNet.InviteFriend, 'gameAccountID', SecretBadAccount)
        "#,
    ).unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(64217)
    );
}

#[test]
fn b02_tainted_denial_precedes_validation_and_has_no_invitation_side_effect() {
    let env = new_env();
    install_secrets(&env);
    env.exec(
        r#"
        local function probe()
            assert(debug.getstacktaint() == 'B02Probe')
            C_BattleNet.InviteFriend(91023, false, {})
            RejectCall(C_BattleNet.InviteFriend, 'untainted caller', SecretAccountID)
            RejectCall(C_BattleNet.InviteFriend, 'untainted caller', SecretBadAccount)
            RejectCall(C_BattleNet.InviteFriend, 'untainted caller', SecretNil)
            RejectCall(C_BattleNet.InviteFriend, 'untainted caller', false, SecretAccountID)
            RejectCall(C_BattleNet.InviteFriend, 'untainted caller', nil, 0, SecretFalse)
            assert(issecretvalue(SecretAccountID) and issecretvalue(SecretFalse))
            assert(debug.getstacktaint() == 'B02Probe')
        end
        debug.setobjecttaint(probe, 'B02Probe')
        probe()
        assert(issecure())
        "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(91023)
    );
    env.exec("C_BattleNet.InviteFriend(SecretAccountID)")
        .unwrap();
    assert_eq!(
        env.state().borrow().last_bnet_invite_game_account_id,
        Some(64217)
    );
}

fn bag(capacity: i32, family: i32) -> BagInfo {
    BagInfo {
        num_slots: capacity,
        family,
        name: None,
        inventory_slot: None,
        item_id: None,
        hyperlink: None,
    }
}

fn item() -> BagItem {
    BagItem {
        item_id: 2589,
        stack_count: 3,
        hyperlink: None,
    }
}

#[test]
fn b03_free_slots_follow_capacity_occupancy_family_and_missing_bags_live() {
    let env = new_env();
    {
        let mut state = env.state().borrow_mut();
        state.bag_info.clear();
        state.bag_items.clear();
        state.bag_info.insert(0, bag(4, 0));
        state.bag_info.insert(5, bag(3, 32));
        state.bag_info.insert(-1, bag(80, 0));
        state.bag_info.insert(6, bag(90, 0));
        state.bag_items.insert((0, 1), item());
        state.bag_items.insert((0, 4), item());
        state.bag_items.insert((5, 2), item());
        state.bag_items.insert((5, 8), item());
    }
    let metadata = env.state().borrow().bag_info.clone();
    let value: f64 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(value, 4.0);
    env.exec("assert(select('#', C_Container.CalculateTotalNumberOfFreeBagSlots()) == 1)")
        .unwrap();
    assert_eq!(env.state().borrow().bag_info, metadata);
    assert_eq!(env.state().borrow().bag_items.len(), 4);
    assert_eq!(env.state().borrow().bag_items[&(0, 1)].stack_count, 3);
    env.state()
        .borrow_mut()
        .bag_info
        .get_mut(&5)
        .unwrap()
        .num_slots = 5;
    let value: f64 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(value, 6.0);
    env.state().borrow_mut().bag_items.remove(&(0, 1));
    let value: f64 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(value, 7.0);
    env.state().borrow_mut().bag_info.remove(&5);
    let value: f64 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(value, 3.0);
    env.state().borrow_mut().bag_info.clear();
    let value: f64 = env
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(value, 0.0);
}

#[test]
fn b03_no_argument_query_ignores_secret_extras_and_environments_are_isolated() {
    let first = new_env();
    let second = new_env();
    for (env, capacity) in [(&first, 7), (&second, 11)] {
        let mut state = env.state().borrow_mut();
        state.bag_info.clear();
        state.bag_items.clear();
        state.bag_info.insert(0, bag(capacity, 0));
    }
    install_secrets(&first);
    first
        .exec(
            r#"
        local function probe()
            assert(C_Container.CalculateTotalNumberOfFreeBagSlots(SecretNil, SecretFalse) == 7)
            assert(debug.getstacktaint() == 'B03Probe')
        end
        debug.setobjecttaint(probe, 'B03Probe')
        probe()
        assert(issecure())
        "#,
        )
        .unwrap();
    first.state().borrow_mut().bag_info.clear();
    let value: f64 = second
        .eval("return C_Container.CalculateTotalNumberOfFreeBagSlots()")
        .unwrap();
    assert_eq!(value, 11.0);
}

#[test]
fn b04_title_reads_live_host_text_replacement_removal_and_isolation() {
    let first = new_env();
    let second = new_env();
    first.exec("CheckTitle(nil)").unwrap();
    first.state().borrow_mut().delve_entrance_title = Some("Host A — Deep Entrance".into());
    first.exec("CheckTitle('Host A — Deep Entrance')").unwrap();
    first.state().borrow_mut().delve_entrance_title = Some("Host B".into());
    first.exec("CheckTitle('Host B')").unwrap();
    first.state().borrow_mut().delve_entrance_title = Some(String::new());
    first.exec("CheckTitle('')").unwrap();
    first.state().borrow_mut().delve_entrance_title = None;
    first.exec("CheckTitle(nil)").unwrap();
    second.exec("CheckTitle(nil)").unwrap();
    second.state().borrow_mut().delve_entrance_title = Some("Independent Entrance".into());
    second.exec("CheckTitle('Independent Entrance')").unwrap();
    first.exec("CheckTitle(nil)").unwrap();
}

#[test]
fn b04_no_declared_title_arguments_are_read_and_picker_default_is_compatible() {
    let env = new_env();
    install_secrets(&env);
    env.exec(
        r#"
        assert((C_DelvesUI.GetDelveEntranceTitleString() or 'Delve') == 'Delve')
        local function probe()
            CheckTitle(nil, SecretNil, SecretFalse, {})
            assert(issecretvalue(SecretNil) and issecretvalue(SecretFalse))
            assert(debug.getstacktaint() == 'B04Probe')
        end
        debug.setobjecttaint(probe, 'B04Probe')
        probe()
        assert(issecure())
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().delve_entrance_title, None);
}
