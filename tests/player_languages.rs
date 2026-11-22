#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::Path;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

const CHAT_LANGUAGE_MENU_OBSERVATION: &str = r#"
local function inspectLanguageMenu(race)
    A_Admin.SetPlayerRace(race)
    ChatFrame1:ConfigEventHandler("PLAYER_ENTERING_WORLD", false, false)
    ChatFrame1.editBox.languageID = -1
    ChatFrameMenuButton:ValidateSelectedLanguage()
    ChatFrameMenuButton:GenerateMenu()

    local languageMenu
    MenuUtil.TraverseMenu(ChatFrameMenuButton:GetMenuDescription(), function(description)
        if MenuUtil.GetElementText(description) == LANGUAGE then
            languageMenu = description
            return true
        end
    end)

    local languages = {}
    for _, description in languageMenu:EnumerateElementDescriptions() do
        table.insert(languages, MenuUtil.GetElementText(description))
    end
    return table.concat(languages, ","), ChatFrame1.defaultLanguage,
        ChatFrame1.editBox.language, ChatFrame1.alternativeDefaultLanguage
end

local allianceLanguages, allianceDefault, allianceShown, allianceAlternative = inspectLanguageMenu(0)
local hordeLanguages, hordeDefault, hordeShown, hordeAlternative = inspectLanguageMenu(7)
return allianceLanguages, allianceDefault, allianceShown,
    tostring(allianceAlternative or "nil"),
    hordeLanguages .. "|" .. hordeDefault .. "|" .. hordeShown .. "|" .. hordeAlternative
"#;

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();

    for (name, toc_path) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Game) {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn chat_language_menu_uses_the_player_race() {
    let env = load_full_game_ui();
    let observed: (String, String, String, String, String) = env
        .eval(CHAT_LANGUAGE_MENU_OBSERVATION)
        .expect("the Blizzard chat language menu should read the player languages");

    assert_eq!(observed.0, "Common");
    assert_eq!(observed.1, "Common");
    assert_eq!(observed.2, "Common");
    assert_eq!(observed.3, "nil");
    assert_eq!(observed.4, "Orcish,Troll|Orcish|Orcish|Troll");
}
