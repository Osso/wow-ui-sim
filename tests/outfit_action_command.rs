#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_transmog_outfit_info::OutfitEntry;
use wow_ui_sim::lua_api::WowLuaEnv;

fn seed_catalog(env: &WowLuaEnv) {
    env.state().borrow_mut().transmog_outfit_catalog.entries = vec![
        OutfitEntry {
            outfit_id: 91,
            name: "Raid".into(),
            situation_categories: vec![],
            icon: 135771,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 1,
        },
        OutfitEntry {
            outfit_id: 305,
            name: "Travel".into(),
            situation_categories: vec![],
            icon: 132489,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 7,
        },
    ];
}

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    seed_catalog(&env);
    env
}

fn assert_active(env: &WowLuaEnv, expected: Option<i64>) {
    assert_eq!(env.state().borrow().active_transmog_outfit_id, expected);
    let active: f64 = env
        .eval("return C_TransmogOutfitInfo.GetActiveOutfitID()")
        .unwrap();
    assert_eq!(active, expected.unwrap_or(0) as f64);
}

#[test]
fn active_outfit_reads_host_snapshot_and_is_environment_local() {
    let first = seeded_env();
    let second = seeded_env();
    assert_active(&first, None);
    first.state().borrow_mut().active_transmog_outfit_id = Some(305);
    assert_active(&first, Some(305));
    assert_active(&second, None);
    first
        .exec("C_TransmogOutfitInfo.__activeOutfitID = 999; C_TransmogOutfitInfo.ClearOutfit()")
        .unwrap();
    assert_active(&first, None);
    assert_active(&second, None);
}

#[test]
fn outfit_change_resolves_sparse_index_and_toggles_only_same_selection() {
    let env = seeded_env();
    env.exec("C_TransmogOutfitInfo.ChangeToOutfit(1, false)")
        .unwrap();
    assert_active(&env, Some(91));
    env.exec("C_TransmogOutfitInfo.ChangeToOutfit(7, true)")
        .unwrap();
    assert_active(&env, Some(305));
    env.exec("C_TransmogOutfitInfo.ChangeToOutfit(7, false)")
        .unwrap();
    assert_active(&env, Some(305));
    env.exec("C_TransmogOutfitInfo.ChangeToOutfit(7, true)")
        .unwrap();
    assert_active(&env, None);
    env.exec("C_TransmogOutfitInfo.ChangeToOutfit(1, false)")
        .unwrap();
    env.exec("C_TransmogOutfitInfo.ClearOutfit()").unwrap();
    assert_active(&env, None);
    env.exec("C_TransmogOutfitInfo.ClearOutfit()").unwrap();
    assert_active(&env, None);
}

#[test]
fn invalid_outfit_index_does_not_fabricate_or_clear_selection() {
    let env = seeded_env();
    env.state().borrow_mut().active_transmog_outfit_id = Some(91);
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        for _, index in ipairs({91, 305, 999, -1, 1.5, math.huge, 0/0}) do
            assert(select('#', api.ChangeToOutfit(index, true)) == 0)
            assert(api.GetActiveOutfitID() == 91)
        end
        assert(not pcall(api.ChangeToOutfit, '1', false))
        assert(not pcall(api.ChangeToOutfit, 1, 1))
        assert(not pcall(api.ChangeToOutfit, 1))
    "#,
    )
    .unwrap();
    assert_active(&env, Some(91));
    let empty = WowLuaEnv::new().unwrap();
    empty
        .exec("C_TransmogOutfitInfo.ChangeToOutfit(1, false)")
        .unwrap();
    assert_active(&empty, None);
}

