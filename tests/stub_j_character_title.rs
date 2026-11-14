//! Retirement RED: original Retail CharacterFrame titles survive the first
//! real post-event workaround, without replacing original frames or methods.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]
use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, find_toc_file,
    load_startup_addon, register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn original_character_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc = find_toc_file(&ui.join("Blizzard_UIPanels_Game")).unwrap();
    assert!(toc.ends_with("Blizzard_UIPanels_Game_Mainline.toc"));
    let contents = std::fs::read_to_string(&toc).unwrap();
    assert!(contents.contains("## AllowLoad: game"));
    assert!(contents.contains("## AllowLoadGameType: mainline"));
    assert!(contents.contains("Mainline\\CharacterFrame.lua"));
    assert!(contents.contains("Mainline\\CharacterFrame.xml"));
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    assert!(startup.iter().any(|a| a.name == "Blizzard_UIPanels_Game"));
    assert!(startup.iter().any(|a| a.name == "Blizzard_TokenUI"));
    for addon in startup {
        if !env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded)
        {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env.exec(
        r#"
        -- Capture actual XML script and Lua method owners before simulator hooks.
        JOriginalCharacterOnShow = CharacterFrame:GetScript("OnShow")
        JOriginalCharacterRefreshDisplay = CharacterFrame.RefreshDisplay
        assert(type(JOriginalCharacterOnShow) == "function")
        assert(JOriginalCharacterRefreshDisplay == CharacterFrameMixin.RefreshDisplay)
        assert(debug.getinfo(JOriginalCharacterRefreshDisplay).source:find(
            "Blizzard_UIPanels_Game/Mainline/CharacterFrame.lua", 1, true))
    "#,
    )
    .unwrap();
    env.apply_post_load_workarounds();
    assert!(!env.state().borrow().post_event_workarounds_applied);
    env.exec(r#"
        for _, callback in ipairs({ToggleCharacter, CharacterFrameMixin.ShowSubFrame, CharacterFrameMixin.UpdateTitle}) do
            assert(debug.getinfo(callback).source:find('Blizzard_UIPanels_Game/Mainline/CharacterFrame.lua', 1, true))
        end
        assert(CharacterFrame.TitleContainer.TitleText and ReputationFrame and TokenFrame and PaperDollFrame)
        JTitleText = CharacterFrame.TitleContainer.TitleText
        JHeadIcon = CharacterHeadSlot.icon
        JHeadTexture = JHeadIcon:GetTexture()
    "#).unwrap();
    env
}

fn assert_title(env: &WowLuaEnv, tab: &str, expected: &str) {
    env.exec(&format!(
        r#"
        assert(CharacterFrame.activeSubframe == '{tab}', 'active=' .. tostring(CharacterFrame.activeSubframe))
        assert(CharacterFrame.TitleContainer.TitleText == JTitleText, 'title identity')
        assert(JTitleText:GetText() == {expected}, 'title=' .. tostring(JTitleText:GetText()))
    "#
    ))
    .unwrap();
}

fn exercise_title(tab: &str, expected: &str) {
    let env = original_character_env();
    env.exec(&format!("ToggleCharacter('{tab}', true)"))
        .unwrap();
    env.exec("assert(CharacterFrame:IsShown(), 'original ToggleCharacter must show the panel')")
        .unwrap();
    println!("J title before first post-event hook");
    assert_title(&env, tab, expected);
    env.exec("JHeadTexture = JHeadIcon:GetTexture()").unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    assert!(env.state().borrow().post_event_workarounds_applied);
    println!("J title after first post-event hook");
    env.exec("assert(CharacterHeadSlot.icon == JHeadIcon, 'slot identity'); assert(JHeadIcon:GetTexture() == JHeadTexture, 'slot texture')").unwrap();
    assert_title(&env, tab, expected);
    for (tab, title) in [
        ("PaperDollFrame", "UnitPVPName('player')"),
        ("ReputationFrame", "REPUTATION"),
        ("TokenFrame", "CURRENCY"),
    ] {
        env.exec(&format!("ToggleCharacter('{tab}', true)"))
            .unwrap();
        assert_title(&env, tab, title);
    }
    for error in &env.state().borrow().lua_error_records {
        println!("J title retained diagnostic: {}", error.message);
        assert!(
            !error.message.contains("CharacterFrame.lua")
                && !error.message.contains("PaperDollFrame.lua"),
            "{}",
            error.message
        );
    }
}

#[test]
fn original_reputation_title_survives_first_post_event_refresh() {
    exercise_title("ReputationFrame", "REPUTATION");
}

#[test]
fn original_currency_title_survives_first_post_event_refresh() {
    exercise_title("TokenFrame", "CURRENCY");
}

// Ownership retirement RED, distinct from the populated preservation test below.
fn assert_original_display_callbacks(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(CharacterFrame:GetScript("OnShow") == JOriginalCharacterOnShow,
            "simulator must not replace original CharacterFrame XML OnShow")
        assert(CharacterFrame.RefreshDisplay == JOriginalCharacterRefreshDisplay,
            "simulator must not replace original CharacterFrame RefreshDisplay")
        assert(rawget(_G, "__wow_character_frame_onshow_wrapper") == nil)
        assert(rawget(_G, "__wow_character_frame_refresh_display_wrapper") == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn original_character_display_callbacks_survive_postload_and_world_entry() {
    let env = original_character_env();
    // Current bridge fails here even where its extra refreshes happen to
    // preserve visible output. Do not mislabel this as a native behavior RED.
    assert_original_display_callbacks(&env);
    env.exec("ToggleCharacter('PaperDollFrame', true)").unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    assert!(env.state().borrow().post_event_workarounds_applied);
    assert_original_display_callbacks(&env);
}

fn assert_original_slot_presentation(env: &WowLuaEnv, equipped: bool) {
    env.exec(&format!(
        r#"
        assert(CharacterFrame:IsShown() and PaperDollFrame:IsShown())
        assert(CharacterFrame.activeSubframe == "PaperDollFrame")
        assert(CharacterFrame.TitleContainer.TitleText:GetText() == UnitPVPName("player"))
        local slot = CharacterHeadSlot
        assert(slot.icon == JHeadIcon, "original icon identity")
        assert(slot:IsEventRegistered("PLAYER_EQUIPMENT_CHANGED"),
            "original slot OnShow must register equipment refresh")
        local texture = GetInventoryItemTexture("player", slot:GetID())
        if {equipped} then
            assert(texture ~= nil, "existing equipped item must have a texture")
            assert(slot.icon:GetTexture() == texture, "original equipped slot texture")
        else
            assert(texture == nil, "host scenario removed the equipped item")
            assert(slot.backgroundTextureName ~= nil, "XML/OnLoad owns empty-slot background")
            assert(slot.icon:GetTexture() == slot.backgroundTextureName,
                "original empty-slot texture")
            assert(not slot.Cooldown:IsShown(), "empty slot clears cooldown")
        end
    "#
    ))
    .unwrap();
}

fn assert_no_character_owner_errors(env: &WowLuaEnv) {
    for error in &env.state().borrow().lua_error_records {
        println!("J slot retained diagnostic: {}", error.message);
        assert!(
            !error.message.contains("CharacterFrame.lua")
                && !error.message.contains("PaperDollFrame.lua"),
            "{}",
            error.message
        );
    }
}

/// PRESERVATION through original slot events and open/reopen lifecycle. The
/// existing modeled head item is moved out/back; no new item/native API is
/// fabricated, and no original callback is replaced or invoked by a fake sink.
#[test]
fn original_character_slot_refreshes_equipped_empty_and_reopened_states() {
    let env = original_character_env();
    let original_head = env
        .state()
        .borrow()
        .player
        .equipped_items
        .get(&1)
        .expect("SETUP: existing modeled head item required")
        .clone();
    env.exec(
        r#"
        assert(CharacterHeadSlot:GetID() == 1)
        for _, callback in ipairs({PaperDollItemSlotButton_Update,
            PaperDollItemSlotButton_OnShow, PaperDollItemSlotButton_OnEvent}) do
            assert(debug.getinfo(callback).source:find(
                "Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua", 1, true))
        end
        ToggleCharacter("PaperDollFrame", true)
    "#,
    )
    .unwrap();
    assert_original_slot_presentation(&env, true);
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    // The original world-entry lifecycle legitimately closes the panel.
    // Reopen through its original controller, never force frame visibility.
    env.exec("assert(not CharacterFrame:IsShown(), 'world-entry lifecycle closes panel'); ToggleCharacter('PaperDollFrame', true)")
        .unwrap();
    assert_original_slot_presentation(&env, true);

    // Actual registered original OnEvent refreshes the now-empty visible slot.
    env.state().borrow_mut().player.equipped_items.remove(&1);
    env.fire_event_with_args(
        "PLAYER_EQUIPMENT_CHANGED",
        &[rilua::Val::Num(1.0), rilua::Val::Bool(false)],
    )
    .unwrap();
    assert_original_slot_presentation(&env, false);
    env.exec("ToggleCharacter('PaperDollFrame'); assert(not CharacterFrame:IsShown())")
        .unwrap();

    // Change the same existing item while hidden; reopening must be sufficient
    // to refresh through original child OnShow, not a post-event replay.
    env.state()
        .borrow_mut()
        .player
        .equipped_items
        .insert(1, original_head.clone());
    env.exec("ToggleCharacter('PaperDollFrame', true)").unwrap();
    assert_original_slot_presentation(&env, true);
    env.exec("ToggleCharacter('ReputationFrame', true); assert(CharacterFrame.activeSubframe == 'ReputationFrame')").unwrap();
    env.state().borrow_mut().player.equipped_items.remove(&1);
    env.exec("ToggleCharacter('PaperDollFrame', true)").unwrap();
    assert_original_slot_presentation(&env, false);
    env.state()
        .borrow_mut()
        .player
        .equipped_items
        .insert(1, original_head);
    env.fire_event_with_args(
        "PLAYER_EQUIPMENT_CHANGED",
        &[rilua::Val::Num(1.0), rilua::Val::Bool(true)],
    )
    .unwrap();
    assert_original_slot_presentation(&env, true);
    assert_no_character_owner_errors(&env);
}
