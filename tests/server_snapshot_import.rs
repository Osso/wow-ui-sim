use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};

const SERVER_SNAPSHOT_ADDON_LUA: &str =
    include_str!("../docs/addons/ServerSnapshot/ServerSnapshot.lua");

const CAPTURE_BAG_APIS: &str = r#"
    NUM_TOTAL_EQUIPPED_BAG_SLOTS = 5
    BACKPACK_CONTAINER = 0
    C_Container = {
        GetContainerNumSlots = function(bag)
            return ({[0]=20, [1]=2, [2]=0, [3]=0, [4]=0, [5]=36})[bag]
        end,
        GetContainerNumFreeSlots = function(bag) return 0, bag == 5 and 1024 or 0 end,
        GetContainerItemInfo = function(bag, slot)
            if bag == 0 and slot == 1 then return {itemID=6948, stackCount=1, hyperlink='item:6948'} end
            if bag == 5 and slot == 30 then
                return {itemID=190315, stackCount=17,
                    hyperlink='|Hitem:190315:::::::::::::1:9999|h[Captured ore]|h'}
            end
        end,
        ContainerIDToInventoryID = function(bag) return 19+bag end,
        GetBagName = function(bag) return bag == 0 and 'Captured Backpack' or 'Captured Bag '..bag end,
    }
    GetInventoryItemID = function(_, slot) if slot == 20 then return 200000 end end
    GetInventoryItemLink = function(_, slot) if slot == 20 then return 'item:200000' end end
"#;

fn write_producer_bag_snapshot(root: &std::path::Path) {
    let directory = root.join("Account/CapturedAccount/SavedVariables");
    let source = WowLuaEnv::new().unwrap();
    let mut writer = SavedVariablesManager::with_storage_dir(&directory);
    writer
        .init_for_addon(
            source.rilua_mut().state_mut(),
            "ServerSnapshot",
            &["ServerSnapshotDB".into()],
            &[],
        )
        .unwrap();
    source.exec(CAPTURE_BAG_APIS).unwrap();
    source.exec(SERVER_SNAPSHOT_ADDON_LUA).unwrap();
    source.fire_event("BAG_UPDATE_DELAYED").unwrap();
    writer
        .save_addon(source.rilua_mut().state_mut(), "ServerSnapshot")
        .unwrap();
}

fn import_producer_bag_snapshot(root: &std::path::Path, env: &WowLuaEnv) {
    let mut reader = SavedVariablesManager::with_storage_dir(root.join("sim-local"));
    reader.set_wtf_config(WtfConfig::new(
        root,
        "CapturedAccount",
        "Realm",
        "Character",
    ));
    wow_ui_sim::server_snapshot_import::load_from_saved_variables(env, &mut reader).unwrap();
}