#[test]
fn outfit_change_authenticates_both_secret_arguments_before_mutation() {
    let env = seeded_env();
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        api.ChangeToOutfit(secretwrap(1), secretwrap(false))
        assert(api.GetActiveOutfitID() == 91)
        local function addonIndex()
            api.ChangeToOutfit(secretwrap(7), false)
        end
        local function addonToggle()
            api.ChangeToOutfit(7, secretwrap(false))
        end
        debug.setobjecttaint(addonIndex, 'OutfitProbe')
        debug.setobjecttaint(addonToggle, 'OutfitProbe')
        assert(not pcall(addonIndex))
        assert(not pcall(addonToggle))
        assert(api.GetActiveOutfitID() == 91)
        local function ordinary()
            assert(not issecure())
            api.ChangeToOutfit(7, false)
            assert(not issecure())
        end
        debug.setobjecttaint(ordinary, 'OutfitProbe')
        ordinary()
        assert(issecure())
    "#,
    )
    .unwrap();
    assert_active(&env, Some(305));
}

#[test]
fn conditional_outfit_macro_executes_selected_branch_not_ignored_command() {
    let env = seeded_env();
    env.state().borrow_mut().active_transmog_outfit_id = Some(305);
    env.exec(
        r#"
        assert(pcall(C_Macro.RunMacroText, '/outfit [@player,noexists] 1'))
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
        C_Macro.RunMacroText('/outfit [@player,exists] 1')
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91)
        C_Macro.RunMacroText('/outfit [@player,noexists] 1; [@player,exists] 7')
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
        C_Macro.RunMacroText('/outfit [combat] 1; [nocombat] !7')
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
    "#,
    )
    .unwrap();
    assert_active(&env, Some(305));
    env.state().borrow_mut().player.in_combat = true;
    env.exec("C_Macro.RunMacroText('/outfit [combat] 1; [nocombat] !7')")
        .unwrap();
    assert_active(&env, Some(91));
}

#[test]
fn outfit_macro_toggle_bang_clear_and_invalid_inputs_share_selection() {
    let env = seeded_env();
    env.exec("C_Macro.RunMacroText('/outfit 1')").unwrap();
    assert_active(&env, Some(91));
    env.exec("C_Macro.RunMacroText('/OUTFIT !1')").unwrap();
    assert_active(&env, Some(91));
    env.exec(
        r#"
        for _, text in ipairs({
            '/outfit 91', '/outfit 999', '/outfit nonsense',
            '/outfit [@player,exists 7', '/outfit [unmodeled] 7'
        }) do
            assert(pcall(C_Macro.RunMacroText, text))
            assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91)
        end
    "#,
    )
    .unwrap();
    assert_active(&env, Some(91));
    env.exec("C_Macro.RunMacroText('/outfit 1')").unwrap();
    assert_active(&env, None);
    env.exec(r#"C_Macro.RunMacroText('/outfit 7\n/outfit')"#)
        .unwrap();
    assert_active(&env, None);
}

#[test]
fn unchanged_vendor_secure_outfit_handler_changes_toggles_and_clears() {
    let env = seeded_env();
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(ui.join("Blizzard_FrameXML/SecureTemplates.lua"))
        .expect("profile-scoped cached vendor SecureTemplates.lua required");
    env.exec(&source)
        .expect("load complete unchanged vendor SecureTemplates.lua");
    env.exec(
        r#"
        local button = CreateFrame('Button', 'OutfitActionProbe', UIParent)
        button:SetAttribute('type', 'outfit')
        button:SetAttribute('outfit-index', 1)
        button:SetAttribute('action', 'change')
        assert(SecureActionButton_OnClick(button, 'LeftButton', false, false, true))
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91)
        button:SetAttribute('outfit-index', 7)
        button:SetAttribute('action', 'toggle')
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
        button:SetAttribute('action', 'change')
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 305)
        button:SetAttribute('action', nil)
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 0)
        button:SetAttribute('action', 'change')
        button:SetAttribute('outfit-index', 1)
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        button:SetAttribute('outfit-index', 999)
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91)
        button:SetAttribute('action', 'clear')
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 0)
    "#,
    )
    .unwrap();
    assert_active(&env, None);
    assert!(env.state().borrow().lua_errors.is_empty());
}
