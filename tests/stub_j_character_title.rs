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