#[test]
fn server_snapshot_bags_roundtrip_actual_producer_saved_variables_and_queries() {
    let root = tempfile::tempdir().unwrap();
    write_producer_bag_snapshot(root.path());
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.AddBagItem(1, 1, 6948, 9)
        A_Admin.AddBagItem(-1, 1, 6948, 2)
        HeaderItemBeforeBagImport = GetInventoryItemID('player', 1)
        BagImportEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('BAG_UPDATE')
        listener:RegisterEvent('BAG_UPDATE_DELAYED')
        listener:SetScript('OnEvent', function(_, event, bag)
            assert(C_Container.GetContainerNumSlots(0) == 20)
            assert(C_Container.GetContainerNumSlots(5) == 36)
            assert(C_Container.GetContainerItemInfo(5, 30).stackCount == 17)
            table.insert(BagImportEvents, {event, bag})
        end)
    "#,
    )
    .unwrap();
    import_producer_bag_snapshot(root.path(), &env);
    env.exec(r#"
        assert(C_Container.GetContainerNumSlots(0) == 20)
        assert(C_Container.GetContainerNumSlots(1) == 2)
        assert(C_Container.GetContainerNumSlots(2) == 0)
        assert(C_Container.GetContainerNumSlots(5) == 36)
        assert(C_Container.GetBagName(0) == 'Captured Backpack')
        assert(C_Container.GetBagName(5) == 'Captured Bag 5')
        assert(C_Container.ContainerIDToInventoryID(1) == 20)
        assert(GetInventoryItemID('player', 20) == 200000)
        assert(GetInventoryItemLink('player', 20) == 'item:200000')
        assert(GetInventoryItemID('player', 1) == HeaderItemBeforeBagImport)
        assert(C_Container.GetContainerItemInfo(1, 1) == nil)
        assert(C_Container.GetContainerItemInfo(0, 1).itemID == 6948)
        assert(C_Container.GetContainerItemInfo(5, 30).itemID == 190315)
        assert(C_Container.GetContainerItemInfo(5, 30).stackCount == 17)
        local expectedLink = '|Hitem:190315:::::::::::::1:9999|h[Captured ore]|h'
        assert(C_Container.GetContainerItemLink(5, 30) == expectedLink)
        assert(C_Container.GetContainerItemInfo(5, 30).hyperlink == expectedLink)
        assert(C_Container.GetContainerItemInfo(5, 31) == nil)
        local free, family = C_Container.GetContainerNumFreeSlots(5)
        assert(free == 35 and family == 1024)
        assert(#C_Container.GetContainerFreeSlots(5) == 35)
        assert(C_Container.GetContainerItemInfo(-1, 1).stackCount == 2)
        assert(#BagImportEvents == 7)
        for index=1,6 do assert(BagImportEvents[index][1]=='BAG_UPDATE' and BagImportEvents[index][2]==index-1) end
        assert(BagImportEvents[7][1]=='BAG_UPDATE_DELAYED')
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn server_snapshot_empty_bags_clear_stale_items_but_missing_domain_preserves_state() {
    let root = tempfile::tempdir().unwrap();
    write_producer_bag_snapshot(root.path());
    let env = WowLuaEnv::new().unwrap();
    import_producer_bag_snapshot(root.path(), &env);
    env.exec(
        r#"
        SelectedBagSnapshot = ServerSnapshotDB.characters[ServerSnapshotDB.lastCharacterKey]
        CapturedBagDomain = SelectedBagSnapshot.bags
        SelectedBagSnapshot.bags = nil
    "#,
    )
    .unwrap();
    wow_ui_sim::server_snapshot_import::apply_loaded_snapshot(&env).unwrap();
    env.exec(
        r#"
        assert(C_Container.GetContainerItemInfo(5, 30).stackCount == 17)
        SelectedBagSnapshot.bags = CapturedBagDomain
        CapturedBagDomain.containers[0].items = {}
        CapturedBagDomain.containers[5].items = {}
        CapturedBagDomain.containers[1].numSlots = 0
        CapturedBagDomain.containers[1].itemID = nil
        CapturedBagDomain.containers[1].hyperlink = nil
    "#,
    )
    .unwrap();
    wow_ui_sim::server_snapshot_import::apply_loaded_snapshot(&env).unwrap();
    env.exec(
        r#"
        assert(C_Container.GetContainerNumSlots(0) == 20)
        assert(C_Container.GetContainerItemInfo(0, 1) == nil)
        assert(C_Container.GetContainerItemInfo(5, 30) == nil)
        assert(C_Container.GetContainerNumFreeSlots(5) == 36)
        assert(C_Container.GetContainerNumSlots(1) == 0)
        assert(GetInventoryItemID('player', 20) == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn server_snapshot_rejects_invalid_bags_before_mutating_inventory_or_actions() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.AddBagItem(0, 1, 6948, 3)
        A_Admin.SetActionSlot(1, 19750)
        ServerSnapshotDB = {lastCharacterKey='Bad', characters={Bad={
            actionBars={slots={[1]={type='spell',id=4987}}},
            bags={maxBagID=0,containers={[0]={numSlots=2,family=0,
                items={[3]={itemID=190315,stackCount=1}}}}},
        }}}
    "#,
    )
    .unwrap();
    let error = wow_ui_sim::server_snapshot_import::apply_loaded_snapshot(&env).unwrap_err();
    assert!(error.to_string().contains("bags.containers[0].items key"));
    env.exec(
        r#"
        assert(C_Container.GetContainerNumSlots(0) == 16)
        assert(C_Container.GetContainerItemInfo(0, 1).stackCount == 3)
        local kind, id = GetActionInfo(1)
        assert(kind == 'spell' and id == 19750)
        ServerSnapshotDB.characters.Bad.actionBars = nil
        ServerSnapshotDB.characters.Bad.bags.containers[0].items = {}
    "#,
    )
    .unwrap();
    assert_eq!(
        wow_ui_sim::server_snapshot_import::apply_loaded_snapshot(&env).unwrap(),
        0
    );
    env.exec("assert(C_Container.GetContainerNumSlots(0) == 2); assert(C_Container.GetContainerNumSlots(1) == 0)").unwrap();
}

#[test]
fn server_snapshot_action_bars_seed_get_action_info() {
    let env = WowLuaEnv::new().expect("Lua env");

    env.exec(
        r#"
        ServerSnapshotDB = {
            lastCharacterKey = "SimRealm/SimPlayer",
            characters = {
                ["SimRealm/SimPlayer"] = {
                    actionBars = {
                        slots = {
                            [1] = { type = "spell", id = 19750, spellID = 19750 },
                            [2] = { type = "macro", id = 1 },
                            [3] = { empty = true },
                            [13] = { type = "spell", id = 4987 },
                        },
                    },
                },
            },
        }
        "#,
    )
    .expect("seed ServerSnapshotDB");

    let imported = wow_ui_sim::server_snapshot_import::apply_loaded_snapshot(&env)
        .expect("import snapshot action bars");
    assert_eq!(imported, 2);

    let (slot1_type, slot1_id, slot13_type, slot13_id, slot2_has, slot3_has): (
        String,
        i64,
        String,
        i64,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local t1, id1 = GetActionInfo(1)
            local t13, id13 = GetActionInfo(13)
            return t1, id1, t13, id13, HasAction(2), HasAction(3)
            "#,
        )
        .expect("read action bars");

    assert_eq!(slot1_type, "spell");
    assert_eq!(slot1_id, 19750);
    assert_eq!(slot13_type, "spell");
    assert_eq!(slot13_id, 4987);
    assert!(!slot2_has, "non-spell snapshot entries are ignored for now");
    assert!(!slot3_has, "empty snapshot entries stay empty");
}

#[test]
fn server_snapshot_uses_latest_character_when_last_key_missing() {
    let env = WowLuaEnv::new().expect("Lua env");

    env.exec(
        r#"
        ServerSnapshotDB = {
            characters = {
                Older = {
                    capturedAt = 10,
                    actionBars = {
                        slots = {
                            [1] = { type = "spell", id = 111 },
                        },
                    },
                },
                Newer = {
                    capturedAt = 20,
                    actionBars = {
                        slots = {
                            [1] = { type = "spell", id = 222 },
                        },
                    },
                },
            },
        }
        "#,
    )
    .expect("seed ServerSnapshotDB");

    let imported = wow_ui_sim::server_snapshot_import::apply_loaded_snapshot(&env)
        .expect("import latest snapshot");
    assert_eq!(imported, 1);

    let spell_id: i64 = env
        .eval(
            r#"
            local _type, id = GetActionInfo(1)
            return id
            "#,
        )
        .expect("read imported spell id");
    assert_eq!(spell_id, 222);
}

#[test]
fn server_snapshot_loads_from_wtf_saved_variables_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    let saved_vars_dir = temp.path().join("Account/AccountName/SavedVariables");
    std::fs::create_dir_all(&saved_vars_dir).expect("create saved vars dir");
    std::fs::write(
        saved_vars_dir.join("ServerSnapshot.lua"),
        r#"
        ServerSnapshotDB = {
            lastCharacterKey = "RealmName/CharacterName",
            characters = {
                ["RealmName/CharacterName"] = {
                    capturedAt = 123,
                    actionBars = {
                        slots = {
                            [1] = { type = "spell", id = 19750, spellID = 19750 },
                        },
                    },
                },
            },
        }
        "#,
    )
    .expect("write ServerSnapshot saved vars");

    let env = WowLuaEnv::new().expect("Lua env");
    let mut saved_vars = SavedVariablesManager::with_storage_dir(temp.path().join("local"));
    saved_vars.set_wtf_config(WtfConfig::new(
        temp.path(),
        "AccountName",
        "RealmName",
        "CharacterName",
    ));

    let imported =
        wow_ui_sim::server_snapshot_import::load_from_saved_variables(&env, &mut saved_vars)
            .expect("load ServerSnapshot from WTF");
    assert_eq!(imported, 1);

    let (action_type, spell_id): (String, i64) = env
        .eval(
            r#"
            local actionType, id = GetActionInfo(1)
            return actionType, id
            "#,
        )
        .expect("read imported action");
    assert_eq!(action_type, "spell");
    assert_eq!(spell_id, 19750);
}

#[test]
fn server_snapshot_addon_derives_active_edit_mode_layout_name_from_c_api() {
    let env = WowLuaEnv::new().expect("Lua env");

    env.exec(
        r#"
        ServerSnapshotDB = nil
        EditModeManagerFrame = nil
        C_EditMode = {
            GetLayouts = function()
                return {
                    activeLayout = 3,
                    layouts = {
                        { layoutName = "Modern" },
                        { layoutName = "Classic" },
                        { layoutName = "Ultrawide" },
                    },
                }
            end,
        }
        "#,
    )
    .expect("seed EditMode API");

    env.exec(SERVER_SNAPSHOT_ADDON_LUA)
        .expect("load ServerSnapshot addon");

    let active_layout_name: String = env
        .eval(
            r#"
            local snapshot = ServerSnapshot:Snapshot("test")
            return snapshot.editMode.activeLayoutName or ""
            "#,
        )
        .expect("snapshot active layout");

    assert_eq!(active_layout_name, "Ultrawide");
}

#[test]
fn server_snapshot_addon_loads_edit_mode_before_capturing_layout_name() {
    let env = WowLuaEnv::new().expect("Lua env");

    env.exec(
        r#"
        ServerSnapshotDB = nil
        EditModeManagerFrame = nil
        C_EditMode = nil
        ServerSnapshotTestLoadCalls = 0
        UIParentLoadAddOn = function(name)
            if name == "Blizzard_EditMode" then
                ServerSnapshotTestLoadCalls = ServerSnapshotTestLoadCalls + 1
                EditModeManagerFrame = {
                    GetActiveLayoutInfo = function()
                        return { layoutName = "Ultrawide" }
                    end,
                }
                return true
            end
            return false
        end
        "#,
    )
    .expect("seed lazy EditMode addon");

    env.exec(SERVER_SNAPSHOT_ADDON_LUA)
        .expect("load ServerSnapshot addon");

    let (load_calls, active_layout_name): (i64, String) = env
        .eval(
            r#"
            local snapshot = ServerSnapshot:Snapshot("test")
            local editMode = snapshot.editMode or {}
            return ServerSnapshotTestLoadCalls, editMode.activeLayoutName or ""
            "#,
        )
        .expect("snapshot active layout");

    assert!(load_calls >= 1);
    assert_eq!(active_layout_name, "Ultrawide");
}
